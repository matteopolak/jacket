//! `-dev_body_probe`: which shared tables does each procedure body mutate while it is lowered?
//!
//! A study aid for `docs/compiler/parallel-sema.md`. Around every `lower_body` it snapshots the
//! sizes of the compiler's shared tables, charges the growth to the body that caused it (growth
//! made by bodies lowered on demand inside it is charged to those), and prints a histogram when
//! the compile ends. It costs nothing when the variable is unset.
use super::Compiler;
use super::scope::ScopeKind;
use std::cell::RefCell;
use std::time::{Duration, Instant};

macro_rules! writes {
    ($($v:ident $n:literal,)*) => {
        /// A piece of shared compiler state whose in-place writes the probe counts.
        #[derive(Clone, Copy)]
        pub enum W { $($v),* }
        const WRITE_NAMES: &[&str] = &[$($n),*];
    };
}

writes! {
    PrProcs "pr.procs",
    PrCodes "pr.codes",
    PrLenientFailures "pr.lenient_failures",
    PrProvisionalErrors "pr.provisional_errors",
    PrAutoBakeVariants "pr.auto_bake_variants",
    PrBodyQueue "pr.body_queue",
    PrParkedBodies "pr.parked_bodies",
    PrMultiConsts "pr.multi_consts",
    PrCodeProcs "pr.code_procs",
    PrFuncTargets "pr.func_targets",
    PrLiveProcs "pr.live_procs",
    PrExports "pr.exports",
    PrAnonymousProcs "pr.anonymous_procs",
    PrCallsInFlight "pr.calls_in_flight",
    PrLocalConsts "pr.local_consts",
    PrDiscardParams "pr.discard_params",
    PrConstMacroParams "pr.const_macro_params",
    PrLocalDeclTypes "pr.local_decl_types",
    PrCodeScopes "pr.code_scopes",
    PrUnscopedCodes "pr.unscoped_codes",
    PrProcDeclNotes "pr.proc_decl_notes",
    PrInterpolated "pr.interpolated",
    PrIfxOperands "pr.ifx_operands",
    PrFirstArguments "pr.first_arguments",
    PrFuncProcs "pr.func_procs",
    PrAsmRegs "pr.asm_regs",
    PrAsmPins "pr.asm_pins",
    PrCallableEntities "pr.callable_entities",
    PrCallableFields "pr.callable_fields",
    TyNominalOrigins "ty.nominal_origins",
    TyPolyStructs "ty.poly_structs",
    TyStructAsts "ty.struct_asts",
    TyFieldTypes "ty.field_types",
    TyFieldDeclarations "ty.field_declarations",
    TyFieldTypePending "ty.field_type_pending",
    TyFieldFilterCache "ty.field_filter_cache",
    TyFieldFilterPending "ty.field_filter_pending",
    TyFieldUsingOrder "ty.field_using_order",
    TyInfos "ty.infos",
    TyFailedInfos "ty.failed_infos",
    TyDeferredInfos "ty.deferred_infos",
    TyVariadicCases "ty.variadic_cases",
    TyCasesBuilding "ty.cases_building",
    TyBuildingInfos "ty.building_infos",
    TySnapshotInfos "ty.snapshot_infos",
    TyTableFuncs "ty.table_funcs",
    TyInfoFlags "ty.info_flags",
    TyAnonymous "ty.anonymous",
    TyPolyAliases "ty.poly_aliases",
    TyEnumMemberScopes "ty.enum_member_scopes",
    TyEnumBaseSpans "ty.enum_base_spans",
    TyEnumBorrowed "ty.enum_borrowed",
    TyEnumMemberEntities "ty.enum_member_entities",
    TyMemberAliases "ty.member_aliases",
    TyUsingFilters "ty.using_filters",
    TyDefaultImages "ty.default_images",
    TyDefaultGlobals "ty.default_globals",
    TyInitializers "ty.initializers",
    TyGenericStructInfos "ty.generic_struct_infos",
    NsModules "ns.modules",
    NsModuleCache "ns.module_cache",
    NsModuleFailures "ns.module_failures",
    NsScopes "ns.scopes",
    NsEntities "ns.entities",
    NsLibraries "ns.libraries",
    NsAddContexts "ns.add_contexts",
    NsPokes "ns.pokes",
    NsDefaultScopes "ns.default_scopes",
    NsOverloadSetsChecked "ns.overload_sets_checked",
    NsFailedImports "ns.failed_imports",
    NsReexportVisiting "ns.reexport_visiting",
    NsThunkScopes "ns.thunk_scopes",
    NsExportEntities "ns.export_entities",
    NsDeferredPending "ns.deferred_pending",
    NsUnsettled "ns.unsettled",
    OutRegisteredDataSegments "out.registered_data_segments",
    OutStrings "out.strings",
    OutConstAddrGlobals "out.const_addr_globals",
    OutPacks "out.packs",
    OutOutput "out.output",
    OutNoResetGlobals "out.no_reset_globals",
    CachesModuleNames "caches.module_names",
    CachesModuleMisses "caches.module_misses",
    CachesScopeMisses "caches.scope_misses",
    CachesUsingMemberFree "caches.using_member_free",
    SrcFiles "src.files",
    SrcOrigins "src.origins",
    SrcFileByPath "src.file_by_path",
    SrcHoistedImports "src.hoisted_imports",
    SrcProvidedTexts "src.provided_texts",
    SrcInsertDepth "src.insert_depth",
    SrcInsertSites "src.insert_sites",
    SrcTwinOutbox "src.twin_outbox",
    DiagWarnings "diag.warnings",
    DiagDeferredErrors "diag.deferred_errors",
    CtTopLevelRuns "ct.top_level_runs",
    CtAsserts "ct.asserts",
    CtFrozen "ct.frozen",
    CtInterp "ct.interp",
    CtTwin "ct.twin",
    DiagDependencies "diag.dependencies",
    MetaExport "meta.export",
    MetaPerf "meta.perf",
    OutProgram "out.program",
    SrcCanonCache "src.canon_cache",
    SrcAhead "src.ahead",
    SrcSources "src.sources",
    TyTypes "ty.types",
    Ed "ed",
    EpochsNamespaceGeneration "epochs.namespace_generation",
    EpochsPlaceholderMisses "epochs.placeholder_misses",
    EpochsInProgressMisses "epochs.in_progress_misses",
    EpochsResolvingDepth "epochs.resolving_depth",
    EpochsMappingImports "epochs.mapping_imports",
    CtExecutions "ct.executions",
    PrLastCallMust "pr.last_call_must",
    PrLoweringDepth "pr.lowering_depth",
    SrcLibraryFiles "src.library_files",
    CachesDeclaringImports "caches.declaring_imports",
    CachesStdlibView "caches.stdlib_view",
    NsCheckHandlerResolved "ns.check_handler_resolved",
    OldProcs "pr.procs[i], another body's procedure",
    OldEntities "ns.entities[i], older than the body",
    OldModuleScopes "ns.scopes[i], a root, module or file scope",
    OldStructScopes "ns.scopes[i], a struct scope",
}

