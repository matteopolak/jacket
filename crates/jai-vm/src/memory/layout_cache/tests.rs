use super::*;
use jai_types::*;
use std::cell::Cell;

struct Counted<'a> {
    types: &'a TypeRegistry,
    kinds: Cell<usize>,
    records: Cell<usize>,
}
impl TypeView for Counted<'_> {
    fn kind(&self, id: TypeId) -> Result<&TypeKind, TypeError> {
        self.kinds.set(self.kinds.get() + 1);
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
    fn record(&self, id: RecordId) -> Result<&RecordDefinition, TypeError> {
        self.records.set(self.records.get() + 1);
        self.types.record(id)
    }
    fn enumeration(&self, id: EnumId) -> Result<&EnumDefinition, TypeError> {
        self.types.enumeration(id)
    }
    fn distinct(&self, id: DistinctId) -> Result<&DistinctDefinition, TypeError> {
        self.types.distinct(id)
    }
    fn procedure_type(&self, id: ProcedureTypeId) -> Result<&ProcedureType, TypeError> {
        self.types.procedure_type(id)
    }
    fn record_type(&self, id: RecordId) -> Result<TypeId, TypeError> {
        self.types.record_type(id)
    }
}
fn counted(types: &TypeRegistry) -> Counted<'_> {
    Counted {
        types,
        kinds: Cell::new(0),
        records: Cell::new(0),
    }
}

#[test]
fn repeated_wide_field_hits_share_layout_without_rescanning_record() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let wide = types.reserve_record(RecordKind::Struct);
    types.define_record(wide, vec![word; 4096]).unwrap();
    let view = counted(&types);
    let mut cache = RootLayoutCache::new(LayoutPolicy::lp64());
    let cold = cache.work_cost(&view, wide, 30000, 100000).unwrap();
    assert!(cold > 4096);
    assert_eq!(cache.cells(), 0);
    let first = cache.layout(&view, wide, 30000, 100000).unwrap();
    assert_eq!(cache.cells(), 4097);
    assert_eq!(first.field_offsets[4095], 4095 * 8);
    view.records.set(0);
    view.kinds.set(0);
    for _ in 0..20 {
        assert_eq!(cache.work_cost(&view, wide, 30000, 1).unwrap(), 1);
        let again = cache.layout(&view, wide, 30000, 1).unwrap();
        assert!(Arc::ptr_eq(&first, &again));
    }
    assert_eq!(view.records.get(), 0);
    assert_eq!(view.kinds.get(), 40);
}

#[test]
fn pending_miss_is_not_cached_and_pointer_layout_does_not_demand_pointee() {
    let mut types = TypeRegistry::new();
    let pending = types.reserve_record(RecordKind::Struct);
    let pointer = types.pointer(pending).unwrap();
    let mut cache = RootLayoutCache::new(LayoutPolicy::lp64());
    assert!(
        matches!(cache.layout(&types, pending, 1000, 1000), Err(Error::Type(TypeError::Incomplete(id))) if id == pending)
    );
    assert_eq!(cache.cells(), 0);
    assert_eq!(cache.layout(&types, pointer, 1000, 1000).unwrap().size, 8);
    let before = cache.cells();
    types.define_record(pending, [pointer]).unwrap();
    assert_eq!(cache.layout(&types, pending, 1000, 1000).unwrap().size, 8);
    assert_eq!(cache.cells(), before + 2);
}

#[test]
fn cached_dependencies_still_charge_the_full_cold_root_closure() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let child = types.reserve_record(RecordKind::Struct);
    types.define_record(child, vec![word; 100]).unwrap();
    let parent = types.fixed_array(child, 2).unwrap();
    let mut cache = RootLayoutCache::new(LayoutPolicy::lp64());
    cache.layout(&types, child, 2000, 10000).unwrap();
    assert_eq!(cache.work_cost(&types, child, 2000, 1).unwrap(), 1);
    assert!(cache.work_cost(&types, parent, 2000, 10000).unwrap() > 100);
    assert!(matches!(
        cache.layout(&types, parent, 2000, 10),
        Err(Error::Limit(LimitKind::Fuel))
    ));
    assert_eq!(cache.cells(), 101);
}

