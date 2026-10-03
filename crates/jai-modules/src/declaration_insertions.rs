//! Original declaration syntax is staged before a graph publication transaction.
use super::*;
use jai_syntax::{FileItem, InsertDirective};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

static INSERTION_SESSIONS: AtomicU64 = AtomicU64::new(1);

/// Identifies a source insertion in one retained graph discovery session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InsertionRequestId {
    session: u64,
    index: usize,
}
impl InsertionRequestId {
    pub fn index(self) -> usize {
        self.index
    }
}

/// A receipt is consumed before the graph can publish the staged source body.
/// Cancelling it invalidates its generation without changing the source request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InsertionTransaction {
    request: InsertionRequestId,
    generation: u64,
    admitted_revision: Option<u64>,
}
impl InsertionTransaction {
    pub fn request(self) -> InsertionRequestId {
        self.request
    }
    pub(super) fn admitted_revision(self) -> Option<u64> {
        self.admitted_revision
    }
}

/// Sealed admission of the exact source payload on one immutable discovery
/// frontier. Clones retain the same request generation and cannot be replayed
/// after staging, cancellation, publication, or a discovery mutation.
#[derive(Clone, Debug)]
pub struct InsertionAdmission {
    pub(super) request: InsertionRequestId,
    pub(super) generation: u64,
    pub(super) revision: u64,
    pub(super) code: Arc<DeclarationInsertionCode>,
}

/// Registry-independent captured source. All declaration syntax remains in its
/// original source coordinates; destination publication is a separate operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceCaptureValue {
    Scalar(jai_eval::Value),
    Type(ModuleType),
    Enumeration(EnumParameter),
    String(Box<[u8]>),
}
impl From<ModuleBoundArgument> for SourceCaptureValue {
    fn from(value: ModuleBoundArgument) -> Self {
        match value {
            ModuleBoundArgument::Type(value) => Self::Type(value),
            ModuleBoundArgument::Integer(value) => Self::Scalar(jai_eval::Value::Int(value)),
            ModuleBoundArgument::Boolean(value) => Self::Scalar(jai_eval::Value::Bool(value)),
            ModuleBoundArgument::Float(value) => Self::Scalar(jai_eval::Value::Float(value)),
            ModuleBoundArgument::Enumeration(value) => Self::Enumeration(value),
            ModuleBoundArgument::String(value) => Self::String(value),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DeclarationInsertionCode {
    /// Lexical lookup origin, which can differ from the quote's source instance.
    pub file: FileInstanceId,
    pub source_file: FileInstanceId,
    pub location: SourceSpan,
    pub items: Vec<FileItem>,
    pub bindings: Vec<(Symbol, Binding)>,
    pub values: Vec<(Symbol, SourceCaptureValue)>,
    pub checks: jai_syntax::SafetyChecks,
    pub debug: jai_types::DebugPolicy,
    pub origins: Vec<SourceSpan>,
}

#[derive(Clone, Debug)]
pub struct DeclarationInsertionRequest {
    pub id: InsertionRequestId,
    pub file: FileInstanceId,
    pub directive: InsertDirective,
    pub visibility: Visibility,
    pub location: SourceSpan,
    pub publication: Option<usize>,
    generation: u64,
    staged: Option<Arc<DeclarationInsertionCode>>,
}

/// Genuine declarations and their own expansion scope share the original source
/// record. This records where names were published and where their code was quoted.
#[derive(Clone, Debug)]
pub struct DeclarationInsertionPublication {
    pub request: InsertionRequestId,
    pub destination: FileInstanceId,
    pub file: FileInstanceId,
    pub location: SourceSpan,
    pub code: Arc<DeclarationInsertionCode>,
    pub declarations: Vec<DeclarationId>,
    pub scope: jai_syntax::InsertScope,
    pub(super) own_names: std::collections::HashSet<Symbol>,
}

#[derive(Debug)]
pub enum InsertionPublicationError {
    Response(InsertionResponseError),
    Graph(GraphError),
}
impl fmt::Display for InsertionPublicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Response(error) => error.fmt(formatter),
            Self::Graph(error) => error.fmt(formatter),
        }
    }
}
impl std::error::Error for InsertionPublicationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Response(error) => Some(error),
            Self::Graph(error) => Some(error),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsertionResponseError {
    UnknownRequest,
    AlreadyStaged,
    AlreadyPublished,
    StaleTransaction,
    StaleAdmission,
    InvalidSource,
    InvalidBinding,
    DuplicateCapture,
    InvalidModuleHeader,
    CyclicInsertion,
    ExpansionDepth,
    FailedDiscovery,
}
impl fmt::Display for InsertionResponseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnknownRequest => "insertion request belongs to another discovery session",
            Self::AlreadyStaged => "insertion request already has a staged response",
            Self::AlreadyPublished => "insertion request has already published declarations",
            Self::StaleTransaction => "insertion transaction was consumed or cancelled",
            Self::StaleAdmission => {
                "insertion admission no longer matches the source request or discovery frontier"
            }
            Self::InvalidSource => "insertion response has invalid original source provenance",
            Self::InvalidBinding => "insertion capture references an unavailable source binding",
            Self::DuplicateCapture => "insertion capture repeats a bound source name",
            Self::InvalidModuleHeader => "inserted source cannot introduce #module_parameters",
            Self::CyclicInsertion => "cyclic captured declaration insertion",
            Self::ExpansionDepth => "declaration insertion exceeds the expansion depth limit",
            Self::FailedDiscovery => "cannot publish an insertion into a failed discovery session",
        })
    }
}
impl std::error::Error for InsertionResponseError {}

