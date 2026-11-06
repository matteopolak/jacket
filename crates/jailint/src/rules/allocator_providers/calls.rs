//! Direct-call composition, preserving observed-prefix and per-exit alternatives.
use super::*;

pub(super) enum Next {
    Continue,
    Forked,
    Partial,
}

impl Solver<'_> {
    pub(super) fn call(
        &mut self,
        e: &Event,
        anchor: &Site,
        s: &mut State,
        summary: &mut Summary,
        next: &mut Vec<State>,
    ) -> Next {
        let Event::Call {
            callee,
            args,
            results,
            memory,
            ..
        } = e
        else {
            return Next::Continue;
        };
        if let Some(child) = self.summary(*callee) {
            let mut probe = s.clone();
            let (uses, stores) = {
                let mut bind = Bindings {
                    c: self.c,
                    args,
                    state: &mut probe,
                    call: anchor,
                    summary: &child,
                };
                let uses: Vec<_> = child
                    .observed_uses
                    .iter()
                    .filter_map(|u| bind.use_(u))
                    .collect();
                let stores: Vec<_> = child
                    .observed_stores
                    .iter()
                    .filter_map(|u| bind.store(u))
                    .collect();
                (uses, stores)
            };
            for u in uses {
                if summary.observed_uses.len() < MAX_ITEMS && !summary.observed_uses.contains(&u) {
                    summary.observed_uses.push(u);
                }
            }
            for u in stores {
                if summary.observed_stores.len() < MAX_ITEMS
                    && !summary.observed_stores.contains(&u)
                {
                    summary.observed_stores.push(u);
                }
            }
            summary.partial |= child.partial;
            for exit in &child.exits {
                let mut fork = s.clone();
                let (uses, stores, values, guards) = {
                    let mut bind = Bindings {
                        c: self.c,
                        args,
                        state: &mut fork,
                        call: anchor,
                        summary: &child,
                    };
                    let uses: Vec<_> = exit.uses.iter().filter_map(|u| bind.use_(u)).collect();
                    let stores: Vec<_> = exit.stores.iter().filter_map(|s| bind.store(s)).collect();
                    let values: Vec<_> = exit
                        .return_values
                        .iter()
                        .map(|(i, v)| (*i, bind.value(v)))
                        .collect();
                    let guards = bind.guards(&exit.guards);
                    (uses, stores, values, guards)
                };
                let mut possible = true;
                for mut guard in guards {
                    guard.predicate = fork.simplified(&guard.predicate);
                    if guard.predicate.known().is_some_and(|k| k != guard.truth) {
                        possible = false;
                        break;
                    }
                    if guard.predicate.known().is_none() {
                        fork.guards.push(guard);
                    }
                }
                if !possible {
                    continue;
                }
                if exit.memory_effects || child.partial {
                    for u in &uses {
                        if let Some(Address {
                            root: Root::Private(v),
                            ..
                        }) = &u.data.address
                        {
                            fork.expose(*v);
                        }
                    }
                    opaque(self.c, &mut fork, args);
                    fork.memory_effects = true;
                }
                fork.uses.extend(uses);
                fork.stores.extend(stores);
                fork.incomplete |= exit.incomplete || child.partial;
                for (index, v, ty) in results {
                    let value = values
                        .iter()
                        .find(|(i, _)| i == index)
                        .map_or_else(|| Value::unknown(*ty), |(_, v)| v.retyped(self.c, *ty));
                    fork.values.insert(*v, value);
                }
                observe(summary, &fork);
                if !fork.bounded() {
                    summary.partial = true;
                    return Next::Partial;
                }
                next.push(fork);
            }
            return Next::Forked;
        }
        let readonly = memory.as_ref().is_some_and(|m| m.writes == 0)
            || memory
                .as_ref()
                .and_then(|m| m.count)
                .and_then(|i| args.iter().find(|a| a.formal == i as usize))
                .is_some_and(|a| a.constant == Some(0));
        if !readonly {
            opaque(self.c, s, args);
            s.memory_effects = true;
            s.incomplete = true;
        }
        for (_, v, ty) in results {
            s.values.insert(*v, Value::unknown(*ty));
        }
        Next::Continue
    }
}
