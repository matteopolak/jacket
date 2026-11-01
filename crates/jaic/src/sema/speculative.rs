//! `-dev_speculative_bodies`: stage 1 of parallel checking (`docs/compiler/parallel-sema.md`).
//!
//! `drain_bodies` takes the next few queued bodies as a batch and checks each one speculatively
//! first. A speculative body may read everything and may write only what the body owns: the
//! scopes and entities it makes (cut back off the shared arenas when it ends) and its finished
//! function (held in the delta instead of the program). The first write to anything else, or any
//! error, abandons the body before the write happens; it then runs on the normal serial path.
//! Bodies that finish are committed in queue order: their scopes and entities are appended to
//! the arenas with their ids shifted, and the function is stored.
//!
//! Everything runs on one thread. The point is to show the protocol keeps the output identical
//! and to count what stops bodies from taking it. Interception needs the `speculative-bodies`
//! cargo feature, which turns every write to a `Probed` table into a check; without it the
//! hooks compile to nothing and the flag only prints a notice.
use super::body_probe::{self, W};
use super::procs::BodyState;
use super::scope::{Entity, EntityId, EntityKind, Scope, ScopeId, UsingEntry};
use super::{Compiler, ProcId};
use crate::fxhash::HashMap;
use crate::ir;
use crate::sema::Span;
use crate::types::StructId;
use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};

/// Whether this build can intercept writes at all.
pub const BUILT: bool = cfg!(feature = "speculative-bodies");

/// How many queued bodies one batch takes.
const BATCH: usize = 64;

thread_local! {
    /// A speculative body is being checked.
    static ACTIVE: Cell<bool> = const { Cell::new(false) };
    /// The lengths of the scope and entity arenas when it began: elements from there on are its own.
    static BASES: Cell<[usize; 2]> = const { Cell::new([0, 0]) };
    /// The editor is idle, so its event methods change nothing.
    static IDLE_EDITOR: Cell<bool> = const { Cell::new(false) };
    /// The write the body was abandoned at.
    static BLOCKER: Cell<usize> = const { Cell::new(0) };
    /// The expression types the body logged.
    static EXPRESSIONS: RefCell<Vec<(ProcId, Span, crate::types::TypeId)>> = const { RefCell::new(Vec::new()) };
    /// Length of the program's globals when the body began, and the globals it asked for.
    static LOCAL_GLOBALS: RefCell<(usize, Vec<LocalGlobal>)> = const { RefCell::new((0, Vec::new())) };
    /// Changes the body made to shared tables in place, taken back out when it ends.
    static UNDO: RefCell<Vec<Undo>> = const { RefCell::new(Vec::new()) };
    /// Changes the body leaves for its commit.
    static OPS: RefCell<Vec<Op>> = const { RefCell::new(Vec::new()) };
    /// The finished function.
    static DEPOSIT: RefCell<Option<Deposit>> = const { RefCell::new(None) };
    static STATS: RefCell<Stats> = RefCell::new(Stats::default());
    static ON: Cell<bool> = const { Cell::new(false) };
}

/// Memoised lookups: a pure function of the world they are stamped with, so an entry made by a
/// body that is later abandoned is still true.
const MEMOS: [usize; 4] = [
    W::CachesModuleNames as usize,
    W::CachesModuleMisses as usize,
    W::CachesScopeMisses as usize,
    W::CachesUsingMemberFree as usize,
];

/// What `spec_fingerprint` lists, in order.
const FINGERPRINT: [&str; 54] = [
    "ns.check_handler_resolved",
    "ns.named_imports_done",
    "ns.preload",
    "ns.runtime_support",
    "ns.main_module",
    "ns.program_param_settings",
    "ct.context",
    "ct.context_global",
    "ct.temporary_buffer",
    "ct.executions",
    "ct.traps",
    "ct.runs_done",
    "ct.asserts_done",
    "src.added",
    "src.program_sources",
    "src.library_files",
    "ty.context_type",
    "ty.defer_infos",
    "ty.strict_sizes",
    "ty.in_snapshot",
    "ty.snapshot_key.0",
    "ty.snapshot_key.1",
    "ty.snapshot_key.2",
    "ty.in_element_pattern",
    "pr.runtime_interpreter_assertion",
    "pr.entry_point",
    "pr.parked_epoch",
    "pr.lowering_depth",
    "out.runtime_info",
    "out.runtime_storage_global",
    "out.asm_pair_lock",
    "out.asm_aes_tables",
    "out.declared_globals",
    "out.runtime_storage_dependencies.functions",
    "out.runtime_storage_dependencies.entities",
    "out.runtime_storage_dependencies.layouts",
    "out.runtime_storage_dependencies.captures",
    "out.unreferenced_from",
    "epochs.namespace_generation",
    "epochs.placeholder_misses",
    "epochs.in_progress_misses",
    "epochs.resolving_depth",
    "epochs.mapping_imports",
    "epochs.retrying_pending",
    "epochs.lookup_without_expansion",
    "epochs.placeholders_final",
    "ty.types",
    "out.program.globals",
    "out.program.funcs",
    "out.program.foreigns",
    "pr.procs",
    "ns.modules",
    "src.files",
    "caches.module_names_unsettled",
];

