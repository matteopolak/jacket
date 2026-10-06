//! jailint's findings as diagnostics, and its fixes as code actions.
//!
//! Lints come from the same type-checked compile that hover, inlay hints and code lenses use
//! (`semantic::Cache`, keyed by the open texts), so an edit costs one compile however many
//! features ask. A document is linted only when it parses. Its settings come from the nearest
//! `jailint.toml` above it: an open one (the client sent it with `didOpen`, which is how the
//! browser build gets one), else one on disk. `format_arg_count` is left out: format strings
//! already have their own diagnostics (`jai-format`), which work without type checking.
//!
//! Each machine-applicable fix is a preferred `quickfix` on its finding, carrying the finding
//! as its diagnostic and the rule as `data.rule`. `source.fixAll.jailint` applies all of them
//! that do not overlap, as `jailint --fix` would.
use crate::document::DocumentUri;
use crate::model::{CodeAction, Diagnostic, DiagnosticCode, DiagnosticSeverity, TextEdit};
use crate::position::Range;
use crate::session::Session;
use jailint::Lint;
use jailint::config::{Config, FILE_NAME, Level};
use std::path::Path;

/// The kind of the action that applies every safe fix in a document.
pub const FIX_ALL_KIND: &str = "source.fixAll.jailint";

/// Where each rule is documented.
pub fn rule_url(rule: &str) -> String {
    format!("https://github.com/matteopolak/jai/blob/main/docs/tools/jailint.md#{rule}")
}

/// The document is a jailint settings file rather than Jai source.
pub(crate) fn is_config(uri: &DocumentUri) -> bool {
    Path::new(uri.path())
        .file_name()
        .is_some_and(|n| n == FILE_NAME)
}

/// What a code action request asks for, from its `context`.
#[derive(Clone, Debug, Default)]
pub struct ActionContext {
    /// Only actions of these kinds (or their sub-kinds); `None` for all.
    pub only: Option<Vec<String>>,
    /// Lint diagnostics the client shows at the request: `(rule, range)`.
    pub diagnostics: Vec<(String, Range)>,
}

impl ActionContext {
    /// An action of `kind` was asked for.
    pub fn wants(&self, kind: &str) -> bool {
        self.only.as_ref().is_none_or(|only| {
            only.iter().any(|k| {
                kind == k
                    || kind
                        .strip_prefix(k.as_str())
                        .is_some_and(|r| r.starts_with('.'))
            })
        })
    }
}

impl Session {
    /// Settings for the document at `path`: the nearest `jailint.toml` (open, else on disk), or
    /// the defaults; `None` when it excludes the document.
    pub(crate) fn lint_config(&self, path: &Path) -> Option<Config> {
        let mut config = self.find_lint_config(path).unwrap_or_default();
        if config.excluded(path) {
            return None;
        }
        config
            .levels
            .insert("format_arg_count".into(), Level::Allow);
        Some(config)
    }

    fn find_lint_config(&self, path: &Path) -> Option<Config> {
        let mut dir = path.parent();
        while let Some(d) = dir {
            let file = d.join(FILE_NAME);
            if let Some(open) = self
                .lint_configs
                .iter()
                .find(|(uri, _)| Path::new(uri.path()) == file)
            {
                // A settings file being edited may not parse yet: use the defaults meanwhile.
                return Some(Config::parse(&open.1.text, d).unwrap_or_default());
            }
            if file.is_file() {
                return Some(Config::load(&file).unwrap_or_default());
            }
            dir = d.parent();
        }
        None
    }

    /// Lints of the open document `uri`, from the cached compile of its program.
    pub(crate) fn lints(&self, uri: &DocumentUri) -> Vec<Lint> {
        let Ok(doc) = self.document(uri) else {
            return Vec::new();
        };
        if crate::semantic::parse_error(&doc.text).is_some() {
            return Vec::new();
        }
        let Some(config) = self.lint_config(Path::new(uri.path())) else {
            return Vec::new();
        };
        self.with_semantic(uri, &doc.text, |analysis, path| {
            analysis.lints(path, &config)
        })
        .unwrap_or_default()
    }

