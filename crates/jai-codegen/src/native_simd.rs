//! Native vector operations preserve the checked x86 ISA contract.
use inkwell::{
    IntPredicate,
    builder::{Builder, BuilderError},
    context::Context,
    targets::TargetMachine,
    types::VectorType,
    values::{BasicValue, IntValue, PointerValue, VectorValue},
};
use jai_ir::{SimdBlock, SimdError, SimdFeatures, SimdInterpretation, SimdRegisterId, SimdWidth};
use std::fmt;

#[derive(Debug)]
pub enum Error {
    UnsupportedArchitecture(String),
    Arm64TrapTarget(String),
    MissingFeature(&'static str),
    Register(SimdError),
    Build(BuilderError),
    Alignment,
    UninitializedRegister,
}
impl From<SimdError> for Error {
    fn from(error: SimdError) -> Self {
        Self::Register(error)
    }
}
impl From<BuilderError> for Error {
    fn from(error: BuilderError) -> Self {
        Self::Build(error)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedArchitecture(triple) => {
                write!(f, "x86 SIMD assembly is unsupported for target {triple}")
            }
            Self::Arm64TrapTarget(triple) => {
                write!(f, "ARM64 BRK #1 is unsupported for target {triple}")
            }
            Self::MissingFeature(feature) => write!(
                f,
                "x86 SIMD assembly requires enabled target feature {feature}"
            ),
            Self::Register(error) => write!(f, "invalid SIMD registers: {error}"),
            Self::Build(error) => write!(f, "SIMD instruction construction failed: {error}"),
            Self::Alignment => f.write_str("SIMD memory alignment construction failed"),
            Self::UninitializedRegister => f.write_str("SIMD register has no emitted value"),
        }
    }
}
impl std::error::Error for Error {}

/// Explicit feature overrides are applied in order. A selected host CPU may
/// inherit its actual LLVM-reported features; named cross CPUs require explicit
/// enables because the LLVM C API does not expose their resolved feature tables.
pub fn check_architecture(triple: &str) -> Result<(), Error> {
    if !triple.starts_with("x86_64-") {
        return Err(Error::UnsupportedArchitecture(triple.to_owned()));
    }
    Ok(())
}
pub fn check_arm64_architecture(triple: &str) -> Result<(), Error> {
    if !triple.starts_with("aarch64-") && !triple.starts_with("arm64-") {
        return Err(Error::Arm64TrapTarget(triple.to_owned()));
    }
    Ok(())
}

pub fn arm64_debug_trap<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
) -> Result<(), Error> {
    let signature = context.void_type().fn_type(&[], false);
    let instruction = context.create_inline_asm(
        signature,
        "brk #1".to_owned(),
        "~{memory}".to_owned(),
        true,
        false,
        None,
        false,
    );
    builder.build_indirect_call(signature, instruction, &[], "")?;
    Ok(())
}

pub fn check_target(
    triple: &str,
    cpu: &str,
    features: &str,
    required: SimdFeatures,
) -> Result<(), Error> {
    check_architecture(triple)?;
    let mut avx = false;
    let mut avx2 = false;
    let mut sse2 = true; // Guaranteed by the x86-64 baseline unless explicitly disabled.
    let host = TargetMachine::get_default_triple();
    if host.as_str().to_string_lossy() == triple
        && TargetMachine::get_host_cpu_name().to_string() == cpu
    {
        apply_features(
            &TargetMachine::get_host_cpu_features().to_string(),
            &mut sse2,
            &mut avx,
            &mut avx2,
        );
    }
    apply_features(features, &mut sse2, &mut avx, &mut avx2);
    if !sse2 {
        return Err(Error::MissingFeature("sse2"));
    }
    if required.requires_avx() && !avx {
        return Err(Error::MissingFeature("avx"));
    }
    if required.avx2 && !avx2 {
        return Err(Error::MissingFeature("avx2"));
    }
    Ok(())
}
fn apply_features(features: &str, sse2: &mut bool, avx: &mut bool, avx2: &mut bool) {
    for feature in features.split(',') {
        match feature {
            "+sse2" => *sse2 = true,
            "-sse2" | "-sse" => {
                *sse2 = false;
                *avx = false;
                *avx2 = false;
            }
            "+avx" => {
                *sse2 = true;
                *avx = true;
            }
            "-avx" => {
                *avx = false;
                *avx2 = false;
            }
            "+avx2" => {
                *sse2 = true;
                *avx = true;
                *avx2 = true;
            }
            "-avx2" => *avx2 = false,
            _ => {}
        }
    }
}
fn vector_type(
    context: &Context,
    width: SimdWidth,
    interpretation: SimdInterpretation,
) -> VectorType<'_> {
    let count = interpretation.lanes(width) as u32;
    match interpretation {
        SimdInterpretation::F32 => context.f32_type().vec_type(count),
        SimdInterpretation::U8 => context.i8_type().vec_type(count),
    }
}

