//! Indirect calls intersect guarantees over every closed target; an open-world member
//! removes guarantees without erasing known may effects.
use super::{Effect, Event, HashMap, HashSet, ProcId, Resolver, Span, Summary};
use crate::rules::lifetime_targets::Set;

impl Resolver<'_> {
    pub fn unknown_effects(&self, event: &Event, all: &HashMap<ProcId, Summary>) -> Vec<Effect> {
        let (Event::Call {
            args,
            expands: false,
            ..
        }
        | Event::IndirectCall {
            args,
            allocator: false,
            ..
        }) = event
        else {
            return Vec::new();
        };
        let targets = self.call_targets(event);
        let unknown = targets.unknown
            || targets.procedures.iter().any(|p| {
                let info = &self.compiler.pr.procs[p.0 as usize];
                info.lit.body.is_none()
                    && !self.compiler.proc_has_note(*p, "NoLifetimeEffects")
                    && all.get(p).is_none_or(|s| {
                        s.memory.is_none() && s.may.is_empty() && s.returns.is_empty()
                    })
            });
        if !unknown {
            return Vec::new();
        }
        args.iter()
            .filter(|a| {
                self.compiler.ty.types.is_pointer(a.ty)
                    || matches!(
                        self.compiler.ty.types.kind(a.ty),
                        jaic::types::TypeKind::Array { .. }
                            | jaic::types::TypeKind::String
                            | jaic::types::TypeKind::Any
                    )
            })
            .filter_map(|arg| {
                self.argument_trace(arg, all, &mut HashSet::default(), 0)
                    .map(|target| Effect {
                        target,
                        kind: super::Kind::Unknown,
                        guard: None,
                    })
            })
            .collect()
    }

    pub fn call_targets(&self, event: &Event) -> Set {
        match event {
            Event::Call {
                callee, ..
            } => Set {
                procedures: HashSet::from_iter([*callee]),
                unknown: false,
                ..Set::default()
            },
            Event::IndirectCall {
                callee,
                ty,
                ..
            } => {
                let mut set = self.targets.get(self.body.procedure, *callee);
                set.procedures.retain(|p| {
                    let compatible = self.compiler.pr.procs[p.0 as usize]
                        .sig
                        .as_ref()
                        .is_some_and(|s| s.ty == *ty);
                    set.unknown |= !compatible;
                    compatible
                });
                set
            }
            _ => Set {
                unknown: true,
                ..Set::default()
            },
        }
    }

    pub fn call_effects(
        &self,
        event: &Event,
        all: &HashMap<ProcId, Summary>,
        guaranteed: bool,
    ) -> Vec<(Effect, Vec<Span>)> {
        let (Event::Call {
            args, ..
        }
        | Event::IndirectCall {
            args, ..
        }) = event
        else {
            return Vec::new();
        };
        let targets = self.call_targets(event);
        if guaranteed && targets.unknown {
            return Vec::new();
        }
        let mut result = HashMap::<Effect, Vec<Span>>::default();
        for (index, callee) in targets.ordered().into_iter().enumerate() {
            let current: HashMap<_, _> = self
                .effects(callee, args, all, guaranteed)
                .into_iter()
                .collect();
            if index == 0 || !guaranteed {
                for (effect, chain) in current {
                    result.entry(effect).or_insert(chain);
                }
            } else {
                result.retain(|effect, _| current.contains_key(effect));
            }
        }
        // An effect exported by a known member remains a may fact in an open set.
        result.into_iter().collect()
    }

    pub fn call_stops(&self, event: &Event, all: &HashMap<ProcId, Summary>) -> bool {
        let targets = self.call_targets(event);
        !targets.unknown
            && !targets.procedures.is_empty()
            && targets
                .procedures
                .iter()
                .all(|p| all.get(p).is_some_and(|s| s.normal_return == Some(false)))
    }
}

pub(super) fn edges(
    event: &Event,
    procedure: ProcId,
    targets: &crate::rules::lifetime_targets::Universe,
) -> Vec<ProcId> {
    match event {
        Event::Call {
            callee,
            expands: false,
            ..
        } => vec![*callee],
        Event::IndirectCall {
            callee, ..
        } => targets.get(procedure, *callee).ordered(),
        _ => Vec::new(),
    }
}

