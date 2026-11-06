//! Shared semantic-CFG analysis for released resources and escaping storage.
//! Each candidate follows only its copy closure; joins preserve uncertainty.
use super::{Block, Cell, Cells, DEAD, LIVE, Model, Op as CfgOp, UNKNOWN, build};
use crate::rules::lifetime_facts::current_body;
use crate::rules::lifetime_places::Places;
use crate::rules::lifetime_summary::{Kind as EffectKind, Prepared, Resolver, Root, Summaries};
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

mod addresses;
mod boxed;
mod calls;
mod diagnostics;
mod origins;
mod quiescent;
mod retained;
mod retained_witness;
mod stores;
mod temporary;
use diagnostics::{Proof, emit};

#[derive(Clone, Copy, Debug)]
pub(super) enum Origin {
    Heap,
    Temporary,
    Stack(ScopeId),
    Statement(Span),
    Incoming,
    Until(ProcId),
}

#[derive(Clone, Debug)]
pub(super) enum Op {
    Read {
        cell: usize,
        span: Span,
    },
    Seed {
        possible: bool,
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
        result: usize,
    },
    Escape {
        location: Option<Box<(Val, crate::rules::lifetime_facts::ByteRange)>>,
        cell: usize,
        span: Span,
        pointer: bool,
        target: usize,
        store: bool,
        temporary_target: bool,
        parameter_target: bool,
    },
    ScopeEnd(ScopeId, Span),
    StatementEnd(Span),
    Reset(Span),
    Mark(usize),
    Restore(usize, Span),
    Exit,
    MayFree {
        cell: usize,
        span: Span,
    },
    MayRegion {
        region: usize,
        cells: Vec<usize>,
        span: Span,
    },
    MayReset(Span),
    MayRestore(usize, Span),
    Advance(ProcId, Span),
    MayAdvance(ProcId, Span),
}

#[derive(Default)]
pub(crate) struct Reports {
    pub use_after_free: Vec<Finding>,
    pub double_free: Vec<Finding>,
    pub returned_local_address: Vec<Finding>,
    pub temp_storage_escape: Vec<Finding>,
    external_primary: usize,
    fast_quiet: usize,
    covered_fresh_loads: usize,
}

impl Reports {
    /// Capture report lengths before one proof emits diagnostics.
    fn counts(&self) -> [usize; 4] {
        [
            self.use_after_free.len(),
            self.double_free.len(),
            self.returned_local_address.len(),
            self.temp_storage_escape.len(),
        ]
    }

    /// Visit only findings emitted after a captured report boundary.
    fn added(&mut self, before: [usize; 4]) -> impl Iterator<Item = &mut Finding> {
        self.use_after_free[before[0]..]
            .iter_mut()
            .chain(self.double_free[before[1]..].iter_mut())
            .chain(self.returned_local_address[before[2]..].iter_mut())
            .chain(self.temp_storage_escape[before[3]..].iter_mut())
    }
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

pub(super) fn operand_cell(
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

pub(super) fn value_cell(
    cx: &Cx,
    places: &mut Places,
    cells: &mut Cells,
    value: Val,
    ty: TypeId,
) -> usize {
    if cx.compiler.ir_ty(ty).is_none() {
        cells.id(Cell::Place(places.place(value, ty)))
    } else {
        cells.id(Cell::Value(value))
    }
}

#[derive(Clone, Copy)]
struct Call<'a> {
    callee: &'a ProcId,
    event: &'a Event,
    args: &'a [LifetimeArgument],
    results: &'a [(usize, Val, TypeId)],
    span: &'a Span,
}

pub(super) struct Builder<'a> {
    cx: &'a Cx<'a>,
    summaries: &'a Summaries,
    locals: HashMap<Val, ScopeId>,
    boxed: HashMap<Val, ScopeId>,
    parameters: HashSet<Val>,
    stack: HashSet<Val>,
    returns: HashSet<Val>,
    released_values: HashSet<Val>,
    value_spans: HashMap<(Val, TypeId), Span>,
    store_spans: HashMap<Span, Span>,
    member_spans: HashMap<Span, Span>,
    capture_scope: std::cell::Cell<Option<bool>>,
    capture_work: std::cell::Cell<usize>,
    capture_index: std::cell::RefCell<Option<Option<crate::rules::lifetime_summary::QueryIndex>>>,
}

