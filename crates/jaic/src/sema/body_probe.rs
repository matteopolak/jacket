//! `JAIC_BODY_PROBE=1`: which shared tables does each procedure body mutate while it is lowered?
//!
//! A study aid for `docs/compiler/parallel-sema.md`. Around every `lower_body` it snapshots the
//! sizes of the compiler's shared tables, charges the growth to the body that caused it (growth
//! made by bodies lowered on demand inside it is charged to those), and prints a histogram when
//! the compile ends. It costs nothing when the variable is unset.
use super::Compiler;
use std::cell::RefCell;
use std::time::{Duration, Instant};

/// What a counter measures; `SETS` groups them by how hard they are to make concurrent.
#[derive(Clone, Copy)]
pub enum C {
    Types,
    Structs,
    Strings,
    Packs,
    ConstAddr,
    TypeInfos,
    AnonTypes,
    Procs,
    Funcs,
    AnonProcs,
    Globals,
    Foreigns,
    Modules,
    Files,
    NamespaceGen,
    CtExec,
    EntResolve,
    SigResolve,
    Layout,
    Nested,
    Entities,
    Scopes,
    Warnings,
}

const N: usize = 23;

const NAMES: [&str; N] = [
    "types",
    "structs",
    "strings",
    "packs",
    "const_addr",
    "type_infos",
    "anon_types",
    "procs",
    "funcs",
    "anon_procs",
    "globals",
    "foreigns",
    "modules",
    "files",
    "namespace_gen",
    "ct_exec",
    "entity_resolve",
    "signature_resolve",
    "struct_layout",
    "nested_body",
    "entities(arena)",
    "scopes(arena)",
    "warnings",
];

/// A: compile-time execution and module/namespace changes (inherently serial).
/// B: interning (types, strings, constants, descriptors).
/// C: lazy resolution of declarations that already exist.
/// D: new procedures or functions.
/// E: globals/foreigns (new program-wide storage).
const SETS: [(&str, &[C]); 5] = [
    (
        "A serial (ct_exec,modules,files,namespace_gen)",
        &[C::CtExec, C::Modules, C::Files, C::NamespaceGen],
    ),
    (
        "B interning (types,structs,strings,packs,const_addr,type_infos,anon_types)",
        &[
            C::Types,
            C::Structs,
            C::Strings,
            C::Packs,
            C::ConstAddr,
            C::TypeInfos,
            C::AnonTypes,
        ],
    ),
    (
        "C lazy resolution (entity,signature,layout,nested body)",
        &[C::EntResolve, C::SigResolve, C::Layout, C::Nested],
    ),
    (
        "D new procs (procs,funcs,anon_procs)",
        &[C::Procs, C::Funcs, C::AnonProcs],
    ),
    (
        "E new globals (globals,foreigns)",
        &[C::Globals, C::Foreigns],
    ),
];

struct Frame {
    start: [u64; N],
    child: [u64; N],
    at: Instant,
    child_time: Duration,
    id: u32,
}

#[derive(Default)]
struct State {
    bumps: [u64; 3],
    frames: Vec<Frame>,
    bodies: u64,
    touched: [u64; N],
    /// Per mask of touched sets: bodies, self time.
    masks: [(u64, Duration); 32],
    total: Duration,
    /// Body that was being lowered when each body was queued (`u32::MAX`: none), and each
    /// body's own lowering time: the queue forms a tree whose longest path bounds any speedup.
    parent: std::collections::HashMap<u32, u32>,
    own_time: std::collections::HashMap<u32, Duration>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

pub fn enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("JAIC_BODY_PROBE").is_some())
}

/// Count one lazy resolution (`C::EntResolve`, `C::SigResolve` or `C::Layout`).
pub fn bump(c: C) {
    if enabled() {
        STATE.with(|s| s.borrow_mut().bumps[c as usize - C::EntResolve as usize] += 1);
    }
}

fn snapshot(c: &Compiler, bumps: [u64; 3], nested: u64) -> [u64; N] {
    let mut v = [0u64; N];
    v[C::Types as usize] = c.types.type_count() as u64;
    v[C::Structs as usize] = c.types.structs.len() as u64;
    v[C::Strings as usize] = c.strings.len() as u64;
    v[C::Packs as usize] = c.packs.len() as u64;
    v[C::ConstAddr as usize] = c.const_addr_globals.len() as u64;
    v[C::TypeInfos as usize] = c.type_infos.len() as u64;
    v[C::AnonTypes as usize] = c.anonymous_types.len() as u64;
    v[C::Procs as usize] = c.procs.len() as u64;
    v[C::Funcs as usize] = c.program.funcs.len() as u64;
    v[C::AnonProcs as usize] = c.anonymous_procs.len() as u64;
    v[C::Globals as usize] = c.program.globals.len() as u64;
    v[C::Foreigns as usize] = c.program.foreigns.len() as u64;
    v[C::Modules as usize] = c.modules.len() as u64;
    v[C::Files as usize] = c.files.len() as u64;
    v[C::NamespaceGen as usize] = c.namespace_generation;
    v[C::CtExec as usize] = c.ct_executions;
    v[C::EntResolve as usize] = bumps[0];
    v[C::SigResolve as usize] = bumps[1];
    v[C::Layout as usize] = bumps[2];
    v[C::Nested as usize] = nested;
    v[C::Entities as usize] = c.entities.len() as u64;
    v[C::Scopes as usize] = c.scopes.len() as u64;
    v[C::Warnings as usize] = c.warnings.len() as u64;
    v
}

