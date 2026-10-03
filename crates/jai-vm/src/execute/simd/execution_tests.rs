use super::*;
use jai_ir::{SimdBuilder, SimdFeatures, SimdWidth};
use jai_types::{CastMode, Integer, IntegerType, ScalarType, TypeId, TypeRegistry, TypeView};

fn pointer32_target() -> crate::ByteTarget {
    let base = jai_types::LayoutPolicy::lp64();
    crate::ByteTarget {
        policy: jai_types::LayoutPolicy::new(
            jai_types::ScalarLayout::new(4, 4),
            [
                IntegerType::U8,
                IntegerType::U16,
                IntegerType::U32,
                IntegerType::U64,
            ]
            .map(|ty| base.integer(ty)),
            [jai_types::FloatType::F32, jai_types::FloatType::F64].map(|ty| base.float(ty)),
            base.boolean(),
        )
        .unwrap(),
        endian: Endian::Little,
    }
}

struct Provider {
    types: TypeRegistry,
    globals: Vec<Global>,
    byte_pointer: TypeId,
    array_pointer: TypeId,
}
impl ProcedureProvider for Provider {
    fn types(&self) -> &dyn TypeView {
        &self.types
    }
    fn globals(&self) -> &[Global] {
        &self.globals
    }
    fn procedure(&self, _: ProcedureId) -> ProcedureAvailability<'_> {
        ProcedureAvailability::Missing
    }
}
impl Provider {
    fn new(source: &[u8]) -> Self {
        Self::with_length(source, 64)
    }

    fn with_length(source: &[u8], length: u64) -> Self {
        let mut types = TypeRegistry::new();
        let byte = types.scalar(ScalarType::Int(IntegerType::U8));
        let array = types.fixed_array(byte, length).unwrap();
        let byte_pointer = types.pointer(byte).unwrap();
        let array_pointer = types.pointer(array).unwrap();
        let globals = [source.to_vec(), vec![0; length as usize]]
            .into_iter()
            .enumerate()
            .map(|(index, mut bytes)| {
                bytes.resize(length as usize, 0);
                Global::new_typed(
                    index,
                    ConstantValue {
                        ty: array,
                        kind: ConstantKind::Array(
                            bytes
                                .into_iter()
                                .map(|value| ConstantValue {
                                    ty: byte,
                                    kind: ConstantKind::Int(Integer::wrapping(
                                        IntegerType::U8,
                                        i128::from(value),
                                    )),
                                })
                                .collect(),
                        ),
                    },
                    &types,
                )
                .unwrap()
            })
            .collect();
        Self {
            types,
            globals,
            byte_pointer,
            array_pointer,
        }
    }

    fn address(&self, global: usize, offset: i128) -> ValueExpr {
        ValueExpr::PointerOffset {
            pointer: Box::new(ValueExpr::PointerCast {
                value: Box::new(ValueExpr::AddressOf {
                    place: self.globals[global].place(),
                    ty: self.array_pointer,
                }),
                ty: self.byte_pointer,
                mode: CastMode::Checked,
            }),
            offset: IntExpr::constant(Integer::wrapping(IntegerType::S64, offset)),
            subtract: false,
            ty: self.byte_pointer,
        }
    }

    fn block(
        &self,
        width: SimdWidth,
        interpretation: SimdInterpretation,
        in_place: bool,
    ) -> SimdBlock {
        let mut builder = SimdBuilder::new(SimdFeatures {
            avx: width == SimdWidth::Y256,
            avx2: interpretation == SimdInterpretation::U8,
        })
        .unwrap();
        let left = builder.register(width).unwrap();
        let right = builder.register(width).unwrap();
        let output = if in_place {
            left
        } else {
            builder.register(width).unwrap()
        };
        for (destination, offset) in [(left, 0), (right, width.bytes() as i128)] {
            builder
                .instruction(
                    SimdInstruction::Load {
                        destination,
                        interpretation,
                        address: self.address(0, offset),
                    },
                    &self.types,
                )
                .unwrap();
        }
        builder
            .instruction(
                SimdInstruction::Add {
                    destination: output,
                    left,
                    right,
                    interpretation,
                },
                &self.types,
            )
            .unwrap();
        builder
            .instruction(
                SimdInstruction::Store {
                    source: output,
                    interpretation,
                    address: self.address(1, 1),
                },
                &self.types,
            )
            .unwrap();
        builder.finish()
    }
}