impl<'a> Builder<'a> {
    pub fn new(
        cx: &'a Cx<'a>,
        body: &LifetimeBody,
        summaries: &'a Summaries,
        places: &Places,
    ) -> Self {
        let mut locals = HashMap::default();
        let mut parameters = HashSet::default();
        let mut returns = HashSet::default();
        let mut released_values = HashSet::default();
        let mut value_spans = HashMap::default();
        let mut store_spans = HashMap::default();
        let mut member_spans = HashMap::default();
        for e in body.blocks.iter().flat_map(|b| &b.events) {
            match e {
                Event::Local {
                    place,
                    scope,
                    ..
                } => {
                    locals.insert(*place, addresses::storage_scope(body, *scope));
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
                } if crate::rules::lifetime_movement::released_formal(cx.compiler, *callee)
                    .is_some() =>
                {
                    if let Some(a) = args.iter().find(|a| {
                        Some(a.formal)
                            == crate::rules::lifetime_movement::released_formal(
                                cx.compiler,
                                *callee,
                            )
                    }) && let LifetimeOperand::Value(v) = a.operand
                    {
                        released_values.insert(v);
                    }
                }
                Event::IndirectCall {
                    callee,
                    args,
                    allocator: false,
                    ..
                } => {
                    let targets = summaries.targets.get(body.procedure, *callee);
                    for e in targets
                        .procedures
                        .iter()
                        .filter_map(|p| summaries.procs.get(p))
                        .flat_map(|s| s.may.keys())
                        .filter(|e| e.kind == EffectKind::Free && e.target.steps.is_empty())
                    {
                        if let Root::Parameter(formal) = e.target.root
                            && let Some(arg) = args.iter().find(|a| a.formal == formal)
                            && let LifetimeOperand::Value(v) = arg.operand
                        {
                            released_values.insert(v);
                        }
                    }
                }
                _ => {}
            }
        }
        if let Some(syntax) = &body.syntax.body {
            jaic::ast::walk::walk(&syntax.stmts, &mut |node, _| {
                if let Some(expr) = node.expr()
                    && let jaic::ast::ExprKind::Member(base, member) = &expr.kind
                {
                    member_spans.insert(member.span, base.span);
                }
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
        let boxed = boxed::holders(cx, body, places, &locals);
        Self {
            cx,
            summaries,
            locals,
            boxed,
            parameters,
            returns,
            released_values,
            value_spans,
            store_spans,
            member_spans,
            capture_scope: std::cell::Cell::new(None),
            capture_work: std::cell::Cell::new(8192),
            capture_index: std::cell::RefCell::new(None),
            stack: body.stack_places.iter().copied().collect(),
        }
    }

    pub fn member_read_span(&self, span: Span) -> Span {
        self.member_spans.get(&span).copied().unwrap_or(span)
    }

    pub fn result_place(&self, value: Val) -> bool {
        self.returns.contains(&value)
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
                    possible: false,
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
                            possible: false,
                            cell: cells.id(Cell::Value(*place)),
                            origin: Origin::Stack(scope),
                            span: *span,
                        });
                    }
                } else if self.parameters.contains(place) {
                    // The address is that of the callee's by-value parameter copy.
                    ops.push(Op::Seed {
                        possible: false,
                        cell: cells.id(Cell::Value(*place)),
                        origin: Origin::Stack(ScopeId(u32::MAX)),
                        span: *span,
                    });
                }
            }
            Event::View {
                value,
                source,
                from,
                to,
                span,
            } if matches!(
                cx.compiler.ty.types.kind(*from),
                TypeKind::Array {
                    kind: jaic::types::ArrayKind::Fixed(_),
                    ..
                }
            ) =>
            {
                let p = places.place(*source, *from);
                if !places.unresolved(p)
                    && let Some(scope) = self.locals.get(&places.paths[p].0.0).copied()
                {
                    ops.push(Op::Seed {
                        possible: false,
                        cell: cells.id(Cell::Place(places.place(*value, *to))),
                        origin: Origin::Stack(scope),
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
                    && !places.unresolved(p)
                    && matches!(
                        cx.compiler.ty.types.kind(*region_ty),
                        TypeKind::Array {
                            kind: jaic::types::ArrayKind::Fixed(_),
                            ..
                        }
                    )
                {
                    ops.push(Op::Seed {
                        possible: false,
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
                if let Some(scope) = self.locals.get(&places.paths[p].0.0).copied()
                    && !places.unresolved(p)
                {
                    ops.push(Op::Seed {
                        possible: false,
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
                let origin = boxed::origin(self, places, *value, *place, *ty, *temporary, *span);
                if let Some(origin) = origin {
                    ops.push(Op::Seed {
                        possible: false,
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
                index,
                ty,
                span,
                ..
            } => {
                ops.push(Op::Return {
                    cell: value_cell(cx, places, cells, *value, *ty),
                    span: *span,
                    result: *index,
                });
                if cx.compiler.ir_ty(*ty).is_none() {
                    let parent = places.place(*value, *ty);
                    for child in places.descendants(parent) {
                        ops.push(Op::Return {
                            cell: cells.id(Cell::Place(child)),
                            span: *span,
                            result: *index,
                        });
                    }
                }
            }
            Event::Store {
                ..
            } => ops.extend(self.store_escapes(event, places, cells, resolver)),
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
                        event,
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
                allocator: false,
                args,
                results,
                span,
                ..
            } => {
                const INDIRECT: ProcId = ProcId(u32::MAX);
                ops.extend(self.call(
                    Call {
                        callee: &INDIRECT,
                        event,
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
                allocator: true, ..
            } => ops.extend(self.allocation(event, places, cells, resolver)),
            _ => {}
        }
        ops
    }
}

fn analyze(cx: &Cx) -> Reports {
    let mut reports = Reports::default();
    let Some(facts) = cx.compiler.ed.facts() else {
        return reports;
    };
    let summaries = cx.run.lifetimes.get_or_init(|| {
        Summaries::build_for(
            cx.compiler,
            &cx.run.lifetime_files,
            cx.run.lifetime_entry_module,
        )
    });
    let mut bodies: Vec<_> = facts.lifetime_bodies_in(cx.file).collect();
    bodies.sort_by_key(|b| b.procedure.0);
    let mut checked = HashSet::default();
    let contracts = cx
        .compiler
        .pr
        .procs
        .iter()
        .filter(|p| p.span.file == cx.file && !p.is_poly && p.lit.body.is_none())
        .count();
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
            if p.lit.body.is_none() {
                continue;
            }
            if !checked.insert(id) {
                continue;
            }
            if !facts.lifetime_bodies.contains_key(&id) {
                if jacket_config::env::LIFETIME_STATS.is_set() {
                    eprintln!(
                        "lifetime-body-rejection {}",
                        serde_json::json!({"file":cx.compiler.src.sources.get(cx.file).path.to_string(),"name":p.name.as_str(),"start":p.span.start,"end":p.span.end,"state":format!("{:?}",p.body_state),"instance":p.bindings.is_some()})
                    );
                }
                *rejected
                    .entry(if p.body_state == BodyState::Done {
                        "missing-sidecar"
                    } else if p.body_state == BodyState::NotNeeded && p.bindings.is_some() {
                        "unselected-overload-instance"
                    } else {
                        "failed-or-unchecked"
                    })
                    .or_default() += 1;
            }
        }
    }
    checked.extend(bodies.iter().map(|body| body.procedure));
    let checked_bodies = checked.len();
    let mut solved = 0;
    for body in bodies {
        if !current_body(cx.compiler, body.procedure, body) {
            *rejected.entry("failed-or-obsolete").or_default() += 1;
            continue;
        }
        let proc = &cx.compiler.pr.procs[body.procedure.0 as usize];
        cfgs += 1;
        let trace = jacket_config::env::LIFETIME_STATS.is_set()
            && body.blocks.iter().map(|b| b.events.len()).sum::<usize>() > 5000;
        let started = jaic::platform::Stopwatch::start();
        if trace {
            eprintln!(
                "lifetime-resource-body begin file={} proc={} name={} events={}",
                cx.compiler.src.sources.get(cx.file).path,
                body.procedure.0,
                proc.name,
                body.blocks.iter().map(|b| b.events.len()).sum::<usize>()
            );
        }
        bytes += body.vector_bytes();
        let mut places = Places::new(cx.compiler, body);
        if trace {
            eprintln!(
                "lifetime-resource-body places proc={} wall_ms={:.3}",
                body.procedure.0,
                started.elapsed().as_secs_f64() * 1000.0
            );
        }
        let mut cells = Cells::default();
        let mut chains = HashMap::default();
        let prepared = Prepared::for_body(
            crate::rules::lifetime_facts::CheckedSession::new(cx.compiler)
                .body((body).procedure)
                .unwrap(),
            summaries,
        );
        let Some(built) = build(
            cx,
            body,
            &prepared,
            &mut places,
            &mut cells,
            summaries,
            &mut chains,
        ) else {
            continue;
        };
        let blocks = built.blocks;
        let borrows = built.borrows;
        if trace {
            eprintln!(
                "lifetime-resource-body cfg proc={} blocks={} cells={} wall_ms={:.3}",
                body.procedure.0,
                blocks.len(),
                cells.ids.len(),
                started.elapsed().as_secs_f64() * 1000.0
            );
        }
        let forward = addresses::value_edges(&cells, &blocks);
        let mut origins = None;
        let exposed = addresses::exposed(cx, body, &prepared, summaries, &mut places, &borrows);
        if trace {
            eprintln!(
                "lifetime-resource-body addresses proc={} wall_ms={:.3}",
                body.procedure.0,
                started.elapsed().as_secs_f64() * 1000.0
            );
        }
        let model = Model {
            places,
            cells,
            exposed,
            blocks,
            forward,
            chains,
        };
        let observable = quiescent::observable(&model);
        let operations = quiescent::Index::new(&model);
        let covered = quiescent::covered_loads(&model);
        let mut traced_candidates = 0;
        for (candidate, block) in model.blocks.iter().enumerate() {
            if covered[candidate] {
                solved += 1;
                reports.covered_fresh_loads += 1;
                continue;
            }
            if matches!(
                block.op,
                CfgOp::Resource(Op::Seed { .. })
                    | CfgOp::PointerLoad(..)
                    | CfgOp::Borrow(..)
                    | CfgOp::ElementBorrow(..)
            ) {
                if let Some((_, _, _, Some(region))) = self::candidate(block) {
                    let reason = if model.places.unresolved(region)
                        && !model.places.stable_projection(region)
                    {
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
                if trace {
                    traced_candidates += 1;
                    if traced_candidates < 10 || traced_candidates % 100 == 0 {
                        eprintln!(
                            "lifetime-resource-body candidate proc={} candidate={} ordinal={} wall_ms={:.3}",
                            body.procedure.0,
                            candidate,
                            traced_candidates,
                            started.elapsed().as_secs_f64() * 1000.0
                        );
                    }
                }
                let before = reports.counts();
                solved += usize::from(solve(
                    cx,
                    body,
                    &model,
                    candidate,
                    &observable,
                    &operations,
                    &mut reports,
                ));
                if let CfgOp::PointerLoad(target, ..) = block.op
                    && let Cell::Value(value) = model.cells.values[target]
                {
                    origins::annotate(&mut origins, &mut reports, before, value, || {
                        origins::Index::build(cx, body, &prepared, &model.blocks, &model.cells)
                    });
                }
            }
        }
        if trace {
            eprintln!(
                "lifetime-resource-body end proc={} candidates={} wall_ms={:.3}",
                body.procedure.0,
                traced_candidates,
                started.elapsed().as_secs_f64() * 1000.0
            );
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
            "lifetime-resource-stats file={} bodies={checked_bodies} contracts={contracts} cfgs={cfgs} candidates={solved} sidecar_bytes={bytes} rejected={:?} rejected_candidates={:?} use_after_free={} double_free={} stack_escapes={} temp_escapes={} external_primary={} fast_quiet={} covered_fresh_loads={}",
            cx.compiler.src.sources.get(cx.file).path,
            rank(rejected),
            rank(rejected_candidates),
            reports.use_after_free.len(),
            reports.double_free.len(),
            reports.returned_local_address.len(),
            reports.temp_storage_escape.len(),
            reports.external_primary,
            reports.fast_quiet,
            reports.covered_fresh_loads
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
            ..
        }) => Some((cell, origin, span, None)),
        CfgOp::PointerLoad(cell, _, region, span, _) => {
            Some((cell, Origin::Incoming, span, Some(region)))
        }
        CfgOp::Borrow(cell, region, span) | CfgOp::ElementBorrow(cell, region, _, _, span) => {
            Some((cell, Origin::Incoming, span, Some(region)))
        }
        _ => None,
    }
}

fn solve(
    cx: &Cx,
    body: &LifetimeBody,
    model: &Model,
    seed: usize,
    observable: &[bool],
    operations: &quiescent::Index,
    out: &mut Reports,
) -> bool {
    let Some((first, origin_kind, origin, region)) = candidate(&model.blocks[seed]) else {
        return false;
    };
    if operations.independent_quiet(origin_kind) {
        out.fast_quiet += 1;
        return true;
    }
    let mut relevant = HashMap::default();
    let source = match model.blocks[seed].op {
        CfgOp::PointerLoad(_, source, _, _, _) => Some(source),
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
    if !operations.used(&relevant) {
        return false;
    }
    if quiescent::quiet(
        cx,
        model,
        origin_kind,
        region,
        &relevant,
        observable,
        operations,
    ) {
        out.fast_quiet += 1;
        return true;
    }
    let exposed: Vec<_> = if model.exposed.is_empty() {
        Vec::new()
    } else {
        model
            .cells
            .ids
            .iter()
            .filter_map(|(key, &id)| {
                let dense = index(id)?;
                let Cell::Place(p) = key else {
                    return None;
                };
                model
                    .exposed
                    .iter()
                    .any(|&a| model.places.contains(a, *p))
                    .then_some(dense)
            })
            .collect()
    };
    let width = relevant.len();
    let born = width * 2;
    let tokens = retained::tokens(model, born + 1);
    let mut states = vec![None; model.blocks.len()];
    states[0] = Some(vec![UNKNOWN; born + 1 + tokens.len()]);
    let mut queue = VecDeque::from([0]);
    while let Some(i) = queue.pop_front() {
        let mut state = states[i].clone().unwrap();
        match &model.blocks[i].op {
            CfgOp::Borrow(target, _, _)
            | CfgOp::ElementBorrow(target, ..)
            | CfgOp::Resource(Op::Seed {
                cell: target, ..
            }) => {
                if i == seed && matches!(origin_kind, Origin::Heap | Origin::Temporary) {
                    // Each execution creates a new generation. Retain already-dead aliases,
                    // but do not make old live aliases refer to this fresh allocation.
                    retained::new_generation(&mut state, width);
                }
                if let Some(target) = index(*target) {
                    state[target] = if i == seed {
                        addresses::seed_state(&model.blocks[seed].op)
                    } else {
                        UNKNOWN
                    };
                }
                if i == seed {
                    state[born] = LIVE;
                }
            }
            CfgOp::PointerLoad(target, source, _, _, captured) => addresses::load_pointer(
                model,
                (*target, *source, *captured),
                &mut state,
                &index,
                i == seed,
                born,
            ),
            CfgOp::Define(target) => addresses::define(&mut state, index(*target)),
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
                for p in std::iter::once(*place).chain(model.places.descendants(*place)) {
                    if let Some(&cell) = model.cells.ids.get(&Cell::Place(p))
                        && let Some(cell) = index(cell)
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
            CfgOp::MemoryWrite => {
                for (&cell, &slot) in &relevant {
                    if matches!(model.cells.values[cell], Cell::Place(_)) {
                        state[slot] = UNKNOWN;
                        state[slot + width] = UNKNOWN;
                    }
                }
            }
            CfgOp::Barrier => state[..width].fill(UNKNOWN),
            CfgOp::Resource(op) if kills(cx, op, origin_kind, region, &state[..=born], &index) => {
                for value in &mut state[..width] {
                    if *value & LIVE != 0 {
                        *value = if matches!(
                            op,
                            Op::MayFree { .. }
                                | Op::MayRegion { .. }
                                | Op::MayReset(_)
                                | Op::MayRestore(..)
                                | Op::MayAdvance(..)
                        ) {
                            *value | DEAD
                        } else {
                            (*value & !LIVE) | DEAD
                        };
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
            CfgOp::Task(op) => retained::transfer(cx, op, &tokens, &index, &mut state),
            _ => {}
        }
        for &id in &exposed {
            state[id] = UNKNOWN;
        }
        for &next in &model.blocks[i].next {
            let mut outgoing = state.clone();
            if let CfgOp::Decision(_, yes, _, Some(token), _) = model.blocks[i].op
                && next == yes
                && let Some(&at) = tokens.get(&token)
            {
                outgoing[at] = UNKNOWN;
            }
            let changed = if let Some(previous) = &mut states[next] {
                let mut changed = false;
                for (old, &new) in previous.iter_mut().zip(&outgoing) {
                    let joined = *old | new;
                    changed |= joined != *old;
                    *old = joined;
                }
                changed
            } else {
                states[next] = Some(outgoing);
                true
            };
            if changed {
                queue.push_back(next);
            }
        }
    }
    let proof = Proof {
        cx,
        body,
        model,
        kind: origin_kind,
        seed,
        origin,
        region,
        relevant: &relevant,
        states: &states,
    };
    emit(proof, out);
    retained::emit(proof, &tokens, out);
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
    match op {
        Op::MayFree {
            cell, ..
        } => {
            cx.lifetime_mode == jacket_config::LifetimeMode::Strict
                && !matches!(origin, Origin::Stack(_) | Origin::Statement(_))
                && index(*cell).is_some_and(|c| state[c] & LIVE != 0)
        }
        Op::MayRegion {
            region: r,
            cells,
            ..
        } => {
            cx.lifetime_mode == jacket_config::LifetimeMode::Strict
                && !matches!(origin, Origin::Stack(_) | Origin::Statement(_))
                && (region == Some(*r)
                    || cells
                        .iter()
                        .any(|c| index(*c).is_some_and(|n| state[n] & LIVE != 0)))
        }
        Op::MayReset(_) => {
            cx.lifetime_mode == jacket_config::LifetimeMode::Strict
                && matches!(origin, Origin::Temporary)
        }
        Op::MayRestore(cell, _) => {
            cx.lifetime_mode == jacket_config::LifetimeMode::Strict
                && matches!(origin, Origin::Temporary)
                && index(*cell).is_some_and(|c| state[c + (state.len() - 1) / 2] & DEAD != 0)
        }
        Op::Advance(p, _) | Op::MayAdvance(p, _) => matches!(origin, Origin::Until(id) if id == *p),
        Op::Free {
            cell, ..
        } => {
            !matches!(origin, Origin::Stack(_) | Origin::Statement(_))
                && index(*cell).is_some_and(|c| {
                    if cx.lifetime_mode == jacket_config::LifetimeMode::Strict {
                        state[c] & LIVE != 0
                    } else {
                        state[c] == LIVE
                    }
                })
        }
        Op::FreeRegion {
            region: r,
            cell,
            ..
        } => {
            region == Some(*r)
                || index(*cell).is_some_and(|c| {
                    if cx.lifetime_mode == jacket_config::LifetimeMode::Strict {
                        state[c] & LIVE != 0
                    } else {
                        state[c] == LIVE
                    }
                })
        }
        Op::Reset(_) => matches!(origin, Origin::Temporary),
        Op::Restore(cell, _) => {
            matches!(origin, Origin::Temporary)
                && index(*cell).is_some_and(|c| state[c + (state.len() - 1) / 2] == DEAD)
        }
        Op::ScopeEnd(parent, _) => {
            matches!(origin, Origin::Stack(child) if inside(cx, child, *parent))
        }
        Op::StatementEnd(span) => matches!(origin, Origin::Statement(at) if contains(*span, at)),
        _ => false,
    }
}
