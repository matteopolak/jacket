//! Reborrow expansion state so inferred definition substitutions stay immutable.
use super::*;
use crate::{Block, polymorphism::Substitution};

impl Resolver<'_> {
    pub(crate) fn expand_bound_target_body(
        &mut self,
        target: &ExpandedTarget,
        bindings: Vec<(Symbol, Binding)>,
        initializers: Vec<Statement>,
        span: Span,
    ) -> Result<Block, Diagnostic> {
        self.expand_bound_substituted_target_body(target, bindings, initializers, None, span)
    }

    pub(crate) fn expand_bound_substituted_target_body(
        &mut self,
        target: &ExpandedTarget,
        bindings: Vec<(Symbol, Binding)>,
        initializers: Vec<Statement>,
        substitution: Option<&Substitution>,
        span: Span,
    ) -> Result<Block, Diagnostic> {
        self.prepare_caller_cleanup_export(span)?;
        let substitution = substitution.or_else(|| {
            target
                .capture
                .as_ref()
                .and_then(|capture| capture.substitution.as_ref())
        });
        let caller_depth = self.debug.caller_origins().len();
        if let Some(source) = self
            .debug
            .source()
            .or_else(|| self.graph_scope.map(|scope| scope.source()))
        {
            self.debug.push_caller_origin(jai_source::SourceSpan {
                source,
                span,
            });
        }
        let result = self.with_definition_scope(target.file, substitution, |resolver| {
            resolver.expand_bound_definition_body(target, bindings, initializers, span)
        });
        self.debug.restore_caller_origins(caller_depth);
        result
    }

    pub(super) fn with_definition_scope<T>(
        &mut self,
        file: FileInstanceId,
        substitution: Option<&Substitution>,
        evaluate: impl FnOnce(&mut Resolver<'_>) -> Result<T, Diagnostic>,
    ) -> Result<T, Diagnostic> {
        // This shortened borrow owns the same runtime procedure state. It lets
        // FileScope borrow a stack-owned inference result without retaining it
        // in the caller resolver or borrowing the caller's specialization.
        let mut scope = self.graph_scope.map(|scope| scope.code_file(file));
        if let Some(scope) = &mut scope {
            scope.substitution = substitution;
        }
        let mut child = Resolver {
            debug: std::mem::take(&mut self.debug),
            checks: self.checks,
            context: self.context,
            context_available: self.context_available,
            meta: &mut *self.meta,
            graph_scope: scope,
            compile_time: self.compile_time,
            target_layout: self.target_layout,
            procedure: self.procedure,
            expression_owner: self.expression_owner,
            types: &mut *self.types,
            places: &mut *self.places,
            signatures: self.signatures,
            symbols: self.symbols,
            scopes: std::mem::take(&mut self.scopes),
            local_scopes: std::mem::take(&mut self.local_scopes),
            globals: self.globals,
            locals: std::mem::take(&mut self.locals),
            span: self.span,
            results: self.results,
            loops: std::mem::take(&mut self.loops),
            next_loop: self.next_loop,
            cleanups: std::mem::take(&mut self.cleanups),
            active_push: self.active_push,
            next_push: self.next_push,
            deferred_scopes: std::mem::take(&mut self.deferred_scopes),
            cleanup_context: self.cleanup_context,
        };
        let result = evaluate(&mut child);
        self.debug = child.debug;
        self.scopes = child.scopes;
        self.local_scopes = child.local_scopes;
        self.locals = child.locals;
        self.loops = child.loops;
        self.next_loop = child.next_loop;
        self.cleanups = child.cleanups;
        self.active_push = child.active_push;
        self.next_push = child.next_push;
        self.deferred_scopes = child.deferred_scopes;
        self.cleanup_context = child.cleanup_context;
        result
    }
}
