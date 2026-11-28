//! Lifetime CFG built from the optional checker's per-instantiation semantic sidecar.
//! Joins union states; only a sole DEAD state is a must proof. Copies transfer provenance
//! at their actual control-flow position, never through a mutable checker-wide alias table.
use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;

use jaic::fxhash::{HashMap, HashSet};
use jaic::ide::lifetimes::{LifetimeBody, LifetimeEvent as Event, LifetimeOperand};
use jaic::ir::Val;
use jaic::sema::procs::BodyState;
use jaic::source::Span;
use jaic::types::{ArrayKind, TypeId, TypeKind};

use super::{
    finding,
    lifetime_places::{Places, Projection},
    lifetime_summary::{Kind, Prepared, Resolver, Root, Step, Summaries},
};
use crate::{Finding, syntax::Cx};

mod build;
mod capacity;
mod controls;
mod diagnostics;
mod paths;
mod region;
use region::{Graph, States};
pub(super) mod resources;

fn dead(cx: &Cx, state: u8) -> bool {
    if cx.lifetime_mode == jacket_config::LifetimeMode::Strict {
        state & DEAD != 0
    } else {
        state == DEAD
    }
}

pub(super) use build::Built;
use build::shared;
mod iteration;
mod tasks;
mod witness;
pub(super) use iteration::check as check_iteration;

use super::lifetime_movement::{
    Action, Mutation, Selector,
    facts::{Facts, Site},
};


#[derive(Clone)]
enum Op {
    Nop,
    Control(usize),
    Decision(Span, usize, usize, Option<tasks::Token>, Val),
    Indirect(super::lifetime_targets::Set, Span),
    Use(usize, Span),
    ObserveValue(usize, Span),
    Borrow(usize, usize, Span),
    ElementBorrow(usize, usize, Selector, Option<i128>, Span),
    LogicalBorrow(usize, usize, Selector, Option<i128>, Span),
    Mutate(usize, Mutation, Span),
    ForgetElement(usize),
    Iteration {
        region: usize,
        index: usize,
        reverse: bool,
        name: jaic::intern::Sym,
        index_name: jaic::intern::Sym,
        span: Span,
        advance: bool,
    },
    IterationMutation(usize, Mutation, Span),
    IndexWrite(usize),
    PointerLoad(usize, usize, usize, Span),
    Copy(Vec<(usize, usize)>),
    Define(usize),
    Replace(usize),
    Grow(usize, Span),
    EmptyArray(usize),
    Capacity(usize, capacity::Change, Span),
    MayGrow(usize, Span),
    MayReach(usize, Span),
    MayMutate(usize, Mutation, Span),
    Barrier,
    Stop,
    Resource(resources::Op),
    Task(tasks::Op),
}

#[derive(Clone)]
struct Block {
    op: Op,
    next: Vec<usize>,
    site: Option<Span>,
}

struct Model {
    places: Places,
    cells: Cells,
    exposed: HashSet<usize>,
    blocks: Rc<Vec<Block>>,
    forward: Vec<Vec<usize>>,
    chains: HashMap<(Span, usize), Vec<Span>>,
}

const UNKNOWN: u8 = 1;
const LIVE: u8 = 2;
const DEAD: u8 = 4;
const POSITION_UNKNOWN: u8 = 8;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Cell {
    Place(usize),
    Value(Val),
}

#[derive(Clone, Default)]
struct Cells {
    ids: HashMap<Cell, usize>,
    values: Vec<Cell>,
}

impl Cells {
    fn id(&mut self, key: Cell) -> usize {
        let n = self.ids.len();

        *self.ids.entry(key).or_insert_with(|| {
            self.values.push(key);
            n
        })
    }
}

/// For each cell, the cells that receive a copy of it.
fn copy_targets(cells: usize, blocks: &[Block]) -> Vec<Vec<usize>> {
    let mut forward = vec![Vec::new(); cells];

    for block in blocks {
        match &block.op {
            Op::Copy(pairs) => {
                for &(target, source) in pairs {
                    forward[source].push(target);
                }
            }
            Op::PointerLoad(target, source, ..) => forward[*source].push(*target),
            _ => {}
        }
    }

    forward
}

