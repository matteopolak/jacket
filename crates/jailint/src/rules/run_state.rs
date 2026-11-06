//! State shared by rules and files during one lint invocation.
//!
//! The caller owns this state for exactly one compiler and root selection. Nothing here
//! can be reused after checking, source, configuration, or filesystem changes.
use jaic::fxhash::HashMap;
use jaic::lexer::Token;
use jaic::sema::Compiler;
use jaic::source::FileId;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

/// What the rules remember across the files of one lint run. A run makes its own, so nothing
/// a rule learned about the file system outlives it (a file edited between runs is read anew).
#[derive(Default)]
pub(crate) struct RunState {
    pub(super) resource_reports:
        RefCell<HashMap<FileId, Rc<super::lifetime_cfg::resources::Reports>>>,
    pub(super) call_bindings: std::cell::OnceCell<super::call_bindings::Bindings>,
    pub(super) lock_primitives: std::cell::OnceCell<bool>,
    pub(super) scalar_openings: std::cell::OnceCell<super::scalar_openings::Openings>,
    pub(super) lifetimes: std::cell::OnceCell<super::lifetime_summary::Summaries>,
    pub(crate) lifetime_entry_module: Option<jaic::sema::ModuleId>,
    pub(crate) lifetime_files: jaic::fxhash::HashSet<FileId>,
    /// The definitions of the `.jai` files around a directory, by directory.
    pub(super) variants: RefCell<HashMap<PathBuf, Rc<super::unused_parameter::Variants>>>,
    /// Resolved paths.
    pub(super) canon: jaic::paths::CanonCache,
    /// The tokens of the other files of the program, lexed when a rule asks.
    lexed: RefCell<HashMap<FileId, Option<Rc<[Token]>>>>,
}

impl RunState {
    pub(crate) fn with_lifetime_entry(module: Option<jaic::sema::ModuleId>) -> Self {
        Self {
            lifetime_entry_module: module,
            ..Self::default()
        }
    }

    /// The tokens of `file`, which must lex.
    pub(crate) fn tokens(&self, compiler: &Compiler, file: FileId) -> Option<Rc<[Token]>> {
        self.lexed
            .borrow_mut()
            .entry(file)
            .or_insert_with(|| {
                let text = &compiler.src.sources.try_get(file)?.text;
                jaic::lexer::lex(file, text).ok().map(Rc::from)
            })
            .clone()
    }
}