pub(crate) struct Registers<'ctx> {
    values: Vec<Option<VectorValue<'ctx>>>,
}
impl<'ctx> Registers<'ctx> {
    pub(crate) fn new(block: &SimdBlock) -> Self {
        Self {
            values: vec![None; block.registers().len()],
        }
    }
    fn read(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        block: &SimdBlock,
        register: SimdRegisterId,
        interpretation: SimdInterpretation,
    ) -> Result<VectorValue<'ctx>, Error> {
        let ty = vector_type(context, block.width(register)?, interpretation);
        let value = self
            .values
            .get(register.index())
            .and_then(|value| *value)
            .ok_or(Error::UninitializedRegister)?;
        Ok(if value.get_type() == ty {
            value
        } else {
            builder
                .build_bit_cast(value, ty, "simd.reinterpret")?
                .into_vector_value()
        })
    }
    pub(crate) fn load(
        &mut self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        block: &SimdBlock,
        destination: SimdRegisterId,
        interpretation: SimdInterpretation,
        pointer: PointerValue<'ctx>,
    ) -> Result<(), Error> {
        let ty = vector_type(context, block.width(destination)?, interpretation);
        let value = builder
            .build_load(ty, pointer, "simd.load")?
            .into_vector_value();
        value
            .as_instruction_value()
            .ok_or(Error::Alignment)?
            .set_alignment(1)
            .map_err(|_| Error::Alignment)?;
        self.values[destination.index()] = Some(value);
        Ok(())
    }
    pub(crate) fn store(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        block: &SimdBlock,
        source: SimdRegisterId,
        interpretation: SimdInterpretation,
        pointer: PointerValue<'ctx>,
    ) -> Result<(), Error> {
        let value = self.read(context, builder, block, source, interpretation)?;
        builder
            .build_store(pointer, value)?
            .set_alignment(1)
            .map_err(|_| Error::Alignment)?;
        Ok(())
    }
    pub(crate) fn add(
        &mut self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        block: &SimdBlock,
        registers: (SimdRegisterId, SimdRegisterId, SimdRegisterId),
        interpretation: SimdInterpretation,
    ) -> Result<(), Error> {
        let (destination, left, right) = registers;
        let width = block.width(destination)?;
        if block.width(left)? != width || block.width(right)? != width {
            return Err(SimdError::WidthMismatch.into());
        }
        // Resolve both operands before replacing a destructive destination.
        let left = self.read(context, builder, block, left, interpretation)?;
        let right = self.read(context, builder, block, right, interpretation)?;
        self.values[destination.index()] = Some(match interpretation {
            SimdInterpretation::F32 => builder.build_float_add(left, right, "simd.add.f32")?,
            SimdInterpretation::U8 => builder.build_int_add(left, right, "simd.add.u8")?,
        });
        Ok(())
    }
}

