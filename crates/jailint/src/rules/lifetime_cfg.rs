//! Lifetime CFG built from the optional checker's per-instantiation semantic sidecar.
//! Joins union states; only a sole DEAD state is a must proof. Copies transfer provenance
//! at their actual control-flow position, never through a mutable checker-wide alias table.
use super::{
    finding,
    lifetime_places::{Places, Projection},
};
use crate::{Finding, syntax::Cx};
use jaic::fxhash::{HashMap, HashSet};
use jaic::ide::lifetimes::{LifetimeBody, LifetimeEvent as Event, LifetimeOperand};
use jaic::ir::Val;
use jaic::sema::{ProcId, procs::BodyState};
use jaic::source::Span;
use jaic::types::{ArrayKind, TypeId, TypeKind};
use std::collections::{BTreeMap, VecDeque};

#[derive(Clone)]
enum Op {
    Nop,
    Use(usize, Span),
    Borrow(usize, usize, Span),
    Copy(Vec<(usize, usize)>),
    Replace(usize),
    Grow(usize, Span),
    Barrier,
}

struct Block {
    op: Op,
    next: Vec<usize>,
}

struct Model {
    places: Places,
    cells: Cells,
    exposed: HashSet<usize>,
    blocks: Vec<Block>,
    forward: Vec<Vec<usize>>,
}

const UNKNOWN: u8 = 1;
const LIVE: u8 = 2;
const DEAD: u8 = 4;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Cell {
    Place(usize),
    Value(Val),
}

#[derive(Default)]
struct Cells {
    ids: HashMap<Cell, usize>,
}