#[derive(Clone)]
pub(super) struct InsertionRequestStore {
    session: u64,
    pub(super) requests: Vec<DeclarationInsertionRequest>,
}
impl Default for InsertionRequestStore {
    fn default() -> Self {
        Self {
            session: INSERTION_SESSIONS.fetch_add(1, Ordering::Relaxed),
            requests: vec![],
        }
    }
}
impl InsertionRequestStore {
    pub(super) fn retain(
        &mut self,
        file: FileInstanceId,
        directive: &InsertDirective,
        visibility: Visibility,
        location: SourceSpan,
    ) -> InsertionRequestId {
        if let Some(request) = self
            .requests
            .iter()
            .find(|request| request.file == file && request.location == location)
        {
            return request.id;
        }
        let id = InsertionRequestId {
            session: self.session,
            index: self.requests.len(),
        };
        self.requests.push(DeclarationInsertionRequest {
            id,
            file,
            directive: directive.clone(),
            visibility,
            location,
            publication: None,
            generation: 0,
            staged: None,
        });
        id
    }
    pub(super) fn request(
        &self,
        id: InsertionRequestId,
    ) -> Result<&DeclarationInsertionRequest, InsertionResponseError> {
        self.requests
            .get(id.index)
            .filter(|request| request.id == id)
            .ok_or(InsertionResponseError::UnknownRequest)
    }
    fn request_mut(
        &mut self,
        id: InsertionRequestId,
    ) -> Result<&mut DeclarationInsertionRequest, InsertionResponseError> {
        self.requests
            .get_mut(id.index)
            .filter(|request| request.id == id)
            .ok_or(InsertionResponseError::UnknownRequest)
    }
    pub(super) fn stage(
        &mut self,
        id: InsertionRequestId,
        code: Arc<DeclarationInsertionCode>,
    ) -> Result<InsertionTransaction, InsertionResponseError> {
        self.stage_with_revision(id, code, None)
    }
    pub(super) fn admission_generation(
        &self,
        id: InsertionRequestId,
    ) -> Result<u64, InsertionResponseError> {
        let request = self.request(id)?;
        if request.publication.is_some() {
            return Err(InsertionResponseError::AlreadyPublished);
        }
        if request.staged.is_some() {
            return Err(InsertionResponseError::AlreadyStaged);
        }
        Ok(request.generation)
    }
    pub(super) fn stage_admitted(
        &mut self,
        id: InsertionRequestId,
        admission: InsertionAdmission,
    ) -> Result<InsertionTransaction, InsertionResponseError> {
        if admission.request != id || self.admission_generation(id)? != admission.generation {
            return Err(InsertionResponseError::StaleAdmission);
        }
        self.stage_with_revision(id, admission.code, Some(admission.revision))
    }
    fn stage_with_revision(
        &mut self,
        id: InsertionRequestId,
        code: Arc<DeclarationInsertionCode>,
        admitted_revision: Option<u64>,
    ) -> Result<InsertionTransaction, InsertionResponseError> {
        let request = self.request_mut(id)?;
        if request.publication.is_some() {
            return Err(InsertionResponseError::AlreadyPublished);
        }
        if request.staged.is_some() {
            return Err(InsertionResponseError::AlreadyStaged);
        }
        request.generation = request
            .generation
            .checked_add(1)
            .expect("insertion transaction generation space exhausted");
        request.staged = Some(code);
        Ok(InsertionTransaction {
            request: id,
            generation: request.generation,
            admitted_revision,
        })
    }
    pub(super) fn staged(
        &self,
        transaction: InsertionTransaction,
    ) -> Result<Arc<DeclarationInsertionCode>, InsertionResponseError> {
        let request = self.request(transaction.request)?;
        if request.generation != transaction.generation {
            return Err(InsertionResponseError::StaleTransaction);
        }
        request
            .staged
            .clone()
            .ok_or(InsertionResponseError::StaleTransaction)
    }
    pub(super) fn cancel(
        &mut self,
        transaction: InsertionTransaction,
    ) -> Result<(), InsertionResponseError> {
        self.staged(transaction)?;
        self.request_mut(transaction.request)?.staged = None;
        Ok(())
    }
    pub(super) fn publish(
        &mut self,
        transaction: InsertionTransaction,
        publication: usize,
    ) -> Result<(), InsertionResponseError> {
        self.staged(transaction)?;
        let request = self.request_mut(transaction.request)?;
        request.publication = Some(publication);
        request.staged = None;
        Ok(())
    }
    pub(super) fn is_ready(&self) -> bool {
        self.requests
            .iter()
            .all(|request| request.publication.is_some())
    }
}