#[test]
fn checked_block_executes_both_float_widths_and_in_place_or_distinct_registers() {
    for width in [SimdWidth::X128, SimdWidth::Y256] {
        let lanes = width.bytes() / 4;
        let mut source = Vec::new();
        for values in [vec![16_777_216.0_f32; lanes], vec![1.0_f32; lanes]] {
            for value in values {
                source.extend_from_slice(&value.to_bits().to_le_bytes());
            }
        }
        let provider = Provider::new(&source);
        for in_place in [false, true] {
            let block = provider.block(width, SimdInterpretation::F32, in_place);
            let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
            assert!(matches!(vm.simd(&block, 0), Ok(Control::Next)));
            let pointer = vm
                .value(&provider.address(1, 1), 0)
                .ok()
                .unwrap()
                .pointer()
                .unwrap()
                .clone();
            assert_eq!(
                vm.memory
                    .simd_read_bytes(&provider.types, &pointer, width.bytes())
                    .unwrap(),
                source[..width.bytes()]
            );
        }
    }
}

#[test]
fn checked_block_wraps_all_byte_lanes_and_charges_each_instruction() {
    let provider = Provider::new(&[255; 64]);
    let block = provider.block(SimdWidth::X128, SimdInterpretation::U8, true);
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    assert!(matches!(vm.simd(&block, 0), Ok(Control::Next)));
    let pointer = vm
        .value(&provider.address(1, 1), 0)
        .ok()
        .unwrap()
        .pointer()
        .unwrap()
        .clone();
    assert_eq!(
        vm.memory
            .simd_read_bytes(&provider.types, &pointer, 16)
            .unwrap(),
        vec![254; 16]
    );
    assert!(vm.statistics.steps >= block.instructions().len() as u64);
    let mut vm = Vm::new(
        &provider,
        crate::NoEffects,
        Limits {
            fuel: 0,
            ..Limits::default()
        },
    )
    .unwrap();
    assert!(matches!(
        vm.simd(&block, 0),
        Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
    ));
}

#[test]
fn checked_block_rejects_register_budget_and_non_x64_storage_before_effects() {
    let provider = Provider::new(&[0; 64]);
    let mut builder = SimdBuilder::new(SimdFeatures {
        avx: true,
        avx2: false,
    })
    .unwrap();
    for _ in 0..6 {
        builder.register(SimdWidth::Y256).unwrap();
    }
    let block = builder.finish();
    let mut vm = Vm::new(
        &provider,
        crate::NoEffects,
        Limits {
            value_cells: 191,
            ..Limits::default()
        },
    )
    .unwrap();
    assert!(matches!(
        vm.simd(&block, 0),
        Err(Halt::Failed(Error::Limit(LimitKind::ValueCells)))
    ));
    assert_eq!(vm.memory.allocation_count(), 0);
    let block = provider.block(SimdWidth::Y256, SimdInterpretation::F32, false);
    for target in [
        crate::ByteTarget {
            endian: Endian::Big,
            ..Default::default()
        },
        pointer32_target(),
    ] {
        let mut vm =
            Vm::new_with_target(&provider, crate::NoEffects, Limits::default(), target).unwrap();
        assert!(matches!(
            vm.simd(&block, 0),
            Err(Halt::Failed(Error::InvalidIr(_)))
        ));
        assert_eq!(vm.memory.allocation_count(), 0);
    }
}

