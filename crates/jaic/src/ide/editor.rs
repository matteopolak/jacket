//! The compiler's handle on an editor analysis. A batch compile has an idle `Editor`: every
//! event method returns at once, so recording costs the checker one branch per site.
use super::IdeFacts;
use crate::intern::Sym;
use crate::parser::{ParseReuse, SharedParse};
use crate::sema::{EntityId, ProcId, Result};
use crate::source::{Diagnostic, DiagnosticKind, FileId, Span};
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
    }

    /// Stops recording and hands back what was recorded.
    pub fn stop(&mut self) -> Option<Box<IdeFacts>> {
        self.facts.take()
    }

    /// Records into `facts` again, after `stop`.
    pub(crate) fn resume(&mut self, facts: Box<IdeFacts>) {
        self.facts = Some(facts);
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

    /// An expression was checked: a failed one is not checked again by the recovery of an
    /// enclosing expression until it succeeds.
    #[inline]
    pub(crate) fn expr_checked(&mut self, span: Span, ok: bool) {
        if let Some(facts) = &mut self.facts {
            if ok {
                facts.failed_exprs.remove(&span);
            } else {
                facts.failed_exprs.insert(span);
            }
        }
    }

    /// A sketch began checking the expression at `span`.
    #[inline]
    pub(crate) fn sketch_tried(&mut self, span: Span) {
        if let Some(facts) = self.facts.as_mut().filter(|facts| facts.sketching) {
            facts.sketch_tried.insert(span);
        }
    }

    /// Whether a sketch began checking the expression at `span`.
    pub(crate) fn sketch_checked(&self, span: Span) -> bool {
        self.facts()
            .is_some_and(|facts| facts.sketch_tried.contains(&span))
    }

    /// Whether a sketch gave the expression at `span` a type.
    pub(crate) fn sketch_typed(&self, span: Span) -> bool {
        self.facts()
            .is_some_and(|facts| facts.sketch_types.contains_key(&span))
    }

    /// The name at `span` resolved to `ids` (one entity, or overloads): a use of each, for a
    /// linter.
    #[inline]
    pub(crate) fn named(&mut self, span: Span, ids: &[EntityId]) {
        if let Some(facts) = &mut self.facts {
            facts.last_entity = Some((span, ids[0]));
            if facts.lint {
                facts.used.extend(ids.iter().copied());
            }
        }
    }

    /// The name at `span` resolved to `id`, outside an identifier (a struct constant, a named
    /// expression).
    #[inline]
    pub(crate) fn resolved(&mut self, span: Span, id: EntityId) {
        if let Some(facts) = &mut self.facts {
            facts.last_entity = Some((span, id));
        }
    }

    /// The name just checked stands for `member` of a `using`d value.
    #[inline]
    pub(crate) fn named_using_member(&mut self, member: Sym) {
        if let Some(facts) = &mut self.facts {
            facts.last_using_member = Some(member);
        }
    }

    /// The `#run` at `span`, of type `ty`, failed.
    pub(crate) fn run_failed(&mut self, span: Span, ty: TypeId) {
        if let Some(facts) = &mut self.facts {
            facts.failed_runs.insert(span, ty);
        }
    }

    /// The type of the failed `#run` at `span`.
    pub(crate) fn failed_run_type(&self, span: Span) -> Option<TypeId> {
        self.facts()?.failed_runs.get(&span).copied()
    }

    /// `.name` at `span` was inferred against `ty`.
    pub(crate) fn inferred(&mut self, span: Span, ty: TypeId) {
        if let Some(facts) = &mut self.facts {
            let seen = facts.inferred_expected.entry(span).or_default();
            if !seen.contains(&ty) {
                seen.push(ty);
            }
        }
    }

    /// The macro `proc` was expanded.
    #[inline]
    pub(crate) fn macro_expanded(&mut self, proc: ProcId) {
        if let Some(facts) = &mut self.facts {
            facts.expanded_macros.insert(proc);
        }
    }

    /// The body of `proc` expanded a macro that returns from its caller.
    pub(crate) fn macro_returned(&mut self, proc: ProcId) {
        if let Some(facts) = &mut self.facts {
            facts.macro_returners.insert(proc);
        }
    }

    /// Errors recorded so far, to drop the ones a speculative check adds (`forget_errors`).
    #[inline]
    pub(crate) fn error_count(&self) -> usize {
        self.facts.as_ref().map_or(0, |facts| facts.errors.len())
    }

    /// The errors recorded since `count` of them were.
    pub(crate) fn errors_since(&self, count: usize) -> Vec<Diagnostic> {
        self.facts.as_ref().map_or_else(Vec::new, |facts| {
            facts.errors.get(count..).unwrap_or(&[]).to_vec()
        })
    }

    /// Drop one recorded copy of each of `errors`.
    pub(crate) fn forget_matching(&mut self, errors: &[Diagnostic]) {
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
    pub(crate) fn forget_errors(&mut self, count: usize) {
        if let Some(facts) = &mut self.facts {
            facts.errors.truncate(count);
        }
    }

    /// An error checking went on past, so the editor still shows it.
    #[inline]
    pub(crate) fn error(&mut self, error: &crate::source::Diagnostic) {
        if let Some(facts) = &mut self.facts {
            facts.errors.push(error.clone());
        }
    }

    /// Compile-time code at `span` was refused because the workspace is untrusted: `true` when
    /// it was (and the refusal is recorded).
    #[inline]
    pub(crate) fn refuse_untrusted(&mut self, span: Span) -> bool {
        match self.facts.as_mut().filter(|facts| facts.untrusted) {
            Some(facts) => {
                facts.refused.push(span);
                true
            }
            None => false,
        }
    }

    /// Compile-time code at `span` called something the editor's sandbox does not provide (a
    /// foreign procedure it does not emulate): what it would have produced is unknown, not
    /// wrong, so it is recorded like a refusal. `true` when recording.
    #[inline]
    pub(crate) fn refuse_unavailable(&mut self, span: Span) -> bool {
        match self.facts.as_mut() {
            Some(facts) => {
                facts.refused.push(span);
                true
            }
            None => false,
        }
    }

    /// The text and line index kept from an earlier compile that read `bytes` from `path`.
    pub(crate) fn kept_source(
        &self,
        path: &Path,
        bytes: &[u8],
    ) -> Option<(Rc<str>, crate::source::LineIndex)> {
        let (reuse, compile) = self.parse_reuse.as_ref()?;
        reuse.borrow_mut().source(*compile, path, bytes)
    }

    /// Keep what `bytes` from `path` decoded to, for later compiles.
    pub(crate) fn keep_source(
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

    /// Parses `text` (file `file`, read from `path`) the editor's way, with every syntax
    /// error recovered from and recorded: `None` when not recording, and the batch parser
    /// applies.
    pub(crate) fn parse(
        &mut self,
        path: &Path,
        file: FileId,
        text: &Rc<str>,
    ) -> Option<Rc<SharedParse>> {
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

    /// Errors reported along with the first: a batch compile stops at the first, an editor
    /// shows them all.
    pub(crate) fn errors<'a>(&mut self, errors: impl IntoIterator<Item = &'a Diagnostic>) {
        if let Some(facts) = &mut self.facts {
            facts.errors.extend(errors.into_iter().cloned());
        }
    }

    // --- Policy: where an editor analysis checks differently from a build. A batch compile
    // (not recording) always gets the build's answer.

    /// A sketch (a body checked with stand-ins for what failed) is being checked.
    #[inline]
    pub(crate) fn sketching(&self) -> bool {
        self.facts.as_ref().is_some_and(|facts| facts.sketching)
    }

    /// An expression that failed has its operands checked for their facts, and gets a
    /// stand-in value when one of them is a stand-in (expressions over a failed
    /// declaration's values are unknown, not wrong). Always, not only once something left
    /// a stand-in behind: what an edit elsewhere declares must not change these facts.
    #[inline]
    pub(crate) fn recovers_exprs(&self) -> bool {
        self.is_live()
    }

    /// Checking goes on past a failed statement of a block, for the facts of the rest.
    #[inline]
    pub(crate) fn continues_past_errors(&self) -> bool {
        self.is_live() && !self.sketching()
    }

    pub(crate) fn stmt_failed(&self, kind: DiagnosticKind, is_decl: bool) -> StmtRecovery {
        if !self.continues_past_errors() {
            StmtRecovery::Propagate
        } else if kind.is_unknown_result() {
            StmtRecovery::Salvage
        } else if is_decl {
            StmtRecovery::Declare
        } else {
            StmtRecovery::Continue
        }
    }

    /// A declaration or import of a file failed: an editor records the error and declares the
    /// rest of the file, so the names after it keep their types and definitions.
    pub(crate) fn keep_going(&mut self, error: Box<Diagnostic>) -> Result<()> {
        match &mut self.facts {
            Some(facts) => {
                facts.errors.push(*error);
                Ok(())
            }
            None => Err(error),
        }
    }

    /// `keep_going` for an error that may be reported again (an import retried from each
    /// file that names it): recorded once.
    pub(crate) fn keep_going_once(&mut self, error: Box<Diagnostic>) -> Result<()> {
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

    /// Bodies no build reaches are checked too (an entry point in a program without `main`).
    #[inline]
    pub(crate) fn checks_every_body(&self) -> bool {
        self.is_live()
    }

    /// Files may be parsed ahead on worker threads. An editor parses on its own thread, with
    /// error recovery and shared parses (`Editor::parse`).
    #[inline]
    pub(crate) fn allows_parse_ahead(&self) -> bool {
        !self.is_live()
    }

    /// `path` is the entry file the editor picked (canonical), so it may be inspected even
    /// where loading it would otherwise be refused.
    pub(crate) fn is_selected_entry(&self, path: &Path) -> bool {
        self.selected_entry.as_deref() == Some(path)
    }

    /// The end of a body that walking it shows is never reached owes no result. A build keeps
    /// the reference compiler's simpler rule.
    #[inline]
    pub(crate) fn trusts_reachability(&self) -> bool {
        self.is_live()
    }

    /// Failed bodies are retried only after scopes changed: a change to what a scope holds is
    /// progress for `Compiler::lower_epoch`.
    #[inline]
    pub(crate) fn counts_scope_changes(&self) -> bool {
        self.is_live()
    }

    /// A sketch made only the signature of polymorph instance `inst` (`ide_sketch_refuses`).
    pub(crate) fn signature_only(&mut self, inst: crate::sema::ProcId) {
        if let Some(facts) = &mut self.facts {
            facts.sketch_signatures.insert(inst);
        }
    }

    /// `proc` has only a signature: its body is never lowered.
    pub(crate) fn is_signature_only(&self, proc: crate::sema::ProcId) -> bool {
        self.facts
            .as_ref()
            .is_some_and(|facts| facts.sketch_signatures.contains(&proc))
    }
}
