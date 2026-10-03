use super::*;
use crate::{ByteImage, Number, virtual_heap::VirtualHeap};
use jai_ir::NativePointerConstant;
use jai_types::{LayoutPolicy, ScalarLayout, TypeRegistry};
fn target(width: u64, endian: Endian) -> ByteTarget {
    ByteTarget {
        policy: LayoutPolicy::new(
            ScalarLayout::new(width, width as u32),
            [
                ScalarLayout::new(1, 1),
                ScalarLayout::new(2, 2),
                ScalarLayout::new(4, 4),
                ScalarLayout::new(8, 4),
            ],
            [ScalarLayout::new(4, 4), ScalarLayout::new(8, 4)],
            ScalarLayout::new(1, 1),
        )
        .unwrap(),
        endian,
    }
}
fn raw(
    memory: &Memory,
    types: &TypeRegistry,
    bits: i128,
    source: IntegerType,
    mode: CastMode,
) -> Result<Pointer, Error> {
    memory.integer_to_pointer(
        types,
        Number::plain(Integer::wrapping(source, bits)),
        types.void(),
        mode,
    )
}
#[test]
fn native_and_runtime_numeric_casts_preserve_selected_width_bits() {
    let mut types = TypeRegistry::new();
    let ty = types.pointer(types.void()).unwrap();
    for bytes in [4, 8] {
        let t = target(bytes, Endian::Little);
        let memory = Memory::with_target(Limits::default(), t);
        let width = bytes as u32 * 8;
        for mode in [CastMode::Checked, CastMode::Unchecked, CastMode::Truncate] {
            let c = NativePointerConstant::new_weak(ty, 1, mode, &types).unwrap();
            let p = crate::constants::native_pointer(&types, &c, t)
                .unwrap()
                .pointer()
                .unwrap()
                .clone();
            assert_eq!(p.opaque_address_bits(), Some((1, width)));
            assert!(!p.is_null());
            assert_eq!(p.data_allocation_key(), None);
            assert_eq!(p.code_pointer(), None);
            assert!(
                memory
                    .same_address(
                        &types,
                        &p,
                        &raw(&memory, &types, 1, IntegerType::U64, mode).unwrap()
                    )
                    .unwrap()
            );
            assert!(
                !memory
                    .same_address(&types, &p, &Pointer::null(types.void()))
                    .unwrap()
            );
            let n = memory
                .pointer_to_integer(&types, &p, IntegerType::U64, mode)
                .unwrap();
            assert_eq!(n.bits(), 1);
            assert!(n.provenance().is_none());
        }
        let negative = NativePointerConstant::new_weak(ty, -1, CastMode::Checked, &types).unwrap();
        let p = crate::constants::native_pointer(&types, &negative, t)
            .unwrap()
            .pointer()
            .unwrap()
            .clone();
        assert_eq!(
            p.opaque_address_bits(),
            Some((u64::MAX >> (64 - width), width))
        );
        let strong = NativePointerConstant::new(
            ty,
            Integer::wrapping(IntegerType::S32, -1),
            CastMode::Checked,
            &types,
        )
        .unwrap();
        assert_eq!(
            crate::constants::native_pointer(&types, &strong, t)
                .unwrap()
                .pointer()
                .unwrap(),
            &p
        );
        assert_eq!(
            memory.pointer_to_integer(&types, &p, IntegerType::U8, CastMode::Checked),
            Err(Error::CheckedCast)
        );
        assert_eq!(
            memory
                .pointer_to_integer(&types, &p, IntegerType::U8, CastMode::Truncate)
                .unwrap()
                .bits(),
            255
        );
        let large = 1i128 << 32;
        if bytes == 4 {
            assert_eq!(
                raw(&memory, &types, large, IntegerType::U64, CastMode::Checked),
                Err(Error::CheckedCast)
            );
            for mode in [CastMode::Unchecked, CastMode::Truncate] {
                assert!(
                    raw(&memory, &types, large, IntegerType::U64, mode)
                        .unwrap()
                        .is_null()
                );
            }
        } else {
            assert_eq!(
                raw(&memory, &types, large, IntegerType::U64, CastMode::Checked)
                    .unwrap()
                    .opaque_address_bits(),
                Some((large as u64, 64))
            );
        }
    }
}
#[test]
fn byte_storage_preserves_numeric_bits_without_creating_relocation_authority() {
    let mut types = TypeRegistry::new();
    let ty = types.pointer(types.void()).unwrap();
    for bytes in [4, 8] {
        for endian in [Endian::Little, Endian::Big] {
            let t = target(bytes, endian);
            let mut memory = Memory::with_target(Limits::default(), t);
            let p = raw(
                &memory,
                &types,
                0x12345678,
                IntegerType::U32,
                CastMode::Checked,
            )
            .unwrap();
            let value = Value::Pointer(p.clone());
            let image = ByteImage::encode(&types, t, ty, &value, 128).unwrap();
            assert_eq!(image.metadata_cells(), 0);
            assert_eq!(image.read(&types, t, 0, ty).unwrap(), value);
            let raw_image = ByteImage::from_bytes(t, image.bytes().to_vec(), 128).unwrap();
            assert_eq!(raw_image.read(&types, t, 0, ty).unwrap(), value);
            let slot = memory.allocate(&types, ty, Some(value.clone())).unwrap();
            assert_eq!(memory.load(&types, &slot).unwrap(), value);
            let one = Value::Pointer(
                raw(&memory, &types, 1, IntegerType::U64, CastMode::Checked).unwrap(),
            );
            memory.store(&types, &slot, one.clone()).unwrap();
            assert_eq!(memory.load(&types, &slot).unwrap(), one);
            assert!(raw_image.read(&types, t, 0, types.meta_type()).is_err());
        }
    }
}
#[test]
fn matching_real_heap_address_bits_never_forge_a_heap_or_host_capability() {
    let types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    for bytes in [4, 8] {
        let mut memory = Memory::with_target(Limits::default(), target(bytes, Endian::Little));
        let mut heap = VirtualHeap::default();
        let root = heap.malloc(&mut memory, &types, 8).unwrap();
        let address = memory
            .pointer_to_integer(&types, &root, IntegerType::U64, CastMode::Checked)
            .unwrap();
        for bits in [1, address.bits()] {
            let p = raw(
                &memory,
                &types,
                i128::from(bits),
                IntegerType::U64,
                CastMode::Unchecked,
            )
            .unwrap();
            let p = memory
                .cast_pointer(&types, &p, byte, CastMode::Checked)
                .unwrap();
            let denied = Err(Error::UnsupportedPointerOperation(
                "numeric address has no allocation provenance",
            ));
            assert_eq!(memory.load(&types, &p), denied.clone());
            assert_eq!(
                memory.store(
                    &types,
                    &p,
                    Value::Int(Integer::wrapping(IntegerType::U8, 3))
                ),
                Err(Error::UnsupportedPointerOperation(
                    "numeric address has no allocation provenance"
                ))
            );
            assert_eq!(
                memory.host_read_bytes(&types, &p, 1),
                Err(Error::UnsupportedPointerOperation(
                    "numeric address has no allocation provenance"
                ))
            );
            assert_eq!(
                heap.free(&mut memory, &types, &p),
                Err(Error::InvalidIr("pointer is not owned by the virtual heap"))
            );
            assert_eq!(
                heap.realloc(&mut memory, &types, &p, 16),
                Err(Error::InvalidIr("pointer is not owned by the virtual heap"))
            );
        }
        assert_eq!(heap.allocation_count(), 1);
        heap.free(&mut memory, &types, &root).unwrap();
        assert_eq!(heap.allocation_count(), 0);
    }
}
#[test]
fn numeric_pointer_admission_is_bounded_and_target_specific() {
    let types = TypeRegistry::new();
    let narrow = Memory::with_target(Limits::default(), target(4, Endian::Little));
    let p = raw(&narrow, &types, 1, IntegerType::U32, CastMode::Checked).unwrap();
    assert_eq!(
        narrow.prepare_pointer_layouts(&types, &p, false, 0),
        (0, Err(Error::Limit(LimitKind::Fuel)))
    );
    assert_eq!(
        narrow.prepare_pointer_layouts(&types, &p, false, 1),
        (1, Ok(()))
    );
    assert_eq!(
        narrow.prepare_pointer_layouts(&types, &p, true, 1),
        (
            1,
            Err(Error::UnsupportedPointerOperation(
                "numeric address has no allocation provenance"
            ))
        )
    );
    let wide = Memory::with_target(Limits::default(), target(8, Endian::Little));
    assert_eq!(
        wide.cast_pointer(&types, &p, types.void(), CastMode::Checked),
        Err(Error::UnsupportedPointerOperation(
            "numeric pointer belongs to another target width"
        ))
    );
}
#[test]
fn actual_data_and_code_origins_keep_their_receipts_after_numeric_support() {
    use jai_types::{CallingConvention, ContextMode, ProcedureType, Variadic};
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let pointer_type = types.pointer(types.void()).unwrap();
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    for bytes in [4, 8] {
        let t = target(bytes, Endian::Little);
        let mut memory = Memory::with_target(Limits::default(), t);
        let data = memory
            .allocate(
                &types,
                byte,
                Some(Value::Int(Integer::wrapping(IntegerType::U8, 9))),
            )
            .unwrap();
        let n = memory
            .pointer_to_integer(&types, &data, IntegerType::U64, CastMode::Checked)
            .unwrap();
        assert!(n.provenance().is_some());
        let recovered = memory
            .integer_to_pointer(&types, n.clone(), byte, CastMode::Checked)
            .unwrap();
        assert_eq!(
            memory.load(&types, &recovered).unwrap(),
            Value::Int(Integer::wrapping(IntegerType::U8, 9))
        );
        let guessed = memory
            .integer_to_pointer(&types, Number::plain(n.integer()), byte, CastMode::Checked)
            .unwrap();
        assert!(memory.same_address(&types, &guessed, &data).unwrap());
        assert!(guessed.is_opaque());
        assert_eq!(guessed.data_allocation_key(), None);
        let procedure = Value::Procedure {
            signature,
            procedure: Some(jai_ir::ProcedureId::new(7)),
        };
        let cell = memory
            .allocate(&types, signature, Some(procedure.clone()))
            .unwrap();
        let view = memory
            .cast_pointer(&types, &cell, pointer_type, CastMode::Checked)
            .unwrap();
        let code = memory
            .load(&types, &view)
            .unwrap()
            .pointer()
            .unwrap()
            .clone();
        assert!(code.code_pointer().is_some());
        assert_eq!(code.data_allocation_key(), None);
        let n = memory
            .pointer_to_integer(&types, &code, IntegerType::U64, CastMode::Checked)
            .unwrap();
        let recovered = memory
            .integer_to_pointer(&types, n.clone(), types.void(), CastMode::Checked)
            .unwrap();
        assert_eq!(recovered.code_pointer(), code.code_pointer());
        let fake = raw(
            &memory,
            &types,
            i128::from(n.bits()),
            IntegerType::U64,
            CastMode::Checked,
        )
        .unwrap();
        assert!(fake.is_opaque());
        assert_eq!(fake.code_pointer(), None);
        let image = ByteImage::encode(&types, t, pointer_type, &Value::Pointer(fake), 128).unwrap();
        assert_eq!(
            image.read(&types, t, 0, signature),
            Err(Error::InvalidIr(
                "byte storage cannot forge procedure provenance"
            ))
        );
        memory.release(&cell).unwrap();
        memory.release(&data).unwrap();
    }
}
#[test]
fn genuine_vm_ir_numeric_values_support_truth_equality_and_integer_roundtrip() {
    use jai_ir::{BoolExpr, IntExpr, IntExprKind, ProgramBuilder, ValueExpr};
    use jai_types::Equality;
    let mut types = TypeRegistry::new();
    let ty = types.pointer(types.void()).unwrap();
    let c = NativePointerConstant::new_weak(ty, 1, CastMode::Checked, &types).unwrap();
    let library = ProgramBuilder::new(types.freeze().unwrap())
        .finish_library()
        .unwrap();
    for bytes in [4, 8] {
        let mut vm = crate::Vm::new_with_target(
            &library,
            crate::NoEffects,
            Limits::default(),
            target(bytes, Endian::Little),
        )
        .unwrap();
        let literal = ValueExpr::NativePointer(c.clone());
        let runtime = ValueExpr::PointerFromInteger {
            value: IntExpr::constant(Integer::wrapping(IntegerType::U64, 1)),
            ty,
            mode: CastMode::Checked,
        };
        for expr in [
            ValueExpr::Bool(BoolExpr::FromPointer(Box::new(literal.clone()))),
            ValueExpr::Bool(BoolExpr::ComparePointers(
                Equality::Equal,
                Box::new(literal.clone()),
                Box::new(runtime),
            )),
        ] {
            assert_eq!(
                vm.evaluate(&expr).outcome,
                crate::Outcome::Complete(vec![Value::Bool(true)])
            );
        }
        let int = ValueExpr::Int(IntExpr::new(
            IntegerType::U64,
            IntExprKind::FromPointer {
                value: Box::new(literal),
                mode: CastMode::Checked,
            },
        ));
        assert_eq!(
            vm.evaluate(&int).outcome,
            crate::Outcome::Complete(vec![Value::Int(Integer::wrapping(IntegerType::U64, 1))])
        );
    }
}

