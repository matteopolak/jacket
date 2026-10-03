use super::*;
use jai_types::{
    CallingConvention, ContextMode, ProcedureType, ScalarType, TypeRegistry, Variadic,
};

fn procedure(
    types: &mut TypeRegistry,
    name: &str,
    parameters: &[TypeId],
    results: &[TypeId],
) -> RuntimeProcedure {
    let signature = types
        .procedure(ProcedureType {
            parameters: parameters.into(),
            results: results.into(),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    RuntimeProcedure {
        signature,
        intrinsic: RuntimeIntrinsic::bind(name, signature, types, jai_types::LayoutPolicy::lp64())
            .unwrap(),
    }
}
fn count(value: i128) -> Value {
    Value::Int(Integer::wrapping(IntegerType::S64, value))
}

#[test]
fn layout_preparation_skips_zero_counts_and_borrows_nonempty_range_operands() {
    let mut types = TypeRegistry::new();
    let pointer = types.pointer(types.void()).unwrap();
    let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
    let copy = procedure(&mut types, "memcpy", &[pointer, pointer, s64], &[]);
    let mut visits = 0;
    // Neither operand may be inspected when the count is zero, even if a caller
    // invokes this preparation helper before its separate ABI validation.
    copy.visit_pointer_layouts(
        &[Value::Bool(false), Value::Bool(false), count(0)],
        |_, _| {
            visits += 1;
            Ok::<_, Error>(())
        },
    )
    .unwrap();
    assert_eq!(visits, 0);
    let arguments = [
        Value::Pointer(Pointer::null(types.void())),
        Value::Pointer(Pointer::null(types.void())),
        count(1),
    ];
    copy.visit_pointer_layouts(&arguments, |borrowed, include_pointee| {
        assert!(!include_pointee);
        assert!(std::ptr::eq(borrowed, arguments[visits].pointer().unwrap()));
        visits += 1;
        Ok::<_, Error>(())
    })
    .unwrap();
    assert_eq!(visits, 2);
    assert_eq!(
        copy.visit_pointer_layouts(
            &[Value::Bool(false), Value::Bool(false), count(-1)],
            |_, _| Ok::<_, Error>(())
        ),
        Err(Error::CheckedCast)
    );
}

#[test]
fn pool_layout_preparation_admits_fields_and_new_backing_before_execution() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::S64));
    let opaque = types.pointer(types.void()).unwrap();
    let pool = types.reserve_record(jai_types::RecordKind::Struct);
    types
        .define_record(pool, [word, word, opaque, word])
        .unwrap();
    let pool_pointer = types.pointer(pool).unwrap();
    let get = procedure(&mut types, "get", &[pool_pointer, word], &[opaque]);
    let empty = [Value::Bool(false), count(0)];
    get.visit_pointer_layouts(&empty, |_, _| Err::<(), Error>(Error::RuntimeTrap))
        .unwrap();
    get.visit_additional_layouts(&empty, &types, |_| Err::<(), Error>(Error::RuntimeTrap))
        .unwrap();
    let arguments = [Value::Pointer(Pointer::null(pool)), count(8)];
    get.visit_pointer_layouts(&arguments, |pointer, include| {
        assert!(include);
        assert!(std::ptr::eq(pointer, arguments[0].pointer().unwrap()));
        Ok::<_, Error>(())
    })
    .unwrap();
    let mut demands = vec![];
    get.visit_additional_layouts(&arguments, &types, |ty| {
        demands.push(ty);
        Ok::<_, Error>(())
    })
    .unwrap();
    assert_eq!(demands, [word, word, opaque, word, types.string()]);
    assert_eq!(
        get.visit_additional_layouts(&arguments, &types, |_| Err::<(), Error>(Error::Limit(
            LimitKind::Fuel
        ))),
        Err(Error::Limit(LimitKind::Fuel))
    );
}

#[test]
fn negative_counts_and_malformed_arguments_do_not_mutate_memory() {
    let mut types = TypeRegistry::new();
    let void_pointer = types.pointer(types.void()).unwrap();
    let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
    let u8 = types.scalar(ScalarType::Int(IntegerType::U8));
    let set = procedure(&mut types, "memset", &[void_pointer, u8, s64], &[]);
    let mut memory = Memory::new(crate::Limits::default());
    let root = memory.allocate(&types, s64, Some(count(7))).unwrap();
    let void_root = memory
        .cast_pointer(&types, &root, types.void(), jai_types::CastMode::Unchecked)
        .unwrap();
    let byte = Value::Int(Integer::wrapping(IntegerType::U8, 0));
    let arguments = [Value::Pointer(void_root), byte, count(-1)];
    assert_eq!(set.work_cost(&arguments), Err(Error::CheckedCast));
    assert_eq!(
        set.invoke(&arguments, &mut memory, &types),
        Err(Error::CheckedCast)
    );
    assert_eq!(memory.load(&types, &root).unwrap(), count(7));
    assert!(set.invoke(&[], &mut memory, &types).is_err());
}

