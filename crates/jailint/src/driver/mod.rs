//! From paths on the command line to lints: which files to compile as what, and which
//! compiled files to report on.
//!
//! - A directory stands for the `.jai` files under it; a file for itself.
//! - Files another listed file `#load`s are compiled as part of it, not on their own.
//! - A file a module's `module.jai` `#load`s is compiled as part of that module even when only
//!   the file is listed: as a program of its own, its exported procedures would look unused.
//! - A file a nearby program `#load`s (in its directory or up to two above) is compiled as part
//!   of that program too, so listing only `basics.jai` of a program that loads it lints it in
//!   context; only the listed files are reported.
//! - `module.jai` (and a `.jai` file directly in an import directory, such as the stdlib) is
//!   a module: it is compiled by importing it from an empty program, and every procedure in
//!   it is checked, called or not.
//! - Anything else is a program and is compiled from that file.
//!
//! With `platform_leak` on, each root whose entry file is portable is compiled again for every
//! target operating system ([`leak`]), keeping going past failing `#assert`s.
//!
//! Lints are reported for the listed files and the files listed files `#load`. A file several
//! roots compile is reported from the first root (modules before programs, then by path).
//!
//! The modules, in the order a run uses them: [`plan`] (roots and files to report), [`compile`]
//! (one root, through the compile the language server also uses), [`lint_root`] (check and lint
//! what it built), [`claims`] (which root reports a shared file), [`leak`] (the per-system
//! pass), with [`asserts`] and [`rooted_fs`] as their shared parts.
/// Most roots compiled at once when the number is left to `jailint`: each holds a whole
/// compiler, so memory grows with it long before the CPUs run out.
mod asserts;
mod claims;
mod compile;
mod leak;
mod lint_root;
mod marked_target;
mod plan;
mod rooted_fs;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;

pub use asserts::{AssertFailure, ImportSite};
use claims::Claims;
use jaic::paths::CanonCache;
use jaic::sema::Options as CompilerOptions;
use lint_root::{RootResult, Shared, lint_root};
pub use plan::root_import_paths;

use crate::Lint;
use crate::config::{Config, Configs};

pub const MAX_AUTO_JOBS: usize = 8;

pub struct Options {
    /// Compiler source/target/runtime settings, cloned for each root and workspace.
    pub compiler_options: CompilerOptions,
    /// Optional actual enclosing program supplying imports, module arguments and source origin.
    /// `paths` still determines which files receive lint reports.
    pub source: Option<PathBuf>,
    pub paths: Vec<PathBuf>,
    /// Extra import directories, searched after the root's `modules/` folder.
    pub import_dirs: Vec<PathBuf>,
    pub stdlib: PathBuf,
    pub config: Config,
    /// Compile this many roots at once; 0 picks one per CPU, at most [`MAX_AUTO_JOBS`].
    pub jobs: usize,
    /// Print each root as it is compiled.
    pub verbose: bool,
    /// Additional source strings, in the program's scope (same as jaic -add).
    pub add_sources: Vec<String>,
    /// What bounds the compile-time code of each root: set by whoever runs the linter, never
    /// by the project.
    pub limits: jaic::ide::lint_compile::CheckLimits,
    /// Files compiled from this text instead of what is on disk, by canonical path
    /// (`migrate` checks edits this way before writing them).
    pub overlay: BTreeMap<PathBuf, Arc<[u8]>>,
}

