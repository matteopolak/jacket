//! Lint one compiled root: check every program it built and lint the files it owns.
use super::asserts::{AssertFailure, failures_of};
use super::claims::{Claims, Release};
use super::compile::{Compiled, compile_root};
use super::marked_target::marked_target;
use super::plan::canonical;
use super::{CheckedContext, Options};
use crate::config::Configs;
use crate::{Lint, lint_files_with_entry};
use jaic::paths::CanonCache;
use jaic::project::Program;
use jaic::sema::Compiler;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// What linting one root found.
pub(super) struct RootResult {
    pub(super) lints: Vec<Lint>,
    /// Text of every file a lint of this root points into, by path.
    pub(super) texts: BTreeMap<String, String>,
    pub(super) error: Option<String>,
    pub(super) contexts: Vec<CheckedContext>,
    pub(super) failures: Vec<AssertFailure>,
}

/// Compile `root` with lint facts and lint the files in `report` it compiled, following the
/// workspaces its metaprogram creates (`build.jai` compiling the real program).
pub(super) fn lint_root(
    root: &Program,
    index: usize,
    options: &Options,
    configs: &Configs,
    report: &BTreeSet<PathBuf>,
    canon: &CanonCache,
    claims: &Claims,
) -> RootResult {
    let _release = Release {
        claims,
        index,
    };
    let limits = options.limits;
    // A file that says it is for another system (`// test: only(.WINDOWS)`) is compiled for it,
    // as a program or, when it sits in an import folder, as the module it counts as.
    let (os, cpu) = std::fs::read_to_string(root.path()).map_or((None, None), |source| {
        marked_target(
            &source,
            options.compiler_options.os,
            options.compiler_options.cpu,
        )
    });
    let Compiled {
        mut compiler,
        workspaces,
        done,
        reports,
        error,
        host,
    } = compile_root(root, options, os, cpu, Some(report), canon, limits);
    let mut error = error;
    let top_complete = error.is_none();
    if top_complete
        && let Err(message) =
            jaic::build::finish_all(&workspaces, &mut compiler.ct.interp.block_budget)
    {
        error = Some(message);
    }
    // The workspaces' programs first: a file both compile is the target program's.
    let mut compilers: Vec<(Box<Compiler>, bool)> = std::mem::take(&mut *done.borrow_mut());
    if error.is_none()
        && let Some((_, true)) = compilers.iter().find(|(_, failed)| *failed)
    {
        error = reports
            .borrow()
            .first()
            .map(|r| r.lines().next().unwrap_or_default().to_string())
            .or_else(|| Some("a workspace failed to compile".into()));
    }
    compilers.reverse();
    compilers.push((compiler, !top_complete));
    // One notice for the root, whatever the evaluation that ran into the limit was.
    let stopped = host.borrow_mut().has_stopped();
    if stopped {
        error = Some(limits.stopped_notice(jaic::source::Span::NONE).message);
    }
    // Check every program first: which files this root compiled settles which it reports.
    struct Checked {
        compiler: Box<Compiler>,
        complete: bool,
        /// Facts cover every requested file, including loads outside the root folder.
        files: Vec<(jaic::source::FileId, PathBuf)>,
    }
    let mut checked = Vec::new();
    let mut failures = Vec::new();
    for (mut compiler, failed) in compilers {
        failures.extend(failures_of(&compiler, root.path()));
        let diagnostics = compiler.ide_check_all();
        let complete = !failed && diagnostics.is_empty();
        if error.is_none()
            && let Some(diagnostic) = diagnostics.first()
        {
            error = Some(diagnostic.message.clone());
        }
        let files = (0..compiler.src.sources.len() as u32)
            .map(jaic::source::FileId)
            .filter_map(|f| {
                let file = compiler.src.sources.get(f).path.as_file()?;
                let path = canon.get_or_insert_with(file, canonical);
                report.contains(&path).then_some((f, path))
            })
            .collect();
        checked.push(Checked {
            compiler,
            complete,
            files,
        });
    }
    let compiled: BTreeSet<PathBuf> = checked
        .iter()
        .flat_map(|c| c.files.iter().map(|(_, path)| path.clone()))
        .collect();
    claims.publish(index, compiled.iter().cloned());
    let owned = claims.owned(index, &compiled);
    let mut lints = Vec::new();
    let mut texts = BTreeMap::new();
    let mut contexts = Vec::new();
    for Checked {
        mut compiler,
        complete,
        files,
    } in checked
    {
        for (f, path) in &files {
            if owned.contains(path) {
                texts.insert(
                    path.to_string_lossy().into_owned(),
                    compiler.src.sources.get(*f).text.to_string(),
                );
            }
        }
        contexts.push(CheckedContext {
            root: root.path().to_path_buf(),
            os: compiler.options.os,
            cpu: compiler.options.cpu,
            complete,
            visited_files: files.iter().map(|(_, path)| path.clone()).collect(),
        });
        // Files an earlier root compiled are linted, and reported, from that root.
        let own: Vec<jaic::source::FileId> = files
            .iter()
            .filter(|(_, path)| owned.contains(path))
            .map(|&(f, _)| f)
            .collect();
        if !own.is_empty() {
            let entry = if matches!(&root.root, jaic::project::Root::Module { entry, .. } if !jaic::paths::is_module_entry(entry))
                && super::plan::declares_entry(root.path(), &compiler.options.entry_point)
                && compiler.src.files.iter().any(|f| {
                    Some(f.module) == compiler.ns.main_module
                        && compiler
                            .src
                            .sources
                            .get(f.id)
                            .path
                            .as_file()
                            .is_some_and(|p| {
                                p.file_name()
                                    .is_some_and(|n| n == jaic::project::MODULE_ROOT_STUB)
                            })
                }) {
                files
                    .iter()
                    .find(|(_, path)| path == root.path())
                    .map(|(id, _)| *id)
            } else {
                None
            };
            let mut found = lint_files_with_entry(&mut compiler, &own, configs, complete, entry);
            for l in &mut found {
                l.path = canon
                    .get_or_insert_with(Path::new(&l.path), canonical)
                    .to_string_lossy()
                    .into_owned();
            }
            lints.extend(found);
        }
        if options.jobs == 1 {
            crate::facts::profile_retained(&mut compiler, root.path(), complete);
        }
        // Drop the compiler now: they hold the workspace registry, which held them.
        drop(compiler);
    }
    RootResult {
        lints,
        texts,
        error,
        contexts,
        failures,
    }
}
