//! Selected ordinary defaults use genuine retained source owners and the shared VM cache.
use super::*;
use crate::source_parameter_defaults::{SourceParameterDefault, SourceParameterDefaultKey};
use std::rc::Rc;
use std::sync::Arc;

type Requests = HashMap<SourceParameterDefaultKey, usize>;
pub(super) struct BindOptions<'a> {
    pub(super) resolve: &'a crate::ResolveOptions,
    pub(super) prefix: bool,
}

fn sources(declarations: &ScopedDeclarations<'_>) -> Vec<Rc<SourceParameterDefault>> {
    let mut sources: Vec<_> = declarations
        .signatures
        .values()
        .flat_map(|signature| signature.parameters.iter())
        .filter_map(|parameter| match &parameter.default {
            Some(ParameterDefault::Source(source)) => Some(Rc::clone(source)),
            _ => None,
        })
        .collect();
    sources.sort_by_key(|source| {
        let key = source.key;
        (key.file.index(), key.declaration.index(), key.parameter)
    });
    sources
}

pub(super) fn request_snapshot(declarations: &ScopedDeclarations<'_>) -> Requests {
    sources(declarations)
        .into_iter()
        .map(|source| (source.key, source.requests()))
        .collect()
}

/// The selected binder records typed source demand before returning its diagnostic.
pub(super) fn requested_since(
    declarations: &ScopedDeclarations<'_>,
    before: &Requests,
) -> Option<SourcePreparationPending> {
    sources(declarations)
        .into_iter()
        .find(|source| {
            source.ready().is_none()
                && source.requests() > before.get(&source.key).copied().unwrap_or(0)
        })
        .map(|source| pending(source.key))
}
fn pending(key: SourceParameterDefaultKey) -> SourcePreparationPending {
    SourcePreparationPending::new(aggregates::parameterized::PendingType::ProcedureDefault {
        declaration: key.declaration,
        parameter: key.parameter,
        location: key.location,
    })
}

