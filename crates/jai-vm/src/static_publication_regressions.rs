use super::*;
use std::{cell::Cell, sync::Arc};

struct CountedTypes<'a> {
    registry: &'a TypeRegistry,
    queries: Cell<usize>,
}
impl TypeView for CountedTypes<'_> {
    fn kind(&self, ty: TypeId) -> std::result::Result<&jai_types::TypeKind, jai_types::TypeError> {
        self.queries.set(self.queries.get() + 1);
        self.registry.kind(ty)
    }
    fn lookup(&self, kind: &jai_types::TypeKind) -> Option<TypeId> {
        self.registry.lookup(kind)
    }
    fn scalar(&self, ty: ScalarType) -> TypeId {
        self.registry.scalar(ty)
    }
    fn float(&self, ty: jai_types::FloatType) -> TypeId {
        self.registry.float(ty)
    }
    fn any_type(&self) -> Option<TypeId> {
        self.registry.any_type()
    }
    fn runtime_type_header(&self) -> Option<TypeId> {
        self.registry.runtime_type_header()
    }
    fn record(
        &self,
        id: jai_types::RecordId,
    ) -> std::result::Result<&jai_types::RecordDefinition, jai_types::TypeError> {
        self.registry.record(id)
    }
    fn enumeration(
        &self,
        id: jai_types::EnumId,
    ) -> std::result::Result<&jai_types::EnumDefinition, jai_types::TypeError> {
        self.registry.enumeration(id)
    }
    fn distinct(
        &self,
        id: jai_types::DistinctId,
    ) -> std::result::Result<&jai_types::DistinctDefinition, jai_types::TypeError> {
        self.registry.distinct(id)
    }
    fn procedure_type(
        &self,
        id: jai_types::ProcedureTypeId,
    ) -> std::result::Result<&ProcedureType, jai_types::TypeError> {
        self.registry.procedure_type(id)
    }
    fn record_type(
        &self,
        id: jai_types::RecordId,
    ) -> std::result::Result<TypeId, jai_types::TypeError> {
        self.registry.record_type(id)
    }
}
struct CountedProvider<'a> {
    fixture: &'a Fixture,
    types: CountedTypes<'a>,
}
impl ProcedureProvider for CountedProvider<'_> {
    fn types(&self) -> &dyn TypeView {
        &self.types
    }
    fn signatures(&self) -> &std::collections::HashMap<ProcedureId, TypeId> {
        &self.fixture.signatures
    }
    fn procedure(&self, id: ProcedureId) -> ProcedureAvailability<'_> {
        self.fixture.procedure(id)
    }
    fn globals(&self) -> &[Global] {
        &self.fixture.globals
    }
    fn places(&self) -> Option<&Places> {
        Some(&self.fixture.places)
    }
}

fn define_integer(builder: &mut StaticDataBuilder, f: &Fixture, number: i128) -> StaticObjectId {
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let id = builder.reserve(integer, &f.types).unwrap();
    builder
        .define(
            id,
            StaticValue::constant(ConstantValue {
                ty: integer,
                kind: ConstantKind::Int(Integer::wrapping(IntegerType::S64, number)),
            }),
        )
        .unwrap();
    id
}

fn address(data: Arc<StaticData>, object: StaticObjectId, ty: TypeId) -> ValueExpr {
    ValueExpr::StaticAddress {
        data,
        address: StaticAddress::new(object),
        ty,
    }
}

fn complete_pointer(result: Execution) -> Pointer {
    let Outcome::Complete(values) = result.outcome else {
        panic!("{result:?}");
    };
    values[0].pointer().unwrap().clone()
}

fn repeated_static_read_fixture(objects: usize) -> (Fixture, ValueExpr) {
    let mut f = fixture();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer = f.types.pointer(integer).unwrap();
    let mut builder = StaticDataBuilder::new();
    let root = define_integer(&mut builder, &f, 42);
    for number in 1..objects {
        define_integer(&mut builder, &f, number as i128);
    }
    let data = Arc::new(
        builder
            .finish(&f.types, StaticDataLimits::default())
            .unwrap(),
    );
    let expression = address(data, root, pointer);
    let iterator = local(&f, 0, 0);
    let p = procedure(
        &mut f,
        0,
        vec![],
        vec![iterator.local()],
        vec![
            Statement::Range(RangeLoop {
                id: LoopId::new(0),
                iterator,
                start: int(0),
                end: int(31),
                direction: Direction::Forward,
                body: block(vec![Statement::DiscardValue(expression.clone())]),
            }),
            ret(int(42)),
        ],
    );
    f.procedures.push(p);
    (f, expression)
}

#[test]
fn repeated_static_reads_in_a_body_do_not_rescan_cached_graph_objects() {
    let mut work = vec![];
    let mut type_queries = vec![];
    for objects in [1, 64] {
        let (f, expression) = repeated_static_read_fixture(objects);
        let provider = CountedProvider {
            fixture: &f,
            types: CountedTypes {
                registry: &f.types,
                queries: Cell::new(0),
            },
        };
        let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
        let publication = vm.evaluate(&expression);
        assert!(publication.statistics.steps >= 4 * objects as u64);
        complete_pointer(publication);
        assert_eq!(vm.memory().allocation_count(), objects);
        provider.types.queries.set(0);
        let (_, checkpoint_work, _) = vm.test_snapshot_costs();
        let result = vm.execute(ProcedureId::new(0), vec![]);
        assert_eq!(result.outcome, Outcome::Complete(vec![value(42)]));
        assert!(result.statistics.steps >= checkpoint_work);
        work.push(result.statistics.steps - checkpoint_work);
        type_queries.push(provider.types.queries.get());
        assert_eq!(vm.memory().allocation_count(), objects);
    }
    assert_eq!(work[0], work[1]);
    assert_eq!(type_queries[0], type_queries[1]);
}