const WN: usize = WRITE_NAMES.len();

/// The name of the shared state write number `i` is counted under.
pub fn write_name(i: usize) -> &'static str {
    WRITE_NAMES[i]
}


/// A shared table with its writes counted: reading goes through `Deref`, anything that borrows it
/// mutably (`push`, `insert`, `entry`, `get_mut`, indexing for assignment, passing `&mut`) goes
/// through `DerefMut` and counts one write for `I`. A mutable borrow that does not change anything
/// still counts, so the numbers are an upper bound. Free when `-dev_body_probe` is off apart from
/// one relaxed load.
#[derive(Default)]
pub struct Probed<T, const I: usize>(T);

impl<T, const I: usize> Probed<T, I> {
    pub fn new(value: T) -> Self {
        Probed(value)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

/// Interning looks the type up first, so only a type that is new counts as a write (and makes a
/// speculative body give up).
impl<const I: usize> Probed<crate::types::Types, I> {
    pub fn intern(&mut self, kind: crate::types::TypeKind) -> crate::types::TypeId {
        if let Some(id) = self.0.find(&kind) {
            return id;
        }
        write_at(I);
        self.0.intern(kind)
    }

    pub fn pointer(&mut self, to: crate::types::TypeId) -> crate::types::TypeId {
        self.intern(crate::types::TypeKind::Pointer(to))
    }

    pub fn array(
        &mut self,
        elem: crate::types::TypeId,
        kind: crate::types::ArrayKind,
    ) -> crate::types::TypeId {
        self.intern(crate::types::TypeKind::Array {
            elem,
            kind,
        })
    }
}

impl<T, const I: usize> Probed<T, I> {
    /// The table without counting a write: speculation changes entries only it can see, and
    /// takes them out again (`speculative::journal`).
    pub(super) fn quiet(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T, const I: usize> Probed<Vec<T>, I> {
    /// Append an element that only the body being checked can see yet. Speculation allows it
    /// (the arena is cut back to its length at the start of the body afterwards); any other
    /// mutable borrow of the table aborts the speculative body.
    #[inline(always)]
    pub fn push_new(&mut self, value: T) {
        if super::speculative::BUILT {
            super::speculative::append_hit(I);
        }
        if COUNTING && ON.load(std::sync::atomic::Ordering::Relaxed) {
            WRITES.with(|w| w.borrow_mut()[I] += 1);
        }
        self.0.push(value);
    }

    /// Take the elements from `at` on, without counting a write (speculation's rollback).
    pub(super) fn split_off_tail(&mut self, at: usize) -> Vec<T> {
        self.0.split_off(at)
    }

    /// Append elements without counting a write (speculation's commit).
    pub(super) fn extend_tail(&mut self, tail: Vec<T>) {
        self.0.extend(tail);
    }
}

impl<T, const I: usize> From<T> for Probed<T, I> {
    fn from(value: T) -> Self {
        Probed(value)
    }
}

impl<T, const I: usize> std::ops::Deref for Probed<T, I> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T, const I: usize> std::ops::DerefMut for Probed<T, I> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        write_at(I);
        &mut self.0
    }
}

// `iter` and `iter_mut` are the wrapped table's, reached through `Deref`.
#[allow(clippy::into_iter_without_iter)]
impl<'a, T, const I: usize> IntoIterator for &'a Probed<T, I>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

// `iter` and `iter_mut` are the wrapped table's, reached through `Deref`.
#[allow(clippy::into_iter_without_iter)]
impl<'a, T, const I: usize> IntoIterator for &'a mut Probed<T, I>
where
    &'a mut T: IntoIterator,
{
    type Item = <&'a mut T as IntoIterator>::Item;
    type IntoIter = <&'a mut T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        write_at(I);
        (&mut self.0).into_iter()
    }
}

impl<T: IntoIterator, const I: usize> IntoIterator for Probed<T, I> {
    type Item = T::Item;
    type IntoIter = T::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: std::fmt::Debug, const I: usize> std::fmt::Debug for Probed<T, I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// What indexing a vector with `P` yields.
type Reached<T, P> = <P as std::slice::SliceIndex<[T]>>::Output;

impl<T, P: std::slice::SliceIndex<[T]> + First, const I: usize> std::ops::Index<P>
    for Probed<Vec<T>, I>
{
    type Output = P::Output;

    #[inline(always)]
    fn index(&self, at: P) -> &Reached<T, P> {
        &self.0[at]
    }
}

/// Indexing for assignment also notes whether the element existed before the body began: those
/// writes touch something another body could be reading.
impl<T, P: std::slice::SliceIndex<[T]> + First, const I: usize> std::ops::IndexMut<P>
    for Probed<Vec<T>, I>
{
    #[inline(always)]
    fn index_mut(&mut self, at: P) -> &mut Reached<T, P> {
        write_at_index(I, at.first());
        old_write(I, at.first());
        &mut self.0[at]
    }
}

/// The first element an index reaches.
pub trait First {
    fn first(&self) -> usize;
}

impl First for usize {
    fn first(&self) -> usize {
        *self
    }
}

impl First for std::ops::Range<usize> {
    fn first(&self) -> usize {
        self.start
    }
}

impl First for std::ops::RangeFrom<usize> {
    fn first(&self) -> usize {
        self.start
    }
}

impl First for std::ops::RangeTo<usize> {
    fn first(&self) -> usize {
        0
    }
}

impl First for std::ops::RangeFull {
    fn first(&self) -> usize {
        0
    }
}

impl First for std::ops::RangeInclusive<usize> {
    fn first(&self) -> usize {
        *self.start()
    }
}

impl First for std::ops::RangeToInclusive<usize> {
    fn first(&self) -> usize {
        0
    }
}

thread_local! {
    /// The lengths of the procedure, entity and scope tables when the innermost body began.
    static LIMITS: std::cell::Cell<[usize; 3]> = const { std::cell::Cell::new([0, 0, usize::MAX]) };
}

fn old_write(table: usize, at: usize) {
    if !COUNTING || !enabled() {
        return;
    }
    let limits = LIMITS.with(std::cell::Cell::get);
    if table == W::PrProcs as usize {
        // The body's own record changes as it is lowered; any other procedure's is shared.
        if at != limits[2] {
            write(W::OldProcs);
        }
    } else if table == W::NsEntities as usize && at < limits[1] {
        write(W::OldEntities);
    }
}

/// A write through `scope_mut`: the scopes of modules, files and structs are shared, the
/// scopes of a body's blocks are its own.
pub fn scope_written(kind: ScopeKind) {
    match kind {
        ScopeKind::Root | ScopeKind::Module | ScopeKind::File => write(W::OldModuleScopes),
        ScopeKind::Struct(_) | ScopeKind::StructParams => write(W::OldStructScopes),
        ScopeKind::Proc | ScopeKind::Block | ScopeKind::Macro => {}
    }
}

/// Count one in-place write to state that is not a `Probed` table (a counter, a flag, a
/// `RefCell` behind a shared reference).
#[inline(always)]
pub fn write(w: W) {
    write_at(w as usize);
}

/// Whether in-place writes are counted at all: the `body-probe` feature of `jaic`. Without it
/// `Probed` is a plain wrapper and every hook compiles to nothing, so a normal build pays no
/// instructions for the audit.
pub const COUNTING: bool = cfg!(feature = "body-probe");

#[inline(always)]
fn write_at(i: usize) {
    if super::speculative::BUILT {
        super::speculative::write_hit(i, None);
    }
    if COUNTING && ON.load(std::sync::atomic::Ordering::Relaxed) {
        WRITES.with(|w| w.borrow_mut()[i] += 1);
    }
}

/// [`write_at`] for a write through an index: speculation lets the elements a body made itself
/// be changed.
#[inline(always)]
fn write_at_index(i: usize, at: usize) {
    if super::speculative::BUILT {
        super::speculative::write_hit(i, Some(at));
    }
    if COUNTING && ON.load(std::sync::atomic::Ordering::Relaxed) {
        WRITES.with(|w| w.borrow_mut()[i] += 1);
    }
}

static ON: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Read the knob once; `Compiler::new` calls this.
pub fn init() {
    ON.store(
        crate::knobs::get().body_probe,
        std::sync::atomic::Ordering::Relaxed,
    );
}

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

const N0: usize = 23;
const N: usize = N0 + WN;

const NAMES: [&str; N0] = [
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
    touched: Vec<u64>,
    /// Writes made by the bodies themselves (nested bodies excluded), per `W`.
    writes: Vec<u64>,
    /// Per mask of touched sets: bodies, self time.
    masks: [(u64, Duration); 32],
    total: Duration,
    /// Body that was being lowered when each body was queued (`u32::MAX`: none), and each
    /// body's own lowering time: the queue forms a tree whose longest path bounds any speedup.
    parent: crate::fxhash::HashMap<u32, u32>,
    own_time: crate::fxhash::HashMap<u32, Duration>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
    static WRITES: RefCell<[u64; WN]> = const { RefCell::new([0; WN]) };
}

#[inline(always)]
pub fn enabled() -> bool {
    ON.load(std::sync::atomic::Ordering::Relaxed)
}

/// Count one lazy resolution (`C::EntResolve`, `C::SigResolve` or `C::Layout`).
pub fn bump(c: C) {
    if enabled() {
        STATE.with(|s| s.borrow_mut().bumps[c as usize - C::EntResolve as usize] += 1);
    }
}

fn snapshot(c: &Compiler, bumps: [u64; 3], nested: u64) -> [u64; N] {
    let mut v = [0u64; N];
    v[C::Types as usize] = c.ty.types.type_count() as u64;
    v[C::Structs as usize] = c.ty.types.structs.len() as u64;
    v[C::Strings as usize] = c.out.strings.len() as u64;
    v[C::Packs as usize] = c.out.packs.len() as u64;
    v[C::ConstAddr as usize] = c.out.const_addr_globals.len() as u64;
    v[C::TypeInfos as usize] = c.ty.infos.len() as u64;
    v[C::AnonTypes as usize] = c.ty.anonymous.len() as u64;
    v[C::Procs as usize] = c.pr.procs.len() as u64;
    v[C::Funcs as usize] = c.out.program.funcs.len() as u64;
    v[C::AnonProcs as usize] = c.pr.anonymous_procs.len() as u64;
    v[C::Globals as usize] = c.out.program.globals.len() as u64;
    v[C::Foreigns as usize] = c.out.program.foreigns.len() as u64;
    v[C::Modules as usize] = c.ns.modules.len() as u64;
    v[C::Files as usize] = c.src.files.len() as u64;
    v[C::NamespaceGen as usize] = c.epochs.namespace_generation;
    v[C::CtExec as usize] = c.ct.executions;
    v[C::EntResolve as usize] = bumps[0];
    v[C::SigResolve as usize] = bumps[1];
    v[C::Layout as usize] = bumps[2];
    v[C::Nested as usize] = nested;
    v[C::Entities as usize] = c.ns.entities.len() as u64;
    v[C::Scopes as usize] = c.ns.scopes.len() as u64;
    v[C::Warnings as usize] = c.diag.warnings.len() as u64;
    WRITES.with(|w| v[N0..].copy_from_slice(&*w.borrow()));
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
        if s.touched.is_empty() {
            s.touched = vec![0; N];
            s.writes = vec![0; WN];
        }
        s.bodies += 1;
        let start = snapshot(c, s.bumps, s.bodies);
        LIMITS.with(|l| l.set([0, start[C::Entities as usize] as usize, id as usize]));
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
        LIMITS.with(|l| {
            l.set(s.frames.last().map_or([0, 0, usize::MAX], |p| {
                [0, p.start[C::Entities as usize] as usize, p.id as usize]
            }));
        });
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
        for i in 0..WN {
            s.writes[i] += own[N0 + i];
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
        for i in 0..N0 {
            eprintln!("  {:<20} {:>8} ({:.1}%)", NAMES[i], s.touched[i], 100.0 * s.touched[i] as f64 / s.bodies.max(1) as f64);
        }
        if COUNTING {
            eprintln!("-- in-place writes (mutable borrows of shared state) made while a body is lowered:");
            eprintln!("  {:<30} {:>8} bodies ({:>5}) {:>12} writes", "state", "wrote", "%", "total");
            let mut rows: Vec<_> = (0..WN).filter(|&i| s.touched[N0 + i] > 0).collect();
            rows.sort_by_key(|&i| std::cmp::Reverse(s.touched[N0 + i]));
            for i in rows {
                eprintln!(
                    "  {:<30} {:>8} bodies ({:>4.1}%) {:>12}",
                    WRITE_NAMES[i],
                    s.touched[N0 + i],
                    100.0 * s.touched[N0 + i] as f64 / s.bodies.max(1) as f64,
                    s.writes[i]
                );
            }
            let quiet: Vec<_> = (0..WN).filter(|&i| s.touched[N0 + i] == 0).map(|i| WRITE_NAMES[i]).collect();
            eprintln!("  never written by a body: {}", quiet.join(", "));
        } else {
            eprintln!("-- in-place writes were not counted: build jaic with `--features body-probe`");
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
        let mut memo: crate::fxhash::HashMap<u32, (Duration, u32)> = crate::fxhash::HashMap::default();
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

/// The next queued body: the last queued, or with `-dev_body_fifo` the first (a study of
/// how much the numbering of ids and functions depends on the order bodies are lowered in).
pub fn next_body(queue: &mut Vec<super::ProcId>) -> Option<super::ProcId> {
    static FIFO: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *FIFO.get_or_init(|| crate::knobs::get().body_fifo) {
        (!queue.is_empty()).then(|| queue.remove(0))
    } else {
        queue.pop()
    }
}
