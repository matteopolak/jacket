//! Bounded path specialization over the lowered lifetime operations.
use super::{Block, HashMap, LifetimeBody, Op};

/// Specialize only the analysis CFG, keeping semantic definitions and summaries unique.
/// Immutable private predicates need no alias invalidation; a bounded product retains
/// correlations between repeated tests without changing the compiler's executable IR.
pub(super) fn feasible_paths(blocks: Vec<Block>, body: &LifetimeBody) -> Vec<Block> {
    specialize(blocks, body, true)
}

fn specialize(mut blocks: Vec<Block>, body: &LifetimeBody, scalar: bool) -> Vec<Block> {
    let controls = super::controls::Controls::new(&body.controls);
    if scalar {
        for (i, insts) in body.controls.blocks.iter().enumerate() {
            if !insts.is_empty() {
                blocks[i].op = Op::Control(i);
            }
        }
    }
    let predicates: HashMap<_, _> = body
        .boolean_branches
        .iter()
        .map(|&(condition, slot, inverted)| (condition, (slot, inverted)))
        .collect();
    let mut counts = HashMap::<jaic::ir::Val, usize>::default();
    for &(slot, _) in predicates.values() {
        *counts.entry(slot).or_default() += 1;
    }
    let mut slots: Vec<_> = counts
        .into_iter()
        .filter_map(|(slot, n)| (n > 1).then_some(slot))
        .collect();
    slots.sort_unstable_by_key(|v| v.0);
    if slots.is_empty() && (!scalar || body.controls.slots.is_empty()) {
        return blocks;
    }
    let ids: HashMap<_, _> = slots.into_iter().enumerate().map(|(i, v)| (v, i)).collect();
    let initial = (0usize, vec![0u8; ids.len()], controls.initial());
    let mut seen = HashMap::from_iter([(initial.clone(), 0)]);
    let mut pending = vec![initial];
    let mut out = Vec::new();
    let limit = blocks
        .len()
        .saturating_mul(if scalar {
            128
        } else {
            4
        })
        .clamp(64, 65536);
    let mut cursor = 0;
    while cursor < pending.len() {
        let (original, state, mut numbers) = pending[cursor].clone();
        if scalar && let Op::Control(i) = blocks[original].op {
            controls.apply(i, &mut numbers);
        }
        let mut block = blocks[original].clone();
        block.next.clear();
        let mut remap = HashMap::default();
        for &next in &blocks[original].next {
            if scalar
                && let Op::Decision(_, yes, no, _, condition) = block.op
                && yes != no
            {
                if let Some(truth) = controls.condition(condition, &numbers) {
                    if (next == yes) != truth {
                        continue;
                    }
                } else if controls.numeric_condition(condition) {
                    return specialize(blocks, body, false);
                }
            }
            let mut edge = state.clone();
            if let Op::Decision(_, yes, no, _, condition) = block.op
                && yes != no
                && let Some(&(slot, inverted)) = predicates.get(&condition)
                && let Some(&slot) = ids.get(&slot)
            {
                let truth = (next == yes) ^ inverted;
                let known = if truth {
                    2
                } else {
                    1
                };
                if edge[slot] != 0 && edge[slot] != known {
                    continue;
                }
                edge[slot] = known;
            }
            let key = (next, edge, numbers.clone());
            let id = if let Some(&id) = seen.get(&key) {
                id
            } else {
                if pending.len() >= limit {
                    return if scalar {
                        specialize(blocks, body, false)
                    } else {
                        blocks
                    };
                }
                let id = pending.len();
                seen.insert(key.clone(), id);
                pending.push(key);
                id
            };
            block.next.push(id);
            remap.insert(next, id);
        }
        if let Op::Decision(_, yes, no, _, _) = &mut block.op {
            *yes = remap.get(yes).copied().unwrap_or(cursor);
            *no = remap.get(no).copied().unwrap_or(cursor);
        }
        out.push(block);
        cursor += 1;
    }
    out
}
