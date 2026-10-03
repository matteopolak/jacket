//! The semantic bridge retains source requests and commits canonical responses once.
use jai_modules::{
    GraphDiscovery, GraphOptions, ModuleBoundArgument, ModuleBuiltin, ModuleType,
    ParameterResponse, ParameterResponseError, ParameterTask, ParameterValue, SourceOverlay,
};
use jai_syntax::{FileDeclarationKind, TypeSyntax};
use jai_types::{IntegerType, ScalarType};
use std::path::Path;

fn sources() -> SourceOverlay {
    let mut sources = SourceOverlay::new();
    for (path, text) in [
        (
            "/parameter-discovery/main.jai",
            "Box::struct(T:Type){value:T;} marker::17; A::#import,file \"library.jai\"(T=Box(u8));",
        ),
        (
            "/parameter-discovery/library.jai",
            "#module_parameters(T:Type=int);",
        ),
    ] {
        sources
            .insert(Path::new(path), text.as_bytes().to_vec())
            .unwrap();
    }
    sources
}
fn session(sources: &SourceOverlay) -> GraphDiscovery<'_> {
    GraphDiscovery::new(
        Path::new("/parameter-discovery/main.jai"),
        GraphOptions::default(),
        sources,
    )
    .unwrap()
}
fn box_response(discovery: &GraphDiscovery<'_>, ty: IntegerType) -> ParameterResponse {
    let template = discovery.graph().declarations().iter().find(|declaration| {
        matches!(&declaration.syntax().kind, FileDeclarationKind::Record(record) if !record.parameters.is_empty())
    }).unwrap().id();
    ParameterResponse::Type(ModuleType::Application {
        template,
        arguments: vec![ModuleBoundArgument::Type(ModuleType::Builtin(
            ModuleBuiltin::Scalar(ScalarType::Int(ty)),
        ))],
    })
}
#[test]
fn original_application_survives_retries_and_canonical_response_resumes() {
    let inputs = sources();
    let mut discovery = session(&inputs);
    assert!(!discovery.advance().unwrap().is_complete());
    let requests = discovery.pending_parameter_requests();
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    let ParameterTask::ResolveType {
        syntax: TypeSyntax::Application(application),
    } = &request.task
    else {
        panic!("expected original type application")
    };
    assert_eq!(application.arguments.len(), 1);
    let source = discovery
        .graph()
        .sources()
        .get(request.location.source)
        .unwrap();
    assert_eq!(request.location.span.text(source.text()), "Box(u8)");
    let ids: Vec<_> = discovery
        .graph()
        .declarations()
        .iter()
        .map(|declaration| declaration.id())
        .collect();
    discovery.advance().unwrap();
    assert_eq!(discovery.pending_parameter_requests()[0].id, request.id);
    assert_eq!(discovery.parameter_requests().len(), 1);
    assert_eq!(
        ids,
        discovery
            .graph()
            .declarations()
            .iter()
            .map(|declaration| declaration.id())
            .collect::<Vec<_>>()
    );
    let response = box_response(&discovery, IntegerType::U8);
    discovery
        .resolve_parameter(request.id, response.clone())
        .unwrap();
    discovery
        .resolve_parameter(request.id, response.clone())
        .unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    let graph = discovery.into_graph().ok().unwrap();
    let ParameterResponse::Type(expected) = response else {
        unreachable!()
    };
    assert_eq!(graph.parameters()[0].value, ParameterValue::Type(expected));
}

