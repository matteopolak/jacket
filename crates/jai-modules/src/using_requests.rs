//! Source-backed using decisions publish canonical names and graph bindings.
use super::*;
use jai_syntax::{ExpressionKind, UsingDirective};
use std::{
    collections::HashSet,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_USING_SESSION: AtomicU64 = AtomicU64::new(1);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UsingRequestId {
    session: u64,
    index: usize,
}
impl UsingRequestId {
    pub fn index(self) -> usize {
        self.index
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsingBinding {
    pub name: String,
    pub binding: Binding,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsingAlias {
    pub source: Symbol,
    pub destination: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsingStorageMember {
    pub owner: DeclarationId,
    pub path: Vec<Symbol>,
    pub destination: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsingPlaceholder {
    pub name: String,
    pub placeholder: PlaceholderId,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileUsingDecision {
    pub bindings: Vec<UsingBinding>,
    pub selected_operator_declarations: Vec<DeclarationId>,
    pub source_module: Option<ModuleId>,
    pub aliases: Vec<UsingAlias>,
    pub storage_members: Vec<UsingStorageMember>,
    pub placeholders: Vec<UsingPlaceholder>,
}
#[derive(Clone, Debug)]
pub enum UsingDeclarationSource {
    File(Box<jai_syntax::FileDeclaration>),
    Statement(Box<jai_syntax::Statement>),
}
#[derive(Clone, Debug)]
pub struct FileUsingRequest {
    pub id: UsingRequestId,
    pub file: FileInstanceId,
    pub location: SourceSpan,
    pub directive: UsingDirective,
    pub declaration: Option<UsingDeclarationSource>,
    pub visibility: Visibility,
    pub owner: Option<DeclarationId>,
    pub context: DiscoveryConditionContext,
    pub specialization: Option<SourceSpecializationKey>,
    pub decision: Option<FileUsingDecision>,
}
#[derive(Clone, Debug)]
pub struct UsingPublication {
    pub file: FileInstanceId,
    pub location: SourceSpan,
    pub visibility: Visibility,
    pub owner: Option<DeclarationId>,
    pub specialization: Option<SourceSpecializationKey>,
    pub bindings: Vec<(Symbol, Binding)>,
    pub selected_operator_declarations: Vec<DeclarationId>,
    pub aliases: Vec<(Symbol, Symbol)>,
    pub placeholders: Vec<(Symbol, PlaceholderId)>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UsingResponseError {
    UnknownRequest,
    AlreadyResolved,
    DuplicateName,
    InvalidName,
    InvalidSourceIdentity,
    PrivateBinding,
    WrongSourceModule,
}
impl fmt::Display for UsingResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnknownRequest => "using request does not belong to this discovery session",
            Self::AlreadyResolved => "using request already has a different immutable decision",
            Self::DuplicateName => "using decision contains duplicate destination names",
            Self::InvalidName => "using destination is not one canonical source identifier",
            Self::InvalidSourceIdentity => {
                "using decision references an unavailable source binding"
            }
            Self::PrivateBinding => "using decision promotes a private namespace member",
            Self::WrongSourceModule => "using decision does not identify its source namespace",
        })
    }
}
impl std::error::Error for UsingResponseError {
}

#[derive(Clone)]
pub(super) struct UsingRequestStore {
    session: u64,
    requests: Vec<FileUsingRequest>,
}
impl Default for UsingRequestStore {
    fn default() -> Self {
        Self {
            session: NEXT_USING_SESSION.fetch_add(1, Ordering::Relaxed),
            requests: vec![],
        }
    }
}
impl UsingRequestStore {
    pub(super) fn is_ready(&self) -> bool {
        self.requests
            .iter()
            .all(|request| request.decision.is_some())
    }
}
impl GraphDiscovery<'_> {
    /// Retain an original source directive before semantic name evaluation.
    /// This staging API does not activate additional parser syntax.
    pub fn retain_using(
        &mut self,
        file: FileInstanceId,
        directive: UsingDirective,
        visibility: Visibility,
        context: DiscoveryConditionContext,
    ) -> Result<UsingRequestId, GraphError> {
        self.invalidate_insertion_admissions();
        let source = self
            .builder
            .graph
            .file(file)
            .and_then(|file| self.builder.graph.sources.get(file.source()))
            .ok_or_else(|| response_error(UsingResponseError::InvalidSourceIdentity))?;
        let span = directive.span;
        if span.start > span.end
            || span.end > source.text().len()
            || !source.text().is_char_boundary(span.start)
            || !source.text().is_char_boundary(span.end)
        {
            return Err(response_error(UsingResponseError::InvalidSourceIdentity));
        }
        if let DiscoveryConditionContext::Lexical {
            declaration, ..
        } = &context
            && self
                .builder
                .graph
                .declaration(*declaration)
                .is_none_or(|owner| owner.file() != file)
        {
            return Err(response_error(UsingResponseError::InvalidSourceIdentity));
        }
        let location = SourceSpan {
            source: source.id(),
            span,
        };
        match self
            .builder
            .defer_using(file, &directive, visibility, location, context)
        {
            Ok(())
            | Err(GraphError::Pending {
                ..
            }) => {}
            Err(error) => return Err(error),
        }
        let specialization = self.builder.specialization_for_span(file, span);
        self.builder
            .using_requests
            .requests
            .iter()
            .find(|request| {
                request.file == file
                    && request.location == location
                    && request.specialization.as_ref() == specialization
            })
            .map(|request| request.id)
            .ok_or_else(|| response_error(UsingResponseError::UnknownRequest))
    }
    pub fn using_requests(&self) -> Vec<FileUsingRequest> {
        self.builder.using_requests.requests.clone()
    }
    pub fn pending_using_requests(&self) -> Vec<FileUsingRequest> {
        self.using_requests()
            .into_iter()
            .filter(|request| request.decision.is_none())
            .collect()
    }
    pub fn resolve_using(
        &mut self,
        id: UsingRequestId,
        decision: FileUsingDecision,
    ) -> Result<(), GraphError> {
        self.invalidate_insertion_admissions();
        let request = self
            .builder
            .using_requests
            .requests
            .get(id.index)
            .filter(|request| request.id == id)
            .cloned()
            .ok_or_else(|| response_error(UsingResponseError::UnknownRequest))?;
        if let Some(previous) = request.decision {
            return if previous == decision {
                Ok(())
            } else {
                Err(response_error(UsingResponseError::AlreadyResolved))
            };
        }
        validate_decision(&self.builder.graph, &request, &decision).map_err(response_error)?;
        let storage_count = self.builder.graph.storage_members.len();
        let mut bindings = decision
            .bindings
            .iter()
            .map(|row| (self.builder.graph.symbols.intern(&row.name), row.binding))
            .collect::<Vec<_>>();
        for member in &decision.storage_members {
            let id = self.builder.graph.storage_members.intern(
                member.owner,
                &member.path,
                request.location,
            );
            bindings.push((
                self.builder.graph.symbols.intern(&member.destination),
                Binding::StorageMember(id),
            ));
        }
        let aliases = decision
            .aliases
            .iter()
            .map(|alias| {
                (
                    alias.source,
                    self.builder.graph.symbols.intern(&alias.destination),
                )
            })
            .collect();
        let placeholders = decision
            .placeholders
            .iter()
            .map(|row| {
                (
                    self.builder.graph.symbols.intern(&row.name),
                    row.placeholder,
                )
            })
            .collect::<Vec<_>>();
        if request.owner.is_none() {
            let module = self.builder.graph.files[request.file.index()].module;
            let private = self.builder.graph.files[request.file.index()]
                .private
                .clone();
            let bindings_before = self.builder.graph.modules[module.index()].bindings.clone();
            let exports_before = self.builder.graph.modules[module.index()].exports.clone();
            let overloads_before = self.builder.graph.overload_sets.len();
            let placeholders_before = self.builder.graph.placeholders.clone();
            for (name, binding) in &bindings {
                if let Err(error) = self.builder.bind(
                    request.file,
                    request.visibility,
                    *name,
                    *binding,
                    request.location,
                    true,
                ) {
                    self.builder.graph.files[request.file.index()].private = private;
                    self.builder.graph.modules[module.index()].bindings = bindings_before;
                    self.builder.graph.modules[module.index()].exports = exports_before;
                    self.builder.graph.overload_sets.truncate(overloads_before);
                    self.builder.graph.storage_members.truncate(storage_count);
                    self.builder.graph.placeholders = placeholders_before;
                    return Err(error);
                }
            }
            for &(name, marker) in &placeholders {
                if let Err(error) = self.builder.link_placeholder_import(
                    request.file,
                    request.visibility,
                    name,
                    marker,
                    request.location,
                ) {
                    self.builder.graph.files[request.file.index()].private = private;
                    self.builder.graph.modules[module.index()].bindings = bindings_before;
                    self.builder.graph.modules[module.index()].exports = exports_before;
                    self.builder.graph.overload_sets.truncate(overloads_before);
                    self.builder.graph.storage_members.truncate(storage_count);
                    self.builder.graph.placeholders = placeholders_before;
                    return Err(error);
                }
            }
        }
        self.builder
            .graph
            .using_publications
            .push(UsingPublication {
                file: request.file,
                location: request.location,
                visibility: request.visibility,
                owner: request.owner,
                specialization: request.specialization,
                bindings,
                aliases,
                placeholders,
                selected_operator_declarations: decision.selected_operator_declarations.clone(),
            });
        self.builder.using_requests.requests[id.index].decision = Some(decision);
        Ok(())
    }
}
fn response_error(error: UsingResponseError) -> GraphError {
    GraphError::InvalidUsingResponse(error)
}

impl ModuleGraph {
    pub fn using_publications(&self) -> &[UsingPublication] {
        &self.using_publications
    }
    pub fn using_publication(
        &self,
        file: FileInstanceId,
        span: jai_source::Span,
        specialization: Option<&SourceSpecializationKey>,
    ) -> Option<&UsingPublication> {
        self.using_publications.iter().find(|publication| {
            publication.file == file
                && publication.location.span == span
                && publication.specialization.as_ref() == specialization
        })
    }
}
impl Builder<'_> {
    pub(super) fn using_dependencies(&self) -> Vec<DeferredDependency> {
        self.using_requests
            .requests
            .iter()
            .filter(|request| request.decision.is_none())
            .map(|request| DeferredDependency {
                module: self.graph.files[request.file.index()].module,
                diagnostic: self.graph.diagnostic(
                    request.location,
                    "using promotion is awaiting semantic source evaluation",
                ),
            })
            .collect()
    }

    pub(super) fn defer_using_declaration(
        &mut self,
        file: FileInstanceId,
        directive: &UsingDirective,
        visibility: Visibility,
        location: SourceSpan,
        context: DiscoveryConditionContext,
        declaration: UsingDeclarationSource,
    ) -> Result<(), GraphError> {
        let result = self.defer_using(file, directive, visibility, location, context);
        let specialization = self.specialization_for_span(file, location.span).cloned();
        if let Some(request) = self.using_requests.requests.iter_mut().find(|request| {
            request.file == file
                && request.location == location
                && request.specialization == specialization
        }) {
            request.declaration = Some(declaration);
        }
        result
    }

    pub(super) fn defer_using(
        &mut self,
        file: FileInstanceId,
        directive: &UsingDirective,
        visibility: Visibility,
        location: SourceSpan,
        context: DiscoveryConditionContext,
    ) -> Result<(), GraphError> {
        let specialization = self.specialization_for_span(file, location.span).cloned();
        if let Some(request) = self.using_requests.requests.iter().find(|request| {
            request.file == file
                && request.location == location
                && request.specialization == specialization
        }) {
            if request.decision.is_some() {
                return Ok(());
            }
        } else {
            let owner = match &context {
                DiscoveryConditionContext::File => None,
                DiscoveryConditionContext::Lexical {
                    declaration, ..
                } => Some(*declaration),
            };
            self.using_requests.requests.push(FileUsingRequest {
                id: UsingRequestId {
                    session: self.using_requests.session,
                    index: self.using_requests.requests.len(),
                },
                file,
                location,
                directive: directive.clone(),
                declaration: None,
                visibility,
                owner,
                context,
                specialization,
                decision: None,
            });
        }
        let diagnostic = self.graph.diagnostic(
            location,
            "using promotion is awaiting semantic source evaluation",
        );
        let rendered = diagnostic.render(&self.graph.sources);
        Err(GraphError::Pending {
            diagnostic,
            rendered,
        })
    }
}

fn validate_decision(
    graph: &ModuleGraph,
    request: &FileUsingRequest,
    decision: &FileUsingDecision,
) -> Result<(), UsingResponseError> {
    let mut names = HashSet::new();
    for name in decision
        .bindings
        .iter()
        .map(|row| row.name.as_str())
        .chain(
            decision
                .aliases
                .iter()
                .map(|alias| alias.destination.as_str()),
        )
        .chain(
            decision
                .storage_members
                .iter()
                .map(|member| member.destination.as_str()),
        )
        .chain(decision.placeholders.iter().map(|row| row.name.as_str()))
    {
        let tokens = jai_lexer::lex(name).map_err(|_| UsingResponseError::InvalidName)?;
        if tokens.len() != 2
            || tokens[0].kind != jai_lexer::Kind::Ident
            || tokens[0].span != jai_source::Span::new(0, name.len())
            || tokens[0].spelling(name).as_ref() != name
        {
            return Err(UsingResponseError::InvalidName);
        }
        if !names.insert(name) {
            return Err(UsingResponseError::DuplicateName);
        }
    }
    if decision
        .aliases
        .iter()
        .any(|alias| graph.symbols.get(alias.source).is_none())
    {
        return Err(UsingResponseError::InvalidSourceIdentity);
    }
    if request.owner.is_none() && !decision.aliases.is_empty() {
        return Err(UsingResponseError::InvalidSourceIdentity);
    }
    for row in &decision.placeholders {
        let module = decision
            .source_module
            .ok_or(UsingResponseError::WrongSourceModule)?;
        if graph.placeholder(row.placeholder).is_none() {
            return Err(UsingResponseError::InvalidSourceIdentity);
        }
        if !graph
            .module_placeholder_exports(module)
            .iter()
            .any(|(_, marker)| *marker == row.placeholder)
        {
            return Err(UsingResponseError::PrivateBinding);
        }
    }
    for member in &decision.storage_members {
        let owner = graph
            .declaration(member.owner)
            .ok_or(UsingResponseError::InvalidSourceIdentity)?;
        if !matches!(owner.syntax().kind, FileDeclarationKind::Global(_))
            || member.path.is_empty()
            || member.path.len() > MAX_SOURCE_STORAGE_PATH_DEPTH
            || member
                .path
                .iter()
                .any(|name| graph.symbols.get(*name).is_none())
        {
            return Err(UsingResponseError::InvalidSourceIdentity);
        }
        let path = match &request.directive.target.kind {
            ExpressionKind::Name(name) => NamePath {
                root: *name,
                members: vec![],
            },
            ExpressionKind::QualifiedName(path) => path.clone(),
            _ => return Err(UsingResponseError::InvalidSourceIdentity),
        };
        let origin = (0..=path.members.len())
            .rev()
            .find_map(|len| {
                let binding = graph
                    .lookup(
                        request.file,
                        &NamePath {
                            root: path.root,
                            members: path.members[..len].to_vec(),
                        },
                    )
                    .ok()?;
                match binding {
                    Binding::Declaration(id)
                        if matches!(
                            graph.declaration(id)?.syntax().kind,
                            FileDeclarationKind::Global(_)
                        ) =>
                    {
                        Some((id, path.members[len..].to_vec()))
                    }
                    Binding::StorageMember(id) => {
                        let record = graph.source_storage_member(id)?;
                        let mut fields = record.path().to_vec();
                        fields.extend_from_slice(&path.members[len..]);
                        Some((record.owner(), fields))
                    }
                    _ => None,
                }
            })
            .ok_or(UsingResponseError::PrivateBinding)?;
        if origin.0 != member.owner || member.path[..member.path.len() - 1] != origin.1 {
            return Err(UsingResponseError::InvalidSourceIdentity);
        }
    }
    let target_path = match &request.directive.target.kind {
        ExpressionKind::Name(name) => Some(NamePath {
            root: *name,
            members: vec![],
        }),
        ExpressionKind::QualifiedName(path) => Some(path.clone()),
        _ => None,
    };
    let target_module = target_path
        .and_then(|path| graph.lookup(request.file, &path).ok())
        .and_then(|binding| {
            if let Binding::Module(module) = binding {
                Some(module)
            } else {
                None
            }
        });
    if target_module.is_some() && target_module != decision.source_module {
        return Err(UsingResponseError::WrongSourceModule);
    }
    let source_module = decision
        .source_module
        .map(|module| {
            graph
                .module(module)
                .ok_or(UsingResponseError::InvalidSourceIdentity)
        })
        .transpose()?;
    for row in &decision.bindings {
        let valid = match row.binding {
            Binding::Declaration(id) => graph.declaration(id).is_some(),
            Binding::OverloadSet(id) => graph.overload_set(id).is_some(),
            Binding::Module(id) => graph.module(id).is_some(),
            Binding::Parameter(id) => graph.parameter(id).is_some(),
            Binding::StorageMember(id) => graph.source_storage_member(id).is_some(),
            Binding::SourceMember {
                declaration,
                member,
            } => valid_source_member(graph, declaration, member),
        };
        if !valid {
            return Err(UsingResponseError::InvalidSourceIdentity);
        }
        if let Some(module) = source_module
            && !module
                .exports
                .values()
                .any(|binding| *binding == row.binding)
        {
            return Err(UsingResponseError::PrivateBinding);
        }
    }
    for id in &decision.selected_operator_declarations {
        let Some(declaration) = graph.declaration(*id) else {
            return Err(UsingResponseError::InvalidSourceIdentity);
        };
        let FileDeclarationKind::Procedure(procedure) = &declaration.syntax().kind else {
            return Err(UsingResponseError::InvalidSourceIdentity);
        };
        let Some(operator) = procedure.operator else {
            return Err(UsingResponseError::InvalidSourceIdentity);
        };
        if let Some(module) = decision.source_module
            && !graph
                .exported_operator_declarations(module, operator.kind)
                .contains(id)
        {
            return Err(UsingResponseError::PrivateBinding);
        }
    }
    Ok(())
}

pub(super) fn valid_source_member(graph: &ModuleGraph, owner: DeclarationId, name: Symbol) -> bool {
    let Some(declaration) = graph.declaration(owner) else {
        return false;
    };
    match &declaration.syntax().kind {
        FileDeclarationKind::Enum(enumeration) => {
            jai_syntax::EnumMemberSyntax::new(&enumeration.members)
                .any(|member| member.name == name)
        }
        FileDeclarationKind::Record(record) => record_static_member(&record.members, name),
        _ => false,
    }
}
fn record_static_member(members: &[jai_syntax::RecordMember], name: Symbol) -> bool {
    members.iter().any(|member| match member {
        jai_syntax::RecordMember::Constant(constant) => constant.name == name,
        jai_syntax::RecordMember::Procedure(procedure) => procedure.name == name,
        jai_syntax::RecordMember::ProcedurePrototype(procedure) => procedure.name == name,
        jai_syntax::RecordMember::TypeAlias(alias) => alias.name == name,
        jai_syntax::RecordMember::Record(record) => record.name == name,
        jai_syntax::RecordMember::Enum(enumeration) => enumeration.name == name,
        jai_syntax::RecordMember::Conditional {
            then_members,
            else_members,
            ..
        } => record_static_member(then_members, name) || record_static_member(else_members, name),
        jai_syntax::RecordMember::CompileTimeCases {
            cases, ..
        } => {
            cases
                .arms
                .iter()
                .any(|arm| record_static_member(&arm.body, name))
                || cases
                    .default
                    .as_ref()
                    .is_some_and(|default| record_static_member(&default.body, name))
        }
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Fixture(std::path::PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "jai-using-request-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).unwrap();
            fs::write(
                path.join("main.jai"),
                "Lib :: #import,file \"lib.jai\"; clash :: 7; main :: () {}",
            )
            .unwrap();
            fs::write(
                path.join("lib.jai"),
                "value :: 42; #scope_module hidden :: 3;",
            )
            .unwrap();
            Self(path)
        }
        fn discovery(&self) -> GraphDiscovery<'static> {
            let mut discovery = GraphDiscovery::new(
                &self.0.join("main.jai"),
                GraphOptions::default(),
                &Filesystem,
            )
            .unwrap();
            assert!(discovery.advance().unwrap().is_complete());
            discovery
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn retain(
        discovery: &mut GraphDiscovery<'_>,
    ) -> (UsingRequestId, FileInstanceId, ModuleId, Binding) {
        let graph = discovery.graph();
        let file = graph.module(graph.root()).unwrap().entry();
        let name = graph.symbols().find("Lib").unwrap();
        let Binding::Module(module) = graph
            .lookup(
                file,
                &NamePath {
                    root: name,
                    members: vec![],
                },
            )
            .unwrap()
        else {
            panic!()
        };
        let value =
            graph.module(module).unwrap().exports()[&graph.symbols().find("value").unwrap()];
        let span = jai_source::Span::new(0, 3);
        let id = discovery
            .retain_using(
                file,
                UsingDirective {
                    target: jai_syntax::Expression {
                        kind: ExpressionKind::Name(name),
                        span,
                    },
                    selection: jai_syntax::UsingSelection::All,
                    span,
                },
                Visibility::Export,
                DiscoveryConditionContext::File,
            )
            .unwrap();
        (id, file, module, value)
    }
    #[test]
    fn parsed_pending_using_prevents_same_spelled_global_guard_selection() {
        let fixture = Fixture::new();
        fs::write(fixture.0.join("main.jai"), "Lib::#import,file \"lib.jai\"; Enabled::false; main::(){using Lib; #if Enabled { #import,file \"lib.jai\"; } else { #import,file \"missing.jai\"; }}").unwrap();
        fs::write(fixture.0.join("lib.jai"), "Enabled::true;").unwrap();
        let mut discovery = GraphDiscovery::new(
            &fixture.0.join("main.jai"),
            GraphOptions::default(),
            &Filesystem,
        )
        .unwrap();
        assert!(!discovery.advance().unwrap().is_complete());
        assert_eq!(discovery.pending_using_requests().len(), 1);
        assert_eq!(discovery.pending_conditions().count(), 1);
        assert!(discovery.graph().source_condition_selections().is_empty());
        assert!(
            discovery
                .graph()
                .sources()
                .records()
                .iter()
                .all(|source| !source.path().ends_with("missing.jai"))
        );
    }
    #[test]
    fn declaration_requests_retain_original_child_and_name_span() {
        for source in [
            "using Choice::enum{ ANSWER::42; } main::(){}",
            "Record::struct{value:int;} main::(){using record:Record;}",
        ] {
            let fixture = Fixture::new();
            fs::write(fixture.0.join("main.jai"), source).unwrap();
            let mut discovery = GraphDiscovery::new(
                &fixture.0.join("main.jai"),
                GraphOptions::default(),
                &Filesystem,
            )
            .unwrap();
            assert!(!discovery.advance().unwrap().is_complete());
            let requests = discovery.pending_using_requests();
            assert_eq!(requests.len(), 1);
            let request = &requests[0];
            let child_span = match request.declaration.as_ref().unwrap() {
                UsingDeclarationSource::File(child) => {
                    assert!(
                        discovery
                            .graph()
                            .declarations()
                            .iter()
                            .any(|declaration| declaration.file == request.file
                                && declaration.syntax.location == child.location)
                    );
                    child.location.span
                }
                UsingDeclarationSource::Statement(child) => child.span,
            };
            assert!(request.directive.target.span.end < child_span.end);
            assert!(request.location.span.start < child_span.start);
        }
    }

    #[test]
    fn lexical_alias_publication_keeps_original_and_destination_distinct() {
        let fixture = Fixture::new();
        let mut discovery = fixture.discovery();
        let graph = discovery.graph();
        let file = graph.module(graph.root()).unwrap().entry();
        let source = graph.symbols().find("clash").unwrap();
        let main = graph.symbols().find("main").unwrap();
        let Binding::Declaration(owner) = graph
            .lookup(
                file,
                &NamePath {
                    root: main,
                    members: vec![],
                },
            )
            .unwrap()
        else {
            panic!()
        };
        let span = jai_source::Span::new(0, 3);
        let id = discovery
            .retain_using(
                file,
                UsingDirective {
                    target: jai_syntax::Expression {
                        kind: ExpressionKind::Name(source),
                        span,
                    },
                    selection: jai_syntax::UsingSelection::All,
                    span,
                },
                Visibility::File,
                DiscoveryConditionContext::Lexical {
                    declaration: owner,
                    scopes: vec![],
                },
            )
            .unwrap();
        discovery
            .resolve_using(
                id,
                FileUsingDecision {
                    aliases: vec![UsingAlias {
                        source,
                        destination: "renamed_place".into(),
                    }],
                    ..Default::default()
                },
            )
            .unwrap();
        let graph = discovery.graph();
        let destination = graph.symbols().find("renamed_place").unwrap();
        assert_ne!(source, destination);
        let publication = graph.using_publication(file, span, None).unwrap();
        assert_eq!(publication.owner, Some(owner));
        assert_eq!(publication.aliases, vec![(source, destination)]);
        assert!(
            graph
                .lookup(
                    file,
                    &NamePath {
                        root: destination,
                        members: vec![]
                    }
                )
                .is_err()
        );
    }
    #[test]
    fn unresolved_requests_prevent_freezing_and_publication_preserves_identity() {
        let fixture = Fixture::new();
        let mut discovery = fixture.discovery();
        let (id, file, module, value) = retain(&mut discovery);
        assert!(!discovery.advance().unwrap().is_complete());
        let mut discovery = *discovery.into_graph().unwrap_err();
        let decision = FileUsingDecision {
            bindings: vec![UsingBinding {
                name: "published".into(),
                binding: value,
            }],
            source_module: Some(module),
            ..Default::default()
        };
        discovery.resolve_using(id, decision.clone()).unwrap();
        discovery.resolve_using(id, decision).unwrap();
        assert!(discovery.advance().unwrap().is_complete());
        let graph = discovery.into_graph().unwrap();
        let name = graph.symbols().find("published").unwrap();
        assert_eq!(
            graph
                .lookup(
                    file,
                    &NamePath {
                        root: name,
                        members: vec![]
                    }
                )
                .unwrap(),
            value
        );
        assert_eq!(graph.using_publications().len(), 1);
    }
    #[test]
    fn collision_rollback_and_response_privacy_are_checked_before_publication() {
        let fixture = Fixture::new();
        let mut discovery = fixture.discovery();
        let (id, file, module, value) = retain(&mut discovery);
        let decision = FileUsingDecision {
            bindings: vec![
                UsingBinding {
                    name: "published".into(),
                    binding: value,
                },
                UsingBinding {
                    name: "clash".into(),
                    binding: value,
                },
            ],
            source_module: Some(module),
            ..Default::default()
        };
        assert!(matches!(
            discovery.resolve_using(id, decision),
            Err(GraphError::Located { .. })
        ));
        assert!(discovery.graph().using_publications().is_empty());
        let name = discovery.graph().symbols().find("published").unwrap();
        assert!(
            discovery
                .graph()
                .lookup(
                    file,
                    &NamePath {
                        root: name,
                        members: vec![]
                    }
                )
                .is_err()
        );
        let hidden = discovery.graph().module(module).unwrap().bindings()
            [&discovery.graph().symbols().find("hidden").unwrap()];
        let private = FileUsingDecision {
            bindings: vec![UsingBinding {
                name: "stolen".into(),
                binding: hidden,
            }],
            source_module: Some(module),
            ..Default::default()
        };
        assert!(matches!(
            discovery.resolve_using(id, private),
            Err(GraphError::InvalidUsingResponse(
                UsingResponseError::PrivateBinding
            ))
        ));
        assert!(discovery.graph().symbols().find("stolen").is_none());
    }
    #[test]
    fn names_and_request_sessions_are_not_fabricated() {
        let fixture = Fixture::new();
        let mut first = fixture.discovery();
        let mut second = fixture.discovery();
        let (id, _, module, value) = retain(&mut first);
        retain(&mut second);
        let decision = FileUsingDecision {
            bindings: vec![UsingBinding {
                name: "padded ".into(),
                binding: value,
            }],
            source_module: Some(module),
            ..Default::default()
        };
        assert!(matches!(
            first.resolve_using(id, decision),
            Err(GraphError::InvalidUsingResponse(
                UsingResponseError::InvalidName
            ))
        ));
        assert!(matches!(
            second.resolve_using(id, FileUsingDecision::default()),
            Err(GraphError::InvalidUsingResponse(
                UsingResponseError::UnknownRequest
            ))
        ));
        assert!(first.graph().using_publications().is_empty());
    }
}
