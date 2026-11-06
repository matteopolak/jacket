//! Optional call refinements are installed only after complete ordinary lowering.
use super::{Block, Cells, HashMap, LifetimeBody, Op, Places, Site, Span};
use crate::rules::lifetime_summary::no_growth::calls::Calls;
use jaic::sema::Compiler;

pub(super) struct Pending {
    pub site: Site,
    pub entry: usize,
    pub end: usize,
    pub ops: Vec<Op>,
}

/// Validate single-entry event intervals before changing any ordinary transfer.
fn intervals(blocks: &[Block], pending: &[Pending], prefix: usize) -> Option<Vec<bool>> {
    let mut removed = vec![false; blocks.len()];
    let mut occupied = vec![false; blocks.len()];
    for p in pending {
        if p.entry < prefix || p.entry >= p.end || p.end > blocks.len() || p.ops.is_empty() {
            return None;
        }
        if blocks[p.end - 1]
            .next
            .iter()
            .any(|&next| p.entry <= next && next < p.end)
        {
            return None;
        }
        for i in p.entry..p.end {
            if occupied[i] || matches!(blocks[i].op, Op::Stop | Op::Decision(..) | Op::Control(_)) {
                return None;
            }
            occupied[i] = true;
            if i + 1 < p.end && blocks[i].next.as_slice() != [i + 1] {
                return None;
            }
            if i != p.entry {
                removed[i] = true;
            }
        }
    }
    for (i, b) in blocks.iter().enumerate() {
        for &next in &b.next {
            if next >= blocks.len() {
                return None;
            }
            if removed[next]
                && !pending
                    .iter()
                    .any(|p| p.entry <= i && i < p.end && p.entry < next && next < p.end)
            {
                return None;
            }
        }
        if let Op::Decision(_, yes, no, ..) = b.op
            && (yes >= blocks.len() || no >= blocks.len() || removed[yes] || removed[no])
        {
            return None;
        }
    }
    Some(removed)
}

pub(super) fn install(
    c: &Compiler,
    body: &LifetimeBody,
    inventory: (&Places, &Cells),
    blocks: &mut Vec<Block>,
    calls: &mut Calls,
    pending: Vec<Pending>,
    chains: &mut HashMap<(Span, usize), Vec<Span>>,
) -> bool {
    let (places, cells) = inventory;
    if pending.is_empty() {
        return true;
    }
    let work = blocks
        .len()
        .checked_mul(pending.len() + 3)
        .and_then(|n| n.checked_add(pending.len() * pending.len()));
    if work.is_none_or(|n| calls.charge(n).is_none()) {
        return false;
    }
    let Some(mut removed) = intervals(blocks, &pending, body.blocks.len()) else {
        return false;
    };
    let sites: Vec<_> = pending
        .iter()
        .filter(|p| calls.get(p.site).is_some())
        .map(|p| p.site)
        .collect();
    let Some(affected) = super::content_writes::prepare(c, body, places, cells, calls, &sites)
    else {
        return false;
    };
    let mut affected: HashMap<_, _> = sites.into_iter().zip(affected).collect();
    let added = pending.iter().try_fold(0usize, |n, p| {
        n.checked_add(p.ops.len())
            .and_then(|n| n.checked_add(affected.get(&p.site).map_or(0, Vec::len)))
    });
    if added.is_none_or(|n| calls.charge(n.saturating_mul(2)).is_none()) {
        return false;
    }
    // From here publication is bounded and cannot fail partway through.
    for p in pending {
        let continuation = blocks[p.end - 1].next.clone();
        let mut replacement = Vec::new();
        for (i, op) in p.ops.into_iter().enumerate() {
            if i == 1
                && let Some(cells) = affected.remove(&p.site)
            {
                replacement.extend(cells.into_iter().map(Op::Define));
            } else {
                replacement.push(op);
            }
        }
        if let Some(call) = calls.get(p.site)
            && let Some(count_origin) = call.count_origin
        {
            chains.retain(|(span, _), _| *span != call.span);
            if let Some(region) = replacement.iter().find_map(|op| match op {
                Op::Mutate(region, ..) => Some(*region),
                _ => None,
            }) {
                chains.insert((call.span, region), vec![call.span, count_origin]);
            }
        }
        let mut ops = replacement.into_iter();
        blocks[p.entry].op = ops.next().unwrap_or(Op::Nop);
        blocks[p.entry].next.clear();
        let mut previous = p.entry;
        for op in ops {
            let at = blocks.len();
            blocks.push(Block {
                op,
                next: Vec::new(),
                site: blocks[p.entry].site,
            });
            removed.push(false);
            blocks[previous].next.push(at);
            previous = at;
        }
        blocks[previous].next = continuation;
    }
    // Remove only superseded event nodes; preserve original basic-block prefix
    // and remap the two operation operands that actually hold CFG node indexes.
    let mut remap = vec![0; blocks.len()];
    let mut count = 0;
    for (i, &drop) in removed.iter().enumerate() {
        if !drop {
            remap[i] = count;
            count += 1;
        }
    }
    let mut i = 0;
    blocks.retain(|_| {
        let keep = !removed[i];
        i += 1;
        keep
    });
    for b in blocks {
        for next in &mut b.next {
            *next = remap[*next];
        }
        if let Op::Decision(_, yes, no, ..) = &mut b.op {
            *yes = remap[*yes];
            *no = remap[*no];
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(next: &[usize]) -> Block {
        Block {
            op: Op::Nop,
            next: next.to_vec(),
            site: None,
        }
    }

    fn pending(entry: usize, end: usize) -> Pending {
        Pending {
            site: Site {
                block: 0,
                event: entry,
            },
            entry,
            end,
            ops: vec![Op::Nop],
        }
    }

    #[test]
    fn event_intervals_keep_entries_and_reject_control_or_external_entries() {
        let blocks = vec![node(&[1]), node(&[2]), node(&[3]), node(&[])];
        assert_eq!(
            intervals(&blocks, &[pending(1, 3)], 1),
            Some(vec![false, false, true, false])
        );
        assert_eq!(
            intervals(&blocks, &[pending(1, 2), pending(2, 3)], 1),
            Some(vec![false; 4])
        );
        let mut external = blocks.clone();
        external[0].next.push(2);
        assert!(intervals(&external, &[pending(1, 3)], 1).is_none());
        let mut stop = blocks.clone();
        stop[2].op = Op::Stop;
        assert!(intervals(&stop, &[pending(1, 3)], 1).is_none());
        assert!(intervals(&blocks, &[pending(1, 3), pending(2, 3)], 1).is_none());
    }

    #[test]
    fn event_interval_cannot_reenter_its_removed_interior() {
        let blocks = vec![node(&[1]), node(&[2]), node(&[2])];
        assert!(intervals(&blocks, &[pending(1, 3)], 1).is_none());
        let blocks = vec![node(&[1]), node(&[2]), node(&[1])];
        assert!(intervals(&blocks, &[pending(1, 3)], 1).is_none());
    }
}
