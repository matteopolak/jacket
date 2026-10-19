use crate::analysis::{Span, Token};
use crate::semantic::{self, Environment};
use crate::{DocumentUri, Limits, analysis::Analysis, position::LineIndex};
use jaic::ide::{IdeLayout, IdeName};
use std::borrow::Cow;
use std::cell::{OnceCell, RefCell};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;

pub(crate) struct Document {
    pub(crate) version: i32,
    pub(crate) text: String,
    pub(crate) index: LineIndex,
    /// Whether `text` has an `#asm` block anywhere, worked out on first use.
    pub(crate) asm: OnceCell<bool>,
    /// The links in `text`'s doc comments, found on first use.
    pub(crate) sites: OnceCell<Vec<jaic::docs::Site>>,
    pub(crate) lexical_damage: Vec<Span>,
}

impl Document {
    pub(crate) fn ide_text(&self) -> Cow<'_, str> {
        crate::lexical_recovery::mask(&self.text, &self.lexical_damage)
            .map_or_else(|| Cow::Borrowed(self.text.as_str()), Cow::Owned)
    }

    pub(crate) fn same_positions(&self, text: &str) -> bool {
        self.text == text || (!self.lexical_damage.is_empty() && self.ide_text().as_ref() == text)
    }

    pub(crate) fn has_asm(&self) -> bool {
        *self.asm.get_or_init(|| self.text.contains("#asm"))
    }

    pub(crate) fn sites(&self) -> &[jaic::docs::Site] {
        self.sites.get_or_init(|| jaic::docs::sites(&self.text))
    }
}

/// The tokens whose span holds `byte` (touching it at either end), in order, found by binary
/// search: `tokens` are in source order.
pub(crate) fn tokens_at(tokens: &[Token], byte: usize) -> impl Iterator<Item = (usize, &Token)> {
    let first = tokens.partition_point(|t| t.span.end < byte);
    tokens[first..]
        .iter()
        .enumerate()
        .map(move |(i, t)| (first + i, t))
        .take_while(move |(_, t)| t.span.start <= byte)
        .filter(move |(_, t)| byte <= t.span.end)
}

/// Byte ranges each other target skipped, by file and target.
type Unreached = BTreeMap<(PathBuf, u8), Vec<(usize, usize)>>;

pub struct Session {
    pub(crate) limits: Limits,
    pub(crate) documents: BTreeMap<DocumentUri, Document>,
    pub(crate) analyses: BTreeMap<DocumentUri, Rc<Analysis>>,
    /// Last analysis of each document that parsed: completion keeps offering its names while
    /// the text being typed does not parse.
    pub(crate) parsed: BTreeMap<DocumentUri, Rc<Analysis>>,
    /// Type-checked answers (absent: syntax only).
    pub(crate) environment: Option<Environment>,
    pub(crate) semantic: RefCell<semantic::Cache>,
    /// `Cache::recorded_version` as of the last time discovery was given the recorded builds.
    synced_builds: std::cell::Cell<u64>,
    /// Open `jailint.toml` files: settings for the lints of documents under their directory.
    /// They are not Jai documents, so they get no analysis or diagnostics of their own.
    pub(crate) lint_configs: BTreeMap<DocumentUri, Document>,
    /// Auto-import completion is on (`jai.completion.autoImport`).
    pub(crate) auto_import: bool,
    /// What accepting a procedure completion inserts (`jai.completion.callSnippets`).
    call_snippets: crate::call_snippets::CallSnippets,
    /// The client takes snippet completion items (`completionItem.snippetSupport`).
    snippet_support: bool,
    /// Declarations at least this many lines long get a hint at their closing brace.
    pub(crate) closing_brace_min_lines: usize,
    /// The client's workspace folders: projects without a `jai.toml` are inferred in them.
    pub(crate) workspace_folders: Vec<PathBuf>,
    /// What modules and project files declare, for auto-import completion.
    pub(crate) index: RefCell<crate::auto_import::Index>,
    /// Bumped by anything that can change a compile's inputs: an open document, a file on
    /// disk, the workspace folders.
    generation: std::cell::Cell<u64>,
    /// The last `compile_inputs` answer, with the generation, document and text it was for:
    /// working them out reads folders and copies every open document, which a burst of
    /// requests on one version would otherwise repeat.
    /// The text is `None` when it was the document's own.
    inputs: RefCell<Option<InputsMemo>>,
    /// Filesystem-proven identities of open documents, invalidated with every input change.
    open_file_identities: RefCell<Option<Vec<(PathBuf, DocumentUri)>>>,
    /// The code each other target's compile skipped too (by file and target, byte ranges), for
    /// the generation it was found in: not compiled for again. Code no target checks (a file
    /// nothing loads, a branch on yet another CPU) would otherwise compile every target on
    /// every hover, more of them than the cache keeps.
    unreached: RefCell<(u64, Unreached)>,
}