#[test]
fn appended_static_suffix_resolves_forward_references_and_preserves_older_snapshots() {
    let mut f = fixture();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer = f.types.pointer(integer).unwrap();
    let pointer_pointer = f.types.pointer(pointer).unwrap();
    let mut builder = StaticDataBuilder::new();
    let first = define_integer(&mut builder, &f, 7);
    let original = Arc::new(
        builder
            .publish(&f.types, StaticDataLimits::default())
            .unwrap(),
    );
    let original_expression = address(Arc::clone(&original), first, pointer);
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    let first_pointer = complete_pointer(vm.evaluate(&original_expression));
    let referent = builder.reserve(pointer, &f.types).unwrap();
    let final_object = define_integer(&mut builder, &f, 9);
    builder
        .define(
            referent,
            StaticValue {
                ty: pointer,
                kind: StaticValueKind::Address(StaticAddress::new(final_object)),
            },
        )
        .unwrap();
    let extended = Arc::new(
        builder
            .publish(&f.types, StaticDataLimits::default())
            .unwrap(),
    );
    let extended_expression = address(extended, referent, pointer_pointer);
    let holder = complete_pointer(vm.evaluate(&extended_expression));
    let referent = vm.memory().load(&f.types, &holder).unwrap();
    assert_eq!(
        vm.memory()
            .load(&f.types, referent.pointer().unwrap())
            .unwrap(),
        value(9)
    );
    assert_eq!(vm.memory().allocation_count(), 3);
    let old = vm.evaluate(&original_expression);
    assert_eq!(
        old.outcome,
        Outcome::Complete(vec![Value::Pointer(first_pointer.clone())])
    );
    let old_work = old.statistics.steps;
    let mut restored = Vm::with_state(&f, NoEffects, Limits::default(), vm.into_state()).unwrap();
    let restored_old = restored.evaluate(&original_expression);
    assert_eq!(
        restored_old.outcome,
        Outcome::Complete(vec![Value::Pointer(first_pointer)])
    );
    assert_eq!(restored_old.statistics.steps, old_work);
    complete_pointer(restored.evaluate(&extended_expression));
    assert_eq!(restored.memory().allocation_count(), 3);
}

#[test]
fn fuel_exhaustion_initializing_a_suffix_restores_the_prior_publication() {
    let mut f = fixture();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer = f.types.pointer(integer).unwrap();
    let mut builder = StaticDataBuilder::new();
    let first = define_integer(&mut builder, &f, 7);
    let original = Arc::new(
        builder
            .publish(&f.types, StaticDataLimits::default())
            .unwrap(),
    );
    let original_expression = address(original, first, pointer);
    // Establish the prior publication; the bounded attempts below retain the
    // original 32 source-work credit plus their measured rollback/inspection.
    let limits = Limits::default();
    let mut vm = Vm::new(&f, NoEffects, limits).unwrap();
    let first_pointer = complete_pointer(vm.evaluate(&original_expression));
    for number in 0..9 {
        define_integer(&mut builder, &f, number);
    }
    let extended = Arc::new(
        builder
            .publish(&f.types, StaticDataLimits::default())
            .unwrap(),
    );
    let extended_expression = address(extended, first, pointer);
    for _ in 0..2 {
        vm.test_budget_source_work(32, true, 0);
        assert_eq!(
            vm.evaluate(&extended_expression).outcome,
            Outcome::Failed(Error::Limit(LimitKind::Fuel))
        );
        assert_eq!(vm.memory().allocation_count(), 1);
        vm.test_budget_source_work(32, true, 0);
        assert_eq!(
            vm.evaluate(&original_expression).outcome,
            Outcome::Complete(vec![Value::Pointer(first_pointer.clone())])
        );
        assert_eq!(
            vm.memory().load(&f.types, &first_pointer).unwrap(),
            value(7)
        );
    }
}

#[test]
fn cached_static_address_projections_charge_each_step() {
    let mut f = fixture();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let array = f.types.fixed_array(integer, 4).unwrap();
    let pointer = f.types.pointer(integer).unwrap();
    let array_pointer = f.types.pointer(array).unwrap();
    let mut builder = StaticDataBuilder::new();
    let object = builder.reserve(array, &f.types).unwrap();
    builder
        .define(
            object,
            StaticValue {
                ty: array,
                kind: StaticValueKind::Array(
                    (0..4)
                        .map(|number| {
                            StaticValue::constant(ConstantValue {
                                ty: integer,
                                kind: ConstantKind::Int(Integer::wrapping(
                                    IntegerType::S64,
                                    number,
                                )),
                            })
                        })
                        .collect(),
                ),
            },
        )
        .unwrap();
    let data = Arc::new(
        builder
            .finish(&f.types, StaticDataLimits::default())
            .unwrap(),
    );
    let root = address(Arc::clone(&data), object, array_pointer);
    let projection = ValueExpr::StaticAddress {
        data,
        address: StaticAddress::new(object).project(StaticProjection::Index(3)),
        ty: pointer,
    };
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    complete_pointer(vm.evaluate(&root));
    let root_work = vm.evaluate(&root).statistics.steps;
    let projected = vm.evaluate(&projection);
    // The projection and its warm enclosing target-layout lookup each cost one.
    assert_eq!(projected.statistics.steps, root_work + 2);
    assert_eq!(
        vm.memory()
            .load(&f.types, &complete_pointer(projected))
            .unwrap(),
        value(3)
    );
}
