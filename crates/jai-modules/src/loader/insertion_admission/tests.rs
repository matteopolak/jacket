use super::*;
use jai_syntax::{CodeBody, ExpressionKind, NamePath, StatementKind};
use std::{cell::Cell, io};

const ENTRY: &str = "/admission/main.jai";

struct Inputs {
    sources: SourceOverlay,
    calls: Cell<usize>,
}
impl Inputs {
    fn new(source: &str) -> Self {
        let mut sources = SourceOverlay::new();
        sources
            .insert(Path::new(ENTRY), source.as_bytes().to_vec())
            .unwrap();
        Self {
            sources,
            calls: Cell::new(0),
        }
    }
    fn discovery(&self) -> GraphDiscovery<'_> {
        let mut discovery =
            GraphDiscovery::new(Path::new(ENTRY), GraphOptions::default(), self).unwrap();
        discovery.advance().unwrap();
        discovery
    }
    fn called(&self) {
        self.calls.set(self.calls.get() + 1);
    }
}
impl SourceProvider for Inputs {
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        self.called();
        self.sources.canonicalize(path)
    }
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.called();
        self.sources.read(path)
    }
    fn is_file(&self, path: &Path) -> bool {
        self.called();
        self.sources.is_file(path)
    }
}

fn quote(graph: &ModuleGraph, name: &str, visibility: Visibility) -> DeclarationInsertionCode {
    let declaration = graph
        .declarations()
        .iter()
        .find(|declaration| graph.symbols().name(declaration.name()) == name)
        .unwrap();
    let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
        panic!("expected a source Code constant");
    };
    let ExpressionKind::Code(CodeBody::Block(statements)) = &constant.initializer.kind else {
        panic!("expected original quoted statements");
    };
    let items = statements
        .iter()
        .map(|statement| {
            let location = SourceSpan {
                source: declaration.location().source,
                span: statement.span,
            };
            match &statement.kind {
                StatementKind::Constant(constant) => FileItem::Declaration(FileDeclaration {
                    program_export: None,
                    visibility,
                    kind: FileDeclarationKind::Constant(constant.clone()),
                    location,
                }),
                StatementKind::Insert(directive) => FileItem::Insert {
                    directive: directive.clone(),
                    location,
                },
                StatementKind::Import(import) => FileItem::Import(ImportDeclaration {
                    namespace: import.namespace,
                    using: import.using,
                    visibility,
                    mode: import.mode,
                    target: import.target.clone(),
                    arguments: import.arguments.clone(),
                    location,
                }),
                _ => panic!("unsupported test quote"),
            }
        })
        .collect();
    DeclarationInsertionCode {
        file: declaration.file(),
        source_file: declaration.file(),
        location: SourceSpan {
            source: declaration.location().source,
            span: constant.initializer.span,
        },
        items,
        bindings: vec![],
        values: vec![],
        checks: Default::default(),
        debug: Default::default(),
        origins: vec![],
    }
}

fn name(graph: &ModuleGraph, spelling: &str) -> NamePath {
    NamePath {
        root: graph.symbols().find(spelling).unwrap(),
        members: vec![],
    }
}

