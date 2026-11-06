//! Bounded neutral execution topology retained at the editor's IR-release boundary.
//! Reachability and lifetime policy belong to the consumer, not this snapshot.
use crate::ast;
use crate::ir::{self, FuncId, GlobalId};
use crate::sema::procs::{BodyState, ProcTarget};
use crate::sema::{Compiler, ModuleId, ProcId};
use crate::target::{TargetCpu, TargetOs};
use std::cell::RefCell;
use std::mem::size_of;
use std::rc::Rc;
use std::sync::Arc;

const MAX_BYTES: usize = 8 * 1024 * 1024;
const MAX_WORK: usize = 8 * 1024 * 1024;

#[derive(Clone, Copy)]
struct Limits {
    bytes: usize,
    work: usize,
}

const LIMITS: Limits = Limits {
    bytes: MAX_BYTES,
    work: MAX_WORK,
};

struct Work {
    used: usize,
    limit: usize,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// A structural function/global identity in this compiler only.
pub enum Node {
    Function(FuncId),
    Global(GlobalId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// The last actual call, including an opaque indirect/foreign call.
pub enum Call {
    Direct(FuncId),
    Other,
}

/// Structural function metadata; the consumer decides which call can be frozen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Function {
    pub procedure: bool,
    pub exported: bool,
    /// Last call of a generated one-block function; never a frozen-return certificate.
    pub generated_single_block_last_call: Option<Call>,
}

fn function(f: &ir::Func) -> Function {
    let generated_single_block_last_call =
        if f.origin != ir::FuncOrigin::Procedure && f.blocks.len() == 1 {
            f.blocks[0].insts.iter().rev().find_map(|inst| match inst {
                ir::Inst::Call(call) => Some(match call.callee {
                    ir::Callee::Func(id) => Call::Direct(id),
                    _ => Call::Other,
                }),
                _ => None,
            })
        } else {
            None
        };
    Function {
        procedure: f.origin == ir::FuncOrigin::Procedure,
        exported: matches!(f.linkage, ir::Linkage::Export(_)),
        generated_single_block_last_call,
    }
}

fn edges(program: &ir::Program, node: Node, mut visit: impl FnMut(Node)) {
    match node {
        Node::Function(id) => {
            if let Some(f) = program.funcs.get(id.0 as usize).and_then(Option::as_ref) {
                for inst in f.blocks.iter().flat_map(|b| &b.insts) {
                    match inst {
                        ir::Inst::Call(call) => {
                            if let ir::Callee::Func(id) = call.callee {
                                visit(Node::Function(id));
                            }
                        }
                        ir::Inst::FuncAddr {
                            func, ..
                        } => visit(Node::Function(*func)),
                        ir::Inst::GlobalAddr {
                            global, ..
                        } => visit(Node::Global(*global)),
                        _ => {}
                    }
                }
            }
        }
        Node::Global(id) => {
            if let Some(g) = program.globals.get(id.0 as usize) {
                for reloc in &g.relocs {
                    match reloc.target {
                        ir::RelocTarget::Func(id) => visit(Node::Function(id)),
                        ir::RelocTarget::Global(id) => visit(Node::Global(id)),
                        ir::RelocTarget::Foreign(_) => {}
                    }
                }
            }
        }
    }
}

fn target(p: Option<ProcTarget>) -> Option<(bool, u32)> {
    p.map(|p| match p {
        ProcTarget::Func(id) => (true, id.0),
        ProcTarget::Foreign(id) => (false, id.0),
    })
}

fn selected_entry_valid(c: &Compiler) -> bool {
    c.pr.entry_point.is_none_or(|id| {
        c.pr.procs
            .get(id.0 as usize)
            .is_some_and(|proc| match proc.target {
                Some(ProcTarget::Func(id)) => c
                    .out
                    .program
                    .funcs
                    .get(id.0 as usize)
                    .is_some_and(Option::is_some),
                Some(ProcTarget::Foreign(id)) => (id.0 as usize) < c.out.program.foreigns.len(),
                None => false,
            })
    })
}

struct Procedure {
    name: crate::intern::Sym,
    syntax: Arc<ast::ProcLit>,
    body: Option<Arc<ast::ProcLit>>,
    target: Option<(bool, u32)>,
    state: BodyState,
    module: ModuleId,
}

pub(crate) struct Snapshot {
    os: TargetOs,
    cpu: TargetCpu,
    host: crate::sema::HostPlatform,
    long_double: Option<crate::wide_float::WideFloat>,
    runtime_support: bool,
    runtime_initialization: bool,
    check_failed: Option<FuncId>,
    entry_name: String,
    runtime_entry: bool,
    main_module: Option<ModuleId>,
    entry: Option<ProcId>,
    procedures: Vec<Procedure>,
    functions: Vec<Option<Function>>,
    globals: Vec<u8>,
    offsets: Vec<usize>,
    edges: Vec<Node>,
    work: usize,
}

fn reserve<T>(len: usize) -> Option<Vec<T>> {
    let mut out = Vec::new();
    out.try_reserve_exact(len).ok()?;
    Some(out)
}

fn charge(work: &mut Work, amount: usize) -> Option<()> {
    work.used = work.used.checked_add(amount)?;
    (work.used <= work.limit && !crate::interrupt::should_stop()).then_some(())
}

impl Snapshot {
    fn capture(c: &Compiler, limits: Limits) -> Option<Self> {
        let p = &c.out.program;
        let facts = c.ed.facts()?;
        if !facts.lint || !facts.refused.is_empty() || !selected_entry_valid(c) {
            return None;
        }
        if p.check_failed
            .is_some_and(|id| !p.funcs.get(id.0 as usize).is_some_and(Option::is_some))
        {
            return None;
        }
        let mut work = Work {
            used: 0,
            limit: limits.work,
        };
        let nodes = p.funcs.len().checked_add(p.globals.len())?;
        charge(
            &mut work,
            nodes
                .checked_add(c.pr.procs.len())?
                .checked_add(facts.lifetime_bodies.len())?,
        )?;
        let mut edge_count = 0usize;
        // Charge all source instructions, not just edges surviving projection.
        for f in p.funcs.iter().flatten() {
            charge(&mut work, f.blocks.len())?;
            for b in &f.blocks {
                charge(&mut work, b.insts.len())?;
            }
            if f.origin != ir::FuncOrigin::Procedure && f.blocks.len() == 1 {
                charge(&mut work, f.blocks[0].insts.len())?;
            }
        }
        for g in &p.globals {
            charge(&mut work, g.relocs.len())?;
        }
        for i in 0..nodes {
            let mut valid = true;
            edges(
                p,
                if i < p.funcs.len() {
                    Node::Function(FuncId(i as u32))
                } else {
                    Node::Global(GlobalId((i - p.funcs.len()) as u32))
                },
                |node| {
                    valid &= match node {
                        Node::Function(id) => {
                            p.funcs.get(id.0 as usize).is_some_and(Option::is_some)
                        }
                        Node::Global(id) => (id.0 as usize) < p.globals.len(),
                    };
                    edge_count = edge_count.saturating_add(1);
                },
            );
            if !valid {
                return None;
            }
        }
        // The two edge walks and identity construction are bounded as well.
        let second_pass = work.used;
        charge(&mut work, second_pass)?;
        let bytes = (size_of::<Self>() + 2 * size_of::<usize>())
            .checked_add(c.pr.procs.len().checked_mul(size_of::<Procedure>())?)?
            .checked_add(p.funcs.len().checked_mul(size_of::<Option<Function>>())?)?
            .checked_add(p.globals.len())?
            .checked_add(nodes.checked_add(1)?.checked_mul(size_of::<usize>())?)?
            .checked_add(edge_count.checked_mul(size_of::<Node>())?)?
            .checked_add(c.options.entry_point.len())?;
        if bytes > limits.bytes {
            return None;
        }
        let mut out = Self {
            os: c.options.os,
            cpu: c.options.cpu,
            host: c.options.host,
            long_double: c.options.long_double,
            runtime_support: c.options.runtime_support,
            runtime_initialization: c.options.runtime_initialization,
            check_failed: p.check_failed,
            entry_name: c.options.entry_point.clone(),
            runtime_entry: c.options.runtime_entry_point,
            main_module: c.ns.main_module,
            entry: c.pr.entry_point,
            procedures: reserve(c.pr.procs.len())?,
            functions: reserve(p.funcs.len())?,
            globals: reserve(p.globals.len())?,
            offsets: reserve(nodes + 1)?,
            edges: reserve(edge_count)?,
            work: work.used,
        };
        for (i, proc) in c.pr.procs.iter().enumerate() {
            let body = facts.lifetime_bodies.get(&ProcId(i as u32));
            if body.is_some()
                && !matches!(proc.target,Some(ProcTarget::Func(id)) if p.funcs.get(id.0 as usize).is_some_and(Option::is_some))
            {
                return None;
            }

            if body.is_some_and(|b| {
                b.procedure != ProcId(i as u32)
                    || proc.body_state != BodyState::Done
                    || !Arc::ptr_eq(&b.syntax, &proc.lit)
            }) {
                return None;
            }
            out.procedures.push(Procedure {
                name: proc.name,
                syntax: proc.lit.clone(),
                body: body.map(|b| b.syntax.clone()),
                target: target(proc.target),
                state: proc.body_state,
                module: c.ns.scopes.get(proc.scope.0 as usize)?.module,
            });
        }
        if facts
            .lifetime_bodies
            .keys()
            .any(|id| id.0 as usize >= c.pr.procs.len())
        {
            return None;
        }
        out.functions
            .extend(p.funcs.iter().map(|f| f.as_ref().map(function)));
        out.globals
            .extend(p.globals.iter().map(|g| u8::from(g.export.is_some())));
        for i in 0..nodes {
            out.offsets.push(out.edges.len());
            edges(
                p,
                if i < p.funcs.len() {
                    Node::Function(FuncId(i as u32))
                } else {
                    Node::Global(GlobalId((i - p.funcs.len()) as u32))
                },
                |node| out.edges.push(node),
            );
        }
        out.offsets.push(out.edges.len());
        (out.owned_bytes() <= limits.bytes && !crate::interrupt::should_stop()).then_some(out)
    }

