//! Prove quiet candidates without a value-state fixed point when no observable kill exists.
use super::super::tasks::Op as Task;
use super::{CfgOp, Cx, DEAD, HashMap, LIVE, Model, Op, Origin, UNKNOWN, VecDeque, kills};

pub(super) fn observable(model: &Model) -> Vec<bool> {
    let mut incoming = vec![Vec::new(); model.blocks.len()];
    for (from, block) in model.blocks.iter().enumerate() {
        for &to in &block.next {
            incoming[to].push(from);
        }
    }
    let mut queue: VecDeque<_> = model
        .blocks
        .iter()
        .enumerate()
        .filter_map(|(i, b)| {
            matches!(
                b.op,
                CfgOp::Use(..)
                    | CfgOp::Task(Task::Start { .. })
                    | CfgOp::Resource(Op::Free { .. } | Op::FreeRegion { .. })
            )
            .then_some(i)
        })
        .collect();
    let mut reachable = vec![false; model.blocks.len()];
    while let Some(i) = queue.pop_front() {
        if !reachable[i] {
            reachable[i] = true;
            queue.extend(&incoming[i]);
        }
    }
    reachable
}

pub(super) fn quiet(
    cx: &Cx,
    model: &Model,
    kind: Origin,
    region: Option<usize>,
    relevant: &HashMap<usize, usize>,
    observable: &[bool],
) -> bool {
    let index = |cell| relevant.get(&cell).copied();
    let local = matches!(
        kind,
        Origin::Stack(_) | Origin::Statement(_) | Origin::Temporary
    );
    let escapes = local
        && model.blocks.iter().any(|b| match b.op {
            CfgOp::Resource(
                Op::Return {
                    cell, ..
                }
                | Op::Escape {
                    cell, ..
                },
            ) => index(cell).is_some(),
            _ => false,
        });
    let retained = model.blocks.iter().any(|b| match &b.op {
        CfgOp::Task(Task::Start {
            cells, ..
        }) => cells.iter().any(|c| index(*c).is_some()),
        _ => false,
    });
    if escapes || retained && matches!(kind, Origin::Stack(_)) {
        return false;
    }
    // This state includes every possible live alias and every pre-birth watermark. If even
    // it cannot be killed, no concrete state in the existing solver can be killed either.
    let width = relevant.len();
    let mut possible = vec![LIVE; width * 2 + 1];
    possible[width..width * 2].fill(DEAD);
    possible[width * 2] = UNKNOWN;
    !model.blocks.iter().enumerate().any(|(i, b)| {
        let CfgOp::Resource(op) = &b.op else {
            return false;
        };
        if matches!(op, Op::ScopeEnd(..) | Op::StatementEnd(..)) && !observable[i] && !retained {
            return false;
        }
        kills(cx, op, kind, region, &possible, &index)
    })
}