/// Unwind payload of an abandoned speculative body.
struct Abandoned;

/// Read the switch once; `Compiler::new` calls this.
pub fn init() {
    let on = crate::knobs::get().speculative_bodies
        || jacket_config::env::SPECULATIVE_BODIES.get().is_some();
    if on && !BUILT {
        eprintln!(
            "note: speculative bodies need a jaic built with `--features speculative-bodies`; checking serially"
        );
    }
    ON.with(|o| o.set(on && BUILT));
}

pub fn enabled() -> bool {
    BUILT && ON.with(Cell::get)
}

#[inline(always)]
pub fn write_hit(table: usize, at: Option<usize>) {
    if ACTIVE.with(Cell::get) {
        blocked_unless_own(table, at);
    }
}

#[inline(always)]
pub fn append_hit(table: usize) {
    if ACTIVE.with(Cell::get) && table != W::NsScopes as usize && table != W::NsEntities as usize {
        abandon(table);
    }
}

#[cold]
fn blocked_unless_own(table: usize, at: Option<usize>) {
    let bases = BASES.with(Cell::get);
    // Scratch the checker leaves behind and the body restores (`last_call_must`), and an editor
    // that records nothing.
    if MEMOS.contains(&table)
        || table == W::NsReexportVisiting as usize
        || table == W::PrLastCallMust as usize
        || (table == W::Ed as usize && IDLE_EDITOR.with(Cell::get))
    {
        return;
    }
    let own = match at {
        Some(at) if table == W::NsScopes as usize => at >= bases[0],
        Some(at) if table == W::NsEntities as usize => at >= bases[1],
        _ => false,
    };
    if !own {
        abandon(table);
    }
}

#[cold]
fn abandon(table: usize) -> ! {
    ACTIVE.with(|a| a.set(false));
    BLOCKER.with(|b| b.set(table));
    std::panic::resume_unwind(Box::new(Abandoned))
}

/// The finished function of a body, held until commit.
struct Deposit {
    func_id: ir::FuncId,
    func: ir::Func,
    storage_constants: Vec<i64>,
    span: Span,
}

/// How far a body's scope and entity ids move when it is committed.
pub(super) struct Shift {
    scope_base: u32,
    scope: u32,
    entity_base: u32,
    entity: u32,
}

impl Shift {
    pub(super) fn scope(&self, id: ScopeId) -> ScopeId {
        if id.0 >= self.scope_base {
            ScopeId(id.0 + self.scope)
        } else {
            id
        }
    }

    pub(super) fn entity(&self, id: EntityId) -> EntityId {
        if id.0 >= self.entity_base {
            EntityId(id.0 + self.entity)
        } else {
            id
        }
    }
}

/// Takes a change back out of a shared table when a speculative body ends.
type Undo = Box<dyn FnOnce(&mut Compiler)>;

/// A write to shared state a speculative body leaves for its commit.
type Op = Box<dyn FnOnce(&mut Compiler, &Shift)>;

/// Leave `op` for the commit of the speculative body being checked. `false`: not speculating, the
/// caller makes the write itself.
pub(super) fn defer(op: impl FnOnce(&mut Compiler, &Shift) + 'static) -> bool {
    if !BUILT || !ACTIVE.with(Cell::get) {
        return false;
    }
    OPS.with(|o| o.borrow_mut().push(Box::new(op)));
    true
}

/// A global a speculative body asked for, made when it is committed.
enum LocalGlobal {
    String(std::sync::Arc<[u8]>),
    Const {
        size: u64,
        align: u64,
        init: Vec<u8>,
        relocs: Vec<ir::Reloc>,
    },
}

/// The ids a speculative body hands out for globals start after the program's last one.
fn local_global(global: LocalGlobal) -> ir::GlobalId {
    LOCAL_GLOBALS.with(|g| {
        let mut g = g.borrow_mut();
        let id = ir::GlobalId((g.0 + g.1.len()) as u32);
        g.1.push(global);
        id
    })
}

/// The global for a string literal that does not exist yet, when speculating.
pub(super) fn local_string(bytes: &std::sync::Arc<[u8]>) -> Option<ir::GlobalId> {
    if !BUILT || !ACTIVE.with(Cell::get) {
        return None;
    }
    let known = LOCAL_GLOBALS.with(|g| {
        let g = g.borrow();
        g.1.iter()
            .position(|l| matches!(l, LocalGlobal::String(s) if s == bytes))
            .map(|k| ir::GlobalId((g.0 + k) as u32))
    });
    Some(known.unwrap_or_else(|| local_global(LocalGlobal::String(bytes.clone()))))
}

