//! Type-checked answers for hover and completion: the open documents are compiled with jaic
//! (bundled or on-disk stdlib) and the compiler's editor facts (`jaic::ide`) are queried.
//! Compilation happens on demand and is cached per source text.
mod cache;
mod checked_inputs;
pub(crate) use checked_inputs::CheckedInputs;
mod compile;
pub(crate) use self::compile::find_file;
mod hints;
mod notices;
mod observations;
pub(crate) use observations::{Observations, ObservedFs};
mod overlay;
mod parses;
mod probes;
pub(crate) use probes::ProbeStatus;
#[cfg(test)]
mod hint_tests;
mod queries;
mod shift;

pub use self::cache::Cache;
pub(crate) use self::cache::{QueryAnalysis, QuerySelection};
pub use self::overlay::Reads;
pub(crate) use self::parses::text_key;
pub use self::parses::{note_parse, parse_error};
pub use self::queries::EvalCost;
pub(crate) use self::queries::{CheckFinding, LimitKind};
pub use self::shift::Shift;
use jaic::ide::IdeLayout;
use jaic::sema::{Compiler, FileSystem, Options};
use jaic::source::FileId;
use jaic::target::TargetOs;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

/// Compiler options selected using the supplied observed filesystem and program root.
pub type OptionsCallback = dyn Fn(&dyn FileSystem, &Path) -> Options;

/// Where the language server finds modules: a file system for everything that is not an open
/// document, and compiler options for a main file (import paths, Preload, target).
pub struct Environment {
    pub fs: Rc<dyn FileSystem>,
    /// Options for a root, using only the supplied filesystem for dynamic discovery.
    /// Native callers capture those actual responses before cache reuse. Other callback
    /// state must remain stable until the session receives its settings notification/reset.
    pub options: Box<OptionsCallback>,
    /// Compile-time services for trusted workspaces (`services::EditorServices`); `None`
    /// answers every call as unavailable.
    pub services: Option<Rc<dyn jaic::service::ServiceProvider>>,
}

/// Observed semantic analysis coverage, without source text or diagnostic payloads.
#[derive(Clone, Debug, serde::Serialize)]
pub struct SemanticCoverage {
    pub context_id: String,
    /// Passive provenance of the selected analysis/program, not configured CLI intent.
    pub analysis: SemanticAnalysisContext,
    pub source_loaded: bool,
    pub recorded_references: usize,
    pub recorded_fields: usize,
    pub program_complete: bool,
    pub program_errors: usize,
    pub target: String,
}

/// An observed native path. Audit serialization and Debug expose only its opaque identity.
/// Trusted non-redacted consumers may explicitly inspect path; no filesystem lookup occurs.
#[derive(Clone, serde::Serialize)]
pub struct SemanticPath {
    #[serde(skip)]
    pub path: PathBuf,
    pub identity: String,
}

impl SemanticPath {
    fn observed(path: PathBuf) -> Self {
        use std::hash::{Hash, Hasher};
        let mut hash = jaic::fxhash::FxHasher::default();
        "semantic-path".hash(&mut hash);
        path.hash(&mut hash);
        Self {
            path,
            identity: format!("path:{:016x}", hash.finish()),
        }
    }
}

impl std::fmt::Debug for SemanticPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SemanticPath")
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}

/// Actual cached-analysis provenance and the selected compiler's options.
/// Unknown observations stay None. Workspace IDs are scoped by `coverage.context_id`.
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct SemanticAnalysisContext {
    /// The root argument used to create this cached analysis, including after focus switches.
    pub root: Option<SemanticPath>,
    /// Directly observed `compile_program` entry argument; unknown for generated workspaces.
    pub entry: Option<SemanticPath>,
    /// Exact selected compiler `options.import_paths`, preserving order and native spelling.
    pub import_dirs: Option<Vec<SemanticPath>>,
    /// Actual selected compiler.meta.workspace, not a scanner-assigned label.
    pub workspace_id: Option<i64>,
}

/// A declaration in a compiled file: (path, text of the file, start, end of its name).
pub type Decl = (String, Arc<str>, usize, usize);

/// What the type checker says about a hover.
pub struct HoverFacts {
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub layout: Option<IdeLayout>,
    pub decls: Vec<Decl>,
}

/// Compilers kept for reuse: the newest one that checked (what completion falls back to while
/// the text does not parse) and the newest of all. A compile is hundreds of MiB for a large
/// program, so room is made before the next one starts: at most `CACHED` are alive at once,
/// the one being built included.
const CACHED: usize = 2;

/// Analyses whose source and captured proof storage fit this allowance are small. Up to
/// `SMALL_KEPT` stay besides `CACHED`, so opening another small file need not discard the
/// program being typed in. Rebuild protection ranks source work separately from storage.
const SMALL_COST: usize = 1 << 20;

/// Small compiles kept besides `CACHED`.
const SMALL_KEPT: usize = 2;

/// Compiles for another target kept besides those (for code an `#if OS == ...` skipped), the
/// one being built included.
const ALTERNATES: usize = 3;

/// The targets asked, in order, about code the environment's target skips.
pub const OTHER_TARGETS: [TargetOs; 4] = [
    TargetOs::Linux,
    TargetOs::Windows,
    TargetOs::MacOS,
    TargetOs::Wasm,
];

