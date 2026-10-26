//! The `platform_leak` pass: compile portable roots once per target system.
use super::Options;
use super::asserts::{AssertFailure, failures_of};
use super::compile::{Compiled, compile_root};
use crate::config::Configs;
use crate::{Level, Lint};
use jaic::fxhash::HashSet;
use jaic::paths::CanonCache;
use jaic::project::Program;
use jaic::sema::{ImportEdge, TargetOs};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Mutex;

/// What one portable root's compile for one system found.
type LeakRun = (usize, TargetOs, Vec<ImportEdge>, Vec<AssertFailure>);

/// `platform_leak`: compile each root whose entry file is portable once per target system, with
/// failing `#assert`s recorded rather than stopping the compile, and report the imports (or
/// files) that portable code should not have, as [`rules::platform_leak`] explains.
pub(super) fn leak_pass(
    (roots, report): (&[Program], &BTreeSet<PathBuf>),
    options: &Options,
    configs: &Configs,
    canon: &CanonCache,
    lints: &mut Vec<Lint>,
    texts: &mut BTreeMap<String, Rc<str>>,
    failed_asserts: &mut Vec<AssertFailure>,
) {
    use crate::rules::platform_leak::{Leak, Probe, analyze};
    let config = &options.config;
    let Some(rule) = crate::RULES.iter().find(|r| r.name == "platform_leak") else {
        return;
    };
    if config.portability.portable.is_empty()
        || !report
            .iter()
            .any(|f| configs.at(f).level_at(rule, f) != Level::Allow)
    {
        return;
    }
    let probe = Probe::new(config);
    let mut jobs: Vec<(usize, TargetOs)> = Vec::new();
    for (i, root) in roots.iter().enumerate() {
        let entry = root.path();
        if configs.at(entry).level_at(rule, entry) == Level::Allow {
            continue;
        }
        for &os in config.portability.targets() {
            if probe.portable(entry, os) {
                jobs.push((i, os));
            }
        }
    }
    let next = std::sync::atomic::AtomicUsize::new(0);
    let collected: Mutex<Vec<LeakRun>> = Mutex::new(Vec::new());
    let workers = options.workers().min(jobs.len().max(1));
    jaic::pool::get().scope(|s| {
        for _ in 0..workers {
            let (jobs, next, collected) = (&jobs, &next, &collected);
            s.spawn(move |_| {
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(&(index, os)) = jobs.get(i) else {
                        break;
                    };
                    let root = &roots[index];
                    if options.verbose {
                        eprintln!(
                            "jailint: checking {} for {}",
                            root.path().display(),
                            jaic::target::os_name(os)
                        );
                    }
                    let Compiled {
                        mut compiler,
                        workspaces,
                        done,
                        error,
                        ..
                    } = compile_root(root, options, Some(os), None, canon);
                    if error.is_none() {
                        let _ = jaic::build::finish_all(
                            &workspaces,
                            &mut compiler.ct.interp.block_budget,
                        );
                    }
                    let mut compilers = std::mem::take(&mut *done.borrow_mut());
                    compilers.push((compiler, false));
                    let mut edges = Vec::new();
                    let mut failures = Vec::new();
                    for (compiler, _) in &compilers {
                        edges.extend(compiler.import_edges());
                        failures.extend(failures_of(compiler, root.path()));
                    }
                    drop(compilers);
                    collected
                        .lock()
                        .expect("no thread panicked holding the lock")
                        .push((index, os, edges, failures));
                }
            });
        }
    });
    let mut runs = collected.into_inner().unwrap_or_default();
    runs.sort_by(|a, b| (a.0, jaic::target::os_name(a.1)).cmp(&(b.0, jaic::target::os_name(b.1))));
    let mut read: BTreeMap<PathBuf, Option<Rc<str>>> = BTreeMap::new();
    let mut text_of = |path: &Path, texts: &BTreeMap<String, Rc<str>>| -> Option<Rc<str>> {
        read.entry(path.to_path_buf())
            .or_insert_with(|| {
                texts
                    .get(path.to_string_lossy().as_ref())
                    .cloned()
                    .or_else(|| std::fs::read_to_string(path).ok().map(Rc::from))
            })
            .clone()
    };
    let mut seen = HashSet::default();
    for (_, os, edges, failures) in runs {
        let asserts: Vec<_> = failures
            .iter()
            .filter(|f| f.about_os)
            .map(|f| jaic::sema::FailedAssert {
                path: f.path.clone(),
                start: f.start,
                end: f.end,
                message: f.message.clone(),
                about_os: true,
            })
            .collect();
        for leak in analyze(&probe, os, &edges, &asserts) {
            let (path, start, assert) = match &leak {
                Leak::Import {
                    from,
                    start,
                    assert,
                    ..
                } => (from, *start, assert),
                Leak::Own {
                    assert,
                } => (&assert.path, assert.start, assert),
            };
            if !report.contains(path)
                || !seen.insert((path.clone(), start, os as u8))
                || configs.at(path).level_at(rule, path) == Level::Allow
            {
                continue;
            }
            let (Some(text), Some(assert_text)) =
                (text_of(path, texts), text_of(&assert.path, texts))
            else {
                continue;
            };
            if allowed_by_comment(rule, &text, start) {
                continue;
            }
            let lint = leak_lint(&leak, os, &probe, configs, rule, &assert_text);
            texts
                .entry(path.to_string_lossy().into_owned())
                .or_insert(text);
            lints.push(lint);
        }
        failed_asserts.extend(failures);
    }
}