pub struct Outcome {
    pub lints: Vec<Lint>,
    /// Compile errors in the reported files, each reported once, from the root that owns the file.
    pub errors: Vec<CompileError>,
    /// Every file the run reports on.
    pub files: Vec<PathBuf>,
    /// Text of every file a lint points into, by path.
    pub texts: BTreeMap<String, Arc<str>>,
    /// Roots the compiler could not finish, with its first error (their lints may be fewer).
    pub incomplete: Vec<(PathBuf, String)>,
    pub roots: usize,
    /// Actual compiler contexts, including child workspaces and failed body checks.
    pub contexts: Vec<CheckedContext>,
    /// Requested/load-discovered files absent from every compiled context (e.g. inactive loads).
    pub unvisited_files: Vec<PathBuf>,
    /// Every `#assert` that failed in a compile, with the imports that led to its file. A normal
    /// compile stops at its first; the `platform_leak` compiles keep going.
    pub failed_asserts: Vec<AssertFailure>,
    /// Nested `jacket.toml` files that could not be read; their files used the first root's settings.
    pub config_errors: Vec<String>,
    /// `// test:` notes about where a file runs that did not parse, each with its file's path;
    /// they were ignored.
    pub test_note_problems: Vec<(PathBuf, String)>,
}

/// A compile error in a reported file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompileError {
    /// Canonical path of the file.
    pub path: PathBuf,
    /// Byte range into the file's text.
    pub start: usize,
    pub end: usize,
    /// The diagnostic code (`unknown-identifier`).
    pub code: &'static str,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct CheckedContext {
    pub root: PathBuf,
    pub os: jaic::target::TargetOs,
    pub cpu: jaic::target::TargetCpu,
    pub complete: bool,
    /// Source files visited by linting; complete=false never establishes full body coverage.
    pub visited_files: Vec<PathBuf>,
    /// With `unreachable_declaration` on: the declared names (`path`, start, end) this program
    /// never reaches, or `None` when it did not compile cleanly and cannot say.
    pub dead: Option<BTreeSet<(String, usize, usize)>>,
}

impl Options {
    /// How many roots are compiled at once.
    fn workers(&self) -> usize {
        if self.jobs == 0 {
            jaic::pool::cores().min(MAX_AUTO_JOBS)
        } else {
            self.jobs
        }
    }
}

/// Lint what `options` names.
pub fn run(options: &Options) -> Outcome {
    run_planned(options, None)
}

/// Lint the sources actually included by project entries, in their compiled contexts.
/// `paths` bounds reporting; files outside those entries' graphs remain unvisited.
/// For example, a CLI with no paths uses `run_project(options, &["first.jai".into()])`.
pub fn run_project(options: &Options, entries: &[PathBuf]) -> Outcome {
    run_planned(options, Some(entries))
}