/// Validate a response while the source evaluator still owns its effects token.
/// Only canonical source identities can survive retirement of its semantic arena.
pub(super) fn validate_code(
    graph: &ModuleGraph,
    code: &DeclarationInsertionCode,
) -> Result<(), InsertionResponseError> {
    if graph.file(code.file).is_none() {
        return Err(InsertionResponseError::InvalidSource);
    }
    let Some(file) = graph.file(code.source_file) else {
        return Err(InsertionResponseError::InvalidSource);
    };
    if file.source() != code.location.source || !valid_location(graph, code.location) {
        return Err(InsertionResponseError::InvalidSource);
    }
    if code
        .origins
        .iter()
        .any(|origin| !valid_location(graph, *origin))
    {
        return Err(InsertionResponseError::InvalidSource);
    }
    let mut names = std::collections::HashSet::new();
    for (name, binding) in &code.bindings {
        if graph.symbols.get(*name).is_none() || !valid_binding(graph, *binding) {
            return Err(InsertionResponseError::InvalidBinding);
        }
        if !names.insert(*name) {
            return Err(InsertionResponseError::DuplicateCapture);
        }
    }
    for (name, value) in &code.values {
        let source_response = match value {
            SourceCaptureValue::Type(value) => Some(ParameterResponse::Type(value.clone())),
            SourceCaptureValue::Enumeration(value) => Some(ParameterResponse::Value(
                ParameterValue::Enumeration(*value),
            )),
            SourceCaptureValue::Scalar(_) | SourceCaptureValue::String(_) => None,
        };
        if graph.symbols.get(*name).is_none()
            || source_response.as_ref().is_some_and(|response| {
                crate::parameter_requests::validate_response(graph, response).is_err()
            })
        {
            return Err(InsertionResponseError::InvalidBinding);
        }
        if !names.insert(*name) {
            return Err(InsertionResponseError::DuplicateCapture);
        }
    }
    let mut work = vec![(code.items.as_slice(), 0usize)];
    let mut visited = 0usize;
    while let Some((items, depth)) = work.pop() {
        if depth > 128 {
            return Err(InsertionResponseError::ExpansionDepth);
        }
        for item in items {
            visited += 1;
            if visited > 100_000 {
                return Err(InsertionResponseError::ExpansionDepth);
            }
            let location = match item {
                FileItem::Declaration(declaration)
                | FileItem::UsingDeclaration { declaration, .. } => {
                    if graph.symbols.get(declaration_name(declaration)).is_none() {
                        return Err(InsertionResponseError::InvalidBinding);
                    }
                    declaration.location
                }
                FileItem::Import(import) => import.location,
                FileItem::Load(load) => load.location,
                FileItem::Run(run) => run.location,
                FileItem::Parameters(_) => return Err(InsertionResponseError::InvalidModuleHeader),
                FileItem::Conditional {
                    then_items,
                    else_items,
                    location,
                    ..
                } => {
                    work.push((then_items, depth + 1));
                    work.push((else_items, depth + 1));
                    *location
                }
                FileItem::CompileTimeCases { cases, location } => {
                    for arm in &cases.arms {
                        work.push((&arm.body, depth + 1));
                    }
                    if let Some(default) = &cases.default {
                        work.push((&default.body, depth + 1));
                    }
                    *location
                }
                FileItem::Insert { location, .. }
                | FileItem::ContextField { location, .. }
                | FileItem::Scope { location, .. }
                | FileItem::Using { location, .. }
                | FileItem::Assert { location, .. } => *location,
            };
            if location.source != code.location.source
                || !valid_location(graph, location)
                || location.span.start < code.location.span.start
                || location.span.end > code.location.span.end
            {
                return Err(InsertionResponseError::InvalidSource);
            }
        }
    }
    Ok(())
}

fn valid_location(graph: &ModuleGraph, location: SourceSpan) -> bool {
    graph.sources.get(location.source).is_some_and(|source| {
        source
            .text()
            .get(location.span.start..location.span.end)
            .is_some()
    })
}
fn valid_binding(graph: &ModuleGraph, binding: Binding) -> bool {
    match binding {
        Binding::Declaration(id) => graph.declaration(id).is_some(),
        Binding::SourceMember {
            declaration,
            member,
        } => crate::using_requests::valid_source_member(graph, declaration, member),
        Binding::OverloadSet(id) => graph.overload_set(id).is_some(),
        Binding::Module(id) => graph.module(id).is_some(),
        Binding::Parameter(id) => graph.parameter(id).is_some(),
        Binding::StorageMember(id) => graph.source_storage_member(id).is_some(),
    }
}
fn declaration_name(declaration: &FileDeclaration) -> Symbol {
    match &declaration.kind {
        FileDeclarationKind::Library(value) => value.name,
        FileDeclarationKind::Procedure(value) => value.name,
        FileDeclarationKind::OperatorAlias(value) => value.name,
        FileDeclarationKind::ProcedurePrototype(value) => value.name,
        FileDeclarationKind::Global(value) => value.declaration.name(),
        FileDeclarationKind::Constant(value) => value.name,
        FileDeclarationKind::Record(value) => value.name,
        FileDeclarationKind::Enum(value) => value.name,
        FileDeclarationKind::TypeAlias(value) => value.name,
        FileDeclarationKind::Placeholder(value) => value.name,
    }
}