/// What one run of a build metaprogram did: per workspace it created, the files it added with
/// `add_build_file` (canonical keys), how many strings with `add_build_string`, and the module
/// folders its build options add to the import path. Recorded while the metaprogram runs, so
/// computed paths (`tprint`, variables, loops) are known exactly.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildFacts {
    pub programs: Vec<BuiltProgram>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuiltProgram {
    pub files: Vec<PathBuf>,
    pub strings: usize,
    pub import_dirs: Vec<PathBuf>,
    /// This produced workspace completed its independent IDE check without errors.
    pub checked: bool,
}

/// The open documents' texts by path, shared so that asking again with the same inputs is
/// known by address alone.
pub type Buffers = Rc<BTreeMap<PathBuf, Arc<[u8]>>>;

type ReferenceCache = Option<Rc<jaic::ide::meta::ReferenceIndex>>;

pub struct Analysis {
    pub(crate) authority: Rc<()>,
    /// Analysis-wide accepted mutable-fact view, shared by every contained Program.
    pub(crate) facts_accepted: Rc<std::cell::Cell<bool>>,
    pub compiler: Compiler,
    /// A cancelled semantic query may have changed this compiler; no later lookup may reuse it.
    query_valid: bool,
    /// Entry observation belongs to the selected compiler and swaps with it.
    entry: Option<PathBuf>,
    key: u64,
    /// `quick_key` of the inputs last answered from this compile, with those inputs, which
    /// keeps their buffers alive so no other text can take their addresses.
    quick: Option<(u64, Buffers)>,
    /// The target it was compiled for, when not the environment's (see `other_targets`).
    os: Option<TargetOs>,
    /// The program's root file, the module folders searched and the text of every open file it
    /// was compiled from (shared with the compiler's file system, so no copy).
    pub(crate) root: PathBuf,
    pub(crate) dirs: Vec<PathBuf>,
    pub(crate) files: BTreeMap<PathBuf, Arc<[u8]>>,
    /// The paths the compile and the queries since asked about: a buffer outside them can
    /// change without the compile going stale.
    pub(crate) reads: Rc<RefCell<Reads>>,
    pub(crate) build_facts: Option<BuildFacts>,
    pub(crate) build_complete: bool,
    /// Advisory analyses never authorize exact navigation or accepted build facts.
    advisory: bool,
    /// Live filesystem inputs move only after exact/unaffected cache validation.
    overlay: Rc<overlay::OverlayFs>,
    /// The compile got far enough to record scopes and names (the text parsed), so its facts
    /// can answer for an edit that left the code before the cursor alone.
    usable: bool,
    /// Bytes of source the compile read (its programs' included): what dropping it would cost
    /// to redo.
    cost: usize,
    /// The program compiled without errors.
    complete: bool,
    /// Why it did not: the compiler's first error, then the independent errors the editor
    /// check of the other declarations and bodies found.
    errors: Vec<jaic::diagnostics::Diagnostic>,
    /// Lints found so far, by file.
    lints: BTreeMap<FileId, Vec<jailint::Lint>>,
    /// Immutable reference resolution for this exact selected compiler state.
    references: ReferenceCache,
    /// Opaque identity survives immutable snapshots without retaining this compiler.
    reference_owner: Rc<()>,
    /// Hint observations belong only to the current query, across Program focus changes.
    hints: Rc<hints::HintCapture>,
    /// `file`'s answers by the path asked for: canonicalizing every compiled file's path for
    /// each request was a quarter of a hover.
    found: RefCell<BTreeMap<PathBuf, Option<FileId>>>,
    /// The programs the metaprogram built (`compiler_create_workspace`, as a `first.jai` does):
    /// the files they compiled are answered from them (see `focus`).
    others: Vec<Program>,
}

/// A program one compile produced, with what is known of it so far. The answering one lives in
/// `Analysis`'s own fields; the others wait in `Analysis::others`.
struct Program {
    compiler: Compiler,
    entry: Option<PathBuf>,
    complete: bool,
    errors: Vec<jaic::diagnostics::Diagnostic>,
    lints: BTreeMap<FileId, Vec<jailint::Lint>>,
    /// Immutable reference resolution for this exact selected compiler state.
    references: ReferenceCache,
    /// Opaque identity survives immutable snapshots without retaining this compiler.
    reference_owner: Rc<()>,
}


#[cfg(test)]
mod tests {
    use super::parses::PARSES_KEPT;
    use super::*;

    #[test]
    fn parse_results_are_remembered_by_text() {
        let good = "main :: () { x := 1; }\n";
        let bad = "main :: () { x := ; }\n";
        assert_eq!(parse_error(good), None);
        let at = parse_error(bad);
        assert!(at.is_some());
        // Both are answered from memory now, and more texts than are kept push the oldest out.
        assert_eq!(parse_error(good), None);
        assert_eq!(parse_error(bad), at);
        for n in 0..=PARSES_KEPT {
            assert_eq!(parse_error(&format!("f{n} :: () {{}}\n")), None);
        }
        assert_eq!(parse_error(bad), at);
    }
}
