//! Lint one compiled root: check every program it built and lint the files it owns.
use super::asserts::{AssertFailure, failures_of};
use super::claims::{Claims, Release};
use super::compile::{Compiled, RunCaches, compile_root};
use super::marked_target::{Marked, marked_target};
use super::plan::{Notes, canonical};
use super::{CheckedContext, CompileError, Options};
use crate::config::Configs;
use crate::{Edit, Fix, Level, Lint, SecondaryLabel, lint_files_with_entry};
use jaic::paths::CanonCache;
use jaic::project::Program;
use jaic::sema::Compiler;
use jaic::source::Applicability;
use jaic::source::Span;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// What linting one root found.
pub(super) struct RootResult {
    pub(super) lints: Vec<Lint>,
    /// Text of every file a lint of this root points into, by path.
    pub(super) texts: BTreeMap<String, String>,
    pub(super) error: Option<String>,
    /// Compile errors in the files this root owns.
    pub(super) errors: Vec<CompileError>,
    pub(super) contexts: Vec<CheckedContext>,
    pub(super) failures: Vec<AssertFailure>,
}

/// What every root of a run shares.
pub(super) struct Shared<'a> {
    pub(super) options: &'a Options,
    pub(super) configs: &'a Configs,
    /// The files to report on.
    pub(super) report: &'a BTreeSet<PathBuf>,
    pub(super) notes: &'a Notes,
    pub(super) canon: &'a CanonCache,
    pub(super) caches: &'a RunCaches,
    pub(super) claims: &'a Claims,
}

/// Compile `root` with lint facts and lint the files in `report` it compiled, following the
/// workspaces its metaprogram creates (`build.jai` compiling the real program).
pub(super) fn lint_root(root: &Program, index: usize, shared: &Shared) -> RootResult {
    let Shared {
        options,
        configs,
        report,
        notes,
        canon,
        caches,
        claims,
    } = *shared;
    let _release = Release {
        claims,
        index,
    };
    let limits = options.limits;
    // A file that says it is for another system (`// test: only(.WINDOWS)`) is compiled for it,
    // as a program or, when it sits in an import folder, as the module it counts as.
    let (os, cpu) = match marked_target(
        &notes[root.path()].target,
        options.compiler_options.os,
        options.compiler_options.cpu,
    ) {
        Marked::Other(os, cpu) => (Some(os), Some(cpu)),
        // A file named on the command line that no target runs is still compiled on the host.
        _ => (None, None),
    };
    let Compiled {
        mut compiler,
        workspaces,
        done,
        reports,
        error,
        stopping,
        host,
    } = compile_root(root, options, os, cpu, Some(report), canon, caches);
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
        diagnostics: Vec<jaic::diagnostics::Diagnostic>,
    }
    let mut checked = Vec::new();
    let mut failures = Vec::new();
    let top = compilers.len() - 1;
    for (i, (mut compiler, failed)) in compilers.into_iter().enumerate() {
        failures.extend(failures_of(&compiler, root.path()));
        let mut diagnostics = compiler.ide_check_all();
        if i == top
            && let Some(stopping) = &stopping
        {
            diagnostics.insert(0, (**stopping).clone());
        }
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
            diagnostics,
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
    let mut errors = Vec::new();
    let mut contexts = Vec::new();
    for Checked {
        mut compiler,
        complete,
        files,
        diagnostics,
    } in checked
    {
        for d in &diagnostics {
            if d.severity != jaic::diagnostics::Severity::Error
                || d.span.file == jaic::source::Span::NONE.file
            {
                continue;
            }
            let Some((_, path)) = files.iter().find(|(f, _)| *f == d.span.file) else {
                continue;
            };
            if owned.contains(path) {
                errors.push(CompileError {
                    path: path.clone(),
                    start: d.span.start as usize,
                    end: d.span.end as usize,
                    code: d.code.name(),
                    message: d.message.clone(),
                });
            }
        }
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
            lints.extend(compiler_warnings(&compiler, &own, configs, canon));
        }
        if options.jobs == 1 {
            crate::facts::profile_retained(&mut compiler, root.path(), complete);
        }
        // Drop the compiler now: they hold the workspace registry, which held them.
        drop(compiler);
    }
    errors.sort_by(|a, b| (&a.path, a.start, a.code).cmp(&(&b.path, b.start, b.code)));
    errors.dedup_by(|a, b| a.path == b.path && a.start == b.start && a.code == b.code);
    RootResult {
        lints,
        texts,
        error,
        errors,
        contexts,
        failures,
    }
}

/// The compiler's warnings in `files`, at the levels the settings give them, as findings named
/// by their code. Only the command line reports them: the language server already shows
/// the compiler's diagnostics.
fn compiler_warnings(
    compiler: &Compiler,
    files: &[jaic::source::FileId],
    configs: &Configs,
    canon: &CanonCache,
) -> Vec<Lint> {
    let path_of = |file| {
        let path = compiler.src.sources.get(file).path.as_file()?;
        Some(
            canon
                .get_or_insert_with(path, canonical)
                .to_string_lossy()
                .into_owned(),
        )
    };
    let mut lints = Vec::new();
    for warning in &*compiler.diag.warnings {
        if !files.contains(&warning.span.file) {
            continue;
        }
        let Some(path) = path_of(warning.span.file) else {
            continue;
        };
        let level = configs
            .at(Path::new(&path))
            .compiler_level(warning.code, Path::new(&path));
        if level == Level::Allow {
            continue;
        }
        let warning = warning.clone();
        let mut notes = Vec::new();
        let mut secondary = Vec::new();
        for (span, message) in warning.notes.iter().chain(warning.secondary_labels()) {
            if let Some(path) = (*span != Span::NONE).then(|| path_of(span.file)).flatten() {
                secondary.push(SecondaryLabel {
                    path,
                    text: compiler.src.sources.get(span.file).text.clone(),
                    start: span.start as usize,
                    end: span.end as usize,
                    message: message.clone(),
                });
            } else {
                notes.push(message.clone());
            }
        }
        // The suggestions come first, as the compiler prints them; the first one's edits are the
        // fix when they all stay in this file.
        let suggestions: Vec<_> = warning.suggestions().iter().filter(|s| s.listed).collect();
        let fix = suggestions.first().and_then(|s| {
            let edits = s
                .edits
                .iter()
                .map(|edit| match edit {
                    jaic::source::Edit::Replace {
                        span,
                        text,
                    } if span.file == warning.span.file => Some(Edit {
                        start: span.start as usize,
                        end: span.end as usize,
                        text: text.clone(),
                    }),
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()?;
            Some(Fix {
                title: s.title.clone(),
                edits,
                machine_applicable: s.applicability == Applicability::MachineApplicable,
            })
        });
        let help: Vec<String> = suggestions
            .iter()
            .map(|s| s.title.clone())
            .chain(warning.help.iter().cloned())
            .collect();
        lints.push(Lint {
            rule: warning.code.name(),
            level,
            path,
            start: warning.span.start as usize,
            end: warning.span.end as usize,
            message: warning.message,
            label: warning.label.map(String::from),
            help: (!help.is_empty()).then(|| help.join("\n")),
            notes,
            secondary,
            fix,
        });
    }
    lints
}
