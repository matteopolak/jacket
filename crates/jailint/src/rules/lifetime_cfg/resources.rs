//! Shared semantic-CFG analysis for released resources and escaping storage.
//! Each candidate follows only its copy closure; joins preserve uncertainty.
use super::{Block, Cell, Cells, DEAD, LIVE, Model, Op as CfgOp, UNKNOWN, build};
use crate::rules::lifetime_places::Places;
use crate::rules::lifetime_summary::{Kind as EffectKind, Resolver, Root, Summaries};
use crate::{Finding, syntax::Cx};
use jaic::fxhash::{HashMap, HashSet};
use jaic::ide::lifetimes::{
    LifetimeArgument, LifetimeBody, LifetimeEvent as Event, LifetimeOperand,
};
use jaic::ir::Val;
use jaic::sema::{ProcId, ScopeId, procs::BodyState};
use jaic::source::Span;
use jaic::types::{TypeId, TypeKind};
use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;

mod diagnostics;
use diagnostics::{Proof, emit};

#[derive(Clone, Copy, Debug)]
pub(super) enum Origin {
    Heap,
    Temporary,
    Stack(ScopeId),
    Statement(Span),
    Incoming,
}

#[derive(Clone)]
pub(super) enum Op {
    Seed {
        cell: usize,
        origin: Origin,
        span: Span,
    },
    Free {
        cell: usize,
        span: Span,
        callee: ProcId,
        contents: bool,
    },
    FreeRegion {
        region: usize,
        cell: usize,
        span: Span,
    },
    Return {
        cell: usize,
        span: Span,
    },
    Escape {
        cell: usize,
        span: Span,
        pointer: bool,
        target: usize,
        store: bool,
    },
    ScopeEnd(ScopeId, Span),
    StatementEnd(Span),
    Reset(Span),
    Mark(usize),
    Restore(usize, Span),
    Exit,
}

#[derive(Default)]
pub(crate) struct Reports {
    pub use_after_free: Vec<Finding>,
    pub double_free: Vec<Finding>,
    pub returned_local_address: Vec<Finding>,
    pub temp_storage_escape: Vec<Finding>,
    external_primary: usize,
}

pub(crate) fn reports(cx: &Cx) -> Rc<Reports> {
    if let Some(result) = cx.run.resource_reports.borrow().get(&cx.file) {
        return result.clone();
    }
    let result = Rc::new(analyze(cx));
    cx.run
        .resource_reports
        .borrow_mut()
        .insert(cx.file, result.clone());
    result
}

fn operand_cell(
    cx: &Cx,
    places: &mut Places,
    cells: &mut Cells,
    arg: &LifetimeArgument,
) -> Option<usize> {
    Some(match arg.operand {
        LifetimeOperand::Value(v) => value_cell(cx, places, cells, v, arg.ty),
        LifetimeOperand::Place(v) => cells.id(Cell::Place(places.place(v, arg.ty))),
        LifetimeOperand::Other => return None,
    })
}

fn value_cell(cx: &Cx, places: &mut Places, cells: &mut Cells, value: Val, ty: TypeId) -> usize {
    if cx.compiler.ir_ty(ty).is_none() {
        cells.id(Cell::Place(places.place(value, ty)))
    } else {
        cells.id(Cell::Value(value))
    }
}

struct Call<'a> {
    callee: &'a ProcId,
    args: &'a [LifetimeArgument],
    results: &'a [(usize, Val, TypeId)],
    span: &'a Span,
}

pub(super) struct Builder<'a> {
    cx: &'a Cx<'a>,
    summaries: &'a Summaries,
    locals: HashMap<Val, ScopeId>,
    parameters: HashSet<Val>,
    stack: HashSet<Val>,
    returns: HashSet<Val>,
    released_values: HashSet<Val>,
    value_spans: HashMap<(Val, TypeId), Span>,
    store_spans: HashMap<Span, Span>,
}