/// The open files, the root file and the module folders of a compile.
type CompileInputs = (semantic::Buffers, PathBuf, Vec<PathBuf>);

/// `compile_inputs`' last answer: generation, document, its text (`None` for the document's
/// own) and the inputs.
type InputsMemo = (u64, DocumentUri, Option<String>, CompileInputs);

/// What completion asks the type checker: the word being typed, the names it could be, and the
/// path of each file they are declared in (by file number).
type Completions = (String, Vec<IdeName>, BTreeMap<u32, String>);

impl Session {
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            documents: BTreeMap::new(),
            analyses: BTreeMap::new(),
            parsed: BTreeMap::new(),
            environment: None,
            semantic: RefCell::default(),
            synced_builds: std::cell::Cell::new(0),
            lint_configs: BTreeMap::new(),
            auto_import: true,
            call_snippets: crate::call_snippets::CallSnippets::default(),
            snippet_support: false,
            closing_brace_min_lines: crate::closing_hints::DEFAULT_MIN_LINES,
            workspace_folders: Vec::new(),
            index: RefCell::default(),
            generation: std::cell::Cell::new(0),
            inputs: RefCell::default(),
            open_file_identities: RefCell::default(),
            unreached: RefCell::default(),
        }
    }
}

pub(crate) fn contains(span: Span, byte: usize) -> bool {
    span.start <= byte && byte <= span.end
}

/// A hover's memory layout line: `size 24, align 8 (4 bytes of padding)` for a type,
/// `offset 8, size 4, align 4 (4 bytes of padding before)` for a field.
fn layout_line(layout: IdeLayout) -> String {
    let bytes = |n: u64| {
        if n == 1 {
            "1 byte".to_owned()
        } else {
            format!("{n} bytes")
        }
    };
    let mut line = match layout.offset {
        Some(offset) => format!(
            "offset {offset}, size {}, align {}",
            layout.size, layout.align
        ),
        None => format!("size {}, align {}", layout.size, layout.align),
    };
    if layout.padding > 0 {
        let before = if layout.offset.is_some() {
            " before"
        } else {
            ""
        };
        line.push_str(&format!(" ({} of padding{before})", bytes(layout.padding)));
    }
    line
}

mod compile;
mod completion;
mod config;
mod documents;
mod navigation;
mod symbols;
mod tokens;
#[cfg(test)]
#[path = "uri_alias_tests.rs"]
mod uri_alias_tests;

/// Where the top-level declaration around `byte` starts: the last line before it that begins in
/// column 0 with something other than a closing brace or a comment.
fn enclosing_declaration_start(text: &str, byte: usize) -> usize {
    let mut start = 0;
    let mut at = 0;
    for line in text[..byte].split_inclusive('\n') {
        let top = line
            .chars()
            .next()
            .is_some_and(|c| !c.is_whitespace() && c != '}')
            && !line.starts_with("//");
        if top {
            start = at;
        }
        at += line.len();
    }
    start
}
