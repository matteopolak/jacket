//! Type-checked answers for hover and completion: the open documents are compiled with jaic
//! (bundled or on-disk stdlib) and the compiler's editor facts (`jaic::sema::ide`) are queried.
//! Compilation happens on demand and is cached per source text.
use jaic::intern::Sym;
use jaic::interp::{SandboxHost, SharedHost};
use jaic::sema::ide::{IdeFacts, IdeName};
use jaic::sema::{Compiler, FileSystem, Options};
use jaic::source::FileId;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Where the language server finds modules: a file system for everything that is not an open
/// document, and compiler options for a main file (import paths, Preload, target).
pub struct Environment {
    pub fs: Rc<dyn FileSystem>,
    pub options: Box<dyn Fn(&Path) -> Options>,
}

/// Basic blocks compile-time code may run per analysis (an edit can make a `#run` loop forever).
const BLOCK_BUDGET: u64 = 20_000_000;
/// Compilers kept for reuse: the text as typed and the completion probe.
const CACHED: usize = 3;

/// Open documents over the environment's file system.
struct OverlayFs {
    base: Rc<dyn FileSystem>,
    files: BTreeMap<PathBuf, Rc<[u8]>>,
}
impl FileSystem for OverlayFs {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        match self.files.get(&self.base.canonical(path)) {
            Some(bytes) => Some(bytes.to_vec()),
            None => self.base.read(path),
        }
    }
    fn is_file(&self, path: &Path) -> bool {
        self.files.contains_key(&self.base.canonical(path)) || self.base.is_file(path)
    }
    fn is_dir(&self, path: &Path) -> bool {
        let p = self.base.canonical(path);
        self.files.keys().any(|k| k.starts_with(&p) && *k != p) || self.base.is_dir(path)
    }
    fn canonical(&self, path: &Path) -> PathBuf {
        self.base.canonical(path)
    }
    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        self.base.list_dir(path)
    }
}

pub struct Analysis {
    pub compiler: Compiler,
    key: u64,
}

impl Analysis {
    fn file(&self, path: &Path) -> Option<FileId> {
        let want = self.compiler.fs.canonical(path);
        (0..self.compiler.sources.len() as u32)
            .map(FileId)
            .find(|&f| {
                self.compiler
                    .fs
                    .canonical(Path::new(&self.compiler.sources.get(f).path))
                    == want
            })
    }
    pub fn hover(&mut self, path: &Path, offset: usize) -> Option<(usize, usize, String)> {
        let file = self.file(path)?;
        let (span, text) = self.compiler.ide_hover(file, offset as u32)?;
        Some((span.start as usize, span.end as usize, text))
    }
    /// Declarations the identifier at `offset` names, as (path, text, start, end) with byte
    /// offsets into that file's text (which may be a module or stdlib file, not an open document).
    pub fn definition(
        &mut self,
        path: &Path,
        offset: usize,
    ) -> Vec<(String, Rc<str>, usize, usize)> {
        let Some(file) = self.file(path) else {
            return Vec::new();
        };
        self.compiler
            .ide_definition(file, offset as u32)
            .into_iter()
            .map(|span| {
                let source = self.compiler.sources.get(span.file);
                (
                    source.path.clone(),
                    source.text.clone(),
                    span.start as usize,
                    span.end as usize,
                )
            })
            .collect()
    }
    /// Completion candidates at `offset`, members of `chain` when it is not empty.
    pub fn complete(&mut self, path: &Path, offset: usize, chain: &[&str]) -> Option<Vec<IdeName>> {
        let file = self.file(path)?;
        let scope = self.compiler.ide_scope_at(file, offset as u32)?;
        if chain.is_empty() {
            return Some(self.compiler.ide_visible(scope, file, offset as u32));
        }
        let names: Vec<Sym> = chain.iter().map(|n| Sym::intern(n)).collect();
        let receiver = self.compiler.ide_receiver(scope, &names)?;
        Some(self.compiler.ide_members(receiver))
    }
}

#[derive(Default)]
pub struct Cache {
    entries: Vec<Analysis>,
}

fn hash(root: &Path, files: &BTreeMap<PathBuf, Rc<[u8]>>) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    root.hash(&mut h);
    files.hash(&mut h);
    h.finish()
}

impl Cache {
    /// The analysis of `root` with `files` open (compiled now unless cached).
    pub fn analyze(
        &mut self,
        env: &Environment,
        root: &Path,
        files: BTreeMap<PathBuf, Rc<[u8]>>,
    ) -> &mut Analysis {
        let files: BTreeMap<PathBuf, Rc<[u8]>> = files
            .into_iter()
            .map(|(p, t)| (env.fs.canonical(&p), t))
            .collect();
        let key = hash(root, &files);
        if let Some(at) = self.entries.iter().position(|a| a.key == key) {
            let entry = self.entries.remove(at);
            self.entries.push(entry);
        } else {
            if self.entries.len() >= CACHED {
                self.entries.remove(0);
            }
            self.entries.push(Analysis {
                compiler: compile(env, root, files),
                key,
            });
        }
        self.entries.last_mut().expect("just pushed")
    }
}

fn compile(env: &Environment, root: &Path, files: BTreeMap<PathBuf, Rc<[u8]>>) -> Compiler {
    let fs: Rc<dyn FileSystem> = Rc::new(OverlayFs {
        base: env.fs.clone(),
        files,
    });
    let dir = root.parent().map(Path::to_path_buf).unwrap_or_default();
    let mut compiler = Compiler::new((env.options)(root), fs.clone());
    // Compile-time output goes nowhere: stdout may be the protocol channel.
    let host = Rc::new(RefCell::new(SandboxHost::with_files(
        fs,
        &dir.to_string_lossy(),
    )));
    compiler.interp.host = Box::new(SharedHost(host));
    compiler.interp.block_budget = Some(BLOCK_BUDGET);
    let mut prefix = env.fs.canonical(&dir).to_string_lossy().into_owned();
    if !prefix.ends_with('/') {
        prefix.push('/');
    }
    compiler.ide = Some(Box::new(IdeFacts::new(vec![prefix])));
    let _ = compiler.compile_program(root);
    compiler.ide_check_all();
    compiler
}

/// Where `text` fails to lex or parse (a byte offset), if it does.
pub fn parse_error(text: &str) -> Option<usize> {
    jaic::parser::parse_file(FileId(0), text)
        .err()
        .map(|d| d.span.start as usize)
}
