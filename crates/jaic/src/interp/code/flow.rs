//! Fewer jumps on the hot path of a procedure's ops.
//!
//! Lowering leaves a jump wherever a block does not fall into its successor: the back edge of
//! every loop, the end of each arm of an `if`. Each one is a dispatch that does no work. This pass
//! runs on the ops of each block before locations are taken out and
//!
//! - sends a jump (or branch) that lands on a block holding only a jump straight to where that
//!   one goes, and
//! - ends a block that jumps (or falls) into a few cheap ops with a copy of them instead. When
//!   the jump goes back to a loop's test, the test is copied inverted, so one iteration ends in
//!   a single branch back to the body.
//!
//! Only ops that cannot fail are copied, so the location a failing op reports (found by its
//! position, see `Code::loc_at`) is never one of a copy. Every cycle still passes through the
//! start of some block, which is what `Interp::watches_blocks` looks at.
use super::{CodeBlock, Op};

/// Most ops (locations aside) of a block that is copied into another.
const COPY_LIMIT: usize = 4;

/// Most ops a block may grow to by copying.
const GROW_LIMIT: usize = 16;

/// Most blocks followed from one block.
const CHAIN: usize = 4;

pub(super) fn straighten(ops: &mut Vec<Op>, blocks: &mut [CodeBlock]) {
    let mut list: Vec<Vec<Op>> = blocks
        .iter()
        .map(|b| ops[b.start as usize..b.end as usize].to_vec())
        .collect();
    thread_jumps(&mut list);
    for _ in 0..2 {
        let source = list.clone();
        for (p, block) in list.iter_mut().enumerate() {
            if let Some(longer) = extend(&source, p, block) {
                *block = longer;
            }
        }
    }
    drop_jumps_to_next(&mut list);
    let mut flat = Vec::with_capacity(ops.len());
    for (block, body) in blocks.iter_mut().zip(list) {
        block.start = flat.len() as u32;
        flat.extend(body);
        block.end = flat.len() as u32;
    }
    *ops = flat;
}

/// Take out a jump to the block that follows anyway (any blocks between are empty).
fn drop_jumps_to_next(list: &mut [Vec<Op>]) {
    for p in 0..list.len() {
        let Some(last) = list[p].iter().rposition(|op| !is_loc(op)) else {
            continue;
        };
        let Op::Jump {
            target,
        } = list[p][last]
        else {
            continue;
        };
        let target = target as usize;
        if target > p && list[p + 1..target].iter().all(|b| b.is_empty()) {
            list[p].remove(last);
        }
    }
}

fn is_loc(op: &Op) -> bool {
    matches!(op, Op::Loc { .. })
}

/// Ops that never continue into the next block.
fn leaves(op: &Op) -> bool {
    matches!(
        op,
        Op::Jump { .. } | Op::Ret0 | Op::Ret1 { .. } | Op::RetN { .. } | Op::Term { .. }
    )
}

/// Ops that cannot fail and do not depend on where they are in the code.
pub(super) fn copyable(op: &Op) -> bool {
    matches!(
        op,
        Op::Const { .. }
            | Op::Mov { .. }
            | Op::Ext { .. }
            | Op::FrameAddr { .. }
            | Op::AddScaled { .. }
            | Op::Add { .. }
            | Op::AddImm { .. }
            | Op::Sub { .. }
            | Op::Mul { .. }
            | Op::MulImm { .. }
            | Op::Bin { .. }
            | Op::BinImm { .. }
            | Op::Cmp { .. }
            | Op::CmpImm { .. }
            | Op::Jump { .. }
            | Op::Branch { .. }
            | Op::BranchCmp { .. }
            | Op::BranchCmpImm { .. }
            | Op::BranchTest { .. }
            | Op::Ret0
            | Op::Ret1 { .. }
    )
}

