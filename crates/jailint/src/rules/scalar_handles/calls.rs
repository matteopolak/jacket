//! Instantiate ordinary checked call relations without carrying caller probes into the cache.
use super::{
    Expr, HandleEffect, ProcId, Site, Solver, State, TypeId, Val, caller, guards, related,
    substitute,
};

impl Solver<'_> {
    /// Apply one checked call to captured actuals and preserve its exit alternatives.
    pub(super) fn call(
        &mut self,
        mut s: State,
        callee: ProcId,
        args: &[jaic::ide::lifetimes::LifetimeArgument],
        results: &[(usize, Val, TypeId)],
        site: Site,
        probes: Option<&caller::Plan>,
    ) -> Option<Vec<State>> {
        let mut next = Vec::new();
        let Site {
            block,
            event,
            ..
        } = site;
        if let Some(rel) = self.relation(callee) {
            for exit in &rel.exits {
                if jaic::interrupt::should_stop() {
                    return None;
                }
                let mut fork = s.clone();
                if exit.interferes {
                    fork.opaque(args);
                }
                let subst = |x: &Expr| substitute(self.c, x, args, &s, &site);
                if exit.conditions.iter().any(|(x, truth)| {
                    guards::constant_truth(&subst(x)).is_some_and(|v| v != *truth)
                }) {
                    continue;
                }
                for (cond, v) in &exit.conditions {
                    let x = subst(cond);
                    if related(&x) && !fork.conditions.contains(&(x.clone(), *v)) {
                        fork.conditions.push((x, *v));
                    }
                }
                for (required, kind) in &exit.requirements {
                    let x = subst(required);
                    if let Expr::Fresh(token) = &x
                        && token.kind != *kind
                    {
                        fork.invalid.insert(token.clone());
                    }
                    if related(&x) && !fork.requirements.contains(&(x.clone(), *kind)) {
                        fork.requirements.push((x, *kind));
                    }
                }
                for kind in &exit.global_consumed {
                    fork.consume_kind(*kind);
                }
                if exit.global_unknown {
                    fork.unknown_global();
                }
                if !exit.global_sites.is_empty() {
                    fork.global(site.clone());
                }
                for uncertain in &exit.uncertain {
                    fork.apply_effect(
                        HandleEffect {
                            value: subst(&uncertain.value),
                            kind: uncertain.kind,
                        },
                        false,
                    );
                }
                for consumed in &exit.consumed {
                    fork.apply_effect(
                        HandleEffect {
                            value: subst(&consumed.value),
                            kind: consumed.kind,
                        },
                        true,
                    );
                }
                for (idx, slot, x) in &exit.returns {
                    if let Some((_, val, ty)) = results.iter().find(|(i, _, _)| i == idx) {
                        if let Some(slot) = slot {
                            fork.region(*val, self.c.ty.types.size_of(*ty));
                            fork.put(*val, *slot, subst(x));
                        } else {
                            fork.values.insert(*val, subst(x));
                        }
                    }
                }
                for (token, frames) in &exit.frames {
                    if frames.is_empty() {
                        continue;
                    }
                    if let Expr::Fresh(token) = subst(&Expr::Fresh(token.clone())) {
                        fork.frame_for(token, site.clone());
                    }
                }
                // A callee's explicit release invalidates only matching acquisitions.
                for token in &exit.invalid {
                    if let Expr::Fresh(t) = subst(&Expr::Fresh(token.clone())) {
                        fork.invalid.insert(t);
                    }
                }
                if let Some(plan) = probes {
                    plan.capture(&mut fork, block, event, results);
                }
                if fork.liveness_exhausted {
                    return None;
                }
                if !next.contains(&fork) {
                    next.push(fork);
                }
            }
            return Some(next);
        }
        s.opaque(args);
        s.global(Site {
            procedure: site.procedure,
            block,
            event,
        });
        for (_, v, _) in results {
            s.values.insert(*v, Expr::Unknown);
        }
        if let Some(plan) = probes {
            plan.capture(&mut s, block, event, results);
        }
        Some(vec![s])
    }
}
