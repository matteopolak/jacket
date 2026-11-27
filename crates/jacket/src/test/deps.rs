//! `jacket test --list --deps`: the source files the programs the tests build read.
use super::build::{PLUGIN, Toolchain};
use super::deps_store::{self, Program};
use super::{Case, Kind, discover};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A program whose reads are wanted, and how the store knows it.
struct Query {
    /// Its place in the batch; names the scratch list when there is no store.
    id: usize,
    path: PathBuf,
    /// Its text when jacket generated it into a folder that differs from run to run (a root's
    /// unit tests, the stand-in for a compile-fail test): the store names it by that, not by
    /// where it is.
    text: Option<String>,
    /// The root whose flags it compiles with.
    root: usize,
    /// Compile with the plugin that finds the tests (a root's generated program).
    plugin: bool,
}

impl Query {
    /// A program of the project, named by its path.
    fn file(id: usize, path: PathBuf, root: usize) -> Self {
        Self {
            id,
            path,
            text: None,
            root,
            plugin: false,
        }
    }

    /// A program jacket generated at `path`, named by its text; `None` when it cannot be read.
    fn generated(id: usize, path: PathBuf, root: usize, plugin: bool) -> Option<Self> {
        let text = std::fs::read_to_string(&path).ok()?;
        Some(Self {
            id,
            path,
            text: Some(text),
            root,
            plugin,
        })
    }

    fn program(&self) -> Program<'_> {
        match &self.text {
            Some(text) => Program::Text(text),
            None => Program::File(&self.path),
        }
    }
}

/// How a program is compiled to learn what it reads: `true` when it wrote the list to the path.
type Compile<'a> = &'a (dyn Fn(&Query, &Path) -> bool + Sync);

/// Compiles the program of `query` with `jaic -check -deps_file list`.
fn compile_list(query: &Query, root_flags: &[String], toolchain: &Toolchain, list: &Path) -> bool {
    let mut command = toolchain.jaic_command();
    command
        .current_dir(query.path.parent().unwrap_or(Path::new(".")))
        .arg(&query.path)
        .args(["-check", "-quiet"])
        .args(&toolchain.flags)
        .args(root_flags)
        .arg("-deps_file")
        .arg(list);
    if query.plugin {
        command.arg(PLUGIN);
    }
    command.output().is_ok_and(|o| o.status.success())
}

/// The source files the program of `query` reads, `None` when it does not compile: the list the
/// store has, else what `compile` writes (into the store, when there is one).
///
/// A list recorded by an earlier compile may be out of date, which is sound for `--changed`: a
/// program starts reading a file only if a file it already read changed to import or load it,
/// and that file is in the old list, so the program is selected anyway. A new file that shadows
/// module resolution is the unlisted file of a root that `unknown_to_deps` selects, and a program
/// that does not compile has no list and is always kept.
fn list(
    query: &Query,
    root_flags: &[String],
    toolchain: &Toolchain,
    work: &Path,
    compile: Compile,
) -> Option<Vec<String>> {
    let entry = toolchain.deps_entry(query.program(), root_flags, query.plugin);
    if let Some(files) = entry.as_deref().and_then(deps_store::read) {
        return Some(files);
    }
    let target = if let (Some(entry), Some(store)) = (&entry, &toolchain.deps) {
        store.prepare(entry);
        entry.clone()
    } else {
        let _ = std::fs::create_dir_all(work);
        work.join(format!("deps-{}.list", query.id))
    };
    let files = compile(query, &target)
        .then(|| deps_store::read(&target))
        .flatten();
    if entry.is_none() {
        let _ = std::fs::remove_file(&target);
    }
    files
}

/// `list` of every query, `jobs` at a time, in the order of `queries`.
fn lists(
    queries: &[Query],
    root_flags: &[Vec<String>],
    toolchain: &Toolchain,
    work: &Path,
    jobs: usize,
    compile: Compile,
) -> Vec<Option<Vec<String>>> {
    let next = AtomicUsize::new(0);
    let found: Mutex<Vec<Option<Vec<String>>>> = Mutex::new(vec![None; queries.len()]);
    std::thread::scope(|scope| {
        for _ in 0..jobs.max(1).min(queries.len()) {
            scope.spawn(|| {
                while let Some(query) = queries.get(next.fetch_add(1, Ordering::Relaxed)) {
                    let files = list(query, &root_flags[query.root], toolchain, work, compile);
                    found.lock().expect("lock")[query.id] = files;
                }
            });
        }
    });
    found.into_inner().expect("lock")
}

