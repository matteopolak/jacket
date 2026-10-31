//! The hooks checking calls to tell an editor analysis what it saw, and the policy questions it
//! asks (where an editor checks differently from a build). `Compiler` holds an `Editor`, which
//! implements `Recorder`; a batch compile has an idle one, so every event returns at once and
//! costs one branch. Checking code outside `ide` goes through this trait only, so a worker that
//! checks a body can be given another implementation (one that buffers events for the driver to
//! replay in order) without the checker changing.
use super::editor::StmtRecovery;
use super::interval::Interval;
use super::values::Narrowing;
use crate::ast::{self, BinOp};
use crate::diagnostics::{Code, Diagnostic};
use crate::intern::Sym;
use crate::parser::SharedParse;
use crate::sema::{EntityId, ProcId, Result};
use crate::source::{FileId, Span};
use crate::types::TypeId;
use std::path::Path;
use std::rc::Rc;

pub(crate) trait Recorder {
    /// An expression was checked: a failed one is not checked again by the recovery of an
    /// enclosing expression until it succeeds.
    fn expr_checked(&mut self, span: Span, ok: bool);

    /// A sketch began checking the expression at `span`.
    fn sketch_tried(&mut self, span: Span);

    /// Whether a sketch began checking the expression at `span`.
    fn sketch_checked(&self, span: Span) -> bool;

    /// Whether a sketch gave the expression at `span` a type.
    fn sketch_typed(&self, span: Span) -> bool;

    /// The name at `span` resolved to `ids` (one entity, or overloads): a use of each, for a
    /// linter.
    fn named(&mut self, span: Span, ids: &[EntityId]);

    /// The name at `span` resolved to `id`, outside an identifier (a struct constant, a named
    /// expression).
    fn resolved(&mut self, span: Span, id: EntityId);

    /// The name just checked stands for `member` of a `using`d value.
    fn named_using_member(&mut self, member: Sym);

    /// The `#run` at `span`, of type `ty`, failed.
    fn run_failed(&mut self, span: Span, ty: TypeId);

    /// The type of the failed `#run` at `span`.
    fn failed_run_type(&self, span: Span) -> Option<TypeId>;

    /// `.name` at `span` was inferred against `ty`.
    fn inferred(&mut self, span: Span, ty: TypeId);

    /// The macro `proc` was expanded.
    fn macro_expanded(&mut self, proc: ProcId);

    /// The body of `proc` expanded a macro that returns from its caller.
    fn macro_returned(&mut self, proc: ProcId);

    /// Errors recorded so far, to drop the ones a speculative check adds (`forget_errors`).
    fn error_count(&self) -> usize;

    /// The errors recorded since `count` of them were.
    fn errors_since(&self, count: usize) -> Vec<Diagnostic>;

    /// Drop one recorded copy of each of `errors`.
    fn forget_matching(&mut self, errors: &[Diagnostic]);

    fn forget_errors(&mut self, count: usize);

    /// An error checking went on past, so the editor still shows it.
    fn error(&mut self, error: &crate::diagnostics::Diagnostic);

    /// Compile-time code at `span` was refused because the workspace is untrusted: `true` when
    /// it was (and the refusal is recorded).
    fn refuse_untrusted(&mut self, span: Span) -> bool;

    /// Compile-time code at `span` called something the editor's sandbox does not provide (a
    /// foreign procedure it does not emulate): what it would have produced is unknown, not
    /// wrong, so it is recorded like a refusal. `true` when recording.
    fn refuse_unavailable(&mut self, span: Span) -> bool;

    /// The text and line index kept from an earlier compile that read `bytes` from `path`.
    fn kept_source(&self, path: &Path, bytes: &[u8])
    -> Option<(Rc<str>, crate::source::LineIndex)>;

    /// Keep what `bytes` from `path` decoded to, for later compiles.
    fn keep_source(
        &self,
        path: &Path,
        bytes: &[u8],
        text: &Rc<str>,
        lines: &crate::source::LineIndex,
    );

