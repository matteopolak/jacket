//! Retain original alias requests and constants across genuine type readiness.
//! The cursor owns original declaration IDs and the evaluator for one graph.
use super::*;
use aggregates::parameterized::{PendingType, TypePreparation};
mod constant_annotations;

/// Only the preparation controller can create this source dependency.
/// VM dependencies remain separate in LibraryPending.dependencies.
#[derive(Clone, Copy, Debug)]
pub struct SourcePreparationPending {
    cause: Cause,
}

#[derive(Clone, Copy, Debug)]
enum Cause {
    Type(PendingType),
    Lookup(super::source_lookup_demands::Demand),
}

impl SourcePreparationPending {
    pub(crate) fn new(cause: PendingType) -> Self {
        Self {
            cause: Cause::Type(cause),
        }
    }

    pub fn location(self) -> SourceSpan {
        match self.cause {
            Cause::Type(cause) => cause.location(),
            Cause::Lookup(demand) => demand.location,
        }
    }

    pub fn placeholder(self) -> Option<jai_modules::PlaceholderId> {
        match self.cause {
            Cause::Type(PendingType::Placeholder(demand)) => Some(demand.placeholder),
            _ => None,
        }
    }

    pub fn procedure_default(self) -> Option<(DeclarationId, usize)> {
        match self.cause {
            Cause::Type(PendingType::ProcedureDefault {
                declaration,
                parameter,
                ..
            }) => Some((declaration, parameter)),
            _ => None,
        }
    }

    pub fn diagnostic(self, graph: &ModuleGraph) -> LocatedDiagnostic {
        match self.cause {
            Cause::Type(cause) => cause.diagnostic(graph),
            Cause::Lookup(demand) => demand.diagnostic(graph),
        }
    }

    /// A lookup wait owns an existing consumer; it claims no future declaration.
    pub fn lookup_consumer(self) -> Option<DeclarationId> {
        match self.cause {
            Cause::Lookup(demand) => Some(demand.consumer),
            _ => None,
        }
    }

    pub fn lookup_root(self) -> Option<Symbol> {
        match self.cause {
            Cause::Lookup(demand) => Some(demand.root),
            _ => None,
        }
    }

    pub(crate) fn lookup(demand: super::source_lookup_demands::Demand) -> Self {
        Self {
            cause: Cause::Lookup(demand),
        }
    }

    pub(crate) fn cause(self) -> Option<PendingType> {
        match self.cause {
            Cause::Type(cause) => Some(cause),
            Cause::Lookup(_) => None,
        }
    }
}

pub(super) enum AliasProgress {
    Complete,
    Pending(SourcePreparationPending),
}

/// The owning phase supplies its retained nominal/type registries. The cursor
/// keeps the original graph identities and evaluator, and advances only after
/// the alias helper has published a real Ready representation.
pub(super) struct PendingAliases<'graph> {
    graph: &'graph ModuleGraph,
    constants: Constants<'graph>,
    sources: Vec<DeclarationId>,
    next: usize,
    pending: Option<SourcePreparationPending>,
    context_headers: HashMap<DeclarationId, procedure_headers::ContextHeaderPrerequisite>,
}

impl<'graph> PendingAliases<'graph> {
    pub(super) fn new(graph: &'graph ModuleGraph, constants: Constants<'graph>) -> Self {
        let sources = graph
            .declarations()
            .iter()
            .filter_map(|source| {
                matches!(
                    source.syntax().kind,
                    FileDeclarationKind::TypeAlias(_) | FileDeclarationKind::Constant(_)
                )
                .then_some(source.id())
            })
            .collect();
        Self {
            graph,
            constants,
            sources,
            next: 0,
            pending: None,
            context_headers: HashMap::new(),
        }
    }

    pub(super) fn drive(
        &mut self,
        declarations: &mut ScopedDeclarations<'graph>,
        types: &mut TypeRegistry,
        meta: &mut crate::reflection::MetaContext,
    ) -> Result<AliasProgress, LocatedDiagnostic> {
        self.check_graph(declarations)?;
        while let Some(&source) = self.sources.get(self.next) {
            if let Some(cause) =
                self.prepare_constant_annotation(source, declarations, types, meta)?
            {
                let pending = SourcePreparationPending::new(cause);
                self.pending = Some(pending);
                return Ok(AliasProgress::Pending(pending));
            }
            let outcome = declarations.nominals.prepare_alias(
                self.graph,
                source,
                types,
                &mut meta.record_specializations,
                &mut |file, expression| self.constants.prepare_evaluate_lazy(file, expression),
            )?;
            match outcome {
                None | Some(TypePreparation::Ready(_)) => {
                    self.pending = None;
                    self.next += 1;
                }
                Some(TypePreparation::Pending(mut cause)) => {
                    if let PendingType::Constant {
                        declaration, ..
                    } = cause
                        && let Some(annotation_wait) = self.prepare_constant_annotation(
                            declaration,
                            declarations,
                            types,
                            meta,
                        )?
                    {
                        cause = annotation_wait;
                    }
                    let pending = SourcePreparationPending::new(cause);
                    self.pending = Some(pending);
                    return Ok(AliasProgress::Pending(pending));
                }
            }
        }
        Ok(AliasProgress::Complete)
    }

