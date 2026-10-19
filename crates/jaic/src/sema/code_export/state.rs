//! State kept across exports: modified statements, expression types and the pending procedures.
use crate::sema::{
    EntityId, FileId, HashMap, HashSet, ModuleId, PathBuf, ProcId, Rc, ScopeId, Span, Sym, TypeId,
    ast,
};

/// A statement of a body given to `compiler_modify_procedure`.
#[derive(Debug, Clone)]
pub enum ModifiedStmt {
    /// A statement the compiler exported, unchanged.
    Record(i64),
    /// A new or edited statement, printed by the metaprogram.
    Source(String),
}

/// Exporter state kept by a workspace's compiler between messages.
#[derive(Default)]
pub struct ExportState {
    pub(super) files: HashMap<FileId, i64>,
    pub(super) modules: HashMap<ModuleId, i64>,
    pub(super) types: HashMap<TypeId, i64>,
    pub(super) files_announced: usize,
    /// Entities before this index were considered for TYPECHECKED messages.
    pub(super) next_entity: usize,
    /// Directory of the system modules (`None` inside: not known).
    pub(super) system_dir: Option<Option<PathBuf>>,
    pub(super) system_modules: HashMap<ModuleId, bool>,
    /// Procedure of each exported procedure-body record.
    pub bodies: HashMap<i64, ProcId>,
    /// Statement behind each exported statement record.
    pub stmts: HashMap<i64, Rc<ast::Stmt>>,
    /// Procedures whose header went out but whose body is not lowered yet:
    /// (procedure, header record, scope for type expressions). The body is reported
    /// in a later TYPECHECKED message, once lowered (like Jai, unused bodies never are).
    pub(super) pending_bodies: Vec<(ProcId, i64, ScopeId)>,
    /// A procedure may be reached through several aliases and call records.
    pub(super) scheduled_bodies: HashSet<ProcId>,
    pub(super) reported_headers: HashSet<ProcId>,
    pub(super) reported_bodies: HashSet<ProcId>,
    /// Records of this compiler's own type descriptors (`__address`), by type.
    pub(super) own_types: HashMap<TypeId, i64>,
    /// Modified procedures to lower again (`relower_modified`).
    pub(super) relower: Vec<ProcId>,
    /// The header record of each procedure (shared by `resolved_procedure_expression` and
    /// reported headers), by procedure and whether it is for this compiler's own
    /// compile-time code (`Exporter::own`: those types carry real descriptors).
    pub(super) resolved_headers: HashMap<(ProcId, bool), i64>,
    /// Declarations made for `resolved_declaration` of names outside the exported tree.
    pub(super) resolved_decls: HashMap<EntityId, i64>,
    /// What each name resolved to in each scope, so an identifier used many times in a body is
    /// looked up once. Dropped whenever an entity has been added since, as a new declaration
    /// (from a metaprogram) can change what a name means.
    pub(super) name_lookups: HashMap<(ScopeId, Sym), Option<EntityId>>,
    /// The module of the file the last exported node was in.
    pub(super) last_file_module: Option<(FileId, ModuleId)>,
    /// The type a plain identifier names in a scope, by the same rules.
    pub(super) type_names: HashMap<(ScopeId, Sym), Option<TypeId>>,
    /// The entity count the cached lookups were made at.
    name_lookups_at: usize,
    /// The polymorph instances made for each call (by the call's span), so a call's
    /// `resolved_procedure_expression` is the instance it called, not the generic procedure.
    pub call_instances: HashMap<Span, Vec<ProcId>>,
    /// Checked results, scoped to a concrete procedure instance. Repeated source
    /// spans with incompatible expansion types are ambiguous rather than guessed.
    pub(in crate::sema) expression_types: HashMap<ProcId, ExpressionTypes>,
}

/// The types a body's expressions checked to, in checking order. Only a metaprogram reading
/// the body's code asks for them, so checking just appends; the by-span table behind
/// `get` is built on a read, and again if more were recorded since.
#[derive(Default)]
pub(in crate::sema) struct ExpressionTypes {
    log: Vec<(Span, TypeId)>,
    table: std::cell::RefCell<(usize, HashMap<Span, Option<TypeId>>)>,
}

impl ExpressionTypes {
    pub(in crate::sema) fn record(&mut self, span: Span, ty: TypeId) {
        self.log.push((span, ty));
    }

    /// `Some(None)`: the span checked to different types in different expansions.
    pub(in crate::sema) fn get(&self, span: Span) -> Option<Option<TypeId>> {
        let mut table = self.table.borrow_mut();
        if table.0 != self.log.len() {
            let (done, map) = &mut *table;
            for &(span, ty) in &self.log[*done..] {
                map.entry(span)
                    .and_modify(|known| {
                        if *known != Some(ty) {
                            *known = None;
                        }
                    })
                    .or_insert(Some(ty));
            }
            *done = self.log.len();
        }
        table.1.get(&span).copied()
    }
}

impl ExportState {
    /// Forget cached name lookups when entities were added since they were made.
    pub(super) fn refresh_name_caches(&mut self, entities: usize) {
        if self.name_lookups_at != entities {
            self.name_lookups.clear();
            self.type_names.clear();
            self.name_lookups_at = entities;
        }
    }

    pub(super) fn schedule_body(&mut self, p: ProcId, header: i64, scope: ScopeId) {
        if self.scheduled_bodies.insert(p) {
            self.pending_bodies.push((p, header, scope));
        }
    }

    /// Procedures whose header was reported and whose body is still waiting.
    pub fn pending_procs(&self) -> impl Iterator<Item = ProcId> + '_ {
        self.pending_bodies.iter().map(|&(p, _, _)| p)
    }
}

/// A FILE or IMPORT message: (event kind, message record).
pub type FileEvent = (i64, i64);
