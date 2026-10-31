//! The compiler's handle on an editor analysis. A batch compile has an idle `Editor`: every
//! event method returns at once, so recording costs the checker one branch per site.
use super::interval::Interval;
use super::values::Narrowing;
use super::{IdeFacts, Recorder};
use crate::ast::{self, BinOp};
use crate::diagnostics::Code;
use crate::diagnostics::Diagnostic;
use crate::intern::Sym;
use crate::parser::{ParseReuse, SharedParse};
use crate::sema::{EntityId, ProcId, Result};
use crate::source::{FileId, Span};
use crate::types::TypeId;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// What checking does after a statement of a block failed (`Editor::stmt_failed`).
pub(crate) enum StmtRecovery {
    /// Stop: the body fails with the error.
    Propagate,
    /// Record the error and go on with the next statement.
    Continue,
    /// Also declare the names the failed declaration did not.
    Declare,
    /// Also check what the statement did not get to and declare its names: compile-time
    /// code it needed was refused, so its values are unknown, not wrong.
    Salvage,
}

#[derive(Default)]
pub struct Editor {
    /// What the analysis has recorded so far; `None` outside the language server and linter.
    facts: Option<Box<IdeFacts>>,
    /// Lookups over `facts` and the compiler's tables, built on first use (see `span_index`).
    pub(super) index: RefCell<super::span_index::SpanIndex>,
    /// Contextual type probes may inspect signatures but must never execute code.
    pub(crate) type_probe: bool,
    /// Parses shared with other compiles, and this compile's number in it (see
    /// `ParseReuse::begin`). Only read while recording.
    pub parse_reuse: Option<(Rc<RefCell<ParseReuse>>, u64)>,
    /// The entry file the editor picked, admitted for inspection without authorising imports.
    pub(crate) selected_entry: Option<PathBuf>,
}

impl Editor {
    /// Starts recording into `facts`.
    pub fn record(&mut self, facts: IdeFacts) {
        self.facts = Some(Box::new(facts));
        *self.index.get_mut() = super::span_index::SpanIndex::default();
    }

    /// Stops recording and hands back what was recorded.
    pub fn stop(&mut self) -> Option<Box<IdeFacts>> {
        *self.index.get_mut() = super::span_index::SpanIndex::default();
        self.facts.take()
    }

    /// Records into `facts` again, after `stop`.
    pub(crate) fn resume(&mut self, facts: Box<IdeFacts>) {
        self.facts = Some(facts);
        *self.index.get_mut() = super::span_index::SpanIndex::default();
    }

    #[inline]
    pub fn is_live(&self) -> bool {
        self.facts.is_some()
    }

    /// The entry file captured when sources were prepared (`Compiler::prepare_sources`):
    /// `None` for a string-only workspace, and never inferred from files added later.
    pub fn selected_entry(&self) -> Option<&Path> {
        self.selected_entry.as_deref()
    }

    #[inline]
    pub fn facts(&self) -> Option<&IdeFacts> {
        self.facts.as_deref()
    }

    #[inline]
    pub fn facts_mut(&mut self) -> Option<&mut IdeFacts> {
        self.facts.as_deref_mut()
    }
}

impl Recorder for Editor {
    #[inline]
    fn expr_checked(&mut self, span: Span, ok: bool) {
        if let Some(facts) = &mut self.facts {
            if ok {
                facts.failed_exprs.remove(&span);
            } else {
                facts.failed_exprs.insert(span);
            }
        }
    }

    #[inline]
    fn sketch_tried(&mut self, span: Span) {
        if let Some(facts) = self.facts.as_mut().filter(|facts| facts.sketching) {
            facts.sketch_tried.insert(span);
        }
    }

    fn sketch_checked(&self, span: Span) -> bool {
        self.facts()
            .is_some_and(|facts| facts.sketch_tried.contains(&span))
    }

    fn sketch_typed(&self, span: Span) -> bool {
        self.facts()
            .is_some_and(|facts| facts.sketch_types.contains_key(&span))
    }