impl<'a> Builder<'a> {
    pub fn new(cx: &'a Cx<'a>, body: &LifetimeBody, summaries: &'a Summaries) -> Self {
        let mut locals = HashMap::default();
        let mut parameters = HashSet::default();
        let mut returns = HashSet::default();
        let mut released_values = HashSet::default();
        let mut value_spans = HashMap::default();
        let mut store_spans = HashMap::default();
        for e in body.blocks.iter().flat_map(|b| &b.events) {
            match e {
                Event::Local {
                    place,
                    scope,
                    ..
                } => {
                    locals.insert(*place, *scope);
                }
                Event::Parameter {
                    place, ..
                } => {
                    parameters.insert(*place);
                }
                Event::Expression {
                    operand: LifetimeOperand::Value(v),
                    ty,
                    span,
                    ..
                } => {
                    value_spans.entry((*v, *ty)).or_insert(*span);
                }
                Event::Return {
                    value,
                    ty,
                    ..
                } if cx.compiler.ir_ty(*ty).is_none() => {
                    returns.insert(*value);
                }
                Event::Call {
                    callee,
                    args,
                    ..
                } if cx.compiler.proc_has_note(*callee, "FreesArgument")
                    || cx.compiler.proc_has_note(*callee, "Closes") =>
                {
                    if let Some(a) = args.iter().find(|a| a.formal == 0)
                        && let LifetimeOperand::Value(v) = a.operand
                    {
                        released_values.insert(v);
                    }
                }
                _ => {}
            }
        }
        if let Some(syntax) = &body.syntax.body {
            jaic::ast::walk::walk(&syntax.stmts, &mut |node, _| {
                if let Some(stmt) = node.stmt()
                    && let jaic::ast::StmtKind::Assign {
                        lhs,
                        rhs,
                        ..
                    } = &stmt.kind
                    && lhs.len() == 1
                    && rhs.len() == 1
                {
                    store_spans.insert(stmt.span, rhs[0].span);
                }
                true
            });
        }
        Self {
            cx,
            summaries,
            locals,
            parameters,
            returns,
            released_values,
            value_spans,
            store_spans,
            stack: body.stack_places.iter().copied().collect(),
        }
    }

    pub fn resource_load(&self, value: Val) -> bool {
        self.released_values.contains(&value)
    }

    fn long_lived(&self, places: &Places, p: usize) -> bool {
        !self.returns.contains(&places.paths[p].0.0)
            && (places.unresolved(p) || !self.stack.contains(&places.paths[p].0.0))
    }

