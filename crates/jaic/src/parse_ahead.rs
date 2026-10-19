//! Lexes and parses the files a compile will load on worker threads, ahead of sema.
//!
//! Sema stays single-threaded and asks `ParseAhead::claim` for each file it loads. Workers crawl
//! the import graph from the files the compile starts with: each parses its file, reads the
//! `#load`s and `#import`s the tree declares, resolves them as sema does, and queues any file not
//! yet seen. A claim returns the finished parse (waiting for a worker that is mid-file), or
//! `None` when nothing was predicted or the file has not been started: sema then parses it
//! itself, which is also the path for every failure, so diagnostics come from one place.
//!
//! A worker cannot know the file id, the `AstId` numbers or the interner state the compile will
//! have when sema reaches its file. It parses with placeholders and a per-job interner, and
//! `Prepared::finish` (on the main thread, at claim time) rewrites them (see `ast::relocate`),
//! giving the tree a serial parse at that point would have built. The crawl is only a guess at
//! sema's order: a wrong guess costs a wasted parse, never a different result.
//!
//! `-serial_parse` (`[experimental] serial_parse`) turns the crawl off and `-parse_threads N` sets the
//! worker count (`crate::knobs`).
pub use imp::{ParseAhead, Prepared, PreparedTree};

#[cfg(target_arch = "wasm32")]
mod imp {
    use crate::ast;
    use crate::sema::Options;
    use crate::source::{FileId, LineIndex};
    use std::path::{Path, PathBuf};
    use std::rc::Rc;

    pub struct ParseAhead;
    pub struct Prepared;
    pub struct PreparedTree;

    impl ParseAhead {
        pub fn start(_: &Options, _: Vec<PathBuf>) -> Option<ParseAhead> {
            None
        }

        pub fn claim(&self, _: &Path) -> Option<Prepared> {
            None
        }

        pub fn discover(&self, _: &ast::File, _: &Path) {
        }
    }

    impl Prepared {
        pub fn into_parts(self) -> (Rc<str>, LineIndex, PreparedTree) {
            unreachable!()
        }
    }

