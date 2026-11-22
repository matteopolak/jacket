//! `freestanding`: code that cannot work, or misbehaves, with no operating system.
//!
//! ```jai
//! main :: () {
//!     names: [..] string;
//!     array_add(*names, "ada");   // the default allocator has no heap on `OS == .NONE`
//! }
//! ```
//!
//! A program for a microcontroller has no operating system, no heap unless it brings one and a
//! small fixed stack. Most of what breaks there compiles fine for the machine that is linted,
//! so the rule reads the code for it. Five things are reported:
//!
//! - **OS-only modules.** An `#import` of a module that asserts the systems it builds for
//!   (`#assert OS == .LINUX || OS == .MACOS`) without `NONE`. The systems are the ones
//!   `jacket doc` lists for the module, so there is no list to keep. An import under `#if` is
//!   left alone.
//! - **Allocation through the default allocator.** `alloc`, `New`, `NewArray`, `array_add` and
//!   `tprint` reached from `main`, a `#program_export` procedure or an `@Interrupt` procedure
//!   on a path with no `push_allocator` or `push_context` (one that names a context) around
//!   the call. A call into another file is followed, and the finding is at the call in this
//!   file that leads to it.
//! - **Interrupt handlers that do too much.** Inside an `@Interrupt` procedure and everything
//!   it calls: allocation (even with an allocator pushed), `print`, `write_string`, `log`,
//!   locks and calls that may block (`sleep_milliseconds`, `wait`, `wait_for`).
//! - **Unbounded stack.** A procedure that is part of a call cycle, and a procedure whose
//!   locals add up to more than `[lint.freestanding] max_frame_bytes` (1024 by default).
//! - **Work the CPU cannot do.** `float64` arithmetic, or `s64` and `u64` division in a loop,
//!   when the linted target is freestanding and its CPU has no double-precision float unit or
//!   no 64-bit divide. The only freestanding CPU today, AArch64, has both, so this part never
//!   fires yet.
//!
//! Calls are followed through procedures of the program, in every file, and stop at the
//! standard library: what it does inside is its own business, and the rule names only the
//! calls above. Indirect calls (through a procedure value) are not followed. A library file
//! with no `main`, exported procedure or interrupt handler of its own has no path to follow, so
//! only its imports, recursion and frames are checked. Macros and generated files are left
//! alone. No fix. Off by default (`allow`, category `restriction`): it is for code that is
//! meant to run without an operating system. Turn it on with `-W freestanding` or
//! `[lint.rules] freestanding = "warn"`.
use super::{callee_procs, finding, has_note, large_stack_array, proc_label, statement_lists};
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::walk::{Node, walk};
use jaic::ast::{AssignOp, BinOp, Block, ExprKind as E, ImportSource, Stmt, StmtKind as S};
use jaic::fxhash::{HashMap, HashSet};
use jaic::sema::ProcId;
use jaic::source::Span;
use jaic::target::{TargetCpu, TargetOs};
use jaic::types::TypeId;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "freestanding",
    category: crate::config::Category::Restriction,
    summary: "code that cannot work, or misbehaves, with no operating system or on a microcontroller",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() {
        return;
    }
    let mut report = Report {
        out,
        seen: HashSet::default(),
    };
    imports(cx, &mut report);
    let graph = Graph::new(cx);
    let roots = graph.roots();
    // Handlers first: what is wrong inside one is told as that, not as an allocation.
    graph.reach(&roots.interrupts, Mode::Interrupt, &mut report);
    let entries: Vec<ProcId> = roots
        .entries
        .iter()
        .chain(&roots.interrupts)
        .copied()
        .collect();
    graph.reach(&entries, Mode::Entry, &mut report);
    graph.cycles(&mut report);
    frames(cx, &mut report);
    cpu_work(cx, cpu_gaps(&cx.compiler.options), &mut report);
}

