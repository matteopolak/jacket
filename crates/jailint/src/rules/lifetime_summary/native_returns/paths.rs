//! Bounded complete path metadata over the existing flow driver.
use super::{Event, HashMap, HashSet, MAX_PATHS, Producer, Producers, Resolver, Site, Val, Work};
const MAX_PREFIXES: usize = 128;

#[derive(Clone, Copy)]
struct Prefix {
    parent: Option<usize>,
    block: usize,
    visited: u32,
}

pub(super) struct Trace {
    pub(super) blocks: Vec<usize>,
    pub(super) block_mask: u32,
    pub(super) cuts: HashSet<(usize, usize)>,
    pub(super) branches: Vec<(Site, Val, bool)>,
    pub(super) result: Val,
    pub(super) constant: Option<u64>,
    pub(super) returned: Site,
}

/// Complete path records are joined, never independently combined guard/value sets.
pub(super) fn traces(r: &Resolver<'_>, work: &Work) -> Option<Vec<Trace>> {
    let body = r.body;
    if body.blocks.is_empty() || body.blocks.len() > 32 {
        return None;
    }
    work.charge(body.blocks.len() + body.comparisons.len() + body.address_casts.len())?;
    let mut events = 0usize;
    let mut edges = 0usize;
    for (block, b) in body.blocks.iter().enumerate() {
        events = events.checked_add(b.events.len())?;
        edges = edges.checked_add(b.successors.len())?;
        if events > 256
            || edges > 64
            || b.successors.len() > 2
            || b.successors.iter().any(|s| *s >= body.blocks.len())
        {
            if jacket_config::env::LIFETIME_STATS.is_set() {
                eprintln!(
                    "lifetime-native-stop proc={} stage=shape block={} events={} edges={}",
                    body.procedure.0, block, events, edges
                );
            }
            return None;
        }
        work.charge(b.events.len() + b.successors.len() + 1)?;
        let branches: Vec<_> = b
            .events
            .iter()
            .filter_map(|e| match e {
                Event::Branch {
                    then_block,
                    else_block,
                    ..
                } => Some((*then_block, *else_block)),
                _ => None,
            })
            .collect();
        if b.successors.len() == 2 {
            if branches.len() != 1
                || branches[0].0 == branches[0].1
                || b.successors != [branches[0].0, branches[0].1]
            {
                if jacket_config::env::LIFETIME_STATS.is_set() {
                    eprintln!(
                        "lifetime-native-stop proc={} stage=branch block={} successors={:?} branches={:?}",
                        body.procedure.0, block, b.successors, branches
                    );
                }
                return None;
            }
        } else if !branches.is_empty() {
            if jacket_config::env::LIFETIME_STATS.is_set() {
                eprintln!(
                    "lifetime-native-stop proc={} stage=folded-branch block={} successors={:?} branches={:?}",
                    body.procedure.0, block, b.successors, branches
                );
            }
            return None;
        }
    }
    work.charge(MAX_PREFIXES)?;
    let mut prefixes = Vec::with_capacity(MAX_PREFIXES);
    let mut prefix_index = HashMap::default();
    prefixes.push(Prefix {
        parent: None,
        block: 0,
        visited: 1,
    });
    let mut edge = |from: usize, to: usize, state: &mut u128| -> Option<()> {
        work.charge(2)?; // The propagated state itself is a fixed-size Copy value.
        let mut input = *state;
        let mut next = 0u128;
        while input != 0 {
            let parent = input.trailing_zeros() as usize;
            input &= input - 1;
            let prior = *prefixes.get(parent)?;
            if prior.block != from || prior.visited & (1u32 << to) != 0 {
                if jacket_config::env::LIFETIME_STATS.is_set() {
                    eprintln!(
                        "lifetime-native-stop proc={} stage=edge from={} to={} prior={} visited={}",
                        body.procedure.0, from, to, prior.block, prior.visited
                    );
                }
                return None;
            }
            work.charge(2)?;
            let id = if let Some(&id) = prefix_index.get(&(parent, to)) {
                id
            } else {
                if prefixes.len() == MAX_PREFIXES {
                    if jacket_config::env::LIFETIME_STATS.is_set() {
                        eprintln!(
                            "lifetime-native-stop proc={} stage=prefixes work={}",
                            body.procedure.0,
                            work.0.get()
                        );
                    }
                    return None;
                }
                prefixes.push(Prefix {
                    parent: Some(parent),
                    block: to,
                    visited: prior.visited | (1u32 << to),
                });
                let id = prefixes.len() - 1;
                prefix_index.insert((parent, to), id);
                id
            };
            next |= 1u128 << id;
        }
        *state = next;
        Some(())
    };
    let states = super::super::flow::states_except_edges(
        body,
        &super::super::flow::Live::all(body),
        1u128,
        &HashSet::default(),
        |into, from| {
            if work.charge(1).is_none() {
                return false;
            }
            let before = *into;
            *into |= *from;
            before != *into
        },
        |_, _, _| {
            let _ = work.charge(1);
        },
        Some(&mut edge),
    );
    if jacket_config::env::LIFETIME_STATS.is_set() {
        eprintln!(
            "lifetime-native-flow proc={} states={} prefixes={} work={}",
            body.procedure.0,
            states.is_some(),
            prefixes.len(),
            work.0.get()
        );
    }
    let states = states?;
    work.charge(body.blocks.len())?;
    let mut out = Vec::new();
    for (block, (b, state)) in body.blocks.iter().zip(states).enumerate() {
        if !b.normal_return {
            continue;
        }
        let Some(mut bits) = state else {
            continue;
        };
        // Constant null returns have no successful range to materialize. Their
        // reachability is checked separately when closing the success predicate.
        if b.events.iter().any(|e|matches!(e,Event::Return{index:0,constant:Some(0),ty,..} if r.compiler.ty.types.is_pointer(*ty))) {continue;}
        while bits != 0 {
            if out.len() == MAX_PATHS {
                if jacket_config::env::LIFETIME_STATS.is_set() {
                    eprintln!(
                        "lifetime-native-stop proc={} stage=paths work={}",
                        body.procedure.0,
                        work.0.get()
                    );
                }
                return None;
            }
            let mut id = bits.trailing_zeros() as usize;
            let block_mask = prefixes.get(id)?.visited;
            bits &= bits - 1;
            work.charge(body.blocks.len() * 4 + 1)?;
            let mut blocks = Vec::new();
            loop {
                let p = *prefixes.get(id)?;
                blocks.push(p.block);
                if let Some(parent) = p.parent {
                    id = parent;
                } else {
                    break;
                }
            }
            blocks.reverse();
            let mut cuts = HashSet::default();
            let mut branches = Vec::new();
            for pair in blocks.windows(2) {
                let (from, to) = (pair[0], pair[1]);
                for (event, e) in body.blocks[from].events.iter().enumerate() {
                    if let Event::Branch {
                        condition,
                        then_block,
                        else_block,
                        ..
                    } = e
                    {
                        if to != *then_block && to != *else_block {
                            return None;
                        }
                        cuts.insert((
                            from,
                            if to == *then_block {
                                *else_block
                            } else {
                                *then_block
                            },
                        ));
                        branches.push((
                            Site {
                                block: from,
                                event,
                            },
                            *condition,
                            to == *then_block,
                        ));
                    }
                }
            }
            let mut returns = b
                .events
                .iter()
                .enumerate()
                .filter_map(|(event, e)| match e {
                    Event::Return {
                        index: 0,
                        value,
                        constant,
                        ty,
                        ..
                    } if r.compiler.ty.types.is_pointer(*ty) => Some((
                        Site {
                            block,
                            event,
                        },
                        *value,
                        *constant,
                    )),
                    _ => None,
                });
            let (returned, result, constant) = returns.next()?;
            if returns.next().is_some() {
                return None;
            }
            out.push(Trace {
                blocks,
                block_mask,
                cuts,
                branches,
                result,
                constant,
                returned,
            });
        }
    }
    work.charge(out.len())?;
    (!out.is_empty()).then_some(out)
}