#[test]
fn wide_and_distinct_offset_budgets_fail_before_computation_or_retention() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let wide = types.reserve_record(RecordKind::Struct);
    types.define_record(wide, vec![word; 1000]).unwrap();
    let view = counted(&types);
    let mut cache = RootLayoutCache::new(LayoutPolicy::lp64());
    assert!(matches!(
        cache.layout(&view, wide, 10000, 10),
        Err(Error::Limit(LimitKind::Fuel))
    ));
    assert!(view.kinds.get() < 10);
    assert_eq!(cache.cells(), 0);
    let mut child = wide;
    for _ in 0..8 {
        let wrapper = types.reserve_distinct(DistinctKind::Distinct);
        types.define_distinct(wrapper, child).unwrap();
        child = wrapper;
    }
    assert!(matches!(
        cache.layout(&types, child, 3000, 100000),
        Err(Error::Limit(LimitKind::ValueCells))
    ));
    assert_eq!(cache.cells(), 0);
}

#[test]
fn cumulative_retention_and_registry_identity_are_enforced_atomically() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let mut cache = RootLayoutCache::new(LayoutPolicy::lp64());
    for _ in 0..19 {
        let array = types.fixed_array(word, cache.cells() as u64 + 1).unwrap();
        cache.layout(&types, array, 20, 100).unwrap();
    }
    assert_eq!(cache.cells(), 19);
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [word]).unwrap();
    assert!(matches!(
        cache.layout(&types, record, 20, 100),
        Err(Error::Limit(LimitKind::ValueCells))
    ));
    assert_eq!(cache.cells(), 19);
    let other = TypeRegistry::new();
    let other_word = other.scalar(ScalarType::Int(IntegerType::U64));
    assert!(matches!(
        cache.layout(&other, other_word, 1000, 1000),
        Err(Error::InvalidIr(_))
    ));
    assert_eq!(cache.cells(), 19);
}

#[test]
fn policy_and_record_options_match_layout_engine_without_host_assumptions() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer = types.pointer(word).unwrap();
    let record = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_layout(
            record,
            [pointer, word],
            RecordLayout {
                packed: true,
                ..Default::default()
            },
        )
        .unwrap();
    let narrow = LayoutPolicy::new(
        ScalarLayout::new(4, 4),
        [
            ScalarLayout::new(1, 1),
            ScalarLayout::new(2, 2),
            ScalarLayout::new(4, 4),
            ScalarLayout::new(8, 4),
        ],
        [ScalarLayout::new(4, 4), ScalarLayout::new(8, 4)],
        ScalarLayout::new(1, 1),
    )
    .unwrap();
    for policy in [narrow, LayoutPolicy::lp64()] {
        let mut cache = RootLayoutCache::new(policy);
        let got = cache.layout(&types, record, 1000, 1000).unwrap();
        let mut engine = LayoutEngine::new(&types, policy);
        assert_eq!(got.as_ref(), engine.layout(record).unwrap());
        assert_eq!(got.field_offsets[1], policy.pointer().size);
    }
}