    impl PreparedTree {
        pub fn finish(self, _: FileId) -> ast::File {
            unreachable!()
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use crate::ast::{self, AstId, StmtKind, relocate};
    use crate::fxhash::HashMap;
    use crate::intern;
    use crate::sema::{
        BootstrapModule, FileSystem, NativeFs, Options, TargetCpu, TargetOs, find_module_in,
    };
    use crate::source::{FileId, LineIndex};
    use std::collections::VecDeque;
    use std::path::{Path, PathBuf};
    use std::rc::Rc;
    use std::sync::{Arc, Condvar, Mutex};

    /// A file's finished parse, still numbered for the worker that made it.
    pub struct Prepared {
        text: Handoff<Rc<str>>,
        lines: LineIndex,
        tree: PreparedTree,
    }

    /// The tree of a `Prepared`, before it is placed in a compile.
    pub struct PreparedTree {
        file: Handoff<ast::File>,
        spellings: Vec<&'static str>,
        ast_count: u32,
    }

    /// A value moved between threads. Its `Rc`s are only ever shared inside the value, and the
    /// worker drops every other handle before the value is published.
    struct Handoff<T>(T);

    // SAFETY: see `Handoff`. Nothing else points into the value, so the reference counts it
    // holds are touched by one thread at a time.
    #[allow(unsafe_code)]
    unsafe impl<T> Send for Handoff<T> {
    }

    impl Prepared {
        /// The file's text, the offsets of its lines, and its tree.
        pub fn into_parts(self) -> (Rc<str>, LineIndex, PreparedTree) {
            (self.text.0, self.lines, self.tree)
        }
    }

    impl PreparedTree {
        /// The tree for a compile that gave the file `file`.
        pub fn finish(self, file: FileId) -> ast::File {
            let syms = intern::Remap::new(&self.spellings);
            let base = AstId::reserve(self.ast_count);
            let mut tree = self.file.0;
            relocate::relocate(&mut tree, file, base, &syms);
            tree
        }
    }

    enum Entry {
        Queued,
        Running,
        Done(Option<Box<Prepared>>),
        /// Claimed, or taken by sema before a worker started it.
        Taken,
    }

    struct State {
        queue: VecDeque<PathBuf>,
        entries: HashMap<PathBuf, Entry>,
        shutdown: bool,
    }

    struct Shared {
        state: Mutex<State>,
        /// Workers wait here for queued files.
        work: Condvar,
        /// `claim` waits here for a running file.
        done: Condvar,
        import_paths: Vec<PathBuf>,
        preload: Option<PathBuf>,
        /// The canonical standard library root.
        stdlib: Option<PathBuf>,
        os: TargetOs,
        cpu: TargetCpu,
    }

    pub struct ParseAhead {
        shared: Arc<Shared>,
    }

    impl Drop for ParseAhead {
        fn drop(&mut self) {
            self.shared.state.lock().unwrap().shutdown = true;
            self.shared.work.notify_all();
        }
    }

    /// Worker stacks: the parser recurses to `MAX_NESTING`.
    const WORKER_STACK: usize = 256 << 20;

    impl ParseAhead {
        /// Starts the workers on `roots` (the files the compile loads first), or `None` when the
        /// compile should parse serially.
        pub fn start(options: &Options, roots: Vec<PathBuf>) -> Option<ParseAhead> {
            let knobs = crate::knobs::get();
            if knobs.serial_parse {
                return None;
            }
            let workers = knobs
                .parse_threads
                .unwrap_or_else(|| crate::pool::threads().saturating_sub(1).min(4));
            if workers == 0 {
                return None;
            }
            let shared = Arc::new(Shared {
                state: Mutex::new(State {
                    queue: VecDeque::new(),
                    entries: HashMap::default(),
                    shutdown: false,
                }),
                work: Condvar::new(),
                done: Condvar::new(),
                import_paths: options.import_paths.clone(),
                preload: options.preload.clone(),
                stdlib: options.stdlib_root().map(|root| NativeFs.canonical(root)),
                os: options.os,
                cpu: options.cpu,
            });
            let ahead = ParseAhead {
                shared,
            };
            let canonical: Vec<PathBuf> = roots.iter().map(|p| NativeFs.canonical(p)).collect();
            ahead.shared.enqueue(canonical);
            for _ in 0..workers {
                let shared = ahead.shared.clone();
                let spawned = std::thread::Builder::new()
                    .name("jaic-parse".into())
                    .stack_size(WORKER_STACK)
                    .spawn(move || shared.work_loop());
                if spawned.is_err() {
                    break;
                }
            }
            Some(ahead)
        }

        /// The parse of `path` (canonical), if the crawl reached it and it parsed cleanly. Waits
        /// for a worker that is parsing it; a file no worker has started is left to the caller.
        pub fn claim(&self, path: &Path) -> Option<Prepared> {
            let mut state = self.shared.state.lock().unwrap();
            loop {
                match state.entries.get_mut(path)? {
                    entry @ Entry::Queued => {
                        *entry = Entry::Taken;
                        state.queue.retain(|queued| queued != path);
                        return None;
                    }
                    Entry::Running => {
                        state = self.shared.done.wait(state).unwrap();
                    }
                    entry @ Entry::Done(_) => {
                        let Entry::Done(prepared) = std::mem::replace(entry, Entry::Taken) else {
                            unreachable!()
                        };
                        return prepared.map(|p| *p);
                    }
                    Entry::Taken => return None,
                }
            }
        }

        /// Queues what `file` (parsed by the caller, at `path`) loads and imports.
        pub fn discover(&self, file: &ast::File, path: &Path) {
            self.shared.discover(file, path);
        }
    }

    /// `Shared::parse` for a test, with a crawl that has nothing to find.
    #[cfg(test)]
    pub fn prepare_for_test(path: &Path) -> Option<Prepared> {
        let shared = Shared {
            state: Mutex::new(State {
                queue: VecDeque::new(),
                entries: HashMap::default(),
                shutdown: false,
            }),
            work: Condvar::new(),
            done: Condvar::new(),
            import_paths: Vec::new(),
            preload: None,
            stdlib: None,
            os: TargetOs::Linux,
            cpu: TargetCpu::X64,
        };
        shared.parse(path)
    }

    impl Shared {
        fn enqueue(&self, paths: Vec<PathBuf>) {
            let mut state = self.state.lock().unwrap();
            let mut added = 0;
            for path in paths {
                if !state.entries.contains_key(&path) {
                    state.entries.insert(path.clone(), Entry::Queued);
                    state.queue.push_back(path);
                    added += 1;
                }
            }
            drop(state);
            for _ in 0..added {
                self.work.notify_one();
            }
        }

        fn work_loop(&self) {
            loop {
                let path = {
                    let mut state = self.state.lock().unwrap();
                    loop {
                        if state.shutdown {
                            return;
                        }
                        if let Some(path) = state.queue.pop_front() {
                            state.entries.insert(path.clone(), Entry::Running);
                            break path;
                        }
                        state = self.work.wait(state).unwrap();
                    }
                };
                let prepared =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.parse(&path)))
                        .ok()
                        .flatten();
                let mut state = self.state.lock().unwrap();
                if let Some(entry) = state.entries.get_mut(&path) {
                    *entry = Entry::Done(prepared.map(Box::new));
                }
                drop(state);
                self.done.notify_all();
            }
        }