    pub fn operations(
        &self,
        event: &Event,
        places: &mut Places,
        cells: &mut Cells,
        resolver: &mut Resolver<'_>,
    ) -> Vec<Op> {
        let cx = self.cx;
        let mut ops = Vec::new();
        match event {
            Event::Parameter {
                incoming,
                ty,
                span,
                ..
            } if cx.compiler.ty.types.is_pointer(*ty) => {
                ops.push(Op::Seed {
                    cell: cells.id(Cell::Value(*incoming)),
                    origin: Origin::Incoming,
                    span: *span,
                });
            }
            Event::Address {
                place,
                ty,
                span,
            } => {
                let p = places.place(*place, *ty);
                if let Some(scope) = self.locals.get(&places.paths[p].0.0).copied() {
                    if !places.unresolved(p) {
                        ops.push(Op::Seed {
                            cell: cells.id(Cell::Value(*place)),
                            origin: Origin::Stack(scope),
                            span: *span,
                        });
                    }
                } else if self.parameters.contains(place) {
                    // The address is that of the callee's by-value parameter copy.
                    ops.push(Op::Seed {
                        cell: cells.id(Cell::Value(*place)),
                        origin: Origin::Stack(ScopeId(u32::MAX)),
                        span: *span,
                    });
                }
            }
            Event::Borrow {
                value,
                region,
                region_ty,
                span,
            } => {
                let p = places.place(*region, *region_ty);
                if let Some(scope) = self.locals.get(&places.paths[p].0.0).copied()
                    && matches!(
                        cx.compiler.ty.types.kind(*region_ty),
                        TypeKind::Array {
                            kind: jaic::types::ArrayKind::Fixed(_),
                            ..
                        }
                    )
                {
                    ops.push(Op::Seed {
                        cell: cells.id(Cell::Value(*value)),
                        origin: Origin::Stack(scope),
                        span: *span,
                    });
                }
            }
            Event::Field {
                base: LifetimeOperand::Place(base),
                result: LifetimeOperand::Value(value),
                descriptor: true,
                base_ty,
                result_ty,
                name,
                span,
                ..
            } if name.as_str() == "data"
                && matches!(
                    cx.compiler.ty.types.kind(*base_ty),
                    TypeKind::Array {
                        kind: jaic::types::ArrayKind::Fixed(_),
                        ..
                    }
                ) =>
            {
                let p = places.place(*base, *base_ty);
                if let Some(scope) = self.locals.get(&places.paths[p].0.0).copied() {
                    ops.push(Op::Seed {
                        cell: cells.id(Cell::Value(*value)),
                        origin: Origin::Stack(scope),
                        span: self
                            .value_spans
                            .get(&(*value, *result_ty))
                            .copied()
                            .unwrap_or(*span),
                    });
                }
            }
            Event::BoxAny {
                value,
                place,
                ty,
                temporary,
                span,
            } => {
                let p = places.place(*place, *ty);
                let origin = if *temporary {
                    Some(Origin::Statement(*span))
                } else {
                    self.locals
                        .get(&places.paths[p].0.0)
                        .copied()
                        .or_else(|| self.parameters.contains(place).then_some(ScopeId(u32::MAX)))
                        .filter(|_| !places.unresolved(p))
                        .map(Origin::Stack)
                };
                if let Some(origin) = origin {
                    ops.push(Op::Seed {
                        cell: cells.id(Cell::Place(places.place(*value, TypeId::ANY))),
                        origin,
                        span: *span,
                    });
                }
            }
            Event::ScopeEnd {
                scope,
                span,
            } => ops.push(Op::ScopeEnd(*scope, *span)),
            Event::StatementEnd {
                span,
            } => ops.push(Op::StatementEnd(*span)),
            Event::Return {
                value,
                ty,
                span,
                ..
            } => {
                ops.push(Op::Return {
                    cell: value_cell(cx, places, cells, *value, *ty),
                    span: *span,
                });
                if cx.compiler.ir_ty(*ty).is_none() {
                    let parent = places.place(*value, *ty);
                    for child in places.descendants(parent) {
                        ops.push(Op::Return {
                            cell: cells.id(Cell::Place(child)),
                            span: *span,
                        });
                    }
                }
            }
            Event::Store {
                place,
                value,
                ty,
                span,
            } => {
                let p = places.place(*place, *ty);
                if self.long_lived(places, p) {
                    ops.push(Op::Escape {
                        cell: value_cell(cx, places, cells, *value, *ty),
                        span: self.store_spans.get(span).copied().unwrap_or(*span),
                        pointer: places.unresolved(p),
                        target: cells.id(Cell::Place(p)),
                        store: true,
                    });
                }
            }
            Event::Call {
                callee,
                args,
                results,
                span,
                expands: false,
                ..
            } => {
                ops.extend(self.call(
                    Call {
                        callee,
                        args,
                        results,
                        span,
                    },
                    places,
                    cells,
                    resolver,
                ));
            }
            Event::IndirectCall {
                allocator: true,
                args,
                results,
                span,
                ..
            } => {
                if args.first().and_then(|a| a.constant) == Some(0) {
                    for &(_, value, ty) in results {
                        ops.push(Op::Seed {
                            cell: value_cell(cx, places, cells, value, ty),
                            origin: Origin::Heap,
                            span: *span,
                        });
                    }
                } else if args.first().and_then(|a| a.constant) == Some(2)
                    && let Some(arg) = args.iter().find(|a| a.formal == 3)
                    && let Some(cell) = operand_cell(cx, places, cells, arg)
                {
                    ops.push(Op::Free {
                        cell,
                        span: *span,
                        callee: ProcId(u32::MAX),
                        contents: false,
                    });
                }
            }
            _ => {}
        }
        ops
    }