#[test]
fn failed_pending_and_exhausted_attempts_report_consumed_work_without_cache_install() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let wide = types.reserve_record(RecordKind::Struct);
    types.define_record(wide, vec![word; 200]).unwrap();
    let pending = types.reserve_record(RecordKind::Struct);
    let root = types.reserve_record(RecordKind::Struct);
    types.define_record(root, [wide, pending]).unwrap();
    let mut cache = RootLayoutCache::new(LayoutPolicy::lp64());
    let (spent, result) = cache.layout_attempt(&types, root, 10000, 10000);
    assert!(matches!(result, Err(Error::Type(TypeError::Incomplete(id))) if id == pending));
    assert!(spent > 200);
    assert_eq!(cache.cells(), 0);
    let (spent, result) = cache.layout_attempt(&types, root, 10000, 1);
    assert_eq!(spent, 1);
    assert!(matches!(result, Err(Error::Limit(LimitKind::Fuel))));
    assert_eq!(cache.cells(), 0);
    types.define_record(pending, []).unwrap();
    let (spent, result) = cache.layout_attempt(&types, root, 10000, 10000);
    assert!(result.is_ok());
    assert!(spent > 200);
    assert_eq!(cache.cells(), 3);
}

#[test]
fn placement_validation_is_bounded_without_charging_extra_cached_offsets() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let sequential = types.reserve_record(RecordKind::Struct);
    types.define_record(sequential, [word; 3]).unwrap();
    let placed = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_placements(
            placed,
            [word; 3],
            RecordLayout::default(),
            [None, Some(0), None],
        )
        .unwrap();
    let baseline = preflight(&types, sequential, 1000, 1000).unwrap();
    let placement = preflight(&types, placed, 1000, 1000).unwrap();
    assert_eq!(placement.work, baseline.work + 6);
    assert_eq!(placement.root_offsets, baseline.root_offsets);
    let mut cache = RootLayoutCache::new(LayoutPolicy::lp64());
    let (spent, result) = cache.layout_attempt(&types, placed, 1000, baseline.work);
    assert_eq!(spent, baseline.work);
    assert!(matches!(result, Err(Error::Limit(LimitKind::Fuel))));
    assert_eq!(cache.cells(), 0);
    let layout = cache.layout(&types, placed, 1000, 1000).unwrap();
    assert_eq!(layout.field_offsets.as_ref(), &[0, 0, 8]);
    assert_eq!(layout.size, 16);
    assert_eq!(cache.cells(), 4);
}

#[test]
fn memory_cache_charges_retention_and_restores_facts_with_transaction_cells() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer = types.pointer(word).unwrap();
    let memory = crate::Memory::new(crate::Limits::default());
    let (spent, result) = memory.prepare_layout(&types, word, 1000);
    result.unwrap();
    assert!(spent > 1);
    assert_eq!(memory.value_cells(), 1);
    assert_eq!(memory.prepare_layout(&types, word, 1).0, 1);
    let snapshot = memory.snapshot();
    memory.prepare_layout(&types, pointer, 1000).1.unwrap();
    assert_eq!(memory.value_cells(), 2);
    let mut memory = memory;
    memory.restore(snapshot);
    assert_eq!(memory.value_cells(), 1);
    assert!(
        memory
            .layout_cache
            .borrow()
            .work_cost(&types, pointer, 1000, 1000)
            .unwrap()
            > 1
    );
    assert_eq!(
        memory
            .layout_cache
            .borrow()
            .work_cost(&types, word, 1000, 1000)
            .unwrap(),
        1
    );
}

#[test]
fn address_only_preparation_does_not_demand_cast_pointee_until_access() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let pending = types.reserve_record(RecordKind::Struct);
    let mut memory = crate::Memory::new(crate::Limits::default());
    let root = memory.allocate(&types, word, None).unwrap();
    let cast = memory
        .cast_pointer(&types, &root, pending, CastMode::Checked)
        .unwrap();
    assert_eq!(
        memory.prepare_pointer_layouts(&types, &cast, false, 1),
        (0, Ok(()))
    );
    let before = memory.value_cells();
    let (spent, result) = memory.prepare_pointer_layouts(&types, &cast, true, 1000);
    assert!(spent > 0);
    assert!(matches!(result, Err(Error::Type(TypeError::Incomplete(id))) if id == pending));
    assert_eq!(memory.value_cells(), before);
}