fn resizable(cx: &Cx, places: &Places, p: usize) -> bool {
    places.types.get(&p).is_some_and(|&ty| {
        matches!(
            cx.compiler.ty.types.kind(ty),
            TypeKind::Array {
                kind: ArrayKind::Resizable,
                ..
            }
        )
    })
}

fn view(cx: &Cx, ty: TypeId) -> bool {
    matches!(
        cx.compiler.ty.types.kind(ty),
        TypeKind::String
            | TypeKind::Array {
                kind: ArrayKind::View,
                ..
            }
    )
}


/// Places whose address is taken without a call that is known to keep it valid.
fn unallowed_addresses(
    body: &LifetimeBody,
    places: &mut Places,
    allowed: &HashSet<(Val, Span)>,
    derived: &HashSet<(usize, Span)>,
) -> HashSet<usize> {
    body.blocks
        .iter()
        .flat_map(|b| &b.events)
        .filter_map(|event| match event {
            Event::Address {
                place,
                ty,
                span,
            } => {
                let p = places.place(*place, *ty);
                (!allowed.contains(&(*place, *span)) && !derived.contains(&(p, *span))).then_some(p)
            }
            _ => None,
        })
        .collect()
}

pub(super) fn check(cx: &Cx, out: &mut Vec<Finding>, resizable_only: bool) {
    let Some(facts) = cx.compiler.ed.facts() else {
        return;
    };
    let mut cfgs = 0;
    let mut solved = 0;
    let mut sweep_blocks = 0;
    let mut checked = HashSet::default();
    let contracts = cx
        .compiler
        .pr
        .procs
        .iter()
        .filter(|p| p.span.file == cx.file && !p.is_poly && p.lit.body.is_none())
        .count();
    let mut rejected = BTreeMap::<&str, usize>::new();
    let mut rejected_candidates = BTreeMap::<&str, usize>::new();

    for site in &cx.procs {
        for &id in cx.compiler.ide_procs_with_header(site.header.span).iter() {
            let proc = &cx.compiler.pr.procs[id.0 as usize];

            if proc.is_poly || proc.is_macro {
                continue;
            }

            if proc.lit.body.is_none() {
                continue;
            }

            if !checked.insert(id) {
                continue;
            }

            if !facts.lifetime_bodies.contains_key(&id) {
                let reason = if proc.body_state == BodyState::Done {
                    "missing-sidecar"
                } else if proc.body_state == BodyState::NotNeeded && proc.bindings.is_some() {
                    "unselected-overload-instance"
                } else {
                    "failed-or-unchecked"
                };

                *rejected.entry(reason).or_default() += 1;
            }
        }
    }

    let mut bytes = 0;
    let summaries = cx.run.lifetimes.get_or_init(|| {
        Summaries::build_for(
            cx.compiler,
            &cx.run.lifetime_files,
            cx.run.lifetime_entry_module,
        )
    });
    // Hash iteration order must not affect the order of diagnostics.
    let mut selected: Vec<_> = facts.lifetime_bodies_in(cx.file).collect();

    selected.sort_by_key(|b| b.procedure.0);
    checked.extend(selected.iter().map(|body| body.procedure));
    let bodies = checked.len();

    for body in selected {
        if jaic::interrupt::should_stop() {
            return;
        }

        let proc = &cx.compiler.pr.procs[body.procedure.0 as usize];
        let Some(syntax) = &body.syntax.body else {
            continue;
        };

        if syntax.span.file != cx.file {
            continue;
        }

        if proc.body_state != BodyState::Done || !std::sync::Arc::ptr_eq(&proc.lit, &body.syntax) {
            *rejected.entry("failed-or-obsolete").or_default() += 1;
            continue;
        }

        bytes += body.vector_bytes();
        let defs = Prepared::for_body(cx.compiler, body, summaries);
        let built = shared(cx, body, &defs, summaries);

        if jaic::interrupt::fired() {
            return;
        }

        let mut places = built.places.clone();
        let cells = built.cells.clone();
        let chains = built.chains.clone();
        let blocks = built.blocks.clone();
        let mut names = HashMap::default();
        let mut allowed_addresses = HashSet::default();
        let mut derived_addresses = HashSet::default();
        let mut resolver = Resolver::for_prepared(cx.compiler, body, &defs, summaries);

        for event in body.blocks.iter().flat_map(|b| &b.events) {
            match event {
                Event::Expression {
                    operand: LifetimeOperand::Place(place),
                    entity: Some(id),
                    ..
                } => {
                    names
                        .entry(*place)
                        .or_insert_with(|| cx.compiler.entity(*id).name.as_str());
                }
                Event::Call {
                    args, ..
                } if cx.lifetime_mode == jacket_config::LifetimeMode::Strict => {
                    for arg in args {
                        if let LifetimeOperand::Value(v) = arg.operand {
                            allowed_addresses.insert((v, arg.span));
                        }
                    }
                }
                Event::Call {
                    callee,
                    args,
                    ..
                } if summaries.procs.contains_key(callee)
                    && (cx.compiler.pr.procs[callee.0 as usize].lit.body.is_some()
                        || summaries.procs[callee].memory.is_some()
                        || cx.compiler.proc_has_note(*callee, "NoLifetimeEffects")
                        || !summaries.procs[callee].may.is_empty()
                        || !summaries.procs[callee].returns.is_empty()) =>
                {
                    let summary = &summaries.procs[callee];

                    for arg in args.iter().filter(|a| {
                        !summary.unknown_write.contains(&a.formal)
                            && (summary.may.keys().any(|e| {
                                matches!(e.kind, Kind::Resize | Kind::Mutation(_))
                                    && e.target.root == Root::Parameter(a.formal)
                            }) || summary.returns.values().any(|(p, _)| {
                                p.root == Root::Parameter(a.formal)
                                    && p.steps
                                        .iter()
                                        .any(|s| matches!(s, Step::Storage | Step::Deref))
                            }) || !summary.escapes.contains(&a.formal)
                                && !summary.may.keys().any(|e| {
                                    e.kind == Kind::Unknown
                                        && e.target.root == Root::Parameter(a.formal)
                                }))
                    }) {
                        if let LifetimeOperand::Value(place) = arg.operand {
                            allowed_addresses.insert((place, arg.span));

                            if let Some(path) = resolver.value(place, &summaries.procs)
                                && let Some(id) = resolver.materialize(&path)
                            {
                                derived_addresses
                                    .insert((places.import(&resolver.places, id), arg.span));
                            }
                        }
                    }
                }
                Event::IndirectCall {
                    args,
                    allocator: false,
                    ..
                } => {
                    let targets = resolver.call_targets(event);

                    for arg in args {
                        if (cx.lifetime_mode == jacket_config::LifetimeMode::Strict
                            || !targets.unknown
                                && targets.procedures.iter().all(|p| {
                                    summaries
                                        .procs
                                        .get(p)
                                        .is_some_and(|s| !s.unknown_write.contains(&arg.formal))
                                }))
                            && let LifetimeOperand::Value(v) = arg.operand
                        {
                            allowed_addresses.insert((v, arg.span));
                        }
                    }
                }
                _ => {}
            }
        }

        let exposed =
            unallowed_addresses(body, &mut places, &allowed_addresses, &derived_addresses);

        cfgs += 1;
        let forward = copy_targets(cells.ids.len(), &blocks);
        let model = Model {
            places,
            cells,
            exposed,
            blocks,
            forward,
            chains,
        };
        let graph = Graph::new(&model.blocks);
        let observed = Observed::new(&model.blocks);

        for (origin, block) in model.blocks.iter().enumerate() {
            if jaic::interrupt::should_stop() {
                return;
            }

            if let Op::Borrow(_, region, _)
            | Op::ElementBorrow(_, region, ..)
            | Op::LogicalBorrow(_, region, ..)
            | Op::PointerLoad(_, _, region, _) = block.op
            {
                if resizable_only && !resizable(cx, &model.places, region) {
                    continue;
                }

                if matches!(block.op, Op::PointerLoad(..)) && !observed.grown.contains(&region) {
                    continue;
                }

                if model.places.unresolved(region) && !model.places.stable_projection(region) {
                    *rejected_candidates.entry("indirect-region").or_default() += 1;
                    continue;
                }

                if model
                    .exposed
                    .iter()
                    .any(|&p| model.places.contains(p, region))
                {
                    *rejected_candidates.entry("address-exposed").or_default() += 1;
                    continue;
                }

                let (root, path) = &model.places.paths[region];
                let mut name = names.get(&root.0).copied().unwrap_or("storage").to_owned();

                for projection in path {
                    match projection {
                        Projection::Field(name_) => {
                            name.push('.');
                            name.push_str(name_.as_str());
                        }
                        Projection::Element(index) => name.push_str(&format!("[{index}]")),
                        Projection::DynamicElement(_) => name.push_str("[?]"),
                        Projection::Deref => {}
                    }
                }

                solved += usize::from(solve(
                    cx,
                    &model,
                    (&graph, &observed),
                    origin,
                    &name,
                    out,
                    (resizable_only, &mut sweep_blocks),
                ));
            }
        }
    }

    if jacket_config::env::LIFETIME_STATS.is_set() {
        let rank = |map: BTreeMap<&'static str, usize>| {
            let mut pairs: Vec<_> = map.into_iter().collect();

            pairs.sort_by_key(|(reason, count)| (std::cmp::Reverse(*count), *reason));
            pairs
        };

        eprintln!(
            "lifetime-stats file={} bodies={bodies} contracts={contracts} cfgs={cfgs} candidates={solved} sweep_blocks={sweep_blocks} sidecar_bytes={bytes} rejected={:?} rejected_candidates={:?}",
            cx.compiler.src.sources.get(cx.file).path,
            rank(rejected),
            rank(rejected_candidates)
        );
    }
}