#[test]
fn rejected_admission_preserves_live_names_pending_work_and_allocation() {
    let inputs =
        Inputs::new("ANSWER :: 7; QUOTE :: #code { fresh :: 1; ANSWER :: 2; }; #insert QUOTE;");
    let mut discovery = inputs.discovery();
    let request = discovery
        .pending_insertion_requests()
        .next()
        .unwrap()
        .clone();
    let mut code = quote(discovery.graph(), "QUOTE", Visibility::Export);
    let graph_before = format!("{:?}", discovery.graph());
    let identities_before = format!("{:?}", discovery.builder.identities);
    let pending_before = format!("{:?}", discovery.builder.pending);
    let provider_calls = inputs.calls.get();
    discovery
        .graph()
        .validate_insertion_code(request.file, &code)
        .unwrap();
    assert!(matches!(
        discovery.admit_insertion(request.id, &code),
        Err(InsertionPublicationError::Graph(_))
    ));
    assert_eq!(format!("{:?}", discovery.graph()), graph_before);
    assert_eq!(
        format!("{:?}", discovery.builder.identities),
        identities_before
    );
    assert_eq!(format!("{:?}", discovery.builder.pending), pending_before);
    assert_eq!(inputs.calls.get(), provider_calls);
    assert_eq!(
        discovery
            .builder
            .insertion_requests
            .admission_generation(request.id),
        Ok(0)
    );

    let mut expected = discovery.builder.identities.clone();
    let next_scope = expected.scope();
    let next_declaration = expected.declaration();
    code.items.truncate(1);
    let admission = discovery.admit_insertion(request.id, &code).unwrap();
    let transaction = discovery
        .prepare_insertion_admitted(request.id, admission)
        .unwrap();
    let publication = discovery.commit_insertion(transaction).unwrap();
    let file = publication.file;
    assert_eq!(publication.declarations, [next_declaration]);
    assert_eq!(discovery.graph().file(file).unwrap().scope(), next_scope);
    assert!(discovery.advance().unwrap().is_complete());
}

#[test]
fn sealed_payload_preserves_captures_sources_and_the_actual_identity_frontier() {
    let inputs = Inputs::new(
        "captured :: 4; portable :: 0; QUOTE :: #code { answer :: captured; }; #insert QUOTE;",
    );
    let mut discovery = inputs.discovery();
    // Allocated but unpublished scopes cannot be reconstructed from visible IDs.
    discovery.builder.identities.scope();
    discovery.builder.identities.scope();
    let request = discovery
        .pending_insertion_requests()
        .next()
        .unwrap()
        .clone();
    let mut code = quote(discovery.graph(), "QUOTE", Visibility::Export);
    let captured = name(discovery.graph(), "captured");
    let binding = discovery.graph().lookup(request.file, &captured).unwrap();
    code.bindings.push((captured.root, binding));
    code.values.push((
        name(discovery.graph(), "portable").root,
        SourceCaptureValue::String(b"raw\0bytes".to_vec().into_boxed_slice()),
    ));
    code.origins.push(request.location);
    let original = code.clone();
    let frontier = discovery.builder.clone();
    let source = original.location.source;
    assert!(Arc::ptr_eq(
        &discovery
            .graph()
            .sources()
            .get(source)
            .unwrap()
            .shared_text(),
        &frontier.graph.sources().get(source).unwrap().shared_text(),
    ));
    let mut identities = discovery.builder.identities.clone();
    let scope = identities.scope();
    let declaration = identities.declaration();
    let admission = discovery.admit_insertion(request.id, &code).unwrap();
    code.items.clear();
    code.bindings.clear();
    code.values.clear();
    code.origins.clear();
    code.source_file = FileInstanceId(usize::MAX);
    let transaction = discovery
        .prepare_insertion_admitted(request.id, admission)
        .unwrap();
    let publication = discovery.commit_insertion(transaction).unwrap();
    let file = publication.file;
    assert_eq!(publication.declarations, [declaration]);
    assert_eq!(publication.code.items.len(), 1);
    assert_eq!(publication.code.location, original.location);
    assert_eq!(publication.code.source_file, original.source_file);
    assert_eq!(publication.code.bindings, original.bindings);
    assert_eq!(publication.code.values, original.values);
    assert_eq!(publication.code.origins, original.origins);
    assert_eq!(discovery.graph().file(file).unwrap().scope(), scope);
}

