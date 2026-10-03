//! A backticked root reads the retained invocation frame without rebinding operands.
use super::*;

impl Resolver<'_> {
    fn with_caller_reference<T>(
        &mut self,
        span: Span,
        evaluate: impl FnOnce(&mut Resolver<'_>) -> Result<T, Diagnostic>,
    ) -> Result<T, Diagnostic> {
        let capture = self
            .meta
            .codes
            .exports
            .last()
            .and_then(|frame| frame.caller_scope.clone())
            .ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "caller references require an active #expand invocation",
                )
            })?;
        if capture.procedure != self.procedure {
            return Err(Diagnostic::new(
                span,
                "caller reference storage cannot escape its invoking procedure",
            ));
        }
        self.with_definition_scope(capture.file, capture.substitution.as_ref(), |resolver| {
            let mut frames = capture.frames.clone();
            let caller = frames.last_mut().ok_or_else(|| {
                Diagnostic::new(span, "caller reference has no retained lexical frame")
            })?;
            for frame in &resolver.meta.codes.exports {
                caller.extend(frame.bindings.clone());
            }
            let scopes = std::mem::replace(&mut resolver.scopes, frames);
            let mut caller_locals = capture.local_scopes.clone();
            caller_locals.resume_after_expansion(&resolver.local_scopes);
            let mut locals = std::mem::replace(&mut resolver.local_scopes, caller_locals);
            resolver.meta.codes.source_files.push(capture.source_file);
            // The leaf's source still belongs to the macro definition. Only
            // lookup changes; ordinary postfix operands are resolved afterward.
            let result = evaluate(resolver);
            resolver.meta.codes.source_files.pop();
            resolver.scopes = scopes;
            locals.resume_after_expansion(&resolver.local_scopes);
            resolver.local_scopes = locals;
            result
        })
    }

    pub(crate) fn caller_reference_expression(
        &mut self,
        name: Symbol,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        self.with_caller_reference(span, |resolver| {
            resolver.path_expression(
                &syntax::NamePath {
                    root: name,
                    members: Vec::new(),
                },
                span,
            )
        })
    }

    pub(crate) fn caller_reference_place(
        &mut self,
        name: Symbol,
        span: Span,
    ) -> Result<jai_ir::Place, Diagnostic> {
        self.with_caller_reference(span, |resolver| {
            let place = resolver.path_place(
                &syntax::NamePath {
                    root: name,
                    members: Vec::new(),
                },
                span,
            )?;
            let storage = crate::Storage::from_place(place, resolver.types)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            resolver.check_local_storage_capture(storage, span)?;
            Ok(place)
        })
    }

    pub(crate) fn describe_caller_reference(
        &mut self,
        name: Symbol,
        span: Span,
    ) -> Result<crate::overloads::ArgumentInfo, Diagnostic> {
        self.with_caller_reference(span, |resolver| {
            resolver.describe_argument(&syntax::Expression {
                kind: syntax::ExpressionKind::Name(name),
                span,
            })
        })
    }
}