impl Cells {
    fn id(&mut self, key: Cell) -> usize {
        let n = self.ids.len();
        *self.ids.entry(key).or_insert(n)
    }
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


/// Step 3's local return proof: all returns must trace through single-assignment values
/// to a descriptor's built-in data field. Unknown producers, multiple writes and cycles
/// fail closed. No container name or return type implies provenance.
fn returned_storage(cx: &Cx, body: &LifetimeBody, index: usize) -> Option<usize> {
    let mut places = Places::new(cx.compiler, body);
    let mut parameters = HashMap::default();
    let mut writes: HashMap<usize, Vec<Val>> = HashMap::default();
    let mut values: HashMap<Val, Vec<Cell>> = HashMap::default();
    let mut returns = Vec::new();
    let mut parameter_places = Vec::new();
    let mut exposed_parameters = HashSet::default();
    for event in body.blocks.iter().flat_map(|b| &b.events) {
        match *event {
            Event::Parameter {
                formal,
                place,
                ty,
                ..
            } => {
                let p = places.place(place, ty);
                parameters.insert(places.project(p, Projection::Deref), formal);
                parameter_places.push(p);
            }
            Event::Store {
                place,
                value,
                ty,
                ..
            } => {
                let p = places.place(place, ty);
                writes.entry(p).or_default().push(value);
            }
            Event::Load {
                place,
                value,
                ty,
                ..
            } => {
                let p = places.place(place, ty);
                values.entry(value).or_default().push(Cell::Place(p));
            }
            Event::Address {
                place,
                ty,
                ..
            } => {
                exposed_parameters.insert(places.place(place, ty));
            }
            Event::Copy {
                value,
                source,
            } => {
                values.entry(value).or_default().push(Cell::Value(source));
            }
            Event::Return {
                value,
                index: i,
            } if i == index => returns.push(value),
            _ => {}
        }
    }
    if parameter_places
        .iter()
        .any(|p| exposed_parameters.contains(p) || writes.get(p).is_none_or(|w| w.len() != 1))
    {
        return None;
    }
    fn trace(
        key: Cell,
        places: &Places,
        parameters: &HashMap<usize, usize>,
        writes: &HashMap<usize, Vec<Val>>,
        values: &HashMap<Val, Vec<Cell>>,
        visiting: &mut HashSet<Cell>,
    ) -> Option<usize> {
        if !visiting.insert(key) {
            return None;
        }
        let result = match key {
            Cell::Place(p) if places.data.contains_key(&p) => {
                parameters.get(&places.data[&p].0).copied()
            }
            Cell::Place(p) => match writes.get(&p)?.as_slice() {
                [v] => trace(
                    Cell::Value(*v),
                    places,
                    parameters,
                    writes,
                    values,
                    visiting,
                ),
                _ => None,
            },
            Cell::Value(v) => match values.get(&v)?.as_slice() {
                [key] => trace(*key, places, parameters, writes, values, visiting),
                _ => None,
            },
        };
        visiting.remove(&key);
        result
    }
    let mut formal = None;
    for value in returns {
        let result = trace(
            Cell::Value(value),
            &places,
            &parameters,
            &writes,
            &values,
            &mut HashSet::default(),
        )?;
        if formal.is_some_and(|f| f != result) {
            return None;
        }
        formal = Some(result);
    }
    // Deliberately restricted to the existing growth contract in this step. SCC summaries,
    // including projected returns and unknown callees, replace this local proof in step 4.
    formal
}

fn build(
    cx: &Cx,
    body: &LifetimeBody,
    places: &mut Places,
    cells: &mut Cells,
    returns: &mut HashMap<(ProcId, usize), Option<usize>>,
) -> Vec<Block> {
    let mut grown = HashMap::default();
    let mut blocks: Vec<_> = body
        .blocks
        .iter()
        .map(|_| Block {
            op: Op::Nop,
            next: Vec::new(),
        })
        .collect();
    for (i, block) in body.blocks.iter().enumerate() {
        let mut previous = i;
        for event in &block.events {
            let mut ops = Vec::new();
            match *event {
                Event::ReadMemory {
                    value,
                    span,
                } => ops.push(Op::Use(cells.id(Cell::Value(value)), span)),
                Event::Use {
                    place,
                    ty,
                    span,
                } => {
                    let p = places.place(place, ty);
                    ops.push(Op::Use(cells.id(Cell::Place(p)), span));
                }
                Event::Borrow {
                    value,
                    region,
                    region_ty,
                    span,
                } => {
                    let r = places.place(region, region_ty);
                    let target = cells.id(Cell::Value(value));
                    if resizable(cx, places, r) {
                        ops.push(Op::Borrow(target, r, span));
                    } else {
                        ops.push(Op::Copy(vec![(target, cells.id(Cell::Place(r)))]));
                    }
                }
                Event::Load {
                    place,
                    value,
                    ty,
                    ..
                } => {
                    let p = places.place(place, ty);
                    let target = cells.id(Cell::Value(value));
                    if let Some(&(r, at)) = places.data.get(&p) {
                        if resizable(cx, places, r) {
                            ops.push(Op::Borrow(target, r, at));
                        } else {
                            ops.push(Op::Copy(vec![(target, cells.id(Cell::Place(r)))]));
                        }
                    } else {
                        ops.push(Op::Copy(vec![(target, cells.id(Cell::Place(p)))]));
                    }
                }
                Event::Store {
                    place,
                    value,
                    ty,
                    ..
                } => {
                    let p = places.place(place, ty);
                    let source = places.place(value, ty);
                    if places.unresolved(p) {
                        ops.push(Op::Barrier);
                    }
                    ops.push(Op::Replace(p));
                    if let Some(&r) = places.descriptor_fields.get(&p) {
                        ops.push(Op::Replace(r));
                    }
                    let target = cells.id(Cell::Place(p));
                    if view(cx, ty) || cx.compiler.ty.types.is_pointer(ty) {
                        ops.push(Op::Copy(vec![(target, cells.id(Cell::Value(value)))]));
                    } else {
                        let mut copies = vec![(target, cells.id(Cell::Place(source)))];
                        for (dst, src) in places.copied_fields(p, source) {
                            copies.push((cells.id(Cell::Place(dst)), cells.id(Cell::Place(src))));
                        }
                        ops.push(Op::Copy(copies));
                    }
                    if let Some(&(parent, _)) = places.data.get(&p)
                        && places.types.get(&parent).is_some_and(|&ty| view(cx, ty))
                    {
                        ops.push(Op::Copy(vec![(
                            cells.id(Cell::Place(parent)),
                            cells.id(Cell::Value(value)),
                        )]));
                    }
                }
                Event::Copy {
                    value,
                    source,
                } => ops.push(Op::Copy(vec![(
                    cells.id(Cell::Value(value)),
                    cells.id(Cell::Value(source)),
                )])),
                Event::View {
                    value,
                    source,
                    from,
                    to,
                    span,
                } => {
                    let source = places.place(source, from);
                    let target = cells.id(Cell::Value(value));
                    if resizable(cx, places, source) {
                        ops.push(Op::Borrow(target, source, span));
                    } else {
                        ops.push(Op::Copy(vec![(target, cells.id(Cell::Place(source)))]));
                    }
                    let target_place = places.place(value, to);
                    ops.push(Op::Copy(vec![(
                        cells.id(Cell::Place(target_place)),
                        target,
                    )]));
                }
                Event::Grow {
                    region,
                    ty,
                    span,
                } => {
                    let r = places.place(region, ty);
                    grown.insert(span, r);
                    ops.push(Op::Grow(r, span));
                }
                Event::Call {
                    callee,
                    ref results,
                    span,
                    ..
                } if cx.compiler.proc_has_note(callee, "GrowsArray") => {
                    if let Some(callee_body) = cx
                        .compiler
                        .ed
                        .facts()
                        .and_then(|f| f.lifetime_bodies.get(&callee))
                    {
                        let info = &cx.compiler.pr.procs[callee.0 as usize];
                        if info.body_state != BodyState::Done
                            || !std::sync::Arc::ptr_eq(&info.lit, &callee_body.syntax)
                        {
                            continue;
                        }
                        for &(index, value, _) in results {
                            if *returns
                                .entry((callee, index))
                                .or_insert_with(|| returned_storage(cx, callee_body, index))
                                == Some(0)
                                && let Some(&r) = grown.get(&span)
                            {
                                ops.push(Op::Borrow(cells.id(Cell::Value(value)), r, span));
                            }
                        }
                    }
                }
                Event::Barrier => ops.push(Op::Barrier),
                _ => {}
            }
            for op in ops {
                let index = blocks.len();
                blocks.push(Block {
                    op,
                    next: Vec::new(),
                });
                blocks[previous].next.push(index);
                previous = index;
            }
        }
        blocks[previous].next.clone_from(&block.successors);
    }
    blocks
}

pub(super) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    let Some(facts) = cx.compiler.ed.facts() else {
        return;
    };
    let mut cfgs = 0;
    let mut solved = 0;
    let mut bodies = 0;
    let mut rejected = BTreeMap::<&str, usize>::new();
    let mut rejected_candidates = BTreeMap::<&str, usize>::new();
    for site in &cx.procs {
        for &id in cx.compiler.ide_procs_with_header(site.header.span).iter() {
            let proc = &cx.compiler.pr.procs[id.0 as usize];
            if proc.is_poly || proc.is_macro {
                continue;
            }
            bodies += 1;
            if !facts.lifetime_bodies.contains_key(&id) {
                let reason = if proc.body_state == BodyState::Done {
                    "missing-sidecar"
                } else {
                    "failed-or-unchecked"
                };
                *rejected.entry(reason).or_default() += 1;
            }
        }
    }
    let mut bytes = 0;
    let mut returns = HashMap::default();
    // Hash iteration order must not affect the order of diagnostics.
    let mut selected: Vec<_> = facts.lifetime_bodies_in(cx.file).collect();
    selected.sort_by_key(|b| b.procedure.0);
    for body in selected {
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
        let mut places = Places::new(cx.compiler, body);
        let mut cells = Cells::default();
        let blocks = build(cx, body, &mut places, &mut cells, &mut returns);
        let mut names = HashMap::default();
        let mut allowed_addresses = HashSet::default();
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
                    callee,
                    args,
                    ..
                } if cx.compiler.proc_has_note(*callee, "GrowsArray") => {
                    for arg in args.iter().filter(|a| a.formal == 0) {
                        if let LifetimeOperand::Value(place) = arg.operand {
                            allowed_addresses.insert((place, arg.span));
                        }
                    }
                }
                _ => {}
            }
        }
        let exposed: HashSet<_> = body
            .blocks
            .iter()
            .flat_map(|b| &b.events)
            .filter_map(|event| match event {
                Event::Address {
                    place,
                    ty,
                    span,
                } if !allowed_addresses.contains(&(*place, *span)) => {
                    Some(places.place(*place, *ty))
                }
                _ => None,
            })
            .collect();
        cfgs += 1;
        let mut forward = vec![Vec::new(); cells.ids.len()];
        for block in &blocks {
            if let Op::Copy(ref pairs) = block.op {
                for &(target, source) in pairs {
                    forward[source].push(target);
                }
            }
        }
        let model = Model {
            places,
            cells,
            exposed,
            blocks,
            forward,
        };
        for (origin, block) in model.blocks.iter().enumerate() {
            if let Op::Borrow(_, region, _) = block.op {
                if model.places.unresolved(region) {
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
                        Projection::Deref => {}
                    }
                }
                solved += usize::from(solve(cx, &model, origin, &name, out));
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
            "lifetime-stats file={} bodies={bodies} cfgs={cfgs} candidates={solved} sidecar_bytes={bytes} rejected={:?} rejected_candidates={:?}",
            cx.compiler.src.sources.get(cx.file).path,
            rank(rejected),
            rank(rejected_candidates)
        );
    }
}