/// `lists`, compiling with jaic.
fn read_all(
    queries: &[Query],
    root_flags: &[Vec<String>],
    toolchain: &Toolchain,
    work: &Path,
    jobs: usize,
) -> Vec<Option<Vec<String>>> {
    lists(
        queries,
        root_flags,
        toolchain,
        work,
        jobs,
        &|query, list| compile_list(query, &root_flags[query.root], toolchain, list),
    )
}

/// What `--list --deps` reads programs with.
pub(super) struct Listing<'a> {
    pub root_flags: &'a [Vec<String>],
    pub work: &'a Path,
    pub toolchain: &'a Toolchain,
    pub bench: bool,
    pub jobs: usize,
}

/// The files every program the selected tests build reads, one per line, sorted: each root's
/// generated unit-test program (written by `build::unit` in `work/build-N`) and each golden
/// program.
pub(super) fn print(selected: &[Case], roots: &[discover::Root], ctx: &Listing) {
    let mut queries = Vec::new();
    for (i, root) in roots.iter().enumerate() {
        let wanted = selected
            .iter()
            .any(|c| c.root == i && matches!(c.kind, Kind::Test | Kind::Bench));
        if let Some(program) = unit_program(ctx.work, i, root, ctx.bench).filter(|_| wanted) {
            queries.extend(Query::generated(queries.len(), program, i, true));
        }
    }
    for case in selected {
        if matches!(case.kind, Kind::Golden | Kind::CompileFail) {
            queries.push(Query::file(queries.len(), case.file.clone(), case.root));
        }
    }
    let found: BTreeSet<String> =
        read_all(&queries, ctx.root_flags, ctx.toolchain, ctx.work, ctx.jobs)
            .into_iter()
            .flatten()
            .flatten()
            // A list outlives the files it names.
            .filter(|file| Path::new(file).is_file())
            .collect();
    for file in found {
        println!("{file}");
    }
}

/// The generated unit-test (or bench) program of root `i`, if it has one.
fn unit_program(work: &Path, i: usize, root: &discover::Root, bench: bool) -> Option<PathBuf> {
    let units = if bench {
        &root.bench
    } else {
        &root.unit
    };
    (!units.is_empty()).then(|| work.join(format!("build-{i}")).join("tests.jai"))
}

/// Whether a changed `path` of `root` is one no dependency list can speak for: a file of the
/// root that no program read (a new test, a data file, a C source) and that is not a plain
/// module source, whose readers are all in the lists.
fn unknown_to_deps(path: &Path, root: &Path, listed: &BTreeSet<PathBuf>) -> bool {
    if !path.starts_with(root) || listed.contains(path) {
        return false;
    }
    path.extension().is_none_or(|e| e != "jai") || path.starts_with(root.join("tests"))
}

/// Whether a program that read `deps` (`None`: it did not compile) is affected by `changed`.
fn touches(changed: &[PathBuf], deps: Option<&BTreeSet<PathBuf>>) -> bool {
    deps.is_none_or(|deps| changed.iter().any(|path| deps.contains(path)))
}

/// The modules `text` imports by name, or `None` when it reads anything else (`#load`, a
/// relative or parameterised `#import`), which a program of imports cannot stand in for.
fn imported_modules(text: &str) -> Option<BTreeSet<String>> {
    let mut found = BTreeSet::new();
    for line in text.lines().map(str::trim_start) {
        if line.starts_with("//") {
            continue;
        }
        for (at, _) in line.match_indices("#load") {
            if !line[at + 5..].starts_with(char::is_alphanumeric) {
                return None;
            }
        }
        let mut rest = line;
        while let Some(at) = rest.find("#import") {
            let after = &rest[at + "#import".len()..];
            let name = after.trim_start().strip_prefix('"')?;
            let (name, tail) = name.split_once('"')?;
            if name.starts_with(['.', '/']) {
                return None;
            }
            found.insert(name.to_string());
            rest = tail;
        }
    }
    Some(found)
}

