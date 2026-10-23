//! Quick fixes from the suggestions of compile errors and warnings.
//!
//! Every type checker diagnostic can carry `jaic::source::Suggestion`s (a title, edits, how far
//! to trust it). Each one becomes a `quickfix` that applies its edits to the document the
//! diagnostic is in. The edits are replacements and inserts at spans, and `#import`s: an import
//! goes after the document's last top-level `#import`, else above its first line of code (below
//! leading comments) with a blank line after it. A suggestion whose import the document already
//! has is not offered. Only machine-applicable suggestions are `isPreferred`.
//!
//! These fixes are not part of `source.fixAll.jailint`: that action applies jailint's
//! machine-applicable fixes, while a suggestion of the compiler is a choice about the program,
//! made one diagnostic at a time.
use crate::analysis::Span;
use crate::document::DocumentUri;
use crate::lints::ActionContext;
use crate::model::{CodeAction, TextEdit};
use crate::session::Session;
use jaic::source::{Applicability, Edit, ImportSuggestion};

/// The code of the type checker's diagnostic on the wire.
pub const CHECK_CODE: &str = "check";

impl Session {
    /// One `quickfix` per suggestion of each compile error and warning in `uri` that touches
    /// `start..end` (byte offsets) or that the client named.
    pub(crate) fn suggestion_actions(
        &self,
        uri: &DocumentUri,
        start: usize,
        end: usize,
        context: &ActionContext,
    ) -> Vec<CodeAction> {
        if !context.wants("quickfix") {
            return Vec::new();
        }
        let Ok(doc) = self.document(uri) else {
            return Vec::new();
        };
        let mut actions = Vec::new();
        for finding in self.check_findings(uri) {
            if finding.suggestions.is_empty() {
                continue;
            }
            let named = context
                .diagnostics
                .iter()
                .any(|(code, range)| code == CHECK_CODE && *range == finding.diagnostic.range);
            if !named && (finding.start > end || start > finding.end) {
                continue;
            }
            actions.extend(self.actions_of(
                uri,
                &doc.text,
                &finding.diagnostic,
                &finding.suggestions,
            ));
        }
        // Lexical and syntax errors carry suggestions too (a missing `;`).
        if let Some(analysis) = self.analysis_for(uri) {
            for fix in &analysis.syntax_fixes {
                let named = context.diagnostics.iter().any(|(code, range)| {
                    code == fix.diagnostic.code.wire().0 && *range == fix.diagnostic.range
                });
                if !named && (fix.span.start > end || start > fix.span.end) {
                    continue;
                }
                if !analysis.diagnostics.contains(&fix.diagnostic) {
                    continue;
                }
                actions.extend(self.actions_of(uri, &doc.text, &fix.diagnostic, &fix.suggestions));
            }
        }
        actions
    }
}

impl Session {
    /// One `quickfix` per suggestion of `diagnostic` that applies to the document `text`.
    fn actions_of(
        &self,
        uri: &DocumentUri,
        text: &str,
        diagnostic: &crate::model::Diagnostic,
        suggestions: &[jaic::source::Suggestion],
    ) -> Vec<CodeAction> {
        let mut actions = Vec::new();
        for suggestion in suggestions {
            let mut edits = Vec::new();
            let mut applies = true;
            for edit in &suggestion.edits {
                let (at, new_text) = match edit {
                    Edit::Replace {
                        span,
                        text,
                    } => (
                        Span::new(span.start as usize, span.end as usize),
                        text.clone(),
                    ),
                    Edit::Import(import) => {
                        let placed = planned(text, std::slice::from_ref(import));
                        // Nothing to add when the document has the import already.
                        let Some((_, offset, new_text)) = placed.into_iter().next() else {
                            applies = false;
                            break;
                        };
                        (Span::new(offset, offset), new_text)
                    }
                };
                let Some(range) = self.range_of(uri, at) else {
                    applies = false;
                    break;
                };
                edits.push(TextEdit {
                    range,
                    new_text,
                });
            }
            if !applies || edits.is_empty() {
                continue;
            }
            actions.push(CodeAction {
                title: suggestion.action_title(),
                kind: Some("quickfix"),
                edit: Some((uri.as_str().into(), edits)),
                diagnostics: vec![diagnostic.clone()],
                is_preferred: suggestion.applicability == Applicability::MachineApplicable,
                ..CodeAction::default()
            });
        }
        actions
    }
}

