//! Rule queries use checked body summaries, including selected polymorphic instances.
use super::{callee_procs, lifetime_summary};
use crate::syntax::Cx;
use jaic::ast::{Expr, ExprKind};
use jaic::sema::ProcId;

#[derive(Clone, Copy)]
pub(super) enum Effect {
    Output,
    Exit,
    PushContext,
    Minimum,
    Maximum,
}

impl Effect {
    fn matches(self, summary: &lifetime_summary::Summary) -> bool {
        match self {
            Self::Output => summary.observable.output,
            Self::Exit => summary.observable.exit,
            Self::PushContext => summary.pushes_context,
            Self::Minimum => summary.ordering == Some(lifetime_summary::Ordering::Minimum),
            Self::Maximum => summary.ordering == Some(lifetime_summary::Ordering::Maximum),
        }
    }
}

pub(super) fn calls(cx: &Cx, callee: &Expr, effect: Effect) -> bool {
    let procs = callee_procs(cx, callee);
    let span = match &callee.kind {
        ExprKind::Member(_, name) => name.span,
        _ => callee.span,
    };
    let summaries = cx.run.lifetimes.get_or_init(|| {
        lifetime_summary::Summaries::build_for(cx.compiler, &cx.run.lifetime_files)
    });
    let matches = |proc: ProcId| {
        if let Some(summary) = summaries.procs.get(&proc) {
            return effect.matches(summary);
        }
        let mut instances = cx.compiler.pr.procs[proc.0 as usize]
            .instances
            .values()
            .filter_map(|id| summaries.procs.get(id))
            .peekable();
        instances.peek().is_some() && instances.all(|summary| effect.matches(summary))
    };
    if let Some(chosen) = cx.compiler.ide_resolved_overload(span, &procs) {
        return matches(chosen);
    }
    !procs.is_empty() && procs.iter().all(|&proc| matches(proc))
}