/// The stand-in for a program that cannot be compiled alone (a compile-fail test) or is not
/// built here (a doc example): a program that imports `modules`, written to `work`. `None` when
/// it cannot be written.
fn stand_in(modules: &BTreeSet<String>, work: &Path, root: usize, id: usize) -> Option<Query> {
    let text: String = modules
        .iter()
        .enumerate()
        .map(|(i, m)| format!("__dep{i} :: #import \"{m}\";\n"))
        .collect();
    let program = work.join(format!("deps-{id}.jai"));
    std::fs::create_dir_all(work).ok()?;
    std::fs::write(&program, text).ok()?;
    Query::generated(id, program, root, false)
}

/// What `--changed` reads programs with.
pub(super) struct Context<'a> {
    pub roots: &'a [discover::Root],
    pub root_flags: &'a [Vec<String>],
    pub work: &'a Path,
    pub toolchain: &'a Toolchain,
    pub bench: bool,
    /// How many programs are read at once.
    pub jobs: usize,
}

/// `affected`, saying on standard error how many of the cases stay.
pub(super) fn narrow(selected: Vec<Case>, changed: &[PathBuf], ctx: &Context) -> Vec<Case> {
    let before = selected.len();
    let kept = affected(selected, changed, ctx);
    eprintln!(
        "jacket: {} of {before} tests can be affected by the {} changed files",
        kept.len(),
        changed.len()
    );
    kept
}