/// The global for an aggregate constant, when speculating.
pub(super) fn local_const(
    size: u64,
    align: u64,
    agg: &super::value::Aggregate,
) -> Option<ir::GlobalId> {
    if !BUILT || !ACTIVE.with(Cell::get) {
        return None;
    }
    Some(local_global(LocalGlobal::Const {
        size,
        align,
        init: agg.bytes.clone(),
        relocs: agg.relocs.clone(),
    }))
}

/// Change a shared table in place for the rest of the speculative body being checked, which
/// sees the change, and take it out again when the body ends. `redo` makes the change for real
/// when the body is committed. Callers check [`active`] first.
pub(super) fn journal(
    undo: impl FnOnce(&mut Compiler) + 'static,
    redo: impl FnOnce(&mut Compiler, &Shift) + 'static,
) {
    UNDO.with(|u| u.borrow_mut().push(Box::new(undo)));
    OPS.with(|o| o.borrow_mut().push(Box::new(redo)));
}

/// A speculative body notes the type an expression checked to; `false`: not speculating.
#[inline]
pub(super) fn log_expression(procedure: ProcId, span: Span, ty: crate::types::TypeId) -> bool {
    if !BUILT || !ACTIVE.with(Cell::get) {
        return false;
    }
    EXPRESSIONS.with(|e| e.borrow_mut().push((procedure, span, ty)));
    true
}

/// Whether a speculative body is being checked.
#[inline]
pub(super) fn active() -> bool {
    BUILT && ACTIVE.with(Cell::get)
}

/// Keep the finished function of the speculative body being checked until it is committed.
pub(super) fn deposit(
    func_id: ir::FuncId,
    func: ir::Func,
    storage_constants: Vec<i64>,
    span: Span,
) {
    DEPOSIT.with(|d| {
        *d.borrow_mut() = Some(Deposit {
            func_id,
            func,
            storage_constants,
            span,
        });
    });
}

/// Why a body took the serial path.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
enum Reason {
    /// It wrote this shared table (a `body_probe::W` index).
    Wrote(usize),
    /// Checking it reported an error.
    Failed,
    /// What it made cannot be moved to other ids yet.
    Immovable(&'static str),
    /// It finished, but an earlier body of the batch that ran serially changed the world.
    Invalidated,
    /// An earlier body of the batch lowered it first.
    LoweredMeanwhile,
    /// The editor is live, or the body is not at the top of the lowering stack.
    Unsupported,
}

#[derive(Default)]
struct Stats {
    batches: u64,
    batched: u64,
    committed: u64,
    aborted: std::collections::BTreeMap<Reason, (u64, Duration)>,
    view_time: Duration,
    view_wasted: Duration,
    commit_time: Duration,
    serial_time: Duration,
    serial_bodies: u64,
    scopes_moved: u64,
    entities_moved: u64,
}

/// What a finished speculative body made.
pub(super) struct Delta {
    id: ProcId,
    scopes_base: usize,
    entities_base: usize,
    scopes: Vec<Scope>,
    entities: Vec<Entity>,
    deposit: Deposit,
    expressions: Vec<(ProcId, Span, crate::types::TypeId)>,
    ops: Vec<Op>,
    globals_base: usize,
    globals: Vec<LocalGlobal>,
    took: Duration,
}

pub(super) enum Outcome {
    Ready(Box<Delta>),
    Abandoned,
}

/// The parts of the world a serial body can change that a finished speculative body read.
type Stamp = [u64; 7];

impl Compiler {
    fn spec_stamp(&self) -> Stamp {
        [
            self.epochs.namespace_generation,
            self.src.added as u64,
            self.src.files.len() as u64,
            self.ns.modules.len() as u64,
            self.ct.runs_done as u64,
            self.ct.executions,
            self.src.sources.len() as u64,
        ]
    }

    /// `local_consts[e] = (value, ty)` for a constant entity of the body being checked.
    pub(super) fn set_local_const(
        &mut self,
        e: EntityId,
        value: super::Value,
        ty: crate::types::TypeId,
    ) {
        if active() {
            let kept = value.clone();
            journal(
                move |c| {
                    c.pr.local_consts.quiet().remove(&e);
                },
                move |c, shift| {
                    c.pr.local_consts.insert(shift.entity(e), (kept, ty));
                },
            );
            self.pr.local_consts.quiet().insert(e, (value, ty));
            return;
        }
        self.pr.local_consts.insert(e, (value, ty));
    }