    pub(super) fn prepare_global_annotations(
        &mut self,
        declarations: &mut ScopedDeclarations<'graph>,
        types: &mut TypeRegistry,
        meta: &mut crate::reflection::MetaContext,
    ) -> Result<AliasProgress, LocatedDiagnostic> {
        for source in self.graph.declarations() {
            let FileDeclarationKind::Global(global) = &source.syntax().kind else {
                continue;
            };
            let annotation = match global.declaration.source() {
                syntax::Declaration::UnresolvedExplicit {
                    ty, ..
                }
                | syntax::Declaration::External {
                    ty, ..
                } => ty,
                _ => continue,
            };
            match aggregates::parameterized::prepare_type_paired(
                self.graph,
                aggregates::parameterized::TypeRequest::new(source.file(), annotation, global.span),
                types,
                &declarations.nominals,
                &mut meta.record_specializations,
                &mut |file, expression| self.constants.prepare_evaluate_lazy(file, expression),
            )? {
                TypePreparation::Ready(ty) => {
                    declarations.nominals.value_types.insert(source.id(), ty);
                }
                TypePreparation::Pending(cause) => {
                    let pending = SourcePreparationPending::new(cause);
                    self.pending = Some(pending);
                    return Ok(AliasProgress::Pending(pending));
                }
            }
        }
        Ok(AliasProgress::Complete)
    }

    fn check_graph(
        &self,
        declarations: &ScopedDeclarations<'graph>,
    ) -> Result<(), LocatedDiagnostic> {
        if std::ptr::eq(self.graph, declarations.graph) {
            Ok(())
        } else {
            Err(self.lifecycle_error("alias continuation belongs to another source graph"))
        }
    }

    fn lifecycle_error(&self, message: &str) -> LocatedDiagnostic {
        let location = self
            .pending
            .map(SourcePreparationPending::location)
            .or_else(|| {
                self.sources
                    .get(self.next)
                    .and_then(|&source| self.graph.declaration(source))
                    .map(|source| source.location())
            })
            .unwrap_or_else(|| SourceSpan {
                source: self
                    .graph
                    .file(self.graph.module(self.graph.root()).unwrap().entry())
                    .unwrap()
                    .source(),
                span: Span::default(),
            });
        LocatedDiagnostic {
            location,
            message: message.into(),
        }
    }