/// Findings, once per place.
struct Report<'o> {
    out: &'o mut Vec<Finding>,
    seen: HashSet<(u32, u32)>,
}

impl Report<'_> {
    fn add(&mut self, span: Span, message: String, help: &str) {
        if self.seen.insert((span.start, span.end)) {
            self.out.push(finding(span, message, Some(help.into())));
        }
    }
}

/// What a call in a library does that a freestanding program may not want.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Alloc,
    Output,
    Lock,
    Block,
}

impl Kind {
    /// The procedure of the standard library `name` in `internal/<dir>/` has this effect.
    fn of(dir: &str, name: &str) -> Option<Kind> {
        Some(match (dir, name) {
            ("basic", "alloc" | "New" | "NewArray" | "array_add" | "tprint") => Kind::Alloc,
            ("basic", "print" | "log" | "log_error")
            | ("runtime" | "runtime-freestanding", "write_string" | "write_strings") => {
                Kind::Output
            }
            ("thread", "lock" | "unlock" | "lock_registry" | "unlock_registry")
            | ("thread", "csection_lock" | "csection_unlock") => Kind::Lock,
            ("thread", "wait" | "wait_for" | "cvariable_wait")
            | ("basic", "sleep_milliseconds") => Kind::Block,
            _ => return None,
        })
    }

    fn does(self) -> &'static str {
        match self {
            Kind::Alloc => "allocates with the default allocator",
            Kind::Output => "writes to the console",
            Kind::Lock => "takes a lock",
            Kind::Block => "may block",
        }
    }
}

/// Which problem a walk down the call graph looks for.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// From `main` and exported procedures: allocation with no allocator pushed.
    Entry,
    /// From an interrupt handler: every kind of call, pushed allocator or not.
    Interrupt,
}

impl Mode {
    fn wants(self, kind: Kind, guarded: bool) -> bool {
        match self {
            Mode::Entry => kind == Kind::Alloc && !guarded,
            Mode::Interrupt => true,
        }
    }
}

/// A call in a procedure body.
#[derive(Clone, Copy)]
struct Edge {
    span: Span,
    callee: ProcId,
    /// A `push_allocator` or `push_context` encloses the call.
    guarded: bool,
}

struct Roots {
    /// `main` and the exported procedures.
    entries: Vec<ProcId>,
    /// The `@Interrupt` procedures.
    interrupts: Vec<ProcId>,
}

struct Graph<'a, 'c> {
    cx: &'c Cx<'a>,
    stdlib: Option<PathBuf>,
    edges: RefCell<HashMap<ProcId, Rc<[Edge]>>>,
}