    #[inline]
    fn named(&mut self, span: Span, ids: &[EntityId]) {
        if let Some(facts) = &mut self.facts {
            facts.last_entity = Some((span, ids[0]));
            if facts.lint {
                facts.used.extend(ids.iter().copied());
            }
        }
    }

    #[inline]
    fn resolved(&mut self, span: Span, id: EntityId) {
        if let Some(facts) = &mut self.facts {
            facts.last_entity = Some((span, id));
        }
    }

    #[inline]
    fn named_using_member(&mut self, member: Sym) {
        if let Some(facts) = &mut self.facts {
            facts.last_using_member = Some(member);
        }
    }

    fn run_failed(&mut self, span: Span, ty: TypeId) {
        if let Some(facts) = &mut self.facts {
            facts.failed_runs.insert(span, ty);
        }
    }

    fn failed_run_type(&self, span: Span) -> Option<TypeId> {
        self.facts()?.failed_runs.get(&span).copied()
    }

    fn inferred(&mut self, span: Span, ty: TypeId) {
        if let Some(facts) = &mut self.facts {
            let seen = facts.inferred_expected.entry(span).or_default();
            if !seen.contains(&ty) {
                seen.push(ty);
            }
        }
    }

    #[inline]
    fn macro_expanded(&mut self, proc: ProcId) {
        if let Some(facts) = &mut self.facts {
            facts.expanded_macros.insert(proc);
        }
    }

    fn macro_returned(&mut self, proc: ProcId) {
        if let Some(facts) = &mut self.facts {
            facts.macro_returners.insert(proc);
        }
    }

    #[inline]
    fn error_count(&self) -> usize {
        self.facts.as_ref().map_or(0, |facts| facts.errors.len())
    }

    fn errors_since(&self, count: usize) -> Vec<Diagnostic> {
        self.facts.as_ref().map_or_else(Vec::new, |facts| {
            facts.errors.get(count..).unwrap_or(&[]).to_vec()
        })
    }

    fn forget_matching(&mut self, errors: &[Diagnostic]) {
        let Some(facts) = &mut self.facts else {
            return;
        };
        for error in errors {
            if let Some(at) = facts
                .errors
                .iter()
                .position(|e| e.span == error.span && e.message == error.message)
            {
                facts.errors.remove(at);
            }
        }
    }

    #[inline]
    fn forget_errors(&mut self, count: usize) {
        if let Some(facts) = &mut self.facts {
            facts.errors.truncate(count);
        }
    }

    #[inline]
    fn error(&mut self, error: &crate::diagnostics::Diagnostic) {
        if let Some(facts) = &mut self.facts {
            facts.errors.push(error.clone());
        }
    }

    #[inline]
    fn refuse_untrusted(&mut self, span: Span) -> bool {
        match self.facts.as_mut().filter(|facts| facts.untrusted) {
            Some(facts) => {
                facts.refused.push(span);
                true
            }
            None => false,
        }
    }

    #[inline]
    fn refuse_unavailable(&mut self, span: Span) -> bool {
        match self.facts.as_mut() {
            Some(facts) => {
                facts.refused.push(span);
                true
            }
            None => false,
        }
    }

    fn kept_source(
        &self,
        path: &Path,
        bytes: &[u8],
    ) -> Option<(Rc<str>, crate::source::LineIndex)> {
        let (reuse, compile) = self.parse_reuse.as_ref()?;
        reuse.borrow_mut().source(*compile, path, bytes)
    }

    fn keep_source(
        &self,
        path: &Path,
        bytes: &[u8],
        text: &Rc<str>,
        lines: &crate::source::LineIndex,
    ) {
        if let Some((reuse, compile)) = &self.parse_reuse {
            reuse
                .borrow_mut()
                .keep_source(*compile, path, bytes, text, lines);
        }
    }

