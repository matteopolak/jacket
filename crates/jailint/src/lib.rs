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
pub mod render;
mod rules;
mod suppress;
pub mod syntax;
pub mod target;

pub use config::{Config, Level};
pub use rules::{RULES, RuleInfo};

use jaic::sema::Compiler;
use jaic::source::FileId;

/// One replacement in the linted file: bytes `start..end` become `text`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edit {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// A suggested change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fix {
    /// What the change does (`loop over the elements`).
    pub title: String,
    pub edits: Vec<Edit>,
    /// Safe to apply without review (`--fix` applies only these).
    pub machine_applicable: bool,
}

/// A finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lint {
    pub rule: &'static str,
    pub level: Level,
    pub path: String,
    /// Byte range into the file's text.
    pub start: usize,
    pub end: usize,
    pub message: String,
    /// Text shown under the highlighted range.
    pub label: Option<String>,
    /// Advice; with a fix, the changed code is shown after it.
    pub help: Option<String>,
    pub fix: Option<Fix>,
}

/// Lint `files` of the program `compiler` checked (with lint facts enabled). Lints at level
/// `allow`, and those an in-source `jailint: allow` covers, are left out. `complete` says the
/// compile finished without errors; when it did not, rules that need to have seen every use
/// (`unused_import` of an unnamed import) stay quiet.
pub fn lint_files(
    compiler: &mut Compiler,
    files: &[FileId],
    config: &Config,
    complete: bool,
) -> Vec<Lint> {
    let mut out = Vec::new();
    let mut module_facts = rules::unused_import::ModuleFacts::default();
    let run = rules::RunState::default();
    let Some(facts) = facts::Facts::collect(compiler, files) else {
        return out;
    };
    for &file in files {
        let path = compiler.src.sources.get(file).path.to_string();
        let text = compiler.src.sources.get(file).text.clone();
        // Lex once; the parser hands the tokens back for the rules and suppressions.
        let (tokens, comments, lex_errors) = jaic::lexer::lex_with_trivia(file, &text);
        if !lex_errors.is_empty() {
            continue;
        }
        let (ast, tokens) = jaic::parser::parse_tokens(file, &text, tokens);
        let Ok(ast) = ast else {
            continue;
        };
        // Override globs are matched against the canonical path, as `exclude` is.
        let at = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone().into());
        let at = at.as_path();
        let wanted: Vec<(&RuleInfo, Level)> = RULES
            .iter()
            .map(|r| (r, config.level_at(r, at)))
            .filter(|(_, level)| *level != Level::Allow)
            .collect();
        if wanted.is_empty() {
            continue;
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
        let mut cx = syntax::Cx::new(
            compiler, &facts, &text, &ast, &tokens, &comments, imports, &run,
        );
        cx.complete = complete;
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
        cx.doc_links = doc_links;
        for &(rule, level) in &wanted {
            let mut found = Vec::new();
            (rule.check)(&cx, &mut found);
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
                    fix: None,
                });
            }
        }
    }
    out.sort_by(|a, b| (&a.path, a.start, a.rule).cmp(&(&b.path, b.start, b.rule)));
    out.dedup_by(|a, b| a.path == b.path && a.start == b.start && a.rule == b.rule);
    out
}

/// What a rule reports, before the level and suppressions are applied.
#[derive(Debug, Default)]
pub(crate) struct Finding {
    pub start: usize,
    pub end: usize,
    pub message: String,
    pub label: Option<String>,
    pub help: Option<String>,
    pub fix: Option<Fix>,
}