impl<'a, 'c> Graph<'a, 'c> {
    fn new(cx: &'c Cx<'a>) -> Self {
        Graph {
            cx,
            stdlib: cx
                .compiler
                .options
                .preload_stdlib
                .as_deref()
                .map(|dir| std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf())),
            edges: RefCell::default(),
        }
    }

    fn name(&self, id: ProcId) -> &str {
        self.cx.compiler.proc(id).name.as_str()
    }

    fn path(&self, id: ProcId) -> PathBuf {
        let file = self.cx.compiler.proc(id).span.file;
        self.cx
            .compiler
            .src
            .sources
            .get(file)
            .path
            .to_path()
            .into_owned()
    }

    /// The procedure is part of the standard library.
    fn library(&self, id: ProcId) -> bool {
        let path = self.path(id);
        path.components().any(|c| c.as_os_str() == "internal")
            || self
                .stdlib
                .as_ref()
                .is_some_and(|dir| path.starts_with(dir))
    }

    /// What calling `id` does, when it is one of the library procedures the rule names.
    fn kind(&self, id: ProcId) -> Option<Kind> {
        let path = self.path(id);
        let mut parts = path.components().map(|c| c.as_os_str().to_string_lossy());
        parts.find(|c| c == "internal")?;
        Kind::of(&parts.next()?, self.name(id))
    }

    fn in_file(&self, id: ProcId) -> bool {
        self.cx.compiler.proc(id).span.file == self.cx.file
    }

    /// The calls in the body of `id`, empty when it has none (or no body).
    fn edges(&self, id: ProcId) -> Rc<[Edge]> {
        if let Some(found) = self.edges.borrow().get(&id) {
            return found.clone();
        }
        let mut out = Vec::new();
        if let Some(body) = &self.cx.compiler.proc(id).lit.body {
            let guards = self.guards(body);
            walk(&body.stmts, &mut |n, _| {
                if let Node::Expr(e) = n
                    && let E::Call {
                        callee, ..
                    } = &e.kind
                    && !(e.span.file == self.cx.file && self.cx.in_code_operand(e.span))
                {
                    let guarded = guards
                        .iter()
                        .any(|&(start, end)| start <= e.span.start && e.span.end <= end);
                    for callee in callees(self.cx, callee) {
                        out.push(Edge {
                            span: e.span,
                            callee,
                            guarded,
                        });
                    }
                }
                true
            });
        }
        let out: Rc<[Edge]> = out.into();
        self.edges.borrow_mut().insert(id, out.clone());
        out
    }

    /// The byte ranges of `body` that run with an allocator pushed: what follows a
    /// `push_allocator(...)` in its block, and the body of a `push_context` that names a context.
    fn guards(&self, body: &Block) -> Vec<(u32, u32)> {
        let mut out = Vec::new();
        statement_lists(&body.stmts, &mut |list| {
            if let Some(at) = list.iter().position(|s| self.pushes_allocator(s)) {
                let end = list.last().map_or(0, |s| s.span.end);
                out.push((list[at].span.end, end));
            }
        });
        walk(&body.stmts, &mut |n, _| {
            if let Some(s) = n.stmt()
                && let S::PushContext(push) = &s.kind
                && !push.bare
            {
                out.push((push.body.span.start, push.body.span.end));
            }
            true
        });
        out
    }

    fn pushes_allocator(&self, stmt: &Stmt) -> bool {
        let S::Expr(e) = &stmt.kind else {
            return false;
        };
        let E::Call {
            callee, ..
        } = &e.kind
        else {
            return false;
        };
        callees(self.cx, callee)
            .iter()
            .any(|&p| self.name(p) == "push_allocator" && self.library(p))
    }

    /// The procedures of this file the program starts in.
    fn roots(&self) -> Roots {
        let mut roots = Roots {
            entries: Vec::new(),
            interrupts: Vec::new(),
        };
        for site in self.cx.procs.iter().filter(|p| !p.is_macro) {
            let is_main = site.decl.is_some_and(|d| {
                d.names.first().is_some_and(|n| n.name.as_str() == "main")
                    && self
                        .cx
                        .stmts
                        .iter()
                        .any(|s| matches!(&s.kind, S::Decl(top) if std::ptr::eq(&**top, d)))
            });
            for &id in &*self.cx.procs_with_header(site.header.span) {
                if has_note(self.cx, id, "Interrupt") {
                    roots.interrupts.push(id);
                } else if is_main || self.cx.compiler.proc(id).export.is_some() {
                    roots.entries.push(id);
                }
            }
        }
        roots
    }

    /// Walk the calls from `roots`, reporting the ones `mode` looks for. A call in this file is
    /// reported where it is; one reached through another file, at the call in this file that
    /// leads to it.
    fn reach(&self, roots: &[ProcId], mode: Mode, report: &mut Report) {
        let mut seen = HashSet::default();
        // The procedure, and the call in this file the walk came through (its span and callee).
        let mut stack: Vec<(ProcId, Option<(Span, ProcId)>)> =
            roots.iter().map(|&r| (r, None)).collect();
        while let Some((id, via)) = stack.pop() {
            if !seen.insert(id) {
                continue;
            }
            let here = self.in_file(id);
            for edge in self.edges(id).iter() {
                if let Some(kind) = self.kind(edge.callee) {
                    if mode.wants(kind, edge.guarded) {
                        self.say(report, mode, kind, edge, here.then_some(id), via);
                    }
                    continue;
                }
                if self.library(edge.callee) {
                    continue;
                }
                let next = if here {
                    Some((edge.span, edge.callee))
                } else {
                    via
                };
                stack.push((edge.callee, next));
            }
        }
    }

    fn say(
        &self,
        report: &mut Report,
        mode: Mode,
        kind: Kind,
        edge: &Edge,
        here: Option<ProcId>,
        via: Option<(Span, ProcId)>,
    ) {
        let leaf = self.name(edge.callee);
        let (span, message) = match (here, via) {
            (Some(_), _) => (edge.span, format!("`{leaf}` {}", kind.does())),
            (None, Some((span, first))) => (
                span,
                format!(
                    "`{}` reaches `{leaf}`, which {}",
                    self.name(first),
                    kind.does()
                ),
            ),
            (None, None) => return,
        };
        let (message, help) = match mode {
            Mode::Entry => (
                message,
                "the default allocator has no heap here: wrap the call in `push_allocator(...)`, or `push_context` with an allocator set",
            ),
            Mode::Interrupt => (
                format!("{message}, in an interrupt handler"),
                match kind {
                    Kind::Alloc => "a handler must not allocate: use storage set aside beforehand",
                    Kind::Output => {
                        "a handler must not print: set a flag or a counter and report it from the main loop"
                    }
                    Kind::Lock => {
                        "a handler must not take a lock: the code it interrupts may hold it. Use a flag or an atomic"
                    }
                    Kind::Block => "a handler must return quickly: do not wait in it",
                },
            ),
        };
        report.add(span, message, help);
    }

    /// Recursion: a cycle in the calls of the program, reported once, at the member of it that
    /// is declared first in this file.
    fn cycles(&self, report: &mut Report) {
        let sites: HashMap<ProcId, usize> = self
            .cx
            .procs
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.is_macro)
            .flat_map(|(i, p)| {
                self.cx
                    .procs_with_header(p.header.span)
                    .iter()
                    .map(|&id| (id, i))
                    .collect::<Vec<_>>()
            })
            .collect();
        let mut tarjan = Tarjan {
            graph: self,
            index: HashMap::default(),
            low: HashMap::default(),
            open: Vec::new(),
            on_open: HashSet::default(),
            next: 0,
            groups: Vec::new(),
        };
        let mut starts: Vec<ProcId> = sites.keys().copied().collect();
        starts.sort_by_key(|id| id.0);
        for id in starts {
            tarjan.visit(id);
        }
        for group in tarjan.groups {
            let own = group
                .iter()
                .filter_map(|id| sites.get(id).map(|&i| (i, *id)))
                .min_by_key(|&(i, _)| self.cx.procs[i].header.span.start);
            let Some((site, member)) = own else {
                continue;
            };
            let Some(path) = self.cycle_through(member, &group) else {
                continue;
            };
            let (span, label) = proc_label(&self.cx.procs[site]);
            let message = match path.as_slice() {
                [_] => format!("{label} calls itself, so its stack use has no bound"),
                _ => {
                    let names: Vec<&str> = path.iter().map(|&id| self.name(id)).collect();
                    format!(
                        "{label} is part of a call cycle ({} -> {}), so its stack use has no bound",
                        names.join(" -> "),
                        names[0]
                    )
                }
            };
            report.add(
                span,
                message,
                "rewrite it as a loop, or keep an explicit stack of a fixed size",
            );
        }
    }

    /// The shortest way from `start` back to itself within `group`: the procedures on it.
    fn cycle_through(&self, start: ProcId, group: &[ProcId]) -> Option<Vec<ProcId>> {
        let mut came: HashMap<ProcId, ProcId> = HashMap::default();
        let mut queue = std::collections::VecDeque::from([start]);
        while let Some(id) = queue.pop_front() {
            for edge in self.edges(id).iter() {
                let next = edge.callee;
                if !group.contains(&next) {
                    continue;
                }
                if next == start {
                    let mut path = vec![id];
                    while path.last() != Some(&start) {
                        path.push(came[path.last()?]);
                    }
                    path.reverse();
                    return Some(path);
                }
                if let std::collections::hash_map::Entry::Vacant(slot) = came.entry(next) {
                    slot.insert(id);
                    queue.push_back(next);
                }
            }
        }
        None
    }
}