    fn valid(&self, c: &Compiler) -> bool {
        let Some(facts) = c.ed.facts() else {
            return false;
        };
        if !facts.lint
            || !facts.refused.is_empty()
            || !selected_entry_valid(c)
            || c.options.os != self.os
            || c.options.cpu != self.cpu
            || c.options.host != self.host
            || c.options.long_double != self.long_double
            || c.options.runtime_support != self.runtime_support
            || c.options.runtime_initialization != self.runtime_initialization
            || c.out.program.check_failed != self.check_failed
            || c.options.entry_point != self.entry_name
            || c.options.runtime_entry_point != self.runtime_entry
            || c.ns.main_module != self.main_module
            || c.pr.entry_point != self.entry
            || c.pr.procs.len() != self.procedures.len()
            || c.out.program.funcs.len() != self.functions.len()
            || c.out.program.globals.len() != self.globals.len()
        {
            return false;
        }
        let mut work = Work {
            used: 0,
            limit: MAX_WORK,
        };
        let Some(metadata_work) = self
            .procedures
            .len()
            .checked_add(self.functions.len())
            .and_then(|n| n.checked_add(self.globals.len()))
            .and_then(|n| n.checked_add(facts.lifetime_bodies.len()))
        else {
            return false;
        };
        if charge(&mut work, metadata_work).is_none() {
            return false;
        }
        for (i, (proc, old)) in c.pr.procs.iter().zip(&self.procedures).enumerate() {
            if i % 128 == 0 && crate::interrupt::should_stop() {
                return false;
            }
            if proc.name != old.name
                || !Arc::ptr_eq(&proc.lit, &old.syntax)
                || target(proc.target) != old.target
                || proc.body_state != old.state
                || c.ns
                    .scopes
                    .get(proc.scope.0 as usize)
                    .is_none_or(|scope| scope.module != old.module)
            {
                return false;
            }
            match (facts.lifetime_bodies.get(&ProcId(i as u32)), &old.body) {
                (Some(body), Some(old))
                    if body.procedure == ProcId(i as u32) && Arc::ptr_eq(&body.syntax, old) => {}
                (None, None) => {}
                _ => return false,
            }
        }
        if facts
            .lifetime_bodies
            .keys()
            .any(|id| id.0 as usize >= self.procedures.len())
        {
            return false;
        }
        for (i, (f, old)) in c.out.program.funcs.iter().zip(&self.functions).enumerate() {
            if i % 128 == 0 && crate::interrupt::should_stop() {
                return false;
            }
            match (f, old) {
                (Some(f), Some(old))
                    if f.blocks.is_empty()
                        && f.vals.is_empty()
                        && f.slots.is_empty()
                        && f.trusted.is_empty()
                        && (f.origin == ir::FuncOrigin::Procedure) == old.procedure
                        && matches!(f.linkage, ir::Linkage::Export(_)) == old.exported => {}
                (None, None) => {}
                _ => return false,
            }
        }
        let globals_valid = c
            .out
            .program
            .globals
            .iter()
            .zip(&self.globals)
            .enumerate()
            .all(|(i, (g, exported))| {
                (i % 128 != 0 || !crate::interrupt::should_stop())
                    && g.init.is_empty()
                    && g.relocs.is_empty()
                    && g.export.is_some() == (*exported != 0)
            });
        globals_valid && !crate::interrupt::should_stop()
    }

