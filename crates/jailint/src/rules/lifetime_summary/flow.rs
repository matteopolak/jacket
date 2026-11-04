//! Control-flow fixed points and witness cuts for summaries.
use super::{Effect, Event, HashMap, HashSet, LifetimeBody, MAX_CHAIN, ProcId, Span, Summary};

/// The blocks that are reachable from the entry and from which a normal return is reachable.
pub(super) struct Live(Vec<bool>);

impl Live {
    pub(super) fn contains(&self, block: usize) -> bool {
        self.0[block]
    }

    pub(super) fn is_empty(&self) -> bool {
        !self.0.contains(&true)
    }
}

pub(super) fn returning_blocks(
    body: &LifetimeBody,
    all: &HashMap<ProcId, Summary>,
    targets: &crate::rules::lifetime_targets::Universe,
) -> Live {
    let stopped: Vec<_> = body
        .blocks
        .iter()
        .map(|b| {
            b.events.iter().any(|e| match e {
                Event::Call {
                    callee,
                    expands: false,
                    ..
                } => all
                    .get(callee)
                    .is_some_and(|s| s.normal_return == Some(false)),
                Event::IndirectCall {
                    callee,
                    allocator: false,
                    ..
                } => {
                    let set = targets.get(body.procedure, *callee);
                    !set.unknown
                        && !set.procedures.is_empty()
                        && set
                            .procedures
                            .iter()
                            .all(|p| all.get(p).is_some_and(|s| s.normal_return == Some(false)))
                }
                _ => false,
            })
        })
        .collect();
    let mut forward = vec![false; body.blocks.len()];
    let mut stack = vec![0];
    while let Some(i) = stack.pop() {
        if !std::mem::replace(&mut forward[i], true) && !stopped[i] {
            stack.extend(&body.blocks[i].successors);
        }
    }
    let mut incoming = vec![Vec::new(); body.blocks.len()];
    for (i, b) in body.blocks.iter().enumerate() {
        if !stopped[i] {
            for &next in &b.successors {
                incoming[next].push(i);
            }
        }
    }
    let mut back = vec![false; body.blocks.len()];
    let mut stack: Vec<_> = body
        .blocks
        .iter()
        .enumerate()
        .filter(|(i, b)| b.normal_return && !stopped[*i])
        .map(|(i, _)| i)
        .collect();
    while let Some(i) = stack.pop() {
        if !std::mem::replace(&mut back[i], true) {
            stack.extend(&incoming[i]);
        }
    }
    for (reached, back) in forward.iter_mut().zip(back) {
        *reached &= back;
    }
    Live(forward)
}

pub(super) fn guaranteed(
    body: &LifetimeBody,
    effects: &[Vec<(Effect, Vec<Span>)>],
    target: &Effect,
) -> Option<Vec<Span>> {
    guaranteed_except(body, effects, target, &HashSet::default())
}

pub(super) fn guaranteed_except(
    body: &LifetimeBody,
    effects: &[Vec<(Effect, Vec<Span>)>],
    target: &Effect,
    skipped: &HashSet<(usize, usize)>,
) -> Option<Vec<Span>> {
    let mut states = vec![0u8; body.blocks.len()];
    states[0] = 1;
    let mut queue = std::collections::VecDeque::from([0]);
    while let Some(i) = queue.pop_front() {
        let state = if effects[i].iter().any(|(e, _)| e == target) {
            2
        } else {
            states[i]
        };
        for &next in &body.blocks[i].successors {
            if skipped.contains(&(i, next)) {
                continue;
            }
            let joined = states[next] | state;
            if joined != states[next] {
                states[next] = joined;
                queue.push_back(next);
            }
        }
    }
    let mut returns = 0;
    for (i, b) in body.blocks.iter().enumerate() {
        if b.normal_return && states[i] != 0 {
            returns += 1;
            if states[i] & 1 != 0 && !effects[i].iter().any(|(e, _)| e == target) {
                return None;
            }
        }
    }
    if returns == 0 {
        return None;
    }
    let mut incoming = vec![Vec::new(); body.blocks.len()];
    for (i, block) in body
        .blocks
        .iter()
        .enumerate()
        .filter(|(i, _)| states[*i] != 0)
    {
        for &next in &block.successors {
            if !skipped.contains(&(i, next)) {
                incoming[next].push(i);
            }
        }
    }
    let mut queue: std::collections::VecDeque<_> = body
        .blocks
        .iter()
        .enumerate()
        .filter(|(i, b)| b.normal_return && states[*i] != 0)
        .map(|(i, _)| i)
        .collect();
    let mut seen = HashSet::default();
    let mut witnesses = Vec::new();
    while let Some(i) = queue.pop_front() {
        if !seen.insert(i) {
            continue;
        }
        if let Some((_, witness)) = effects[i].iter().rev().find(|(e, _)| e == target) {
            witnesses.push(witness.clone());
        } else {
            queue.extend(&incoming[i]);
        }
    }
    witnesses.sort_by_key(|w| w.first().map(|s| (s.file.0, s.start, s.end)));
    let mut out = Vec::new();
    for site in witnesses.into_iter().flatten() {
        if !out.contains(&site) {
            out.push(site);
        }
    }
    if out.len() > MAX_CHAIN {
        let tail = out.split_off(out.len() - 3);
        out.truncate(3);
        out.push(Span::NONE);
        out.extend(tail);
    }
    Some(out)
}

