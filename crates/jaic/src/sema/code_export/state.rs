//! State kept across exports: modified statements, expression types and the pending procedures.
use crate::ide::Recorder;
use crate::sema::{
    Arc, EntityId, FileId, HashMap, HashSet, ModuleId, PathBuf, ProcId, ScopeId, Span, Sym, TypeId,
    ast,
};

/// A body given to `compiler_modify_procedure`: the body record, its new statements and the
/// text their spans point into (built under `PLACEHOLDER_FILE`).
#[derive(Debug, Clone)]
pub struct Modification {
    pub body: i64,
    pub stmts: Vec<ast::Stmt>,
    pub source: Arc<str>,
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
    /// Procedures whose header went out but whose body is not lowered yet:
    /// (procedure, header record, scope for type expressions). The body is reported
    /// in a later TYPECHECKED message, once lowered (like Jai, unused bodies never are).
    pub(super) pending_bodies: Vec<(ProcId, i64, ScopeId)>,
    /// A procedure may be reached through several aliases and call records.
    pub(super) scheduled_bodies: HashSet<ProcId>,
    pub(super) reported_headers: HashSet<ProcId>,
    /// Procedures whose header went out in a TYPECHECKED message without their body (nothing
    /// could read it, see `Contents::WithoutBodies`), and the scope their body is exported in.
    pub(super) bodiless: Vec<(ProcId, ScopeId)>,
    /// Set while the export leaves procedure bodies out of their headers.
    pub(super) omit_bodies: bool,
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
    /// Checked constants and defaults owned by a file or concrete struct scope.
    pub(in crate::sema) scope_expression_types: HashMap<ScopeId, ExpressionTypes>,
    pub(super) code_expression_types: HashMap<usize, ExpressionTypes>,
}

#[derive(Clone, Copy)]
pub(in crate::sema) enum ExpressionOwner {
    Procedure(ProcId),
    Scope(ScopeId),
    Code(usize),
}

/// The types a body's expressions checked to, in checking order. Only a metaprogram reading
/// the body's code asks for them, so checking just appends; the by-span table behind
/// `get` is built on a read, and again if more were recorded since.
#[derive(Default)]
pub(in crate::sema) struct ExpressionTypes {
    log: Vec<(Span, TypeId, bool)>,
    table: std::cell::RefCell<(usize, HashMap<Span, ExpressionTypeHistory>)>,
}

struct ExpressionTypeHistory {
    previous: Option<TypeId>,
    current: TypeId,
    ambiguous: bool,
}

impl ExpressionTypes {
    pub(in crate::sema) fn record(&mut self, span: Span, ty: TypeId) {
        self.log.push((span, ty, false));
    }

    /// Settle the most recent check, keeping earlier expansions separate. Logging remains
    /// constant-time; the exporter builds the lookup only when a metaprogram reads it.
    pub(in crate::sema) fn refine(&mut self, span: Span, ty: TypeId) {
        self.log.push((span, ty, true));
    }

    /// `Some(None)`: completed checks at this span have incompatible contextual types.
    pub(in crate::sema) fn get(&self, span: Span) -> Option<Option<TypeId>> {
        let mut table = self.table.borrow_mut();
        if table.0 != self.log.len() {
            let (done, map) = &mut *table;
            for &(span, ty, refine) in &self.log[*done..] {
                if refine {
                    if let Some(history) = map.get_mut(&span) {
                        history.current = ty;
                    }
                } else {
                    map.entry(span)
                        .and_modify(|history| {
                            history.ambiguous |=
                                history.previous.is_some_and(|t| t != history.current);
                            history.previous = Some(history.current);
                            history.current = ty;
                        })
                        .or_insert(ExpressionTypeHistory {
                            previous: None,
                            current: ty,
                            ambiguous: false,
                        });
                }
            }
            *done = self.log.len();
        }
        table.1.get(&span).map(|history| {
            (!history.ambiguous && history.previous.is_none_or(|t| t == history.current))
                .then_some(history.current)
        })
    }
}

impl ExportState {
    pub(in crate::sema) fn expression_types_mut(
        &mut self,
        owner: ExpressionOwner,
    ) -> &mut ExpressionTypes {
        match owner {
            ExpressionOwner::Procedure(p) => self.expression_types.entry(p).or_default(),
            ExpressionOwner::Scope(scope) => self.scope_expression_types.entry(scope).or_default(),
            ExpressionOwner::Code(code) => self.code_expression_types.entry(code).or_default(),
        }
    }

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

impl crate::sema::Compiler {
    /// Retain actual semantic results; exporting syntax must not recheck it in another scope.
    pub(in crate::sema) fn record_expression_type(
        &mut self,
        f: &crate::sema::lower::FnCtx,
        span: Span,
        ty: TypeId,
        refine: bool,
    ) {
        if (f.type_only && f.expression_code.is_none())
            || self.ed.sketching()
            || ty == TypeId::COMPILE_TIME
        {
            return;
        }
        let mut procedure = f.proc;
        let mut owner = f.expression_scope;
        if procedure.is_none() {
            while let Some(scope) = owner {
                let info = self.scope(scope);
                use crate::sema::scope::ScopeKind;
                match info.kind {
                    ScopeKind::Struct(_)
                    | ScopeKind::StructParams
                    | ScopeKind::File
                    | ScopeKind::Module
                    | ScopeKind::Root => break,
                    ScopeKind::Proc if info.proc.is_some() => {
                        procedure = info.proc;
                        break;
                    }
                    _ => owner = info.parent,
                }
            }
        }
        let owner = if let Some(code) = f.expression_code {
            ExpressionOwner::Code(code)
        } else if let Some(procedure) = procedure {
            ExpressionOwner::Procedure(procedure)
        } else if let Some(scope) = owner {
            ExpressionOwner::Scope(scope)
        } else {
            return;
        };
        if crate::sema::speculative::log_expression(owner, span, ty, refine) {
            return;
        }
        let types = self.meta.export.expression_types_mut(owner);
        if refine {
            types.refine(span, ty);
        } else {
            types.record(span, ty);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contextual_refinements_preserve_expansion_ambiguity_across_reads() {
        let mut types = ExpressionTypes::default();
        let at = Span::NONE;
        types.record(at, TypeId::S64);
        types.refine(at, TypeId::U8);
        assert_eq!(types.get(at), Some(Some(TypeId::U8)));
        types.record(at, TypeId::S64);
        types.refine(at, TypeId::U8);
        assert_eq!(types.get(at), Some(Some(TypeId::U8)));
        types.record(at, TypeId::S64);
        types.refine(at, TypeId::U16);
        assert_eq!(types.get(at), Some(None));
        types.refine(at, TypeId::U8);
        assert_eq!(types.get(at), Some(Some(TypeId::U8)));
        types.record(at, TypeId::U16);
        types.record(at, TypeId::U8);
        assert_eq!(types.get(at), Some(None));
    }
}