    fn owned_bytes(&self) -> usize {
        size_of::<Self>()
            + 2 * size_of::<usize>()
            + self.procedures.capacity() * size_of::<Procedure>()
            + self.functions.capacity() * size_of::<Option<Function>>()
            + self.globals.capacity()
            + self.offsets.capacity() * size_of::<usize>()
            + self.edges.capacity() * size_of::<Node>()
            + self.entry_name.capacity()
    }
}

/// A zero-copy view used by the single lifetime execution classifier.
pub struct View<'a>(ViewKind<'a>);

enum ViewKind<'a> {
    Live(&'a ir::Program),
    Retained {
        snapshot: Rc<Snapshot>,
        owner: &'a RefCell<Option<Rc<Snapshot>>>,
    },
}

impl View<'_> {
    /// A lease is usable only while it is still the compiler's current snapshot.
    pub fn available(&self) -> bool {
        match &self.0 {
            ViewKind::Live(_) => true,
            ViewKind::Retained {
                snapshot,
                owner,
            } => owner.try_borrow().is_ok_and(|current| {
                current
                    .as_ref()
                    .is_some_and(|current| Rc::ptr_eq(current, snapshot))
            }),
        }
    }

    pub fn function_count(&self) -> usize {
        match &self.0 {
            ViewKind::Live(p) => p.funcs.len(),
            ViewKind::Retained {
                snapshot: s, ..
            } => s.functions.len(),
        }
    }

