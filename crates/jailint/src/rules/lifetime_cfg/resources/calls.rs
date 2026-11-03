//! Selected direct and finite-indirect resource contracts.
use super::{
    Builder, Call, Cell, Cells, EffectKind, LifetimeOperand, Op, Origin, Places, Root, TypeKind,
    operand_cell, value_cell,
};
use crate::rules::lifetime_summary::Resolver;

impl Builder<'_> {
    pub(super) fn call(
        &self,
        call: Call<'_>,
        places: &mut Places,
        cells: &mut Cells,
        resolver: &mut Resolver<'_>,
    ) -> Vec<Op> {
        let Call {
            callee,
            event,
            args,
            span,
            ..
        } = call;
        let cx = self.cx;
        let summaries = self.summaries;
        let mut ops = Vec::new();

        let c = cx.compiler;
        let direct = callee.0 != u32::MAX;
        let noted = |name: &str| direct && c.proc_has_note(*callee, name);
        let strict = cx.lifetime_mode == jacket_config::LifetimeMode::Strict;
        let released_formal = if direct {
            crate::rules::lifetime_movement::released_formal(c, *callee)
        } else {
            None
        };
        let frees = released_formal.is_some();
        if let Some(formal) = released_formal
            && let Some(arg) = args.iter().find(|a| a.formal == formal)
            && let Some(cell) = operand_cell(cx, places, cells, arg)
        {
            let contents = c.ty.types.pointee(arg.ty).is_some_and(|ty| {
                matches!(
                    c.ty.types.kind(c.ty.types.repr_struct(ty)),
                    TypeKind::Struct(_)
                )
            });
            ops.push(Op::Free {
                cell,
                span: *span,
                callee: *callee,
                contents,
            });
        }
        if !frees && let Some(summary) = summaries.procs.get(callee) {
            for effect in summary.guaranteed.keys().filter(|e| {
                e.kind == EffectKind::Free
                    && e.guard.is_none_or(|(formal, constant)| {
                        args.iter()
                            .any(|a| a.formal == formal && a.constant == Some(constant))
                    })
                    && e.target.steps.is_empty()
            }) {
                if let Root::Parameter(formal) = effect.target.root
                    && let Some(arg) = args.iter().find(|a| a.formal == formal)
                    && let Some(cell) = operand_cell(cx, places, cells, arg)
                {
                    ops.push(Op::Free {
                        cell,
                        span: *span,
                        callee: *callee,
                        contents: false,
                    });
                }
            }
        }
        let guaranteed = resolver.call_effects(event, &summaries.procs, true);
        if !direct {
            for formal in resolver.released_formals(event, &summaries.procs, true) {
                if let Some(arg) = args.iter().find(|a| a.formal == formal)
                    && let Some(cell) = operand_cell(cx, places, cells, arg)
                {
                    ops.push(Op::Free {
                        cell,
                        span: *span,
                        callee: *callee,
                        contents: false,
                    });
                }
            }
        }
        if strict {
            let definite = resolver.released_formals(event, &summaries.procs, true);
            for formal in resolver
                .released_formals(event, &summaries.procs, false)
                .difference(&definite)
            {
                if let Some(arg) = args.iter().find(|a| a.formal == *formal)
                    && let Some(cell) = operand_cell(cx, places, cells, arg)
                {
                    ops.push(Op::MayFree {
                        cell,
                        span: *span,
                    });
                }
            }
            let mut may = resolver.call_effects(event, &summaries.procs, false);
            may.extend(
                resolver
                    .unknown_effects(event, &summaries.procs)
                    .into_iter()
                    .map(|e| (e, Vec::new())),
            );
            for (effect, _) in may {
                if guaranteed.iter().any(|(e, _)| e == &effect) {
                    continue;
                }
                if let (EffectKind::Advance, Root::Until(p)) = (effect.kind, &effect.target.root) {
                    ops.push(Op::MayAdvance(*p, *span));
                }
                if effect.kind == EffectKind::ResetTemporary {
                    ops.push(Op::MayReset(*span));
                }
                if matches!(
                    effect.kind,
                    EffectKind::Free | EffectKind::Unknown | EffectKind::RestoreTemporary
                ) && let Some(p) = resolver.materialize_storage(&effect.target)
                {
                    let region = places.import(&resolver.places, p);
                    if effect.kind == EffectKind::RestoreTemporary {
                        ops.push(Op::MayRestore(cells.id(Cell::Place(region)), *span));
                    } else {
                        let mut ids: Vec<_> = places
                            .descendants(region)
                            .into_iter()
                            .map(|p| cells.id(Cell::Place(p)))
                            .collect();
                        ids.push(cells.id(Cell::Place(region)));
                        ops.push(Op::MayRegion {
                            region,
                            cells: ids,
                            span: *span,
                        });
                    }
                }
            }
            // An unknown call may release a directly passed allocation, even when its
            // provenance is a fresh heap root with no descriptor place to materialize.
            if !resolver.unknown_effects(event, &summaries.procs).is_empty() {
                for arg in args {
                    if let Some(cell) = operand_cell(cx, places, cells, arg)
                        && c.ty.types.is_pointer(arg.ty)
                    {
                        ops.push(Op::MayFree {
                            cell,
                            span: *span,
                        });
                    }
                }
            }
        }
        for (effect, _) in guaranteed {
            if let (EffectKind::Advance, Root::Until(p)) = (effect.kind, &effect.target.root) {
                ops.push(Op::Advance(*p, *span));
            }
            if effect.kind == EffectKind::ResetTemporary && effect.guard.is_none() {
                ops.push(Op::Reset(*span));
            }
            if effect.kind == EffectKind::RestoreTemporary
                && effect.guard.is_none()
                && let Some(p) = resolver.materialize_storage(&effect.target)
            {
                let p = places.import(&resolver.places, p);
                ops.push(Op::Restore(cells.id(Cell::Place(p)), *span));
            }
            if effect.kind == EffectKind::Free
                && effect.guard.is_none()
                && let Some(p) = resolver.materialize_storage(&effect.target)
            {
                let region = places.import(&resolver.places, p);
                ops.push(Op::FreeRegion {
                    region,
                    cell: cells.id(Cell::Place(region)),
                    span: *span,
                });
            }
        }
        ops.extend(self.return_origins(call, places, cells, resolver));
        if noted("ResetsTemporaryStorage") {
            ops.push(Op::Reset(*span));
        }
        if noted("RestoresTemporaryStorage")
            && let Some(arg) = args.iter().find(|a| a.formal == 0)
            && let Some(cell) = operand_cell(cx, places, cells, arg)
        {
            ops.push(Op::Restore(cell, *span));
        }
        if noted("AddsToArray") && args.len() >= 2 {
            let array = args.iter().find(|a| a.formal == 0);
            let value = args.iter().find(|a| a.formal == 1);
            if let (Some(array), Some(value)) = (array, value)
                && let LifetimeOperand::Value(v) | LifetimeOperand::Place(v) = array.operand
            {
                let ty = c.ty.types.pointee(array.ty).unwrap_or(array.ty);
                let p = if let Some(p) = resolver.object(array) {
                    places.import(&resolver.places, p)
                } else {
                    places.place(v, ty)
                };
                if self.long_lived(places, p)
                    && let Some(cell) = operand_cell(cx, places, cells, value)
                {
                    ops.push(Op::Escape {
                        cell,
                        span: value.span,
                        pointer: places.unresolved(p),
                        target: cells.id(Cell::Place(p)),
                        store: false,
                        parameter_target: self.defers_temporary(event, resolver),
                        temporary_target: resolver
                            .value(places.paths[p].0.0, &summaries.procs)
                            .is_some_and(|p| p.root == Root::Temporary),
                    });
                }
            }
        }

        ops.extend(self.temporary_stores(event, places, cells, resolver));
        ops
    }
}