#[test]
fn address_derived_counts_cannot_drive_byte_work_or_mutate_storage() {
    let mut types = TypeRegistry::new();
    let void_pointer = types.pointer(types.void()).unwrap();
    let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
    let u8 = types.scalar(ScalarType::Int(IntegerType::U8));
    let set = procedure(&mut types, "memset", &[void_pointer, u8, s64], &[]);
    let mut memory = Memory::new(crate::Limits::default());
    let root = memory.allocate(&types, s64, Some(count(7))).unwrap();
    let address = memory
        .pointer_to_integer(
            &types,
            &root,
            IntegerType::S64,
            jai_types::CastMode::Unchecked,
        )
        .unwrap();
    let destination = memory
        .cast_pointer(&types, &root, types.void(), jai_types::CastMode::Unchecked)
        .unwrap();
    let arguments = [
        Value::Pointer(destination),
        Value::Int(Integer::wrapping(IntegerType::U8, 0)),
        address.into_value(),
    ];
    assert!(matches!(
        set.work_cost(&arguments),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    assert!(matches!(
        set.work_cost_for_target(&arguments, &types, &memory),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    assert!(matches!(
        set.invoke(&arguments, &mut memory, &types),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    assert_eq!(memory.load(&types, &root).unwrap(), count(7));
}

#[test]
fn runtime_adapter_has_closed_type_checked_results_and_a_real_trap() {
    let mut types = TypeRegistry::new();
    let void_pointer = types.pointer(types.void()).unwrap();
    let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
    let s16 = types.scalar(ScalarType::Int(IntegerType::S16));
    let compare = procedure(
        &mut types,
        "memcmp",
        &[void_pointer, void_pointer, s64],
        &[s16],
    );
    let null = Value::Pointer(crate::Pointer::null(types.void()));
    let mut memory = Memory::new(crate::Limits::default());
    assert_eq!(
        compare.work_cost_for_target(&[null.clone(), null.clone(), count(0)], &types, &memory),
        Ok(0)
    );
    assert_eq!(
        compare
            .invoke(&[null.clone(), null, count(0)], &mut memory, &types)
            .unwrap(),
        vec![Value::Int(Integer::wrapping(IntegerType::S16, 0))]
    );
    let trap = procedure(&mut types, "llvm.debugtrap", &[], &[]);
    assert_eq!(
        trap.invoke(&[], &mut memory, &types),
        Err(Error::RuntimeTrap)
    );
    let malformed = RuntimeProcedure {
        signature: trap.signature,
        intrinsic: RuntimeIntrinsic::MemorySet,
    };
    assert!(matches!(
        malformed.invoke(&[], &mut memory, &types),
        Err(Error::IrValidation(_))
    ));
}

#[test]
fn staged_atomic_type_dependencies_remain_typed_before_invocation() {
    let mut types = TypeRegistry::new();
    let pending = types.reserve_enum(IntegerType::U8);
    let pointer = types.pointer(pending).unwrap();
    let boolean = types.scalar(ScalarType::Bool);
    let signature = types
        .procedure(ProcedureType {
            parameters: vec![pointer, pending, pending].into(),
            results: vec![boolean, pending].into(),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let runtime = RuntimeProcedure {
        signature,
        intrinsic: RuntimeIntrinsic::CompareAndSwap { value: pending },
    };
    let mut memory = Memory::new(crate::Limits::default());
    let expected_error = Error::Type(jai_types::TypeError::Incomplete(pending));
    assert_eq!(
        runtime.invoke(&[], &mut memory, &types),
        Err(expected_error.clone())
    );
    assert_eq!(
        memory.compare_and_swap(&types, &crate::Pointer::null(pending), &count(0), &count(1),),
        Err(expected_error)
    );
}

struct RuntimeProvider {
    types: jai_types::Types,
    signature: std::collections::HashMap<jai_ir::ProcedureId, TypeId>,
    runtime: RuntimeProcedure,
}
impl RuntimeProvider {
    fn new(types: TypeRegistry, runtime: RuntimeProcedure) -> Self {
        Self {
            types: types.freeze().unwrap(),
            signature: [(jai_ir::ProcedureId::new(0), runtime.signature)].into(),
            runtime,
        }
    }
}
impl crate::ProcedureProvider for RuntimeProvider {
    fn types(&self) -> &dyn TypeView {
        &self.types
    }
    fn signatures(&self) -> &std::collections::HashMap<jai_ir::ProcedureId, TypeId> {
        &self.signature
    }
    fn procedure(&self, id: jai_ir::ProcedureId) -> crate::ProcedureAvailability<'_> {
        if id == jai_ir::ProcedureId::new(0) {
            crate::ProcedureAvailability::Runtime(self.runtime)
        } else {
            crate::ProcedureAvailability::Missing
        }
    }
}

#[test]
fn vm_dispatch_charges_all_byte_work_before_write_and_keeps_successful_results() {
    let mut types = TypeRegistry::new();
    let void_pointer = types.pointer(types.void()).unwrap();
    let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
    let u8 = types.scalar(ScalarType::Int(IntegerType::U8));
    let set = procedure(&mut types, "memset", &[void_pointer, u8, s64], &[]);
    let provider = RuntimeProvider::new(types, set);
    for (fuel, outcome, expected) in [
        (
            26,
            crate::Outcome::Failed(Error::Limit(LimitKind::Fuel)),
            count(7),
        ),
        (27, crate::Outcome::Complete(vec![]), count(0)),
    ] {
        let mut vm = crate::Vm::new(
            &provider,
            crate::NoEffects,
            crate::Limits {
                fuel,
                ..crate::Limits::default()
            },
        )
        .unwrap();
        let root = vm
            .memory_mut()
            .allocate(&provider.types, s64, Some(count(7)))
            .unwrap();
        let destination = vm
            .memory()
            .cast_pointer(
                &provider.types,
                &root,
                provider.types.lookup(&jai_types::TypeKind::Void).unwrap(),
                jai_types::CastMode::Unchecked,
            )
            .unwrap();
        vm.test_budget_source_work(fuel, false, 0);
        let result = vm.execute(
            jai_ir::ProcedureId::new(0),
            vec![
                Value::Pointer(destination),
                Value::Int(Integer::wrapping(IntegerType::U8, 0)),
                count(8),
            ],
        );
        assert_eq!(result.outcome, outcome);
        assert_eq!(vm.memory().load(&provider.types, &root).unwrap(), expected);
    }
}

#[test]
fn one_byte_memory_and_atomic_operations_charge_large_roots_before_effects() {
    for atomic in [false, true] {
        let mut types = TypeRegistry::new();
        let u8 = types.scalar(ScalarType::Int(IntegerType::U8));
        let array = types.fixed_array(u8, 1024).unwrap();
        let pointer = types.pointer(u8).unwrap();
        let void_pointer = types.pointer(types.void()).unwrap();
        let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
        let boolean = types.scalar(ScalarType::Bool);
        let operation = if atomic {
            procedure(
                &mut types,
                "compare_and_swap",
                &[pointer, u8, u8],
                &[boolean, u8],
            )
        } else {
            procedure(&mut types, "memset", &[void_pointer, u8, s64], &[])
        };
        let provider = RuntimeProvider::new(types, operation);
        let mut vm = crate::Vm::new(
            &provider,
            crate::NoEffects,
            crate::Limits {
                fuel: 100,
                ..crate::Limits::default()
            },
        )
        .unwrap();
        let zero = Value::Int(Integer::wrapping(IntegerType::U8, 0));
        let initial = Value::Array {
            ty: array,
            elements: vec![zero.clone(); 1024],
        };
        let root = vm
            .memory_mut()
            .allocate(&provider.types, array, Some(initial.clone()))
            .unwrap();
        let data = vm.memory().sequence_data(&provider.types, &root).unwrap();
        let arguments = if atomic {
            vec![
                Value::Pointer(data),
                zero,
                Value::Int(Integer::wrapping(IntegerType::U8, 1)),
            ]
        } else {
            let destination = vm
                .memory()
                .cast_pointer(
                    &provider.types,
                    &data,
                    provider.types.lookup(&jai_types::TypeKind::Void).unwrap(),
                    jai_types::CastMode::Unchecked,
                )
                .unwrap();
            vec![
                Value::Pointer(destination),
                Value::Int(Integer::wrapping(IntegerType::U8, 1)),
                count(1),
            ]
        };
        assert!(
            operation
                .work_cost_for_target(&arguments, &provider.types, vm.memory())
                .unwrap()
                > 1024
        );
        assert_eq!(
            vm.execute(jai_ir::ProcedureId::new(0), arguments).outcome,
            crate::Outcome::Failed(Error::Limit(LimitKind::Fuel))
        );
        assert_eq!(vm.memory().load(&provider.types, &root).unwrap(), initial);
    }
}

#[test]
fn vm_dispatch_preserves_multiresults_and_rejects_bad_binding_before_memory_effects() {
    let mut types = TypeRegistry::new();
    let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer = types.pointer(s64).unwrap();
    let boolean = types.scalar(ScalarType::Bool);
    let compare = procedure(
        &mut types,
        "compare_and_swap",
        &[pointer, s64, s64],
        &[boolean, s64],
    );
    let mut provider = RuntimeProvider::new(types, compare);
    {
        let mut vm = crate::Vm::new(&provider, crate::NoEffects, crate::Limits::default()).unwrap();
        let root = vm
            .memory_mut()
            .allocate(&provider.types, s64, Some(count(1)))
            .unwrap();
        let result = vm.execute(
            jai_ir::ProcedureId::new(0),
            vec![Value::Pointer(root.clone()), count(1), count(2)],
        );
        assert_eq!(
            result.outcome,
            crate::Outcome::Complete(vec![Value::Bool(true), count(1)])
        );
        assert_eq!(vm.memory().load(&provider.types, &root).unwrap(), count(2));
    }
    provider.runtime.intrinsic = RuntimeIntrinsic::DebugTrap;
    let mut vm = crate::Vm::new(&provider, crate::NoEffects, crate::Limits::default()).unwrap();
    let root = vm
        .memory_mut()
        .allocate(&provider.types, s64, Some(count(1)))
        .unwrap();
    let result = vm.execute(
        jai_ir::ProcedureId::new(0),
        vec![Value::Pointer(root.clone()), count(1), count(2)],
    );
    assert!(matches!(
        result.outcome,
        crate::Outcome::Failed(Error::IrValidation(_))
    ));
    assert_eq!(vm.memory().load(&provider.types, &root).unwrap(), count(1));
}

#[test]
fn vm_debugtrap_stops_execution_and_retains_preexisting_memory() {
    let mut types = TypeRegistry::new();
    let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
    let trap = procedure(&mut types, "llvm.debugtrap", &[], &[]);
    let provider = RuntimeProvider::new(types, trap);
    let mut vm = crate::Vm::new(&provider, crate::NoEffects, crate::Limits::default()).unwrap();
    let root = vm
        .memory_mut()
        .allocate(&provider.types, s64, Some(count(42)))
        .unwrap();
    assert_eq!(
        vm.execute(jai_ir::ProcedureId::new(0), vec![]).outcome,
        crate::Outcome::Failed(Error::RuntimeTrap)
    );
    assert_eq!(vm.memory().load(&provider.types, &root).unwrap(), count(42));
}

#[test]
fn swap_charges_zero_sized_aggregate_shape_before_loading_or_writing() {
    let mut types = TypeRegistry::new();
    let empty = types.reserve_record(jai_types::RecordKind::Struct);
    types.define_record(empty, []).unwrap();
    let array = types.fixed_array(empty, 400).unwrap();
    let pointer = types.pointer(array).unwrap();
    let swap = procedure(&mut types, "swap", &[pointer, pointer], &[]);
    let provider = RuntimeProvider::new(types, swap);
    let mut vm = crate::Vm::new(
        &provider,
        crate::NoEffects,
        crate::Limits {
            fuel: 10,
            ..crate::Limits::default()
        },
    )
    .unwrap();
    let value = Value::Array {
        ty: array,
        elements: vec![
            Value::Record {
                ty: empty,
                fields: vec![]
            };
            400
        ],
    };
    let first = vm
        .memory_mut()
        .allocate(&provider.types, array, Some(value.clone()))
        .unwrap();
    let second = vm
        .memory_mut()
        .allocate(&provider.types, array, Some(value.clone()))
        .unwrap();
    let work = swap
        .work_cost_for_target(
            &[
                Value::Pointer(first.clone()),
                Value::Pointer(second.clone()),
            ],
            &provider.types,
            vm.memory(),
        )
        .unwrap();
    assert!(work >= 800);
    let result = vm.execute(
        jai_ir::ProcedureId::new(0),
        vec![
            Value::Pointer(first.clone()),
            Value::Pointer(second.clone()),
        ],
    );
    assert_eq!(
        result.outcome,
        crate::Outcome::Failed(Error::Limit(LimitKind::Fuel))
    );
    assert_eq!(vm.memory().load(&provider.types, &first).unwrap(), value);
    assert_eq!(vm.memory().load(&provider.types, &second).unwrap(), value);
}
