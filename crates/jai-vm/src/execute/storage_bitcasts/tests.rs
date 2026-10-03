use super::*;
use crate::NoEffects;
use jai_types::{
    CallingConvention, ContextMode, FloatType, FloatValue, IntegerType, LayoutPolicy, RecordKind,
    ScalarType, StorageBitcastStrength, TypeRegistry, Variadic,
};
use std::{cell::Cell, collections::HashMap};

struct Provider<T = TypeRegistry> {
    types: T,
}
impl<T: TypeView> ProcedureProvider for Provider<T> {
    fn types(&self) -> &dyn TypeView {
        &self.types
    }
    fn procedure(&self, _: ProcedureId) -> ProcedureAvailability<'_> {
        ProcedureAvailability::Missing
    }
}
fn provider() -> Provider {
    Provider {
        types: TypeRegistry::new(),
    }
}

struct CountedTypes<'a> {
    types: &'a TypeRegistry,
    records: Cell<usize>,
}
impl TypeView for CountedTypes<'_> {
    fn kind(&self, id: TypeId) -> std::result::Result<&TypeKind, jai_types::TypeError> {
        self.types.kind(id)
    }
    fn lookup(&self, kind: &TypeKind) -> Option<TypeId> {
        self.types.lookup(kind)
    }
    fn scalar(&self, ty: ScalarType) -> TypeId {
        self.types.scalar(ty)
    }
    fn float(&self, ty: FloatType) -> TypeId {
        self.types.float(ty)
    }
    fn any_type(&self) -> Option<TypeId> {
        self.types.any_type()
    }
    fn allocator_schema(&self) -> Option<jai_types::AllocatorSchema> {
        self.types.allocator_schema()
    }
    fn record(
        &self,
        id: jai_types::RecordId,
    ) -> std::result::Result<&jai_types::RecordDefinition, jai_types::TypeError> {
        self.records.set(self.records.get() + 1);
        self.types.record(id)
    }
    fn enumeration(
        &self,
        id: jai_types::EnumId,
    ) -> std::result::Result<&jai_types::EnumDefinition, jai_types::TypeError> {
        self.types.enumeration(id)
    }
    fn distinct(
        &self,
        id: jai_types::DistinctId,
    ) -> std::result::Result<&jai_types::DistinctDefinition, jai_types::TypeError> {
        self.types.distinct(id)
    }
    fn procedure_type(
        &self,
        id: jai_types::ProcedureTypeId,
    ) -> std::result::Result<&jai_types::ProcedureType, jai_types::TypeError> {
        self.types.procedure_type(id)
    }
    fn record_type(
        &self,
        id: jai_types::RecordId,
    ) -> std::result::Result<TypeId, jai_types::TypeError> {
        self.types.record_type(id)
    }
}
fn cast(types: &dyn TypeView, source: TypeId, target: TypeId, prefix: bool) -> StorageBitcast {
    StorageBitcast::prove(
        types,
        LayoutPolicy::lp64(),
        source,
        target,
        if prefix {
            StorageBitcastStrength::Prefix
        } else {
            StorageBitcastStrength::EqualSize
        },
    )
    .unwrap()
}
fn unsigned(bits: u64) -> ValueExpr {
    ValueExpr::Int(IntExpr::constant(Integer::wrapping(
        IntegerType::U64,
        i128::from(bits),
    )))
}
fn both(provider: &impl ProcedureProvider, expression: &ValueExpr, expected: Value) {
    let mut vm = Vm::new(provider, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.evaluate(expression).outcome,
        Outcome::Complete(vec![expected.clone()])
    );
    let mut vm = Vm::new(provider, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.start_resumable_expression(expression).outcome,
        ResumableOutcome::AwaitingPublication
    );
    assert_eq!(vm.resumable_values(), Some([expected.clone()].as_slice()));
    assert_eq!(
        vm.finish_resumable_validated(|_, _| Ok(())).outcome,
        Outcome::Complete(vec![expected])
    );
}

