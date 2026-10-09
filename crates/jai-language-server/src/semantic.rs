//! Type-checked answers for hover and completion: the open documents are compiled with jaic
//! (bundled or on-disk stdlib) and the compiler's editor facts (`jaic::sema::ide`) are queried.
//! Compilation happens on demand and is cached per source text.
use jaic::build::{BuildEnv, Workspaces};
use jaic::intern::Sym;
use jaic::interp::{SandboxHost, SharedHost};
use jaic::sema::ide::{IdeFacts, IdeLayout, IdeName};
use jaic::sema::{Compiler, FileSystem, Options};
use jaic::source::{FileId, ImportSuggestion};
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

/// Compilers kept for reuse: the newest one that checked (what completion falls back to while
/// the text does not parse) and the newest of all. A compile is hundreds of MiB for a large
/// program, so room is made before the next one starts: at most `CACHED` are alive at once,
/// the one being built included.
const CACHED: usize = 2;

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
    /// The program's root file, the module folders searched and the text of every open file it
    /// was compiled from (shared with the compiler's file system, so no copy).
    root: PathBuf,
    dirs: Vec<PathBuf>,
    files: BTreeMap<PathBuf, Rc<[u8]>>,
    /// The compile got far enough to record scopes and names (the text parsed), so its facts
    /// can answer for an edit that left the code before the cursor alone.
    usable: bool,
    /// The program compiled without errors.
    complete: bool,
    /// Why it did not: the compiler's first error, then the independent errors the editor
    /// check of the other declarations and bodies found.
    errors: Vec<jaic::source::Diagnostic>,
    /// Lints found so far, by file.
    lints: BTreeMap<FileId, Vec<jailint::Lint>>,
    /// The imports that would declare the error's unknown name (worked out on first use: it
    /// reads the standard library).
    imports: Option<Vec<ImportSuggestion>>,
}

impl Analysis {
    /// jailint's findings in `path` (computed once per compile).
    pub fn lints(
        &mut self,
        path: &Path,
        config: &jailint::config::Config,
    ) -> Option<Vec<jailint::Lint>> {
        let file = self.file(path)?;
        if let Some(found) = self.lints.get(&file) {
            return Some(found.clone());
        }
        let found = jailint::lint_files(&mut self.compiler, &[file], config, self.complete);
        self.lints.insert(file, found.clone());
        Some(found)
    }

    /// The compile errors as (start, end, message) in `path`: each at its own span when that
    /// is in the file, else at the first of its notes that is (a module procedure the file
    /// called).
    pub fn check_errors(&self, path: &Path) -> Vec<(usize, usize, String)> {
        self.located(path, &self.errors)
    }

    /// The compiler's warnings (a print format with the wrong argument count, a body that can
    /// fall off its end), placed like the errors.
    pub fn check_warnings(&self, path: &Path) -> Vec<(usize, usize, String)> {
        self.located(path, &self.compiler.warnings)
    }

    fn located(
        &self,
        path: &Path,
        diagnostics: &[jaic::source::Diagnostic],
    ) -> Vec<(usize, usize, String)> {
        let Some(file) = self.file(path) else {
            return Vec::new();
        };
        let mut out: Vec<(usize, usize, String)> = Vec::new();
        for error in diagnostics {
            let found = if error.span.file == file {
                let span = error.span;
                Some((
                    span.start as usize,
                    span.end as usize,
                    error.message.clone(),
                ))
            } else {
                error
                    .notes
                    .iter()
                    .find(|(span, _)| span.file == file)
                    .map(|(span, note)| {
                        (
                            span.start as usize,
                            span.end as usize,
                            format!("{}\n{note}", error.message),
                        )
                    })
            };
            if let Some(found) = found
                && !out.iter().any(|o| o.0 == found.0 && o.1 == found.1)
            {
                out.push(found);
            }
        }
        out
    }

    /// The `#import`s that would declare the unknown name the compile error in `path` reports,
    /// best first, with the error's span as (start, end).
    pub fn missing_imports(
        &mut self,
        path: &Path,
    ) -> Option<(usize, usize, Vec<ImportSuggestion>)> {
        let file = self.file(path)?;
        let error = self.errors.first()?;
        if error.span.file != file {
            return None;
        }
        let span = error.span;
        let imports = self.imports.get_or_insert_with(|| {
            self.compiler
                .with_name_suggestion(error)
                .and_then(|d| d.fixes)
                .map(|fixes| fixes.imports)
                .unwrap_or_default()
        });
        Some((span.start as usize, span.end as usize, imports.clone()))
    }

