//! Resolve runtime context reads only when every incoming CFG path selects one address.
use jaic::fxhash::{HashMap, HashSet};
use jaic::ide::lifetimes::{LifetimeBody, LifetimeEvent as Event};
use jaic::ir::Val;

pub(super) fn aliases(body: &LifetimeBody) -> HashMap<Val, Val> {
    if !body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .any(|e| matches!(e, Event::ContextRead { .. }))
    {
        return HashMap::default();
    }
    let mut incoming = vec![HashSet::default(); body.blocks.len()];
    let mut outgoing = incoming.clone();
    if incoming.is_empty() {
        return HashMap::default();
    }
    incoming[0].insert(None);
    loop {
        let mut changed = false;
        for (i, block) in body.blocks.iter().enumerate() {
            let mut state = incoming[i].clone();
            if state.is_empty() {
                continue;
            }
            for event in &block.events {
                if let Event::ContextSet {
                    value,
                } = event
                {
                    state.clear();
                    state.insert(Some(*value));
                }
            }
            if state != outgoing[i] {
                outgoing[i].clone_from(&state);
                changed = true;
                for &next in &block.successors {
                    incoming[next].extend(&state);
                }
            }
        }
        if !changed {
            break;
        }
    }
    let mut reads = HashMap::default();
    for (i, block) in body.blocks.iter().enumerate() {
        let mut state = incoming[i].clone();
        for event in &block.events {
            match event {
                Event::ContextSet {
                    value,
                } => {
                    state.clear();
                    state.insert(Some(*value));
                }
                Event::ContextRead {
                    value,
                } => {
                    reads.insert(*value, state.clone());
                }
                Event::ContextAlias {
                    value,
                    source,
                } => {
                    reads.insert(*value, [Some(*source)].into_iter().collect());
                }
                _ => {}
            }
        }
    }
    // A loop can save and restore the same context through a cycle of reads.
    // Follow the closure to its concrete anchors; revisiting an edge adds no root.
    // A rootless cycle or two distinct incoming addresses remains unresolved.
    let entry = reads
        .iter()
        .filter(|(_, sources)| sources.len() == 1 && sources.contains(&None))
        .map(|(&value, _)| value)
        .min_by_key(|v| v.0);
    fn resolve(
        value: Val,
        reads: &HashMap<Val, HashSet<Option<Val>>>,
        entry: Option<Val>,
    ) -> Option<Val> {
        let mut seen = HashSet::default();
        let mut queue = vec![value];
        let mut root = None;
        while let Some(value) = queue.pop() {
            if !seen.insert(value) {
                continue;
            }
            if let Some(sources) = reads.get(&value) {
                if sources.is_empty() {
                    return None;
                }
                for source in sources {
                    if let Some(source) = source {
                        queue.push(*source);
                    } else {
                        let source = entry?;
                        if root.is_some_and(|old| old != source) {
                            return None;
                        }
                        root = Some(source);
                    }
                }
            } else {
                if root.is_some_and(|old| old != value) {
                    return None;
                }
                root = Some(value);
            }
        }
        root
    }
    reads
        .keys()
        .filter_map(|&v| resolve(v, &reads, entry).map(|root| (v, root)))
        .collect()
}