struct PendingProvider {
    library: Library,
    pending: Cell<bool>,
}
impl ProcedureProvider for PendingProvider {
    fn types(&self) -> &dyn TypeView {
        self.library.types()
    }
    fn signatures(&self) -> &HashMap<ProcedureId, TypeId> {
        self.library.signatures()
    }
    fn globals(&self) -> &[Global] {
        self.library.globals()
    }
    fn places(&self) -> Option<&Places> {
        Some(self.library.places())
    }
    fn procedure(&self, id: ProcedureId) -> ProcedureAvailability<'_> {
        if id == ProcedureId::new(2) && self.pending.get() {
            ProcedureAvailability::Pending(Dependency::Procedure(id))
        } else {
            self.library
                .checked_procedure(id)
                .map_or(ProcedureAvailability::Missing, ProcedureAvailability::Ready)
        }
    }
}

#[test]
fn a_resumed_storage_cast_does_not_replay_completed_source_effects() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let float = types.float(FloatType::F64);
    let proof = cast(&types, word, float, false);
    let signature = types
        .procedure(jai_types::ProcedureType {
            parameters: Box::new([]),
            results: Box::new([word]),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let global = Global::new(
        0,
        GlobalInitializer::Int(Integer::wrapping(IntegerType::U64, 0)),
        &types,
    );
    let trace = IntPlace::try_from_place(global.place(), &types).unwrap();
    let bits = 0x7ff8_1234_5678_9abc;
    let procedure = |id, statements| Procedure {
        id: ProcedureId::new(id),
        signature,
        parameters: vec![],
        locals: vec![],
        body: Block {
            statements,
            flow: Flow::Terminates,
        },
        cleanups: vec![],
    };
    let ret = |value| {
        Statement::Exit(Exit {
            cleanups: vec![],
            transfer: Transfer::ReturnValues(vec![unsigned(value)]),
        })
    };
    let first = procedure(
        1,
        vec![
            Statement::StoreInt(
                trace,
                IntExpr::new(
                    IntegerType::U64,
                    IntExprKind::Binary(
                        IntOp::Add,
                        Box::new(IntExpr::load(trace)),
                        Box::new(IntExpr::constant(Integer::wrapping(IntegerType::U64, 1))),
                    ),
                ),
            ),
            ret(bits),
        ],
    );
    let wait = procedure(2, vec![ret(0)]);
    let call = |id| {
        IntExpr::new(
            IntegerType::U64,
            IntExprKind::Call(Call::new(ProcedureId::new(id), vec![])),
        )
    };
    let expression = ValueExpr::StorageBitcast {
        source: StorageBitcastSource::Value(Box::new(ValueExpr::Int(IntExpr::new(
            IntegerType::U64,
            IntExprKind::Binary(IntOp::BitOr, Box::new(call(1)), Box::new(call(2))),
        )))),
        cast: proof,
    };
    let provider = PendingProvider {
        library: ProgramBuilder::new(types.freeze().unwrap())
            .globals(vec![global])
            .procedures(vec![first, wait])
            .finish_library()
            .unwrap(),
        pending: Cell::new(true),
    };
    let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.start_resumable_expression(&expression).outcome,
        ResumableOutcome::Suspended(vec![Dependency::Procedure(ProcedureId::new(2))])
    );
    let read_trace = |vm: &Vm<'_, PendingProvider, NoEffects>| {
        vm.memory
            .load(provider.types(), vm.globals[0].as_ref().unwrap())
            .unwrap()
    };
    let one = Value::Int(Integer::wrapping(IntegerType::U64, 1));
    assert_eq!(read_trace(&vm), one);
    provider.pending.set(false);
    assert_eq!(
        vm.resume_resumable().outcome,
        ResumableOutcome::AwaitingPublication
    );
    assert_eq!(read_trace(&vm), one);
    assert_eq!(
        vm.finish_resumable_validated(|_, _| Ok(())).outcome,
        Outcome::Complete(vec![Value::Float(FloatValue::F64(bits))])
    );
    let mut synchronous = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        synchronous.evaluate(&expression).outcome,
        Outcome::Complete(vec![Value::Float(FloatValue::F64(bits))])
    );
    assert_eq!(read_trace(&synchronous), one);
}

