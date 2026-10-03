//! Final publication rejects invented owners and changed native binding identities.
use jai_ir::*;
use jai_source::{Identities, SourceMap, SourceSpan, Span};
use jai_types::{
    CallingConvention, ContextMode, IntegerType, ProcedureExecution, ProcedureType, ScalarType,
    TypeRegistry, Variadic,
};
use std::collections::HashMap;

fn origin() -> (Identities, SourceMap, SourceSpan) {
    let mut sources = SourceMap::default();
    let text = "counter:s64 #elsewhere;";
    let source = sources.insert("owned.jai".into(), text.into());
    (
        Identities::default(),
        sources,
        SourceSpan {
            source,
            span: Span::new(0, text.len()),
        },
    )
}
fn data(
    types: &TypeRegistry,
    id: ExternalDataId,
    location: SourceSpan,
    source: ExternalDataSource,
) -> ExternalData {
    ExternalData::new(
        id,
        types.scalar(ScalarType::Int(IntegerType::S64)),
        source,
        "counter".into(),
        location,
        types,
    )
    .unwrap()
}
fn procedure(
    types: &mut TypeRegistry,
    id: ProcedureId,
    convention: CallingConvention,
) -> Procedure {
    let signature = types
        .procedure(ProcedureType {
            parameters: vec![].into(),
            results: vec![].into(),
            convention,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    Procedure {
        id,
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

#[test]
fn missing_or_prototype_only_local_owner_cannot_publish_data() {
    for prototype_only in [false, true] {
        let mut types = TypeRegistry::new();
        let (_, _, location) = origin();
        let id = ProcedureId::new(99);
        let global = Global::new_external(
            0,
            data(
                &types,
                ExternalDataId::Local {
                    procedure: id,
                    index: LocalExternalDataIndex::new(0),
                },
                location,
                ExternalDataSource::Program,
            ),
        );
        let mut prototypes = vec![];
        if prototype_only {
            let declaration = procedure(&mut types, id, CallingConvention::C);
            prototypes.push(ProcedurePrototype {
                id,
                signature: declaration.signature,
                origin: PrototypeOrigin::Foreign {
                    symbol: "authored_function".into(),
                    library: None,
                },
            });
        }
        let error = ProgramBuilder::new(types.freeze().unwrap())
            .globals(vec![global])
            .prototypes(prototypes)
            .finish_library()
            .unwrap_err();
        assert!(matches!(error, IrError::ExternalData(error)
            if matches!(*error, ExternalDataError::UnknownProcedure(owner) if owner == id)));
    }
}

#[test]
fn prototype_signature_cannot_own_a_local_foreign_library() {
    let mut types = TypeRegistry::new();
    let id = ProcedureId::new(99);
    let declaration = procedure(&mut types, id, CallingConvention::C);
    let error = ProgramBuilder::new(types.freeze().unwrap())
        .prototypes(vec![ProcedurePrototype {
            id,
            signature: declaration.signature,
            origin: PrototypeOrigin::Foreign {
                symbol: "authored_function".into(),
                library: None,
            },
        }])
        .foreign_libraries(vec![ForeignLibrary {
            id: ForeignLibraryId::local(id, 0),
            kind: ForeignLibraryKind::Local {
                path: "unused-authored.a".into(),
            },
            options: Default::default(),
        }])
        .finish_library()
        .unwrap_err();
    assert!(matches!(
        error,
        IrError::UnknownIdentity {
            kind: "foreign library procedure owner",
            index: 99
        }
    ));
}

#[test]
fn authentic_source_only_owner_publishes_local_data_and_local_library() {
    let mut types = TypeRegistry::new();
    let mut sources = SourceMap::default();
    let text = "#run { lib :: #library \"unused-authored.a\"; counter:s64 #elsewhere lib; }";
    let source = sources.insert("owned.jai".into(), text.into());
    let location = SourceSpan {
        source,
        span: Span::new(0, text.len()),
    };
    let start = text.find("counter").unwrap();
    let declaration_location = SourceSpan {
        source,
        span: Span::new(start, start + "counter:s64 #elsewhere lib;".len()),
    };
    let id = ProcedureId::new(7);
    let procedure = procedure(&mut types, id, CallingConvention::Jai);
    let library = ForeignLibrary {
        id: ForeignLibraryId::local(id, 0),
        kind: ForeignLibraryKind::Local {
            path: "unused-authored.a".into(),
        },
        options: Default::default(),
    };
    let global = Global::new_external(
        0,
        data(
            &types,
            ExternalDataId::Local {
                procedure: id,
                index: LocalExternalDataIndex::new(0),
            },
            declaration_location,
            ExternalDataSource::Library(library.clone()),
        ),
    );
    let globals = vec![global];
    let signatures = HashMap::from([(id, procedure.signature)]);
    let places = SourceProcedurePlaces::new(Places::default());
    let checked =
        verify_procedure(&types, &procedure, &signatures, &globals, places.places()).unwrap();
    let prefix = GlobalDefinitionsPrefix::new(&[], 0, &types, &signatures).unwrap();
    let receipt = CheckedSourceProcedureOwner::new(
        checked,
        ProcedureExecution::CompileTimeOnly,
        SourceProcedureIdentity::new(sources.get(location.source).unwrap(), location).unwrap(),
        prefix,
        places.clone(),
    )
    .unwrap();
    let mut owners = SourceProcedureOwners::default();
    owners.insert(receipt).unwrap();
    let published = ProgramBuilder::new(types.freeze().unwrap())
        .globals(globals)
        .foreign_libraries(vec![library])
        .source_procedure_owners(owners)
        .finish_library()
        .unwrap();
    assert!(published.procedures().is_empty());
    assert!(published.prototypes().is_empty());
    assert!(published.signature(id).is_none());
    assert!(published.source_procedure_owners().get(id).is_some());
}

#[test]
fn duplicate_source_identity_cannot_create_multiple_storage_slots() {
    let types = TypeRegistry::new();
    let (mut ids, _, location) = origin();
    let declaration = ids.declaration();
    let binding = data(
        &types,
        ExternalDataId::File(declaration),
        location,
        ExternalDataSource::Program,
    );
    let error = ProgramBuilder::new(types.freeze().unwrap())
        .globals(vec![
            Global::new_external(0, binding.clone()),
            Global::new_external(1, binding),
        ])
        .finish_library()
        .unwrap_err();
    assert!(matches!(error, IrError::ExternalData(error)
        if matches!(*error, ExternalDataError::DuplicateIdentity {first,second,..}
            if first==GlobalId::new(0) && second==GlobalId::new(1))));
}

#[test]
fn changed_provider_descriptor_cannot_borrow_the_canonical_library_id() {
    for changed_options in [false, true] {
        let types = TypeRegistry::new();
        let (mut ids, _, location) = origin();
        let canonical = ForeignLibrary {
            id: ForeignLibraryId::File(ids.declaration()),
            kind: ForeignLibraryKind::Local {
                path: "authored.a".into(),
            },
            options: Default::default(),
        };
        let mut changed = canonical.clone();
        if changed_options {
            changed.options.no_dll = true;
        } else {
            changed.kind = ForeignLibraryKind::Local {
                path: "different.a".into(),
            };
        }
        let binding = data(
            &types,
            ExternalDataId::File(ids.declaration()),
            location,
            ExternalDataSource::Library(changed),
        );
        let error = ProgramBuilder::new(types.freeze().unwrap())
            .globals(vec![Global::new_external(0, binding)])
            .foreign_libraries(vec![canonical])
            .finish_library()
            .unwrap_err();
        assert!(matches!(error, IrError::ExternalData(error)
            if matches!(*error, ExternalDataError::LibraryIdentity(_))));
    }
}

#[test]
fn external_export_alias_cannot_claim_fabricated_native_storage() {
    let types = TypeRegistry::new();
    let (mut ids, _, location) = origin();
    let declaration = ids.declaration();
    let binding = data(
        &types,
        ExternalDataId::File(declaration),
        location,
        ExternalDataSource::Program,
    );
    let error = ProgramBuilder::new(types.freeze().unwrap())
        .globals(vec![Global::new_external(0, binding)])
        .program_exports(vec![ProgramExport {
            declaration,
            target: ExportTarget::Global(GlobalId::new(0)),
            symbol: NativeSymbol::new("renamed").unwrap(),
        }])
        .finish_library()
        .unwrap_err();
    assert!(matches!(error, IrError::ExternalData(error)
        if matches!(*error, ExternalDataError::ExportAlias { .. })));
}