#[test]
fn descriptor_pointer_rejects_another_target_before_storage_retention() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let string = types.string();
    let slice = types.slice(byte).unwrap();
    let dynamic = types.dynamic_array(byte).unwrap();
    let narrow = Memory::with_target(Limits::default(), target(4, Endian::Little));
    let numeric = raw(&narrow, &types, 1, IntegerType::U32, CastMode::Checked).unwrap();
    let numeric = narrow
        .cast_pointer(&types, &numeric, byte, CastMode::Checked)
        .unwrap();
    let mut wide = Memory::with_target(Limits::default(), target(8, Endian::Little));
    for (ty, value) in [
        (
            string,
            Value::StringView {
                pointer: numeric.clone(),
                count: 0,
            },
        ),
        (
            slice,
            Value::Slice {
                ty: slice,
                pointer: numeric.clone(),
                count: 0,
            },
        ),
        (
            dynamic,
            Value::DynamicArray {
                ty: dynamic,
                pointer: numeric,
                count: 0,
                allocated: 0,
                allocator: None,
            },
        ),
    ] {
        assert_eq!(
            wide.allocate(&types, ty, Some(value)),
            Err(Error::UnsupportedPointerOperation(
                "numeric pointer belongs to another target width"
            ))
        );
        assert_eq!(wide.allocation_count(), 0);
    }
    // A native allocator's data payload has its own numeric domain; the top-level
    // descriptor check must not skip this recursively typed child.
    use jai_types::{
        AllocatorMode, CallingConvention, ContextMode, ProcedureType, RecordKind, Variadic,
    };
    let mode = types.reserve_enum(IntegerType::S64);
    types
        .define_enum(mode, AllocatorMode::ALL.map(AllocatorMode::value))
        .unwrap();
    let data = types.pointer(types.void()).unwrap();
    let size = types.scalar(ScalarType::Int(IntegerType::S64));
    let procedure = types
        .procedure(ProcedureType {
            parameters: Box::new([mode, size, size, data, data]),
            results: Box::new([data]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::Implicit,
            variadic: Variadic::None,
        })
        .unwrap();
    let allocator = types.reserve_record(RecordKind::Struct);
    types.define_record(allocator, [procedure, data]).unwrap();
    types.bind_allocator(allocator, mode).unwrap();
    let payload = Value::Record {
        ty: allocator,
        fields: vec![
            Value::Procedure {
                signature: procedure,
                procedure: None,
            },
            Value::Pointer(raw(&narrow, &types, 1, IntegerType::U32, CastMode::Checked).unwrap()),
        ],
    };
    let descriptor = Value::DynamicArray {
        ty: dynamic,
        pointer: wide
            .cast_pointer(
                &types,
                &raw(&wide, &types, 1, IntegerType::U64, CastMode::Checked).unwrap(),
                byte,
                CastMode::Checked,
            )
            .unwrap(),
        count: 0,
        allocated: 0,
        allocator: Some(Box::new(payload)),
    };
    assert_eq!(
        wide.allocate(&types, dynamic, Some(descriptor)),
        Err(Error::UnsupportedPointerOperation(
            "numeric pointer belongs to another target width"
        ))
    );
    assert_eq!(wide.allocation_count(), 0);
}
#[test]
fn compiler_materialization_checks_numeric_domains_and_work_before_cloning() {
    use jai_ir::ProgramBuilder;
    let mut types = TypeRegistry::new();
    let void = types.void();
    types.pointer(void).unwrap();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let slice = types.slice(byte).unwrap();
    let library = ProgramBuilder::new(types.freeze().unwrap())
        .finish_library()
        .unwrap();
    let narrow = Memory::with_target(Limits::default(), target(4, Endian::Little));
    let pointer = narrow
        .integer_to_pointer(
            library.types(),
            Number::plain(Integer::wrapping(IntegerType::U32, 1)),
            byte,
            CastMode::Checked,
        )
        .unwrap();
    let vm = crate::Vm::new_with_target(
        &library,
        crate::NoEffects,
        Limits::default(),
        target(8, Endian::Little),
    )
    .unwrap();
    for value in [
        Value::Pointer(pointer.clone()),
        Value::Slice {
            ty: slice,
            pointer,
            count: 0,
        },
    ] {
        assert_eq!(
            vm.materialize_value(&value),
            Err(Error::UnsupportedPointerOperation(
                "numeric pointer belongs to another target width"
            ))
        );
    }
    let wide = Memory::with_target(Limits::default(), target(8, Endian::Little));
    let value = Value::Pointer(
        wide.integer_to_pointer(
            library.types(),
            Number::plain(Integer::wrapping(IntegerType::U64, 1)),
            void,
            CastMode::Checked,
        )
        .unwrap(),
    );
    assert_eq!(vm.materialize_value(&value).unwrap(), value);
    let limited = crate::Vm::new_with_target(
        &library,
        crate::NoEffects,
        Limits {
            fuel: 1,
            ..Limits::default()
        },
        target(8, Endian::Little),
    )
    .unwrap();
    assert_eq!(
        limited.materialize_value(&value),
        Err(Error::Limit(LimitKind::Fuel))
    );
}
