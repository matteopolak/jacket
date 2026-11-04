//! Known local array bounds refine a growth contract without assuming every add reallocates.
use super::{Block, Cx, Event, HashMap, HashSet, LifetimeBody, Op, Span, VecDeque};

#[derive(Clone, Copy, Debug)]
pub(super) enum Change {
    Append,
    UnknownAppend,
    Reserve(u64),
}

pub(super) fn refine(cx: &Cx, body: &LifetimeBody, blocks: &mut [Block]) {
    let calls: HashMap<Span, Change> = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .filter_map(|event| {
            let Event::Call {
                callee,
                args,
                span,
                ..
            } = event
            else {
                return None;
            };
            let proc = &cx.compiler.pr.procs[callee.0 as usize];
            if !cx
                .compiler
                .src
                .sources
                .get(proc.span.file)
                .path
                .to_string()
                .replace('\\', "/")
                .ends_with("/internal/basic/arrays.jai")
            {
                return None;
            }
            let change = if cx.compiler.proc_has_note(*callee, "AddsToArray") {
                Change::Append
            } else if proc.name.as_str() == "array_add" {
                Change::UnknownAppend
            } else if proc.name.as_str() == "array_reserve" {
                Change::Reserve(args.iter().find(|a| a.formal == 1)?.constant?)
            } else {
                return None;
            };
            Some((*span, change))
        })
        .collect();
    for block in blocks.iter_mut() {
        if let Op::Grow(region, span) = block.op
            && let Some(&change) = calls.get(&span)
        {
            block.op = Op::Capacity(region, change, span);
        }
    }
    let regions: HashSet<_> = blocks
        .iter()
        .filter_map(|b| match b.op {
            Op::Capacity(r, ..) => Some(r),
            _ => None,
        })
        .collect();
    for region in regions {
        // Equal incoming facts survive joins. Loops or rebinding widen to unknown.
        let mut states = vec![None; blocks.len()];
        states[0] = Some(None);
        let mut queue = VecDeque::from([0]);
        while let Some(i) = queue.pop_front() {
            let mut state = states[i].unwrap();
            match blocks[i].op {
                Op::EmptyArray(r) if r == region => state = Some((0u64, 0u64)),
                Op::Replace(r) | Op::Grow(r, _) | Op::MayGrow(r, _) | Op::MayReach(r, _)
                    if r == region =>
                {
                    state = None;
                }
                Op::Mutate(r, _, span) | Op::MayMutate(r, _, span)
                    if r == region && !calls.contains_key(&span) =>
                {
                    state = None;
                }
                Op::Barrier => state = None,
                Op::Capacity(r, change, _) if r == region => {
                    state = state.and_then(|(count, capacity)| match change {
                        Change::Append => Some((
                            count.checked_add(1)?,
                            if count >= capacity {
                                capacity.checked_mul(2)?.max(8)
                            } else {
                                capacity
                            },
                        )),
                        Change::Reserve(n) => Some((count, capacity.max(n))),
                        Change::UnknownAppend => None,
                    });
                }
                _ => {}
            }
            for &next in &blocks[i].next {
                let joined = match states[next] {
                    None => state,
                    Some(old) if old == state => old,
                    _ => None,
                };
                if states[next] != Some(joined) {
                    states[next] = Some(joined);
                    queue.push_back(next);
                }
            }
        }
        for (i, block) in blocks.iter_mut().enumerate() {
            if let Op::Capacity(r, change, span) = block.op
                && r == region
            {
                let room = matches!(change, Change::UnknownAppend)
                    || states[i]
                        .flatten()
                        .is_some_and(|(count, capacity)| match change {
                            Change::Append => count < capacity,
                            Change::Reserve(n) => n <= capacity,
                            Change::UnknownAppend => true,
                        });
                block.op = if room {
                    Op::MayGrow(r, span)
                } else {
                    Op::Grow(r, span)
                };
            }
        }
    }
}
