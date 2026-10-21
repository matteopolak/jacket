//! Recording references, entities and missing modules as the checker visits them.
use super::facts::{
    IdeCast, IdeExprFact, IdeMissingModule, IdeMissingModuleEpoch, IdeRef, IdeRoot, IdeWhat,
};
use crate::ast;
use crate::sema::lower::Operand;
use crate::sema::scope::{EntityId, EntityKind, ScopeId};
use crate::sema::value::ProcId;
use crate::sema::{Compiler, ModuleId, Result};
use crate::source::Diagnostic;
use crate::source::FileId;
use crate::source::SourcePath;
use crate::source::Span;
use crate::types::TypeId;
use crate::types::TypeKind;
use std::path::Path;
use std::rc::Rc;

impl Compiler {
    fn ide_missing_module_epoch(&self) -> IdeMissingModuleEpoch {
        (
            self.src.added,
            self.src.files.len(),
            self.ns.modules.len(),
            self.ct.runs_done,
            self.ct.executions,
        )
    }

    /// Only filesystem discovery misses are reused. Import operands, remaps and
    /// supplied-import rules still use their ordinary resolver paths.
    pub(crate) fn ide_missing_module(
        &self,
        scope: ScopeId,
        span: Span,
        wanted: &str,
        dir: &Path,
    ) -> Option<Box<Diagnostic>> {
        // A metaprogram can change files between two requests in the same run.
        if self.ct.interp.run_effects.is_some() {
            return None;
        }
        let cached = self.ed.facts()?.missing_modules.get(&(scope, span))?;
        (cached.epoch == self.ide_missing_module_epoch()
            && cached.wanted == wanted
            && cached.dir == dir
            && cached.import_paths == self.options.import_paths
            && cached.preload == self.options.preload)
            .then(|| cached.error.clone())
    }

    pub(crate) fn ide_record_missing_module(
        &mut self,
        scope: ScopeId,
        span: Span,
        wanted: &str,
        dir: &Path,
        error: &Diagnostic,
    ) {
        if !self.ed.is_live() || self.ct.interp.run_effects.is_some() {
            return;
        }
        let cached = IdeMissingModule {
            epoch: self.ide_missing_module_epoch(),
            wanted: wanted.into(),
            dir: dir.into(),
            import_paths: self.options.import_paths.clone(),
            preload: self.options.preload.clone(),
            error: Box::new(error.clone()),
        };
        self.ed
            .facts_mut()
            .unwrap()
            .missing_modules
            .insert((scope, span), cached);
    }

    /// The failure `id` had at `epoch` still stands, so lowering it again is not worth it.
    /// A build trusts the epoch; an editor also needs the body's text unchanged (and no
    /// sketch running).
    pub(crate) fn failure_reusable(&self, id: ProcId, epoch: u64) -> bool {
        !self.ed.is_live() || self.ide_cached_body_failure(id, epoch).is_some()
    }

    pub(crate) fn ide_cached_body_failure(
        &self,
        id: ProcId,
        epoch: u64,
    ) -> Option<Box<Diagnostic>> {
        let ide = self.ed.facts()?;
        if !self.epochs.placeholders_final || ide.sketching {
            return None;
        }
        let (at, error) = self.pr.lenient_failures.get(&id)?;
        (*at == epoch && Rc::ptr_eq(ide.failed_body_lits.get(&id)?, &self.proc(id).lit))
            .then(|| error.clone())
    }

    pub(crate) fn ide_record_body_result(
        &mut self,
        id: ProcId,
        epoch: u64,
        misses: (u64, u64),
        result: &Result<()>,
    ) {
        if !self.epochs.placeholders_final || self.ed.facts().is_none_or(|ide| ide.sketching) {
            return;
        }
        match result {
            Ok(()) => {
                self.pr.lenient_failures.remove(&id);
                self.ed.facts_mut().unwrap().failed_body_lits.remove(&id);
            }
            Err(error)
                if (
                    self.epochs.placeholder_misses,
                    self.epochs.in_progress_misses,
                ) == misses =>
            {
                let lit = self.proc(id).lit.clone();
                self.pr.lenient_failures.insert(id, (epoch, error.clone()));
                self.ed
                    .facts_mut()
                    .unwrap()
                    .failed_body_lits
                    .insert(id, lit);
            }
            Err(_) => {}
        }
    }

    pub(crate) fn ide_wants_pub(&mut self, file: FileId) -> bool {
        self.ide_wants(file)
    }

