//! Source admission observes checked types without querying the type interner.
use super::*;
use jai_source::SourceSpan;

impl MetaContext {
    pub(crate) fn register_reflection_source_headers(
        &mut self,
        scope: crate::modules::FileScope<'_>,
        types: &TypeRegistry,
        location: SourceSpan,
    ) -> Result<(), Diagnostic> {
        let facts = scope.reflection_source_facts(types, &self.constants, location)?;
        let mut sources = Vec::with_capacity(facts.len());
        for (ty, location) in facts {
            let source = scope.source_record(location.source).ok_or_else(|| {
                Diagnostic::at_source(
                    location,
                    "source reflection header has no retained original source record",
                )
            })?;
            sources.push((ty, source, location.span));
        }
        self.reflection_catalog
            .register_sources(types, sources)
            .map_err(|error| Diagnostic::at_source(location, error.to_string()))
    }
}

impl Resolver<'_> {
    pub(crate) fn record_reflection_source_type(
        &mut self,
        ty: TypeId,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let Some(scope) = self.graph_scope else {
            // The scalar-only adapter does not own a source graph or expose
            // a Runtime_Info provider. Builtin rows are seeded independently.
            return Ok(());
        };
        let source_id = self
            .debug
            .source()
            .or_else(|| self.compile_time.map(|context| context.source))
            .unwrap_or_else(|| scope.source());
        let source = scope.source_record(source_id).ok_or_else(|| {
            Diagnostic::at_source(
                SourceSpan {
                    source: source_id,
                    span,
                },
                "checked reflection type has no original source record",
            )
        })?;
        self.meta
            .register_reflection_source_type(self.types, ty, source, span)
    }

    pub(crate) fn record_reflection_source_expression(
        &mut self,
        value: &Expr,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let ty = match value {
            Expr::Type(ty)
            | Expr::Pointer {
                ty, ..
            }
            | Expr::Typed {
                ty, ..
            }
            | Expr::Enum {
                ty, ..
            } => *ty,
            Expr::Code(_) => self.types.code_type(),
            Expr::Int(value) => self.types.scalar(ScalarType::Int(value.ty())),
            Expr::Bool(_) => self.types.scalar(ScalarType::Bool),
            Expr::Float(value) => self.types.float(value.ty()),
            Expr::Void(_)
            | Expr::IndirectVoid {
                ..
            } => self.types.void(),
            // Weak literals and untyped null have no canonical concrete type
            // until a checked context chooses one. They add no guessed row.
            Expr::Null | Expr::Literal(_) | Expr::WeakFloat(_) | Expr::WeakConditional(_) => {
                return Ok(());
            }
        };
        self.record_reflection_source_type(ty, span)
    }
}