    pub fn file(&self, path: &Path) -> Option<FileId> {
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

    /// A span of any compiled file as (path, text, start, end); `None` for a span in no file
    /// (`Span::NONE`, which builtins carry).
    pub fn location(&self, span: jaic::source::Span) -> Option<(String, Rc<str>, usize, usize)> {
        let source = self.compiler.sources.try_get(span.file)?;
        Some((
            source.path.clone(),
            source.text.clone(),
            span.start as usize,
            span.end as usize,
        ))
    }

    /// Hover at `offset` as (start, end, description, memory layout).
    pub fn hover(
        &mut self,
        path: &Path,
        offset: usize,
    ) -> Option<(usize, usize, String, Option<IdeLayout>)> {
        let file = self.file(path)?;
        let hover = self.compiler.ide_hover(file, offset as u32)?;
        Some((
            hover.span.start as usize,
            hover.span.end as usize,
            hover.text,
            hover.layout,
        ))
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
            .filter_map(|span| self.location(span))
            .collect()
    }

    /// The members of the enum an inferred `.NAME` at `offset` is expected to be.
    pub fn complete_inferred(&mut self, path: &Path, offset: usize) -> Option<Vec<IdeName>> {
        let file = self.file(path)?;
        Some(self.compiler.ide_inferred_members(file, offset as u32))
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
    /// Programs compiled so far.
    compiles: usize,
}

fn hash(root: &Path, dirs: &[PathBuf], files: &BTreeMap<PathBuf, Rc<[u8]>>) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    root.hash(&mut h);
    dirs.hash(&mut h);
    files.hash(&mut h);
    h.finish()
}

impl Cache {
    /// Programs compiled so far, and compiles alive now.
    pub fn counts(&self) -> (usize, usize) {
        (self.compiles, self.entries.len())
    }

    /// Drop the lints found so far (their settings changed); the compiles stay.
    pub fn forget_lints(&mut self) {
        for entry in &mut self.entries {
            entry.lints.clear();
        }
    }

    /// Drop compiles until one fewer than `CACHED` is left: the newest that checked stays, else
    /// the newest.
    fn make_room(&mut self) {
        while self.entries.len() >= CACHED {
            let keep = self
                .entries
                .iter()
                .rposition(|a| a.usable)
                .unwrap_or(self.entries.len() - 1);
            let victim = (0..self.entries.len()).find(|&i| i != keep).unwrap_or(0);
            self.entries.remove(victim);
        }
    }

    /// A compile of `root` that already knows the code before `prefix.len()` of `path`: the
    /// other open files are as they were compiled and `path` starts with `prefix` (what is
    /// typed after that point is not in its facts, and does not need to be). Completion uses it
    /// instead of compiling again for every keystroke; the names declared by edits since that
    /// compile show up once the next one has run.
    pub fn compiled_before(
        &mut self,
        env: &Environment,
        root: &Path,
        dirs: &[PathBuf],
        files: BTreeMap<PathBuf, Rc<[u8]>>,
        path: &Path,
        prefix: &[u8],
    ) -> Option<&mut Analysis> {
        let canonical: BTreeMap<PathBuf, Rc<[u8]>> = files
            .into_iter()
            .map(|(p, t)| (env.fs.canonical(&p), t))
            .collect();
        let path = env.fs.canonical(path);
        let at = self.entries.iter().rposition(|a| {
            a.usable
                && a.root == root
                && a.dirs == dirs
                && a.files.len() == canonical.len()
                && a.files.iter().all(|(p, bytes)| match canonical.get(p) {
                    Some(new) if *p == path => bytes.starts_with(prefix) && new.starts_with(prefix),
                    Some(new) => bytes == new,
                    None => false,
                })
        })?;
        Some(&mut self.entries[at])
    }

