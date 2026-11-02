//! On-demand reverse product graph: value-copy provenance x whether an invalidation was
//! crossed. BFS picks a shortest witness; a trimmed DAG counts alternatives, or a cycle
//! establishes infinitely many walks. None of this memory is retained in the fixed point.
use super::resources::Op as Resource;
use super::{Cx, Finding, HashMap, Model, Op, Span, VecDeque};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct Node {
    block: usize,
    cell: usize,
    bad: bool,
}

pub(super) fn strict_path(
    model: &Model,
    reachable: &[Option<Vec<u8>>],
    relevant: &HashMap<usize, usize>,
    candidate: usize,
    at: usize,
    cell: usize,
    invalidators: &[Span],
) -> (Vec<(Span, bool)>, String) {
    let mut incoming = vec![Vec::new(); model.blocks.len()];
    for (from, block) in model.blocks.iter().enumerate() {
        if reachable[from].is_some() {
            for &to in &block.next {
                incoming[to].push(from);
            }
        }
    }
    let start = Node {
        block: at,
        cell,
        bad: invalidators.is_empty(),
    };
    let mut ids = HashMap::from_iter([(start, 0)]);
    let mut nodes = vec![start];
    let mut edges = vec![Vec::new()];
    let mut previous = vec![None];
    let mut terminal = vec![false];
    let mut queue = VecDeque::from([0]);
    let mut shortest = None;
    while let Some(id) = queue.pop_front() {
        let node = nodes[id];
        if terminal[id] {
            continue;
        }
        for &p in &incoming[node.block] {
            let mut before = Node {
                block: p,
                ..node
            };
            let mut origin = false;
            match &model.blocks[p].op {
                Op::Copy(pairs) => {
                    if let Some((_, source)) = pairs
                        .iter()
                        .find(|(target, _)| relevant.get(target) == Some(&before.cell))
                    {
                        let Some(&source) = relevant.get(source) else {
                            continue;
                        };
                        before.cell = source;
                    }
                }
                Op::LogicalBorrow(target, ..)
                    if matches!(model.blocks[candidate].op, Op::LogicalBorrow(..))
                        && relevant.get(target) == Some(&before.cell) =>
                {
                    if p != candidate {
                        continue;
                    }
                    origin = true;
                }
                Op::Borrow(target, ..)
                | Op::ElementBorrow(target, ..)
                | Op::Resource(Resource::Seed {
                    cell: target, ..
                }) if relevant.get(target) == Some(&before.cell) => {
                    if p != candidate {
                        continue;
                    }
                    origin = true;
                }
                Op::PointerLoad(target, source, ..)
                    if relevant.get(target) == Some(&before.cell) =>
                {
                    if p == candidate {
                        origin = true;
                    } else if let Some(&source) = relevant.get(source) {
                        before.cell = source;
                    } else {
                        continue;
                    }
                }
                Op::Replace(region)
                    if !node.bad
                        && (borrowed_region(model, candidate)
                            .is_some_and(|r| model.places.contains(*region, r))
                            || model.places.unresolved(*region)) =>
                {
                    continue;
                }
                Op::Barrier => continue,
                _ => {}
            }
            let invalidator = match model.blocks[p].op {
                Op::Grow(_, s)
                | Op::MayGrow(_, s)
                | Op::MayReach(_, s)
                | Op::Mutate(_, _, s)
                | Op::MayMutate(_, _, s) => Some(s),
                Op::Resource(
                    Resource::Free {
                        span, ..
                    }
                    | Resource::FreeRegion {
                        span, ..
                    }
                    | Resource::MayFree {
                        span, ..
                    }
                    | Resource::MayRegion {
                        span, ..
                    },
                ) => Some(span),
                Op::Resource(
                    Resource::Reset(s)
                    | Resource::MayReset(s)
                    | Resource::Restore(_, s)
                    | Resource::MayRestore(_, s)
                    | Resource::Advance(_, s)
                    | Resource::MayAdvance(_, s)
                    | Resource::ScopeEnd(_, s)
                    | Resource::StatementEnd(s),
                ) => Some(s),
                _ => None,
            };
            before.bad |= invalidator.is_some_and(|s| invalidators.contains(&s));
            if origin && !before.bad {
                continue;
            }
            let next = if let Some(&n) = ids.get(&before) {
                n
            } else {
                let n = nodes.len();
                ids.insert(before, n);
                nodes.push(before);
                edges.push(Vec::new());
                previous.push(Some(id));
                terminal.push(origin);
                queue.push_back(n);
                n
            };
            edges[id].push(next);
            if origin && shortest.is_none() {
                shortest = Some(next);
            }
        }
    }
    let Some(mut id) = shortest else {
        return (Vec::new(), "no additional reconstructed bad path".into());
    };
    let mut path = vec![id];
    while let Some(p) = previous[id] {
        path.push(p);
        id = p;
    }
    let decisions = path
        .windows(2)
        .filter_map(|pair| match model.blocks[nodes[pair[0]].block].op {
            Op::Decision(span, yes, no, _) => {
                let next = nodes[pair[1]].block;
                if next == yes {
                    Some((span, true))
                } else if next == no {
                    Some((span, false))
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect();
    (decisions, alternatives(&edges, &terminal))
}

fn borrowed_region(model: &Model, candidate: usize) -> Option<usize> {
    match model.blocks[candidate].op {
        Op::Borrow(_, r, _)
        | Op::ElementBorrow(_, r, ..)
        | Op::LogicalBorrow(_, r, ..)
        | Op::PointerLoad(_, _, r, _) => Some(r),
        _ => None,
    }
}

pub(super) fn alternatives(edges: &[Vec<usize>], terminal: &[bool]) -> String {
    let mut incoming = vec![Vec::new(); edges.len()];
    for (a, next) in edges.iter().enumerate() {
        for &b in next {
            incoming[b].push(a);
        }
    }
    let mut useful = vec![false; edges.len()];
    let mut queue: VecDeque<_> = terminal
        .iter()
        .enumerate()
        .filter_map(|(i, t)| t.then_some(i))
        .collect();
    while let Some(i) = queue.pop_front() {
        if !useful[i] {
            useful[i] = true;
            queue.extend(&incoming[i]);
        }
    }
    let mut indegree = vec![0; edges.len()];
    for (a, next) in edges.iter().enumerate() {
        if useful[a] {
            for &b in next {
                if useful[b] {
                    indegree[b] += 1;
                }
            }
        }
    }
    let mut queue: VecDeque<_> = (0..edges.len())
        .filter(|&i| useful[i] && indegree[i] == 0)
        .collect();
    let mut ways = vec![0u64; edges.len()];
    ways[0] = 1;
    let mut visited = 0;
    while let Some(i) = queue.pop_front() {
        visited += 1;
        for &next in &edges[i] {
            if useful[next] {
                ways[next] = ways[next].saturating_add(ways[i]);
                indegree[next] -= 1;
                if indegree[next] == 0 {
                    queue.push_back(next);
                }
            }
        }
    }
    if visited != useful.iter().filter(|&&x| x).count() {
        return "infinitely many additional bad walks through a loop".into();
    }
    let total = ways
        .iter()
        .zip(terminal)
        .filter(|(_, t)| **t)
        .fold(0u64, |a, (&b, _)| a.saturating_add(b));
    format!(
        "{} other bad path{}",
        total.saturating_sub(1),
        if total == 2 {
            ""
        } else {
            "s"
        }
    )
}

pub(super) fn targets(cx: &Cx, model: &Model, invalidators: &[Span], report: &mut Finding) {
    for block in &model.blocks {
        let Op::Indirect(set, span) = &block.op else {
            continue;
        };
        if !invalidators.contains(span) {
            continue;
        }
        let ids = set.ordered();
        let names = ids
            .iter()
            .take(7)
            .map(|p| cx.compiler.pr.procs[p.0 as usize].name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let site = cx.compiler.src.sources.snippet(*span);
        let callee = site.split('(').next().unwrap_or("the procedure value");
        if cx.lifetime_mode == jacket_config::LifetimeMode::Must {
            report.message.push_str(&format!(
                "; all {} targets of `{callee}` invalidate this storage: {names}",
                ids.len()
            ));
        } else if set.unknown {
            report.message.push_str("; this procedure value includes Unknown (strict mode assumes argument-reachable invalidation)");
        } else {
            report.message.push_str(&format!("; possible targets of `{callee}`: {names}; the labelled chain identifies an invalidating target"));
        }
        if ids.len() > 7 {
            report
                .message
                .push_str(&format!(" ({} more targets)", ids.len() - 7));
        }
        for id in ids.into_iter().take(7) {
            report.secondary.push((
                cx.compiler.pr.procs[id.0 as usize].span,
                "possible procedure target".into(),
            ));
        }
    }
}