    fn call(
        &self,
        call: Call<'_>,
        places: &mut Places,
        cells: &mut Cells,
        resolver: &mut Resolver<'_>,
    ) -> Vec<Op> {
        let Call {
            callee,
            args,
            results,
            span,
        } = call;
        let cx = self.cx;
        let summaries = self.summaries;
        let mut ops = Vec::new();

        let c = cx.compiler;
        let frees = c.proc_has_note(*callee, "FreesArgument") || c.proc_has_note(*callee, "Closes");
        if frees
            && let Some(arg) = args.iter().find(|a| a.formal == 0)
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
                e.kind == EffectKind::Free && e.guard.is_none() && e.target.steps.is_empty()
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
        for (effect, _) in resolver.effects(*callee, args, &summaries.procs, true) {
            if effect.kind == EffectKind::ResetTemporary && effect.guard.is_none() {
                ops.push(Op::Reset(*span));
            }
            if effect.kind == EffectKind::RestoreTemporary
                && effect.guard.is_none()
                && let Some(p) = resolver.materialize(&effect.target)
            {
                let p = places.import(&resolver.places, p);
                ops.push(Op::Restore(cells.id(Cell::Place(p)), *span));
            }
            if effect.kind == EffectKind::Free
                && effect.guard.is_none()
                && let Some(p) = resolver.materialize(&effect.target)
            {
                let region = places.import(&resolver.places, p);
                ops.push(Op::FreeRegion {
                    region,
                    cell: cells.id(Cell::Place(region)),
                    span: *span,
                });
            }
        }
        for &(_, value, ty) in results {
            let origin = resolver
                .value(value, &summaries.procs)
                .and_then(|path| match path.root {
                    Root::Heap => Some(Origin::Heap),
                    Root::Temporary => Some(Origin::Temporary),
                    _ => None,
                });
            if let Some(origin) = origin {
                ops.push(Op::Seed {
                    cell: value_cell(cx, places, cells, value, ty),
                    origin,
                    span: *span,
                });
            }
            if c.proc_has_note(*callee, "TemporaryStorageMark")
                || resolver
                    .value(value, &summaries.procs)
                    .is_some_and(|p| p.root == Root::Watermark)
            {
                ops.push(Op::Mark(value_cell(cx, places, cells, value, ty)));
            }
        }
        if c.proc_has_note(*callee, "ResetsTemporaryStorage") {
            ops.push(Op::Reset(*span));
        }
        if c.proc_has_note(*callee, "RestoresTemporaryStorage")
            && let Some(arg) = args.iter().find(|a| a.formal == 0)
            && let Some(cell) = operand_cell(cx, places, cells, arg)
        {
            ops.push(Op::Restore(cell, *span));
        }
        if c.proc_has_note(*callee, "AddsToArray") && args.len() >= 2 {
            let array = args.iter().find(|a| a.formal == 0);
            let value = args.iter().find(|a| a.formal == 1);
            if let (Some(array), Some(value)) = (array, value)
                && let LifetimeOperand::Value(v) | LifetimeOperand::Place(v) = array.operand
            {
                let ty = c.ty.types.pointee(array.ty).unwrap_or(array.ty);
                let p = places.place(v, ty);
                if self.long_lived(places, p)
                    && let Some(cell) = operand_cell(cx, places, cells, value)
                {
                    ops.push(Op::Escape {
                        cell,
                        span: value.span,
                        pointer: places.unresolved(p),
                        target: cells.id(Cell::Place(p)),
                        store: false,
                    });
                }
            }
        }

        ops
    }
}

