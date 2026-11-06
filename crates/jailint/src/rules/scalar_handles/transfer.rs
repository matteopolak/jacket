use super::{
    Event, Expr, LifetimeBody, MAX_DEPTH, MAX_STATES, MAX_STEPS, ProcId, Relation, Site, Solver,
    StackRoots, State, TypeId, Val, caller, guards, native, related, reverse, scalar,
};

impl Solver<'_> {
    pub(super) fn relation(&mut self, id: ProcId) -> Option<Relation> {
        if jaic::interrupt::should_stop() {
            return None;
        }
        if let Some(s) = self.cache.get(&id) {
            return s.clone();
        }
        if self.active.len() >= MAX_DEPTH || !self.active.insert(id) {
            return None;
        }
        let result = if let Some(native) = native(self.c, id) {
            Some(native)
        } else if let Some(body) = self.bodies.get(&id).copied() {
            self.body(body, None)
        } else {
            None
        };
        self.active.remove(&id);
        if jaic::interrupt::should_stop() {
            return None;
        }
        self.cache.insert(id, result.clone());
        result
    }

    pub(super) fn body(
        &mut self,
        b: &LifetimeBody,
        probes: Option<&caller::Plan>,
    ) -> Option<Relation> {
        if b.blocks.is_empty() {
            return None;
        }
        let roots = std::rc::Rc::new(StackRoots::capture(b, MAX_STEPS)?);
        let mut entry = State {
            addresses: roots.addresses(),
            roots: Some(roots),
            ..State::default()
        };
        for (initialization_step, e) in b.blocks.iter().flat_map(|bb| &bb.events).enumerate() {
            if initialization_step >= MAX_STEPS || jaic::interrupt::should_stop() {
                return None;
            }
            if let Event::Return {
                value,
                ty,
                ..
            } = e
                && !scalar(self.c, *ty)
            {
                entry.region(*value, self.c.ty.types.size_of(*ty));
            }
        }
        let mut work = vec![(0, entry)];
        let mut out = Relation::default();
        let mut steps = 0;
        while let Some((block, mut s)) = work.pop() {
            if jaic::interrupt::should_stop() {
                return None;
            }
            if !s.visited.insert(block) {
                return None;
            }
            let bb = b.blocks.get(block)?;
            let mut states = vec![s];
            let mut branch = None;
            for (event, e) in bb.events.iter().enumerate() {
                if jaic::interrupt::should_stop() {
                    return None;
                }
                steps += states.len();
                if steps > MAX_STEPS {
                    return None;
                }
                let mut next = Vec::new();
                for mut s in states {
                    refresh(&mut s, b);
                    if let Event::Call {
                        callee,
                        args,
                        results,
                        expands: false,
                        ..
                    } = e
                    {
                        next.extend(self.call(
                            s,
                            *callee,
                            args,
                            results,
                            Site {
                                procedure: b.procedure,
                                block,
                                event,
                            },
                            probes,
                        )?);
                        continue;
                    }
                    if s.event(
                        self.c,
                        e,
                        Site {
                            procedure: b.procedure,
                            block,
                            event,
                        },
                        &mut branch,
                    ) {
                        next.push(s);
                        continue;
                    }
                    refresh(&mut s, b);
                    if s.slots.len() > 4096 || s.addresses.len() > 16384 {
                        return None;
                    }
                    next.push(s);
                }
                if next.iter().any(|s| s.liveness_exhausted) {
                    return None;
                }
                if next.len() + work.len() > MAX_STATES {
                    return None;
                }
                states = next;
            }
            for s in states {
                if bb.normal_return {
                    out.exits.push(s.into_exit()?);
                } else if let Some((condition, yes, no)) = branch {
                    let x = s.value(condition);
                    for (target, truth) in [(yes, true), (no, false)] {
                        if guards::constant_truth(&x).is_some_and(|known| known != truth) {
                            continue;
                        }
                        if x != Expr::Unknown
                            && s.conditions.iter().any(|(p, t)| p == &x && *t != truth)
                        {
                            continue;
                        }
                        let mut fork = s.clone();
                        fork.conditions.push((x.clone(), truth));
                        work.push((target, fork));
                    }
                } else {
                    for next in &bb.successors {
                        work.push((*next, s.clone()));
                    }
                }
            }
            if out.exits.len() + work.len() > MAX_STATES {
                return None;
            }
        }
        // Project away helper-internal paths that carry no scalar/parameter relation.
        // Keeping those alternatives would multiply unrelated allocation/logging branches
        // at every caller without adding a resource fact.
        let meaningful = out.exits.iter().any(|s| {
            !s.probes.is_empty()
                || s.interferes
                || !s.requirements.is_empty()
                || !s.consumed.is_empty()
                || !s.uncertain.is_empty()
                || !s.global_consumed.is_empty()
                || s.returns.iter().any(|(_, _, x)| related(x))
                || s.conditions.iter().any(|(x, _)| related(x))
        }) || b.blocks.iter().flat_map(|b| &b.events).all(|e| {
            !matches!(
                e,
                Event::Call { .. } | Event::IndirectCall { .. } | Event::Barrier
            )
        }) && self.c.pr.procs[b.procedure.0 as usize]
            .sig
            .as_ref()
            .is_some_and(|s| s.returns.contains(&TypeId::BOOL));
        if !meaningful {
            return None;
        }
        // Keep every reached exit through all exploration bounds; only then
        // coalesce alternatives with identical exported facts.
        let mut exits = Vec::new();
        for exit in out.exits {
            if !exits.contains(&exit) {
                exits.push(exit);
            }
        }
        Some(Relation {
            exits,
        })
    }
}

fn refresh(s: &mut State, b: &LifetimeBody) {
    for cmp in &b.comparisons {
        let constant = |immediate: Option<u64>, value: Val| {
            immediate.or_else(|| match s.values.get(&value) {
                Some(Expr::Constant(k)) => Some(*k),
                _ => None,
            })
        };
        let expression = if let Some(k) = constant(cmp.right_constant, cmp.right) {
            let left = cmp.left_constant.map(Expr::Constant);
            Expr::compare(
                left.as_ref().or_else(|| s.values.get(&cmp.left)),
                cmp.op,
                k,
                cmp.ty,
            )
        } else if let Some(k) = constant(cmp.left_constant, cmp.left) {
            Expr::compare(s.values.get(&cmp.right), reverse(cmp.op), k, cmp.ty)
        } else {
            continue;
        };
        if expression != Expr::Unknown {
            s.values.insert(cmp.value, expression);
        }
    }
}