/// The branch taken in the cases `op` falls through in, going to `target`.
fn inverted(op: &Op, target: u32) -> Option<Op> {
    Some(match *op {
        Op::Branch {
            cond,
            sense,
            ..
        } => Op::Branch {
            cond,
            sense: !sense,
            target,
        },
        Op::BranchCmp {
            op,
            shift,
            sense,
            a,
            b,
            ..
        } => Op::BranchCmp {
            op,
            shift,
            sense: !sense,
            a,
            b,
            target,
        },
        Op::BranchCmpImm {
            op,
            shift,
            sense,
            a,
            imm,
            ..
        } => Op::BranchCmpImm {
            op,
            shift,
            sense: !sense,
            a,
            imm,
            target,
        },
        Op::BranchTest {
            sense,
            a,
            mask,
            ..
        } => Op::BranchTest {
            sense: !sense,
            a,
            mask,
            target,
        },
        _ => return None,
    })
}

fn target_of(op: &mut Op) -> Option<&mut u32> {
    match op {
        Op::Jump {
            target,
        }
        | Op::Branch {
            target, ..
        }
        | Op::BranchCmp {
            target, ..
        }
        | Op::BranchCmpImm {
            target, ..
        }
        | Op::BranchTest {
            target, ..
        } => Some(target),
        _ => None,
    }
}

/// Send every jump to where a jump to its block ends up when blocks that do nothing but jump
/// on (or nothing at all) are skipped.
fn thread_jumps(list: &mut [Vec<Op>]) {
    let n = list.len();
    let step: Vec<usize> = list
        .iter()
        .enumerate()
        .map(|(i, body)| {
            let mut real = body.iter().filter(|op| !is_loc(op));
            match (real.next(), real.next()) {
                (None, _) if i + 1 < n => i + 1,
                (
                    Some(Op::Jump {
                        target,
                    }),
                    None,
                ) => *target as usize,
                _ => i,
            }
        })
        .collect();
    let end = |mut at: usize| {
        for _ in 0..8 {
            if step[at] == at {
                break;
            }
            at = step[at];
        }
        at
    };
    for body in list.iter_mut() {
        for op in body.iter_mut() {
            if let Some(target) = target_of(op) {
                *target = end(*target as usize) as u32;
            }
        }
    }
}

/// `block` (block `p`), ended with copies of the blocks it jumps or falls into, if that takes a
/// jump off its path.
fn extend(source: &[Vec<Op>], p: usize, block: &[Op]) -> Option<Vec<Op>> {
    let mut cand = block.to_vec();
    // Jumps taken off the path, less jumps put on it.
    let mut net = 0i32;
    let mut first = true;
    for _ in 0..CHAIN {
        let last = cand.iter().rposition(|op| !is_loc(op))?;
        let jumps = matches!(cand[last], Op::Jump { .. });
        let target = match cand[last] {
            Op::Jump {
                target,
            } => target as usize,
            ref op if leaves(op) => break,
            _ if first => p + 1,
            _ => break,
        };
        first = false;
        let Some(from) = source.get(target) else {
            break;
        };
        let body: Vec<Op> = from.iter().filter(|op| !is_loc(op)).copied().collect();
        let size = cand.iter().filter(|op| !is_loc(op)).count();
        if target == p
            || body.is_empty()
            || body.len() > COPY_LIMIT
            || size + body.len() > GROW_LIMIT
            || !body.iter().all(copyable)
        {
            break;
        }
        let tail = body[body.len() - 1];
        if leaves(&tail) {
            if jumps {
                cand.truncate(last);
                net += 1;
            }
            cand.extend(body);
            continue;
        }
        if target + 1 >= source.len() {
            break;
        }
        if jumps && target <= p {
            // A loop's test, reached from below: the common case is to go on into the body.
            let mut exit = tail;
            if let Some(exit_to) = target_of(&mut exit).map(|t| *t)
                && let Some(back) = inverted(&tail, target as u32 + 1)
            {
                cand.truncate(last);
                net += 1;
                cand.extend(&body[..body.len() - 1]);
                cand.push(back);
                cand.push(Op::Jump {
                    target: exit_to,
                });
                break;
            }
        }
        if jumps {
            cand.truncate(last);
            net += 1;
        }
        cand.extend(body);
        cand.push(Op::Jump {
            target: target as u32 + 1,
        });
        net -= 1;
    }
    (net > 0).then_some(cand)
}