fn analyze(cx: &Cx) -> Reports {
    let mut reports = Reports::default();
    let Some(facts) = cx.compiler.ed.facts() else {
        return reports;
    };
    let summaries = cx
        .run
        .lifetimes
        .get_or_init(|| Summaries::build(cx.compiler));
    let mut bodies: Vec<_> = facts.lifetime_bodies_in(cx.file).collect();
    bodies.sort_by_key(|b| b.procedure.0);
    let mut checked_bodies = 0;
    let mut cfgs = 0;
    let mut bytes = 0;
    let mut rejected = BTreeMap::<&str, usize>::new();
    let mut rejected_candidates = BTreeMap::<&str, usize>::new();
    for site in &cx.procs {
        for &id in cx.compiler.ide_procs_with_header(site.header.span).iter() {
            let p = &cx.compiler.pr.procs[id.0 as usize];
            if p.is_poly || p.is_macro {
                continue;
            }
            checked_bodies += 1;
            if !facts.lifetime_bodies.contains_key(&id) {
                *rejected
                    .entry(if p.body_state == BodyState::Done {
                        "missing-sidecar"
                    } else {
                        "failed-or-unchecked"
                    })
                    .or_default() += 1;
            }
        }
    }
    let mut solved = 0;
    for body in bodies {
        let proc = &cx.compiler.pr.procs[body.procedure.0 as usize];
        if proc.body_state != BodyState::Done || !std::sync::Arc::ptr_eq(&proc.lit, &body.syntax) {
            *rejected.entry("failed-or-obsolete").or_default() += 1;
            continue;
        }
        cfgs += 1;
        bytes += body.vector_bytes();
        let mut places = Places::new(cx.compiler, body);
        let mut cells = Cells::default();
        let mut chains = HashMap::default();
        let blocks = build(cx, body, &mut places, &mut cells, summaries, &mut chains);
        let mut forward = vec![Vec::new(); cells.ids.len()];
        for block in &blocks {
            match &block.op {
                CfgOp::Copy(pairs) => {
                    for &(target, source) in pairs {
                        forward[source].push(target);
                    }
                }
                CfgOp::PointerLoad(target, source, _, _) => forward[*source].push(*target),
                _ => {}
            }
        }
        let mut safe_addresses = HashSet::default();
        for e in body.blocks.iter().flat_map(|b| &b.events) {
            if let Event::Call {
                callee,
                args,
                ..
            } = e
                && let Some(summary) = summaries.procs.get(callee)
            {
                for arg in args {
                    if !summary.unknown_write.contains(&arg.formal)
                        && !summary.escapes.contains(&arg.formal)
                        && summary.guaranteed.keys().any(|e| {
                            e.kind == EffectKind::Free
                                && e.target.root == Root::Parameter(arg.formal)
                        })
                        && let LifetimeOperand::Value(value) = arg.operand
                    {
                        safe_addresses.insert((value, arg.span));
                    }
                }
            }
        }
        let exposed = body
            .blocks
            .iter()
            .flat_map(|b| &b.events)
            .filter_map(|e| match e {
                Event::Address {
                    place,
                    ty,
                    span,
                } if !safe_addresses.contains(&(*place, *span)) => Some(places.place(*place, *ty)),
                _ => None,
            })
            .collect();
        let model = Model {
            places,
            cells,
            exposed,
            blocks,
            forward,
            chains,
        };
        for (candidate, block) in model.blocks.iter().enumerate() {
            if matches!(
                block.op,
                CfgOp::Resource(Op::Seed { .. }) | CfgOp::PointerLoad(..) | CfgOp::Borrow(..)
            ) {
                if let Some((_, _, _, Some(region))) = self::candidate(block) {
                    let reason = if model.places.unresolved(region) {
                        Some("indirect-region")
                    } else if model
                        .exposed
                        .iter()
                        .any(|&p| model.places.contains(p, region))
                    {
                        Some("address-exposed")
                    } else {
                        None
                    };
                    if let Some(reason) = reason {
                        *rejected_candidates.entry(reason).or_default() += 1;
                        continue;
                    }
                }
                solved += usize::from(solve(cx, body, &model, candidate, &mut reports));
            }
        }
    }
    for findings in [
        &mut reports.use_after_free,
        &mut reports.double_free,
        &mut reports.returned_local_address,
        &mut reports.temp_storage_escape,
    ] {
        findings.sort_by_key(|f| f.start);
        findings.dedup_by_key(|f| f.start);
    }
    if jacket_config::env::LIFETIME_STATS.is_set() {
        let rank = |map: BTreeMap<&'static str, usize>| {
            let mut entries: Vec<_> = map.into_iter().collect();
            entries.sort_by_key(|(r, n)| (std::cmp::Reverse(*n), *r));
            entries
        };
        eprintln!(
            "lifetime-resource-stats file={} bodies={checked_bodies} cfgs={cfgs} candidates={solved} sidecar_bytes={bytes} rejected={:?} rejected_candidates={:?} use_after_free={} double_free={} stack_escapes={} temp_escapes={} external_primary={}",
            cx.compiler.src.sources.get(cx.file).path,
            rank(rejected),
            rank(rejected_candidates),
            reports.use_after_free.len(),
            reports.double_free.len(),
            reports.returned_local_address.len(),
            reports.temp_storage_escape.len(),
            reports.external_primary
        );
    }
    reports
}