    /// A macro parameter `e` holds a constant argument the macro never writes.
    pub(super) fn set_const_macro_param(
        &mut self,
        e: EntityId,
        value: super::Value,
        ty: crate::types::TypeId,
    ) {
        if active() {
            let kept = value.clone();
            journal(
                move |c| {
                    c.pr.const_macro_params.quiet().remove(&e);
                },
                move |c, shift| {
                    c.pr.const_macro_params.insert(shift.entity(e), (kept, ty));
                },
            );
            self.pr.const_macro_params.quiet().insert(e, (value, ty));
            return;
        }
        self.pr.const_macro_params.insert(e, (value, ty));
    }

    /// Mark the parameter entity `e` as one whose value is discarded.
    pub(super) fn set_discard_param(&mut self, e: EntityId) {
        if active() {
            journal(
                move |c| {
                    c.pr.discard_params.quiet().remove(&e);
                },
                move |c, shift| {
                    c.pr.discard_params.insert(shift.entity(e));
                },
            );
            self.pr.discard_params.quiet().insert(e);
            return;
        }
        self.pr.discard_params.insert(e);
    }

    /// Remember what a callable parameter or alias `e` was declared as.
    pub(super) fn set_callable_entity(
        &mut self,
        e: EntityId,
        info: std::sync::Arc<super::ProcTypeParams>,
    ) {
        if active() {
            let kept = info.clone();
            journal(
                move |c| {
                    c.pr.callable_entities.quiet().remove(&e);
                },
                move |c, shift| {
                    c.pr.callable_entities.insert(shift.entity(e), kept);
                },
            );
            self.pr.callable_entities.quiet().insert(e, info);
            return;
        }
        self.pr.callable_entities.insert(e, info);
    }

    /// The `ifx` at `span` is an operand of a binary operator.
    pub(super) fn set_ifx_operand(&mut self, span: Span) {
        if self.pr.ifx_operands.contains(&span) {
            return;
        }
        if active() {
            journal(
                move |c| {
                    c.pr.ifx_operands.quiet().remove(&span);
                },
                move |c, _| {
                    c.pr.ifx_operands.insert(span);
                },
            );
            self.pr.ifx_operands.quiet().insert(span);
            return;
        }
        self.pr.ifx_operands.insert(span);
    }

    /// The call at `span` was made to `instance` of a polymorphic procedure.
    pub(super) fn note_call_instance(&mut self, span: Span, instance: ProcId) {
        let note = move |c: &mut Compiler, _: &Shift| {
            let made = c.meta.export.call_instances.entry(span).or_default();
            if !made.contains(&instance) {
                made.push(instance);
            }
        };
        if !defer(note) {
            let made = self.meta.export.call_instances.entry(span).or_default();
            if !made.contains(&instance) {
                made.push(instance);
            }
        }
    }

    /// The shared state a body is not allowed to leave changed that no `Probed` hook watches:
    /// plain flags, counters and options. Debug builds compare it before and after every
    /// speculative body, so a write the hooks missed fails a test instead of the output.
    fn spec_fingerprint(&self) -> [u64; FINGERPRINT.len()] {
        let id = |v: Option<u32>| v.map_or(u64::MAX, u64::from);
        [
            u64::from(self.ns.check_handler_resolved),
            self.ns.named_imports_done as u64,
            id(self.ns.preload.map(|m| m.0)),
            id(self.ns.runtime_support.map(|m| m.0)),
            id(self.ns.main_module.map(|m| m.0)),
            self.ns.program_param_settings.len() as u64,
            self.ct.context.unwrap_or(u64::MAX),
            id(self.ct.context_global.map(|g| g.0)),
            id(self.ct.temporary_buffer.map(|g| g.0)),
            self.ct.executions,
            self.ct.traps,
            self.ct.runs_done as u64,
            self.ct.asserts_done as u64,
            self.src.added as u64,
            self.src.program_sources.len() as u64,
            self.src.library_files.borrow().len() as u64,
            id(self.ty.context_type.map(|t| t.0)),
            u64::from(self.ty.defer_infos),
            u64::from(self.ty.strict_sizes),
            u64::from(self.ty.in_snapshot),
            self.ty.snapshot_key.0 as u64,
            self.ty.snapshot_key.1 as u64,
            self.ty.snapshot_key.2 as u64,
            u64::from(self.ty.in_element_pattern),
            id(self.pr.runtime_interpreter_assertion.map(|p| p.0)),
            id(self.pr.entry_point.map(|p| p.0)),
            self.pr.parked_epoch,
            u64::from(self.pr.lowering_depth),
            id(self.out.runtime_info.map(|g| g.0)),
            id(self.out.runtime_storage_global.map(|g| g.0)),
            id(self.out.asm_pair_lock.map(|g| g.0)),
            id(self.out.asm_aes_tables.map(|g| g.0)),
            self.out.declared_globals.len() as u64,
            self.out.runtime_storage_dependencies.functions.len() as u64,
            self.out.runtime_storage_dependencies.entities.len() as u64,
            self.out.runtime_storage_dependencies.layouts.len() as u64,
            self.out.runtime_storage_dependencies.captures.len() as u64,
            u64::from(self.out.unreferenced_from.is_some()),
            self.epochs.namespace_generation,
            self.epochs.placeholder_misses,
            self.epochs.in_progress_misses,
            u64::from(self.epochs.resolving_depth),
            u64::from(self.epochs.mapping_imports),
            u64::from(self.epochs.retrying_pending),
            u64::from(self.epochs.lookup_without_expansion),
            u64::from(self.epochs.placeholders_final),
            self.ty.types.type_count() as u64,
            self.out.program.globals.len() as u64,
            self.out.program.funcs.len() as u64,
            self.out.program.foreigns.len() as u64,
            self.pr.procs.len() as u64,
            self.ns.modules.len() as u64,
            self.src.files.len() as u64,
            u64::from(self.caches.module_names_unsettled),
        ]
    }