/// Remove only contradictions in actual checked Boolean equations. Comparisons
/// and loaded values remain opaque atoms; no scalar range is inferred here.
pub(super) fn feasible(trace: &Trace, producers: &Producers, work: &Work) -> Option<bool> {
    work.charge(trace.branches.len() + 1)?;
    let mut values = HashMap::default();
    for &(_, value, when) in &trace.branches {
        if values.insert(value, when).is_some_and(|old| old != when) {
            return Some(false);
        }
    }
    fn truth(
        value: Val,
        values: &HashMap<Val, bool>,
        p: &Producers,
        w: &Work,
        depth: usize,
    ) -> Option<Option<bool>> {
        w.charge(1)?;
        if depth >= 24 {
            return None;
        }
        let result = match p.get(value) {
            Some(Producer::Not(a)) => truth(a, values, p, w, depth + 1)?.map(|v| !v),
            Some(Producer::And(a, b) | Producer::Or(a, b)) => {
                let left = truth(a, values, p, w, depth + 1)?;
                let right = truth(b, values, p, w, depth + 1)?;
                if matches!(p.get(value), Some(Producer::And(..))) {
                    if left == Some(false) || right == Some(false) {
                        Some(false)
                    } else if left == Some(true) && right == Some(true) {
                        Some(true)
                    } else {
                        None
                    }
                } else if left == Some(true) || right == Some(true) {
                    Some(true)
                } else if left == Some(false) && right == Some(false) {
                    Some(false)
                } else {
                    None
                }
            }
            _ => None,
        };
        if result.zip(values.get(&value)).is_some_and(|(a, b)| a != *b) {
            return Some(None);
        }
        Some(result.or_else(|| values.get(&value).copied()))
    }
    // Test each equation without substituting its own asserted result.
    for &(_, value, when) in &trace.branches {
        let saved = values.remove(&value);
        let actual = truth(value, &values, producers, work, 0)?;
        if let Some(saved) = saved {
            values.insert(value, saved);
        }
        if actual.is_some_and(|v| v != when) {
            return Some(false);
        }
    }
    Some(true)
}
