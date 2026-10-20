//! The compiler's handle on an editor analysis. A batch compile has an idle `Editor`: every
//! event method returns at once, so recording costs the checker one branch per site.
use super::IdeFacts;
use crate::parser::{ParseReuse, SharedParse};
use crate::source::{FileId, Span};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Default)]
pub struct Editor {
    /// What the analysis has recorded so far; `None` outside the language server and linter.
    facts: Option<Box<IdeFacts>>,
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

    /// Errors recorded so far, to drop the ones a speculative check adds (`forget_errors`).
    #[inline]
    pub(crate) fn error_count(&self) -> usize {
        self.facts.as_ref().map_or(0, |facts| facts.errors.len())
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
}