    /// While speculating, note `values` for the entity `id` as the body goes: the body sees them,
    /// they are taken out when it ends, and its commit notes them for real. `false`: not speculating.
    pub(super) fn speculative_storage_entity(&mut self, id: EntityId, values: &[i64]) -> bool {
        if !active() {
            return false;
        }
        let values = values.to_vec();
        let dependencies = &mut self.out.runtime_storage_dependencies;
        let before = dependencies.entities.get(&id).cloned();
        let known = dependencies.entities.entry(id).or_default();
        merge(known, &values);
        let merged = known.clone();
        // What a body notes goes to the enclosing capture, as it does serially.
        if let Some(capture) = dependencies.captures.last_mut() {
            merge(capture, &merged);
        }
        journal(
            move |c| {
                let entities = &mut c.out.runtime_storage_dependencies.entities;
                match before {
                    Some(before) => entities.insert(id, before),
                    None => entities.remove(&id),
                };
            },
            move |c, shift| {
                let known = c
                    .out
                    .runtime_storage_dependencies
                    .entities
                    .entry(shift.entity(id))
                    .or_default();
                merge(known, &values);
            },
        );
        true
    }

    /// The same for the layout of the struct `id`.
    pub(super) fn speculative_storage_layout(&mut self, id: StructId, values: &[i64]) -> bool {
        if !active() {
            return false;
        }
        let values = values.to_vec();
        let dependencies = &mut self.out.runtime_storage_dependencies;
        let before = dependencies.layouts.get(&id).cloned();
        let known = dependencies.layouts.entry(id).or_default();
        merge(known, &values);
        let merged = known.clone();
        if let Some(capture) = dependencies.captures.last_mut() {
            merge(capture, &merged);
        }
        journal(
            move |c| {
                let layouts = &mut c.out.runtime_storage_dependencies.layouts;
                match before {
                    Some(before) => layouts.insert(id, before),
                    None => layouts.remove(&id),
                };
            },
            move |c, _| {
                let known = c
                    .out
                    .runtime_storage_dependencies
                    .layouts
                    .entry(id)
                    .or_default();
                merge(known, &values);
            },
        );
        true
    }