/// A body was queued by whatever body is being lowered now.
pub fn enqueued(id: u32) {
    if enabled() {
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            let p = s.frames.last().map_or(u32::MAX, |f| f.id);
            s.parent.entry(id).or_insert(p);
        });
    }
}

pub fn begin(c: &Compiler, id: u32) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.bodies += 1;
        let start = snapshot(c, s.bumps, s.bodies);
        s.frames.push(Frame {
            start,
            child: [0; N],
            at: Instant::now(),
            child_time: Duration::ZERO,
            id,
        });
    });
}

pub fn end(c: &Compiler) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let bumps = s.bumps;
        // `bodies` already counts this one, which `begin` snapshotted before incrementing.
        let now = snapshot(c, bumps, s.bodies);
        let f = s.frames.pop().unwrap();
        let elapsed = f.at.elapsed();
        let self_time = elapsed.saturating_sub(f.child_time);
        let mut total = [0u64; N];
        let mut own = [0u64; N];
        for i in 0..N {
            total[i] = now[i].saturating_sub(f.start[i]);
            own[i] = total[i].saturating_sub(f.child[i]);
        }
        // This body itself counts as nested for its parent, once.
        if let Some(p) = s.frames.last_mut() {
            for i in 0..N {
                p.child[i] += total[i];
            }
            p.child_time += elapsed;
        }
        let mut mask = 0usize;
        for (k, (_, set)) in SETS.iter().enumerate() {
            if set.iter().any(|&c| own[c as usize] > 0) {
                mask |= 1 << k;
            }
        }
        for i in 0..N {
            if own[i] > 0 {
                s.touched[i] += 1;
            }
        }
        s.masks[mask].0 += 1;
        s.masks[mask].1 += self_time;
        s.total += self_time;
        *s.own_time.entry(f.id).or_default() += self_time;
    });
}

pub fn report() {
    if !enabled() {
        return;
    }
    STATE.with(|s| {
        let s = s.borrow();
        eprintln!("== body probe: {} lower_body calls (self time {:?})", s.bodies, s.total);
        eprintln!("-- bodies whose own lowering grew each table:");
        for i in 0..N {
            eprintln!("  {:<20} {:>8} ({:.1}%)", NAMES[i], s.touched[i], 100.0 * s.touched[i] as f64 / s.bodies.max(1) as f64);
        }
        eprintln!("-- by sets touched (A..E, see body_probe.rs):");
        let mut rows: Vec<_> = (0..32).filter(|&m| s.masks[m].0 > 0).collect();
        rows.sort_by_key(|&m| std::cmp::Reverse(s.masks[m].1));
        for m in rows {
            let label: String = SETS
                .iter()
                .enumerate()
                .filter(|(k, _)| m & (1 << k) != 0)
                .map(|(_, (n, _))| n.chars().next().unwrap())
                .collect();
            let label = if label.is_empty() { "-(pure)".into() } else { label };
            eprintln!(
                "  {:<8} {:>8} bodies ({:.1}%)  self {:>10.3?} ({:.1}%)",
                label,
                s.masks[m].0,
                100.0 * s.masks[m].0 as f64 / s.bodies.max(1) as f64,
                s.masks[m].1,
                100.0 * s.masks[m].1.as_secs_f64() / s.total.as_secs_f64().max(1e-12)
            );
        }
        // Longest chain of queue parents, weighted by own time.
        let mut best = Duration::ZERO;
        let mut deepest = 0;
        let mut memo: std::collections::HashMap<u32, (Duration, u32)> = Default::default();
        for &id in s.parent.keys() {
            let mut chain = vec![];
            let mut cur = id;
            let base = loop {
                if let Some(&m) = memo.get(&cur) {
                    break m;
                }
                chain.push(cur);
                match s.parent.get(&cur) {
                    Some(&p) if p != u32::MAX => cur = p,
                    _ => break (Duration::ZERO, 0),
                }
            };
            let (mut t, mut d) = base;
            for &n in chain.iter().rev() {
                t += s.own_time.get(&n).copied().unwrap_or_default();
                d += 1;
                memo.insert(n, (t, d));
            }
            let (t, d) = memo[&id];
            if t > best {
                best = t;
            }
            deepest = deepest.max(d);
        }
        let roots = s.parent.values().filter(|&&p| p == u32::MAX).count();
        eprintln!(
            "-- queue tree: {} queued bodies, {roots} roots, depth {deepest}, critical path {best:?} of {:?} total",
            s.parent.len(),
            s.total
        );
        for (name, _) in SETS {
            eprintln!("  set {name}");
        }
    });
}

/// The next queued body: the last queued, or with `JAIC_BODY_ORDER=fifo` the first (a study of
/// how much the numbering of ids and functions depends on the order bodies are lowered in).
pub fn next_body(queue: &mut Vec<super::ProcId>) -> Option<super::ProcId> {
    static FIFO: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *FIFO.get_or_init(|| std::env::var("JAIC_BODY_ORDER").is_ok_and(|v| v == "fifo")) {
        (!queue.is_empty()).then(|| queue.remove(0))
    } else {
        queue.pop()
    }
}
