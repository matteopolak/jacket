//! `jailint`: lints for Jai programs. Rules run on the program `jaic` has type-checked (see
//! `jaic::ide`), so they know each expression's type and what each name refers to. Where
//! the compiler did not look (code an `#if` left out, bodies that failed to check) a rule reads
//! the source text instead, and stays conservative about what it reports.
//!
//! Callers compile with [`facts::enable`] set on the compiler, then call [`lint_files`]. The
//! `jailint` binary does this for the files on its command line ([`driver`]); the language
//! server does it for open documents.
#![forbid(unsafe_code)]

pub mod config;
pub mod driver;
pub mod facts;
pub mod fix;
pub mod migrate;
pub mod render;
mod rules;
mod suppress;
pub mod syntax;

pub use config::{Config, Level};

/// The page of `rule` on the documentation site. Findings and editor diagnostics link to it.
pub fn rule_url(rule: &str) -> String {
    format!("https://jacket.matteopolak.com/lint-rules/{rule}")
}

use std::sync::Arc;

use jaic::sema::Compiler;
use jaic::source::{FileId, Span};
pub use rules::unused_note::{LINT_NOTES, TOOL_NOTES};
pub use rules::{RULES, RuleInfo};

/// One replacement in the linted file: bytes `start..end` become `text`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edit {
    /// Byte range into the file's text that `text` replaces.
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// A suggested change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fix {
    /// What the change does (`loop over the elements`).
    pub title: String,
    /// The replacements, in the one file the finding is in.
    pub edits: Vec<Edit>,
    /// Safe to apply without review (`--fix` applies only these).
    pub machine_applicable: bool,
}

/// A labelled source location retained after its compiler is dropped. Text shares the
/// compiler's source allocation, so multiple labels never copy an entire file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecondaryLabel {
    /// The file the label is in; it may differ from the finding's file.
    pub path: String,
    /// The whole text of that file.
    pub text: Arc<str>,
    /// Byte range into `text`.
    pub start: usize,
    pub end: usize,
    /// The text shown at the range.
    pub message: String,
}

/// A finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lint {
    /// The name of the rule that reported it.
    pub rule: &'static str,
    /// The level the rule has for this file, after settings and flags.
    pub level: Level,
    pub path: String,
    /// Byte range into the file's text.
    pub start: usize,
    pub end: usize,
    pub message: String,
    /// Text shown under the highlighted range.
    pub label: Option<String>,
    /// Advice, one `help:` line per line; with a fix, the changed code is shown after the first.
    pub help: Option<String>,
    /// Further facts, printed as `note:` lines after the advice.
    pub notes: Vec<String>,
    /// Other places that explain the finding.
    pub secondary: Vec<SecondaryLabel>,
    pub fix: Option<Fix>,
}

/// How many times this thread has built a program's lifetime summaries, the costly shared part
/// of a lint pass. A caller that lints several files of one program in one pass builds them
/// once; a test compares two counts to check that it did.
///
/// ```
/// let before = jailint::summaries_built();
/// assert!(jailint::summaries_built() >= before);
/// ```
pub fn summaries_built() -> u64 {
    rules::lifetime_summary::built()
}

/// Lint `files` of the program `compiler` checked (with lint facts enabled). Lints at level
/// `allow`, and those an in-source `jailint: allow` covers, are left out. `complete` says the
/// compile finished without errors; when it did not, rules that need to have seen every use
/// (`unused_import` of an unnamed import) stay quiet.
/// When `jaic::interrupt` is armed, polls between files, rules and lifetime bodies can stop
/// this pass. The caller must discard the result when `disarm` reports an interruption.
pub fn lint_files(
    compiler: &mut Compiler,
    files: &[FileId],
    configs: &config::Configs,
    complete: bool,
) -> Vec<Lint> {
    lint_files_with_entry(compiler, files, configs, complete, None, None)
}

pub(crate) fn lint_files_with_entry(
    compiler: &mut Compiler,
    files: &[FileId],
    configs: &config::Configs,
    complete: bool,
    entry: Option<FileId>,
    reach: Option<rules::unreachable_declaration::Reach>,
) -> Vec<Lint> {
    let mut out = Vec::new();
    let done = lint_each_with(
        compiler,
        files,
        configs,
        (complete, entry, reach),
        &|_| true,
        &mut |_, found| out.extend(found),
    );

    if !done {
        return Vec::new();
    }

    out.sort_by(|a, b| (&a.path, a.start, a.rule).cmp(&(&b.path, b.start, b.rule)));
    out.dedup_by(|a, b| a.path == b.path && a.start == b.start && a.rule == b.rule);
    out
}