fn run_planned(options: &Options, entries: Option<&[PathBuf]>) -> Outcome {
    let configs = Configs::new(&options.config);
    let (roots, report, notes) = plan::plan(options, &configs, entries);
    let mut results: Vec<Option<RootResult>> = (0..roots.len()).map(|_| None).collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let collected = std::sync::Mutex::new(Vec::new());
    let canon = CanonCache::new();
    let caches = compile::RunCaches::default();
    let claims = Claims::new(roots.len());
    let wanted = options.workers();
    let jobs = wanted.min(roots.len().max(1));

    // The compiler's pool runs the checkers (its threads have the compiler's large stack, which
    // checking recurses on); the job count is its size unless something started it already.
    jaic::pool::configure(wanted);
    jaic::pool::get().scope(|s| {
        for _ in 0..jobs {
            let (roots, next, collected) = (&roots, &next, &collected);
            let shared = Shared {
                options,
                configs: &configs,
                report: &report,
                notes: &notes,
                canon: &canon,
                caches: &caches,
                claims: &claims,
            };

            s.spawn(move |_| {
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(root) = roots.get(i) else {
                        break;
                    };

                    if options.verbose {
                        eprintln!("jailint: checking {}", root.path().display());
                    }

                    let result = lint_root(root, i, &shared);

                    collected
                        .lock()
                        .expect("no thread panicked holding the lock")
                        .push((i, result));
                }
            });
        }
    });

    for (i, result) in collected.into_inner().unwrap_or_default() {
        results[i] = Some(result);
    }

    let mut lints = Vec::new();
    let mut errors = Vec::new();
    let mut texts: BTreeMap<String, Arc<str>> = BTreeMap::new();
    let mut incomplete = Vec::new();
    let mut owner: BTreeMap<String, usize> = BTreeMap::new();

    for (i, r) in results.iter().enumerate() {
        let Some(r) = r else {
            continue;
        };

        for path in r.texts.keys() {
            owner.entry(path.clone()).or_insert(i);
        }

        if let Some(e) = &r.error {
            incomplete.push((roots[i].path().to_path_buf(), e.clone()));
        }
    }

    let mut contexts = Vec::new();
    let mut failed_asserts = Vec::new();
    // A note is read when any root read it, not only the one that lints its file.
    let read_notes: BTreeSet<(String, usize, usize)> = results
        .iter()
        .flatten()
        .flat_map(|r| r.read_notes.iter().cloned())
        .collect();
    // Each file once, wherever it was loaded from.
    let test_note_problems = notes
        .iter()
        .flat_map(|(path, notes)| notes.problems.iter().map(|p| (path.clone(), p.clone())))
        .collect();

    for (i, r) in results.into_iter().enumerate() {
        let Some(r) = r else {
            continue;
        };

        contexts.extend(r.contexts);
        failed_asserts.extend(r.failures);

        for l in r.lints {
            if l.rule == crate::rules::unused_note::INFO.name
                && read_notes.contains(&(l.path.clone(), l.start, l.end))
            {
                continue;
            }

            if owner.get(&l.path) == Some(&i) {
                lints.push(l);
            }
        }

        for e in r.errors {
            if owner.get(e.path.to_string_lossy().as_ref()) == Some(&i) {
                errors.push(e);
            }
        }

        for (p, text) in r.texts {
            if owner.get(&p) == Some(&i) {
                texts.entry(p).or_insert_with(|| Arc::from(text));
            }
        }
    }

    keep_dead_in_every_program(&mut lints, &contexts);
    leak::leak_pass(
        (&roots, &report),
        options,
        &configs,
        &canon,
        &mut lints,
        &mut texts,
        &mut failed_asserts,
    );
    lints.sort_by(|a, b| (&a.path, a.start, a.rule).cmp(&(&b.path, b.start, b.rule)));
    lints.dedup_by(|a, b| {
        a.path == b.path
            && a.start == b.start
            && a.end == b.end
            && a.rule == b.rule
            && a.message == b.message
    });
    let visited: BTreeSet<_> = contexts
        .iter()
        .flat_map(|context| context.visited_files.iter().cloned())
        .collect();
    let unvisited_files: Vec<_> = report.difference(&visited).cloned().collect();

    if incomplete.is_empty()
        && let Some(source) = &options.source
    {
        for path in options.paths.iter().filter(|path| path.is_file()) {
            let path = plan::canonical(path);

            if unvisited_files.contains(&path) {
                incomplete.push((source.clone(), format!("context did not visit requested file `{}`; choose its actual enclosing program and target", path.display())));
            }
        }
    }

    errors.sort_by(|a, b| (&a.path, a.start, a.code).cmp(&(&b.path, b.start, b.code)));
    Outcome {
        lints,
        errors,
        files: report.iter().cloned().collect(),
        texts,
        incomplete,
        roots: roots.len(),
        contexts,
        unvisited_files,
        failed_asserts,
        config_errors: configs.take_errors(),
        test_note_problems,
    }
}

/// A declaration is unreachable only when every program that compiled its file leaves it
/// unreached: a file two programs `#load` is used by what either of them calls.
fn keep_dead_in_every_program(lints: &mut Vec<Lint>, contexts: &[CheckedContext]) {
    let rule = crate::rules::unreachable_declaration::INFO.name;

    lints.retain(|l| {
        l.rule != rule
            || contexts
                .iter()
                .filter(|c| {
                    c.visited_files
                        .iter()
                        .any(|f| f.to_string_lossy() == l.path)
                })
                .all(|c| {
                    c.dead
                        .as_ref()
                        .is_some_and(|dead| dead.contains(&(l.path.clone(), l.start, l.end)))
                })
    });
}
