//! Check a store against the destination owner's lifetime.
use super::{Builder, Cell, Cells, Event, Op, Places, Root, inside, value_cell};
use crate::rules::lifetime_summary::Resolver;
use jaic::ir::Val;

impl Builder<'_> {
    pub(super) fn store_escapes(
        &self,
        event: &Event,
        places: &mut Places,
        cells: &mut Cells,
        resolver: &mut Resolver<'_>,
    ) -> Vec<Op> {
        let Event::Store {
            place,
            value,
            ty,
            span,
            ..
        } = event
        else {
            return Vec::new();
        };
        let mut ops = Vec::new();
        let p = places.place(*place, *ty);
        if places.unresolved(p)
            && let Some(local) = resolver.local_destination(*place, *ty, &self.summaries.procs)
        {
            let local = places.import(&resolver.places, local);
            if !self.long_lived(places, local)
                && self.local_store_survives(places, local, resolver, *value)
            {
                return ops;
            }
        }
        if self.long_lived(places, p) {
            let pointer = places.unresolved(p);
            let parameter_target = self.defers_temporary(event, resolver);
            let temporary_target = pointer
                && (resolver
                    .value(*place, &self.summaries.procs)
                    .is_some_and(|p| p.root == Root::Temporary)
                    || resolver
                        .value(places.paths[p].0.0, &self.summaries.procs)
                        .is_some_and(|p| p.root == Root::Temporary));
            let mut escaping = vec![(
                value_cell(self.cx, places, cells, *value, *ty),
                cells.id(Cell::Place(p)),
            )];
            // Pointer outputs are replayed at their callers. Concrete value sinks
            // need separate observations for each stored aggregate field.
            if !pointer && self.cx.compiler.ir_ty(*ty).is_none() {
                let source = places.place(*value, *ty);
                escaping.extend(
                    places
                        .copied_fields(p, source)
                        .into_iter()
                        .map(|(dst, src)| (cells.id(Cell::Place(src)), cells.id(Cell::Place(dst)))),
                );
            }
            let mut location = (pointer && self.cx.compiler.ty.types.is_pointer(*ty))
                .then(|| self.capture_location(resolver, *place, *ty))
                .flatten();
            for (cell, target) in escaping {
                ops.push(Op::Escape {
                    location: location.take(),
                    cell,
                    span: self.store_spans.get(span).copied().unwrap_or(*span),
                    pointer,
                    target,
                    store: true,
                    parameter_target,
                    temporary_target,
                });
            }
        }
        ops
    }


    fn local_store_survives(
        &self,
        places: &Places,
        target: usize,
        resolver: &mut Resolver<'_>,
        value: Val,
    ) -> bool {
        let Some(&target_scope) = self.locals.get(&places.paths[target].0.0) else {
            return true;
        };
        let Some(path) = resolver.value(value, &self.summaries.procs) else {
            return true;
        };
        let Root::Local(source) = path.root else {
            return true;
        };
        let Some(&source_scope) = self.locals.get(&resolver.places.paths[source].0.0) else {
            return true;
        };
        inside(self.cx, target_scope, source_scope)
    }
}