#[test]
fn nominal_restrictions_publish_parameters_only_after_a_typed_proof() {
    let mut inputs = sources();
    inputs.insert(Path::new("/parameter-discovery/main.jai"), b"Base::struct{value:int;} Child::struct{using base:Base;} A::#import,file \"library.jai\"(R=Child);".to_vec()).unwrap();
    // Base is an explicit source dependency of the required nominal, not a
    // caller file binding implicitly inherited by the module.
    inputs
        .insert(
            Path::new("/parameter-discovery/library.jai"),
            b"Base::struct{value:int;} #module_parameters(R:$I/Base=Base);".to_vec(),
        )
        .unwrap();
    let mut discovery = session(&inputs);
    assert!(!discovery.advance().unwrap().is_complete());
    let requests = discovery.pending_parameter_requests();
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    let ParameterTask::CheckNominal { actual, required } = &request.task else {
        panic!("expected source nominal ancestry request");
    };
    assert_ne!(
        actual, required,
        "equal shapes do not share source nominal identity"
    );
    assert!(discovery.graph().parameters().is_empty());
    assert_eq!(
        discovery.resolve_parameter(request.id, ParameterResponse::InterfaceSatisfied),
        Err(ParameterResponseError::WrongResponseKind)
    );
    // Response submission checks the semantic scheduler's proof kind. Actual
    // ancestry rejection is exercised by the driver integration fixtures.
    discovery
        .resolve_parameter(request.id, ParameterResponse::NominalSatisfied)
        .unwrap();
    discovery
        .resolve_parameter(request.id, ParameterResponse::NominalSatisfied)
        .unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    let graph = discovery.into_graph().ok().unwrap();
    let parameter = &graph.parameters()[0];
    for name in ["R", "I"] {
        assert!(matches!(
            graph.lookup(
                graph.module(parameter.module).unwrap().entry(),
                &jai_syntax::NamePath {
                    root: graph.symbols().find(name).unwrap(),
                    members: vec![]
                }
            ),
            Ok(jai_modules::Binding::Parameter(_))
        ));
    }
}
#[test]
fn responses_are_session_scoped_typed_and_immutable() {
    let inputs = sources();
    let mut first = session(&inputs);
    let mut second = session(&inputs);
    first.advance().unwrap();
    second.advance().unwrap();
    let id = first.pending_parameter_requests()[0].id;
    let response = box_response(&first, IntegerType::U8);
    assert_eq!(
        second.resolve_parameter(id, response.clone()),
        Err(ParameterResponseError::UnknownRequest)
    );
    assert_eq!(
        first.resolve_parameter(id, ParameterResponse::InterfaceSatisfied),
        Err(ParameterResponseError::WrongResponseKind)
    );
    first.resolve_parameter(id, response).unwrap();
    let changed = box_response(&first, IntegerType::U16);
    assert_eq!(
        first.resolve_parameter(id, changed),
        Err(ParameterResponseError::AlreadyResolved)
    );
}
#[test]
fn responses_reject_non_type_declarations_and_incomplete_formal_keys() {
    let inputs = sources();
    let mut discovery = session(&inputs);
    discovery.advance().unwrap();
    let id = discovery.pending_parameter_requests()[0].id;
    let constant = discovery
        .graph()
        .declarations()
        .iter()
        .find(|declaration| matches!(declaration.syntax().kind, FileDeclarationKind::Constant(_)))
        .unwrap()
        .id();
    assert_eq!(
        discovery.resolve_parameter(
            id,
            ParameterResponse::Type(ModuleType::Declaration(constant))
        ),
        Err(ParameterResponseError::InvalidSourceIdentity)
    );
    let ParameterResponse::Type(ModuleType::Application { template, .. }) =
        box_response(&discovery, IntegerType::U8)
    else {
        unreachable!()
    };
    assert_eq!(
        discovery.resolve_parameter(
            id,
            ParameterResponse::Type(ModuleType::Application {
                template,
                arguments: vec![]
            })
        ),
        Err(ParameterResponseError::InvalidSourceIdentity)
    );
    assert_eq!(
        discovery.resolve_parameter(
            id,
            ParameterResponse::Type(ModuleType::Declaration(template))
        ),
        Err(ParameterResponseError::InvalidSourceIdentity),
        "unspecialized templates are not nominal type values"
    );
    assert_eq!(discovery.pending_parameter_requests().len(), 1);
}
#[test]
fn inherited_interface_retains_actual_nominal_inputs_without_guessing_conformance() {
    let mut inputs = SourceOverlay::new();
    for (path, text) in [
        (
            "/parameter-discovery/main.jai",
            "Base::struct{value:int;} Replacement::struct{using base:Base;} A::#import,file \"library.jai\"()(R=Replacement);",
        ),
        (
            "/parameter-discovery/library.jai",
            "#module_parameters()(R:$I/interface Required=Required){Required::struct{value:int;}}",
        ),
    ] {
        inputs
            .insert(Path::new(path), text.as_bytes().to_vec())
            .unwrap();
    }
    let mut discovery = session(&inputs);
    discovery.advance().unwrap();
    let requests = discovery.pending_parameter_requests();
    assert_eq!(requests.len(), 1);
    assert!(matches!(
        requests[0].task,
        ParameterTask::CheckInterface {
            actual: ModuleType::Declaration(_),
            required: ModuleType::Declaration(_)
        }
    ));
    discovery.advance().unwrap();
    assert_eq!(discovery.pending_parameter_requests()[0].id, requests[0].id);
    assert!(
        discovery.graph().parameters().is_empty(),
        "no unchecked interface binding is published"
    );
}

