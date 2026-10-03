//! A retained source dependency session. Semantic execution is a separate phase.
use super::*;
use jai_syntax::{Expression, Parameter, RecordMember, Statement};

/// Stable identity of one source condition within a discovery session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ConditionRequestId(pub(super) usize);
impl ConditionRequestId {
    pub fn index(self) -> usize {
        self.0
    }
}

/// Original lexical source inputs, ordered from the outermost scope inward.
#[derive(Clone, Debug, Default)]
pub struct DiscoveryLexicalScope {
    pub parameters: Vec<Parameter>,
    pub statements: Vec<Statement>,
    pub runtime_names: Vec<Symbol>,
    pub record_members: Vec<RecordMember>,
}

/// The defining environment of a condition; imports retain normal scope/privacy.
#[derive(Clone, Debug)]
pub enum DiscoveryConditionContext {
    File,
    Lexical {
        declaration: DeclarationId,
        scopes: Vec<DiscoveryLexicalScope>,
    },
}

/// A condition the scalar source evaluator could not decide.
/// `expression` is the original AST, never a substituted or synthetic body.
#[derive(Clone, Debug)]
pub struct DeferredCondition {
    pub id: ConditionRequestId,
    pub file: FileInstanceId,
    pub module: ModuleId,
    pub location: SourceSpan,
    pub expression: Expression,
    pub context: DiscoveryConditionContext,
    pub selected: Option<bool>,
    pub specialization: Option<SourceSpecializationKey>,
}

/// A source dependency awaiting a value other than a condition selection.
#[derive(Clone, Debug)]
pub struct DeferredDependency {
    pub module: ModuleId,
    pub diagnostic: LocatedDiagnostic,
}

#[derive(Clone, Debug)]
pub enum DiscoveryStatus {
    Complete,
    Awaiting {
        conditions: Vec<ConditionRequestId>,
        dependencies: Vec<DeferredDependency>,
    },
}
impl DiscoveryStatus {
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConditionSelectionError {
    UnknownRequest,
    AlreadySelected { previous: bool },
}
impl fmt::Display for ConditionSelectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownRequest => {
                f.write_str("condition request does not belong to this discovery session")
            }
            Self::AlreadySelected {
                previous,
            } => {
                write!(f, "condition has already been selected as {previous}")
            }
        }
    }
}
impl std::error::Error for ConditionSelectionError {
}

/// Retains parsed sources, canonical module instances, identities, and pending
/// work while the semantic phase resolves compile-time source conditions.
pub struct GraphDiscovery<'a> {
    pub(super) builder: Builder<'a>,
    pub(super) insertion_revision: u64,
    failed: Option<String>,
}
impl fmt::Debug for GraphDiscovery<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GraphDiscovery")
            .field("graph", &self.builder.graph)
            .field("conditions", &self.builder.conditions)
            .field("failed", &self.failed)
            .finish_non_exhaustive()
    }
}
impl<'a> GraphDiscovery<'a> {
    pub fn new(
        path: &Path,
        options: GraphOptions,
        provider: &'a dyn SourceProvider,
    ) -> Result<Self, GraphError> {
        Self::with_bootstrap(path, options, BootstrapOptions::disabled(), provider, None)
    }
    pub fn with_target(
        path: &Path,
        options: GraphOptions,
        provider: &'a dyn SourceProvider,
        target: jai_types::BuildTarget,
    ) -> Result<Self, GraphError> {
        Self::with_bootstrap(
            path,
            options,
            BootstrapOptions::disabled(),
            provider,
            Some(target),
        )
    }
    pub fn with_bootstrap(
        path: &Path,
        options: GraphOptions,
        bootstrap: BootstrapOptions,
        provider: &'a dyn SourceProvider,
        target: Option<jai_types::BuildTarget>,
    ) -> Result<Self, GraphError> {
        let mut builder = Builder::new_with_target(options, provider, target);
        builder.initialize(path, bootstrap)?;
        Ok(Self {
            builder,
            insertion_revision: 0,
            failed: None,
        })
    }
    /// A borrow of the current graph; existing identities survive every advance.
    pub fn graph(&self) -> &ModuleGraph {
        &self.builder.graph
    }
    pub(super) fn has_failed(&self) -> bool {
        self.failed.is_some()
    }
    /// Every mutable discovery entry point must retire proofs of its old frontier.
    pub(super) fn invalidate_insertion_admissions(&mut self) {
        self.insertion_revision = self
            .insertion_revision
            .checked_add(1)
            .expect("insertion admission revision space exhausted");
    }
    pub fn conditions(&self) -> &[DeferredCondition] {
        &self.builder.conditions
    }
    pub fn condition(&self, id: ConditionRequestId) -> Option<&DeferredCondition> {
        self.builder.conditions.get(id.index())
    }
    pub fn pending_conditions(&self) -> impl Iterator<Item = &DeferredCondition> {
        self.conditions()
            .iter()
            .filter(|condition| condition.selected.is_none())
    }
    /// Decisions are immutable once recorded. Repeating the same decision is safe.
    pub fn select_condition(
        &mut self,
        id: ConditionRequestId,
        selected: bool,
    ) -> Result<(), ConditionSelectionError> {
        self.invalidate_insertion_admissions();
        let condition = self
            .builder
            .conditions
            .get_mut(id.index())
            .ok_or(ConditionSelectionError::UnknownRequest)?;
        if let Some(previous) = condition.selected
            && previous != selected
        {
            return Err(ConditionSelectionError::AlreadySelected {
                previous,
            });
        }
        condition.selected = Some(selected);
        let (file, span, specialization) = (
            condition.file,
            condition.location.span,
            condition.specialization.clone(),
        );
        self.builder
            .record_semantic_selection(file, span, selected, specialization.as_ref());
        Ok(())
    }
    /// Run source discovery to a fixed point without evaluating semantic #run
    /// expressions or invoking compiler effects.
    pub fn advance(&mut self) -> Result<DiscoveryStatus, GraphError> {
        if let Some(rendered) = &self.failed {
            return Err(GraphError::FailedDiscovery {
                rendered: rendered.clone(),
            });
        }
        self.invalidate_insertion_admissions();
        match self.builder.advance() {
            Ok(status) => Ok(status),
            Err(error) => {
                self.failed = Some(error.to_string());
                Err(error)
            }
        }
    }
    /// Consume a fully discovered graph. Awaiting sessions remain resumable.
    pub fn into_graph(self) -> Result<ModuleGraph, Box<Self>> {
        if self.failed.is_none() && self.builder.discovery_complete() {
            Ok(self.builder.graph)
        } else {
            Err(Box::new(self))
        }
    }
}

