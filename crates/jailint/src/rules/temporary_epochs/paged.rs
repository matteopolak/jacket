//! Conditional no-traversal components. No node or lease identity is inferred here.
use super::checked::{Budget, Summary, body_specialized};
use super::domain::*;
use super::*;
use jaic::fxhash::HashSet;
use jaic::ide::lifetimes::{LifetimeBody, LifetimeEvent as Event};

#[derive(Clone, Debug)]
pub(super) struct Inputs {
    pub receiver_formal: usize,
    pub receiver_ty: TypeId,
    pub receiver_pointer_ty: TypeId,
    pub head_slot: ByteSlot,
    pub stop_formal: usize,
}

/// Fold only checked integer widths; callers enable this solely in specialization.
pub(super) fn constant_compare(p: &Predicate) -> Option<bool> {
    let Predicate::Compare {
        left: Operand::Constant {
            bits: a, ..
        },
        right: Operand::Constant {
            bits: b, ..
        },
        op,
        ty,
    } = p
    else {
        return None;
    };
    let bits = match ty {
        Ty::I8 => 8,
        Ty::I16 => 16,
        Ty::I32 => 32,
        Ty::I64 | Ty::Ptr => 64,
        _ => return None,
    };
    let mask = u64::MAX >> (64 - bits);
    let (a, b) = (a & mask, b & mask);
    let signed = |x: u64| ((x << (64 - bits)) as i64) >> (64 - bits);
    Some(match op {
        CmpOp::Eq => a == b,
        CmpOp::Ne => a != b,
        CmpOp::SLt => signed(a) < signed(b),
        CmpOp::SLe => signed(a) <= signed(b),
        CmpOp::SGt => signed(a) > signed(b),
        CmpOp::SGe => signed(a) >= signed(b),
        CmpOp::ULt => a < b,
        CmpOp::ULe => a <= b,
        CmpOp::UGt => a > b,
        CmpOp::UGe => a >= b,
        _ => return None,
    })
}

fn reachable(graph: &[Vec<usize>], start: usize, budget: &mut Budget) -> Option<HashSet<usize>> {
    let mut found = HashSet::default();
    let mut work = vec![start];
    while let Some(v) = work.pop() {
        budget.spend(1)?;
        if found.insert(v) {
            for next in graph.get(v)? {
                budget.spend(1)?;
                work.push(*next);
            }
        }
    }
    Some(found)
}

/// Recover a genuine SCC and a header edge into it, independent of explored exits.
fn component(b: &LifetimeBody, header: usize, budget: &mut Budget) -> Option<Vec<usize>> {
    budget.spend(b.blocks.len().checked_mul(2)?)?;
    let mut graph = Vec::with_capacity(b.blocks.len());
    for block in &b.blocks {
        budget.spend(block.successors.len())?;
        graph.push(block.successors.clone());
    }
    let mut reverse = vec![Vec::new(); graph.len()];
    for (v, edges) in graph.iter().enumerate() {
        for next in edges {
            budget.spend(1)?;
            reverse.get_mut(*next)?.push(v);
        }
    }
    let forward = reachable(&graph, header, budget)?;
    let backward = reachable(&reverse, header, budget)?;
    budget.spend(forward.len())?;
    let mut nodes: Vec<_> = forward.intersection(&backward).copied().collect();
    nodes.sort_unstable();
    if nodes.len() < 2 && !graph.get(header)?.contains(&header) {
        return Some(Vec::new());
    }
    Some(nodes)
}

fn input(
    c: &Compiler,
    b: &LifetimeBody,
    p: &Predicate,
    formals: &mut Option<HashMap<usize, TypeId>>,
    budget: &mut Budget,
) -> Option<Option<(Inputs, TypeId)>> {
    let Predicate::Compare {
        left,
        right,
        op: CmpOp::Eq | CmpOp::Ne,
        ty: Ty::Ptr | Ty::I64,
    } = p
    else {
        return Some(None);
    };
    for (head, stop) in [(left, right), (right, left)] {
        let (
            Operand::Captured(Captured::ReceiverRead {
                formal,
                receiver_ty,
                slot,
                ..
            }),
            Operand::Captured(Captured::FormalValue {
                formal: stop_formal,
                slot: stop_slot,
            }),
        ) = (head, stop)
        else {
            continue;
        };
        if formal == stop_formal
            || slot.width != 8
            || stop_slot.width != 8
            || stop_slot.offset != 0
            || slot.ty != stop_slot.ty
            || c.ty.types.pointee(slot.ty).is_none()
        {
            continue;
        }
        if formals.is_none() {
            let mut lookup = HashMap::default();
            for block in &b.blocks {
                budget.spend(block.events.len())?;
                for event in &block.events {
                    if let Event::Parameter {
                        formal,
                        ty,
                        ..
                    } = event
                        && (lookup.insert(*formal, *ty).is_some() || lookup.len() > ITEMS)
                    {
                        return None;
                    }
                }
            }
            *formals = Some(lookup);
        }
        let lookup = formals.as_ref()?;
        let (Some(receiver), Some(stop_ty)) = (
            lookup.get(formal).copied(),
            lookup.get(stop_formal).copied(),
        ) else {
            continue;
        };
        if c.ty.types.size_of(receiver) != 8
            || c.ty.types.pointee(receiver) != Some(*receiver_ty)
            || stop_ty != slot.ty
            || slot.offset.checked_add(8)? > c.ty.types.size_of(*receiver_ty)
        {
            continue;
        }
        return Some(Some((
            Inputs {
                receiver_formal: *formal,
                receiver_ty: *receiver_ty,
                receiver_pointer_ty: receiver,
                head_slot: *slot,
                stop_formal: *stop_formal,
            },
            stop_ty,
        )));
    }
    Some(None)
}