    /// Parses `text` (file `file`, read from `path`) the editor's way, with every syntax
    /// error recovered from and recorded: `None` when not recording, and the batch parser
    /// applies.
    fn parse(&mut self, path: &Path, file: FileId, text: &Rc<str>) -> Option<Rc<SharedParse>>;

    /// Errors reported along with the first: a batch compile stops at the first, an editor
    /// shows them all.
    fn errors<'a>(&mut self, errors: impl IntoIterator<Item = &'a Diagnostic>);

    // --- Policy: where an editor analysis checks differently from a build. A batch compile
    // (not recording) always gets the build's answer.
    /// A sketch (a body checked with stand-ins for what failed) is being checked.
    fn sketching(&self) -> bool;

    /// An expression that failed has its operands checked for their facts, and gets a
    /// stand-in value when one of them is a stand-in (expressions over a failed
    /// declaration's values are unknown, not wrong). Always, not only once something left
    /// a stand-in behind: what an edit elsewhere declares must not change these facts.
    fn recovers_exprs(&self) -> bool;

    /// Checking goes on past a failed statement of a block, for the facts of the rest.
    fn continues_past_errors(&self) -> bool;

    fn stmt_failed(&self, code: Code, is_decl: bool) -> StmtRecovery;

    /// A declaration or import of a file failed: an editor records the error and declares the
    /// rest of the file, so the names after it keep their types and definitions.
    fn keep_going(&mut self, error: Box<Diagnostic>) -> Result<()>;

    /// `keep_going` for an error that may be reported again (an import retried from each
    /// file that names it): recorded once.
    fn keep_going_once(&mut self, error: Box<Diagnostic>) -> Result<()>;

    /// Bodies no build reaches are checked too (an entry point in a program without `main`).
    fn checks_every_body(&self) -> bool;

    /// Files may be parsed ahead on worker threads. An editor parses on its own thread, with
    /// error recovery and shared parses (`Editor::parse`).
    fn allows_parse_ahead(&self) -> bool;

    /// `path` is the entry file the editor picked (canonical), so it may be inspected even
    /// where loading it would otherwise be refused.
    fn is_selected_entry(&self, path: &Path) -> bool;

    /// The end of a body that walking it shows is never reached owes no result. A build keeps
    /// the reference compiler's simpler rule.
    fn trusts_reachability(&self) -> bool;

    /// Failed bodies are retried only after scopes changed: a change to what a scope holds is
    /// progress for `Compiler::lower_epoch`.
    fn counts_scope_changes(&self) -> bool;

    /// A sketch made only the signature of polymorph instance `inst` (`ide_sketch_refuses`).
    fn signature_only(&mut self, inst: crate::sema::ProcId);

    /// `proc` has only a signature: its body is never lowered.
    fn is_signature_only(&self, proc: crate::sema::ProcId) -> bool;

    /// What `expr` can be where the walk is: its checked interval, tightened by the narrowing
    /// `n` of the branches around it.
    fn value(&self, n: &Narrowing, expr: &ast::Expr) -> Option<Interval>;

    /// Whether `a op b` always (or never) holds where the walk is.
    fn compare(&self, n: &Narrowing, op: BinOp, a: &ast::Expr, b: &ast::Expr) -> Option<bool>;

    /// Enter `region`, which runs when `cond` is `holds`: narrow the locals it constrains.
    fn narrow(&self, n: &mut Narrowing, cond: &ast::Expr, holds: bool, region: Span);

    /// Enter the `body` of `case`, where the switched `value` is one of the case's values.
    fn narrow_case(&self, n: &mut Narrowing, value: &ast::Expr, case: &ast::Case, body: Span);

    /// A walk of the body at `body` found the runs `dead` that cannot run. A polymorphic
    /// procedure is walked once per instance, so a run counts as dead only if every walk found
    /// it dead (`body_evals` against `dead_runs`).
    fn body_walked(&mut self, body: Span, dead: Vec<Span>);
}
