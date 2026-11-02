//! Lifetime CFG built from the optional checker's per-instantiation semantic sidecar.
//! Expansion, cleanup and jump edges are the selected lowering edges, never a second
//! syntactic interpretation. Joins union states; only a sole DEAD state is a must proof.
use super::finding;
use crate::Finding;
use crate::syntax::Cx;
use jaic::fxhash::{HashMap, HashSet};
use jaic::ide::lifetimes::{LifetimeEvent as Event, LifetimeOperand};
use jaic::ir::Val;
use jaic::sema::procs::BodyState;
use jaic::source::Span;
use std::collections::{BTreeMap, VecDeque};

#[derive(Clone, Copy)]
enum Op {
    Nop,
    Use(Val, Span),
    Set(Val, Option<Val>, Span),
    Grow(Val, Span),
    Barrier,
}

struct Block {
    op: Op,
    next: Vec<usize>,
}

const UNKNOWN: u8 = 1;
const LIVE: u8 = 2;
const DEAD: u8 = 4;

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
        let events = || body.blocks.iter().flat_map(|b| &b.events);
        let mut names = HashMap::default();
        let mut allowed_addresses = HashSet::default();
        for event in events() {
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
        let exposed: HashSet<_> = events()
            .filter_map(|event| match event {
                Event::Address {
                    place,
                    span,
                } if !allowed_addresses.contains(&(*place, *span)) => Some(*place),
                _ => None,
            })
            .collect();
        // One entry node per semantic block; event nodes preserve checking order within it.
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
                let op = match *event {
                    Event::Use {
                        place,
                        span,
                    } => Op::Use(place, span),
                    Event::Set {
                        place,
                        source,
                        span,
                    } => Op::Set(place, source.map(|s| s.0), source.map_or(span, |s| s.1)),
                    Event::Grow {
                        region,
                        span,
                    } => Op::Grow(region, span),
                    Event::Barrier => Op::Barrier,
                    _ => continue,
                };
                let index = blocks.len();
                blocks.push(Block {
                    op,
                    next: Vec::new(),
                });
                blocks[previous].next.push(index);
                previous = index;
            }
            blocks[previous].next.clone_from(&block.successors);
        }
        cfgs += 1;
        let mut candidates = Vec::new();
        for block in &blocks {
            if let Op::Set(pointer, Some(region), origin) = block.op {
                if exposed.contains(&pointer) || exposed.contains(&region) {
                    *rejected_candidates.entry("address-exposed").or_default() += 1;
                } else if !candidates.contains(&(pointer, region, origin)) {
                    candidates.push((pointer, region, origin));
                }
            }
        }
        for (pointer, region, origin) in candidates {
            solved += 1;
            solve(
                cx,
                &blocks,
                pointer,
                region,
                origin,
                names.get(&region).copied().unwrap_or("storage"),
                out,
            );
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
    blocks: &[Block],
    pointer: Val,
    region: Val,
    origin: Span,
    region_name: &str,
    out: &mut Vec<Finding>,
) {
    let mut states = vec![0u8; blocks.len()];
    states[0] = UNKNOWN;
    let mut queue = VecDeque::from([0]);
    while let Some(i) = queue.pop_front() {
        let mut state = states[i];
        match blocks[i].op {
            Op::Set(id, r, at) if id == pointer => {
                state = if r == Some(region) && at == origin {
                    LIVE
                } else {
                    UNKNOWN
                };
            }
            Op::Set(id, _, _) if id == region || id == pointer => {
                state = UNKNOWN;
            }
            Op::Barrier => {
                state = UNKNOWN;
            }
            Op::Grow(id, _) if id == region && state & LIVE != 0 => {
                state = (state & !LIVE) | DEAD;
            }
            _ => {}
        }
        for &next in &blocks[i].next {
            let joined = states[next] | state;
            if joined != states[next] {
                states[next] = joined;
                queue.push_back(next);
            }
        }
    }
    for (i, b) in blocks.iter().enumerate() {
        if states[i] == DEAD
            && let Op::Use(id, at) = b.op
            && id == pointer
        {
            if at.file != cx.file {
                continue;
            }
            let growths = invalidation_cut(blocks, &states, i, region);
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
            out.push(report);
        }
    }
}


/// Collect the last invalidator on every reachable incoming path. The DEAD proof is
/// established by the lattice first; this traversal only explains it. Allocate predecessors
/// and visited bits on demand for a diagnostic, rather than witness sets at every CFG node.
fn invalidation_cut(blocks: &[Block], states: &[u8], use_at: usize, region: Val) -> Vec<Span> {
    let mut incoming = vec![Vec::new(); blocks.len()];
    for (from, block) in blocks.iter().enumerate().filter(|(i, _)| states[*i] != 0) {
        for &to in &block.next {
            incoming[to].push(from);
        }
    }
    let mut seen = vec![false; blocks.len()];
    let mut queue = VecDeque::from([use_at]);
    let mut spans = Vec::new();
    while let Some(i) = queue.pop_front() {
        if std::mem::replace(&mut seen[i], true) {
            continue;
        }
        if let Op::Grow(id, at) = blocks[i].op
            && id == region
        {
            spans.push(at);
        } else {
            queue.extend(incoming[i].iter().copied());
        }
    }
    spans.sort_unstable_by_key(|s| (s.file.0, s.start, s.end));
    spans.dedup();
    spans
}
