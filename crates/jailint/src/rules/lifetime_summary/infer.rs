//! Return provenance, read effects and invalidation inference for one selected body.
use super::{
    Event, HashMap, HashSet, ProcId, Resolver, Span, Summary, chain, guaranteed, insert,
    returning_blocks,
};

impl Resolver<'_> {
    pub(super) fn infer(&mut self, prior: &Summary, all: &HashMap<ProcId, Summary>) -> Summary {
        self.refresh_context(all);
        let body = self.body;
        let live = returning_blocks(body, all, self.targets);
        let mut next = prior.clone();
        next.changes_allocator = self.context.changes_allocator;
        next.allocation_effects.clear();
        next.observable = self.observable_calls.infer(all);
        next.ordering = self.ordering_base.or_else(|| self.aggregate_order(all));
        let swap_facts = body.blocks.iter().flat_map(|b| &b.events).any(|e| matches!(e, Event::Call { callee, .. } if self.compiler.proc_has_note(*callee, "SwapsArguments"))).then(|| crate::rules::lifetime_movement::facts::Facts::new(self.compiler, body));
        let mut effects = vec![Vec::new(); body.blocks.len()];
        for (b, block) in body
            .blocks
            .iter()
            .enumerate()
            .filter(|(i, _)| live.contains(*i))
        {
            for (ordinal, event) in block.events.iter().enumerate() {
                self.infer_allocations(event, (b, ordinal), all, &mut next);
                for (effect, span, definite) in self.temporary_writes(event, all) {
                    if effect.exported() {
                        insert(&mut next.may, effect.clone(), vec![span]);
                    }
                    if definite {
                        effects[b].push((effect, vec![span]));
                    }
                }
                if let Some(effect) = swap_facts.as_ref().and_then(|f| {
                    self.swap_effect(
                        event,
                        all,
                        f,
                        crate::rules::lifetime_movement::facts::Site {
                            block: b,
                            event: ordinal,
                        },
                    )
                }) {
                    let span = match event {
                        Event::Call {
                            span, ..
                        } => *span,
                        _ => Span::NONE,
                    };
                    insert(&mut next.may, effect.clone(), vec![span]);
                    effects[b].push((effect, vec![span]));
                }
                for effect in self.unknown_effects(event, all) {
                    if effect.exported() {
                        insert(
                            &mut next.may,
                            effect,
                            vec![match event {
                                Event::Call {
                                    span, ..
                                }
                                | Event::IndirectCall {
                                    span, ..
                                } => *span,
                                _ => Span::NONE,
                            }],
                        );
                    }
                }
                if let Some(effect) = self.count_effect(event, all) {
                    let span = match event {
                        Event::Store {
                            span, ..
                        } => *span,
                        _ => Span::NONE,
                    };
                    insert(&mut next.may, effect.clone(), vec![span]);
                    effects[b].push((effect, vec![span]));
                }
                match event {
                    Event::Call {
                        span,
                        expands: false,
                        ..
                    }
                    | Event::IndirectCall {
                        span,
                        allocator: false,
                        ..
                    } => {
                        for (effect, witness) in self.call_effects(event, all, false) {
                            if effect.exported() {
                                insert(&mut next.may, effect, chain(*span, &witness));
                            }
                        }
                        for (effect, witness) in self.call_effects(event, all, true) {
                            effects[b].push((effect, chain(*span, &witness)));
                        }
                    }
                    Event::IndirectCall {
                        allocator: true,
                        args,
                        span,
                        ..
                    } => {
                        for effect in self.allocator(args, all) {
                            if effect.exported() {
                                insert(&mut next.may, effect.clone(), vec![*span]);
                            }
                            effects[b].push((effect, vec![*span]));
                        }
                    }
                    _ => {}
                }
            }
        }
        let keys: HashSet<_> = effects
            .iter()
            .flatten()
            .map(|(effect, _)| effect.clone())
            .collect();
        for effect in keys {
            if !effect.exported() {
                continue;
            }
            if let Some(witness) = guaranteed(body, &effects, &effect)
                .or_else(|| self.nullable_guarantee(&effects, &effect, all))
            {
                insert(&mut next.guaranteed, effect, witness);
            }
        }
        self.infer_return_origins(&mut next, all, &live);
        self.infer_provider_allocation(&mut next);
        self.infer_may_returns(&mut next, all, &live);
        next
    }
}