impl Resolver<'_> {
    pub fn released_formals(
        &self,
        event: &Event,
        all: &HashMap<ProcId, Summary>,
        guaranteed: bool,
    ) -> HashSet<usize> {
        let (Event::Call {
            args, ..
        }
        | Event::IndirectCall {
            args, ..
        }) = event
        else {
            return HashSet::default();
        };
        let targets = self.call_targets(event);
        if guaranteed && targets.unknown {
            return HashSet::default();
        }
        let mut result = HashSet::default();
        for (index, p) in targets.ordered().into_iter().enumerate() {
            let mut current = HashSet::default();
            if let Some(s) = all.get(&p) {
                let effects = if guaranteed {
                    &s.guaranteed
                } else {
                    &s.may
                };
                for e in effects
                    .keys()
                    .filter(|e| e.kind == super::Kind::Free && e.target.steps.is_empty())
                {
                    let holds = e.guard.is_none_or(|(n, c)| {
                        args.iter()
                            .find(|a| a.formal == n)
                            .is_some_and(|a| a.constant.map_or(!guaranteed, |v| v == c))
                    });
                    if holds && let super::Root::Parameter(n) = e.target.root {
                        current.insert(n);
                    }
                }
            }
            if index == 0 || !guaranteed {
                result.extend(current);
            } else {
                result.retain(|n| current.contains(n));
            }
        }
        result
    }
}

impl Resolver<'_> {
    pub(super) fn indirect_return(
        &self,
        callee: jaic::ir::Val,
        args: &[super::LifetimeArgument],
        index: usize,
        all: &HashMap<ProcId, Summary>,
        visiting: &mut HashSet<jaic::ir::Val>,
        depth: usize,
    ) -> Option<super::Path> {
        let targets = self.targets.get(self.body.procedure, callee);
        let expected = self.indirect_types.get(&callee)?;
        if targets.procedures.iter().any(|p| {
            self.compiler.pr.procs[p.0 as usize]
                .sig
                .as_ref()
                .is_none_or(|s| s.ty != *expected)
        }) {
            return None;
        }
        if targets.unknown || targets.procedures.is_empty() {
            return None;
        }
        let mut result = None;
        for p in targets.ordered() {
            let (path, guard) = all.get(&p)?.returns.get(&index)?;
            if let Some((formal, expected)) = guard {
                let actual = args.iter().find(|a| a.formal == *formal)?;
                self.mode_guard(actual, *expected, all)?;
            }
            let path = self.substitute(path, args, all, visiting, depth + 1)?;
            if result.as_ref().is_some_and(|old| old != &path) {
                return None;
            }
            result = Some(path);
        }
        result
    }

    pub fn passthrough(
        &self,
        event: &Event,
        index: usize,
        all: &HashMap<ProcId, Summary>,
    ) -> Option<super::LifetimeArgument> {
        let (Event::Call {
            args, ..
        }
        | Event::IndirectCall {
            args, ..
        }) = event
        else {
            return None;
        };
        let targets = self.call_targets(event);
        if targets.unknown || targets.procedures.is_empty() {
            return None;
        }
        let mut selected = None;
        for p in targets.procedures {
            let s = all.get(&p)?;
            let (path, guard) = s.returns.get(&index)?;
            let super::Root::Parameter(formal) = path.root else {
                return None;
            };
            if !path.steps.is_empty()
                || guard.is_some()
                || s.may
                    .keys()
                    .any(|e| e.kind == super::Kind::Resize && e.target.root == path.root)
            {
                return None;
            }
            let arg = args.iter().find(|a| a.formal == formal)?;
            if !self.compiler.ty.types.is_pointer(arg.ty) {
                return None;
            }
            let value = super::value(arg)?;
            if selected.as_ref().is_some_and(|(v, _)| *v != value) {
                return None;
            }
            selected = Some((value, arg.clone()));
        }
        selected.map(|(_, a)| a)
    }
}