#[test]
fn memory_cold_fuel_failure_keeps_cache_and_live_charges_unchanged() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let wide = types.reserve_record(RecordKind::Struct);
    types.define_record(wide, vec![word; 4096]).unwrap();
    let memory = crate::Memory::new(crate::Limits::default());
    let (spent, result) = memory.prepare_layout(&types, wide, 10);
    assert_eq!(spent, 10);
    assert!(matches!(result, Err(Error::Limit(LimitKind::Fuel))));
    assert_eq!(memory.value_cells(), 0);
    assert_eq!(memory.allocation_count(), 0);
}

#[test]
fn prepared_layout_never_hides_a_cold_record_scan() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let wide = types.reserve_record(RecordKind::Struct);
    types.define_record(wide, vec![word; 512]).unwrap();
    let view = counted(&types);
    let memory = crate::Memory::new(crate::Limits::default());
    assert!(matches!(
        memory.prepared_layout(&view, wide),
        Err(Error::InvalidIr(_))
    ));
    assert_eq!(memory.value_cells(), 0);
    memory.prepare_layout(&view, wide, 100000).1.unwrap();
    let first = memory.prepared_layout(&view, wide).unwrap();
    let cells = memory.value_cells();
    view.records.set(0);
    view.kinds.set(0);
    for _ in 0..20 {
        assert!(Arc::ptr_eq(
            &first,
            &memory.prepared_layout(&view, wide).unwrap()
        ));
    }
    assert_eq!(view.records.get(), 0);
    assert_eq!(view.kinds.get(), 20);
    assert_eq!(memory.value_cells(), cells);
}

#[test]
fn field_preparation_bounds_a_cold_child_even_when_parent_layout_is_warm() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let child = types.reserve_record(RecordKind::Struct);
    types.define_record(child, vec![word; 512]).unwrap();
    let parent = types.reserve_record(RecordKind::Struct);
    types.define_record(parent, [child]).unwrap();
    let mut memory = crate::Memory::new(crate::Limits::default());
    let root = memory.allocate(&types, parent, None).unwrap();
    let cells = memory.value_cells();
    assert!(memory.prepared_layout(&types, parent).is_ok());
    assert!(memory.prepared_layout(&types, child).is_err());
    let (parent_spent, result) = memory.prepare_pointer_layouts(&types, &root, true, 10);
    result.unwrap();
    let (child_spent, result) = memory.prepare_layout(&types, child, 10 - parent_spent);
    let spent = parent_spent + child_spent;
    assert_eq!(spent, 10);
    assert!(matches!(result, Err(Error::Limit(LimitKind::Fuel))));
    assert_eq!(memory.value_cells(), cells);
    assert!(memory.prepared_layout(&types, child).is_err());
    memory
        .prepare_pointer_layouts(&types, &root, true, 100000)
        .1
        .unwrap();
    memory.prepare_layout(&types, child, 100000).1.unwrap();
    assert_eq!(
        memory
            .prepared_layout(&types, child)
            .unwrap()
            .field_offsets
            .len(),
        512
    );
}

#[test]
fn null_field_projection_does_not_demand_a_pending_nominal_definition() {
    let mut types = TypeRegistry::new();
    let pending = types.reserve_record(RecordKind::Struct);
    let memory = crate::Memory::new(crate::Limits::default());
    let pointer = crate::Pointer::null(pending);
    assert_eq!(
        memory.prepare_field_layouts(&types, &pointer, 0, 0),
        (0, Err(Error::NullPointer))
    );
    assert_eq!(memory.value_cells(), 0);
}