struct Job {
    source: Rc<SourceParameterDefault>,
    owner: ProcedureId,
}
#[derive(Default)]
pub(super) struct Jobs {
    pending: Vec<Job>,
    admitted: std::collections::HashSet<SourceParameterDefaultKey>,
    completed: usize,
    source_wait: Option<SourcePreparationPending>,
}
impl Jobs {
    pub(super) fn admit_requested(
        &mut self,
        declarations: &ScopedDeclarations<'_>,
    ) -> Result<(), LocatedDiagnostic> {
        for source in sources(declarations) {
            if source.ready().is_some()
                || source.requests() == 0
                || self.admitted.contains(&source.key)
            {
                continue;
            }
            let owner = declarations
                .generics
                .borrow_mut()
                .reserve_local_procedure()
                .map_err(|error| LocatedDiagnostic::new(source.key.location.source, error))?;
            self.admitted.insert(source.key);
            self.pending.push(Job { source, owner });
        }
        Ok(())
    }
    pub(super) fn progress_count(&self) -> usize {
        self.admitted.len() + self.completed
    }
    pub(super) fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
    pub(super) fn source_wait(&self) -> Option<SourcePreparationPending> {
        self.source_wait
    }
    /// A prefix checkpoint follows each actual source default completion too:
    /// an effectful default can add inputs that its selected caller needs.
    pub(super) fn bind(
        &mut self,
        context: &crate::compile_time::Context<'_>,
        declarations: &ScopedDeclarations<'_>,
        types: &mut TypeRegistry,
        places: &mut PlaceRegistry,
        meta: &mut crate::reflection::MetaContext,
        options: BindOptions<'_>,
    ) -> Result<bool, LocatedDiagnostic> {
        let prefix = options.prefix;
        self.admit_requested(declarations)?;
        self.source_wait = None;
        let mut retry = Vec::new();
        let mut completed = false;
        for job in std::mem::take(&mut self.pending) {
            if completed && prefix {
                retry.push(job);
                continue;
            }
            let key = job.source.key;
            let child = context.for_source(job.owner, key.file, key.location.source);
            let requests = request_snapshot(declarations);
            let result = job.evaluate(&child, declarations, types, places, meta, options.resolve);
            let vm_waiting = !child.pending.borrow().is_empty()
                || !child.pending_constants.borrow().is_empty()
                || !child.pending_field_defaults.borrow().is_empty();
            context.merge_pending_from(&child);
            let source_wait = requested_since(declarations, &requests);
            match result {
                Ok(value) => {
                    job.source
                        .publish(value)
                        .map_err(|error| LocatedDiagnostic::new(key.location.source, error))?;
                    self.completed += 1;
                    completed = true;
                }
                Err(_) if vm_waiting => {
                    self.source_wait = Some(pending(key));
                    retry.push(job);
                }
                Err(_) if source_wait.is_some() => {
                    self.source_wait = source_wait;
                    retry.push(job);
                }
                Err(error) => return Err(LocatedDiagnostic::new(key.location.source, error)),
            }
        }
        self.pending = retry;
        self.admit_requested(declarations)?;
        Ok(completed && prefix)
    }
}
impl Job {
    fn evaluate(
        &self,
        context: &crate::compile_time::Context<'_>,
        declarations: &ScopedDeclarations<'_>,
        types: &mut TypeRegistry,
        places: &mut PlaceRegistry,
        meta: &mut crate::reflection::MetaContext,
        options: &crate::ResolveOptions,
    ) -> Result<ParameterDefault, Diagnostic> {
        let key = self.source.key;
        let declaration = declarations
            .graph
            .declaration(key.declaration)
            .ok_or_else(|| {
                Diagnostic::at_source(key.location, "source default declaration is absent")
            })?;
        let record = declarations
            .graph
            .sources()
            .get(key.location.source)
            .ok_or_else(|| {
                Diagnostic::at_source(key.location, "source default immutable source is absent")
            })?;
        if declaration.file() != key.file
            || declaration.location().source != key.location.source
            || declarations.source_procedures.get(key.declaration) != Some(key.procedure)
            || !Arc::ptr_eq(&record.shared_text(), &self.source.source_bytes)
        {
            return Err(Diagnostic::at_source(
                key.location,
                "source default authority changed",
            ));
        }
        let parameters = match &declaration.syntax().kind {
            FileDeclarationKind::Procedure(source) => &source.parameters,
            FileDeclarationKind::ProcedurePrototype(source) => &source.parameters,
            _ => {
                return Err(Diagnostic::at_source(
                    key.location,
                    "source default owner is not callable",
                ));
            }
        };
        let original =
            parameters
                .get(key.parameter)
                .and_then(|parameter| match &parameter.binding {
                    syntax::ParameterBinding::Defaulted { expression, .. }
                    | syntax::ParameterBinding::DefaultedType { expression, .. } => {
                        Some(expression)
                    }
                    syntax::ParameterBinding::Required(_)
                    | syntax::ParameterBinding::RequiredType(_) => None,
                });
        if original.map(|expression| expression.span) != Some(key.location.span) {
            return Err(Diagnostic::at_source(
                key.location,
                "source default occurrence changed",
            ));
        }
        let empty_signatures = HashMap::new();
        let empty_values = HashMap::new();
        let mut resolver = Resolver {
            expression_owner: Some(self.owner),
            debug: crate::debug_capture::Capture::default(),
            checks: crate::safety_checks::ActiveChecks::default(),
            local_scopes: crate::local_declarations::LocalScopes::default(),
            context: declarations.context.as_ref(),
            context_available: true,
            meta,
            procedure: self.owner,
            types,
            target_layout: options.effective_layout(),
            places,
            signatures: &empty_signatures,
            globals: &empty_values,
            graph_scope: Some(FileScope {
                declarations,
                file: key.file,
                substitution: None,
            }),
            compile_time: Some(context),
            symbols: declarations.graph.symbols(),
            scopes: vec![HashMap::new()],
            locals: vec![],
            span: key.location.span,
            results: &[],
            loops: vec![],
            next_loop: 0,
            active_push: None,
            next_push: 0,
            cleanups: vec![],
            deferred_scopes: vec![],
            cleanup_context: None,
        };
        resolver.parameter_default(&self.source.expression, key.expected)
    }
}