#[test]
fn numeric_storage_bitcasts_preserve_nan_payloads_in_both_execution_paths() {
    let provider = provider();
    let word = provider.types.scalar(ScalarType::Int(IntegerType::U64));
    let float = provider.types.float(FloatType::F64);
    let bits = 0x7ff8_1234_5678_9abc;
    let expression = ValueExpr::StorageBitcast {
        source: StorageBitcastSource::Value(Box::new(unsigned(bits))),
        cast: cast(&provider.types, word, float, false),
    };
    both(&provider, &expression, Value::Float(FloatValue::F64(bits)));
}

#[test]
fn prefix_place_cast_does_not_load_an_unwritten_source_tail() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let array = types.fixed_array(word, 2).unwrap();
    let proof = cast(&types, array, word, true);
    let signature = types
        .procedure(jai_types::ProcedureType {
            parameters: Box::new([]),
            results: Box::new([word]),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let local = Local::new_typed(ProcedureId::new(0), 0, array, &types).unwrap();
    let mut places = PlaceRegistry::new();
    let first = places
        .index(
            local.place(),
            IntExpr::constant(Integer::wrapping(IntegerType::S64, 0)),
            &types,
        )
        .unwrap();
    let main = Procedure {
        id: ProcedureId::new(0),
        signature,
        parameters: vec![],
        locals: vec![local],
        body: Block {
            statements: vec![
                Statement::Store(first, unsigned(42)),
                Statement::Exit(Exit {
                    cleanups: vec![],
                    transfer: Transfer::ReturnValues(vec![ValueExpr::StorageBitcast {
                        source: StorageBitcastSource::Place(local.place()),
                        cast: proof,
                    }]),
                }),
            ],
            flow: Flow::Terminates,
        },
        cleanups: vec![],
    };
    let library = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![main])
        .places(places.freeze())
        .finish_library()
        .unwrap();
    let expected = Value::Int(Integer::wrapping(IntegerType::U64, 42));
    let mut vm = Vm::new(&library, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.execute(ProcedureId::new(0), vec![]).outcome,
        Outcome::Complete(vec![expected.clone()])
    );
    let mut vm = Vm::new(&library, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.start_resumable_procedure(ProcedureId::new(0), vec![])
            .outcome,
        ResumableOutcome::AwaitingPublication
    );
    assert_eq!(
        vm.finish_resumable_validated(|_, _| Ok(())).outcome,
        Outcome::Complete(vec![expected])
    );
}

#[test]
fn string_rvalues_are_reinterpreted_as_descriptors_in_both_execution_paths() {
    let provider = provider();
    let word = provider.types.scalar(ScalarType::Int(IntegerType::S64));
    let string = provider.types.string();
    let expression = ValueExpr::StorageBitcast {
        source: StorageBitcastSource::Value(Box::new(ValueExpr::StringBytes {
            ty: string,
            bytes: b"abc".to_vec(),
        })),
        cast: cast(&provider.types, string, word, true),
    };
    both(
        &provider,
        &expression,
        Value::Int(Integer::wrapping(IntegerType::S64, 3)),
    );
}

#[test]
fn storage_bitcasts_reject_noncanonical_boolean_bytes() {
    let provider = provider();
    let byte = provider.types.scalar(ScalarType::Int(IntegerType::U8));
    let boolean = provider.types.scalar(ScalarType::Bool);
    let expression = ValueExpr::StorageBitcast {
        source: StorageBitcastSource::Value(Box::new(ValueExpr::Int(IntExpr::constant(
            Integer::wrapping(IntegerType::U8, 2),
        )))),
        cast: cast(&provider.types, byte, boolean, false),
    };
    let expected = Error::UnsupportedPointerOperation(
        "storage cast contains an invalid boolean representation",
    );
    let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.evaluate(&expression).outcome,
        Outcome::Failed(expected.clone())
    );
    let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.start_resumable_expression(&expression).outcome,
        ResumableOutcome::Failed(expected)
    );
}