impl Builder<'_> {
    pub(super) fn specialization_for_span(
        &self,
        file: FileInstanceId,
        span: jai_source::Span,
    ) -> Option<&SourceSpecializationKey> {
        self.active_specialization.as_ref().filter(|key| {
            self.graph
                .declaration(key.declaration())
                .is_some_and(|declaration| declaration.file() == file)
                && self.graph.dependency_templates.iter().any(|template| {
                    template.declaration() == key.declaration()
                        && template.location() == key.procedure()
                        && template.extent().span.start <= span.start
                        && template.extent().span.end >= span.end
                })
        })
    }
    pub(super) fn selected_condition(
        &self,
        file: FileInstanceId,
        span: jai_source::Span,
    ) -> Option<bool> {
        self.graph
            .selected_condition_for(file, span, self.specialization_for_span(file, span))
    }
    pub(super) fn record_selection(
        &mut self,
        file: FileInstanceId,
        span: jai_source::Span,
        selected: bool,
    ) {
        self.record_selection_for(
            file,
            span,
            selected,
            self.specialization_for_span(file, span).cloned().as_ref(),
        );
    }
    fn record_selection_for(
        &mut self,
        file: FileInstanceId,
        span: jai_source::Span,
        selected: bool,
        specialization: Option<&SourceSpecializationKey>,
    ) {
        if self
            .graph
            .selected_condition_for(file, span, specialization)
            .is_none()
        {
            self.graph.source_conditions.push(SourceConditionSelection {
                file,
                location: SourceSpan {
                    source: self.graph.files[file.index()].source,
                    span,
                },
                selected,
                origin: SourceConditionOrigin::Scalar,
                specialization: specialization.cloned(),
            });
        }
        if let Some(condition) = self.conditions.iter_mut().find(|condition| {
            condition.file == file
                && condition.location.span == span
                && condition.specialization.as_ref() == specialization
        }) {
            condition.selected.get_or_insert(selected);
        }
    }
    fn record_semantic_selection(
        &mut self,
        file: FileInstanceId,
        span: jai_source::Span,
        selected: bool,
        specialization: Option<&SourceSpecializationKey>,
    ) {
        self.record_selection_for(file, span, selected, specialization);
        let selection = self
            .graph
            .source_conditions
            .iter_mut()
            .find(|selection| {
                selection.file == file
                    && selection.location.span == span
                    && selection.specialization.as_ref() == specialization
            })
            .expect("recorded semantic selection");
        selection.origin = SourceConditionOrigin::Semantic;
    }
    pub(super) fn defer_condition(
        &mut self,
        file: FileInstanceId,
        expression: &Expression,
        context: DiscoveryConditionContext,
    ) -> GraphError {
        let module = self.graph.files[file.index()].module;
        match self.prepare_callable_aliases(module) {
            Ok(None) => {}
            Ok(Some(error)) | Err(error) => return error,
        }
        let location = SourceSpan {
            source: self.graph.files[file.index()].source,
            span: expression.span,
        };
        if !self.conditions.iter().any(|condition| {
            condition.file == file
                && condition.location == location
                && condition.specialization.as_ref()
                    == self.specialization_for_span(file, expression.span)
        }) {
            self.conditions.push(DeferredCondition {
                id: ConditionRequestId(self.conditions.len()),
                file,
                module: self.graph.files[file.index()].module,
                location,
                expression: expression.clone(),
                context,
                selected: None,
                specialization: self.specialization_for_span(file, expression.span).cloned(),
            });
        }
        let diagnostic = self.graph.diagnostic(
            location,
            "source condition is awaiting semantic compile-time evaluation",
        );
        let rendered = diagnostic.render(&self.graph.sources);
        GraphError::Pending {
            diagnostic,
            rendered,
        }
    }
}
