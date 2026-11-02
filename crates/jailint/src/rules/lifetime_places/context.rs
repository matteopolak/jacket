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
    fn resolve(
        value: Val,
        reads: &HashMap<Val, HashSet<Option<Val>>>,
        seen: &mut HashSet<Val>,
    ) -> Option<Val> {
        let Some(sources) = reads.get(&value) else {
            return Some(value);
        };
        if !seen.insert(value) || sources.is_empty() {
            return None;
        }
        let mut root = None;
        for source in sources {
            let source = resolve((*source)?, reads, seen)?;
            if root.is_some_and(|r| r != source) {
                return None;
            }
            root = Some(source);
        }
        seen.remove(&value);
        root
    }
    reads
        .keys()
        .filter_map(|&v| resolve(v, &reads, &mut HashSet::default()).map(|root| (v, root)))
        .collect()
}