fn inside(cx: &Cx, child: ScopeId, parent: ScopeId) -> bool {
    let mut current = Some(child);
    while let Some(scope) = current {
        if scope == parent {
            return true;
        }
        current = cx
            .compiler
            .ns
            .scopes
            .get(scope.0 as usize)
            .and_then(|s| s.parent);
    }
    false
}

fn contains(outer: Span, inner: Span) -> bool {
    outer.file == inner.file && outer.start <= inner.start && outer.end >= inner.end
}

fn candidate(block: &Block) -> Option<(usize, Origin, Span, Option<usize>)> {
    match block.op {
        CfgOp::Resource(Op::Seed {
            cell,
            origin,
            span,
        }) => Some((cell, origin, span, None)),
        CfgOp::PointerLoad(cell, _, region, span) => {
            Some((cell, Origin::Incoming, span, Some(region)))
        }
        CfgOp::Borrow(cell, region, span) => Some((cell, Origin::Incoming, span, Some(region))),
        _ => None,
    }
}

fn solve(cx: &Cx, body: &LifetimeBody, model: &Model, seed: usize, out: &mut Reports) -> bool {
    let Some((first, origin_kind, origin, region)) = candidate(&model.blocks[seed]) else {
        return false;
    };
    let mut relevant = HashMap::default();
    let source = match model.blocks[seed].op {
        CfgOp::PointerLoad(_, source, _, _) => Some(source),
        _ => None,
    };
    let mut queue = VecDeque::from([first]);
    queue.extend(source);
    if matches!(origin_kind, Origin::Temporary) {
        queue.extend(model.blocks.iter().filter_map(|b| match b.op {
            CfgOp::Resource(Op::Mark(cell)) => Some(cell),
            _ => None,
        }));
    }
    while let Some(cell) = queue.pop_front() {
        if relevant.contains_key(&cell) {
            continue;
        }
        relevant.insert(cell, relevant.len());
        queue.extend(&model.forward[cell]);
    }
    let index = |cell: usize| relevant.get(&cell).copied();
    if !model.blocks.iter().any(|b| match b.op {
        CfgOp::Use(cell, _)
        | CfgOp::Resource(
            Op::Free {
                cell, ..
            }
            | Op::FreeRegion {
                cell, ..
            }
            | Op::Return {
                cell, ..
            }
            | Op::Escape {
                cell, ..
            },
        ) => index(cell).is_some(),
        _ => false,
    }) {
        return false;
    }
    let width = relevant.len();
    let born = width * 2;
    let mut states = vec![None; model.blocks.len()];
    states[0] = Some(vec![UNKNOWN; born + 1]);
    let mut queue = VecDeque::from([0]);
    while let Some(i) = queue.pop_front() {
        let mut state = states[i].clone().unwrap();
        match &model.blocks[i].op {
            CfgOp::Borrow(target, _, _)
            | CfgOp::Resource(Op::Seed {
                cell: target, ..
            }) => {
                if let Some(target) = index(*target) {
                    state[target] = if i == seed {
                        LIVE
                    } else {
                        UNKNOWN
                    };
                }
                if i == seed {
                    state[born] = LIVE;
                }
            }
            CfgOp::PointerLoad(target, source, _, _) => {
                if let Some(target) = index(*target) {
                    let value = index(*source).map_or(UNKNOWN, |s| state[s]);
                    state[target] = if i == seed && value == UNKNOWN {
                        LIVE
                    } else {
                        value
                    };
                    if i == seed {
                        if let Some(source) = index(*source) {
                            state[source] = state[target];
                        }
                        state[born] = LIVE;
                    }
                }
            }
            CfgOp::Copy(pairs) => {
                let copied: Vec<_> = pairs
                    .iter()
                    .filter_map(|(t, s)| {
                        Some((
                            index(*t)?,
                            index(*s).map_or(UNKNOWN, |s| state[s]),
                            index(*s).map_or(UNKNOWN, |s| state[s + width]),
                        ))
                    })
                    .collect();
                for (t, value, mark) in copied {
                    state[t] = value;
                    state[t + width] = mark;
                }
            }
            CfgOp::Replace(place) => {
                for (key, &id) in &model.cells.ids {
                    if let Cell::Place(p) = key
                        && model.places.contains(*place, *p)
                        && let Some(cell) = index(id)
                    {
                        state[cell] = UNKNOWN;
                    }
                }
                if region.is_some_and(|r| model.places.contains(*place, r)) {
                    for value in &mut state[..width] {
                        *value = (*value & DEAD)
                            | if *value & (LIVE | UNKNOWN) != 0 {
                                UNKNOWN
                            } else {
                                0
                            };
                    }
                }
            }
            CfgOp::Barrier => state[..width].fill(UNKNOWN),
            CfgOp::Resource(op) if kills(cx, op, origin_kind, region, &state, &index) => {
                for value in &mut state[..width] {
                    if *value & LIVE != 0 {
                        *value = (*value & !LIVE) | DEAD;
                    }
                }
            }
            CfgOp::Resource(Op::Mark(cell)) if matches!(origin_kind, Origin::Temporary) => {
                if let Some(cell) = index(*cell) {
                    state[cell + width] = if state[born] == UNKNOWN {
                        DEAD
                    } else {
                        LIVE
                    };
                }
            }
            _ => {}
        }
        for (key, &id) in &model.cells.ids {
            if let Cell::Place(p) = key
                && model.exposed.iter().any(|&a| model.places.contains(a, *p))
                && let Some(id) = index(id)
            {
                state[id] = UNKNOWN;
            }
        }
        for &next in &model.blocks[i].next {
            let changed = if let Some(previous) = &mut states[next] {
                let mut changed = false;
                for (old, &new) in previous.iter_mut().zip(&state) {
                    let joined = *old | new;
                    changed |= joined != *old;
                    *old = joined;
                }
                changed
            } else {
                states[next] = Some(state.clone());
                true
            };
            if changed {
                queue.push_back(next);
            }
        }
    }
    emit(
        Proof {
            cx,
            body,
            model,
            kind: origin_kind,
            origin,
            region,
            relevant: &relevant,
            states: &states,
        },
        out,
    );
    true
}

fn kills(
    cx: &Cx,
    op: &Op,
    origin: Origin,
    region: Option<usize>,
    state: &[u8],
    index: &impl Fn(usize) -> Option<usize>,
) -> bool {
    match *op {
        Op::Free {
            cell, ..
        } => {
            !matches!(origin, Origin::Stack(_) | Origin::Statement(_))
                && index(cell).is_some_and(|c| state[c] == LIVE)
        }
        Op::FreeRegion {
            region: r,
            cell,
            ..
        } => region == Some(r) || index(cell).is_some_and(|c| state[c] == LIVE),
        Op::Reset(_) => matches!(origin, Origin::Temporary),
        Op::Restore(cell, _) => {
            matches!(origin, Origin::Temporary)
                && index(cell).is_some_and(|c| state[c + (state.len() - 1) / 2] == DEAD)
        }
        Op::ScopeEnd(parent, _) => {
            matches!(origin, Origin::Stack(child) if inside(cx, child, parent))
        }
        Op::StatementEnd(span) => matches!(origin, Origin::Statement(at) if contains(span, at)),
        _ => false,
    }
}