impl Builder<'_> {
    pub(super) fn allocation(
        &self,
        event: &jaic::ide::lifetimes::LifetimeEvent,
        places: &mut Places,
        cells: &mut Cells,
        resolver: &mut Resolver<'_>,
    ) -> Vec<Op> {
        let jaic::ide::lifetimes::LifetimeEvent::IndirectCall {
            args,
            results,
            span,
            ..
        } = event
        else {
            return Vec::new();
        };
        let cx = self.cx;
        let mut ops = Vec::new();
        if args.first().and_then(|a| a.constant) == Some(0) {
            for &(_, value, ty) in results {
                let origin = match resolver.value(value, &self.summaries.procs).map(|p| p.root) {
                    Some(Root::Temporary) => Some(Origin::Temporary),
                    Some(Root::Until(p)) => Some(Origin::Until(p)),
                    Some(Root::Heap) | None => Some(Origin::Heap),
                    _ => None,
                };
                if let Some(origin) = origin {
                    ops.push(Op::Seed {
                        possible: false,
                        cell: value_cell(cx, places, cells, value, ty),
                        origin,
                        span: *span,
                    });
                }
            }
        } else if args.first().and_then(|a| a.constant) == Some(2)
            && let Some(arg) = args.iter().find(|a| a.formal == 3)
            && let Some(cell) = operand_cell(cx, places, cells, arg)
        {
            ops.push(Op::Free {
                cell,
                span: *span,
                callee: jaic::sema::ProcId(u32::MAX),
                contents: false,
            });
        }
        ops
    }
}