/// The `selected` cases a change to the `changed` files can affect: those whose program reads one
/// of them, those of a root the change adds an unlisted file to, and every case whose program
/// does not compile (it may fail to because a file is gone). A compile-fail test and a doc
/// example cannot be read alone, so they stand for the modules they import (and their root's).
fn affected(selected: Vec<Case>, changed: &[PathBuf], ctx: &Context) -> Vec<Case> {
    let Context {
        roots,
        root_flags,
        work,
        toolchain,
        bench,
        jobs,
    } = *ctx;
    let changed: Vec<PathBuf> = changed.iter().map(|p| super::watch::canonical(p)).collect();
    let canon = |files: Vec<String>| -> BTreeSet<PathBuf> {
        files
            .iter()
            .map(|f| super::watch::canonical(Path::new(f)))
            .collect()
    };
    let base = |i: usize| -> BTreeSet<String> {
        let mut modules: BTreeSet<String> =
            ["Basic", "Extensions/Testing"].map(String::from).into();
        if roots[i].library {
            modules.insert(roots[i].module.clone());
        }
        modules
    };
    // What each program reads, `None` when it cannot be known (the case stays): a root's unit
    // program, a golden program, the imports of a compile-fail test, and per root the imports
    // of all its doc examples. The programs are collected first and read together, in parallel.
    let mut queries: Vec<Query> = Vec::new();
    let mut unit: BTreeMap<usize, Option<usize>> = BTreeMap::new();
    let mut alone: BTreeMap<PathBuf, Option<usize>> = BTreeMap::new();
    let mut docs: BTreeMap<usize, Option<usize>> = BTreeMap::new();
    let mut add = |make: &dyn Fn(usize) -> Option<Query>| -> Option<usize> {
        let query = make(queries.len())?;
        queries.push(query);
        Some(queries.len() - 1)
    };
    for case in &selected {
        let i = case.root;
        match case.kind {
            Kind::Test | Kind::Bench if !unit.contains_key(&i) => {
                let program = unit_program(work, i, &roots[i], bench);
                unit.insert(
                    i,
                    add(&|id| Query::generated(id, program.clone()?, i, true)),
                );
            }
            Kind::Golden if !alone.contains_key(&case.file) => {
                let query = add(&|id| Some(Query::file(id, case.file.clone(), i)));
                alone.insert(case.file.clone(), query);
            }
            Kind::CompileFail if !alone.contains_key(&case.file) => {
                let modules = std::fs::read_to_string(&case.file)
                    .ok()
                    .and_then(|text| imported_modules(&text))
                    .map(|m| m.into_iter().chain(base(i)).collect());
                let query = add(&|id| stand_in(modules.as_ref()?, work, i, id));
                alone.insert(case.file.clone(), query);
            }
            Kind::Doc if !docs.contains_key(&i) => {
                let modules = roots[i]
                    .docs
                    .iter()
                    .map(|block| imported_modules(&block.code))
                    .try_fold(base(i), |all, block| {
                        block.map(|m| all.into_iter().chain(m).collect())
                    });
                let query = add(&|id| stand_in(modules.as_ref()?, work, i, id));
                docs.insert(i, query);
            }
            _ => {}
        }
    }
    let read = read_all(&queries, root_flags, toolchain, work, jobs);
    let found = |query: Option<usize>| query.and_then(|q| read[q].clone()).map(canon);
    let unit: BTreeMap<usize, Option<BTreeSet<PathBuf>>> =
        unit.into_iter().map(|(k, q)| (k, found(q))).collect();
    let alone: BTreeMap<PathBuf, Option<BTreeSet<PathBuf>>> =
        alone.into_iter().map(|(k, q)| (k, found(q))).collect();
    let docs: BTreeMap<usize, Option<BTreeSet<PathBuf>>> =
        docs.into_iter().map(|(k, q)| (k, found(q))).collect();
    let mut listed: BTreeMap<usize, BTreeSet<PathBuf>> = BTreeMap::new();
    for (i, found) in unit.iter().chain(docs.iter()) {
        listed
            .entry(*i)
            .or_default()
            .extend(found.iter().flatten().cloned());
    }
    for case in &selected {
        if let Some(found) = alone.get(&case.file) {
            listed
                .entry(case.root)
                .or_default()
                .extend(found.iter().flatten().cloned());
        }
    }
    let root_touched = |i: usize| {
        let none = BTreeSet::new();
        let known = listed.get(&i).unwrap_or(&none);
        let dir = super::watch::canonical(&roots[i].dir);
        changed.iter().any(|p| unknown_to_deps(p, &dir, known))
    };
    selected
        .into_iter()
        .filter(|case| {
            let reads = match case.kind {
                Kind::Test | Kind::Bench => unit.get(&case.root),
                Kind::Golden | Kind::CompileFail => alone.get(&case.file),
                Kind::Doc => docs.get(&case.root),
            };
            touches(&changed, reads.and_then(Option::as_ref)) || root_touched(case.root)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(paths: &[&str]) -> BTreeSet<PathBuf> {
        paths.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn a_program_is_touched_by_a_file_it_read() {
        let deps = set(&["/s/Math/module.jai", "/s/Basic/module.jai"]);
        assert!(touches(
            &[PathBuf::from("/s/Basic/module.jai")],
            Some(&deps)
        ));
        assert!(!touches(
            &[PathBuf::from("/s/Sort/module.jai")],
            Some(&deps)
        ));
    }

    #[test]
    fn imports_are_read_by_name() {
        let text = "// #import \"Skipped\";\nA :: #import \"Sort\"; #import \"Basic\";\n";
        assert_eq!(
            imported_modules(text),
            Some(["Basic", "Sort"].map(String::from).into())
        );
        assert_eq!(imported_modules("#load \"x.jai\";"), None);
        assert_eq!(imported_modules("#import,file \"x.jai\";"), None);
        assert_eq!(imported_modules("#import \"./x\";"), None);
        assert!(imported_modules("loaded := 1; // #loader").is_some());
    }

    #[test]
    fn a_program_that_does_not_compile_is_always_touched() {
        assert!(touches(&[PathBuf::from("/s/Sort/module.jai")], None));
    }

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("jacket-deps-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn toolchain(dir: &Path, stored: bool) -> Toolchain {
        let jaic = dir.join("jaic");
        std::fs::write(&jaic, "compiler").unwrap();
        Toolchain {
            deps: stored
                .then(|| std::sync::Arc::new(deps_store::Store::at(dir.join("store"), &jaic))),
            jaic,
            stdlib: PathBuf::from("/stdlib"),
            flags: Vec::new(),
            interp: false,
            runner: Vec::new(),
            os: "linux".into(),
            coverage: false,
            coverage_dir: None,
            check_only: false,
            environment_flags: 0,
        }
    }

    /// A compile that writes `files` to the list and counts how often it ran.
    fn compiling(files: &'static str, runs: &AtomicUsize) -> impl Fn(&Query, &Path) -> bool + Sync {
        move |_, list| {
            runs.fetch_add(1, Ordering::Relaxed);
            std::fs::write(list, files).is_ok()
        }
    }

    #[test]
    fn a_program_with_no_entry_is_compiled_once_and_then_stored() {
        let dir = temp("fallback");
        let toolchain = toolchain(&dir, true);
        let queries: Vec<Query> = (0..3)
            .map(|id| Query::file(id, PathBuf::from(format!("/p/{id}.jai")), 0))
            .collect();
        let flags = [Vec::new()];
        let runs = AtomicUsize::new(0);
        let compile = compiling("/s/a.jai\n/s/b.jai\n", &runs);
        let first = lists(&queries, &flags, &toolchain, &dir, 2, &compile);
        assert_eq!(runs.load(Ordering::Relaxed), 3);
        let wanted = Some(vec!["/s/a.jai".to_string(), "/s/b.jai".to_string()]);
        assert!(first.iter().all(|found| *found == wanted), "{first:?}");
        // The lists are in the store now: nothing compiles, whatever the compile would say.
        let compile = compiling("/s/other.jai\n", &runs);
        let second = lists(&queries, &flags, &toolchain, &dir, 2, &compile);
        assert_eq!(
            runs.load(Ordering::Relaxed),
            3,
            "a stored list is not compiled again"
        );
        assert_eq!(first, second);
    }

    #[test]
    fn a_stored_list_is_used_without_a_compile() {
        let dir = temp("stored");
        let toolchain = toolchain(&dir, true);
        assert!(Query::generated(0, dir.join("tests.jai"), 0, true).is_none());
        std::fs::write(dir.join("tests.jai"), "#load \"a.jai\";\n").unwrap();
        let query = Query::generated(0, dir.join("tests.jai"), 0, true).unwrap();
        let entry = toolchain
            .deps_entry(query.program(), &[], true)
            .expect("a store");
        std::fs::create_dir_all(entry.parent().unwrap()).unwrap();
        std::fs::write(&entry, "/s/a.jai\n").unwrap();
        let runs = AtomicUsize::new(0);
        let compile = compiling("/s/never.jai\n", &runs);
        let found = lists(&[query], &[Vec::new()], &toolchain, &dir, 1, &compile);
        assert_eq!(found, [Some(vec!["/s/a.jai".to_string()])]);
        assert_eq!(runs.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn a_program_that_does_not_compile_has_no_list() {
        let dir = temp("failing");
        let toolchain = toolchain(&dir, true);
        let query = Query::file(0, PathBuf::from("/p/a.jai"), 0);
        let entry = toolchain.deps_entry(query.program(), &[], false).unwrap();
        let failing = |_: &Query, _: &Path| false;
        let found = lists(&[query], &[Vec::new()], &toolchain, &dir, 1, &failing);
        assert_eq!(found, [None]);
        assert!(!entry.exists());
    }

    #[test]
    fn the_flags_of_the_environment_do_not_change_the_entry() {
        let dir = temp("environment");
        let mut toolchain = toolchain(&dir, true);
        let program = Program::File(Path::new("/p/a.jai"));
        let before = toolchain.deps_entry(program, &[], false);
        toolchain.flags.extend(["-add".into(), "#run {}".into()]);
        assert_ne!(before, toolchain.deps_entry(program, &[], false));
        toolchain.environment_flags = 2;
        assert_eq!(before, toolchain.deps_entry(program, &[], false));
    }

    #[test]
    fn without_a_store_a_list_is_read_from_a_scratch_file() {
        let dir = temp("unstored");
        let toolchain = toolchain(&dir, false);
        let queries = [Query::file(0, PathBuf::from("/p/a.jai"), 0)];
        let runs = AtomicUsize::new(0);
        let compile = compiling("/s/a.jai\n", &runs);
        let found = lists(&queries, &[Vec::new()], &toolchain, &dir, 1, &compile);
        assert_eq!(found, [Some(vec!["/s/a.jai".to_string()])]);
        assert!(!dir.join("deps-0.list").exists());
    }

    #[test]
    fn an_unlisted_file_of_a_root_selects_it() {
        let root = Path::new("/s/Math");
        let listed = set(&["/s/Math/module.jai", "/s/Math/tests/a.jai"]);
        assert!(unknown_to_deps(
            Path::new("/s/Math/tests/new.jai"),
            root,
            &listed
        ));
        assert!(unknown_to_deps(
            Path::new("/s/Math/tests/a.stdout"),
            root,
            &listed
        ));
        assert!(unknown_to_deps(
            Path::new("/s/Math/helper.c"),
            root,
            &listed
        ));
        assert!(!unknown_to_deps(
            Path::new("/s/Math/tests/a.jai"),
            root,
            &listed
        ));
        assert!(!unknown_to_deps(
            Path::new("/s/Math/extra.jai"),
            root,
            &listed
        ));
        assert!(!unknown_to_deps(
            Path::new("/s/Sort/helper.c"),
            root,
            &listed
        ));
    }
}