    /// Check `id` speculatively. The shared state is as it was before, whatever the outcome.
    pub(super) fn speculate_body(&mut self, id: ProcId) -> Outcome {
        let started = Instant::now();
        let unsupported = self.ed.is_live()
            || self.pr.lowering_depth != 0
            || self.proc(id).body_state != BodyState::Queued
            || !matches!(
                self.proc(id).target,
                Some(super::procs::ProcTarget::Func(_))
            );
        if unsupported {
            STATS.with(|s| {
                s.borrow_mut()
                    .aborted
                    .entry(Reason::Unsupported)
                    .or_default()
                    .0 += 1;
            });
            return Outcome::Abandoned;
        }
        let bases = [self.ns.scopes.len(), self.ns.entities.len()];
        let fingerprint = cfg!(debug_assertions).then(|| self.spec_fingerprint());
        // What a body takes out of the shared state and puts back when it ends.
        let captures = self.out.runtime_storage_dependencies.captures.clone();
        let in_element_pattern = self.ty.in_element_pattern;
        let names_unsettled = self.caches.module_names_unsettled;
        let last_call_must = self.pr.last_call_must.clone();
        let reexport_visiting = self.ns.reexport_visiting.clone();
        // A settle point inside the body moves the mark over entities it found nothing to resolve in.
        let named_imports_done = self.ns.named_imports_done;
        let type_probe = self.ed.type_probe;
        let sizes = (self.ty.defer_infos, self.ty.strict_sizes);
        let lookup_without_expansion = self.epochs.lookup_without_expansion;
        IDLE_EDITOR.with(|i| i.set(true));
        self.pr.procs[id.0 as usize].body_state = BodyState::Lowering;
        self.pr.lowering_depth += 1;
        DEPOSIT.with(|d| *d.borrow_mut() = None);
        EXPRESSIONS.with(|e| e.borrow_mut().clear());
        OPS.with(|o| o.borrow_mut().clear());
        UNDO.with(|u| u.borrow_mut().clear());
        LOCAL_GLOBALS.with(|g| *g.borrow_mut() = (self.out.program.globals.len(), Vec::new()));
        BASES.with(|b| b.set(bases));
        ACTIVE.with(|a| a.set(true));
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.lower_body_inner_for_speculation(id)
        }));
        let finished_active = ACTIVE.with(|a| a.replace(false));
        self.pr.lowering_depth -= 1;
        self.pr.procs[id.0 as usize].body_state = BodyState::Queued;
        self.out.runtime_storage_dependencies.captures = captures;
        self.ty.in_element_pattern = in_element_pattern;
        self.caches.module_names_unsettled = names_unsettled;
        self.pr.last_call_must = last_call_must;
        self.ns.reexport_visiting = reexport_visiting.into();
        self.ns.named_imports_done = named_imports_done;
        self.ed.type_probe = type_probe;
        (self.ty.defer_infos, self.ty.strict_sizes) = sizes;
        self.epochs.lookup_without_expansion = lookup_without_expansion;
        IDLE_EDITOR.with(|i| i.set(false));
        let undo = UNDO.with(|u| std::mem::take(&mut *u.borrow_mut()));
        for undo in undo.into_iter().rev() {
            undo(self);
        }
        if let Some(before) = fingerprint {
            let after = self.spec_fingerprint();
            for (i, (a, b)) in before.iter().zip(&after).enumerate() {
                assert_eq!(a, b, "a speculative body changed {}", FINGERPRINT[i]);
            }
        }
        let scopes = self.ns.scopes.split_off_tail(bases[0]);
        let entities = self.ns.entities.split_off_tail(bases[1]);
        let deposit = DEPOSIT.with(|d| d.borrow_mut().take());
        let bases_globals = LOCAL_GLOBALS.with(|g| g.borrow().0);
        let took = started.elapsed();
        let reason = match result {
            Ok(Ok(())) if finished_active => match deposit {
                Some(deposit) => {
                    if let Some(why) = immovable(&scopes, &entities, bases) {
                        Reason::Immovable(why)
                    } else {
                        return Outcome::Ready(Box::new(Delta {
                            id,
                            scopes_base: bases[0],
                            entities_base: bases[1],
                            scopes,
                            entities,
                            deposit,
                            expressions: EXPRESSIONS.with(|e| std::mem::take(&mut *e.borrow_mut())),
                            ops: OPS.with(|o| std::mem::take(&mut *o.borrow_mut())),
                            globals_base: bases_globals,
                            globals: LOCAL_GLOBALS.with(|g| std::mem::take(&mut g.borrow_mut().1)),
                            took,
                        }));
                    }
                }
                None => Reason::Immovable("no function"),
            },
            Ok(Ok(())) => unreachable!("a speculative body finished after being abandoned"),
            Ok(Err(_)) => Reason::Failed,
            Err(payload) if payload.is::<Abandoned>() => Reason::Wrote(BLOCKER.with(Cell::get)),
            Err(payload) => std::panic::resume_unwind(payload),
        };
        STATS.with(|s| {
            let mut s = s.borrow_mut();
            let entry = s.aborted.entry(reason).or_default();
            entry.0 += 1;
            entry.1 += took;
            s.view_wasted += took;
        });
        Outcome::Abandoned
    }

    /// Install what a finished speculative body made, as the serial path would have.
    fn commit_body(&mut self, delta: Delta) {
        let started = Instant::now();
        let Delta {
            id,
            scopes_base,
            entities_base,
            mut scopes,
            mut entities,
            deposit,
            expressions,
            ops,
            globals_base,
            globals,
            took,
        } = delta;
        // The globals the body asked for are made first: what it built refers to them by the ids
        // it was given.
        let mut made: Vec<ir::GlobalId> = Vec::new();
        for global in globals {
            let global = match global {
                LocalGlobal::String(bytes) => self.out.string_global(&bytes),
                LocalGlobal::Const {
                    size,
                    align,
                    init,
                    mut relocs,
                } => {
                    remap_relocs(&mut relocs, globals_base, &made);
                    self.out.const_global(
                        size,
                        align,
                        &super::value::Aggregate {
                            bytes: init,
                            relocs,
                        },
                    )
                }
            };
            made.push(global);
        }
        let shift = Shift {
            scope_base: scopes_base as u32,
            scope: self.ns.scopes.len() as u32 - scopes_base as u32,
            entity_base: entities_base as u32,
            entity: self.ns.entities.len() as u32 - entities_base as u32,
        };
        let (moved_scopes, moved_entities) = (scopes.len() as u64, entities.len() as u64);
        for scope in &mut scopes {
            scope.parent = scope.parent.map(|p| shift.scope(p));
            scope.inserted_into = scope.inserted_into.map(|p| shift.scope(p));
            for ids in scope.names.values_mut() {
                for e in ids {
                    *e = shift.entity(*e);
                }
            }
            for using in &mut scope.usings {
                remap_using(using, &shift);
            }
        }
        for entity in &mut entities {
            entity.scope = shift.scope(entity.scope);
            entity.home = shift.scope(entity.home);
            if let EntityKind::Const {
                value: super::Value::Bytes(agg),
                ..
            } = &mut entity.kind
                && agg.relocs.iter().any(|r| refers_past(r, globals_base))
            {
                let mut moved = (**agg).clone();
                remap_relocs(&mut moved.relocs, globals_base, &made);
                *agg = std::sync::Arc::new(moved);
            }
        }
        self.ns.scopes.extend_tail(scopes);
        self.ns.entities.extend_tail(entities);
        let Deposit {
            func_id,
            mut func,
            storage_constants,
            span,
        } = deposit;
        if !made.is_empty() {
            for block in &mut func.blocks {
                for inst in &mut block.insts {
                    if let ir::Inst::GlobalAddr {
                        global, ..
                    } = inst
                        && global.0 as usize >= globals_base
                    {
                        *global = made[global.0 as usize - globals_base];
                    }
                }
            }
        }
        self.diag.dependencies.begin_body(id);
        self.meta.export.expression_types.remove(&id);
        for (procedure, span, ty) in expressions {
            self.meta
                .export
                .expression_types
                .entry(procedure)
                .or_default()
                .record(span, ty);
        }
        for op in ops {
            op(self, &shift);
        }
        self.diag.dependencies.end_body(id, false);
        self.out
            .runtime_storage_dependencies
            .functions
            .insert(func_id, (storage_constants, span));
        self.out.program.funcs[func_id.0 as usize] = Some(func);
        crate::progress::procedure_checked();
        self.pr.lenient_failures.remove(&id);
        self.pr.procs[id.0 as usize].body_state = BodyState::Done;
        STATS.with(|s| {
            let mut s = s.borrow_mut();
            s.committed += 1;
            s.view_time += took;
            s.commit_time += started.elapsed();
            s.scopes_moved += moved_scopes;
            s.entities_moved += moved_entities;
        });
    }

    /// Check the bodies on top of the queue speculatively, in the order the serial path would
    /// run them, against the world as it is now. They stay queued: a serial body that drains the
    /// queue in the middle of checking (compile-time code) must find them there, as it does
    /// without speculation.
    fn speculate_top(&mut self, pending: &mut HashMap<ProcId, (Outcome, Stamp)>) {
        let batch: Vec<ProcId> = self
            .pr
            .body_queue
            .iter()
            .rev()
            .copied()
            .filter(|&id| {
                self.proc(id).body_state == BodyState::Queued && !pending.contains_key(&id)
            })
            .take(BATCH)
            .collect();
        if batch.is_empty() {
            return;
        }
        STATS.with(|s| {
            let mut s = s.borrow_mut();
            s.batches += 1;
            s.batched += batch.len() as u64;
        });
        let stamp = self.spec_stamp();
        for id in batch {
            let outcome = self.speculate_body(id);
            pending.insert(id, (outcome, stamp));
        }
    }

    /// Lower the queued bodies, a batch at a time, in the order the serial path would. A body is
    /// committed if it finished speculatively and nothing that could have changed what it read
    /// happened since; otherwise it runs serially.
    pub(super) fn drain_speculatively(&mut self, pass: &mut super::procs::DrainPass) {
        let mut pending = HashMap::default();
        while let Some(id) = body_probe::next_body(&mut self.pr.body_queue) {
            if self.proc(id).body_state != BodyState::Queued {
                // An earlier body lowered it on demand.
                if let Some((Outcome::Ready(delta), _)) = pending.remove(&id) {
                    STATS.with(|s| {
                        let mut s = s.borrow_mut();
                        let entry = s.aborted.entry(Reason::LoweredMeanwhile).or_default();
                        entry.0 += 1;
                        entry.1 += delta.took;
                        s.view_wasted += delta.took;
                    });
                }
                continue;
            }
            if !pending.contains_key(&id) {
                self.pr.body_queue.push(id);
                self.speculate_top(&mut pending);
                self.pr.body_queue.pop();
            }
            if let Some((Outcome::Ready(delta), stamp)) = pending.remove(&id) {
                let reason = (self.spec_stamp() != stamp).then_some(Reason::Invalidated);
                let Some(reason) = reason else {
                    self.commit_body(*delta);
                    continue;
                };
                STATS.with(|s| {
                    let mut s = s.borrow_mut();
                    let entry = s.aborted.entry(reason).or_default();
                    entry.0 += 1;
                    entry.1 += delta.took;
                    s.view_wasted += delta.took;
                });
            }
            let started = Instant::now();
            self.drain_one(id, pass);
            STATS.with(|s| {
                let mut s = s.borrow_mut();
                s.serial_bodies += 1;
                s.serial_time += started.elapsed();
            });
        }
    }
}