impl Builder<'_> {
    fn return_origins(
        &self,
        call: Call<'_>,
        places: &mut Places,
        cells: &mut Cells,
        resolver: &mut Resolver<'_>,
    ) -> Vec<Op> {
        let Call {
            callee,
            results,
            span,
            ..
        } = call;
        let cx = self.cx;
        let summaries = self.summaries;
        let mut ops = Vec::new();
        let noted = |name| callee.0 != u32::MAX && cx.compiler.proc_has_note(*callee, name);
        for &(index, value, ty) in results {
            if summaries
                .procs
                .get(callee)
                .is_some_and(|s| !s.return_fields.is_empty() || !s.may_return_fields.is_empty())
            {
                for projections in resolver.storage_fields(ty, 0) {
                    let path = resolver.field_origin(value, ty, &projections, &summaries.procs, 0);
                    let definite_temporary =
                        path.as_ref().is_some_and(|p| p.root == Root::Temporary);
                    let maybe =
                        resolver.field_may_temporary(value, ty, &projections, &summaries.procs, 0);
                    let origin = if maybe {
                        Origin::Temporary
                    } else {
                        match path.map(|p| p.root) {
                            Some(Root::Heap | Root::CallerAllocator) => Origin::Heap,
                            _ => continue,
                        }
                    };
                    let mut field = places.place(value, ty);
                    for projection in projections {
                        field = places.project(field, projection);
                    }
                    ops.push(Op::Seed {
                        possible: maybe && !definite_temporary,
                        cell: cells.id(Cell::Place(field)),
                        origin,
                        span: *span,
                    });
                }
            }
            let origin = resolver
                .value(value, &summaries.procs)
                .and_then(|path| match path.root {
                    Root::Heap | Root::CallerAllocator => Some(Origin::Heap),
                    Root::Temporary => Some(Origin::Temporary),
                    Root::Until(p) => Some(Origin::Until(p)),
                    _ => None,
                });
            let maybe = summaries
                .procs
                .get(callee)
                .is_some_and(|s| s.may_returns.contains(&index))
                && !matches!(origin, Some(Origin::Until(_)));
            let definite_temporary = matches!(origin, Some(Origin::Temporary));
            let origin = if maybe {
                Some(Origin::Temporary)
            } else {
                origin
            };
            if let Some(origin) = origin {
                ops.push(Op::Seed {
                    possible: maybe && !definite_temporary,
                    cell: value_cell(cx, places, cells, value, ty),
                    origin,
                    span: *span,
                });
            }
            if noted("TemporaryStorageMark")
                || resolver
                    .value(value, &summaries.procs)
                    .is_some_and(|p| p.root == Root::Watermark)
            {
                ops.push(Op::Mark(value_cell(cx, places, cells, value, ty)));
            }
        }
        ops
    }
}
