//! Source files of one compile: the file system, loaded and provided texts, path canonicalisation and parse-ahead workers.
use super::body_probe::{Probed, W};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::fxhash::{HashMap, HashSet};
use crate::parse_ahead::ParseAhead;
use crate::paths::CanonCache;
use crate::source::{FileId, SourceMap, Span};

use super::modules::{self, FileSystem};
use super::{FileInfo, ModuleId};

pub struct SourceState {
    pub fs: Rc<dyn FileSystem>,
    /// Real paths already worked out (see [`SourceState::canon`]).
    pub(in crate::sema) canon_cache: Probed<CanonCache, { W::SrcCanonCache as usize }>,
    /// Worker threads lexing and parsing the files this compile will load (`parse_ahead`).
    pub(in crate::sema) ahead: Probed<Option<ParseAhead>, { W::SrcAhead as usize }>,
    pub sources: Probed<SourceMap, { W::SrcSources as usize }>,
    pub files: Probed<Vec<FileInfo>, { W::SrcFiles as usize }>,
    /// Lexical provenance of scoped/generated sources, for relative source loads
    /// and the distribution boundary around library-owned internals.
    pub(crate) origins: Probed<HashMap<FileId, FileId>, { W::SrcOrigins as usize }>,
    pub(in crate::sema) file_by_path:
        Probed<HashMap<(PathBuf, ModuleId), modules::FileLoadState>, { W::SrcFileByPath as usize }>,
    /// Whether each source file is the stdlib's, for trap reports (a `#run` can trap often).
    pub(in crate::sema) library_files: RefCell<HashMap<u32, bool>>,
    /// Where the string literals of each source file are, as byte ranges. Worked out once per
    /// file and text (a file whose text was replaced is worked out again): a `#run` that reports often asks for every report (`SourceState::reported_span`).
    pub(in crate::sema) string_ranges: RefCell<HashMap<(u32, usize, usize), Rc<[(usize, usize)]>>>,
    /// The file a metaprogram's report named, by the path it gave (`SourceState::file_by_path`).
    /// Only found files are kept: the first file with a path never changes, a missing one can
    /// still be loaded.
    pub(in crate::sema) reported_files: RefCell<HashMap<String, FileId>>,
    /// Sources added so far (labels of added strings).
    pub(crate) added: usize,
    /// Body `#import`s already added to their file scope (by statement span).
    pub(in crate::sema) hoisted_imports: Probed<HashSet<Span>, { W::SrcHoistedImports as usize }>,
    /// Modules made from `provide_import` texts, by text.
    pub provided_texts: Probed<HashMap<String, ModuleId>, { W::SrcProvidedTexts as usize }>,
    /// How deeply each `#insert`ed string file is nested in other inserted strings (absent: a
    /// real file). Bounds code that inserts itself (`X :: "#insert X;"`).
    pub(in crate::sema) insert_depth: Probed<HashMap<FileId, u32>, { W::SrcInsertDepth as usize }>,
    /// Where each `#insert`ed string file was inserted: the editor shows a declaration in
    /// generated text at the `#insert` that wrote it.
    pub insert_sites: Probed<HashMap<FileId, Span>, { W::SrcInsertSites as usize }>,
    /// The files and strings the program was given, kept in a cross build so the host twin loads
    /// the same program (`sema::host_twin`). Empty otherwise.
    pub(in crate::sema) program_sources: Vec<super::ProgramSource>,
    /// In a host twin: sources its compile-time code added, with the scope a `Code` value named,
    /// waiting for the target compiler to load them too (`sema::host_twin`).
    pub(in crate::sema) twin_outbox:
        Probed<Vec<(super::ProgramSource, Option<super::ScopeId>)>, { W::SrcTwinOutbox as usize }>,
    /// Import sites and failed asserts, for tools that attribute a failure to the file that asked
    /// for the code (`sema::import_trace`).
    pub(in crate::sema) import_trace: super::ImportTrace,
}

impl SourceState {
    pub fn new(fs: Rc<dyn FileSystem>) -> Self {
        Self {
            fs,
            canon_cache: CanonCache::new().into(),
            ahead: None.into(),
            sources: Probed::default(),
            files: Probed::default(),
            origins: Probed::default(),
            file_by_path: Probed::default(),
            library_files: RefCell::default(),
            string_ranges: RefCell::default(),
            reported_files: RefCell::default(),
            added: 0,
            hoisted_imports: Probed::default(),
            provided_texts: Probed::default(),
            insert_depth: Probed::default(),
            insert_sites: Probed::default(),
            program_sources: Vec::default(),
            twin_outbox: Probed::default(),
            import_trace: super::ImportTrace::default(),
        }
    }
}
