use super::*;
use crate::{Block, Exit, Flow, GlobalId, Statement, Transfer, verify_procedure};
use jai_source::{SourceMap, Span};
use jai_types::{CallingConvention, ContextMode, ProcedureType, TypeRegistry, Variadic};

fn procedure(types: &mut TypeRegistry, index: usize) -> Procedure {
    let signature = types
        .procedure(ProcedureType {
            parameters: vec![].into(),
            results: vec![].into(),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    Procedure {
        id: ProcedureId::new(index),
        signature,
        parameters: vec![],
        locals: vec![],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![Statement::Exit(Exit {
                cleanups: vec![],
                transfer: Transfer::ReturnVoid,
            })],
        },
    }
}
fn identity(sources: &mut SourceMap, text: &str) -> SourceProcedureIdentity {
    let source = sources.insert("recipe.jai".into(), text.into());
    SourceProcedureIdentity::new(
        sources.get(source).unwrap(),
        SourceSpan {
            source,
            span: Span::new(0, text.len()),
        },
    )
    .unwrap()
}
fn owner(types: &mut TypeRegistry, id: usize, globals: &[Global]) -> CheckedSourceProcedureOwner {
    let procedure = procedure(types, id);
    let signatures = HashMap::from([(procedure.id, procedure.signature)]);
    let places = SourceProcedurePlaces::new(Places::default());
    let checked =
        verify_procedure(types, &procedure, &signatures, globals, places.places()).unwrap();
    let prefix = GlobalDefinitionsPrefix::new(globals, globals.len(), types, &signatures).unwrap();
    let mut sources = SourceMap::default();
    CheckedSourceProcedureOwner::new(
        checked,
        ProcedureExecution::CompileTimeOnly,
        identity(&mut sources, "#run {}"),
        prefix,
        places.clone(),
    )
    .unwrap()
}
#[test]
fn actual_body_and_source_remain_owned_without_runtime_signature_pollution() {
    let mut types = TypeRegistry::new();
    let globals = vec![Global::new(0, GlobalInitializer::Bool(true), &types)];
    let owner = owner(&mut types, 41, &globals);
    assert_eq!(owner.identity().body_text(), "#run {}");
    assert_eq!(owner.identity().path(), Path::new("recipe.jai"));
    assert_eq!(owner.execution(), ProcedureExecution::CompileTimeOnly);
    let mut ledger = SourceProcedureOwners::default();
    ledger.insert(owner.clone()).unwrap();
    ledger.insert(owner).unwrap();
    let runtime = HashMap::new();
    ledger.validate(&types, &runtime, &globals, None).unwrap();
    assert!(runtime.is_empty());
    assert_eq!(
        ledger.get(ProcedureId::new(41)).unwrap().procedure().id,
        ProcedureId::new(41)
    );
}
#[test]
fn incomplete_or_non_dense_file_prefix_never_seals() {
    let types = TypeRegistry::new();
    let signatures = HashMap::new();
    assert!(matches!(
        GlobalDefinitionsPrefix::new(&[], 1, &types, &signatures),
        Err(SourceProcedureOwnerError::IncompleteGlobalDefinitions {
            expected: 1,
            actual: 0
        })
    ));
    let globals = [Global::new(3, GlobalInitializer::Bool(false), &types)];
    assert!(matches!(
        GlobalDefinitionsPrefix::new(&globals, 1, &types, &signatures),
        Err(SourceProcedureOwnerError::ChangedGlobals)
    ));
}
#[test]
fn changed_global_environment_and_cross_arena_types_reject() {
    let mut types = TypeRegistry::new();
    let globals = [Global::new(0, GlobalInitializer::Bool(false), &types)];
    let owner = owner(&mut types, 12, &globals);
    let mut ledger = SourceProcedureOwners::default();
    ledger.insert(owner).unwrap();
    let changed = [Global::new(0, GlobalInitializer::Bool(true), &types)];
    assert!(matches!(
        ledger.validate(&types, &HashMap::new(), &changed, None),
        Err(SourceProcedureOwnerError::ChangedGlobals)
    ));
    let other_types = TypeRegistry::new();
    assert!(
        ledger
            .validate(&other_types, &HashMap::new(), &globals, None)
            .is_err()
    );
    // Failed validation does not mutate the retained publication candidate.
    ledger
        .validate(&types, &HashMap::new(), &globals, None)
        .unwrap();
    assert_eq!(globals[0].id(), GlobalId::new(0));
}
#[test]
fn different_receipt_and_runtime_collision_reject_atomically() {
    let mut types = TypeRegistry::new();
    let first = owner(&mut types, 12, &[]);
    let signature = first.signature();
    let mut ledger = SourceProcedureOwners::default();
    ledger.insert(first.clone()).unwrap();
    assert!(matches!(
        ledger.insert(owner(&mut types, 12, &[])),
        Err(SourceProcedureOwnerError::DuplicateOwner(_))
    ));
    assert!(Arc::ptr_eq(&ledger.get(first.id()).unwrap().0, &first.0));
    assert!(matches!(
        ledger.validate(&types, &HashMap::from([(first.id(), signature)]), &[], None),
        Err(SourceProcedureOwnerError::DuplicateOwner(_))
    ));
    ledger.validate(&types, &HashMap::new(), &[], None).unwrap();
}
#[test]
fn wrong_source_range_and_runtime_phase_cannot_create_receipts() {
    let mut sources = SourceMap::default();
    let source = sources.insert("recipe.jai".into(), "é".into());
    assert!(matches!(
        SourceProcedureIdentity::new(
            sources.get(source).unwrap(),
            SourceSpan {
                source,
                span: Span::new(1, 2),
            }
        ),
        Err(SourceProcedureOwnerError::InvalidSource)
    ));
    let mut types = TypeRegistry::new();
    let procedure = procedure(&mut types, 0);
    let signatures = HashMap::from([(procedure.id, procedure.signature)]);
    let places = SourceProcedurePlaces::new(Places::default());
    let checked = verify_procedure(&types, &procedure, &signatures, &[], places.places()).unwrap();
    let prefix = GlobalDefinitionsPrefix::new(&[], 0, &types, &signatures).unwrap();
    assert!(matches!(
        CheckedSourceProcedureOwner::new(
            checked,
            ProcedureExecution::RuntimeAndCompileTime,
            identity(&mut sources, "#run {}"),
            prefix,
            places.clone(),
        ),
        Err(SourceProcedureOwnerError::NotCompileTimeOnly)
    ));
}

#[test]
fn unused_deep_place_operands_are_shared_and_disposed_without_recursive_clone() {
    let mut types = TypeRegistry::new();
    let procedure = procedure(&mut types, 19);
    let signatures = HashMap::from([(procedure.id, procedure.signature)]);
    let pointee = types.scalar(jai_types::ScalarType::Bool);
    let pointer = types.pointer(pointee).unwrap();
    let mut expression = crate::ValueExpr::Zero(pointer);
    for _ in 0..20_000 {
        expression = crate::ValueExpr::PointerCast {
            value: Box::new(expression),
            ty: pointer,
            mode: crate::CastMode::Checked,
        };
    }
    let mut registry = crate::PlaceRegistry::default();
    registry.dereference(expression, &types).unwrap();
    let places = SourceProcedurePlaces::new(registry.freeze());
    let checked = verify_procedure(&types, &procedure, &signatures, &[], places.places()).unwrap();
    let prefix = GlobalDefinitionsPrefix::new(&[], 0, &types, &signatures).unwrap();
    let mut sources = SourceMap::default();
    let owner = CheckedSourceProcedureOwner::new(
        checked,
        ProcedureExecution::CompileTimeOnly,
        identity(&mut sources, "#run {}"),
        prefix,
        places.clone(),
    )
    .unwrap();
    let mut ledger = SourceProcedureOwners::default();
    ledger.insert(owner).unwrap();
    ledger.validate(&types, &HashMap::new(), &[], None).unwrap();
    drop(places);
    drop(ledger);
}

#[test]
fn a_different_place_snapshot_cannot_replace_the_checked_environment() {
    let mut types = TypeRegistry::new();
    let procedure = procedure(&mut types, 2);
    let signatures = HashMap::from([(procedure.id, procedure.signature)]);
    let places = Places::default();
    let checked = verify_procedure(&types, &procedure, &signatures, &[], &places).unwrap();
    let prefix = GlobalDefinitionsPrefix::new(&[], 0, &types, &signatures).unwrap();
    let mut sources = SourceMap::default();
    assert!(matches!(
        CheckedSourceProcedureOwner::new(
            checked,
            ProcedureExecution::CompileTimeOnly,
            identity(&mut sources, "#run {}"),
            prefix,
            SourceProcedurePlaces::new(Places::default())
        ),
        Err(SourceProcedureOwnerError::ChangedPlaces)
    ));
}

#[test]
fn unchecked_nonexternal_slots_after_the_file_prefix_are_rejected_before_clone() {
    let mut types = TypeRegistry::new();
    let procedure = procedure(&mut types, 33);
    let signatures = HashMap::from([(procedure.id, procedure.signature)]);
    let ty = types.scalar(jai_types::ScalarType::Bool);
    let mut value = ConstantValue {
        ty,
        kind: ConstantKind::Bool(false),
    };
    for _ in 0..20_000 {
        value = ConstantValue {
            ty,
            kind: ConstantKind::Distinct(Box::new(value)),
        };
    }
    let globals = vec![Global::new(0, GlobalInitializer::Value(value), &types)];
    let places = SourceProcedurePlaces::new(Places::default());
    let checked =
        verify_procedure(&types, &procedure, &signatures, &globals, places.places()).unwrap();
    let prefix = GlobalDefinitionsPrefix::new(&[], 0, &types, &signatures).unwrap();
    let mut sources = SourceMap::default();
    assert!(matches!(
        CheckedSourceProcedureOwner::new(
            checked,
            ProcedureExecution::CompileTimeOnly,
            identity(&mut sources, "#run {}"),
            prefix,
            places.clone()
        ),
        Err(SourceProcedureOwnerError::ChangedGlobals)
    ));
    crate::disposal::globals(globals);
}

#[test]
fn source_owner_publication_authorizes_real_external_storage_without_a_runtime_body() {
    let mut types = TypeRegistry::new();
    let procedure = procedure(&mut types, 55);
    let signatures = HashMap::from([(procedure.id, procedure.signature)]);
    let mut sources = SourceMap::default();
    let identity = identity(&mut sources, "#run { external:int #elsewhere; }");
    let data = crate::ExternalData::new(
        crate::ExternalDataId::Local {
            procedure: procedure.id,
            index: crate::LocalExternalDataIndex::new(0),
        },
        types.scalar(jai_types::ScalarType::Int(jai_types::IntegerType::S64)),
        crate::ExternalDataSource::Program,
        "own_external_data".into(),
        identity.location(),
        &types,
    )
    .unwrap();
    let globals = vec![Global::new_external(0, data)];
    let places = SourceProcedurePlaces::new(Places::default());
    let checked =
        verify_procedure(&types, &procedure, &signatures, &globals, places.places()).unwrap();
    let prefix = GlobalDefinitionsPrefix::new(&[], 0, &types, &signatures).unwrap();
    let owner = CheckedSourceProcedureOwner::new(
        checked,
        ProcedureExecution::CompileTimeOnly,
        identity,
        prefix,
        places.clone(),
    )
    .unwrap();
    let mut ledger = SourceProcedureOwners::default();
    ledger.insert(owner).unwrap();
    let library = crate::ProgramBuilder::new(types.freeze().unwrap())
        .source_procedure_owners(ledger)
        .globals(globals)
        .finish_library()
        .unwrap();
    assert!(library.procedures().is_empty());
    assert!(library.prototypes().is_empty());
    assert!(library.signatures().is_empty());
    assert!(
        library
            .source_procedure_owners()
            .get(ProcedureId::new(55))
            .is_some()
    );
    assert!(matches!(
        library.globals()[0].initializer(),
        GlobalInitializer::External(_)
    ));
}

#[test]
fn identical_source_ids_and_text_from_another_allocation_do_not_match() {
    let mut first = SourceMap::default();
    let first_id = first.insert("recipe.jai".into(), "#run {}".into());
    let location = SourceSpan {
        source: first_id,
        span: Span::new(0, 7),
    };
    let identity = SourceProcedureIdentity::new(first.get(first_id).unwrap(), location).unwrap();
    assert!(identity.matches_source(first.get(first_id).unwrap(), location));
    let mut second = SourceMap::default();
    let second_id = second.insert("recipe.jai".into(), "#run {}".into());
    assert_eq!(first_id, second_id);
    assert!(!identity.matches_source(second.get(second_id).unwrap(), location));
}