    fn parse(&mut self, path: &Path, file: FileId, text: &Rc<str>) -> Option<Rc<SharedParse>> {
        let facts = self.facts.as_mut()?;
        let shared = if let Some((reuse, compile)) = &self.parse_reuse {
            reuse.borrow_mut().parse(*compile, path, file, text)
        } else {
            let parsed = crate::parser::parse_file_ide(file, text);
            Rc::new(SharedParse {
                file: parsed.file,
                diagnostics: parsed.diagnostics,
            })
        };
        facts
            .parse_errors
            .extend(shared.diagnostics.iter().cloned());
        facts.errors.extend(shared.diagnostics.iter().cloned());
        Some(shared)
    }

    fn errors<'a>(&mut self, errors: impl IntoIterator<Item = &'a Diagnostic>) {
        if let Some(facts) = &mut self.facts {
            facts.errors.extend(errors.into_iter().cloned());
        }
    }

    #[inline]
    fn sketching(&self) -> bool {
        self.facts.as_ref().is_some_and(|facts| facts.sketching)
    }

    #[inline]
    fn recovers_exprs(&self) -> bool {
        self.is_live()
    }

    #[inline]
    fn continues_past_errors(&self) -> bool {
        self.is_live() && !self.sketching()
    }

    fn stmt_failed(&self, code: Code, is_decl: bool) -> StmtRecovery {
        if !self.continues_past_errors() {
            StmtRecovery::Propagate
        } else if code.is_unknown_result() {
            StmtRecovery::Salvage
        } else if is_decl {
            StmtRecovery::Declare
        } else {
            StmtRecovery::Continue
        }
    }

    fn keep_going(&mut self, error: Box<Diagnostic>) -> Result<()> {
        match &mut self.facts {
            Some(facts) => {
                facts.errors.push(*error);
                Ok(())
            }
            None => Err(error),
        }
    }

    fn keep_going_once(&mut self, error: Box<Diagnostic>) -> Result<()> {
        match &mut self.facts {
            Some(facts) => {
                if !facts
                    .errors
                    .iter()
                    .any(|known| known.span == error.span && known.message == error.message)
                {
                    facts.errors.push(*error);
                }
                Ok(())
            }
            None => Err(error),
        }
    }

    #[inline]
    fn checks_every_body(&self) -> bool {
        self.is_live()
    }

    #[inline]
    fn allows_parse_ahead(&self) -> bool {
        !self.is_live()
    }

    fn is_selected_entry(&self, path: &Path) -> bool {
        self.selected_entry.as_deref() == Some(path)
    }

    #[inline]
    fn trusts_reachability(&self) -> bool {
        self.is_live()
    }

    #[inline]
    fn counts_scope_changes(&self) -> bool {
        self.is_live()
    }

    fn signature_only(&mut self, inst: crate::sema::ProcId) {
        if let Some(facts) = &mut self.facts {
            facts.sketch_signatures.insert(inst);
        }
    }

    fn is_signature_only(&self, proc: crate::sema::ProcId) -> bool {
        self.facts
            .as_ref()
            .is_some_and(|facts| facts.sketch_signatures.contains(&proc))
    }

    fn value(&self, n: &Narrowing, expr: &ast::Expr) -> Option<Interval> {
        self.facts()?.eval(n, expr)
    }

    fn compare(&self, n: &Narrowing, op: BinOp, a: &ast::Expr, b: &ast::Expr) -> Option<bool> {
        self.facts()?.compare(n, op, a, b)
    }

    fn narrow(&self, n: &mut Narrowing, cond: &ast::Expr, holds: bool, region: Span) {
        if let Some(ide) = self.facts() {
            ide.narrow(n, cond, holds, region);
        }
    }

    fn narrow_case(&self, n: &mut Narrowing, value: &ast::Expr, case: &ast::Case, body: Span) {
        if let Some(ide) = self.facts() {
            ide.narrow_case(n, value, case, body);
        }
    }

    fn body_walked(&mut self, body: Span, mut dead: Vec<Span>) {
        let Some(ide) = self.facts_mut() else {
            return;
        };
        dead.sort_by_key(|s| (s.start, s.end));
        dead.dedup();
        *ide.body_evals.entry(body).or_default() += 1;
        for run in dead {
            *ide.dead_runs.entry((body, run)).or_default() += 1;
        }
    }
}
