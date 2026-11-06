//! Direct checked copy/call composition, with explicit continuation categories.
use super::*;

pub(super) enum Next {
    Continue,
    FinishState,
    Forked,
}

impl Solver<'_> {
    pub(super) fn call(
        &mut self,
        b: &LifetimeBody,
        e: &Event,
        anchor: EventSite,
        site: &Site,
        state: &mut State,
        next: &mut Vec<State>,
    ) -> Option<Next> {
        let Event::Call {
            callee,
            memory,
            args,
            results,
            ..
        } = e
        else {
            return Some(Next::Continue);
        };
        if let Some((destination, source, count, requires_disjoint)) =
            selected_copy(memory.as_ref())
        {
            let binding = Bindings {
                c: self.c,
                args,
                state,
                site: anchor,
                layout: self.layout,
            };
            let destination = binding.argument(destination);
            let source = binding.argument(source);
            let count = binding.argument(count);
            if let (Value::Pointer(destination), Value::Pointer(source), Value::Scalar(count)) =
                (destination, source, count)
            {
                if !super::ranges::record(
                    state,
                    destination,
                    source,
                    count,
                    site.clone(),
                    requires_disjoint,
                    b.procedure,
                ) {
                    return None;
                }
            } else {
                state.opaque(args);
            }
            return Some(Next::FinishState);
        }
        let empty_write = memory
            .as_ref()
            .and_then(jaic::ide::lifetimes::LifetimeMemoryEffects::bounded_write)
            .and_then(|write| {
                args.iter()
                    .find(|a| a.formal == write.count_formal as usize)
            })
            .is_some_and(|a| a.constant == Some(0));
        if empty_write {
            self.results(state, results, site);
            return Some(Next::FinishState);
        }
        let relation = if results.iter().any(|(_, _, ty)| *ty == TypeId::STRING) {
            self.summary(*callee)
        } else {
            None
        };
        if let Some(relation) = relation {
            for exit in &relation.exits {
                if jaic::interrupt::should_stop() {
                    return None;
                }
                let binding = Bindings {
                    c: self.c,
                    args,
                    state,
                    site: anchor,
                    layout: self.layout,
                };
                let returns: Vec<_> = exit
                    .returns
                    .iter()
                    .map(|(i, d)| (*i, binding.output(d)))
                    .collect();
                let copies: Vec<_> = exit.copies.iter().filter_map(|f| binding.fact(f)).collect();
                let mut fork = state.clone();
                fork.opaque(args);
                self.results(&mut fork, results, site);
                for (i, descriptor) in returns {
                    if let Some((_, v, ty)) = results.iter().find(|(index, _, _)| *index == i)
                        && *ty == TypeId::STRING
                    {
                        fork.descriptor_at(*v, &descriptor, self.layout);
                    }
                }
                for mut fact in copies {
                    let Some(len) = super::ranges::length(fact.length) else {
                        continue;
                    };
                    fact.separations.extend(super::ranges::frame_requirements(
                        &fork,
                        b.procedure,
                        super::ranges::copy_range(&fact.site, len),
                        None,
                    ));
                    fork.copies.push(fact);
                }
                if !fork.bounded() {
                    return None;
                }
                next.push(fork);
            }
            return Some(Next::Forked);
        }
        state.opaque(args);
        self.results(state, results, site);
        Some(Next::Continue)
    }
}