/// A component whose summaries depend only on already solved callees: one pass settles it.
pub(super) fn acyclic(component: &[ProcId], graph: &HashMap<ProcId, Vec<ProcId>>) -> bool {
    matches!(component, [only] if !graph[only].contains(only))
}

pub(super) fn sccs(ids: &[ProcId], graph: &HashMap<ProcId, Vec<ProcId>>) -> Vec<Vec<ProcId>> {
    struct Visit<'a> {
        graph: &'a HashMap<ProcId, Vec<ProcId>>,
        clock: usize,
        indices: HashMap<ProcId, usize>,
        low: HashMap<ProcId, usize>,
        stack: Vec<ProcId>,
        active: HashSet<ProcId>,
        out: Vec<Vec<ProcId>>,
    }
    fn visit(v: ProcId, state: &mut Visit<'_>) {
        let index = state.clock;
        state.clock += 1;
        state.indices.insert(v, index);
        state.low.insert(v, index);
        state.stack.push(v);
        state.active.insert(v);
        for &w in &state.graph[&v] {
            if !state.indices.contains_key(&w) {
                visit(w, state);
                state.low.insert(v, state.low[&v].min(state.low[&w]));
            } else if state.active.contains(&w) {
                state.low.insert(v, state.low[&v].min(state.indices[&w]));
            }
        }
        if state.low[&v] == state.indices[&v] {
            let mut component = Vec::new();
            loop {
                let w = state.stack.pop().unwrap();
                state.active.remove(&w);
                component.push(w);
                if w == v {
                    break;
                }
            }
            state.out.push(component);
        }
    }
    let mut state = Visit {
        graph,
        clock: 0,
        indices: HashMap::default(),
        low: HashMap::default(),
        stack: Vec::new(),
        active: HashSet::default(),
        out: Vec::new(),
    };
    for &id in ids {
        if !state.indices.contains_key(&id) {
            visit(id, &mut state);
        }
    }
    state.out
}

/// Blocks that dominate a shared normal-return exit. Immediate dominators keep
/// the retained state linear even for long bodies with many storage fields.
pub(super) fn return_dominators(body: &LifetimeBody) -> Vec<bool> {
    let n = body.blocks.len();
    let mut out = vec![false; n];
    if n == 0 {
        return out;
    }
    let mut edges: Vec<_> = body.blocks.iter().map(|b| b.successors.clone()).collect();
    for (i, b) in body.blocks.iter().enumerate() {
        if b.normal_return {
            edges[i].push(n);
        }
    }
    edges.push(Vec::new());
    let parents = dominators(&edges);
    let Some(mut node) = parents[n] else {
        return out;
    };
    loop {
        out[node] = true;
        if node == 0 {
            break;
        }
        node = parents[node].unwrap();
    }
    out
}

/// Immediate dominators for blocks reachable from entry zero.
pub(super) fn dominators(edges: &[Vec<usize>]) -> Vec<Option<usize>> {
    let n = edges.len();
    if n == 0 {
        return Vec::new();
    }
    let mut predecessors = vec![Vec::new(); n];
    for (i, next) in edges.iter().enumerate() {
        for &to in next {
            predecessors[to].push(i);
        }
    }
    let mut seen = vec![false; n];
    let mut order = Vec::new();
    let mut stack = vec![(0, false)];
    while let Some((node, exit)) = stack.pop() {
        if exit {
            order.push(node);
            continue;
        }
        if std::mem::replace(&mut seen[node], true) {
            continue;
        }
        stack.push((node, true));
        stack.extend(edges[node].iter().rev().map(|&to| (to, false)));
    }
    order.reverse();
    let mut rank = vec![usize::MAX; n];
    for (i, &node) in order.iter().enumerate() {
        rank[node] = i;
    }
    let mut parents = vec![None; n];
    parents[0] = Some(0);
    loop {
        let mut changed = false;
        for &node in order.iter().skip(1) {
            let mut inputs = predecessors[node]
                .iter()
                .copied()
                .filter(|&p| parents[p].is_some());
            let Some(mut parent) = inputs.next() else {
                continue;
            };
            for mut other in inputs {
                while parent != other {
                    if rank[parent] > rank[other] {
                        parent = parents[parent].unwrap();
                    } else {
                        other = parents[other].unwrap();
                    }
                }
            }
            if parents[node] != Some(parent) {
                parents[node] = Some(parent);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    parents
}