/// Caller emits its ordinary trap guard before accessing a nonempty vector.
pub(crate) fn address_valid<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    pointer: PointerValue<'ctx>,
    width: SimdWidth,
) -> Result<IntValue<'ctx>, Error> {
    let address = builder.build_ptr_to_int(pointer, context.i64_type(), "simd.address")?;
    let nonnull = builder.build_is_not_null(pointer, "simd.nonnull")?;
    let fits = builder.build_int_compare(
        IntPredicate::ULE,
        address,
        context
            .i64_type()
            .const_int(u64::MAX - width.bytes() as u64, false),
        "simd.range.fits",
    )?;
    Ok(builder.build_and(nonnull, fits, "simd.address.valid")?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arm64_breakpoint_preserves_its_actual_instruction_bytes_at_o0_and_o2() {
        use inkwell::{
            passes::PassBuilderOptions,
            targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetTriple},
        };
        Target::initialize_all(&InitializationConfig::default());
        let triple = TargetTriple::create("aarch64-unknown-linux-gnu");
        check_arm64_architecture("aarch64-unknown-linux-gnu").unwrap();
        check_arm64_architecture("arm64-apple-darwin").unwrap();
        assert!(matches!(
            check_arm64_architecture("x86_64-unknown-linux-gnu"),
            Err(Error::Arm64TrapTarget(_))
        ));
        let target = Target::from_triple(&triple).unwrap();
        for (pipeline, level) in [
            ("default<O0>", inkwell::OptimizationLevel::None),
            ("default<O2>", inkwell::OptimizationLevel::Default),
        ] {
            let machine = target
                .create_target_machine(
                    &triple,
                    "generic",
                    "",
                    level,
                    RelocMode::PIC,
                    CodeModel::Default,
                )
                .unwrap();
            let context = Context::create();
            let module = context.create_module("arm64.brk1");
            module.set_triple(&triple);
            module.set_data_layout(&machine.get_target_data().get_data_layout());
            let function =
                module.add_function("breakpoint", context.i64_type().fn_type(&[], false), None);
            let builder = context.create_builder();
            builder.position_at_end(context.append_basic_block(function, "entry"));
            arm64_debug_trap(&context, &builder).unwrap();
            builder
                .build_return(Some(&context.i64_type().const_int(42, false)))
                .unwrap();
            module.verify().unwrap();
            module
                .run_passes(pipeline, &machine, PassBuilderOptions::create())
                .unwrap();
            module.verify().unwrap();
            let object = machine
                .write_to_memory_buffer(&module, FileType::Object)
                .unwrap();
            let bytes = object.as_slice();
            assert_eq!(&bytes[..4], b"\x7fELF");
            assert_eq!(u16::from_le_bytes(bytes[18..20].try_into().unwrap()), 183);
            assert!(
                bytes
                    .windows(4)
                    .any(|word| word == [0x20, 0x00, 0x20, 0xd4])
            );
            let assembly = machine
                .write_to_memory_buffer(&module, FileType::Assembly)
                .unwrap();
            let assembly = String::from_utf8_lossy(assembly.as_slice());
            assert!(assembly.contains("brk"), "{assembly}");
            assert!(
                assembly.contains("#0x1") || assembly.contains("#1"),
                "{assembly}"
            );
            assert!(
                assembly.contains("ret"),
                "breakpoint continuation must remain present: {assembly}"
            );
        }
    }
    use jai_ir::{SimdBuilder, SimdInstruction, ValueExpr};
    use jai_types::{FloatType, TypeRegistry};
    #[test]
    fn linux_objects_contain_real_simd_instructions_at_o0_and_o2() {
        use inkwell::{
            passes::PassBuilderOptions,
            targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetTriple},
        };
        Target::initialize_all(&InitializationConfig::default());
        let triple = TargetTriple::create("x86_64-unknown-linux-gnu");
        let target = Target::from_triple(&triple).unwrap();
        let mut types = TypeRegistry::new();
        let float = types.float(FloatType::F32);
        let pointer = types.pointer(float).unwrap();
        let types = types.freeze().unwrap();
        for (pipeline, level) in [
            ("default<O0>", inkwell::OptimizationLevel::None),
            ("default<O2>", inkwell::OptimizationLevel::Default),
        ] {
            let context = Context::create();
            let machine = target
                .create_target_machine(
                    &triple,
                    "generic",
                    "+avx,+avx2",
                    level,
                    RelocMode::PIC,
                    CodeModel::Default,
                )
                .unwrap();
            check_target(
                "x86_64-unknown-linux-gnu",
                "generic",
                "+avx,+avx2",
                SimdFeatures {
                    avx: true,
                    avx2: true,
                },
            )
            .unwrap();
            let module = context.create_module("simd.machine");
            module.set_triple(&triple);
            module.set_data_layout(&machine.get_target_data().get_data_layout());
            for (name, width, interpretation) in [
                ("float4", SimdWidth::X128, SimdInterpretation::F32),
                ("float8", SimdWidth::Y256, SimdInterpretation::F32),
                ("byte16", SimdWidth::X128, SimdInterpretation::U8),
                ("byte32", SimdWidth::Y256, SimdInterpretation::U8),
            ] {
                let mut checked = SimdBuilder::new(SimdFeatures {
                    avx: true,
                    avx2: true,
                })
                .unwrap();
                let left = checked.register(width).unwrap();
                let right = checked.register(width).unwrap();
                for destination in [left, right] {
                    checked
                        .instruction(
                            SimdInstruction::Load {
                                destination,
                                interpretation,
                                address: ValueExpr::Zero(pointer),
                            },
                            &types,
                        )
                        .unwrap();
                }
                checked
                    .instruction(
                        SimdInstruction::Add {
                            destination: left,
                            left,
                            right,
                            interpretation,
                        },
                        &types,
                    )
                    .unwrap();
                checked
                    .instruction(
                        SimdInstruction::Store {
                            source: left,
                            interpretation,
                            address: ValueExpr::Zero(pointer),
                        },
                        &types,
                    )
                    .unwrap();
                let checked = checked.finish();
                let llvm_pointer = context.ptr_type(inkwell::AddressSpace::default());
                let function = module.add_function(
                    name,
                    context
                        .void_type()
                        .fn_type(&[llvm_pointer.into(); 3], false),
                    None,
                );
                let builder = context.create_builder();
                builder.position_at_end(context.append_basic_block(function, "entry"));
                let argument = |index| function.get_nth_param(index).unwrap().into_pointer_value();
                let mut registers = Registers::new(&checked);
                registers
                    .load(
                        &context,
                        &builder,
                        &checked,
                        left,
                        interpretation,
                        argument(0),
                    )
                    .unwrap();
                registers
                    .load(
                        &context,
                        &builder,
                        &checked,
                        right,
                        interpretation,
                        argument(1),
                    )
                    .unwrap();
                registers
                    .add(
                        &context,
                        &builder,
                        &checked,
                        (left, left, right),
                        interpretation,
                    )
                    .unwrap();
                registers
                    .store(
                        &context,
                        &builder,
                        &checked,
                        left,
                        interpretation,
                        argument(2),
                    )
                    .unwrap();
                builder.build_return(None).unwrap();
            }
            module.verify().unwrap();
            module
                .run_passes(pipeline, &machine, PassBuilderOptions::create())
                .unwrap();
            module.verify().unwrap();
            let object = machine
                .write_to_memory_buffer(&module, FileType::Object)
                .unwrap();
            assert_eq!(&object.as_slice()[..4], b"\x7fELF");
            let assembly = machine
                .write_to_memory_buffer(&module, FileType::Assembly)
                .unwrap();
            let assembly = std::str::from_utf8(assembly.as_slice()).unwrap();
            assert!(assembly.contains("vaddps"), "{pipeline}: {assembly}");
            assert!(assembly.contains("vpaddb"), "{pipeline}: {assembly}");
            assert!(assembly.contains("%ymm"), "{pipeline}: {assembly}");
            assert!(assembly.contains("%xmm"), "{pipeline}: {assembly}");
        }
    }
    #[test]
    fn llvm_vector_operations_reinterpret_bits_and_use_unaligned_memory() {
        let context = Context::create();
        let mut types = TypeRegistry::new();
        let float = types.float(FloatType::F32);
        let pointer = types.pointer(float).unwrap();
        let types = types.freeze().unwrap();
        for width in [SimdWidth::X128, SimdWidth::Y256] {
            let mut checked = SimdBuilder::new(SimdFeatures {
                avx: true,
                avx2: true,
            })
            .unwrap();
            let register = checked.register(width).unwrap();
            checked
                .instruction(
                    SimdInstruction::Load {
                        destination: register,
                        interpretation: SimdInterpretation::F32,
                        address: ValueExpr::Zero(pointer),
                    },
                    &types,
                )
                .unwrap();
            checked
                .instruction(
                    SimdInstruction::Add {
                        destination: register,
                        left: register,
                        right: register,
                        interpretation: SimdInterpretation::F32,
                    },
                    &types,
                )
                .unwrap();
            checked
                .instruction(
                    SimdInstruction::Add {
                        destination: register,
                        left: register,
                        right: register,
                        interpretation: SimdInterpretation::U8,
                    },
                    &types,
                )
                .unwrap();
            checked
                .instruction(
                    SimdInstruction::Store {
                        source: register,
                        interpretation: SimdInterpretation::U8,
                        address: ValueExpr::Zero(pointer),
                    },
                    &types,
                )
                .unwrap();
            let checked = checked.finish();
            checked.validate(&types).unwrap();
            let module = context.create_module("simd.vectors");
            let llvm_pointer = context.ptr_type(inkwell::AddressSpace::default());
            let function = module.add_function(
                "transform",
                context
                    .bool_type()
                    .fn_type(&[llvm_pointer.into(), llvm_pointer.into()], false),
                None,
            );
            let builder = context.create_builder();
            builder.position_at_end(context.append_basic_block(function, "entry"));
            let source = function.get_first_param().unwrap().into_pointer_value();
            let destination = function.get_nth_param(1).unwrap().into_pointer_value();
            let valid = address_valid(&context, &builder, source, width).unwrap();
            let mut registers = Registers::new(&checked);
            registers
                .load(
                    &context,
                    &builder,
                    &checked,
                    register,
                    SimdInterpretation::F32,
                    source,
                )
                .unwrap();
            registers
                .add(
                    &context,
                    &builder,
                    &checked,
                    (register, register, register),
                    SimdInterpretation::F32,
                )
                .unwrap();
            registers
                .add(
                    &context,
                    &builder,
                    &checked,
                    (register, register, register),
                    SimdInterpretation::U8,
                )
                .unwrap();
            registers
                .store(
                    &context,
                    &builder,
                    &checked,
                    register,
                    SimdInterpretation::U8,
                    destination,
                )
                .unwrap();
            builder.build_return(Some(&valid)).unwrap();
            module.verify().unwrap();
            let mut instruction = function
                .get_first_basic_block()
                .unwrap()
                .get_first_instruction();
            while let Some(value) = instruction {
                if matches!(
                    value.get_opcode(),
                    inkwell::values::InstructionOpcode::Load
                        | inkwell::values::InstructionOpcode::Store
                ) {
                    assert_eq!(value.get_alignment().unwrap(), 1);
                }
                instruction = value.get_next_instruction();
            }
        }
    }
    #[test]
    fn isa_requirements_reject_wrong_architecture_and_disabled_features() {
        let wide = SimdFeatures {
            avx: true,
            avx2: true,
        };
        assert!(matches!(
            check_target(
                "x86_64-unknown-linux-gnu",
                "generic",
                "-sse2",
                SimdFeatures::default()
            ),
            Err(Error::MissingFeature("sse2"))
        ));
        assert!(matches!(
            check_target("aarch64-apple-darwin", "generic", "+avx,+avx2", wide),
            Err(Error::UnsupportedArchitecture(_))
        ));
        assert!(matches!(
            check_target("x86_64-unknown-linux-gnu", "generic", "", wide),
            Err(Error::MissingFeature("avx"))
        ));
        assert!(matches!(
            check_target(
                "x86_64-unknown-linux-gnu",
                "generic",
                "+avx,+avx2,-avx2",
                wide
            ),
            Err(Error::MissingFeature("avx2"))
        ));
        assert!(matches!(
            check_target("x86_64-unknown-linux-gnu", "generic", "+avx2,-avx", wide),
            Err(Error::MissingFeature("avx"))
        ));
        check_target("x86_64-unknown-linux-gnu", "generic", "+avx2", wide).unwrap();
        check_target(
            "x86_64-unknown-linux-gnu",
            "generic",
            "",
            SimdFeatures::default(),
        )
        .unwrap();
    }
}