    pub fn global_count(&self) -> usize {
        match &self.0 {
            ViewKind::Live(p) => p.globals.len(),
            ViewKind::Retained {
                snapshot: s, ..
            } => s.globals.len(),
        }
    }

    pub fn function(&self, id: FuncId) -> Option<Function> {
        match &self.0 {
            ViewKind::Live(p) => p.funcs.get(id.0 as usize)?.as_ref().map(function),
            ViewKind::Retained {
                snapshot: s, ..
            } => s.functions.get(id.0 as usize).copied().flatten(),
        }
    }

    pub fn global_exported(&self, id: GlobalId) -> bool {
        match &self.0 {
            ViewKind::Live(p) => p
                .globals
                .get(id.0 as usize)
                .is_some_and(|g| g.export.is_some()),
            ViewKind::Retained {
                snapshot: s, ..
            } => s.globals.get(id.0 as usize).is_some_and(|x| *x != 0),
        }
    }

    /// The actual native check handler is an implicit runtime entry, not a named guess.
    pub fn check_handler(&self) -> Option<FuncId> {
        match &self.0 {
            ViewKind::Live(p) => p.check_failed,
            ViewKind::Retained {
                snapshot: s, ..
            } => s.check_failed,
        }
    }

    fn valid_node(&self, node: Node) -> bool {
        match (&self.0, node) {
            (ViewKind::Live(p), Node::Function(id)) => {
                p.funcs.get(id.0 as usize).is_some_and(Option::is_some)
            }
            (
                ViewKind::Retained {
                    snapshot: s, ..
                },
                Node::Function(id),
            ) => s.functions.get(id.0 as usize).is_some_and(Option::is_some),
            (_, Node::Global(id)) => (id.0 as usize) < self.global_count(),
        }
    }

    /// Visits exact structural edges. Invalid IDs fail the whole proof, never disappear.
    pub fn visit_edges(&self, node: Node, mut visit: impl FnMut(Node)) -> bool {
        if !self.available() || !self.valid_node(node) {
            return false;
        }
        let mut valid = true;
        let mut checked = |edge| {
            if valid && self.available() && self.valid_node(edge) {
                visit(edge);
            } else {
                valid = false;
            }
        };
        match &self.0 {
            ViewKind::Live(p) => edges(p, node, &mut checked),
            ViewKind::Retained {
                snapshot: s, ..
            } => {
                let i = match node {
                    Node::Function(id) => id.0 as usize,
                    Node::Global(id) => s.functions.len() + id.0 as usize,
                };
                s.edges[s.offsets[i]..s.offsets[i + 1]]
                    .iter()
                    .copied()
                    .for_each(&mut checked);
            }
        }
        valid && self.available()
    }
}

impl Compiler {
    pub(crate) fn lifetime_capture_execution(&mut self, complete: bool) {
        let retained = if !complete {
            None
        } else if self.out.lowered_released {
            let Some(facts) = self.ed.facts_mut() else {
                return;
            };
            // Validation must precede erasure, or new same-ID IR would disappear.
            let old = facts.lifetime_execution.get_mut().take();
            old.filter(|s| s.valid(self))
        } else {
            Snapshot::capture(self, LIMITS).map(Rc::new)
        };
        if let Some(facts) = self.ed.facts_mut() {
            *facts.lifetime_execution.get_mut() = retained;
        }
    }