#[test]
fn prepared_decoded_shape_is_constant_work_for_wide_nested_aggregates() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let wide = types.reserve_record(RecordKind::Struct);
    types.define_record(wide, vec![word; 512]).unwrap();
    let array = types.fixed_array(wide, 32).unwrap();
    let union = types.reserve_record(RecordKind::Union);
    types.define_record(union, [word, array]).unwrap();
    let view = counted(&types);
    let memory = crate::Memory::new(crate::Limits::default());
    assert!(matches!(
        memory.prepared_decoded_cells(&view, union),
        Err(Error::InvalidIr(_))
    ));
    memory.prepare_layout(&view, union, 100_000).1.unwrap();
    let retained = memory.value_cells();
    view.records.set(0);
    view.kinds.set(0);
    for _ in 0..64 {
        assert_eq!(
            memory.prepared_decoded_cells(&view, union).unwrap(),
            2 + 32 * 513
        );
    }
    assert_eq!(view.records.get(), 0);
    assert_eq!(view.kinds.get(), 64);
    assert_eq!(memory.value_cells(), retained);
}

#[test]
fn enormous_zero_size_arrays_keep_address_layout_but_reject_decoded_cells() {
    let mut types = TypeRegistry::new();
    let empty = types.reserve_record(RecordKind::Struct);
    types.define_record(empty, []).unwrap();
    let giant = types.fixed_array(empty, u64::MAX).unwrap();
    let memory = crate::Memory::new(crate::Limits::default());
    memory.prepare_layout(&types, giant, 1_000).1.unwrap();
    assert_eq!(memory.prepared_layout(&types, giant).unwrap().size, 0);
    assert_eq!(
        memory.prepared_decoded_cells(&types, giant),
        Err(Error::Limit(LimitKind::ValueCells))
    );
    assert_eq!(memory.value_cells(), 1);
}

#[test]
fn decoded_shape_bounds_height_without_rejecting_address_preparation() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let mut deep = word;
    for _ in 0..8 {
        deep = types.fixed_array(deep, 1).unwrap();
    }
    let memory = crate::Memory::new(crate::Limits {
        evaluation_depth: 4,
        ..crate::Limits::default()
    });
    memory.prepare_layout(&types, deep, 1_000).1.unwrap();
    assert_eq!(memory.prepared_layout(&types, deep).unwrap().size, 8);
    assert_eq!(
        memory.prepared_decoded_cells(&types, deep),
        Err(Error::Limit(LimitKind::EvaluationDepth))
    );
}

#[test]
fn codec_closure_counts_zero_array_children_and_repeated_field_edges() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let wide = types.reserve_record(RecordKind::Struct);
    types.define_record(wide, vec![word; 4_096]).unwrap();
    let empty = types.fixed_array(wide, 0).unwrap();
    let repeated = types.reserve_record(RecordKind::Struct);
    types.define_record(repeated, [empty, empty]).unwrap();
    let pointer = types.pointer(wide).unwrap();
    let slice = types.slice(wide).unwrap();
    let memory = crate::Memory::new(crate::Limits::default());
    assert!(matches!(
        memory.prepared_codec_layout_work(&types, empty),
        Err(Error::InvalidIr(_))
    ));
    for ty in [empty, repeated, pointer, slice] {
        memory.prepare_layout(&types, ty, 100_000).1.unwrap();
    }
    assert_eq!(memory.prepared_decoded_cells(&types, empty).unwrap(), 1);
    assert_eq!(
        memory.prepared_codec_layout_work(&types, empty).unwrap(),
        4 * (1 + 1 + 2 * 4_096)
    );
    // The shared empty-array definition is scanned once; both incoming record
    // edges still contribute work. Descriptors do not demand their pointee.
    assert_eq!(
        memory.prepared_codec_layout_work(&types, repeated).unwrap(),
        4 * (1 + 4 + 1 + 2 * 4_096)
    );
    assert_eq!(
        memory.prepared_codec_layout_work(&types, pointer).unwrap(),
        4
    );
    assert_eq!(memory.prepared_codec_layout_work(&types, slice).unwrap(), 4);
}

