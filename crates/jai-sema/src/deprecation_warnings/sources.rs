//! Resolve original declaration and reference spans without consulting debug emission policy.
use super::*;
use crate::{Diagnostic, Resolver, Span, reflection::MetaContext, syntax};
use jai_ir::ProcedureId;
use jai_source::{SourceRecord, SourceSpan, Symbol};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum DeprecationKey {
    Procedure(ProcedureId),
    Macro(crate::metaprogram::MacroId),
}

pub(crate) fn procedure_extent(procedure: &syntax::Procedure) -> Span {
    let end = procedure
        .body
        .iter()
        .map(|statement| statement.span.end)
        .chain(
            procedure
                .deprecation
                .iter()
                .map(|attribute| attribute.span.end),
        )
        .max()
        .unwrap_or(procedure.span.end);
    Span::new(procedure.span.start, end)
}

impl MetaContext {
    pub(crate) fn remember_deprecation(
        &mut self,
        identity: DeprecationKey,
        source: &SourceRecord,
        name: &str,
        deprecation: Option<&syntax::Deprecation>,
        extent: Span,
    ) -> Result<(), Diagnostic> {
        let Some(deprecation) = deprecation else {
            return Ok(());
        };
        let location = WarningLocation::new(
            source,
            SourceSpan {
                source: source.id(),
                span: deprecation.span,
            },
        )
        .map_err(|_| {
            Diagnostic::new(
                deprecation.span,
                "deprecation declaration source span is invalid",
            )
        })?;
        self.deprecations.register(
            identity,
            DeprecatedDeclaration {
                name: name.into(),
                location,
                extent: WarningLocation::new(
                    source,
                    SourceSpan {
                        source: source.id(),
                        span: extent,
                    },
                )
                .map_err(|_| {
                    Diagnostic::new(extent, "deprecated procedure source extent is invalid")
                })?,
                advice: deprecation.message.clone(),
            },
        );
        Ok(())
    }
}

impl Resolver<'_> {
    pub(crate) fn remember_local_deprecation(
        &mut self,
        identity: DeprecationKey,
        name: Symbol,
        deprecation: Option<&syntax::Deprecation>,
        extent: Span,
        source: Option<jai_source::SourceId>,
    ) -> Result<(), Diagnostic> {
        let Some(deprecation) = deprecation else {
            return Ok(());
        };
        let scope = self.graph_scope.ok_or_else(|| {
            Diagnostic::new(
                deprecation.span,
                "deprecation diagnostics require retained source records",
            )
        })?;
        let source = source
            .or(self.debug.source())
            .unwrap_or_else(|| scope.source());
        let record = scope.source_record(source).ok_or_else(|| {
            Diagnostic::new(
                deprecation.span,
                "deprecation declaration source origin is not retained",
            )
        })?;
        self.meta.remember_deprecation(
            identity,
            record,
            self.symbols.name(name),
            Some(deprecation),
            extent,
        )
    }

    pub(crate) fn warn_deprecated_procedure(
        &mut self,
        identity: ProcedureId,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.warn_deprecated(DeprecationKey::Procedure(identity), span)
    }

    pub(crate) fn warn_deprecated(
        &mut self,
        identity: DeprecationKey,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if !self.meta.deprecations.contains(&identity) {
            return Ok(());
        }
        if self
            .meta
            .deprecations
            .contains(&DeprecationKey::Procedure(self.procedure))
        {
            return Ok(());
        }
        let scope = self.graph_scope.ok_or_else(|| {
            Diagnostic::new(
                span,
                "deprecation diagnostics require retained source records",
            )
        })?;
        let source = self.debug.source().unwrap_or_else(|| scope.source());
        let record = scope.source_record(source).ok_or_else(|| {
            Diagnostic::new(span, "deprecation reference source origin is not retained")
        })?;
        let location = WarningLocation::new(record, SourceSpan { source, span })
            .map_err(|_| Diagnostic::new(span, "deprecation reference source span is invalid"))?;
        self.meta
            .deprecations
            .reference(&identity, location)
            .map_err(|message| Diagnostic::new(span, message))
    }
}
