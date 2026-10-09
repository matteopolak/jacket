//! `jailint`: lints for Jai programs. Rules run on the program `jaic` has type-checked (see
//! `jaic::sema::ide`), so they know each expression's type and what each name refers to, and
//! fall back on the source text only to stay safe where the compiler did not look (code an
//! `#if` left out, bodies that failed to check).
//!
//! Callers compile with [`facts::enable`] set on the compiler, then call [`lint_files`]. The
//! `jailint` binary does this for the files on its command line ([`driver`]); the language
//! server does it for open documents.
pub mod config;
pub mod driver;
pub mod facts;
pub mod fix;
pub mod format_string;
pub mod render;
mod rules;
mod suppress;
pub mod syntax;

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
    let Some(facts) = facts::Facts::collect(compiler, files) else {
        return out;
    };
    for &file in files {
        let path = compiler.sources.get(file).path.clone();
        let text = compiler.sources.get(file).text.clone();
        // Lex once; the parser hands the tokens back for the rules and suppressions.
        let Ok(tokens) = jaic::lexer::lex(file, &text) else {
            continue;
        };
        let (ast, tokens) = jaic::parser::parse_tokens(file, &text, tokens);
        let Ok(ast) = ast else {
            continue;
        };
        let wanted: Vec<&RuleInfo> = RULES
            .iter()
            .filter(|r| config.level(r.name, r.default) != Level::Allow)
            .collect();
        if wanted.is_empty() {
            continue;
        }
        let imports = if wanted.iter().any(|r| r.name == "unused_import") {
            rules::unused_import::prepare(compiler, file, &ast, &tokens)
        } else {
            Vec::new()
        };
        let suppressions = suppress::Suppressions::new(&text, &tokens, &ast);
        let mut cx = syntax::Cx::new(compiler, &facts, file, &text, &ast, &tokens, imports);
        cx.complete = complete;
        for rule in wanted {
            let mut found = Vec::new();
            (rule.check)(&cx, &mut found);
            for f in found {
                if suppressions.allows(rule.name, f.start) {
                    continue;
                }
                out.push(Lint {
                    rule: rule.name,
                    level: config.level(rule.name, rule.default),
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
