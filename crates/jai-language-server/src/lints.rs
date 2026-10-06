//! jailint's findings as diagnostics, and its fixes as quick fixes.
//!
//! Lints come from the same type-checked compile that hover, inlay hints and code lenses use
//! (`semantic::Cache`, keyed by the open texts), so an edit costs one compile however many
//! features ask. A document is linted only when it parses; the settings come from the nearest
//! `jailint.toml` above it. `format_arg_count` is left out: format strings already have their
//! own diagnostics (`jai-format`), which work without type checking.
use crate::document::DocumentUri;
use crate::model::{CodeAction, Diagnostic, DiagnosticCode, DiagnosticSeverity, TextEdit};
use crate::session::Session;
use jailint::Lint;
use jailint::config::{Config, Level};
use std::path::Path;

/// Settings for the document at `path`: its `jailint.toml`, or the defaults.
pub(crate) fn config_for(path: &Path) -> Option<Config> {
    let dir = path.parent()?;
    let mut config = match Config::find(dir) {
        Some(file) => Config::load(&file).unwrap_or_default(),
        None => Config::default(),
    };
    if config.excluded(path) {
        return None;
    }
    config
        .levels
        .insert("format_arg_count".into(), Level::Allow);
    Some(config)
}

impl Session {
    /// Lints of the open document `uri`, from the cached compile of its program.
    pub(crate) fn lints(&self, uri: &DocumentUri) -> Vec<Lint> {
        let Ok(doc) = self.document(uri) else {
            return Vec::new();
        };
        if crate::semantic::parse_error(&doc.text).is_some() {
            return Vec::new();
        }
        let Some(config) = config_for(Path::new(uri.path())) else {
            return Vec::new();
        };
        self.with_semantic(uri, &doc.text, |analysis, path| {
            analysis.lints(path, &config)
        })
        .unwrap_or_default()
    }

    /// The lints of `uri` as diagnostics: the rule is the code, `jailint` the source.
    pub(crate) fn lint_diagnostics(&self, uri: &DocumentUri) -> Vec<Diagnostic> {
        self.lints(uri)
            .into_iter()
            .filter_map(|l| {
                let range = self.range_of(uri, crate::analysis::Span::new(l.start, l.end))?;
                let mut message = l.message;
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
            })
            .collect()
    }

    /// Quick fixes of the lints of `uri` that touch `start..end` (byte offsets).
    pub(crate) fn lint_fixes(
        &self,
        uri: &DocumentUri,
        start: usize,
        end: usize,
    ) -> Vec<CodeAction> {
        self.lints(uri)
            .into_iter()
            .filter(|l| l.start <= end && start <= l.end)
            .filter_map(|l| {
                let fix = l.fix?;
                let edits = fix
                    .edits
                    .iter()
                    .map(|e| {
                        Some(TextEdit {
                            range: self
                                .range_of(uri, crate::analysis::Span::new(e.start, e.end))?,
                            new_text: e.text.clone(),
                        })
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(CodeAction {
                    title: format!("{} ({})", fix.title, l.rule),
                    kind: Some("quickfix"),
                    edit: Some((uri.as_str().into(), edits)),
                    command: None,
                })
            })
            .collect()
    }
}
