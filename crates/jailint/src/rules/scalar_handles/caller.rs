//! Caller obligations use the same native identities and effects as wrapper inference.
use super::{
    Analysis, CallOpenings, Event, Expr, FileId, HashMap, HashSet, LifetimeBody, MAX_STEPS,
    NativeKind, Relation, Site, Slot, Solver, State, TypeId, Val, Validity, guards,
};

const MAX_PROBES: usize = 64;
const MAX_PROOF_WORK: usize = 65_536;
const MAX_SAMPLES: usize = 4096;

struct Probe {
    site: Site,
    result: usize,
    slot: Option<Slot>,
    kinds: Vec<(NativeKind, Validity)>,
}

pub(super) struct Plan {
    probes: Vec<Probe>,
    at: HashMap<(usize, usize), Vec<usize>>,
}

impl Plan {
    /// Distinct call/result/slot captures share one caller traversal. No summary
    /// token is re-instantiated outside the checked call's actual substitution.
    fn collect(body: &LifetimeBody, analysis: &Analysis) -> Option<Self> {
        let mut plan = Self {
            probes: Vec::new(),
            at: HashMap::default(),
        };
        for (block, b) in body.blocks.iter().enumerate() {
            if jaic::interrupt::should_stop() {
                return None;
            }
            for (event, e) in b.events.iter().enumerate() {
                let Event::Call {
                    callee,
                    results,
                    expands: false,
                    ..
                } = e
                else {
                    continue;
                };
                let Some(opening) = analysis.opening(*callee) else {
                    continue;
                };
                if opening.incomplete
                    || opening
                        .alternatives
                        .iter()
                        .any(|c| !c.requires_noninterference.is_empty())
                {
                    continue;
                }
                for cap in &opening.alternatives {
                    if !results.iter().any(|(result, _, _)| *result == cap.result) {
                        continue;
                    }
                    if let Some(probe) = plan.probes.iter_mut().find(|probe| {
                        probe.site.block == block
                            && probe.site.event == event
                            && probe.result == cap.result
                            && probe.slot == cap.slot
                    }) {
                        if !probe.kinds.contains(&(cap.kind, cap.validity)) {
                            probe.kinds.push((cap.kind, cap.validity));
                        }
                        continue;
                    }
                    if plan.probes.len() >= MAX_PROBES {
                        return None;
                    }
                    let id = plan.probes.len();
                    plan.at.entry((block, event)).or_default().push(id);
                    plan.probes.push(Probe {
                        site: Site {
                            procedure: body.procedure,
                            block,
                            event,
                        },
                        result: cap.result,
                        slot: cap.slot,
                        kinds: vec![(cap.kind, cap.validity)],
                    });
                }
            }
        }
        Some(plan)
    }

    /// Absence means not executed; an unsupported executed capture is explicitly
    /// `Expr::Unknown`. Capture before later stores can change the local's bits.
    pub(super) fn capture(
        &self,
        state: &mut State,
        block: usize,
        event: usize,
        results: &[(usize, Val, TypeId)],
    ) {
        for &id in self.at.get(&(block, event)).into_iter().flatten() {
            let probe = &self.probes[id];
            let value = results
                .iter()
                .find(|(index, _, _)| *index == probe.result)
                .map_or(Expr::Unknown, |(_, value, _)| {
                    probe
                        .slot
                        .map_or_else(|| state.value(*value), |slot| state.get(*value, slot))
                });
            state.probes.insert(id, value);
        }
    }

