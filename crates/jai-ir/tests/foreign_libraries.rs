use jai_ir::*;
use jai_source::Identities;
use jai_types::{CallingConvention, ContextMode, ProcedureType, TypeRegistry, Variadic};

fn dependency() -> ForeignLibrary {
    ForeignLibrary {
        id: ForeignLibraryId::new(Identities::default().declaration()),
        kind: ForeignLibraryKind::System {
            name: "libc".into(),
        },
        options: ForeignLibraryOptions::default(),
    }
}

fn prototype_builder(binding: ForeignLibrary) -> ProgramBuilder {
    let mut types = TypeRegistry::new();
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            convention: CallingConvention::C,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    ProgramBuilder::new(types.freeze().unwrap()).prototypes(vec![ProcedurePrototype {
        id: ProcedureId::new(0),
        signature,
        origin: PrototypeOrigin::Foreign {
            symbol: "owned_fixture".into(),
            library: Some(binding),
        },
    }])
}

#[test]
fn binding_requires_matching_published_declaration_metadata() {
    let dependency = dependency();
    assert!(
        prototype_builder(dependency.clone())
            .finish_library()
            .is_err()
    );
    let mut different = dependency.clone();
    different.kind = ForeignLibraryKind::System {
        name: "libm".into(),
    };
    assert!(
        prototype_builder(dependency.clone())
            .foreign_libraries(vec![different])
            .finish_library()
            .is_err()
    );
    let library = prototype_builder(dependency.clone())
        .foreign_libraries(vec![dependency.clone()])
        .finish_library()
        .unwrap();
    assert_eq!(library.foreign_libraries(), [dependency]);
}

#[test]
fn duplicate_dependency_identities_cannot_publish() {
    let dependency = dependency();
    assert!(
        prototype_builder(dependency.clone())
            .foreign_libraries(vec![dependency.clone(), dependency])
            .finish_library()
            .is_err()
    );
}

#[test]
fn lexical_dependencies_require_a_published_procedure_owner() {
    let mut dependency = dependency();
    dependency.id = ForeignLibraryId::local(ProcedureId::new(99), 0);
    assert!(
        prototype_builder(dependency.clone())
            .foreign_libraries(vec![dependency])
            .finish_library()
            .is_err()
    );
}

#[test]
fn invalid_unreferenced_system_metadata_cannot_publish() {
    let mut dependency = dependency();
    dependency.kind = ForeignLibraryKind::System {
    };
    assert!(
        ProgramBuilder::new(TypeRegistry::new().freeze().unwrap())
            .foreign_libraries(vec![dependency])
            .finish_library()
            .is_err()
    );
}