#[test]
fn type_query_retains_its_defining_file_and_original_expression() {
    let mut inputs = SourceOverlay::new();
    for (path, text) in [
        (
            "/parameter-discovery/main.jai",
            "sample:u16; #load \"types.jai\"; A::#import,file \"library.jai\"(T=Alias);",
        ),
        (
            "/parameter-discovery/types.jai",
            "#scope_file; Sample::struct{value:u8;} sample:Sample; #scope_export; Alias::type_of(sample.value);",
        ),
        (
            "/parameter-discovery/library.jai",
            "#module_parameters(T:Type=int);",
        ),
    ] {
        inputs
            .insert(Path::new(path), text.as_bytes().to_vec())
            .unwrap();
    }
    let mut discovery = session(&inputs);
    let status = discovery.advance().unwrap();
    assert!(!status.is_complete());
    let requests = discovery.pending_parameter_requests();
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    let ParameterTask::ResolveType {
        syntax: TypeSyntax::TypeOf(expression),
    } = &request.task
    else {
        panic!("expected retained type query")
    };
    let source = discovery
        .graph()
        .sources()
        .get(request.location.source)
        .unwrap();
    assert_eq!(source.path(), Path::new("/parameter-discovery/types.jai"));
    assert_eq!(expression.span.text(source.text()), "sample.value");
    assert!(request.specialization.is_none());
    assert!(
        discovery.graph().parameters().is_empty(),
        "the graph does not guess a scalar type"
    );
    discovery.advance().unwrap();
    assert_eq!(discovery.pending_parameter_requests()[0].id, request.id);
}

#[test]
fn generic_parameter_requests_keep_independent_specialization_responses() {
    let mut inputs = SourceOverlay::new();
    for (path, text) in [
        (
            "/parameter-discovery/main.jai",
            "Box::struct(U:Type){value:U;} choose::($T:Type){M::#import,file \"library.jai\"(T=Box(T));}",
        ),
        (
            "/parameter-discovery/library.jai",
            "#module_parameters(T:Type=int);",
        ),
    ] {
        inputs
            .insert(Path::new(path), text.as_bytes().to_vec())
            .unwrap();
    }
    let mut discovery = session(&inputs);
    assert!(discovery.advance().unwrap().is_complete());
    let template = discovery.graph().dependency_templates()[0];
    let formal = discovery.graph().symbols().find("T").unwrap();
    let key = |ty| {
        jai_modules::SourceSpecializationKey::new(
            template.declaration(),
            template.location(),
            vec![(
                formal,
                ModuleBoundArgument::Type(ModuleType::Builtin(ModuleBuiltin::Scalar(
                    ScalarType::Int(ty),
                ))),
            )],
        )
    };
    for ty in [IntegerType::U8, IntegerType::U16] {
        discovery.discover_specialization(key(ty)).unwrap();
    }
    assert!(!discovery.advance().unwrap().is_complete());
    let requests = discovery.pending_parameter_requests();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].file, requests[1].file);
    assert_eq!(requests[0].location, requests[1].location);
    assert_ne!(requests[0].id, requests[1].id);
    assert_ne!(requests[0].specialization, requests[1].specialization);
    for request in requests {
        let specialization = request.specialization.as_ref().unwrap();
        let (
            _,
            ModuleBoundArgument::Type(ModuleType::Builtin(ModuleBuiltin::Scalar(ScalarType::Int(
                ty,
            )))),
        ) = &specialization.arguments()[0]
        else {
            panic!()
        };
        let response = box_response(&discovery, *ty);
        discovery.resolve_parameter(request.id, response).unwrap();
    }
    assert!(discovery.advance().unwrap().is_complete());
    let graph = discovery.graph();
    assert_eq!(graph.scoped_imports().len(), 2);
    assert_ne!(
        graph.scoped_imports()[0].module(),
        graph.scoped_imports()[1].module()
    );
}