    /// The analysis of `root` with `files` open (compiled now unless cached).
    pub fn analyze(
        &mut self,
        env: &Environment,
        root: &Path,
        dirs: &[PathBuf],
        files: BTreeMap<PathBuf, Rc<[u8]>>,
    ) -> &mut Analysis {
        let files: BTreeMap<PathBuf, Rc<[u8]>> = files
            .into_iter()
            .map(|(p, t)| (env.fs.canonical(&p), t))
            .collect();
        let key = hash(root, dirs, &files);
        if let Some(at) = self.entries.iter().position(|a| a.key == key) {
            let entry = self.entries.remove(at);
            self.entries.push(entry);
        } else {
            // A compile of the same program from text that parses will check, and everything the
            // older one can answer (completion after an unchanged prefix) it answers too, so the
            // older one goes now instead of staying alive through the whole new compile.
            if files.values().all(|bytes| {
                std::str::from_utf8(bytes).is_ok_and(|text| parse_error(text).is_none())
            }) {
                self.entries.retain(|a| !(a.root == root && a.dirs == dirs));
            }
            self.make_room();
            self.compiles += 1;
            let snapshot = files.clone();
            let (compiler, errors) = compile(env, root, dirs, files);
            let usable = errors.is_empty()
                || compiler
                    .ide
                    .as_ref()
                    .is_some_and(|ide| !ide.scopes.is_empty());
            self.entries.push(Analysis {
                compiler,
                key,
                root: root.to_path_buf(),
                dirs: dirs.to_vec(),
                files: snapshot,
                usable,
                complete: errors.is_empty(),
                errors,
                lints: BTreeMap::new(),
                imports: None,
            });
        }
        self.entries.last_mut().expect("just pushed")
    }
}

/// The compiled program, and its errors: the first the compile stopped at, then the independent
/// ones among the declarations and bodies it did not get to.
fn compile(
    env: &Environment,
    root: &Path,
    dirs: &[PathBuf],
    files: BTreeMap<PathBuf, Rc<[u8]>>,
) -> (Compiler, Vec<jaic::source::Diagnostic>) {
    let fs: Rc<dyn FileSystem> = Rc::new(OverlayFs {
        base: env.fs.clone(),
        files,
    });
    let dir = root.parent().map(Path::to_path_buf).unwrap_or_default();
    let options = || {
        let mut options = (env.options)(root);
        crate::modules::splice(&mut options.import_paths, dirs);
        options
    };
    let mut compiler = Compiler::new(options(), fs.clone());
    // Compile-time output goes nowhere: stdout may be the protocol channel.
    let host = Rc::new(RefCell::new(SandboxHost::with_files(
        fs,
        &dir.to_string_lossy(),
    )));
    compiler.interp.host = Box::new(SharedHost(host.clone()));
    compiler.interp.block_budget = Some(BLOCK_BUDGET);
    // Metaprograms may compile other programs (`compiler_create_workspace`): those are checked
    // the same way, with no output and the same host, and their compile-time code draws from
    // this compiler's budget.
    let workspace_host = host.clone();
    let workspaces = Workspaces::new(BuildEnv {
        unwritten_output_hint: None,
        fs: compiler.fs.clone(),
        options: options(),
        backend: None,
        command_line: Vec::new(),
        make_host: Box::new(move |_| Box::new(SharedHost(workspace_host.clone()))),
        report: Box::new(|_| {}),
        observer: None,
    });
    compiler.attach_workspaces(workspaces);
    let mut prefix = env.fs.canonical(&dir).to_string_lossy().into_owned();
    if !prefix.ends_with('/') {
        prefix.push('/');
    }
    let mut facts = IdeFacts::new(vec![prefix]);
    facts.output = Some(host);
    // What jailint needs (expression types, casts, uses) on top of the editor facts.
    facts.lint = true;
    compiler.ide = Some(Box::new(facts));
    let mut errors: Vec<jaic::source::Diagnostic> = compiler
        .compile_program(root)
        .err()
        .into_iter()
        .map(|e| *e)
        .collect();
    for e in compiler.ide_check_all() {
        if !errors
            .iter()
            .any(|o| o.span == e.span && o.message == e.message)
        {
            errors.push(e);
        }
    }
    (compiler, errors)
}

thread_local! {
    /// Where the texts parsed lately fail (by hash and length): hover, references, diagnostics
    /// and lints each ask about the same open document, and a parse of a large file is a
    /// transient tree of tens of MiB each time.
    static PARSES: std::cell::RefCell<Vec<(u64, usize, Option<usize>)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Parse results remembered (the repair of one broken document tries about three texts).
const PARSES_KEPT: usize = 8;

fn text_key(text: &str) -> (u64, usize) {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut h);
    (h.finish(), text.len())
}

/// Remember that `text` parses (`None`) or fails at a byte offset, for `parse_error`.
pub fn note_parse(text: &str, error: Option<usize>) {
    let (hash, len) = text_key(text);
    PARSES.with(|p| {
        let mut p = p.borrow_mut();
        p.retain(|e| (e.0, e.1) != (hash, len));
        if p.len() >= PARSES_KEPT {
            p.remove(0);
        }
        p.push((hash, len, error));
    });
}

/// Where `text` fails to lex or parse (a byte offset), if it does.
pub fn parse_error(text: &str) -> Option<usize> {
    let (hash, len) = text_key(text);
    if let Some(found) = PARSES.with(|p| {
        p.borrow()
            .iter()
            .find(|e| (e.0, e.1) == (hash, len))
            .map(|e| e.2)
    }) {
        return found;
    }
    let error = jaic::parser::parse_file(FileId(0), text)
        .err()
        .map(|d| d.span.start as usize);
    note_parse(text, error);
    error
}

#[cfg(test)]
mod tests {
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
