//! Return provenance, read effects and invalidation inference for one selected body.
use super::{
    Event, HashMap, HashSet, ProcId, Resolver, Span, Summary, chain, guaranteed, insert,
    returning_blocks,
};

impl Resolver<'_> {
    pub(super) fn infer(
        &mut self,
        prior: &Summary,
        all: &HashMap<ProcId, Summary>,
        frame_requested: bool,
        returned_requested: Option<&super::value_writes::ContextReads>,
    ) -> Summary {
        self.refresh_context(all);
        let body = self.body;
        let live = returning_blocks(body, all, self.targets);
        let mut next = prior.clone();
        next.frame = None;
        next.bit_return = None;
        next.no_growth = None;

        let mut borrow = super::no_growth::Builder::new(self, prior.recursive);
        let mut native = prior
            .trusted
            .is_none()
            .then(|| super::native_returns::Builder::new(self, all, prior.recursive))
            .flatten();
        next.native_allocation = None;
        let bits = prior
            .trusted
            .is_none()
            .then(|| self.bit_return_plan(prior.recursive))
            .flatten();
        let mut frame = bits.as_ref().map_or_else(
            || super::frames::Builder::new(self, frame_requested, prior.recursive),
            |bits| bits.observer(self),
        );
        next.locks = self.infer_locks(all);
        self.infer_value_writes(all, &live, &mut next.value_writes);
        next.changes_allocator = self.context.changes_allocator;
        next.allocation_effects.clear();
        next.observable = self.observable_calls.infer(all);
        next.ordering = self.ordering_base.or_else(|| self.aggregate_order(all));
        next.rounded_address = self.rounded_address_base;
        let swap_facts = body.blocks.iter().flat_map(|b| &b.events).any(|e| matches!(e, Event::Call { callee, .. } if super::swap::swaps_arguments(self.compiler, *callee))).then(|| crate::rules::lifetime_movement::facts::Facts::new(self.compiler, body));
        let mut effects = vec![Vec::new(); body.blocks.len()];
        for (b, block) in body.blocks.iter().enumerate() {
            if !live.contains(b) && !frame.active() {
                continue;
            }
            for (ordinal, event) in block.events.iter().enumerate() {
                if let Some(native) = &mut native {
                    native.observe(
                        self,
                        crate::rules::lifetime_movement::facts::Site {
                            block: b,
                            event: ordinal,
                        },
                        event,
                        all,
                    );
                }
                frame.observe(self, b, ordinal, event, all);
                if let Some(borrow) = &mut borrow {
                    borrow.observe(
                        self,
                        crate::rules::lifetime_movement::facts::Site {
                            block: b,
                            event: ordinal,
                        },
                        event,
                        all,
                    );
                }
                if !live.contains(b) {
                    continue;
                }
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
                        // Unknown origins block a must release, but known may
                        // alternatives must not disappear with that exact proof.
                        for effect in self.allocator_possible(args, all) {
                            if effect.exported() {
                                insert(&mut next.may, effect, vec![*span]);
                            }
                        }
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
        let checked = frame.finish(self, &live, all);
        if let Some(bits) = bits {
            let relation = checked
                .map(|frame| bits.finish(frame))
                .filter(|bits| bits.current(self.compiler, all));
            if jacket_config::env::LIFETIME_STATS.is_set()
                && let Some(bits) = &relation
            {
                eprintln!(
                    "lifetime-bit-return proc={} nodes={} captures={} work_with_reservation={} payload_bytes={}",
                    body.procedure.0,
                    bits.nodes.len(),
                    bits.captures.len(),
                    frame.spent(),
                    bits.payload_bytes()
                );
            }
            next.bit_return = relation.map(std::rc::Rc::new);
        } else {
            next.frame = checked
                .or_else(|| {
                    (frame_requested && !prior.recursive)
                        .then(|| super::frames::read_only::infer(self, all, &frame))
                        .flatten()
                })
                .map(std::rc::Rc::new);
        }
        next.native_allocation = native
            .and_then(|native| native.finish(self, all))
            .map(std::rc::Rc::new);
        next.no_growth = borrow
            .and_then(|borrow| borrow.finish(self, all))
            .map(std::rc::Rc::new);
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
        self.infer_pointer_returns(&mut next, all, &live);
        self.infer_return_origins(&mut next, all, &live);
        next.returned_slots = returned_requested
            .and_then(|requests| super::value_writes::ReturnedSlots::capture(self, all, requests));
        self.infer_provider_allocation(&mut next);
        self.infer_may_returns(&mut next, all, &live);
        next
    }
}
