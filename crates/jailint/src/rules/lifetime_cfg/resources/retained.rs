//! Retained worker borrows are independent of the caller's remaining uses.
use super::super::tasks::{Op as Task, Token};
use super::diagnostics::Proof;
use super::{CfgOp, Cx, HashMap, LIVE, Model, Op, Origin, Reports, UNKNOWN, kills};
use crate::rules::finding;

pub(super) fn tokens(model: &Model, start: usize) -> HashMap<Token, usize> {
    let mut out = HashMap::default();
    for block in &model.blocks {
        if let CfgOp::Task(
            Task::Start {
                token, ..
            }
            | Task::End(token),
        ) = block.op
        {
            let next = start + out.len();
            out.entry(token).or_insert(next);
        }
    }
    out
}

pub(super) fn transfer(
    cx: &Cx,
    op: &Task,
    tokens: &HashMap<Token, usize>,
    index: &impl Fn(usize) -> Option<usize>,
    state: &mut [u8],
) {
    match op {
        Task::End(token) => state[tokens[token]] = UNKNOWN,
        Task::Start {
            token,
            cells,
            possible,
            ..
        } => {
            let mut active = UNKNOWN;
            for &cell in cells {
                if let Some(cell) = index(cell) {
                    let value = state[cell];
                    if value == LIVE && !possible {
                        active = LIVE;
                        break;
                    }
                    if value & LIVE != 0 && cx.lifetime_mode == jacket_config::LifetimeMode::Strict
                    {
                        active |= LIVE;
                    }
                }
            }
            state[tokens[token]] = active;
        }
    }
}

pub(super) fn emit(proof: Proof<'_>, tokens: &HashMap<Token, usize>, out: &mut Reports) {
    let strict = proof.cx.lifetime_mode == jacket_config::LifetimeMode::Strict;
    let width = proof.relevant.len();
    let index = |c| proof.relevant.get(&c).copied();
    for (i, block) in proof.model.blocks.iter().enumerate() {
        let Some(state) = &proof.states[i] else {
            continue;
        };
        // The worker first reads already-released memory when it is started.
        if let CfgOp::Task(Task::Start {
            cells,
            possible,
            span,
            ..
        }) = &block.op
        {
            if (!possible || strict)
                && cells
                    .iter()
                    .any(|c| index(*c).is_some_and(|c| super::super::dead(proof.cx, state[c])))
            {
                let mut f = finding(
                    *span,
                    "this starts a borrower with storage whose lifetime has already ended".into(),
                    Some("start and finish the borrower before releasing its storage".into()),
                );
                f.secondary
                    .push((proof.origin, "borrowed storage originated here".into()));
                out.use_after_free.push(f);
            }
            continue;
        }
        let CfgOp::Resource(op) = &block.op else {
            continue;
        };
        let implicit_exit = matches!(op, Op::Exit) && matches!(proof.kind, Origin::Stack(_));
        if !implicit_exit
            && !kills(
                proof.cx,
                op,
                proof.kind,
                proof.region,
                &state[..=width * 2],
                &index,
            )
        {
            continue;
        }
        let at = match op {
            Op::Free {
                span, ..
            }
            | Op::FreeRegion {
                span, ..
            }
            | Op::MayFree {
                span, ..
            }
            | Op::MayRegion {
                span, ..
            }
            | Op::Reset(span)
            | Op::MayReset(span)
            | Op::Restore(_, span)
            | Op::MayRestore(_, span)
            | Op::ScopeEnd(_, span)
            | Op::StatementEnd(span)
            | Op::Advance(_, span)
            | Op::MayAdvance(_, span) => *span,
            Op::Exit => proof.body.syntax.header.span,
            _ => continue,
        };
        for (&token, &slot) in tokens {
            if if strict {
                state[slot] & LIVE == 0
            } else {
                state[slot] != LIVE
            } {
                continue;
            }
            let start = proof
                .model
                .blocks
                .iter()
                .enumerate()
                .filter_map(|(j, b)| {
                    if let CfgOp::Task(Task::Start {
                        token: t,
                        span,
                        ..
                    }) = b.op
                        && t == token
                        && proof.states[j].is_some()
                    {
                        Some((j, span))
                    } else {
                        None
                    }
                })
                .filter_map(|(j, span)| {
                    super::retained_witness::path(proof.model, j, i, token).map(|path| (span, path))
                })
                .min_by_key(|(_, (_, _, distance))| *distance);
            let mut f = finding(
                block.site.filter(|s| s.file == proof.cx.file).unwrap_or(at),
                format!(
                    "{}storage ends while a started borrower may still {}",
                    if strict && state[slot] != LIVE {
                        "strict mode: "
                    } else {
                        ""
                    },
                    if token.header {
                        "retain its receiver"
                    } else {
                        "access it"
                    }
                ),
                Some(
                    if token.header {
                        "keep the receiver alive until borrower deinitialization"
                    } else {
                        "wait for successful borrower completion before releasing captured storage"
                    }
                    .into(),
                ),
            );
            f.secondary
                .push((proof.origin, "borrowed storage originated here".into()));
            if let Some((start, (decisions, count, _))) = start {
                f.secondary.push((
                    start,
                    "borrower started here; no completion on this path".into(),
                ));
                if strict {
                    f.message.push_str(&format!(
                        "; shortest retained-borrow path shown ({count} from this start)"
                    ));
                    for (span, taken) in decisions {
                        f.secondary
                            .push((span, format!("bad path takes this condition as {taken}")));
                    }
                }
            }
            match proof.kind {
                Origin::Stack(_) | Origin::Statement(_) => out.returned_local_address.push(f),
                Origin::Temporary => out.temp_storage_escape.push(f),
                _ => out.use_after_free.push(f),
            }
            break;
        }
    }
}


pub(super) fn new_generation(state: &mut [u8], width: usize) {
    for value in &mut state[..width] {
        *value = (*value & super::DEAD) | UNKNOWN;
    }
    state[width..width * 2].fill(super::DEAD);
    state[width * 2 + 1..].fill(UNKNOWN);
}