    pub(super) fn constants_mut(&mut self) -> &mut Constants<'graph> {
        &mut self.constants
    }

    /// Annotation requests retain their defining declaration; no representation
    /// or primitive fact is published until the paired evaluator is ready.
    pub(super) fn prepare_annotations(
        &mut self,
        declarations: &mut ScopedDeclarations<'graph>,
        types: &mut TypeRegistry,
        meta: &mut crate::reflection::MetaContext,
    ) -> Result<AliasProgress, LocatedDiagnostic> {
        for source in self.graph.declarations() {
            let FileDeclarationKind::Constant(constant) = &source.syntax().kind else {
                continue;
            };
            let Some(annotation) = &constant.ty else {
                continue;
            };
            if self.constants.annotation(source.id()).is_some() {
                continue;
            }
            match aggregates::parameterized::prepare_type_paired(
                self.graph,
                aggregates::parameterized::TypeRequest::new(
                    source.file(),
                    annotation,
                    constant.span,
                ),
                types,
                &declarations.nominals,
                &mut meta.record_specializations,
                &mut |file, expression| self.constants.prepare_evaluate_lazy(file, expression),
            )? {
                TypePreparation::Ready(ty) => {
                    self.constants
                        .register_annotation(source.id(), ty, types)
                        .map_err(|error| {
                            located(
                                self.graph,
                                source.file(),
                                Diagnostic::new(constant.span, error.to_string()),
                            )
                        })?;
                    declarations.nominals.value_types.insert(source.id(), ty);
                }
                TypePreparation::Pending(cause) => {
                    let pending = SourcePreparationPending::new(cause);
                    self.pending = Some(pending);
                    return Ok(AliasProgress::Pending(pending));
                }
            }
        }
        Ok(AliasProgress::Complete)
    }

    pub(super) fn prepare_independent_headers(
        &mut self,
        declarations: &mut ScopedDeclarations<'graph>,
        types: &mut TypeRegistry,
        meta: &mut crate::reflection::MetaContext,
    ) -> Result<Option<SourcePreparationPending>, LocatedDiagnostic> {
        let mut pending = None;
        for source in
            procedure_headers::ordered_sources(self.graph, &declarations.callable_aliases)?
        {
            if declarations.source_procedures.get(source.id()).is_none()
                || declarations.signatures.contains_key(&source.id())
            {
                continue;
            }
            if procedure_headers::source_dependencies(
                self.graph,
                &declarations.callable_aliases,
                source,
            )
            .iter()
            .any(|dependency| !declarations.signatures.contains_key(dependency))
            {
                continue;
            }
            let (parameters, results) = match &source.syntax().kind {
                FileDeclarationKind::Procedure(procedure) => {
                    (&procedure.parameters, &procedure.results)
                }
                FileDeclarationKind::ProcedurePrototype(prototype) => {
                    (&prototype.parameters, &prototype.results)
                }
                _ => continue,
            };
            let mut ready = true;
            let annotations = parameters
                .iter()
                .filter_map(|parameter| match &parameter.binding {
                    syntax::ParameterBinding::RequiredType(ty)
                    | syntax::ParameterBinding::DefaultedType {
                        ty: Some(ty), ..
                    } => Some((ty, parameter.span)),
                    _ => None,
                })
                .chain(results.iter().filter_map(|result| match &result.binding {
                    syntax::ResultBinding::Typed {
                        ty, ..
                    } => Some((ty, result.span)),
                    _ => None,
                }));
            for (annotation, span) in annotations {
                match aggregates::parameterized::prepare_type_paired(
                    self.graph,
                    aggregates::parameterized::TypeRequest::new(source.file(), annotation, span),
                    types,
                    &declarations.nominals,
                    &mut meta.record_specializations,
                    &mut |file, expression| self.constants.prepare_evaluate_lazy(file, expression),
                )? {
                    TypePreparation::Ready(_) => {}
                    TypePreparation::Pending(cause) => {
                        ready = false;
                        pending.get_or_insert(SourcePreparationPending::new(cause));
                        break;
                    }
                }
            }
            if ready {
                match procedure_headers::register_selected(
                    self.graph,
                    source,
                    types,
                    declarations,
                    &mut self.constants,
                    meta,
                )? {
                    procedure_headers::HeaderPreparation::Ready => {
                        self.context_headers.remove(&source.id());
                    }
                    procedure_headers::HeaderPreparation::Pending(request) => {
                        self.context_headers.insert(source.id(), request);
                    }
                }
            }
        }
        Ok(pending)
    }

    pub(super) fn prepare_scalars(
        &mut self,
        declarations: &mut ScopedDeclarations<'graph>,
        deferred: &std::collections::HashSet<DeclarationId>,
        consumed: &std::collections::HashSet<DeclarationId>,
        nominal: &std::collections::HashSet<DeclarationId>,
    ) -> Result<AliasProgress, LocatedDiagnostic> {
        for source in self.graph.declarations() {
            if !matches!(source.syntax().kind, FileDeclarationKind::Constant(_))
                || declarations.values.contains_key(&source.id())
                || declarations.nominals.is_type_alias(self.graph, source.id())
                || consumed.contains(&source.id())
                || declarations.callable_aliases.contains_key(&source.id())
                || deferred.contains(&source.id())
                || nominal.contains(&source.id())
                || self.constants.has_typed_annotation(source.id())
                || sequence_constants::is_sequence_constant(self.graph, source)
            {
                continue;
            }
            match self
                .constants
                .prepare_value(source.id(), source.location())?
            {
                constants::ScalarPreparation::Ready(value) => {
                    declarations
                        .values
                        .insert(source.id(), Binding::Constant(value));
                }
                constants::ScalarPreparation::Pending(cause) => {
                    let pending = SourcePreparationPending::new(cause);
                    self.pending = Some(pending);
                    return Ok(AliasProgress::Pending(pending));
                }
            }
        }
        Ok(AliasProgress::Complete)
    }

    /// Finishing an incomplete cursor returns its original retained state.
    /// The controller must not discard unfinished requests or their evaluator.
    pub(super) fn into_constants(
        self,
    ) -> Result<
        (
            Constants<'graph>,
            Vec<procedure_headers::ContextHeaderPrerequisite>,
        ),
        Self,
    > {
        if self.next == self.sources.len() {
            Ok((self.constants, self.context_headers.into_values().collect()))
        } else {
            Err(self)
        }
    }
}