#[test]
fn cloned_seals_are_bound_to_the_request_session_and_consumed_generation() {
    let inputs = Inputs::new("QUOTE :: #code { answer :: 42; }; #insert QUOTE; #insert QUOTE;");
    let mut discovery = inputs.discovery();
    let requests: Vec<_> = discovery.pending_insertion_requests().cloned().collect();
    let code = quote(discovery.graph(), "QUOTE", Visibility::Export);
    let admission = discovery.admit_insertion(requests[0].id, &code).unwrap();
    assert_eq!(
        discovery.prepare_insertion_admitted(requests[1].id, admission.clone()),
        Err(InsertionResponseError::StaleAdmission)
    );
    let other = inputs.discovery();
    assert!(matches!(
        other.admit_insertion(requests[0].id, &code),
        Err(InsertionPublicationError::Response(
            InsertionResponseError::UnknownRequest
        ))
    ));
    let transaction = discovery
        .prepare_insertion_admitted(requests[0].id, admission.clone())
        .unwrap();
    assert_eq!(
        discovery.prepare_insertion_admitted(requests[0].id, admission.clone()),
        Err(InsertionResponseError::AlreadyStaged)
    );
    discovery.cancel_insertion(transaction).unwrap();
    assert_eq!(
        discovery.prepare_insertion_admitted(requests[0].id, admission),
        Err(InsertionResponseError::StaleAdmission)
    );
    let admission = discovery.admit_insertion(requests[0].id, &code).unwrap();
    let transaction = discovery
        .prepare_insertion_admitted(requests[0].id, admission.clone())
        .unwrap();
    discovery.commit_insertion(transaction).unwrap();
    assert_eq!(
        discovery.prepare_insertion_admitted(requests[0].id, admission),
        Err(InsertionResponseError::StaleAdmission)
    );
    assert!(matches!(
        discovery.commit_insertion(transaction),
        Err(InsertionPublicationError::Response(
            InsertionResponseError::StaleTransaction
        ))
    ));
}

#[test]
fn placeholder_publication_and_discovery_advance_retire_frontier_proofs() {
    let inputs = Inputs::new(
        "#placeholder ANSWER; FIRST :: #code { ANSWER :: 42; }; SECOND :: #code { other :: 7; }; #insert FIRST; #insert SECOND;",
    );
    let mut discovery = inputs.discovery();
    let requests: Vec<_> = discovery.pending_insertion_requests().cloned().collect();
    let first = quote(discovery.graph(), "FIRST", Visibility::Export);
    let second = quote(discovery.graph(), "SECOND", Visibility::Export);
    let first_admission = discovery.admit_insertion(requests[0].id, &first).unwrap();
    let second_admission = discovery.admit_insertion(requests[1].id, &second).unwrap();
    let marker = discovery.graph().placeholders()[0].id();
    assert_eq!(discovery.graph().placeholder_binding(marker), None);
    let transaction = discovery
        .prepare_insertion_admitted(requests[0].id, first_admission)
        .unwrap();
    discovery.commit_insertion(transaction).unwrap();
    assert!(discovery.graph().placeholder_binding(marker).is_some());
    assert_eq!(
        discovery.prepare_insertion_admitted(requests[1].id, second_admission),
        Err(InsertionResponseError::StaleAdmission)
    );
    let second_admission = discovery.admit_insertion(requests[1].id, &second).unwrap();
    let transaction = discovery
        .prepare_insertion_admitted(requests[1].id, second_admission)
        .unwrap();
    discovery.advance().unwrap();
    assert!(matches!(
        discovery.commit_insertion(transaction),
        Err(InsertionPublicationError::Response(
            InsertionResponseError::StaleAdmission
        ))
    ));
    assert_eq!(discovery.graph().insertion_publications().len(), 1);
    assert!(matches!(
        discovery.commit_insertion(transaction),
        Err(InsertionPublicationError::Response(
            InsertionResponseError::StaleTransaction
        ))
    ));
    let admission = discovery.admit_insertion(requests[1].id, &second).unwrap();
    let transaction = discovery
        .prepare_insertion_admitted(requests[1].id, admission)
        .unwrap();
    discovery.commit_insertion(transaction).unwrap();
    assert!(discovery.advance().unwrap().is_complete());
}