#[test]
fn every_register_instruction_consumes_fuel_and_addresses_allocate_once() {
    let provider = Provider::new(&[0; 64]);
    let mut builder = SimdBuilder::new(SimdFeatures::default()).unwrap();
    let register = builder.register(SimdWidth::X128).unwrap();
    builder
        .instruction(
            SimdInstruction::Load {
                destination: register,
                interpretation: SimdInterpretation::U8,
                address: provider.address(0, 0),
            },
            &provider.types,
        )
        .unwrap();
    let load_only = builder.finish();
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    assert!(matches!(vm.simd(&load_only, 0), Ok(Control::Next)));
    let load_cost = vm.statistics.steps;
    let mut builder = SimdBuilder::new(SimdFeatures::default()).unwrap();
    let register = builder.register(SimdWidth::X128).unwrap();
    builder
        .instruction(
            SimdInstruction::Load {
                destination: register,
                interpretation: SimdInterpretation::U8,
                address: provider.address(0, 0),
            },
            &provider.types,
        )
        .unwrap();
    builder
        .instruction(
            SimdInstruction::Add {
                destination: register,
                left: register,
                right: register,
                interpretation: SimdInterpretation::U8,
            },
            &provider.types,
        )
        .unwrap();
    let load_and_add = builder.finish();
    let mut vm = Vm::new(
        &provider,
        crate::NoEffects,
        Limits {
            fuel: load_cost,
            ..Limits::default()
        },
    )
    .unwrap();
    assert!(matches!(
        vm.simd(&load_and_add, 0),
        Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
    ));

    let address = ValueExpr::AddressOfValue {
        value: Box::new(ValueExpr::Zero(provider.globals[0].ty())),
        ty: provider.array_pointer,
    };
    let mut builder = SimdBuilder::new(SimdFeatures::default()).unwrap();
    let register = builder.register(SimdWidth::X128).unwrap();
    builder
        .instruction(
            SimdInstruction::Load {
                destination: register,
                interpretation: SimdInterpretation::U8,
                address: address.clone(),
            },
            &provider.types,
        )
        .unwrap();
    builder
        .instruction(
            SimdInstruction::Store {
                source: register,
                interpretation: SimdInterpretation::U8,
                address,
            },
            &provider.types,
        )
        .unwrap();
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    assert!(matches!(vm.simd(&builder.finish(), 0), Ok(Control::Next)));
    assert_eq!(vm.memory.allocation_count(), 2);
    assert_eq!(vm.root_temporaries.len(), 2);
}

#[test]
fn small_simd_load_from_large_root_exhausts_fuel_before_materializing_its_image() {
    let provider = Provider::with_length(&[7; 16], 32_768);
    let mut builder = SimdBuilder::new(SimdFeatures::default()).unwrap();
    let destination = builder.register(SimdWidth::X128).unwrap();
    builder
        .instruction(
            SimdInstruction::Load {
                destination,
                interpretation: SimdInterpretation::U8,
                address: provider.address(0, 0),
            },
            &provider.types,
        )
        .unwrap();
    let block = builder.finish();
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    // The initialized aggregate exists, but has not acquired its byte image.
    assert!(vm.place(provider.globals[0].place(), 0).is_ok());
    vm.prepare_layout(provider.types.scalar(ScalarType::Int(IntegerType::U8)))
        .unwrap();
    let cells = vm.memory.value_cells();
    vm.statistics.steps = 0;
    vm.limits.fuel = 16;
    assert!(matches!(
        vm.simd(&block, 0),
        Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
    ));
    assert_eq!(vm.memory.value_cells(), cells);
    assert_eq!(vm.memory.allocation_count(), 1);
    assert!(vm.statistics.steps < 16);
}

#[test]
fn canonical_debug_trap_charges_fuel_and_returns_a_virtual_runtime_trap() {
    let provider = Provider::new(&[0; 64]);
    for instruction in [SimdInstruction::DebugTrap, SimdInstruction::Arm64DebugTrap] {
        let mut builder = SimdBuilder::new(SimdFeatures::default()).unwrap();
        builder.instruction(instruction, &provider.types).unwrap();
        let block = builder.finish();
        let mut vm = Vm::new(
            &provider,
            crate::NoEffects,
            Limits {
                fuel: 1,
                ..Limits::default()
            },
        )
        .unwrap();
        assert!(matches!(
            vm.simd(&block, 0),
            Err(Halt::Failed(Error::RuntimeTrap))
        ));
        assert_eq!(vm.statistics.steps, 1);
        assert_eq!(vm.memory.allocation_count(), 0);
        let mut vm = Vm::new(
            &provider,
            crate::NoEffects,
            Limits {
                fuel: 0,
                ..Limits::default()
            },
        )
        .unwrap();
        assert!(matches!(
            vm.simd(&block, 0),
            Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
        ));
        assert_eq!(vm.statistics.steps, 0);
        for target in [
            crate::ByteTarget {
                endian: Endian::Big,
                ..Default::default()
            },
            pointer32_target(),
        ] {
            let mut vm =
                Vm::new_with_target(&provider, crate::NoEffects, Limits::default(), target)
                    .unwrap();
            assert!(matches!(
                vm.simd(&block, 0),
                Err(Halt::Failed(Error::RuntimeTrap))
            ));
            assert_eq!(vm.statistics.steps, 1);
        }
    }
}
