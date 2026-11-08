//! Rule queries use checked body summaries, including selected polymorphic instances.
use super::{callee_procs, lifetime_summary};
use crate::syntax::Cx;
use jaic::ast::{Expr, ExprKind};
use jaic::sema::ProcId;

#[derive(Clone, Copy)]
pub(super) enum Effect {
    OwnedReturn,
    Output,
    Exit,
    PushContext,
    Minimum,
    Maximum,
    Cleanup,
    CleanupFirst,
    ReleasedFirst,
}

impl Effect {
    fn matches(self, cx: &Cx, proc: ProcId, summary: &lifetime_summary::Summary) -> bool {
        match self {
            Self::OwnedReturn => summary.returns.get(&0).is_some_and(|(path, guard)| {
                guard.is_none()
                    && matches!(
                        path.root,
                        lifetime_summary::Root::Heap | lifetime_summary::Root::CallerAllocator
                    )
            }),
            Self::Output => summary.observable.output,
            Self::Exit => summary.observable.exit,
            Self::PushContext => summary.pushes_context,
            Self::Minimum => summary.ordering == Some(lifetime_summary::Ordering::Minimum),
            Self::Maximum => summary.ordering == Some(lifetime_summary::Ordering::Maximum),
            Self::Cleanup | Self::CleanupFirst | Self::ReleasedFirst => {
                // What the call frees below an argument on every path is released too.
                summary
                    .released_contents_all()
                    .iter()
                    .any(|(n, _)| matches!(self, Self::Cleanup) || *n == 0)
                    || summary.may.keys().any(|e| {
                        let lifetime_summary::Root::Parameter(n) = e.target.root else {
                            return false;
                        };
                        if e.kind != lifetime_summary::Kind::Free
                            || !matches!(self, Self::Cleanup) && n != 0
                        {
                            return false;
                        }
                        if e.target.steps.is_empty() {
                            return true;
                        }
                        if !lifetime_summary::descriptor_storage(&e.target.steps) {
                            return false;
                        }
                        let Some(ty) = cx.compiler.pr.procs[proc.0 as usize]
                            .sig
                            .as_ref()
                            .and_then(|s| s.params.get(n))
                            .map(|p| p.ty)
                        else {
                            return false;
                        };
                        let descriptor = |ty| {
                            matches!(
                                cx.compiler.ty.types.kind(ty),
                                jaic::types::TypeKind::String
                                    | jaic::types::TypeKind::Array {
                                        kind: jaic::types::ArrayKind::View
                                            | jaic::types::ArrayKind::Resizable,
                                        ..
                                    }
                            )
                        };
                        descriptor(ty)
                            || !matches!(self, Self::ReleasedFirst)
                                && cx.compiler.ty.types.pointee(ty).is_some_and(descriptor)
                    })
            }
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
        lifetime_summary::Summaries::build_for(
            cx.compiler,
            &cx.run.lifetime_files,
            cx.run.lifetime_entry_module,
        )
    });
    let matches = |proc: ProcId| {
        if let Some(summary) = summaries.procs.get(&proc) {
            return effect.matches(cx, proc, summary);
        }
        let mut instances = cx.compiler.pr.procs[proc.0 as usize]
            .instances
            .values()
            .filter_map(|id| summaries.procs.get(id).map(|s| (*id, s)))
            .peekable();
        instances.peek().is_some() && instances.all(|(id, summary)| effect.matches(cx, id, summary))
    };
    if let Some(chosen) = cx.compiler.ide_resolved_overload(span, &procs) {
        return matches(chosen);
    }
    !procs.is_empty() && procs.iter().all(|&proc| matches(proc))
}