#[test]
fn using_bindings_retire_proofs_without_changing_graph_object_counts() {
    let inputs = Inputs::new(
        "Namespace :: struct { fresh :: 1; } using Namespace; QUOTE :: #code { fresh :: 2; }; #insert QUOTE;",
    );
    let mut discovery = inputs.discovery();
    let request = discovery
        .pending_insertion_requests()
        .next()
        .unwrap()
        .clone();
    let using = discovery.pending_using_requests().remove(0);
    let code = quote(discovery.graph(), "QUOTE", Visibility::Export);
    let admission = discovery.admit_insertion(request.id, &code).unwrap();
    let Binding::Declaration(namespace) = discovery
        .graph()
        .lookup(request.file, &name(discovery.graph(), "Namespace"))
        .unwrap()
    else {
        panic!("expected the original record namespace");
    };
    let counts = (
        discovery.graph().files().len(),
        discovery.graph().modules().len(),
        discovery.graph().declarations().len(),
        discovery.graph().overload_sets().len(),
    );
    discovery
        .resolve_using(
            using.id,
            FileUsingDecision {
                bindings: vec![UsingBinding {
                    name: "fresh".into(),
                    binding: Binding::SourceMember {
                        declaration: namespace,
                        member: name(discovery.graph(), "fresh").root,
                    },
                }],
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        (
            discovery.graph().files().len(),
            discovery.graph().modules().len(),
            discovery.graph().declarations().len(),
            discovery.graph().overload_sets().len(),
        ),
        counts
    );
    assert_eq!(
        discovery.prepare_insertion_admitted(request.id, admission),
        Err(InsertionResponseError::StaleAdmission)
    );
    assert!(matches!(
        discovery.admit_insertion(request.id, &code),
        Err(InsertionPublicationError::Graph(_))
    ));
}

#[test]
fn nested_private_collision_is_checked_against_every_real_destination() {
    let inputs = Inputs::new(
        "#scope_file; collision :: 9; OUTER :: #code { #insert INNER; }; INNER :: #code { okay :: 1; collision :: 2; }; #insert OUTER;",
    );
    let mut discovery = inputs.discovery();
    let outer = discovery
        .pending_insertion_requests()
        .next()
        .unwrap()
        .clone();
    let code = quote(discovery.graph(), "OUTER", Visibility::File);
    let admission = discovery.admit_insertion(outer.id, &code).unwrap();
    let transaction = discovery
        .prepare_insertion_admitted(outer.id, admission)
        .unwrap();
    let outer_file = discovery.commit_insertion(transaction).unwrap().file;
    discovery.advance().unwrap();
    let inner = discovery
        .pending_insertion_requests()
        .next()
        .unwrap()
        .clone();
    let mut code = quote(discovery.graph(), "INNER", Visibility::File);
    let before = format!("{:?}", discovery.graph());
    assert!(matches!(
        discovery.admit_insertion(inner.id, &code),
        Err(InsertionPublicationError::Graph(_))
    ));
    assert_eq!(format!("{:?}", discovery.graph()), before);
    let okay = name(discovery.graph(), "okay");
    assert!(discovery.graph().lookup(outer.file, &okay).is_err());
    assert!(discovery.graph().lookup(outer_file, &okay).is_err());
    code.items.truncate(1);
    let admission = discovery.admit_insertion(inner.id, &code).unwrap();
    let transaction = discovery
        .prepare_insertion_admitted(inner.id, admission)
        .unwrap();
    let binding = Binding::Declaration(
        discovery
            .commit_insertion(transaction)
            .unwrap()
            .declarations[0],
    );
    assert_eq!(discovery.graph().lookup(outer.file, &okay), Ok(binding));
    assert_eq!(discovery.graph().lookup(outer_file, &okay), Ok(binding));
}

#[test]
fn admission_of_original_imports_queues_them_without_provider_calls() {
    let inputs = Inputs::new(
        "QUOTE :: #code { #import,file \"missing.jai\"; answer :: 42; }; #insert QUOTE;",
    );
    let mut discovery = inputs.discovery();
    let request = discovery
        .pending_insertion_requests()
        .next()
        .unwrap()
        .clone();
    let code = quote(discovery.graph(), "QUOTE", Visibility::Export);
    let provider_calls = inputs.calls.get();
    let admission = discovery.admit_insertion(request.id, &code).unwrap();
    assert_eq!(inputs.calls.get(), provider_calls);
    assert!(discovery.graph().imports().is_empty());
    let transaction = discovery
        .prepare_insertion_admitted(request.id, admission)
        .unwrap();
    discovery.commit_insertion(transaction).unwrap();
    assert_eq!(inputs.calls.get(), provider_calls);
    assert!(discovery.graph().imports().is_empty());
}