/// Strongly connected groups of the program's own procedures, by Tarjan's algorithm.
struct Tarjan<'g, 'a, 'c> {
    graph: &'g Graph<'a, 'c>,
    index: HashMap<ProcId, usize>,
    low: HashMap<ProcId, usize>,
    open: Vec<ProcId>,
    on_open: HashSet<ProcId>,
    next: usize,
    /// The groups that are cycles: more than one procedure, or one that calls itself.
    groups: Vec<Vec<ProcId>>,
}

impl Tarjan<'_, '_, '_> {
    fn visit(&mut self, id: ProcId) {
        if self.index.contains_key(&id) {
            return;
        }
        self.index.insert(id, self.next);
        self.low.insert(id, self.next);
        self.next += 1;
        self.open.push(id);
        self.on_open.insert(id);
        let mut calls_itself = false;
        for edge in self.graph.edges(id).iter() {
            let next = edge.callee;
            if self.graph.library(next) || self.graph.cx.compiler.proc(next).lit.body.is_none() {
                continue;
            }
            calls_itself |= next == id;
            if !self.index.contains_key(&next) {
                self.visit(next);
                let low = self.low[&id].min(self.low[&next]);
                self.low.insert(id, low);
            } else if self.on_open.contains(&next) {
                let low = self.low[&id].min(self.index[&next]);
                self.low.insert(id, low);
            }
        }
        if self.low[&id] == self.index[&id] {
            let mut group = Vec::new();
            while let Some(top) = self.open.pop() {
                self.on_open.remove(&top);
                group.push(top);
                if top == id {
                    break;
                }
            }
            if group.len() > 1 || calls_itself {
                self.groups.push(group);
            }
        }
    }
}