#[test]
fn padded_aggregate_rvalues_keep_bytes_for_a_later_storage_roundtrip() {
    let mut provider = provider();
    let byte = provider.types.scalar(ScalarType::Int(IntegerType::U8));
    let word = provider.types.scalar(ScalarType::Int(IntegerType::U64));
    let array = provider.types.fixed_array(byte, 16).unwrap();
    let record = provider.types.reserve_record(RecordKind::Struct);
    provider.types.define_record(record, [byte, word]).unwrap();
    let original = Value::Array {
        ty: array,
        elements: (0..16)
            .map(|value| Value::Int(Integer::wrapping(IntegerType::U8, value)))
            .collect(),
    };
    let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
    let intermediate = vm
        .storage_bitcast_value(
            original.clone(),
            cast(&provider.types, array, record, false),
            0,
        )
        .unwrap();
    assert!(matches!(intermediate, Value::StoredAggregate(_)));
    let roundtrip = vm
        .storage_bitcast_value(intermediate, cast(&provider.types, record, array, false), 0)
        .unwrap();
    assert_eq!(roundtrip.semantic(), &original);
    assert_eq!(vm.materialize_value(&roundtrip).unwrap(), original);
}

#[test]
fn pointer_to_integer_storage_cast_keeps_the_original_allocation_receipt() {
    let mut provider = provider();
    let word = provider.types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer_type = provider.types.pointer(word).unwrap();
    let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
    let pointer = vm
        .memory
        .allocate(
            &provider.types,
            word,
            Some(Value::Int(Integer::wrapping(IntegerType::U64, 42))),
        )
        .unwrap();
    let converted = vm
        .storage_bitcast_value(
            Value::Pointer(pointer.clone()),
            cast(&provider.types, pointer_type, word, false),
            0,
        )
        .unwrap();
    let Value::AddressInteger(number) = &converted else {
        panic!("address integer must retain provenance")
    };
    assert!(matches!(
        number.provenance(),
        Some(crate::AddressProvenance::Pointer(actual)) if actual == &pointer
    ));
    let roundtrip = vm
        .storage_bitcast_value(
            converted,
            cast(&provider.types, word, pointer_type, false),
            0,
        )
        .unwrap();
    assert_eq!(roundtrip, Value::Pointer(pointer));
}

#[test]
fn zero_stride_destination_expansion_is_charged_before_decoding() {
    let mut provider = provider();
    let empty = provider.types.reserve_record(RecordKind::Struct);
    provider.types.define_record(empty, []).unwrap();
    let large = provider.types.fixed_array(empty, 100_000).unwrap();
    let mut vm = Vm::new(
        &provider,
        NoEffects,
        Limits {
            fuel: 32,
            ..Limits::default()
        },
    )
    .unwrap();
    assert!(matches!(
        vm.storage_bitcast_value(
            Value::Record {
                ty: empty,
                fields: vec![]
            },
            cast(&provider.types, empty, large, false),
            0,
        ),
        Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
    ));
    assert!(vm.root_temporaries.is_empty());
    assert_eq!(vm.memory.allocation_count(), 0);
}

#[test]
fn warm_zero_count_wide_cast_charges_codec_layout_work_before_revisiting_record_fields() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let source = types.fixed_array(word, 0).unwrap();
    let wide = types.reserve_record(RecordKind::Struct);
    types.define_record(wide, vec![word; 4096]).unwrap();
    let target = types.fixed_array(wide, 0).unwrap();
    let proof = cast(&types, source, target, false);
    let provider = Provider {
        types: CountedTypes {
            types: &types,
            records: Cell::new(0),
        },
    };
    let mut vm = Vm::new(
        &provider,
        NoEffects,
        Limits {
            fuel: 16,
            ..Limits::default()
        },
    )
    .unwrap();
    // Cache facts without spending this execution's tiny fuel allowance.
    for ty in [source, target] {
        vm.memory
            .prepare_layout(provider.types(), ty, 100_000)
            .1
            .unwrap();
    }
    provider.types.records.set(0);
    assert!(matches!(
        vm.storage_bitcast_value(
            Value::Array {
                ty: source,
                elements: vec![]
            },
            proof,
            0,
        ),
        Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
    ));
    assert_eq!(provider.types.records.get(), 0);
    assert_eq!(vm.memory.allocation_count(), 0);
    assert!(vm.root_temporaries.is_empty());
}