        fn parse(&self, path: &Path) -> Option<Prepared> {
            let bytes = std::fs::read(path).ok()?;
            let text = String::from_utf8(bytes)
                .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned());
            intern::begin_job();
            AstId::restart();
            let parsed = match self.library_platform(path) {
                Some(target) => {
                    crate::parser::parse_file_for_target(relocate::PLACEHOLDER_FILE, &text, target)
                }
                None => crate::parser::parse_file(relocate::PLACEHOLDER_FILE, &text),
            };
            // Discovery reads symbols, so it runs before the job interner is taken away.
            if let Ok(file) = &parsed {
                self.discover(file, path);
            }
            let spellings = intern::end_job();
            let ast_count = AstId::next();
            let file = parsed.ok()?;
            let lines = LineIndex::new(&text);
            Some(Prepared {
                text: Handoff(text.into()),
                lines,
                tree: PreparedTree {
                    file: Handoff(file),
                    spellings,
                    ast_count,
                },
            })
        }

        /// The platform to read `path` for when it is in the standard library, as
        /// `Compiler::library_platform` decides.
        fn library_platform(&self, path: &Path) -> Option<crate::parser::PlatformTarget> {
            if !path.starts_with(self.stdlib.as_ref()?) {
                return None;
            }
            Some(crate::parser::PlatformTarget {
                os: crate::target::os_member(self.os),
                cpu: crate::target::cpu_member(self.cpu),
            })
        }

        fn discover(&self, file: &ast::File, path: &Path) {
            let dir = path.parent().unwrap_or(Path::new(""));
            let mut found = Vec::new();
            self.collect(&file.stmts, dir, &mut found);
            if !found.is_empty() {
                self.enqueue(found);
            }
        }

        /// The files the top-level statements `stmts` load and import, as sema resolves them.
        /// A `#if` on the target OS follows only the branch that holds; any other condition
        /// follows both.
        fn collect(&self, stmts: &[ast::Stmt], dir: &Path, out: &mut Vec<PathBuf>) {
            for stmt in stmts {
                match &stmt.kind {
                    StmtKind::Load {
                        path, ..
                    } => out.push(NativeFs.canonical(&dir.join(&**path))),
                    StmtKind::Import(import) => {
                        let entry = match &import.source {
                            ast::ImportSource::Module(name)
                                if BootstrapModule::named(name)
                                    == Some(BootstrapModule::Preload)
                                    && self.preload.is_some() =>
                            {
                                self.preload.clone()
                            }
                            ast::ImportSource::Module(name) => {
                                find_module_in(&NativeFs, &self.import_paths, name, dir)
                            }
                            ast::ImportSource::File(path) => Some(dir.join(&**path)),
                            ast::ImportSource::Dir(path) => {
                                Some(dir.join(&**path).join("module.jai"))
                            }
                            ast::ImportSource::String(_) => None,
                        };
                        if let Some(entry) = entry {
                            out.push(NativeFs.canonical(&entry));
                        }
                    }
                    StmtKind::StaticIf(branch) => match self.target_test(&branch.cond) {
                        Some(true) => self.collect(&branch.then_branch, dir, out),
                        Some(false) => self.collect(&branch.else_branch, dir, out),
                        None => {
                            self.collect(&branch.then_branch, dir, out);
                            self.collect(&branch.else_branch, dir, out);
                        }
                    },
                    _ => {}
                }
            }
        }