    /// At least one valid live witness is required. Every applicable acquired
    /// exit must also preserve it; a global/alias close or frame rejects it.
    fn classify(&self, relation: &Relation) -> Option<Vec<bool>> {
        let mut nodes = 0usize;
        let mut condition_nodes = 0usize;
        for exit in &relation.exits {
            for (expr, _) in &exit.conditions {
                let n = expr.shape()?.nodes;
                nodes = nodes.checked_add(n)?;
                condition_nodes = condition_nodes.checked_add(n)?;
            }
            for (_, _, expr) in &exit.returns {
                nodes = nodes.checked_add(expr.shape()?.nodes)?;
            }
        }
        let sample_bound = nodes.checked_mul(3)?.checked_add(7)?;
        let work = sample_bound
            .checked_mul(condition_nodes)?
            .checked_mul(self.probes.len())?
            .checked_add(nodes.checked_mul(3)?)?
            .checked_add(relation.exits.len().checked_mul(self.probes.len())?)?;
        if sample_bound > MAX_SAMPLES || work > MAX_PROOF_WORK {
            return None;
        }
        let mut samples = HashMap::<NativeKind, Vec<u64>>::default();
        let mut witnessed = vec![false; self.probes.len()];
        let mut rejected = vec![false; self.probes.len()];
        for exit in &relation.exits {
            if jaic::interrupt::should_stop() {
                return None;
            }
            for (&id, expr) in &exit.probes {
                let probe = self.probes.get(id)?;
                match expr {
                    Expr::Fresh(token) if probe.kinds.contains(&(token.kind, token.validity)) => {
                        let partition = samples
                            .entry(token.kind)
                            .or_insert_with(|| guards::samples(relation, token));
                        if partition.len() > MAX_SAMPLES {
                            return None;
                        }
                        if !partition.iter().any(|value| {
                            token.validity.accepts(*value)
                                && guards::feasible(&exit.conditions, token, *value)
                        }) {
                            continue;
                        }
                        witnessed[id] = true;
                        if exit.invalid.contains(token)
                            || exit.returns.iter().any(|(_,_,value)|matches!(value,Expr::Fresh(returned) if returned==token))
                            || exit
                                .frames
                                .get(token)
                                .is_some_and(|sites| !sites.is_empty())
                        {
                            rejected[id] = true;
                        }
                    }
                    Expr::Constant(value)
                        if probe
                            .kinds
                            .iter()
                            .all(|(_, validity)| !validity.accepts(*value)) => {}
                    _ => rejected[id] = true,
                }
            }
        }
        if jaic::interrupt::should_stop() {
            return None;
        }
        Some(
            witnessed
                .into_iter()
                .zip(rejected)
                .map(|(seen, bad)| seen && !bad)
                .collect(),
        )
    }
}

impl Solver<'_> {
    /// Probe entry bypasses the ordinary relation cache. Recursive ordinary
    /// calls still use unprobed summaries; no caller observation enters them.
    pub(super) fn caller_openings(
        &mut self,
        analysis: &Analysis,
        files: &HashSet<FileId>,
    ) -> CallOpenings {
        let mut out = CallOpenings::default();
        let ids: Vec<_> = self.bodies.keys().copied().collect();
        for id in ids {
            if jaic::interrupt::should_stop() {
                return CallOpenings::default();
            }
            let body = self.bodies[&id];
            if !selected_calls(body, files) {
                continue;
            }
            let Some(plan) = Plan::collect(body, analysis).filter(|plan| !plan.probes.is_empty())
            else {
                continue;
            };
            out.probed_bodies += 1;
            let Some(relation) = self.body(body, Some(&plan)) else {
                continue;
            };
            let Some(accepted) = plan.classify(&relation) else {
                continue;
            };
            for (probe, accepted) in plan.probes.into_iter().zip(accepted) {
                if accepted {
                    out.calls
                        .entry((id, probe.site.block, probe.site.event))
                        .or_default()
                        .push(probe.result);
                }
            }
        }
        for positions in out.calls.values_mut() {
            positions.sort_unstable();
            positions.dedup();
        }
        if jaic::interrupt::should_stop() {
            CallOpenings::default()
        } else {
            out
        }
    }
}

/// Source queries carry a `FileId`, so every checked instance of a requested span
/// necessarily passes. Exhaustion includes the body rather than losing coverage.
fn selected_calls(body: &LifetimeBody, files: &HashSet<FileId>) -> bool {
    if files.is_empty() {
        return true;
    }
    let mut work = 0;
    for block in &body.blocks {
        for event in &block.events {
            work += 1;
            if work > MAX_STEPS || jaic::interrupt::should_stop() {
                return true;
            }
            if matches!(event,Event::Call{span,..} if files.contains(&span.file)) {
                return true;
            }
        }
    }
    false
}