#[test]
fn warm_force_cost_uses_cached_zero_array_closure_without_record_visits() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let wide = types.reserve_record(RecordKind::Struct);
    types.define_record(wide, vec![word; 4_096]).unwrap();
    let source = types.fixed_array(word, 0).unwrap();
    let target = types.fixed_array(wide, 0).unwrap();
    let cast = StorageBitcast::prove(
        &types,
        LayoutPolicy::lp64(),
        source,
        target,
        StorageBitcastStrength::EqualSize,
    )
    .unwrap();
    let view = counted(&types);
    let memory = crate::Memory::new(crate::Limits::default());
    memory.prepare_layout(&view, source, 100_000).1.unwrap();
    memory.prepare_layout(&view, target, 100_000).1.unwrap();
    let value = crate::Value::Array {
        ty: source,
        elements: vec![],
    };
    let baseline = memory.value_cells();
    view.records.set(0);
    for _ in 0..64 {
        assert_eq!(
            memory.prepared_codec_layout_work(&view, target).unwrap(),
            32_776
        );
        let work = memory
            .storage_bitcast_value_work_cost(&view, &value, cast)
            .unwrap();
        assert!(work >= 3 * 32_776);
    }
    assert_eq!(view.records.get(), 0);
    assert_eq!(memory.value_cells(), baseline);
}

fn bind_allocator(types: &mut TypeRegistry) -> AllocatorSchema {
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
            convention: CallingConvention::Jai,
            context: ContextMode::Implicit,
            variadic: Variadic::None,
        })
        .unwrap();
    let allocator = types.reserve_record(RecordKind::Struct);
    types.define_record(allocator, [procedure, data]).unwrap();
    types.bind_allocator(allocator, mode).unwrap()
}

#[test]
fn dynamic_allocator_role_stales_only_by_value_dependent_cache_facts() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let dynamic = types.dynamic_array(byte).unwrap();
    let empty = types.fixed_array(dynamic, 0).unwrap();
    let pointer = types.pointer(dynamic).unwrap();
    let slice = types.slice(dynamic).unwrap();
    let memory = crate::Memory::new(crate::Limits::default());
    for ty in [byte, dynamic, empty, pointer, slice] {
        memory.prepare_layout(&types, ty, 10_000).1.unwrap();
    }
    let baseline = memory.value_cells();
    bind_allocator(&mut types);
    for ty in [dynamic, empty] {
        assert!(matches!(
            memory.prepared_layout(&types, ty),
            Err(Error::InvalidIr(_))
        ));
        assert!(matches!(
            memory.prepared_decoded_cells(&types, ty),
            Err(Error::InvalidIr(_))
        ));
        assert!(matches!(
            memory.prepared_codec_layout_work(&types, ty),
            Err(Error::InvalidIr(_))
        ));
        assert!(matches!(
            memory.prepare_layout(&types, ty, 10_000).1,
            Err(Error::InvalidIr(_))
        ));
    }
    for ty in [byte, pointer, slice] {
        memory.prepared_layout(&types, ty).unwrap();
        memory.prepared_decoded_cells(&types, ty).unwrap();
        memory.prepared_codec_layout_work(&types, ty).unwrap();
    }
    assert_eq!(memory.value_cells(), baseline);
}

#[test]
fn dynamic_allocator_ready_facts_include_typed_payload_without_pointee_walk() {
    let mut types = TypeRegistry::new();
    let pending = types.reserve_record(RecordKind::Struct);
    let dynamic = types.dynamic_array(pending).unwrap();
    bind_allocator(&mut types);
    let memory = crate::Memory::new(crate::Limits::default());
    memory.prepare_layout(&types, dynamic, 10_000).1.unwrap();
    assert_eq!(memory.prepared_decoded_cells(&types, dynamic).unwrap(), 4);
    assert_eq!(
        memory.prepared_codec_layout_work(&types, dynamic).unwrap(),
        24
    );
    assert_eq!(memory.prepared_layout(&types, dynamic).unwrap().size, 40);
}