/// The procedures a call's callee names: the overload it chose, else every candidate.
fn callees(cx: &Cx, callee: &jaic::ast::Expr) -> Vec<ProcId> {
    let procs = callee_procs(cx, callee);
    let span = match &callee.kind {
        E::Member(_, name) => name.span,
        _ => callee.span,
    };
    match cx.compiler.ide_resolved_overload(span, &procs) {
        Some(chosen) => vec![chosen],
        None => procs.to_vec(),
    }
}

/// `#import "Thread"` and the like, for a module that asserts systems and leaves out `NONE`.
fn imports(cx: &Cx, report: &mut Report) {
    let path = cx
        .compiler
        .src
        .sources
        .get(cx.file)
        .path
        .to_path()
        .into_owned();
    let dir = path
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_default();
    for stmt in cx.stmts {
        let S::Import(import) = &stmt.kind else {
            continue;
        };
        let ImportSource::Module(name) = &import.source else {
            continue;
        };
        if import.is_expression() {
            continue;
        }
        let Some(entry) = cx.compiler.find_module(name, &dir) else {
            continue;
        };
        let systems = jaic::doc_comments::module_platforms(&entry);
        if systems.is_empty() || systems.iter().any(|s| s == "NONE") {
            continue;
        }
        report.add(
            stmt.span,
            format!(
                "`{name}` needs an operating system: it builds only for {}",
                words(&systems)
            ),
            "use a module that works with `OS == .NONE`, or import it under `#if OS != .NONE`",
        );
    }
}