#[test]
fn separate_type_applications_inside_one_annotation_never_share_a_response() {
    let mut inputs = sources();
    inputs.insert(Path::new("/parameter-discovery/main.jai"), b"Box::struct(U:Type){value:U;} A::#import,file \"library.jai\"(T=#type (a:Box(u8),b:Box(u16)));".to_vec()).unwrap();
    let mut discovery = session(&inputs);
    assert!(!discovery.advance().unwrap().is_complete());
    let first = discovery.pending_parameter_requests()[0].clone();
    discovery
        .resolve_parameter(first.id, box_response(&discovery, IntegerType::U8))
        .unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    let second = discovery.pending_parameter_requests()[0].clone();
    assert_ne!(first.id, second.id);
    assert_ne!(first.location, second.location);
    discovery
        .resolve_parameter(second.id, box_response(&discovery, IntegerType::U16))
        .unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    let ParameterValue::Type(ModuleType::Procedure(signature)) =
        &discovery.graph().parameters()[0].value
    else {
        panic!()
    };
    assert_ne!(signature.parameters[0], signature.parameters[1]);
}

#[test]
fn required_optional_and_restricted_formals_keep_source_dependencies_dormant() {
    let mut inputs = sources();
    inputs.insert(Path::new("/parameter-discovery/main.jai"), b"Allowed::struct{} required::($X:int){ M::#import,file \"missing.jai\"; } optional::($$X:int){ M::#import,file \"missing.jai\"; } constrained::(x:$T/Allowed){ M::#import,file \"missing.jai\"; }".to_vec()).unwrap();
    let mut discovery = session(&inputs);
    assert!(discovery.advance().unwrap().is_complete());
    assert_eq!(discovery.graph().dependency_templates().len(), 3);
    assert!(discovery.graph().imports().is_empty());
    let baking = discovery
        .graph()
        .declarations()
        .iter()
        .filter_map(|declaration| match &declaration.syntax().kind {
            FileDeclarationKind::Procedure(procedure) => Some(procedure.parameters[0].baking),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        baking,
        [
            jai_syntax::ParameterBaking::Required,
            jai_syntax::ParameterBaking::Optional,
            jai_syntax::ParameterBaking::None
        ]
    );
}

#[test]
fn module_type_restrictions_retain_the_complete_nominal_source_constraint() {
    let mut inputs = sources();
    inputs
        .insert(
            Path::new("/parameter-discovery/main.jai"),
            b"Allowed::struct{} Lib::#import,file \"library.jai\"(T=#type $T/Allowed);".to_vec(),
        )
        .unwrap();
    let mut discovery = session(&inputs);
    assert!(!discovery.advance().unwrap().is_complete());
    let requests = discovery.pending_parameter_requests();
    let ParameterTask::ResolveType {
        syntax:
            TypeSyntax::Restricted {
                variable,
                restriction: jai_syntax::TypeRestrictionSyntax::Nominal(base),
                span,
            },
    } = &requests[0].task
    else {
        panic!("restriction was erased")
    };
    assert_eq!(discovery.graph().symbols().name(*variable), "T");
    assert!(
        matches!(base.as_ref(), TypeSyntax::Named(path) if discovery.graph().symbols().name(path.root)=="Allowed")
    );
    assert!(
        span.text(
            discovery
                .graph()
                .sources()
                .get(requests[0].location.source)
                .unwrap()
                .text()
        )
        .contains("$T/Allowed")
    );
    assert!(discovery.graph().parameters().is_empty());
}
