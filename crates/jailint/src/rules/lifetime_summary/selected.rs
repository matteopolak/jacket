//! Summaries are needed for reported bodies and their selected dependency closure.
//! Procedure values are edges too: a factory can return a callback it never calls.
use super::{Event, HashMap, HashSet, LifetimeBody, ProcId};
use jaic::source::FileId;
use std::collections::VecDeque;

pub(super) fn retain(
    compiler: &jaic::sema::Compiler,
    valid: &mut HashMap<ProcId, &LifetimeBody>,
    files: &HashSet<FileId>,
) -> Option<HashSet<ProcId>> {
    if files.is_empty() {
        return None;
    }
    let globals = crate::rules::lifetime_targets::globals::Globals::build(compiler, valid);
    let mut queue: VecDeque<_> = valid
        .iter()
        .filter(|(_, b)| files.contains(&b.syntax.header.span.file))
        .map(|(&p, _)| p)
        .collect();
    let mut needed = HashSet::default();
    while let Some(proc) = queue.pop_front() {
        if !needed.insert(proc) {
            continue;
        }
        for global in globals.used_by.get(&proc).into_iter().flatten() {
            queue.extend(globals.users.get(global).into_iter().flatten().copied());
            queue.extend(
                globals
                    .initial
                    .get(global)
                    .into_iter()
                    .flat_map(|s| s.procedures.iter().copied()),
            );
        }
        let Some(body) = valid.get(&proc) else {
            continue;
        };
        queue.extend(
            body.blocks
                .iter()
                .flat_map(|b| &b.events)
                .filter_map(|event| match event {
                    Event::Call {
                        callee, ..
                    }
                    | Event::Procedure {
                        callee, ..
                    } => Some(*callee),
                    _ => None,
                }),
        );
    }
    valid.retain(|p, _| needed.contains(p));
    Some(needed)
}