/// What the candidates of one CFG ask about its blocks, indexed once.
struct Observed {
    /// The blocks that read each cell.
    reads: HashMap<usize, Vec<usize>>,
    /// The regions some block grows.
    grown: HashSet<usize>,
}

impl Observed {
    fn new(blocks: &[Block]) -> Self {
        let mut reads = HashMap::<usize, Vec<usize>>::default();
        let mut grown = HashSet::default();

        for (i, block) in blocks.iter().enumerate() {
            match block.op {
                Op::Use(cell, _)
                | Op::ObserveValue(cell, _)
                | Op::Resource(resources::Op::Read {
                    cell, ..
                }) => reads.entry(cell).or_default().push(i),
                Op::Grow(region, _) => {
                    grown.insert(region);
                }
                _ => {}
            }
        }

        Self {
            reads,
            grown,
        }
    }
}

fn solve(
    cx: &Cx,
    model: &Model,
    (graph, observed): (&Graph, &Observed),
    candidate: usize,
    region_name: &str,
    out: &mut Vec<Finding>,
    (resizable_only, sweep_blocks): (bool, &mut usize),
) -> bool {
    let Model {
        blocks,
        places,
        cells,
        exposed,
        forward,
        chains: _,
    } = model;
    let (first, region, _origin, element) = match blocks[candidate].op {
        Op::Borrow(first, region, origin) | Op::PointerLoad(first, _, region, origin) => {
            (first, region, origin, None)
        }
        Op::ElementBorrow(first, region, index, length, origin)
        | Op::LogicalBorrow(first, region, index, length, origin) => {
            (first, region, origin, Some((index, length)))
        }
        _ => return false,
    };
    // Only retain cells reachable by copying this origin. Unrelated scalar loads/stores
    // do not multiply per-candidate state memory.
    let mut relevant = HashMap::default();
    let mut queue = VecDeque::from([first]);

    while let Some(cell) = queue.pop_front() {
        if relevant.contains_key(&cell) {
            continue;
        }

        relevant.insert(cell, relevant.len());
        queue.extend(&forward[cell]);
    }

    let reads: Vec<_> = relevant
        .keys()
        .filter_map(|c| observed.reads.get(c))
        .flatten()
        .copied()
        .collect();

    if reads.is_empty() {
        return false;
    }

    let exposed_cells: HashSet<_> = relevant
        .iter()
        .filter_map(|(&cell, &dense)| match cells.values[cell] {
            Cell::Place(p)
                if places.unresolved(p) || exposed.iter().any(|&a| places.contains(a, p)) =>
            {
                Some(dense)
            }
            _ => None,
        })
        .collect();
    // Before the candidate every cell is unknown, which no operation changes, so the sweep
    // starts at the candidate and stops past the last read of its cells.
    let scope = graph.scope(blocks, candidate, reads);
    let (mut states, mut queue) = scope.begin(graph, vec![UNKNOWN; relevant.len()]);

    while let Some(i) = queue.pop_front() {
        *sweep_blocks += 1;
        let mut state = states.visited(i).unwrap().clone();

        match &blocks[i].op {
            Op::LogicalBorrow(target, ..)
                if matches!(blocks[candidate].op, Op::LogicalBorrow(..)) =>
            {
                if let Some(&cell) = relevant.get(target) {
                    state[cell] = if i == candidate {
                        LIVE
                    } else {
                        UNKNOWN
                    };
                }
            }
            Op::Borrow(target, ..) | Op::ElementBorrow(target, ..) => {
                if let Some(&cell) = relevant.get(target) {
                    state[cell] = if i == candidate {
                        LIVE
                    } else {
                        UNKNOWN
                    };
                }
            }
            Op::PointerLoad(target, source, ..) => {
                if let Some(&cell) = relevant.get(target) {
                    let source = relevant
                        .get(source)
                        .map_or(UNKNOWN, |&source| state[source]);

                    state[cell] = if i == candidate {
                        (source & !UNKNOWN)
                            | if source & UNKNOWN != 0 {
                                LIVE
                            } else {
                                0
                            }
                    } else {
                        source
                    };
                }
            }
            Op::Define(target) => {
                if let Some(&cell) = relevant.get(target) {
                    state[cell] = UNKNOWN;
                }
            }
            Op::Copy(pairs) => {
                let values: Vec<_> = pairs
                    .iter()
                    .filter_map(|(target, source)| {
                        let &target = relevant.get(target)?;
                        let value = relevant
                            .get(source)
                            .map_or(UNKNOWN, |&source| state[source]);

                        Some((target, value))
                    })
                    .collect();

                for (target, value) in values {
                    state[target] = value;
                }
            }
            Op::Replace(p) if places.contains(*p, region) => {
                for cell in &mut state {
                    *cell = (*cell & DEAD)
                        | if *cell & (LIVE | POSITION_UNKNOWN | UNKNOWN) != 0 {
                            UNKNOWN
                        } else {
                            0
                        };
                }
            }
            Op::Mutate(r, mutation, _) if *r == region && element.is_some() && !resizable_only => {
                move_element(&mut state, *mutation, element.unwrap());
            }
            Op::ForgetElement(target) if element.is_some() => {
                if let Some(&cell) = relevant.get(target)
                    && state[cell] & LIVE != 0
                {
                    state[cell] = (state[cell] & !LIVE) | POSITION_UNKNOWN;
                }
            }
            Op::Replace(p)
                if places.unresolved(*p) && places.paths[*p].0 != places.paths[region].0 =>
            {
                state.fill(UNKNOWN);
            }
            Op::MayGrow(r, _) | Op::MayReach(r, _)
                if cx.lifetime_mode == jacket_config::LifetimeMode::Strict
                    && (*r == region
                        || matches!(blocks[i].op, Op::MayReach(..))
                            && places.contains(*r, region)) =>
            {
                possibly_dead(&mut state);
            }
            Op::MayMutate(r, mutation, _)
                if cx.lifetime_mode == jacket_config::LifetimeMode::Strict
                    && !resizable_only
                    && *r == region
                    && element.is_some_and(|(s, len)| {
                        super::lifetime_movement::possibly_invalidates(*mutation, s, len)
                    }) =>
            {
                possibly_dead(&mut state);
            }
            Op::Barrier => state.fill(UNKNOWN),
            Op::Grow(r, _) if *r == region => {
                for cell in &mut state {
                    if *cell & (LIVE | POSITION_UNKNOWN) != 0 {
                        *cell = (*cell & !(LIVE | POSITION_UNKNOWN)) | DEAD;
                    }
                }
            }
            _ => {}
        }

        for &cell in &exposed_cells {
            state[cell] = UNKNOWN;
        }

        for &next in &blocks[i].next {
            if scope.contains(next) && states.join(next, &state) {
                queue.push_back(next);
            }
        }
    }

    out.extend(diagnostics::emit(
        cx,
        model,
        candidate,
        &states,
        &relevant,
        region_name,
        resizable_only,
    ));
    true
}