/// `a, b and c`.
fn words(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [head @ .., last] => format!("{} and {last}", head.join(", ")),
    }
}

/// Procedures whose locals add up to more than `max_frame_bytes`.
fn frames(cx: &Cx, report: &mut Report) {
    let types = &cx.compiler.ty.types;
    for site in cx.procs.iter().filter(|p| p.typed()) {
        let mut bytes = 0u64;
        large_stack_array::locals(cx, &site.body.stmts, &mut |_, ty| {
            bytes += types.size_of(ty)
        });
        if bytes as usize <= cx.max_frame_bytes {
            continue;
        }
        let (span, label) = proc_label(site);
        report.add(
            span,
            format!(
                "{label} keeps {} of locals on the stack, past the limit of {}",
                large_stack_array::size(bytes),
                large_stack_array::size(cx.max_frame_bytes as u64)
            ),
            "keep the large locals in globals or a fixed pool, or raise `[lint.freestanding] max_frame_bytes`",
        );
    }
}

/// What the CPU of the linted target cannot do in hardware.
#[derive(Clone, Copy, Default)]
struct CpuGaps {
    /// No double-precision floating point: `float64` arithmetic is a slow software routine.
    double_float: bool,
    /// No 64-bit integer divide: `s64` division is a slow software routine.
    wide_divide: bool,
}

/// The gaps of the CPU the compile is for. Only a freestanding target has any: a hosted one has
/// an operating system to make up for them.
///
/// AArch64 has both. The freestanding 32-bit ARM CPUs are the Cortex-M4 and M33: their FPU is
/// single precision and they have no 64-bit divide, so both are software routines. A core without
/// an FPU (Cortex-M0, M3) is not a target yet. AVR has no FPU or divider at all.
fn cpu_gaps(options: &jaic::sema::Options) -> CpuGaps {
    if options.os != TargetOs::None {
        return CpuGaps::default();
    }
    match options.cpu {
        TargetCpu::Arm | TargetCpu::Avr => CpuGaps {
            double_float: true,
            wide_divide: true,
        },
        TargetCpu::Arm64 | TargetCpu::X64 | TargetCpu::X86 | TargetCpu::S390x | TargetCpu::Wasm => {
            CpuGaps::default()
        }
    }
}

/// `float64` arithmetic and `s64` division in a loop, where `gaps` say the CPU cannot do them.
fn cpu_work(cx: &Cx, gaps: CpuGaps, report: &mut Report) {
    if !gaps.double_float && !gaps.wide_divide {
        return;
    }
    for site in cx.procs.iter().filter(|p| p.typed()) {
        walk(&site.body.stmts, &mut |n, up| {
            let in_loop = up
                .iter()
                .any(|a| matches!(a, Node::Stmt(s) if matches!(s.kind, S::While(_) | S::For(_))));
            let (span, op, ty) = match n {
                Node::Expr(e) => match &e.kind {
                    E::Binary(op, ..) => (e.span, *op, cx.ty(e)),
                    _ => return true,
                },
                Node::Stmt(s) => match &s.kind {
                    S::Assign {
                        op: AssignOp::Op(op),
                        lhs,
                        ..
                    } => (s.span, *op, lhs.first().and_then(|l| cx.ty(l))),
                    _ => return true,
                },
            };
            if gaps.double_float
                && ty == Some(TypeId::F64)
                && matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div)
            {
                report.add(
                    span,
                    "`float64` arithmetic runs in software on this CPU".into(),
                    "use `float32`, or fixed-point integers",
                );
            }
            if gaps.wide_divide
                && in_loop
                && matches!(ty, Some(TypeId::S64 | TypeId::U64))
                && matches!(op, BinOp::Div | BinOp::Rem)
            {
                report.add(
                    span,
                    "64-bit division in a loop runs in software on this CPU".into(),
                    "use a 32-bit type, or shift when the divisor is a power of two",
                );
            }
            true
        });
    }
}