fn merge(target: &mut Vec<i64>, values: &[i64]) {
    for &value in values {
        if !target.contains(&value) {
            target.push(value);
        }
    }
}

fn refers_past(reloc: &ir::Reloc, base: usize) -> bool {
    matches!(reloc.target, ir::RelocTarget::Global(g) if g.0 as usize >= base)
}

/// Point the relocations at a body's provisional globals to the ones that were made for them.
fn remap_relocs(relocs: &mut [ir::Reloc], base: usize, made: &[ir::GlobalId]) {
    for reloc in relocs {
        if let ir::RelocTarget::Global(g) = &mut reloc.target
            && g.0 as usize >= base
        {
            *g = made[g.0 as usize - base];
        }
    }
}

fn remap_using(using: &mut UsingEntry, shift: &Shift) {
    match using {
        UsingEntry::Place {
            entity, ..
        } => *entity = shift.entity(*entity),
        UsingEntry::Filtered(inner, _) => remap_using(inner, shift),
        UsingEntry::Type(_) | UsingEntry::Module(..) => {}
    }
}

/// Whether what a body made refers to things that cannot be moved by shifting scope and entity ids.
fn immovable(scopes: &[Scope], entities: &[Entity], bases: [usize; 2]) -> Option<&'static str> {
    for scope in scopes {
        if !scope.imports.is_empty() {
            return Some("scope with imports");
        }
        if !scope.pending.is_empty() {
            return Some("scope with pending items");
        }
        if !scope.fallbacks.is_empty() {
            return Some("scope with fallbacks");
        }
        // Names, parents and using entries are shifted; they must name the body's own elements
        // or older ones, never ones that were cut back.
        if scope
            .parent
            .is_some_and(|p| p.0 as usize >= bases[0] + scopes.len())
        {
            return Some("scope parent beyond the body");
        }
    }
    for entity in entities {
        if !matches!(
            entity.kind,
            EntityKind::Local { .. } | EntityKind::Const { .. } | EntityKind::Decl { .. }
        ) {
            return Some("entity of another kind");
        }
    }
    None
}

