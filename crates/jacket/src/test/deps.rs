//! `jacket test --list --deps`: the source files the programs the tests build read.
use super::build::{PLUGIN, Toolchain};
use super::{Case, Kind, discover};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The source files `jaic -deps` says compiling `program` reads, with the plugin that finds the
/// tests when `plugin` is set (a root's generated program). `None` when it does not compile.
fn read_files(
    program: &Path,
    root_flags: &[String],
    toolchain: &Toolchain,
    plugin: bool,
) -> Option<Vec<String>> {
    let mut command = toolchain.jaic_command();
    command
        .current_dir(program.parent().unwrap_or(Path::new(".")))
        .arg(program)
        .args(["-deps", "-quiet"])
        .args(&toolchain.flags)
        .args(root_flags);
    if plugin {
        command.arg(PLUGIN);
    }
    let output = command.output().ok().filter(|o| o.status.success())?;
    // The plugin prints its records on standard output too: a file is a line that names one.
    Some(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| Path::new(line).is_absolute() && Path::new(line).is_file())
            .map(str::to_string)
            .collect(),
    )
}

fn files(
    program: &Path,
    root_flags: &[String],
    toolchain: &Toolchain,
    plugin: bool,
) -> Vec<String> {
    read_files(program, root_flags, toolchain, plugin).unwrap_or_default()
}

/// The files every program the selected tests build reads, one per line, sorted: each root's
/// generated unit-test program (written by `build::unit` in `work/build-N`) and each golden
/// program.
pub(super) fn print(
    selected: &[Case],
    roots: &[discover::Root],
    root_flags: &[Vec<String>],
    work: &Path,
    toolchain: &Toolchain,
    bench: bool,
) {
    let mut found = std::collections::BTreeSet::new();
    for (i, root) in roots.iter().enumerate() {
        let units = if bench {
            &root.bench
        } else {
            &root.unit
        };
        let wanted = selected
            .iter()
            .any(|c| c.root == i && matches!(c.kind, Kind::Test | Kind::Bench));
        if wanted && !units.is_empty() {
            let program = work.join(format!("build-{i}")).join("tests.jai");
            found.extend(files(&program, &root_flags[i], toolchain, true));
        }
    }
    for case in selected {
        if matches!(case.kind, Kind::Golden | Kind::CompileFail) {
            found.extend(files(&case.file, &root_flags[case.root], toolchain, false));
        }
    }
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

/// What a program that imports `modules` reads: the stand-in for a program that cannot be
/// compiled alone (a compile-fail test) or is not built here (a doc example). `None` when the
/// stand-in does not compile.
fn imports_read(
    modules: &BTreeSet<String>,
    flags: &[String],
    work: &Path,
    tag: usize,
    toolchain: &Toolchain,
) -> Option<Vec<String>> {
    let text: String = modules
        .iter()
        .enumerate()
        .map(|(i, m)| format!("__dep{i} :: #import \"{m}\";\n"))
        .collect();
    let program = work.join(format!("deps-{tag}.jai"));
    std::fs::create_dir_all(work).ok()?;
    std::fs::write(&program, text).ok()?;
    read_files(&program, flags, toolchain, false)
}

/// What `--changed` reads programs with.
pub(super) struct Context<'a> {
    pub roots: &'a [discover::Root],
    pub root_flags: &'a [Vec<String>],
    pub work: &'a Path,
    pub toolchain: &'a Toolchain,
    pub bench: bool,
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
    // of all its doc examples.
    let mut unit: BTreeMap<usize, Option<BTreeSet<PathBuf>>> = BTreeMap::new();
    let mut alone: BTreeMap<PathBuf, Option<BTreeSet<PathBuf>>> = BTreeMap::new();
    let mut docs: BTreeMap<usize, Option<BTreeSet<PathBuf>>> = BTreeMap::new();
    let mut tag = 0;
    for case in &selected {
        let i = case.root;
        match case.kind {
            Kind::Test | Kind::Bench if !unit.contains_key(&i) => {
                let found = unit_program(work, i, &roots[i], bench)
                    .and_then(|p| read_files(&p, &root_flags[i], toolchain, true));
                unit.insert(i, found.map(&canon));
            }
            Kind::Golden if !alone.contains_key(&case.file) => {
                let found = read_files(&case.file, &root_flags[i], toolchain, false);
                alone.insert(case.file.clone(), found.map(&canon));
            }
            Kind::CompileFail if !alone.contains_key(&case.file) => {
                tag += 1;
                let found = std::fs::read_to_string(&case.file)
                    .ok()
                    .and_then(|text| imported_modules(&text))
                    .map(|m| m.into_iter().chain(base(i)).collect())
                    .and_then(|m| imports_read(&m, &root_flags[i], work, tag, toolchain));
                alone.insert(case.file.clone(), found.map(&canon));
            }
            Kind::Doc if !docs.contains_key(&i) => {
                tag += 1;
                let found = roots[i]
                    .docs
                    .iter()
                    .map(|block| imported_modules(&block.code))
                    .try_fold(base(i), |all, block| {
                        block.map(|m| all.into_iter().chain(m).collect())
                    })
                    .and_then(|m| imports_read(&m, &root_flags[i], work, tag, toolchain));
                docs.insert(i, found.map(&canon));
            }
            _ => {}
        }
    }
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
