//! Replay temporary writes at actual destinations, where ownership can be established.
use super::{Builder, Cell, Cells, Event, Op, Origin, Places, Resolver, Root};

impl Builder<'_> {
    pub(super) fn defers_temporary(&self, event: &Event, resolver: &mut Resolver<'_>) -> bool {
        // The caller establishes whether the output destination outlives temporary storage.
        // May and guaranteed writes remain separate when replayed there.
        !resolver
            .temporary_writes(event, &self.summaries.procs)
            .is_empty()
    }

    pub(super) fn temporary_stores(
        &self,
        event: &Event,
        places: &mut Places,
        cells: &mut Cells,
        resolver: &mut Resolver<'_>,
    ) -> Vec<Op> {
        let strict = self.cx.lifetime_mode == jacket_config::LifetimeMode::Strict;
        let mut ops = Vec::new();
        let span = match event {
            Event::Call {
                span, ..
            }
            | Event::IndirectCall {
                span, ..
            } => *span,
            _ => return ops,
        };
        let definite = resolver.temporary_destinations(event, &self.summaries.procs, true);
        for p in resolver.temporary_destinations(event, &self.summaries.procs, !strict) {
            let possible = !definite.contains(&p);
            let parameter_target = resolver
                .location(
                    p,
                    &self.summaries.procs,
                    &mut jaic::fxhash::HashSet::default(),
                    0,
                )
                .is_some_and(|path| matches!(path.root, Root::Parameter(_)));
            let temporary_target = resolver
                .location(
                    p,
                    &self.summaries.procs,
                    &mut jaic::fxhash::HashSet::default(),
                    0,
                )
                .is_some_and(|path| path.root == Root::Temporary);
            let p = places.import(&resolver.places, p);
            let cell = cells.id(Cell::Place(p));
            ops.push(Op::Seed {
                possible,
                cell,
                origin: Origin::Temporary,
                span,
            });
            if self.long_lived(places, p) {
                ops.push(Op::Escape {
                    cell,
                    span,
                    pointer: places.unresolved(p),
                    target: cell,
                    store: false,
                    temporary_target,
                    parameter_target,
                });
            }
        }
        ops
    }
}