/// Print the counters.
pub fn report() {
    if !enabled() || !body_probe::enabled() {
        return;
    }
    STATS.with(|s| {
        let s = s.borrow();
        let aborted: u64 = s.aborted.values().map(|a| a.0).sum();
        eprintln!(
            "== speculative bodies: {} batches, {} bodies batched, {} committed, {} abandoned",
            s.batches, s.batched, s.committed, aborted
        );
        let pct = |n: u64| 100.0 * n as f64 / s.batched.max(1) as f64;
        eprintln!(
            "  committed {:.1}% of batched; moved {} scopes, {} entities",
            pct(s.committed),
            s.scopes_moved,
            s.entities_moved
        );
        eprintln!("-- abandoned by first blocker:");
        let mut rows: Vec<_> = s.aborted.iter().collect();
        rows.sort_by_key(|(_, (n, _))| std::cmp::Reverse(*n));
        for (reason, (n, time)) in rows {
            let name = match reason {
                Reason::Wrote(i) => format!("wrote {}", body_probe::write_name(*i)),
                Reason::Failed => "reported an error".into(),
                Reason::Immovable(why) => format!("immovable: {why}"),
                Reason::Invalidated => "invalidated by a serial body".into(),
                Reason::LoweredMeanwhile => "lowered on demand by an earlier body".into(),
                Reason::Unsupported => "not eligible".into(),
            };
            eprintln!(
                "  {name:<44} {n:>8} ({:>4.1}%)  wasted {time:>10.3?}",
                pct(*n)
            );
        }
        let per = |t: Duration, n: u64| t.as_secs_f64() * 1e6 / n.max(1) as f64;
        eprintln!("-- time:");
        eprintln!(
            "  view path, committed   {:>10.3?} ({:.1} us/body)",
            s.view_time,
            per(s.view_time, s.committed)
        );
        eprintln!(
            "  view path, abandoned   {:>10.3?} ({:.1} us/body)",
            s.view_wasted,
            per(s.view_wasted, aborted)
        );
        eprintln!(
            "  commit                 {:>10.3?} ({:.2} us/body)",
            s.commit_time,
            per(s.commit_time, s.committed)
        );
        eprintln!(
            "  serial path (batched)  {:>10.3?} ({:.1} us/body)",
            s.serial_time,
            per(s.serial_time, s.serial_bodies)
        );
    });
}