fn solve(
    cx: &Cx,
    model: &Model,
    candidate: usize,
    region_name: &str,
    out: &mut Vec<Finding>,
) -> bool {
    let Model {
        blocks,
        places,
        cells,
        exposed,
        forward,
    } = model;
    let Op::Borrow(first, region, origin) = blocks[candidate].op else {
        return false;
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
    if !blocks
        .iter()
        .any(|b| matches!(b.op, Op::Use(cell, _) if relevant.contains_key(&cell)))
    {
        return false;
    }
    let exposed_cells: HashSet<_> = cells
        .ids
        .iter()
        .filter_map(|(key, &cell)| match key {
            Cell::Place(p)
                if places.unresolved(*p) || exposed.iter().any(|&a| places.contains(a, *p)) =>
            {
                relevant.get(&cell).copied()
            }
            _ => None,
        })
        .collect();
    let mut states: Vec<Option<Vec<u8>>> = vec![None; blocks.len()];
    states[0] = Some(vec![UNKNOWN; relevant.len()]);
    let mut queue = VecDeque::from([0]);
    while let Some(i) = queue.pop_front() {
        let mut state = states[i].clone().unwrap();
        match &blocks[i].op {
            Op::Borrow(target, _, _) => {
                if let Some(&cell) = relevant.get(target) {
                    state[cell] = if i == candidate {
                        LIVE
                    } else {
                        UNKNOWN
                    };
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
                        | if *cell & (LIVE | UNKNOWN) != 0 {
                            UNKNOWN
                        } else {
                            0
                        };
                }
            }
            Op::Barrier => state.fill(UNKNOWN),
            Op::Grow(r, _) if *r == region => {
                for cell in &mut state {
                    if *cell & LIVE != 0 {
                        *cell = (*cell & !LIVE) | DEAD;
                    }
                }
            }
            _ => {}
        }
        for &cell in &exposed_cells {
            state[cell] = UNKNOWN;
        }
        for &next in &blocks[i].next {
            let changed = if let Some(previous) = &mut states[next] {
                let mut changed = false;
                for (before, &now) in previous.iter_mut().zip(&state) {
                    let join = *before | now;
                    changed |= join != *before;
                    *before = join;
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
    for (i, block) in blocks.iter().enumerate() {
        let Op::Use(id, at) = block.op else {
            continue;
        };
        if at.file != cx.file {
            continue;
        }
        let Some(&cell) = relevant.get(&id) else {
            continue;
        };
        if states[i].as_ref().is_none_or(|s| s[cell] != DEAD) {
            continue;
        }
        let growths = invalidation_cut(blocks, &states, &relevant, i, cell, region);
        if growths.is_empty() {
            continue;
        }
        let explanation = if growths.len() == 1 {
            format!(
                "`{}` points into `{region_name}`, which may be resized: {}",
                cx.src(at),
                cx.compiler.src.sources.snippet(growths[0]).trim()
            )
        } else {
            format!(
                "`{}` points into `{region_name}`; every path to this use passes one of {} invalidating calls",
                cx.src(at),
                growths.len()
            )
        };
        let mut report = finding(
            at,
            explanation,
            Some("take the pointer again after growth, or store an index instead".into()),
        );
        report.label = Some("pointer used here after its storage was invalidated".into());
        report
            .secondary
            .push((origin, "pointer derived here".into()));
        for growth in growths {
            report
                .secondary
                .push((growth, format!("`{region_name}` storage invalidated here")));
        }
        if !out.iter().any(|f| {
            f.start >= at.start as usize
                && f.end <= at.end as usize
                && f.secondary == report.secondary
        }) {
            out.push(report);
        }
    }
    true
}

/// Follow the used value backwards through simultaneous copies. A later grow of a rebound
/// descriptor cannot explain a pointer that was already dead: only LIVE -> DEAD transitions
/// contribute invalidators. Witness storage is allocated on demand, outside the fixed point.
fn invalidation_cut(
    blocks: &[Block],
    states: &[Option<Vec<u8>>],
    relevant: &HashMap<usize, usize>,
    use_at: usize,
    use_cell: usize,
    region: usize,
) -> Vec<Span> {
    let mut incoming = vec![Vec::new(); blocks.len()];
    for (from, block) in blocks
        .iter()
        .enumerate()
        .filter(|(i, _)| states[*i].is_some())
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
            Op::Grow(id, at) if *id == region => {
                let before = states[i].as_ref().unwrap()[cell];
                if before & LIVE != 0 {
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
            Op::Borrow(target, _, _) if relevant.get(target) == Some(&cell) => continue,
            Op::Barrier => continue,
            _ => {}
        }
        queue.extend(incoming[i].iter().map(|&p| (p, cell)));
    }
    spans.sort_unstable_by_key(|s| (s.file.0, s.start, s.end));
    spans.dedup();
    spans
}