    pub(super) fn ide_wants(&mut self, file: FileId) -> bool {
        let Some(ide) = self.ed.facts_mut() else {
            return false;
        };
        if let Some(&w) = ide.wanted.get(&file) {
            return w;
        }
        // Builtins such as `OS` are declared in no file (`Span::NONE`).
        let Some(source) = self.src.sources.try_get(file) else {
            return false;
        };
        let w = match &source.path {
            SourcePath::Virtual(_) => ide.roots.contains(&IdeRoot::Virtual),
            SourcePath::File(path) => {
                // Identity, not spelling: `./a`, `a//b/../b`, a symlink or another case of a
                // folder name one file.
                let key = self.src.fs.canonical(path);
                ide.roots.iter().any(|r| match r {
                    IdeRoot::Under(root) => path.starts_with(root) || key.starts_with(root),
                    IdeRoot::Virtual => false,
                }) || ide.files.contains(&key)
            }
        };
        ide.wanted.insert(file, w);
        w
    }

    pub(crate) fn ide_note_expr(&mut self, expr: &ast::Expr, op: &Operand) {
        if self.ed.facts().is_some_and(|i| i.lint) && self.ide_wants(expr.span.file) {
            let ty = op.ty();
            let constant = matches!(op, Operand::Const { .. });
            if let Some(ide) = self.ed.facts_mut() {
                ide.exprs
                    .entry(expr.span)
                    .and_modify(|f| f.conflicting |= f.ty != ty || f.constant != constant)
                    .or_insert(IdeExprFact {
                        ty,
                        constant,
                        conflicting: false,
                    });
            }
        }
        if self.ed.is_live() {
            self.ide_note_ainfo(expr, op);
        }
        if let ast::ExprKind::Call {
            callee, ..
        } = &expr.kind
            && let Operand::Type(t) = op
            && matches!(self.ty.types.kind(*t), TypeKind::Struct(_))
        {
            let name = match &callee.kind {
                ast::ExprKind::Ident(_) => Some(callee.span),
                ast::ExprKind::Member(_, name) => Some(name.span),
                _ => None,
            };
            if let Some(span) = name
                && self.ide_wants(span.file)
            {
                self.ide_note_instance(span, *t);
            }
            return;
        }
        let (span, what) = match &expr.kind {
            ast::ExprKind::Ident(_) | ast::ExprKind::Member(_, _) => {
                let (span, qualified_member) = match &expr.kind {
                    ast::ExprKind::Member(_, name) => (name.span, Some(name.name)),
                    _ => (expr.span, None),
                };
                // Failed expressions can leave scratch state behind. Only consume
                // the association made for this exact source expression.
                let entity = self
                    .ed
                    .facts_mut()
                    .and_then(|i| i.last_entity.take())
                    .and_then(|(at, id)| (at == span).then_some(id));
                let using_member = self.ed.facts_mut().and_then(|i| i.last_using_member.take());
                let member = qualified_member.or(using_member);
                let what = match op {
                    Operand::Type(t) => {
                        if let Some(e) = entity
                            && let Some(ide) = self.ed.facts_mut()
                        {
                            ide.type_names.insert(span, e);
                        }
                        IdeWhat::Type(*t)
                    }
                    Operand::Procs(p) => IdeWhat::Procs(p.clone()),
                    Operand::Module(m) => IdeWhat::Module(*m),
                    _ => match (entity, member) {
                        (Some(e), _) => IdeWhat::Entity(e),
                        (None, Some(m)) => IdeWhat::Member(m),
                        (None, None) => return,
                    },
                };
                (span, what)
            }
            ast::ExprKind::InferredMember(name) => (name.span, IdeWhat::Member(name.name)),
            _ => return,
        };
        if !self.ide_wants(span.file) {
            return;
        }
        let ty = op.ty();
        if let Some(ide) = self.ed.facts_mut() {
            ide.refs.push(IdeRef {
                span,
                what,
                ty,
                decl: false,
            });
        }
    }

    /// The name after `for :` names the `for_expansion` procedures the loop expands through,
    /// like any other use of a procedure name.
    pub(crate) fn ide_note_for_iterator(&mut self, name: &ast::Ident, procs: &[ProcId]) {
        if self.ed.sketching() || !self.ide_wants(name.span.file) {
            return;
        }
        if let Some(ide) = self.ed.facts_mut() {
            ide.refs.push(IdeRef {
                span: name.span,
                what: IdeWhat::Procs(procs.to_vec()),
                ty: TypeId::COMPILE_TIME,
                decl: false,
            });
        }
    }

    /// Lint mode: `cast` at `span` converted a `from` operand to `target`.
    pub(crate) fn ide_note_cast(&mut self, span: Span, target: TypeId, op: &Operand) {
        if !self.ed.facts().is_some_and(|i| i.lint) || !self.ide_wants(span.file) {
            return;
        }
        let from = op.ty();
        let from_constant = matches!(op, Operand::Const { .. });
        if let Some(ide) = self.ed.facts_mut() {
            ide.casts
                .entry(span)
                .and_modify(|c| {
                    c.conflicting |=
                        c.target != target || c.from != from || c.from_constant != from_constant;
                })
                .or_insert(IdeCast {
                    target,
                    from,
                    from_constant,
                    conflicting: false,
                });
        }
    }

