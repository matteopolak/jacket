//! Return provenance, read effects and invalidation inference for one selected body.
use super::{
    Event, HashMap, HashSet, ProcId, Resolver, Root, Selector, Span, Summary, TypeKind, chain,
    guaranteed, insert, returning_blocks,
};

impl Resolver<'_> {
    pub(super) fn infer(&mut self, prior: &Summary, all: &HashMap<ProcId, Summary>) -> Summary {
        let body = self.body;
        let live = returning_blocks(body, all, self.targets);
        let mut next = prior.clone();
        let swap_facts = body.blocks.iter().flat_map(|b| &b.events).any(|e| matches!(e, Event::Call { callee, .. } if self.compiler.proc_has_note(*callee, "SwapsArguments"))).then(|| crate::rules::lifetime_movement::facts::Facts::new(self.compiler, body));
        let mut effects = vec![Vec::new(); body.blocks.len()];
        for (b, block) in body
            .blocks
            .iter()
            .enumerate()
            .filter(|(i, _)| live.contains(i))
        {
            for (ordinal, event) in block.events.iter().enumerate() {
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
            if let Some(witness) = guaranteed(body, &effects, &effect) {
                insert(&mut next.guaranteed, effect, witness);
            }
        }
        let mut returns = HashMap::default();
        let mut elements = HashMap::default();
        for event in body
            .blocks
            .iter()
            .enumerate()
            .filter(|(i, _)| live.contains(i))
            .flat_map(|(_, b)| &b.events)
        {
            if let Event::Return {
                value,
                index,
                ty,
                constant,
                ..
            } = event
            {
                if *constant == Some(0)
                    && (self.compiler.ty.types.is_pointer(*ty)
                        || matches!(self.compiler.ty.types.kind(*ty), TypeKind::Proc(_)))
                {
                    continue;
                }
                let element = self.element(*value, all).filter(|(p, s)| {
                    matches!(p.root, Root::Parameter(_))
                        && !matches!(s, Selector::Local(_) | Selector::Unknown)
                });
                elements
                    .entry(*index)
                    .and_modify(|old| {
                        if old != &element {
                            *old = None;
                        }
                    })
                    .or_insert(element);
                let path = self
                    .trace(*value, all, &mut HashSet::default(), 0)
                    .filter(|p| self.result_carries_storage(*ty, p))
                    .filter(|p| {
                        matches!(
                            p.root,
                            Root::Parameter(_)
                                | Root::Heap
                                | Root::Temporary
                                | Root::Watermark
                                | Root::Until(_)
                        )
                    })
                    .zip(self.return_guard(*value, all, &mut HashSet::default()));
                returns
                    .entry(*index)
                    .and_modify(|previous| {
                        if previous != &path {
                            *previous = None;
                        }
                    })
                    .or_insert(path);
            }
        }
        for (index, element) in elements {
            if let Some(e) = element {
                next.elements.insert(index, e);
            }
        }
        for (index, path) in returns {
            if let Some(path) = path {
                if index == 0
                    && self
                        .compiler
                        .proc_has_note(self.body.procedure, "ValidUntilNextCall")
                    || ["TempAllocates", "Allocates"].iter().any(|note| {
                        super::lifetime_movement::result_indices(
                            self.compiler,
                            self.body.procedure,
                            note,
                        )
                        .contains(&index)
                    })
                {
                    continue;
                }
                next.returns.insert(index, path);
            }
        }
        next
    }
}
