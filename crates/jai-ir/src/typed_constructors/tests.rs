use super::*;
use jai_source::{Identities, SourceMap, SourceSpan, Span};
use jai_types::{ScalarType, TypeRegistry};

fn source(sources: &mut SourceMap, declaration: DeclarationId) -> TypedConstructorSource {
    let text = "New :: ($T: Type) -> *T {}";
    let id = sources.insert("Basic/allocation.jai".into(), text.into());
    TypedConstructorSource::from_checked_source(
        TypedConstructorOwner::ModuleMacro {
            declaration,
        },
        SourceProcedureIdentity::new(
            sources.get(id).unwrap(),
            SourceSpan {
                source: id,
                span: Span::new(0, text.len()),
            },
        )
        .unwrap(),
    )
}
fn storage(types: &mut TypeRegistry) -> (TypeId, TypeId) {
    let ty = types.scalar(ScalarType::Bool);
    (ty, types.pointer(ty).unwrap())
}
fn uninitialized(
    types: &TypeRegistry,
    source: TypedConstructorSource,
    ty: TypeId,
    pointer: TypeId,
) -> CheckedTypedConstructorReceipt {
    CheckedTypedConstructorReceipt::from_checked_specialization(
        source,
        ty,
        pointer,
        TypedConstructorInitialization::Uninitialized,
        LayoutPolicy::lp64(),
        ByteOrder::Little,
        types,
    )
    .unwrap()
}
fn step(
    types: &TypeRegistry,
    constructor: TypedConstructorSource,
    initializer: TypedConstructorSource,
    scope: TypedConstructorDefaultScope,
    ty: TypeId,
    pointer: TypeId,
) -> CheckedTypedConstructorInitializationStep {
    CheckedTypedConstructorInitializationStep::from_checked_binding(
        constructor,
        initializer,
        scope,
        ty,
        pointer,
        None,
        types,
    )
    .unwrap()
}

#[test]
fn genuine_source_rebind_preserves_source_issuance_and_changes_payload_issuance() {
    let mut ids = Identities::default();
    let mut sources = SourceMap::default();
    let constructor = source(&mut sources, ids.declaration());
    let mut first_types = TypeRegistry::new();
    let (ty, pointer) = storage(&mut first_types);
    let receipt = uninitialized(&first_types, constructor.clone(), ty, pointer);
    let mut fresh_types = TypeRegistry::new();
    let (mapped_ty, mapped_pointer) = storage(&mut fresh_types);
    let rebound = receipt
        .rebind_checked_specialization(
            constructor,
            mapped_ty,
            mapped_pointer,
            TypedConstructorInitialization::Uninitialized,
            &fresh_types,
        )
        .unwrap();
    assert!(receipt.same_source_issuance(&rebound));
    assert!(!receipt.same_issuance(&rebound));
    assert_eq!(rebound.extent(), receipt.extent());
    assert_eq!(rebound.byte_order(), ByteOrder::Little);
    assert!(
        rebound.retained_byte_upper().unwrap()
            >= rebound.constructor().identity().source_text().len()
    );
}

#[test]
fn equal_source_reload_cannot_replace_the_original_allocation() {
    let mut ids = Identities::default();
    let declaration = ids.declaration();
    let mut original = SourceMap::default();
    let mut replacement = SourceMap::default();
    let original_source = source(&mut original, declaration);
    let reloaded = source(&mut replacement, declaration);
    assert_eq!(
        original_source.identity().source_text(),
        reloaded.identity().source_text()
    );
    assert!(!original_source.same_definition(&reloaded));
    let mut types = TypeRegistry::new();
    let (ty, pointer) = storage(&mut types);
    let receipt = uninitialized(&types, original_source, ty, pointer);
    assert!(matches!(
        receipt.rebind_checked_specialization(
            reloaded,
            ty,
            pointer,
            TypedConstructorInitialization::Uninitialized,
            &types,
        ),
        Err(TypedConstructorError::ChangedSource)
    ));
}

#[test]
fn rebind_cannot_substitute_a_fresh_initializer_event_with_equal_policy() {
    let mut ids = Identities::default();
    let mut sources = SourceMap::default();
    let constructor = source(&mut sources, ids.declaration());
    let initializer = source(&mut sources, ids.declaration());
    let scope = TypedConstructorDefaultScope {
        declaration: None,
        identity: constructor.identity().clone(),
    };
    let mut types = TypeRegistry::new();
    let (ty, pointer) = storage(&mut types);
    let original_step = step(
        &types,
        constructor.clone(),
        initializer.clone(),
        scope.clone(),
        ty,
        pointer,
    );
    let receipt = CheckedTypedConstructorReceipt::from_checked_specialization(
        constructor.clone(),
        ty,
        pointer,
        TypedConstructorInitialization::Default {
            initializer: initializer.clone(),
            scope: scope.clone(),
            step: original_step.clone(),
        },
        LayoutPolicy::lp64(),
        ByteOrder::Little,
        &types,
    )
    .unwrap();
    let different_step = step(
        &types,
        constructor.clone(),
        initializer.clone(),
        scope.clone(),
        ty,
        pointer,
    );
    assert!(!original_step.same_source_issuance(&different_step));
    assert!(matches!(
        receipt.rebind_checked_specialization(
            constructor.clone(),
            ty,
            pointer,
            TypedConstructorInitialization::Default {
                initializer: initializer.clone(),
                scope: scope.clone(),
                step: different_step
            },
            &types,
        ),
        Err(TypedConstructorError::ChangedSource)
    ));
    let rebound_step = original_step
        .rebind_checked_binding(constructor, initializer, scope, ty, pointer, None, &types)
        .unwrap();
    assert!(original_step.same_source_issuance(&rebound_step));
    assert!(!original_step.same_issuance(&rebound_step));
}

#[test]
fn checked_rebind_rejects_changed_layout_and_nonpointer_results() {
    let mut ids = Identities::default();
    let mut sources = SourceMap::default();
    let constructor = source(&mut sources, ids.declaration());
    let mut types = TypeRegistry::new();
    let (ty, pointer) = storage(&mut types);
    let receipt = uninitialized(&types, constructor.clone(), ty, pointer);
    assert!(matches!(
        CheckedTypedConstructorReceipt::from_checked_specialization(
            constructor.clone(),
            ty,
            ty,
            TypedConstructorInitialization::Uninitialized,
            LayoutPolicy::lp64(),
            ByteOrder::Little,
            &types,
        ),
        Err(TypedConstructorError::PointerType)
    ));
    let larger = types.scalar(ScalarType::Int(jai_types::IntegerType::U64));
    let larger_pointer = types.pointer(larger).unwrap();
    assert!(matches!(
        receipt.rebind_checked_specialization(
            constructor,
            larger,
            larger_pointer,
            TypedConstructorInitialization::Uninitialized,
            &types,
        ),
        Err(TypedConstructorError::ChangedLayout)
    ));
}