    fn lint_diagnostic(&self, uri: &DocumentUri, l: &Lint) -> Option<Diagnostic> {
        let range = self.range_of(uri, crate::analysis::Span::new(l.start, l.end))?;
        let mut message = l.message.clone();
        if let Some(help) = &l.help {
            message = format!("{message}\n{help}");
        }
        Some(Diagnostic {
            range,
            severity: match l.level {
                Level::Deny => DiagnosticSeverity::Error,
                _ => DiagnosticSeverity::Warning,
            },
            code: DiagnosticCode::Lint(l.rule),
            message,
        })
    }

    /// The lints of `uri` as diagnostics: the rule is the code, `jailint` the source.
    pub(crate) fn lint_diagnostics(&self, uri: &DocumentUri) -> Vec<Diagnostic> {
        self.lints(uri)
            .iter()
            .filter_map(|l| self.lint_diagnostic(uri, l))
            .collect()
    }

    fn edits(&self, uri: &DocumentUri, fix: &jailint::Fix) -> Option<Vec<TextEdit>> {
        fix.edits
            .iter()
            .map(|e| {
                Some(TextEdit {
                    range: self.range_of(uri, crate::analysis::Span::new(e.start, e.end))?,
                    new_text: e.text.clone(),
                })
            })
            .collect()
    }

    /// Fixes of the lints of `uri` that touch `start..end` (byte offsets) or that the client
    /// named in `context`, and the action that applies every safe fix.
    pub(crate) fn lint_actions(
        &self,
        uri: &DocumentUri,
        start: usize,
        end: usize,
        context: &ActionContext,
    ) -> Vec<CodeAction> {
        let lints = self.lints(uri);
        let mut actions = Vec::new();
        if context.wants("quickfix") {
            for l in &lints {
                let Some(fix) = &l.fix else {
                    continue;
                };
                let Some(diagnostic) = self.lint_diagnostic(uri, l) else {
                    continue;
                };
                let named = context
                    .diagnostics
                    .iter()
                    .any(|(rule, range)| rule == l.rule && *range == diagnostic.range);
                if !named && !(l.start <= end && start <= l.end) {
                    continue;
                }
                let Some(edits) = self.edits(uri, fix) else {
                    continue;
                };
                actions.push(CodeAction {
                    title: sentence(&fix.title),
                    kind: Some("quickfix"),
                    edit: Some((uri.as_str().into(), edits)),
                    diagnostics: vec![diagnostic],
                    is_preferred: fix.machine_applicable,
                    rule: Some(l.rule),
                    ..CodeAction::default()
                });
            }
        }
        if context.wants(FIX_ALL_KIND)
            && let Some(all) = self.fix_all(uri, &lints)
        {
            actions.push(all);
        }
        actions
    }

    /// Every machine-applicable fix of `lints` that does not overlap an earlier one, as one
    /// edit (the same choice `jailint --fix` makes).
    fn fix_all(&self, uri: &DocumentUri, lints: &[Lint]) -> Option<CodeAction> {
        let safe: Vec<&Lint> = lints
            .iter()
            .filter(|l| l.fix.as_ref().is_some_and(|f| f.machine_applicable))
            .collect();
        let chosen = jailint::fix::choose(&safe);
        let mut edits = Vec::new();
        let mut diagnostics = Vec::new();
        for l in chosen {
            let fix = l.fix.as_ref()?;
            edits.extend(self.edits(uri, fix)?);
            diagnostics.extend(self.lint_diagnostic(uri, l));
        }
        if edits.is_empty() {
            return None;
        }
        let n = diagnostics.len();
        Some(CodeAction {
            title: if n == 1 {
                "Fix 1 lint problem".into()
            } else {
                format!("Fix {n} lint problems")
            },
            kind: Some(FIX_ALL_KIND),
            edit: Some((uri.as_str().into(), edits)),
            diagnostics,
            ..CodeAction::default()
        })
    }
}

/// `remove the unused variable` → `Remove the unused variable`.
fn sentence(title: &str) -> String {
    let mut chars = title.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