impl ModuleGraph {
    /// Validate source transport and ancestry only. Source execution must use
    /// `GraphDiscovery::admit_insertion` before committing compiler effects,
    /// because declaration registration can also reject namespace collisions.
    pub fn validate_insertion_code(
        &self,
        mut destination: FileInstanceId,
        code: &DeclarationInsertionCode,
    ) -> Result<(), InsertionResponseError> {
        if self.file(destination).is_none() {
            return Err(InsertionResponseError::InvalidSource);
        }
        validate_code(self, code)?;
        let mut depth = 0;
        while let Some(publication) = self.insertion_publication(destination) {
            depth += 1;
            if depth > 128 {
                return Err(InsertionResponseError::ExpansionDepth);
            }
            if publication.code.source_file == code.source_file
                && publication.code.location == code.location
                && publication.code.bindings == code.bindings
                && publication.code.values == code.values
            {
                return Err(InsertionResponseError::CyclicInsertion);
            }
            destination = publication.destination;
        }
        Ok(())
    }
    pub fn insertion_publications(&self) -> &[DeclarationInsertionPublication] {
        &self.insertion_publications
    }
    pub fn insertion_publication(
        &self,
        file: FileInstanceId,
    ) -> Option<&DeclarationInsertionPublication> {
        self.insertion_publications
            .iter()
            .find(|publication| publication.file == file)
    }
    pub fn pending_insertions(&self) -> impl Iterator<Item = &FileInsertion> {
        self.insertions.iter().filter(|request| {
            !self.insertion_publications.iter().any(|publication| {
                publication.destination == request.file && publication.location == request.location
            })
        })
    }
    pub fn insertion_capture_value(
        &self,
        file: FileInstanceId,
        name: Symbol,
    ) -> Option<&SourceCaptureValue> {
        let publication = self.insertion_publication(file)?;
        if publication.scope != jai_syntax::InsertScope::Captured
            || publication.own_names.contains(&name)
        {
            return None;
        }
        publication
            .code
            .values
            .iter()
            .find(|(captured, _)| *captured == name)
            .map(|(_, value)| value)
    }
    pub fn insertion_checks(&self, file: FileInstanceId) -> Option<jai_syntax::SafetyChecks> {
        let publication = self.insertion_publication(file)?;
        (publication.scope == jai_syntax::InsertScope::Captured).then_some(publication.code.checks)
    }
    pub fn insertion_debug_policy(&self, file: FileInstanceId) -> Option<jai_types::DebugPolicy> {
        self.insertion_publication(file)
            .map(|publication| publication.code.debug)
    }
    pub(super) fn insertion_root_binding(
        &self,
        file: FileInstanceId,
        name: Symbol,
    ) -> Option<Result<Binding, LookupError>> {
        let publication = self.insertion_publication(file)?;
        let source = self.file(file).expect("published expansion file");
        if publication.own_names.contains(&name) {
            if let Some(binding) = self.placeholder_namespace_lookup(
                crate::placeholders::PlaceholderScope::File(self.canonical_binding_file(file)),
                name,
            ) {
                return Some(binding);
            }
            if let Some(binding) = source.private.get(&name).copied() {
                return Some(Ok(binding));
            }
            if let Some(binding) = self.placeholder_namespace_lookup(
                crate::placeholders::PlaceholderScope::Module(source.module),
                name,
            ) {
                return Some(binding);
            }
            return Some(
                self.modules[source.module.index()]
                    .bindings
                    .get(&name)
                    .copied()
                    .ok_or(LookupError::UnknownName(name)),
            );
        }
        if publication.scope == jai_syntax::InsertScope::Current {
            return Some(self.lookup(
                publication.destination,
                &NamePath {
                    root: name,
                    members: vec![],
                },
            ));
        }
        if let Some((_, binding)) = publication
            .code
            .bindings
            .iter()
            .find(|(captured, _)| *captured == name)
        {
            return Some(Ok(*binding));
        }
        // Portable values are resolved by semantic binding. Their lexical
        // shadow must not fall through to an unrelated graph declaration.
        if publication
            .code
            .values
            .iter()
            .any(|(captured, _)| *captured == name)
        {
            return Some(Err(LookupError::UnknownName(name)));
        }
        Some(self.lookup(
            publication.code.file,
            &NamePath {
                root: name,
                members: vec![],
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(store: &mut InsertionRequestStore) -> InsertionRequestId {
        let mut sources = SourceMap::default();
        let source = sources.insert("insert.jai".into(), "#insert code;".into());
        let parsed =
            jai_syntax::parse_file(sources.get(source).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Insert {
            directive,
            location,
        } = &parsed.items()[0]
        else {
            panic!()
        };
        store.retain(FileInstanceId(0), directive, Visibility::Export, *location)
    }
    fn code() -> Arc<DeclarationInsertionCode> {
        let mut sources = SourceMap::default();
        let source = sources.insert("quote.jai".into(), "value :: 42;".into());
        let parsed =
            jai_syntax::parse_file(sources.get(source).unwrap(), &mut Symbols::default()).unwrap();
        Arc::new(DeclarationInsertionCode {
            file: FileInstanceId(0),
            source_file: FileInstanceId(0),
            location: SourceSpan {
                source,
                span: jai_source::Span::new(0, 12),
            },
            items: parsed.items().to_vec(),
            bindings: vec![],
            values: vec![],
            checks: Default::default(),
            debug: Default::default(),
            origins: vec![],
        })
    }
    #[test]
    fn cancellation_retires_exact_receipt_and_preserves_the_source_request() {
        let mut store = InsertionRequestStore::default();
        let id = request(&mut store);
        assert_eq!(request(&mut store), id);
        let first = store.stage(id, code()).unwrap();
        assert!(!store.is_ready());
        store.cancel(first).unwrap();
        assert!(matches!(
            store.staged(first),
            Err(InsertionResponseError::StaleTransaction)
        ));
        let second = store.stage(id, code()).unwrap();
        assert_ne!(first, second);
        assert_eq!(
            store.publish(first, 0),
            Err(InsertionResponseError::StaleTransaction)
        );
        store.publish(second, 0).unwrap();
        assert!(store.is_ready());
        assert_eq!(store.request(id).unwrap().publication, Some(0));
        assert!(matches!(
            store.stage(id, code()),
            Err(InsertionResponseError::AlreadyPublished)
        ));
    }
    #[test]
    fn receipts_cannot_mutate_another_discovery_session() {
        let mut first = InsertionRequestStore::default();
        let id = request(&mut first);
        let receipt = first.stage(id, code()).unwrap();
        let mut second = InsertionRequestStore::default();
        assert_ne!(request(&mut second), id);
        assert_eq!(
            second.cancel(receipt),
            Err(InsertionResponseError::UnknownRequest)
        );
        assert!(first.staged(receipt).is_ok());
    }

    fn source_graph() -> ModuleGraph {
        let mut inputs = SourceOverlay::default();
        inputs
            .insert(Path::new("/insertion/quote.jai"), b"value :: 42;".to_vec())
            .unwrap();
        ModuleGraph::load_with_provider(
            Path::new("/insertion/quote.jai"),
            GraphOptions::default(),
            &inputs,
        )
        .unwrap()
    }

    #[test]
    fn source_validation_uses_actual_declarations_and_original_source_ranges() {
        let graph = source_graph();
        let declaration = &graph.declarations()[0];
        let mut code = DeclarationInsertionCode {
            file: declaration.file(),
            source_file: declaration.file(),
            location: declaration.location(),
            items: graph
                .file(declaration.file())
                .unwrap()
                .syntax()
                .items()
                .to_vec(),
            bindings: vec![(declaration.name(), Binding::Declaration(declaration.id()))],
            values: vec![],
            checks: Default::default(),
            debug: Default::default(),
            origins: vec![],
        };
        validate_code(&graph, &code).unwrap();
        code.location.span.end -= 1;
        assert_eq!(
            validate_code(&graph, &code),
            Err(InsertionResponseError::InvalidSource)
        );
        assert_eq!(graph.declarations().len(), 1);
        assert_eq!(graph.sources().records().len(), 1);
    }

    #[test]
    fn source_validation_rejects_capture_ambiguity_without_mutating_the_graph() {
        let graph = source_graph();
        let declaration = &graph.declarations()[0];
        let code = DeclarationInsertionCode {
            file: declaration.file(),
            source_file: declaration.file(),
            location: declaration.location(),
            items: vec![],
            bindings: vec![(declaration.name(), Binding::Declaration(declaration.id()))],
            values: vec![(
                declaration.name(),
                SourceCaptureValue::Scalar(jai_eval::Value::Bool(true)),
            )],
            checks: Default::default(),
            debug: Default::default(),
            origins: vec![],
        };
        assert_eq!(
            validate_code(&graph, &code),
            Err(InsertionResponseError::DuplicateCapture)
        );
        assert_eq!(
            graph
                .lookup(
                    declaration.file(),
                    &NamePath {
                        root: declaration.name(),
                        members: vec![],
                    }
                )
                .unwrap(),
            Binding::Declaration(declaration.id())
        );
    }

    #[test]
    fn original_quote_receipt_survives_a_different_lexical_lookup_file() {
        let mut inputs = SourceOverlay::default();
        inputs
            .insert(
                Path::new("/insertion/main.jai"),
                b"#load \"quote.jai\"; caller :: 1;".to_vec(),
            )
            .unwrap();
        inputs
            .insert(Path::new("/insertion/quote.jai"), b"quoted :: 42;".to_vec())
            .unwrap();
        let graph = ModuleGraph::load_with_provider(
            Path::new("/insertion/main.jai"),
            GraphOptions::default(),
            &inputs,
        )
        .unwrap();
        let destination = graph.module(graph.root()).unwrap().entry();
        let quote = graph
            .declarations()
            .iter()
            .find(|declaration| graph.symbols().name(declaration.name()) == "quoted")
            .unwrap();
        let mut code = DeclarationInsertionCode {
            file: destination,
            source_file: quote.file(),
            location: quote.location(),
            items: vec![FileItem::Declaration(quote.syntax().clone())],
            bindings: vec![],
            values: vec![],
            checks: Default::default(),
            debug: Default::default(),
            origins: vec![],
        };
        assert_ne!(
            graph.file(code.file).unwrap().source(),
            code.location.source
        );
        graph.validate_insertion_code(destination, &code).unwrap();
        code.source_file = destination;
        assert_eq!(
            graph.validate_insertion_code(destination, &code),
            Err(InsertionResponseError::InvalidSource)
        );
        assert_eq!(code.location, quote.location());
    }

    fn quoted_procedure(graph: &ModuleGraph) -> DeclarationInsertionCode {
        quoted_procedure_with_visibility(graph, Visibility::File)
    }

    fn quoted_procedure_with_visibility(
        graph: &ModuleGraph,
        visibility: Visibility,
    ) -> DeclarationInsertionCode {
        let quote = graph
            .declarations()
            .iter()
            .find(|declaration| graph.symbols().name(declaration.name()) == "QUOTE")
            .unwrap();
        let FileDeclarationKind::Constant(constant) = &quote.syntax().kind else {
            panic!()
        };
        let jai_syntax::ExpressionKind::Code(jai_syntax::CodeBody::Block(statements)) =
            &constant.initializer.kind
        else {
            panic!()
        };
        let items = statements
            .iter()
            .map(|statement| {
                let jai_syntax::StatementKind::Procedure(procedure) = &statement.kind else {
                    panic!()
                };
                FileItem::Declaration(FileDeclaration {
                    program_export: None,
                    visibility,
                    kind: FileDeclarationKind::Procedure(procedure.as_ref().clone()),
                    location: SourceSpan {
                        source: quote.location().source,
                        span: statement.span,
                    },
                })
            })
            .collect();
        DeclarationInsertionCode {
            file: quote.file(),
            source_file: quote.file(),
            location: SourceSpan {
                source: quote.location().source,
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

    #[test]
    fn repeated_original_quote_uses_independent_expansion_files_and_branch_caches() {
        let mut inputs = SourceOverlay::default();
        for (path, source) in [
            (
                "/insertion/main.jai",
                "#load \"first.jai\"; #load \"second.jai\"; QUOTE :: #code { generated :: () { #if #run true { Selected :: #import,file \"dependency.jai\"; } else { Missing :: #import,file \"absent.jai\"; } } };",
            ),
            ("/insertion/first.jai", "#scope_file; #insert QUOTE;"),
            ("/insertion/second.jai", "#scope_file; #insert QUOTE;"),
            ("/insertion/dependency.jai", "value :: 42;"),
        ] {
            inputs
                .insert(Path::new(path), source.as_bytes().to_vec())
                .unwrap();
        }
        let mut discovery = GraphDiscovery::new(
            Path::new("/insertion/main.jai"),
            GraphOptions::default(),
            &inputs,
        )
        .unwrap();
        assert!(!discovery.advance().unwrap().is_complete());
        let original = discovery
            .graph()
            .declarations()
            .iter()
            .map(|declaration| (declaration.id(), declaration.location()))
            .collect::<Vec<_>>();
        let requests = discovery
            .pending_insertion_requests()
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(requests.len(), 2);
        let code = quoted_procedure(discovery.graph());
        let first = discovery
            .prepare_insertion(requests[0].id, code.clone())
            .unwrap();
        let first_file = discovery.commit_insertion(first).unwrap().file;
        let second = discovery.prepare_insertion(requests[1].id, code).unwrap();
        let second_file = discovery.commit_insertion(second).unwrap().file;
        assert_ne!(first_file, second_file);
        assert!(!discovery.advance().unwrap().is_complete());
        let conditions = discovery.pending_conditions().cloned().collect::<Vec<_>>();
        assert_eq!(conditions.len(), 2);
        assert_ne!(conditions[0].file, conditions[1].file);
        assert_eq!(conditions[0].location, conditions[1].location);
        for condition in conditions {
            discovery.select_condition(condition.id, true).unwrap();
        }
        assert!(discovery.advance().unwrap().is_complete());
        let graph = discovery.into_graph().ok().unwrap();
        assert_eq!(
            &graph
                .declarations()
                .iter()
                .map(|declaration| (declaration.id(), declaration.location()))
                .collect::<Vec<_>>()[..original.len()],
            original
        );
        assert_eq!(graph.scoped_imports().len(), 2);
        assert_eq!(
            graph.scoped_imports()[0].module(),
            graph.scoped_imports()[1].module()
        );
        let generated = graph.symbols().find("generated").unwrap();
        let path = NamePath {
            root: generated,
            members: vec![],
        };
        let first_binding = graph.lookup(requests[0].file, &path).unwrap();
        let second_binding = graph.lookup(requests[1].file, &path).unwrap();
        assert_ne!(first_binding, second_binding);
        assert!(
            graph
                .lookup(graph.module(graph.root()).unwrap().entry(), &path)
                .is_err()
        );
        assert_eq!(graph.sources().records().len(), 4);
    }

    #[test]
    fn imported_insertion_destination_resumes_after_its_importer_is_complete() {
        let mut inputs = SourceOverlay::default();
        inputs
            .insert(
                Path::new("/insertion/main.jai"),
                b"Target :: #import, file \"target.jai\";".to_vec(),
            )
            .unwrap();
        inputs.insert(Path::new("/insertion/target.jai"),
            b"QUOTE :: #code { generated :: () -> int { Selected :: #import, file \"dependency.jai\"; return Selected.answer; } }; #insert QUOTE;".to_vec()).unwrap();
        inputs
            .insert(
                Path::new("/insertion/dependency.jai"),
                b"answer :: 42;".to_vec(),
            )
            .unwrap();
        let mut discovery = GraphDiscovery::new(
            Path::new("/insertion/main.jai"),
            GraphOptions::default(),
            &inputs,
        )
        .unwrap();
        assert!(!discovery.advance().unwrap().is_complete());
        let request = discovery
            .pending_insertion_requests()
            .next()
            .unwrap()
            .clone();
        let root = discovery
            .graph()
            .module(discovery.graph().root())
            .unwrap()
            .entry();
        assert_ne!(
            discovery.graph().file(request.file).unwrap().module(),
            discovery.graph().root()
        );
        let code = quoted_procedure_with_visibility(discovery.graph(), request.visibility);
        let receipt = discovery.prepare_insertion(request.id, code).unwrap();
        let generated_id = discovery.commit_insertion(receipt).unwrap().declarations[0];
        assert!(discovery.advance().unwrap().is_complete());
        let graph = discovery.graph();
        let path = NamePath {
            root: graph.symbols().find("Target").unwrap(),
            members: vec![graph.symbols().find("generated").unwrap()],
        };
        assert_eq!(
            graph.lookup(root, &path).unwrap(),
            Binding::Declaration(generated_id)
        );
        assert_eq!(graph.scoped_imports().len(), 1);
        assert_eq!(
            graph.scoped_imports()[0].file(),
            graph.declaration(generated_id).unwrap().file()
        );
        assert_eq!(graph.sources().records().len(), 3);
    }

    #[test]
    fn captured_file_names_and_operators_keep_the_original_private_namespace() {
        let mut inputs = SourceOverlay::default();
        inputs.insert(Path::new("/insertion/main.jai"),
            b"#scope_file; captured :: 1; Box :: struct { value:int; } operator + :: (a:Box,b:Box)->Box { return a; } #load \"quote.jai\"; #insert QUOTE; #insert QUOTE;".to_vec()).unwrap();
        inputs.insert(Path::new("/insertion/quote.jai"),
            b"#scope_file; captured :: 2; Box :: struct { value:int; } operator + :: (a:Box,b:int)->Box { return a; } #scope_module; QUOTE :: #code { generated :: ()->int { return captured; } };".to_vec()).unwrap();
        let mut discovery = GraphDiscovery::new(
            Path::new("/insertion/main.jai"),
            GraphOptions::default(),
            &inputs,
        )
        .unwrap();
        discovery.advance().unwrap();
        let request = discovery
            .pending_insertion_requests()
            .next()
            .unwrap()
            .clone();
        let mut code = quoted_procedure(discovery.graph());
        let quote_file = code.file;
        let captured = discovery.graph().symbols().find("captured").unwrap();
        let path = NamePath {
            root: captured,
            members: vec![],
        };
        let original = discovery.graph().lookup(quote_file, &path).unwrap();
        let destination = discovery.graph().lookup(request.file, &path).unwrap();
        assert_ne!(original, destination);
        let kind = jai_syntax::OperatorKind::Binary(jai_syntax::BinaryOp::Add);
        let quote_operators = discovery.graph().operator_declarations(quote_file, kind);
        let destination_operators = discovery.graph().operator_declarations(request.file, kind);
        assert_ne!(quote_operators, destination_operators);
        let receipt = discovery
            .prepare_insertion(request.id, code.clone())
            .unwrap();
        let inserted = discovery.commit_insertion(receipt).unwrap().file;
        assert_eq!(discovery.graph().lookup(inserted, &path).unwrap(), original);
        assert_eq!(
            discovery.graph().operator_declarations(inserted, kind),
            quote_operators
        );
        assert_eq!(
            discovery.graph().operator_declarations(request.file, kind),
            destination_operators
        );
        // A transported local value shadows graph-file names without becoming
        // a fabricated graph declaration.
        code.items.clear();
        code.values.push((
            captured,
            SourceCaptureValue::Scalar(jai_eval::Value::Literal(3)),
        ));
        let value_request = discovery.pending_insertion_requests().next().unwrap().id;
        let receipt = discovery.prepare_insertion(value_request, code).unwrap();
        let values_file = discovery.commit_insertion(receipt).unwrap().file;
        assert!(discovery.graph().lookup(values_file, &path).is_err());
        assert_eq!(
            discovery
                .graph()
                .insertion_capture_value(values_file, captured),
            Some(&SourceCaptureValue::Scalar(jai_eval::Value::Literal(3)))
        );
        assert!(discovery.advance().unwrap().is_complete());
    }

    #[test]
    fn failed_multideclaration_publication_rolls_back_bindings_and_identity_allocation() {
        let mut inputs = SourceOverlay::default();
        inputs.insert(Path::new("/insertion/main.jai"),
            b"#scope_file; collision :: 9; QUOTE :: #code { okay :: 1; collision :: 2; }; #insert QUOTE;".to_vec()).unwrap();
        let mut discovery = GraphDiscovery::new(
            Path::new("/insertion/main.jai"),
            GraphOptions::default(),
            &inputs,
        )
        .unwrap();
        discovery.advance().unwrap();
        let request = discovery
            .pending_insertion_requests()
            .next()
            .unwrap()
            .clone();
        let quote = discovery
            .graph()
            .declarations()
            .iter()
            .find(|declaration| discovery.graph().symbols().name(declaration.name()) == "QUOTE")
            .unwrap();
        let FileDeclarationKind::Constant(constant) = &quote.syntax().kind else {
            panic!()
        };
        let jai_syntax::ExpressionKind::Code(jai_syntax::CodeBody::Block(statements)) =
            &constant.initializer.kind
        else {
            panic!()
        };
        let items = statements
            .iter()
            .map(|statement| {
                let jai_syntax::StatementKind::Constant(constant) = &statement.kind else {
                    panic!()
                };
                FileItem::Declaration(FileDeclaration {
                    program_export: None,
                    visibility: Visibility::File,
                    kind: FileDeclarationKind::Constant(constant.clone()),
                    location: SourceSpan {
                        source: quote.location().source,
                        span: statement.span,
                    },
                })
            })
            .collect();
        let mut code = DeclarationInsertionCode {
            file: quote.file(),
            source_file: quote.file(),
            location: SourceSpan {
                source: quote.location().source,
                span: constant.initializer.span,
            },
            items,
            bindings: vec![],
            values: vec![],
            checks: Default::default(),
            debug: Default::default(),
            origins: vec![],
        };
        let declarations = discovery.graph().declarations().len();
        let files = discovery.graph().files().len();
        let collision = NamePath {
            root: discovery.graph().symbols().find("collision").unwrap(),
            members: vec![],
        };
        let previous = discovery.graph().lookup(request.file, &collision).unwrap();
        let transaction = discovery
            .prepare_insertion(request.id, code.clone())
            .unwrap();
        assert!(matches!(
            discovery.commit_insertion(transaction),
            Err(InsertionPublicationError::Graph(_))
        ));
        assert_eq!(discovery.graph().declarations().len(), declarations);
        assert_eq!(discovery.graph().files().len(), files);
        assert_eq!(
            discovery.graph().lookup(request.file, &collision).unwrap(),
            previous
        );
        assert!(discovery.graph().insertion_publications().is_empty());
        assert_eq!(
            discovery.cancel_insertion(transaction),
            Err(InsertionResponseError::StaleTransaction)
        );
        code.items.truncate(1);
        let retried = discovery.prepare_insertion(request.id, code).unwrap();
        let published = discovery.commit_insertion(retried).unwrap();
        assert_eq!(published.declarations.len(), 1);
        assert_eq!(published.declarations[0].index(), declarations);
        assert!(discovery.advance().unwrap().is_complete());
    }

    #[test]
    fn nested_failed_publication_restores_every_destination_scope() {
        let mut inputs = SourceOverlay::default();
        inputs.insert(Path::new("/insertion/main.jai"),
            b"#scope_file; collision :: 9; OUTER :: #code { #insert INNER; }; INNER :: #code { okay :: 1; collision :: 2; }; #insert OUTER;".to_vec()).unwrap();
        let mut discovery = GraphDiscovery::new(
            Path::new("/insertion/main.jai"),
            GraphOptions::default(),
            &inputs,
        )
        .unwrap();
        discovery.advance().unwrap();
        let outer_request = discovery
            .pending_insertion_requests()
            .next()
            .unwrap()
            .clone();
        let quote = |graph: &ModuleGraph, name: &str| {
            let declaration = graph
                .declarations()
                .iter()
                .find(|declaration| graph.symbols().name(declaration.name()) == name)
                .unwrap();
            let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
                panic!()
            };
            let jai_syntax::ExpressionKind::Code(jai_syntax::CodeBody::Block(statements)) =
                &constant.initializer.kind
            else {
                panic!()
            };
            let items = statements
                .iter()
                .map(|statement| {
                    let location = SourceSpan {
                        source: declaration.location().source,
                        span: statement.span,
                    };
                    match &statement.kind {
                        jai_syntax::StatementKind::Insert(directive) => FileItem::Insert {
                            directive: directive.clone(),
                            location,
                        },
                        jai_syntax::StatementKind::Constant(constant) => {
                            FileItem::Declaration(FileDeclaration {
                                program_export: None,
                                visibility: Visibility::File,
                                kind: FileDeclarationKind::Constant(constant.clone()),
                                location,
                            })
                        }
                        _ => panic!(),
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
        };
        let outer_code = quote(discovery.graph(), "OUTER");
        let receipt = discovery
            .prepare_insertion(outer_request.id, outer_code)
            .unwrap();
        let outer_file = discovery.commit_insertion(receipt).unwrap().file;
        discovery.advance().unwrap();
        let nested_request = discovery
            .pending_insertion_requests()
            .next()
            .unwrap()
            .clone();
        assert_eq!(nested_request.file, outer_file);
        assert_eq!(nested_request.visibility, Visibility::File);
        let mut inner_code = quote(discovery.graph(), "INNER");
        let declarations = discovery.graph().declarations().len();
        let receipt = discovery
            .prepare_insertion(nested_request.id, inner_code.clone())
            .unwrap();
        assert!(matches!(
            discovery.commit_insertion(receipt),
            Err(InsertionPublicationError::Graph(_))
        ));
        let okay = discovery.graph().symbols().find("okay").unwrap();
        let path = NamePath {
            root: okay,
            members: vec![],
        };
        assert!(discovery.graph().lookup(outer_request.file, &path).is_err());
        assert!(discovery.graph().lookup(outer_file, &path).is_err());
        assert!(
            !discovery
                .graph()
                .insertion_publication(outer_file)
                .unwrap()
                .own_names
                .contains(&okay)
        );
        assert_eq!(discovery.graph().declarations().len(), declarations);
        assert_eq!(discovery.graph().insertion_publications().len(), 1);
        inner_code.items.truncate(1);
        let receipt = discovery
            .prepare_insertion(nested_request.id, inner_code)
            .unwrap();
        let binding =
            Binding::Declaration(discovery.commit_insertion(receipt).unwrap().declarations[0]);
        assert_eq!(
            discovery.graph().lookup(outer_request.file, &path).unwrap(),
            binding
        );
        assert_eq!(
            discovery.graph().lookup(outer_file, &path).unwrap(),
            binding
        );
        assert!(discovery.advance().unwrap().is_complete());
    }
}