/// Follow the used value backwards through simultaneous copies. A later grow of a rebound
/// descriptor cannot explain a pointer that was already dead: only LIVE -> DEAD transitions
/// contribute invalidators. Witness storage is allocated on demand, outside the fixed point.
fn invalidation_cut(
    model: &Model,
    states: &States<'_>,
    relevant: &HashMap<usize, usize>,
    use_at: usize,
    use_cell: usize,
    region: usize,
    element: Option<(Selector, Option<i128>)>,
) -> Vec<Span> {
    let blocks = &model.blocks;
    let mut incoming = vec![Vec::new(); blocks.len()];

    for (from, block) in blocks
        .iter()
        .enumerate()
        .filter(|(i, _)| states.get(*i).is_some())
    {
        for &to in &block.next {
            incoming[to].push(from);
        }
    }

    let mut seen = HashSet::default();
    let mut queue = VecDeque::from([(use_at, use_cell)]);
    let mut spans = Vec::new();

    while let Some((i, mut cell)) = queue.pop_front() {
        if !seen.insert((i, cell)) {
            continue;
        }

        match &blocks[i].op {
            Op::Grow(id, at)
            | Op::MayGrow(id, at)
            | Op::MayReach(id, at)
            | Op::Mutate(id, _, at)
            | Op::MayMutate(id, _, at)
                if (*id == region
                    || matches!(blocks[i].op, Op::MayReach(..))
                        && model.places.contains(*id, region))
                    && match blocks[i].op {
                        Op::MayMutate(_, m, _) => element.is_some_and(|(s, len)| {
                            super::lifetime_movement::possibly_invalidates(m, s, len)
                        }),
                        Op::Mutate(_, m, _) => element.is_some_and(|(s, len)| {
                            super::lifetime_movement::invalidates(m, s, len)
                        }),
                        _ => true,
                    } =>
            {
                let before = states.get(i).unwrap()[cell];
                let live = if matches!(blocks[i].op, Op::Mutate(..)) {
                    LIVE
                } else {
                    LIVE | POSITION_UNKNOWN
                };

                if before & live != 0 {
                    spans.push(*at);
                }

                if before & DEAD == 0 {
                    continue;
                }
            }
            Op::Copy(pairs) => {
                if let Some((_, source)) = pairs
                    .iter()
                    .find(|(target, _)| relevant.get(target) == Some(&cell))
                {
                    let Some(&source) = relevant.get(source) else {
                        continue;
                    };

                    cell = source;
                }
            }
            Op::PointerLoad(target, source, ..) if relevant.get(target) == Some(&cell) => {
                let Some(&source) = relevant.get(source) else {
                    continue;
                };

                cell = source;
            }
            Op::LogicalBorrow(target, ..)
                if element.is_some() && relevant.get(target) == Some(&cell) =>
            {
                continue;
            }
            Op::Borrow(target, ..) | Op::ElementBorrow(target, ..)
                if relevant.get(target) == Some(&cell) =>
            {
                continue;
            }
            Op::Barrier => continue,
            _ => {}
        }

        queue.extend(incoming[i].iter().map(|&p| (p, cell)));
    }

    spans.sort_unstable_by_key(|s| (s.file.0, s.start, s.end));
    spans.dedup();
    spans
}


fn possibly_dead(state: &mut [u8]) {
    for cell in state {
        if *cell & (LIVE | POSITION_UNKNOWN) != 0 {
            *cell |= DEAD;
        }
    }
}


fn move_element(
    state: &mut [u8],
    mutation: Mutation,
    (selector, length): (Selector, Option<i128>),
) {
    let replacement = if super::lifetime_movement::invalidates(mutation, selector, length) {
        DEAD
    } else if matches!(selector, Selector::Last | Selector::CountDelta(_)) {
        POSITION_UNKNOWN
    } else {
        return;
    };

    for cell in state {
        if *cell & LIVE != 0 {
            *cell = (*cell & !LIVE) | replacement;
        }
    }
}
