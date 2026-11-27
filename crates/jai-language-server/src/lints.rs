//! jailint's findings as diagnostics, and its fixes as code actions.
//!
//! Lints come from the same type-checked compile that hover, inlay hints and code lenses use
//! (`semantic::Cache`, keyed by the open texts), so an edit costs one compile however many
//! features ask. A document is linted only when it parses. Its settings come from the nearest
//! `jacket.toml` above it: an open one (the client sent it with `didOpen`, which is how the
//! browser build gets one), else one on disk.
//!
//! Each machine-applicable fix is a preferred `quickfix` on its finding, carrying the finding
//! as its diagnostic and the rule as `data.rule`. `source.fixAll.jailint` applies all of them
//! that do not overlap, as `jailint --fix` would.
use crate::document::DocumentUri;
use crate::model::{CodeAction, Diagnostic, DiagnosticCode, DiagnosticSeverity, TextEdit};
use crate::position::Range;
use crate::session::Session;
use jailint::Lint;
use jailint::config::{Config, Level};
use std::path::Path;

/// The kind of the action that applies every safe fix in a document.
pub const FIX_ALL_KIND: &str = "source.fixAll.jailint";


/// A settings file (`jacket.toml`) rather than a Jai document.
pub(crate) fn is_config(uri: &DocumentUri) -> bool {
    jacket_config::is_config_file(Path::new(uri.path()))
}

/// What a code action request asks for, from its `context`.
#[derive(Clone, Debug, Default)]
pub struct ActionContext {
    /// Only actions of these kinds (or their sub-kinds); `None` for all.
    pub only: Option<Vec<String>>,
    /// Diagnostics the client shows at the request, jailint's and jaic's: `(code, range)` (a
    /// lint's code is its rule).
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
    /// Settings for the document at `path`: the nearest `jacket.toml` (open, else on disk), or
    /// the defaults; `None` when it excludes the document.
    pub(crate) fn lint_config(&self, path: &Path) -> Option<Config> {
        let config = self.find_lint_config(path).unwrap_or_default();
        (!config.excluded(path)).then_some(config)
    }

    fn find_lint_config(&self, path: &Path) -> Option<Config> {
        let mut index = self.index.borrow_mut();
        let loaded = crate::config::load(
            &crate::config::SessionFiles(self),
            &mut index,
            path.parent()?,
        )?;
        // `load` checked `[lint]`: it holds nothing invalid.
        Config::from_lint(&loaded.config.lint, loaded.dir()).ok()
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
        // The open documents under the same settings are linted in one pass with this one:
        // the program's lifetime summaries are then built once, not once per document.
        let together: Vec<std::path::PathBuf> = self
            .documents
            .iter()
            .filter(|(_, d)| crate::semantic::parse_error(&d.text).is_none())
            .map(|(u, _)| Path::new(u.path()))
            .filter(|p| self.lint_config(p).is_some_and(|c| c.root == config.root))
            .map(Path::to_path_buf)
            .collect();
        let group = [(config, together)];
        self.with_semantic(uri, &doc.text, |analysis, path| {
            if !analysis.warm_lints(&group) {
                return None;
            }
            analysis.lints(path, &group[0].0)
        })
        .unwrap_or_default()
    }

    fn lint_diagnostic(&self, uri: &DocumentUri, l: &Lint) -> Option<Diagnostic> {
        let range = self.range_of(uri, crate::analysis::Span::new(l.start, l.end))?;
        Some(self.lint_diagnostic_at(l, range))
    }

    pub(crate) fn lint_diagnostic_at(&self, l: &Lint, range: crate::Range) -> Diagnostic {
        let mut message = l.message.clone();
        if let Some(help) = &l.help {
            message = format!("{message}\n{help}");
        }
        Diagnostic {
            range,
            severity: match l.level {
                Level::Deny => DiagnosticSeverity::Error,
                _ => DiagnosticSeverity::Warning,
            },
            code: DiagnosticCode::Lint(l.rule),
            message,
            related: l
                .secondary
                .iter()
                .filter_map(|label| {
                    let at = self.location_in(&label.path, &label.text, label.start, label.end)?;
                    Some(crate::model::RelatedInformation {
                        uri: at.uri,
                        range: at.range,
                        message: label.message.clone(),
                    })
                })
                .collect(),
        }
    }

    /// The lints of `uri` as diagnostics: the rule is the code, `jailint` the source.
    pub(crate) fn lint_diagnostics(&self, uri: &DocumentUri) -> Vec<Diagnostic> {
        self.lints(uri)
            .iter()
            .filter_map(|l| self.lint_diagnostic(uri, l))
            .collect()
    }

    /// Drop the lints found and published so far: their settings changed.
    pub(crate) fn forget_lints(&self) {
        self.semantic.borrow_mut().forget_lints();
        self.published_lints.borrow_mut().clear();
    }

    /// `lint_diagnostics`, remembered with the text they are for, so the publication of the
    /// next edit's compile can show them until its own lints are found (`carried_lints`).
    /// Nothing is remembered of a pass that was stopped.
    pub(crate) fn published_lint_diagnostics(&self, uri: &DocumentUri) -> Vec<Diagnostic> {
        let lints = self.lints(uri);
        if let Ok((key, doc)) = self.document_entry(uri)
            && !jaic::interrupt::fired()
        {
            let found = (std::rc::Rc::from(doc.text.as_str()), lints.clone());
            self.published_lints.borrow_mut().insert(key.clone(), found);
        }
        lints
            .iter()
            .filter_map(|l| self.lint_diagnostic(uri, l))
            .collect()
    }

    /// The lints last published for `uri`, moved to where they are in its text now. A lint an
    /// edit touched since is left out: the lints of the text now will say whether it stays.
    pub(crate) fn carried_lints(&self, uri: &DocumentUri) -> Vec<Diagnostic> {
        let Ok((key, doc)) = self.document_entry(uri) else {
            return Vec::new();
        };
        let published = self.published_lints.borrow();
        let Some((text, lints)) = published.get(key) else {
            return Vec::new();
        };
        let shift = crate::semantic::Shift::between(text.as_bytes(), doc.text.as_bytes());
        let now: std::sync::Arc<str> = doc.text.as_str().into();
        let moved = |label: &mut jailint::SecondaryLabel| {
            if label.path != uri.path() {
                return Some(());
            }
            (label.start, label.end) = shift.to_new_range(label.start, label.end)?;
            label.text = now.clone();
            Some(())
        };
        lints
            .iter()
            .filter_map(|l| {
                let mut l = l.clone();
                (l.start, l.end) = shift.to_new_range(l.start, l.end)?;
                for label in &mut l.secondary {
                    moved(label)?;
                }
                self.lint_diagnostic(uri, &l)
            })
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
                if !named && (l.start > end || start > l.end) {
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
