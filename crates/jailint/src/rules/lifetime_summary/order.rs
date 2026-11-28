//! The order a body frees and writes the same place: the effect sets of a summary are unordered,
//! so this keeps the one order a caller needs to apply them right.
//!
//! ```jai
//! swap_in :: (h: *Holder) { free(h.p); h.p = New(int); }  // frees what h.p held, then replaces it
//! clear   :: (h: *Holder) { h.* = .{}; free(h); }         // replaces, then frees the new value
//! ```
use super::{Effect, HashSet, Kind, LifetimeBody, Path};

/// One effect of an event on a place of a parameter, in the order the body does them.
#[derive(Clone, Debug)]
pub(super) enum Item {
    Free(Path),
    Write(Path),
}

impl Item {
    pub(super) fn of(effect: &Effect) -> Option<Item> {
        if effect.guard.is_some() || !effect.exported() {
            return None;
        }

        match effect.kind {
            Kind::Free => Some(Item::Free(effect.target.clone())),
            Kind::Write => Some(Item::Write(effect.target.clone())),
            _ => None,
        }
    }
}

fn related(a: &Path, b: &Path) -> bool {
    a.root == b.root && (a.steps.starts_with(&b.steps) || b.steps.starts_with(&a.steps))
}

/// The places `items` (per block, in order) frees and then writes on every path that gets to the
/// write: `(freed, written)` for each write of `written` that a free of `freed` always precedes.
/// A place written before it is freed, or freed on only some of the paths to a write, is not
/// listed.
pub(super) fn frees_before_writes(
    body: &LifetimeBody,
    live: &super::flow::Live,
    items: &[Vec<Item>],
) -> HashSet<(Path, Path)> {
    let mut frees: Vec<&Path> = Vec::new();
    let mut writes: Vec<&Path> = Vec::new();

    for item in items.iter().flatten() {
        match item {
            Item::Free(path) if !frees.contains(&path) => frees.push(path),
            Item::Write(path) if !writes.contains(&path) => writes.push(path),
            _ => {}
        }
    }

    let mut pairs = HashSet::default();

    for freed in frees {
        let related_writes: Vec<&Path> = writes
            .iter()
            .copied()
            .filter(|w| related(freed, w))
            .collect();

        if related_writes.is_empty() {
            continue;
        }

        // Whether `freed` has been freed on every path to the start of each block.
        let mut entry = vec![true; body.blocks.len()];

        entry[0] = false;

        loop {
            let mut changed = false;

            for b in (0..body.blocks.len()).filter(|&b| live.contains(b)) {
                let mut state = entry[b];

                for item in &items[b] {
                    if matches!(item, Item::Free(p) if p == freed) {
                        state = true;
                    }
                }

                for &next in &body.blocks[b].successors {
                    if next != 0 && live.contains(next) && entry[next] && !state {
                        entry[next] = false;
                        changed = true;
                    }
                }
            }

            if !changed {
                break;
            }
        }

        let mut seen: HashSet<&Path> = HashSet::default();
        let mut early: HashSet<&Path> = HashSet::default();

        for b in (0..body.blocks.len()).filter(|&b| live.contains(b)) {
            let mut state = entry[b];

            for item in &items[b] {
                match item {
                    Item::Free(p) if p == freed => state = true,
                    Item::Write(p) if related_writes.contains(&p) => {
                        seen.insert(p);

                        if !state {
                            early.insert(p);
                        }
                    }
                    _ => {}
                }
            }
        }

        for written in seen.difference(&early) {
            pairs.insert((freed.clone(), (*written).clone()));
        }
    }

    pairs
}

/// The items of one call, in the order its effects are done. A call's effects carry no order of
/// their own, so writes come before frees (a free is never claimed to precede a write) except for
/// the places the callee itself frees before writing (`pairs`, its `free_then_write`): there the
/// frees come first, so a wrapper of `swap_in` frees then writes too.
pub(super) fn call_items(
    writes: &[&Effect],
    frees: &[&Effect],
    pairs: Option<&HashSet<(Path, Path)>>,
) -> Vec<Item> {
    let tied = |effect: &Effect, free: bool| {
        pairs.is_some_and(|pairs| {
            pairs.iter().any(|(f, w)| {
                let own = if free {
                    f
                } else {
                    w
                };

                effect.target.steps.ends_with(&own.steps)
            })
        })
    };
    let mut items = Vec::new();

    items.extend(
        writes
            .iter()
            .filter(|e| !tied(e, false))
            .filter_map(|e| Item::of(e)),
    );
    items.extend(
        frees
            .iter()
            .filter(|e| tied(e, true))
            .filter_map(|e| Item::of(e)),
    );
    items.extend(
        writes
            .iter()
            .filter(|e| tied(e, false))
            .filter_map(|e| Item::of(e)),
    );
    items.extend(
        frees
            .iter()
            .filter(|e| !tied(e, true))
            .filter_map(|e| Item::of(e)),
    );
    items
}
