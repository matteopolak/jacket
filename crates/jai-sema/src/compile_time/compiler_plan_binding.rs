//! Compiler-only plan binding may type native leaves, but may
//! not execute source directives while constructing the retained plan.
use super::{Cache, Context, EffectService, EffectsMode, ProcedureId};
use jai_modules::FileInstanceId;
use jai_source::{Diagnostic, SourceId, SourceSpan, Span};

impl<'a> Context<'a> {
    /// The caller supplies a fresh cache and a NoEffects service. Readiness and
    /// actual expression owner remain shared; runtime execution is denied.
    pub(crate) fn compiler_plan_for_source<'b>(
        &'b self,
        owner: ProcedureId,
        file: FileInstanceId,
        source: SourceId,
        cache: &'b Cache,
        effects: &'b dyn EffectService,
    ) -> Context<'b> {
        let mut child = self.isolated_for_source(owner, file, source, cache, effects);
        child.effect_mode = EffectsMode::CompilerPlanBinding;
        child
    }
}

impl crate::Resolver<'_> {
    /// Call before lexical keys, committed cache reads, or continuation lookup
    /// in both scalar and ordered-results execution entry points.
    pub(super) fn check_source_execution(&self, span: Span) -> Result<(), Diagnostic> {
        let Some(context) = self.compile_time else {
            return Ok(());
        };
        if context.effect_mode == EffectsMode::CompilerPlanBinding {
            return Err(Diagnostic::at_source(
                SourceSpan {
                    source: self.debug.source().unwrap_or(context.source),
                    span,
                },
                "nested #run cannot execute while binding a compiler Code plan",
            ));
        }
        Ok(())
    }
}
