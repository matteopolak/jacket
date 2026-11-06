//! Native returned acquisitions for the conservative unclosed-local scan.
//!
//! Only complete capabilities without pending liveness frames qualify. The index
//! belongs to one lint invocation; disagreeing checked instances erase a result.
use super::{lifetime_facts::current_body as current, scalar_handles::Analysis};
use crate::syntax::Cx;
use jaic::fxhash::{HashMap, HashSet};
use jaic::ide::lifetimes::LifetimeEvent as Event;
use jaic::sema::{Compiler, ProcId};
use jaic::source::{FileId, Span};

#[derive(Default)]
pub(super) struct Openings {
    calls: HashMap<Span, Vec<usize>>,
}

impl Openings {
    /// Retain only source calls whose checked instances agree about acquisitions.
    /// For example, a handle in result one maps to the second declaration name.
    fn build(c: &Compiler, files: &HashSet<FileId>) -> Self {
        let timer = super::lifetime_stats::Timer::start();
        let callers = Analysis::caller_openings(c, files);
        let Some(facts) = c.ed.facts() else {
            return Self::default();
        };
        let mut calls = HashMap::<Span, Vec<usize>>::default();
        for (&id, body) in &facts.lifetime_bodies {
            if jaic::interrupt::should_stop() {
                return Self::default();
            }
            if !current(c, id, body) {
                continue;
            }
            for (block, b) in body.blocks.iter().enumerate() {
                for (ordinal, event) in b.events.iter().enumerate() {
                    if jaic::interrupt::should_stop() {
                        return Self::default();
                    }
                    let Event::Call {
                        span, ..
                    } = event
                    else {
                        continue;
                    };
                    if let Some(positions) = positions(&callers, id, block, ordinal) {
                        calls
                            .entry(*span)
                            .and_modify(|prior| agree(prior, &positions))
                            .or_insert(positions);
                    }
                }
            }
        }
        // Unknown instances must erase earlier candidates, including when they
        // were visited before a known instance during the first pass.
        for (&id, body) in &facts.lifetime_bodies {
            if jaic::interrupt::should_stop() {
                return Self::default();
            }
            for (block, b) in body.blocks.iter().enumerate() {
                for (ordinal, event) in b.events.iter().enumerate() {
                    if jaic::interrupt::should_stop() {
                        return Self::default();
                    }
                    let (Event::Call {
                        span, ..
                    }
                    | Event::IndirectCall {
                        span, ..
                    }) = event
                    else {
                        continue;
                    };
                    let Some(prior) = calls.get_mut(span) else {
                        continue;
                    };
                    if !current(c, id, body)
                        || positions(&callers, id, block, ordinal).as_ref() != Some(prior)
                    {
                        prior.clear();
                    }
                }
            }
        }
        if jaic::interrupt::should_stop() {
            return Self::default();
        }
        calls.retain(|_, positions| !positions.is_empty());
        timer.finish("scalar_openings", None, calls.len());
        Self {
            calls,
        }
    }
}

/// Query the run-local checked index, never source callee spelling or notes.
/// `results(cx, call.span)` may return `[1]` for `ok, handle := acquire()`.
pub(super) fn results(cx: &Cx, span: Span) -> Option<Vec<usize>> {
    cx.run
        .scalar_openings
        .get_or_init(|| Openings::build(cx.compiler, &cx.run.lifetime_files))
        .calls
        .get(&span)
        .cloned()
}

/// A possible acquisition is enough for this rule's "nothing handles it" policy;
/// an unresolved callback's liveness obligation is never silently discharged.
fn positions(
    callers: &super::scalar_handles::CallOpenings,
    procedure: ProcId,
    block: usize,
    event: usize,
) -> Option<Vec<usize>> {
    callers
        .positions(procedure, block, event)
        .map(<[usize]>::to_vec)
}

/// Disagreement means the source call has no supported common acquisition set.
fn agree(prior: &mut Vec<usize>, positions: &[usize]) {
    if prior != positions {
        prior.clear();
    }
}