    /// Lint mode: a lookup found a name through import `index` of `scope`.
    pub(crate) fn ide_note_import_use(&mut self, scope: ScopeId, index: usize) {
        if let Some(ide) = self.ed.facts_mut()
            && ide.lint
        {
            ide.used_imports.insert((scope, index));
        }
    }

    /// Lint mode: a name resolved to `id`.
    pub(crate) fn ide_note_use(&mut self, id: EntityId) {
        if let Some(ide) = self.ed.facts_mut()
            && ide.lint
        {
            ide.used.insert(id);
        }
    }

    /// Where entity `id` names itself: its span narrowed to the name (an entity's span may cover
    /// the whole declaration). `None` for an entity written nowhere.
    pub(crate) fn ide_entity_name_span(&self, id: EntityId) -> Option<Span> {
        let e = self.entity(id);
        let (span, name) = (e.span, e.name);
        if span.end <= span.start || (span.file.0 as usize) >= self.src.sources.len() {
            return None;
        }
        let text = self.src.sources.snippet(span);
        let at = text.find(name.as_str())?;
        let start = span.start + at as u32;
        Some(Span {
            file: span.file,
            start,
            end: start + name.as_str().len() as u32,
        })
    }

    /// A declaration: hovering its name shows it.
    pub(crate) fn ide_note_entity(&mut self, id: EntityId) {
        let Some(span) = self.ide_entity_name_span(id) else {
            return;
        };
        if !self.ide_wants(span.file) {
            return;
        }
        let ty = match &self.entity(id).kind {
            EntityKind::Local {
                ty, ..
            }
            | EntityKind::Const {
                ty, ..
            } => *ty,
            _ => TypeId::COMPILE_TIME,
        };
        if let Some(ide) = self.ed.facts_mut() {
            ide.refs.push(IdeRef {
                span,
                what: IdeWhat::Entity(id),
                ty,
                decl: true,
            });
        }
    }

    /// A declaration whose name is written at `name`: hovering it shows the entity. A name the
    /// source does not write (a loop's `it`) has no declaration to show.
    pub(crate) fn ide_note_entity_at(&mut self, id: EntityId, name: Option<Span>) {
        let Some(span) = name else {
            return;
        };
        if !self.ide_wants(span.file) {
            return;
        }
        let ty = match &self.entity(id).kind {
            EntityKind::Local {
                ty, ..
            }
            | EntityKind::Const {
                ty, ..
            } => *ty,
            _ => TypeId::COMPILE_TIME,
        };
        if let Some(ide) = self.ed.facts_mut() {
            ide.refs.push(IdeRef {
                span,
                what: IdeWhat::Entity(id),
                ty,
                decl: true,
            });
        }
    }

    /// A semantic identifier resolved through a binding path that does not lower
    /// an ordinary expression (for example a using target or module argument label).
    pub(crate) fn ide_note_entity_use(&mut self, span: Span, id: EntityId, ty: TypeId) {
        if !self.ide_wants(span.file) {
            return;
        }
        if let Some(ide) = self.ed.facts_mut() {
            ide.refs.push(IdeRef {
                span,
                what: IdeWhat::Entity(id),
                ty,
                decl: false,
            });
        }
    }

    pub(crate) fn ide_note_module_arguments(&mut self, module: ModuleId, import: &ast::Import) {
        if !self.ed.is_live() {
            return;
        }
        let parameters = self.ns.modules[module.0 as usize].param_entities.clone();
        for arg in &import.params {
            let Some(name) = arg.name else {
                continue;
            };
            let Some(&id) = parameters
                .iter()
                .find(|&&id| self.entity(id).name == name.name)
            else {
                continue;
            };
            let ty = match self.entity(id).kind {
                EntityKind::Const {
                    ty, ..
                }
                | EntityKind::Local {
                    ty, ..
                } => ty,
                _ => TypeId::COMPILE_TIME,
            };
            self.ide_note_entity_use(name.span, id, ty);
        }
    }

    /// `scope` covers `span` of the source (a block, a procedure body).
    pub(crate) fn ide_scope_span(&mut self, scope: ScopeId, span: Span) {
        if span.end > span.start
            && self.ide_wants(span.file)
            && let Some(ide) = self.ed.facts_mut()
        {
            ide.scopes.push((span, scope));
        }
    }

    /// Recovered bodies supply facts but cannot be executed as valid procedures or code.
    pub(crate) fn ide_require_parsed_body(&self, span: Span) -> Result<()> {
        if let Some(error) = self.ed.facts().and_then(|ide| {
            ide.parse_errors.iter().find(|e| {
                e.span.file == span.file && span.start <= e.span.start && e.span.start < span.end
            })
        }) {
            return Err(Box::new(error.clone()));
        }
        Ok(())
    }
}