/// Whether a `// jailint: allow(platform_leak)` (or the category, or `all`) covers `offset`.
fn allowed_by_comment(rule: &crate::RuleInfo, text: &str, offset: usize) -> bool {
    let file = jaic::source::FileId(0);
    let (tokens, comments, errors) = jaic::lexer::lex_with_trivia(file, text);
    if !errors.is_empty() {
        return false;
    }
    let (ast, tokens) = jaic::parser::parse_tokens(file, text, tokens);
    let Ok(ast) = ast else {
        return false;
    };
    crate::suppress::Suppressions::new(text, &tokens, &comments, &ast).allows_rule(rule, offset)
}

/// The finding for one leak on `os`.
fn leak_lint(
    leak: &crate::rules::platform_leak::Leak,
    os: TargetOs,
    probe: &crate::rules::platform_leak::Probe,
    configs: &Configs,
    rule: &'static crate::RuleInfo,
    assert_text: &str,
) -> Lint {
    use crate::rules::platform_leak::{Leak, display_name, line_col, system_list};
    let os_name = jaic::target::os_name(os);
    let assert = match leak {
        Leak::Import {
            assert, ..
        }
        | Leak::Own {
            assert,
        } => assert,
    };
    let (line, column) = line_col(assert_text, assert.start);
    let shown = assert
        .path
        .strip_prefix(&configs.base().root)
        .unwrap_or(&assert.path);
    let reason = assert
        .message
        .strip_prefix("#assert failed: ")
        .unwrap_or(&assert.message);
    let notes = vec![format!(
        "the `#assert` at {}:{line}:{column} fails on {os_name}: {reason}",
        jaic::paths::slashed(shown)
    )];
    match leak {
        Leak::Import {
            from,
            start,
            end,
            to,
            ..
        } => Lint {
            rule: rule.name,
            level: configs.at(from).level_at(rule, from),
            path: from.to_string_lossy().into_owned(),
            start: *start,
            end: *end,
            message: format!("{} is not available on {os_name}", display_name(to)),
            label: Some(format!(
                "only builds on {}",
                system_list(&probe.systems(to))
            )),
            help: Some(
                "guard the import with an `#if OS ==` check, or list this file under `[lint.portability.platforms]`"
                    .into(),
            ),
            notes,
            fix: None,
        },
        Leak::Own {
            assert,
        } => Lint {
            rule: rule.name,
            level: configs.at(&assert.path).level_at(rule, &assert.path),
            path: assert.path.to_string_lossy().into_owned(),
            start: assert.start,
            end: assert.end,
            message: format!("{} is not available on {os_name}", display_name(&assert.path)),
            label: Some(format!("fails on {os_name}")),
            help: Some(
                "list the file under `[lint.portability.platforms]` or add `// jailint: platforms(...)` at its top"
                    .into(),
            ),
            notes: Vec::new(),
            fix: None,
        },
    }
}
