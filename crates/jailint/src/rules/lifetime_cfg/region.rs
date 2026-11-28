//! The blocks one candidate's dataflow has to visit.
//!
//! A candidate only changes state after it is seeded, and only matters up to the last block
//! that observes one of its cells. Every other block carries the entry state unchanged, so a
//! sweep over the whole CFG for each candidate costs quadratic time in the body's size.
use super::{Block, HashMap, HashSet};
use std::collections::VecDeque;

/// Predecessors, reachability from the entry and a topological order of the strongly
/// connected components of one CFG, shared by every candidate checked against it.
pub(super) struct Graph {
    incoming: Vec<Vec<usize>>,
    reached: Vec<bool>,
    /// Along every edge the order never decreases, and it is equal only inside one cycle.
    order: Vec<usize>,
}

impl Graph {
    pub(super) fn new(blocks: &[Block]) -> Self {
        let mut incoming = vec![Vec::new(); blocks.len()];

        for (from, block) in blocks.iter().enumerate() {
            for &to in &block.next {
                incoming[to].push(from);
            }
        }

        let mut reached = vec![false; blocks.len()];

        if !blocks.is_empty() {
            let mut stack = vec![0];

            reached[0] = true;

            while let Some(i) = stack.pop() {
                for &next in &blocks[i].next {
                    if !reached[next] {
                        reached[next] = true;
                        stack.push(next);
                    }
                }
            }
        }

        Self {
            incoming,
            reached,
            order: components(blocks),
        }
    }

    /// Whether `block` is reachable from the entry.
    pub(super) fn reached(&self, block: usize) -> bool {
        self.reached[block]
    }

    /// The blocks reachable from `seed` without passing a block ordered after every one of
    /// `observations`, and those of them that also receive the entry state from outside.
    pub(super) fn scope(
        &self,
        blocks: &[Block],
        seed: usize,
        observations: impl IntoIterator<Item = usize>,
    ) -> Scope {
        let last = observations
            .into_iter()
            .map(|b| self.order[b])
            .max()
            .unwrap_or(self.order[seed]);
        let mut member = HashSet::default();
        let mut queue = VecDeque::new();

        if self.order[seed] <= last {
            member.insert(seed);
            queue.push_back(seed);
        }

        while let Some(i) = queue.pop_front() {
            for &next in &blocks[i].next {
                if self.order[next] <= last && member.insert(next) {
                    queue.push_back(next);
                }
            }
        }

        let mut starts: Vec<_> = member
            .iter()
            .copied()
            .filter(|&b| {
                b == 0
                    || self.incoming[b]
                        .iter()
                        .any(|&p| !member.contains(&p) && self.reached[p])
            })
            .collect();

        starts.sort_unstable();
        Scope {
            member: Some(member),
            starts,
        }
    }
}

/// Where a sweep begins and which blocks it may enter.
pub(super) struct Scope {
    /// `None` admits every block.
    member: Option<HashSet<usize>>,
    pub(super) starts: Vec<usize>,
}

impl Scope {
    /// Every block reachable from the entry.
    pub(super) fn whole() -> Self {
        Self {
            member: None,
            starts: vec![0],
        }
    }

    pub(super) fn contains(&self, block: usize) -> bool {
        self.member.as_ref().is_none_or(|m| m.contains(&block))
    }

    /// The states before a sweep, holding `entry` at its starts, and the starts to visit.
    pub(super) fn begin<'a>(
        &self,
        graph: &'a Graph,
        entry: Vec<u8>,
    ) -> (States<'a>, VecDeque<usize>) {
        let mut states = States::new(graph, entry);
        let entry = states.entry().to_vec();

        for &start in &self.starts {
            states.join(start, &entry);
        }

        (states, self.starts.iter().copied().collect())
    }
}

/// The states a sweep computed. A block it did not visit but the entry reaches holds the
/// entry state, which is what every path that avoids the candidate leaves there.
pub(super) struct States<'a> {
    graph: &'a Graph,
    swept: HashMap<usize, Vec<u8>>,
    entry: Vec<u8>,
}

impl<'a> States<'a> {
    pub(super) fn new(graph: &'a Graph, entry: Vec<u8>) -> Self {
        Self {
            graph,
            swept: HashMap::default(),
            entry,
        }
    }

    pub(super) fn entry(&self) -> &[u8] {
        &self.entry
    }

    pub(super) fn get(&self, block: usize) -> Option<&Vec<u8>> {
        self.swept
            .get(&block)
            .or_else(|| self.graph.reached(block).then_some(&self.entry))
    }

    pub(super) fn visited(&self, block: usize) -> Option<&Vec<u8>> {
        self.swept.get(&block)
    }

    /// Joins `incoming` into the state at `block`; whether that changed it.
    pub(super) fn join(&mut self, block: usize, incoming: &[u8]) -> bool {
        let Some(previous) = self.swept.get_mut(&block) else {
            self.swept.insert(block, incoming.to_vec());
            return true;
        };
        let mut changed = false;

        for (old, &new) in previous.iter_mut().zip(incoming) {
            let joined = *old | new;

            changed |= joined != *old;
            *old = joined;
        }

        changed
    }

    /// The blocks a sweep visited, in order.
    pub(super) fn swept(&self) -> Vec<usize> {
        let mut blocks: Vec<_> = self.swept.keys().copied().collect();

        blocks.sort_unstable();
        blocks
    }
}

/// Topological order of the strongly connected components (iterative Tarjan).
fn components(blocks: &[Block]) -> Vec<usize> {
    const NONE: usize = usize::MAX;
    let n = blocks.len();
    let mut index = vec![NONE; n];
    let mut low = vec![0; n];
    let mut on_stack = vec![false; n];
    let mut stack = Vec::new();
    let mut component = vec![NONE; n];
    let mut count = 0;
    let mut counter = 0;

    for root in 0..n {
        if index[root] != NONE {
            continue;
        }

        let mut work = vec![(root, 0usize)];

        index[root] = counter;
        low[root] = counter;
        counter += 1;
        stack.push(root);
        on_stack[root] = true;

        while let Some(&(node, edge)) = work.last() {
            if let Some(&next) = blocks[node].next.get(edge) {
                work.last_mut().unwrap().1 += 1;

                if index[next] == NONE {
                    index[next] = counter;
                    low[next] = counter;
                    counter += 1;
                    stack.push(next);
                    on_stack[next] = true;
                    work.push((next, 0));
                } else if on_stack[next] {
                    low[node] = low[node].min(index[next]);
                }
            } else {
                work.pop();

                if let Some(&(parent, _)) = work.last() {
                    low[parent] = low[parent].min(low[node]);
                }

                if low[node] == index[node] {
                    loop {
                        let member = stack.pop().unwrap();

                        on_stack[member] = false;
                        component[member] = count;

                        if member == node {
                            break;
                        }
                    }

                    count += 1;
                }
            }
        }
    }

    // Tarjan finishes a component after everything it reaches.
    component.iter().map(|&c| count - 1 - c).collect()
}
