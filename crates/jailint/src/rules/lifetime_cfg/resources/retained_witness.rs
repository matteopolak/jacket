//! Shortest path from a particular start to release, excluding completion edges.
use super::super::tasks::{Op as Task, Token};
use super::{CfgOp, HashMap, HashSet, Model, Span, VecDeque};

type Witness = (Vec<(Span, bool)>, String, usize);

pub(super) fn path(model: &Model, start: usize, end: usize, token: Token) -> Option<Witness> {
    let mut ids = HashMap::from_iter([(start, 0)]);
    let mut nodes = vec![start];
    let mut edges = vec![Vec::new()];
    let mut previous = vec![None];
    let mut terminal = vec![false];
    let mut queue = VecDeque::from([0]);
    let mut target = None;
    while let Some(id) = queue.pop_front() {
        let block = nodes[id];
        if block == end {
            target.get_or_insert(id);
            terminal[id] = true;
            continue;
        }
        if matches!(model.blocks[block].op, CfgOp::Task(Task::End(t)) if t == token) {
            continue;
        }
        for &next in &model.blocks[block].next {
            if matches!(model.blocks[block].op, CfgOp::Decision(_, yes, _, Some(t), _) if t == token && next == yes)
            {
                continue;
            }
            let to = if let Some(&id) = ids.get(&next) {
                id
            } else {
                let to = nodes.len();
                ids.insert(next, to);
                nodes.push(next);
                edges.push(Vec::new());
                previous.push(Some(id));
                terminal.push(false);
                queue.push_back(to);
                to
            };
            edges[id].push(to);
        }
    }
    let mut current = target?;
    let mut path = vec![nodes[current]];
    while let Some(p) = previous[current] {
        path.push(nodes[p]);
        current = p;
    }
    path.reverse();
    let mut seen_decisions = HashSet::default();
    let decisions = path
        .windows(2)
        .filter_map(|pair| match model.blocks[pair[0]].op {
            CfgOp::Decision(span, yes, no, _, _) if pair[1] == yes || pair[1] == no => {
                Some((span, pair[1] == yes))
            }
            _ => None,
        })
        .filter(|decision| seen_decisions.insert(*decision))
        .collect();
    Some((
        decisions,
        super::super::witness::alternatives(&edges, &terminal),
        path.len(),
    ))
}