    /// Borrow the live topology or an authenticated retained snapshot. Stale data is dropped.
    pub fn lifetime_execution(&self) -> Option<View<'_>> {
        if !self.out.lowered_released {
            return Some(View(ViewKind::Live(&self.out.program)));
        }
        let slot = &self.ed.facts()?.lifetime_execution;
        // Clone only the Rc, then release the slot borrow before validation. An older
        // reader must never prevent removing a cancelled/stale current snapshot.
        let snapshot = slot.try_borrow().ok()?.as_ref()?.clone();
        if !snapshot.valid(self) {
            slot.try_borrow_mut().ok()?.take();
            return None;
        }
        Some(View(ViewKind::Retained {
            snapshot,
            owner: slot,
        }))
    }

    /// Owner-held snapshot/buffer bytes and charged producer work. Retired reader leases
    /// release their shared buffers on drop; this is not process RSS.
    pub fn lifetime_execution_usage(&self) -> (usize, usize) {
        if self.out.lowered_released && self.lifetime_execution().is_none() {
            return (0, 0);
        }
        self.ed
            .facts()
            .and_then(|f| f.lifetime_execution.try_borrow().ok())
            .and_then(|s| s.as_ref().map(|s| (s.owned_bytes(), s.work)))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ide::{IdeFacts, IdeRoot};
    use crate::sema::{NativeFs, Options, ProgramSource};
    use std::rc::Rc;

    #[test]
    fn retained_execution_accounts_and_bounds_actual_owned_storage() {
        let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
        let mut options = Options::host();
        options.import_paths = vec![stdlib.clone()];
        options.set_stdlib(&NativeFs, &stdlib);
        let mut c = Compiler::new(options, Rc::new(NativeFs));
        let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
        facts.lint = true;
        c.ed.record(facts);
        c.compile_sources(&[ProgramSource::String(
            "value :: () -> int {return 7;} VALUE :: #run value(); main :: () {}".into(),
        )])
        .unwrap();
        assert!(c.ide_check_all().is_empty());
        let snapshot = Snapshot::capture(&c, LIMITS).unwrap();
        let bytes = snapshot.owned_bytes();
        eprintln!(
            "compact execution sample: {bytes} owned bytes; {} charged visits",
            snapshot.work
        );
        assert!(bytes > size_of::<Snapshot>() && snapshot.work > 0);
        assert!(
            Snapshot::capture(
                &c,
                Limits {
                    bytes: bytes - 1,
                    work: MAX_WORK
                }
            )
            .is_none()
        );
        assert!(
            Snapshot::capture(
                &c,
                Limits {
                    bytes,
                    work: MAX_WORK
                }
            )
            .is_some()
        );
        assert!(
            Snapshot::capture(
                &c,
                Limits {
                    bytes: MAX_BYTES,
                    work: 0
                }
            )
            .is_none()
        );
        c.out.program.globals[0].relocs.push(ir::Reloc {
            offset: 0,
            target: ir::RelocTarget::Global(GlobalId(u32::MAX)),
            addend: 0,
        });
        assert!(
            Snapshot::capture(&c, LIMITS).is_none(),
            "invalid graph IDs reject the entire snapshot"
        );
        c.out.program.globals[0].relocs.pop();
        let selected_handler = c.out.program.check_failed;
        c.out.program.check_failed = Some(FuncId(u32::MAX));
        assert!(Snapshot::capture(&c, LIMITS).is_none());
        assert!(
            !c.lifetime_execution()
                .unwrap()
                .visit_edges(Node::Function(FuncId(u32::MAX)), |_| {})
        );
        drop(snapshot);
        c.out.program.check_failed = selected_handler;
        c.release_lowered_code_with_lifetime_execution(true);
        let weak = {
            let owner = c.ed.facts().unwrap().lifetime_execution.borrow();
            Rc::downgrade(owner.as_ref().unwrap())
        };
        let lease = c.lifetime_execution().unwrap();
        c.ed.facts().unwrap().lifetime_execution.borrow_mut().take();
        assert!(!lease.available());
        assert_eq!(
            weak.strong_count(),
            1,
            "only the outstanding reader owns the retired payload"
        );
        drop(lease);
        assert!(
            weak.upgrade().is_none(),
            "retired payload is dropped with its final reader"
        );
        assert_eq!(c.lifetime_execution_usage(), (0, 0));
    }
}