        /// Whether `cond` holds for the target, when it only compares `OS` or `CPU` with a
        /// member (combined with `&&`, `||`, `!`).
        fn target_test(&self, cond: &ast::Expr) -> Option<bool> {
            match &cond.kind {
                ast::ExprKind::Unary(ast::UnOp::Not, inner) => self.target_test(inner).map(|b| !b),
                ast::ExprKind::Binary(ast::BinOp::Or, lhs, rhs) => {
                    match (self.target_test(lhs), self.target_test(rhs)) {
                        (Some(true), _) | (_, Some(true)) => Some(true),
                        (Some(false), Some(false)) => Some(false),
                        _ => None,
                    }
                }
                ast::ExprKind::Binary(ast::BinOp::And, lhs, rhs) => {
                    match (self.target_test(lhs), self.target_test(rhs)) {
                        (Some(false), _) | (_, Some(false)) => Some(false),
                        (Some(true), Some(true)) => Some(true),
                        _ => None,
                    }
                }
                ast::ExprKind::Binary(op @ (ast::BinOp::Eq | ast::BinOp::Ne), lhs, rhs) => {
                    let (ast::ExprKind::Ident(name), ast::ExprKind::InferredMember(member)) =
                        (&lhs.kind, &rhs.kind)
                    else {
                        return None;
                    };
                    let (current, known): (&str, &[&str]) = match name.as_str() {
                        "OS" => (
                            match self.os {
                                TargetOs::Windows => "WINDOWS",
                                TargetOs::Linux => "LINUX",
                                TargetOs::MacOS => "MACOS",
                                TargetOs::Wasm => "WASM",
                            },
                            &["WINDOWS", "LINUX", "MACOS", "WASM", "ANDROID", "IOS", "PS5"],
                        ),
                        "CPU" => (
                            match self.cpu {
                                TargetCpu::X64 => "X64",
                                TargetCpu::Arm64 => "ARM64",
                                TargetCpu::Wasm => "WASM",
                            },
                            &["X64", "ARM64", "WASM"],
                        ),
                        _ => return None,
                    };
                    let wanted = member.name.as_str();
                    if !known.contains(&wanted) {
                        return None;
                    }
                    Some((current == wanted) == (*op == ast::BinOp::Eq))
                }
                _ => None,
            }
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::imp::*;
    use crate::ast::AstId;
    use crate::source::FileId;
    use std::path::{Path, PathBuf};

    fn jai_files(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<_> = entries.flatten().map(|e| e.path()).collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                jai_files(&path, out);
            } else if path.extension().is_some_and(|e| e == "jai") {
                out.push(path);
            }
        }
    }

    /// A tree parsed by a worker and relocated is the tree a serial parse builds: same spans,
    /// file ids, `AstId`s and symbols, for the library, the examples and the test sources.
    #[test]
    fn relocated_parse_equals_serial_parse() {
        std::thread::Builder::new()
            .stack_size(256 << 20)
            .spawn(relocated_parse_equals_serial_parse_body)
            .unwrap()
            .join()
            .unwrap();
    }

    fn relocated_parse_equals_serial_parse_body() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut files = Vec::new();
        for dir in ["stdlib", "examples", "tests"] {
            jai_files(&root.join(dir), &mut files);
        }
        assert!(files.len() > 50);
        let mut compared = 0;
        for path in files {
            let text = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned();
            // Make this thread's symbols differ from the worker's, as in a real compile.
            for i in 0..37 {
                crate::intern::Sym::intern(&format!("__relocate_test_{i}"));
            }
            AstId::restart();
            let Ok(serial) = crate::parser::parse_file(FileId(7), &text) else {
                continue;
            };
            let expected = format!("{serial:?}");
            AstId::restart();
            let prepared = {
                let path = path.clone();
                std::thread::Builder::new()
                    .stack_size(64 << 20)
                    .spawn(move || prepare_for_test(&path))
                    .unwrap()
                    .join()
                    .unwrap()
            };
            let (_, _, tree) = prepared.expect("worker parse").into_parts();
            let relocated = tree.finish(FileId(7));
            assert_eq!(expected, format!("{relocated:?}"), "{}", path.display());
            compared += 1;
        }
        assert!(compared > 50);
    }
}