/// [`lint_files`], one file at a time: `sink` receives each file's lints (sorted) as soon as
/// that file is finished, and only the files `pending` accepts are linted; the others still
/// count as part of the program the rules see. Returns `false` when `jaic::interrupt` stopped
/// the pass: every file handed to `sink` before then is complete, so a caller can keep those
/// and lint the rest later.
pub fn lint_each(
    compiler: &mut Compiler,
    files: &[FileId],
    configs: &config::Configs,
    complete: bool,
    entry: Option<FileId>,
    pending: &dyn Fn(FileId) -> bool,
    sink: &mut dyn FnMut(FileId, Vec<Lint>),
) -> bool {
    lint_each_with(
        compiler,
        files,
        configs,
        (complete, entry, None),
        pending,
        sink,
    )
}

/// [`lint_each`] with the program's unreachable declarations already worked out, when the
/// caller needed them for itself.
pub(crate) fn lint_each_with(
    compiler: &mut Compiler,
    files: &[FileId],
    configs: &config::Configs,
    (complete, entry, reach): (
        bool,
        Option<FileId>,
        Option<rules::unreachable_declaration::Reach>,
    ),
    pending: &dyn Fn(FileId) -> bool,
    sink: &mut dyn FnMut(FileId, Vec<Lint>),
) -> bool {
    let mut module_facts = rules::unused_import::ModuleFacts::default();
    let mut run = rules::RunState::with_lifetime_entry(entry.and_then(|id| {
        compiler
            .src
            .files
            .iter()
            .find(|f| f.id == id)
            .map(|f| f.module)
    }));

    run.lifetime_files.extend(files.iter().copied());

    if let Some(reach) = reach {
        let _ = run.reach.set(Some(reach));
    }

    let time_lifetimes = jacket_config::env::LIFETIME_STATS.is_set();

    if !compiler.ed.facts().is_some_and(|ide| ide.lint) {
        for &file in files.iter().filter(|&&f| pending(f)) {
            sink(file, Vec::new());
        }

        return true;
    }

    for &file in files.iter().filter(|&&f| pending(f)) {
        if jaic::interrupt::should_stop() {
            return false;
        }

        let mut out = Vec::new();
        let path = compiler.src.sources.get(file).path.to_string();
        let text = compiler.src.sources.get(file).text.clone();
        // Lex once; the parser hands the tokens back for the rules and suppressions.
        let (tokens, comments, lex_errors) = jaic::lexer::lex_with_trivia(file, &text);

        if !lex_errors.is_empty() {
            sink(file, Vec::new());
            continue;
        }

        let (ast, tokens) = jaic::parser::parse_tokens(file, &text, tokens);
        let Ok(ast) = ast else {
            sink(file, Vec::new());
            continue;
        };
        // Override globs are matched against the canonical path, as `exclude` is.
        let at = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone().into());
        let at = at.as_path();
        let config = &*configs.at(at);
        let wanted: Vec<(&RuleInfo, Level)> = RULES
            .iter()
            .map(|r| (r, config.level_at(r, at)))
            .filter(|(_, level)| *level != Level::Allow)
            .collect();

        if wanted.is_empty() {
            sink(file, Vec::new());
            continue;
        }

        if wanted
            .iter()
            .any(|(r, _)| r.name == rules::unreachable_declaration::INFO.name)
        {
            run.reach
                .get_or_init(|| rules::unreachable_declaration::analyze(compiler, complete));
        }

        let imports = if wanted.iter().any(|(r, _)| r.name == "unused_import") {
            rules::unused_import::prepare(compiler, &mut module_facts, file, &ast, &tokens)
        } else {
            Vec::new()
        };
        let doc_links = if complete && wanted.iter().any(|(r, _)| r.name == "broken_doc_link") {
            rules::broken_doc_link::prepare(compiler, file, &text)
        } else {
            Vec::new()
        };
        let suppressions = suppress::Suppressions::new(&text, &tokens, &comments, &ast);
        let mut cx = syntax::Cx::new(compiler, &text, &ast, &tokens, &comments, imports, &run);

        cx.complete = complete;
        cx.lifetime_mode = config
            .flag_lifetime_mode
            .or(config.lifetime_mode)
            .unwrap_or_default();
        cx.max_nesting = config.max_nesting();
        cx.magic_numbers = config.magic_numbers();
        cx.max_complexity = config
            .max_complexity
            .unwrap_or(config::DEFAULT_MAX_COMPLEXITY);
        cx.max_arguments = config
            .max_arguments
            .unwrap_or(config::DEFAULT_MAX_ARGUMENTS);
        cx.max_bools = config.max_bools.unwrap_or(config::DEFAULT_MAX_BOOLS);
        cx.max_stack_array_bytes = config
            .max_stack_array_bytes
            .unwrap_or(config::DEFAULT_MAX_STACK_ARRAY_BYTES);
        cx.max_frame_bytes = config
            .max_frame_bytes
            .unwrap_or(config::DEFAULT_MAX_FRAME_BYTES);
        cx.doc_links = doc_links;

        for &(rule, level) in &wanted {
            if jaic::interrupt::should_stop() {
                return false;
            }

            let mut found = Vec::new();
            let started = (time_lifetimes
                && matches!(
                    rule.name,
                    "pointer_lifetime"
                        | "pointer_into_resizable"
                        | "remove_in_for"
                        | "use_after_free"
                        | "double_free"
                        | "temp_storage_escape"
                        | "returned_local_address"
                ))
            .then(std::time::Instant::now);

            (rule.check)(&cx, &mut found);

            if let Some(started) = started {
                eprintln!(
                    "lifetime-rule-time file={path} rule={} wall_ms={:.3}",
                    rule.name,
                    started.elapsed().as_secs_f64() * 1000.0
                );
            }

            for f in found {
                if suppressions.allows_rule(rule, f.start) {
                    continue;
                }

                out.push(Lint {
                    rule: rule.name,
                    level,
                    path: path.clone(),
                    start: f.start,
                    end: f.end,
                    message: f.message,
                    label: f.label,
                    help: f.help,
                    notes: Vec::new(),
                    secondary: f
                        .secondary
                        .iter()
                        .filter_map(|(span, message)| {
                            // Generated inserts are explained at their writable source site.
                            let mut span = *span;

                            while let Some(site) = compiler.src.insert_sites.get(&span.file) {
                                span = *site;
                            }

                            let source = compiler.src.sources.try_get(span.file)?;

                            Some(SecondaryLabel {
                                path: source.path.to_string(),
                                text: source.text.clone(),
                                start: span.start as usize,
                                end: span.end as usize,
                                message: message.clone(),
                            })
                        })
                        .collect(),
                    fix: f.fix,
                });
            }
        }

        // After every other rule has had its say: what no finding used is stale.
        if complete
            && let Some(&(rule, level)) =
                wanted.iter().find(|(r, _)| r.name == "unused_suppression")
        {
            let ran: Vec<&str> = wanted.iter().map(|(r, _)| r.name).collect();

            for stale in suppressions.unused(&|name| suppress::could_judge(name, &ran)) {
                if suppressions.allows_rule(rule, stale.start) {
                    continue;
                }

                out.push(Lint {
                    rule: rule.name,
                    level,
                    path: path.clone(),
                    start: stale.start,
                    end: stale.end,
                    message: format!("`{}` allows nothing here", stale.name),
                    label: None,
                    help: Some("nothing it covers is reported any more; remove it".into()),
                    notes: Vec::new(),
                    secondary: Vec::new(),
                    fix: None,
                });
            }
        }

        // A rule that was stopped reported only part of its findings.
        if jaic::interrupt::fired() {
            return false;
        }

        out.sort_by(|a, b| (&a.path, a.start, a.rule).cmp(&(&b.path, b.start, b.rule)));
        out.dedup_by(|a, b| a.path == b.path && a.start == b.start && a.rule == b.rule);
        sink(file, out);
    }

    true
}

/// What a rule reports, before the level and suppressions are applied.
#[derive(Clone, Debug, Default)]
pub(crate) struct Finding {
    pub start: usize,
    pub end: usize,
    pub message: String,
    pub label: Option<String>,
    pub help: Option<String>,
    pub secondary: Vec<(Span, String)>,
    pub fix: Option<Fix>,
}