/// Each of `imports` that `text` does not already have (at its top level), as the statement,
/// the byte offset to insert at and the text inserted.
fn planned(text: &str, imports: &[ImportSuggestion]) -> Vec<(String, usize, String)> {
    let scan = crate::exports::scan(text);
    let existing: Vec<_> = scan.imports.iter().filter(|i| !i.nested).collect();
    let after_imports = existing.iter().map(|i| i.end).max();
    imports
        .iter()
        .filter(|i| {
            !existing
                .iter()
                .any(|e| e.path == i.module && e.name.as_deref() == i.name.as_deref())
        })
        .map(|i| {
            let statement = i.statement();
            let (offset, new_text) = insertion(text, after_imports, scan.first_code, &statement);
            (statement, offset, new_text)
        })
        .collect()
}

/// Where `statement` goes in `text` and the text inserted there: on the line after the last
/// import (which ends at `after_imports`), else above the line of `first_code` (the first
/// token) with a blank line after it.
pub(crate) fn insertion(
    text: &str,
    after_imports: Option<usize>,
    first_code: Option<usize>,
    statement: &str,
) -> (usize, String) {
    if let Some(end) = after_imports {
        return match text[end.min(text.len())..].find('\n') {
            Some(newline) => (end + newline + 1, format!("{statement}\n")),
            None => (text.len(), format!("\n{statement}")),
        };
    }
    match first_code {
        Some(code) => {
            let line = jaic::source::line_start(text, code);
            (line, format!("{statement}\n\n"))
        }
        // Comments only (or nothing): after them.
        None if text.is_empty() || text.ends_with('\n') => (text.len(), format!("{statement}\n")),
        None => (text.len(), format!("\n{statement}\n")),
    }
}

#[cfg(test)]
mod tests {
    use super::{insertion, planned};
    use jaic::source::ImportSuggestion;

    fn import(module: &str, name: Option<&str>) -> ImportSuggestion {
        ImportSuggestion {
            module: module.into(),
            name: name.map(Into::into),
        }
    }

    #[test]
    fn leaves_out_imports_the_file_has() {
        let text = "#import \"Basic\";\nMath :: #import \"Math\";\nmain :: () {}\n";
        let after = text.find("main").unwrap();
        let planned = planned(
            text,
            &[
                import("Basic", None),
                import("Math", Some("Math")),
                import("Math", None),
                import("String", None),
            ],
        );
        assert_eq!(
            planned,
            [
                (
                    "#import \"Math\";".into(),
                    after,
                    "#import \"Math\";\n".into()
                ),
                (
                    "#import \"String\";".into(),
                    after,
                    "#import \"String\";\n".into()
                ),
            ]
        );
    }

    #[test]
    fn goes_after_the_last_import() {
        let text = "#import \"Basic\";\n#import \"Math\";\n\nmain :: () {}\n";
        let end = text.find("\"Math\";").unwrap() + "\"Math\";".len();
        assert_eq!(
            insertion(text, Some(end), None, "#import \"File\";"),
            (
                text.find("\n\nmain").unwrap() + 1,
                "#import \"File\";\n".into()
            )
        );
    }

    #[test]
    fn goes_below_leading_comments() {
        let text = "// A program.\n/* more */\n\nmain :: () {}\n";
        let first = crate::exports::scan(text).first_code;
        assert_eq!(
            insertion(text, None, first, "#import \"Basic\";"),
            (text.find("main").unwrap(), "#import \"Basic\";\n\n".into())
        );
        assert_eq!(
            insertion("main :: () {}", None, Some(0), "#import \"Basic\";"),
            (0, "#import \"Basic\";\n\n".into())
        );
    }
}
