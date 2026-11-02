//! Control-flow fixed points and witness cuts for summaries.
use super::{Effect, Event, HashMap, HashSet, LifetimeBody, MAX_CHAIN, ProcId, Span, Summary};

pub(super) fn returning_blocks(
    body: &LifetimeBody,
    all: &HashMap<ProcId, Summary>,
) -> HashSet<usize> {
    let stopped: Vec<_> = body.blocks.iter().map(|b| b.events.iter().any(|e| matches!(e, Event::Call { callee, expands: false, .. } if all.get(callee).is_some_and(|s| s.normal_return == Some(false))))).collect();
    let mut forward = HashSet::default();
    let mut queue = std::collections::VecDeque::from([0]);
    while let Some(i) = queue.pop_front() {
        if forward.insert(i) && !stopped[i] {
            queue.extend(&body.blocks[i].successors);
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
    let mut back = HashSet::default();
    let mut queue: std::collections::VecDeque<_> = body
        .blocks
        .iter()
        .enumerate()
        .filter(|(i, b)| b.normal_return && !stopped[*i])
        .map(|(i, _)| i)
        .collect();
    while let Some(i) = queue.pop_front() {
        if back.insert(i) {
            queue.extend(&incoming[i]);
        }
    }
    forward.retain(|i| back.contains(i));
    forward
}

pub(super) fn guaranteed(
    body: &LifetimeBody,
    effects: &[Vec<(Effect, Vec<Span>)>],
    target: &Effect,
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
            incoming[next].push(i);
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
