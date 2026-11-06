//! Checked formal-to-actual bindings for syntax consumers.
//!
//! A source span can appear in several checked instances or expansions. Retain a
//! binding only when every recorded instance for that callee agrees; never assume
//! source order equals runtime formal order.
use super::lifetime_facts::current_body;
use crate::syntax::Cx;
use jaic::ast::{Expr, ExprKind};
use jaic::fxhash::HashMap;
use jaic::ide::lifetimes::LifetimeEvent;
use jaic::sema::{Compiler, ProcId};
use jaic::source::Span;

#[derive(Default)]
pub(super) struct Bindings(HashMap<(Span, ProcId, usize), Option<Span>>);

impl Bindings {
    fn build(compiler: &Compiler) -> Self {
        let mut out = Self::default();
        let Some(facts) = compiler.ed.facts() else {
            return out;
        };
        for (&id, body) in &facts.lifetime_bodies {
            if !current_body(compiler, id, body) {
                continue;
            }
            for event in body.blocks.iter().flat_map(|b| &b.events) {
                let LifetimeEvent::Call {
                    callee,
                    args,
                    span,
                    expands: false,
                    ..
                } = event
                else {
                    continue;
                };
                let Some(signature) = compiler.proc(*callee).sig.as_ref() else {
                    continue;
                };
                for formal in 0..signature.params.len() {
                    let mut matching = args
                        .iter()
                        .filter(|arg| arg.formal == formal && !arg.spread);
                    let binding = matching
                        .next()
                        .map(|arg| arg.span)
                        .filter(|_| matching.next().is_none());
                    out.0
                        .entry((*span, *callee, formal))
                        .and_modify(|prior| {
                            if *prior != binding {
                                *prior = None;
                            }
                        })
                        .or_insert(binding);
                }
            }
        }
        out
    }
}

/// The written expression bound to a runtime formal, if checked instances agree.
/// Defaults, spread packs, indirect calls and ambiguous expansions remain unknown.
pub(super) fn argument<'a>(cx: &Cx, call: &'a Expr, formal: usize) -> Option<&'a Expr> {
    let ExprKind::Call {
        callee,
        args,
        ..
    } = &call.kind
    else {
        return None;
    };
    let named = super::callee_procs(cx, callee);
    let span = match &callee.kind {
        ExprKind::Member(_, name) => name.span,
        _ => callee.span,
    };
    let bindings = cx
        .run
        .call_bindings
        .get_or_init(|| Bindings::build(cx.compiler));
    let selected = if let Some(chosen) = cx.compiler.ide_resolved_overload(span, &named) {
        bindings
            .0
            .get(&(call.span, chosen, formal))
            .copied()
            .flatten()?
    } else {
        let mut selected = None;
        for &id in &*named {
            let current = bindings
                .0
                .get(&(call.span, id, formal))
                .copied()
                .flatten()?;
            if selected.is_some_and(|prior| prior != current) {
                return None;
            }
            selected = Some(current);
        }
        selected?
    };
    let mut matching = args
        .iter()
        .filter(|a| !a.spread && a.value.span == selected);
    let only = matching.next()?;
    matching.next().is_none().then_some(&only.value)
}
