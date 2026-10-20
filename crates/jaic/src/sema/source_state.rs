//! Source files of one compile: the file system, loaded and provided texts, path canonicalisation and parse-ahead workers.
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
    pub(in crate::sema) canon_cache: CanonCache,
    /// Worker threads lexing and parsing the files this compile will load (`parse_ahead`).
    pub(in crate::sema) ahead: Option<ParseAhead>,
    pub sources: SourceMap,
    pub files: Vec<FileInfo>,
    /// Lexical provenance of scoped/generated sources, for relative source loads
    /// and the distribution boundary around library-owned internals.
    pub(in crate::sema) origins: HashMap<FileId, FileId>,
    pub(in crate::sema) file_by_path: HashMap<(PathBuf, ModuleId), modules::FileLoadState>,
    /// Whether each source file is the stdlib's, for trap reports (a `#run` can trap often).
    pub(in crate::sema) library_files: RefCell<HashMap<u32, bool>>,
    /// Sources added so far (labels of added strings).
    pub(crate) added: usize,
    /// Body `#import`s already added to their file scope (by statement span).
    pub(in crate::sema) hoisted_imports: HashSet<Span>,
    /// Modules made from `provide_import` texts, by text.
    pub provided_texts: HashMap<String, ModuleId>,
    /// How deeply each `#insert`ed string file is nested in other inserted strings (absent: a
    /// real file). Bounds code that inserts itself (`X :: "#insert X;"`).
    pub(in crate::sema) insert_depth: HashMap<FileId, u32>,
}

impl SourceState {
    pub fn new(fs: Rc<dyn FileSystem>) -> Self {
        Self {
            fs,
            canon_cache: CanonCache::new(),
            ahead: None,
            sources: SourceMap::default(),
            files: Vec::new(),
            origins: HashMap::default(),
            file_by_path: HashMap::default(),
            library_files: RefCell::default(),
            added: 0,
            hoisted_imports: HashSet::default(),
            provided_texts: HashMap::default(),
            insert_depth: HashMap::default(),
        }
    }
}