/// Publish only when every feasible specialized path is supported and bypasses the SCC.
pub(super) fn discover(
    c: &Compiler,
    b: &LifetimeBody,
    summary: &Summary,
    budget: &mut Budget,
) -> Option<Vec<LoopBypass>> {
    let mut found = Vec::new();
    let mut seen = HashSet::default();
    let mut formals = None;
    for (guard, predicate) in &summary.guards {
        budget.spend(1)?;
        let Some((inputs, stop_ty)) = input(c, b, predicate, &mut formals, budget)? else {
            continue;
        };
        let at = guard.chain.last()?;
        if !seen.insert((at.block, at.event)) {
            continue;
        }
        let nodes = component(b, at.block, budget)?;
        let Some(Event::Branch {
            then_block,
            else_block,
            ..
        }) = b.blocks.get(at.block).and_then(|b| b.events.get(at.event))
        else {
            continue;
        };
        let edges: Vec<_> = [*then_block, *else_block]
            .into_iter()
            .filter(|target| nodes.contains(target))
            .map(|to| ControlEdge {
                from: at.block,
                to,
            })
            .collect();
        if edges.len() != 1 || nodes.contains(then_block) == nodes.contains(else_block) {
            continue;
        }
        let specialized = body_specialized(c, b, Some(inputs.clone()), Some(budget))?;
        if specialized.partial
            || specialized.rejected
            || specialized.exits.is_empty()
            || !specialized.reached.contains(&at.block)
            || edges
                .iter()
                .any(|edge| specialized.edges.contains(&(edge.from, edge.to)))
            || nodes
                .iter()
                .any(|node| *node != at.block && specialized.reached.contains(node))
        {
            continue;
        }
        let mut paths = Vec::new();
        let mut valid = true;
        for s in &specialized.exits {
            budget.spend(1)?;
            if s.incomplete
                || s.interfered
                || !s.output_writes.is_empty()
                || !s.visited.contains(&at.block)
                || s.conditions
                    .iter()
                    .any(|c| !predicate_supported(&c.predicate))
            {
                valid = false;
                break;
            }
            let mut writes = Vec::new();
            for w in &s.writes {
                // Charge the borrowed tree before allocating its public representation.
                budget.spend(w.value.size()?.0.checked_add(4)?)?;
                let value = w.value.operand();
                if !matches!(&w.instance.pointer,Captured::FormalValue{formal,..} if *formal==inputs.receiver_formal)
                    || w.instance.view_ty != inputs.receiver_ty
                    || !operand_supported(&value)
                {
                    valid = false;
                    break;
                }
                writes.push(HeaderWrite {
                    site: w.site.clone(),
                    slot: w.slot,
                    value,
                });
            }
            for condition in &s.conditions {
                budget.spend(predicate_size(&condition.predicate)?.0.checked_add(1)?)?;
            }
            paths.push(HeaderWriteCase {
                conditions: s.conditions.clone(),
                writes,
            });
        }
        if !valid {
            continue;
        }
        if found.len() >= ITEMS {
            return None;
        }
        let bytes = c.ty.types.size_of(inputs.receiver_ty);
        found.push(LoopBypass {
            procedure: b.procedure,
            receiver_formal: inputs.receiver_formal,
            receiver_ty: inputs.receiver_ty,
            head_slot: inputs.head_slot,
            stop_formal: inputs.stop_formal,
            guard: guard.clone(),
            component: nodes,
            excluded_body_edges: edges,
            hypotheses: vec![
                LoopHypothesis::ReceiverAddressSnapshot {
                    formal: inputs.receiver_formal,
                    ty: inputs.receiver_pointer_ty,
                },
                LoopHypothesis::ReadableWritableReceiver {
                    formal: inputs.receiver_formal,
                    ty: inputs.receiver_ty,
                    bytes,
                },
                LoopHypothesis::EntryPointerZero {
                    formal: inputs.receiver_formal,
                    slot: inputs.head_slot,
                },
                LoopHypothesis::NullArgumentSnapshot {
                    formal: inputs.stop_formal,
                    ty: stop_ty,
                },
            ],
            paths,
        });
    }
    Some(found)
}
