//! The "add `#import`" quick fix for a compile error's unknown name.
//!
//! When the type checker's error (`jai-check`) is an unknown identifier that a standard-library
//! module declares, the compiler lists the imports that would declare it, best first
//! (`jaic::source::ImportSuggestion`, worked out with the error's "did you mean" help). Each one
//! becomes a `quickfix` that inserts the import into the document the error is in: after its
//! last top-level `#import`, else above its first line of code (below leading comments) with a
//! blank line after it. An import the document already has is not offered again. The first
//! action is preferred only when it is the only one.
//!
//! These fixes are not part of `source.fixAll.jailint`: that action applies jailint's
//! machine-applicable fixes, while an import is a choice about the program (several modules
//! may declare the name), made one error at a time.
use crate::analysis::Span;
use crate::document::DocumentUri;
use crate::lints::ActionContext;
use crate::model::{CodeAction, TextEdit};
use crate::session::Session;
use jaic::ast::{ImportSource, StmtKind};
use jaic::source::{FileId, ImportSuggestion};

/// The code of the type checker's diagnostic on the wire.
pub const CHECK_CODE: &str = "jai-check";

impl Session {
    /// One `quickfix` per import that would declare the unknown name of the compile error in
    /// `uri`, when the error touches `start..end` (byte offsets) or the client named it.
    pub(crate) fn import_actions(
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
        let Some(diagnostic) = self.check_diagnostics(uri) else {
            return Vec::new();
        };
        let Some((at, to, imports)) = self.with_semantic(uri, &doc.text, |analysis, path| {
            analysis.missing_imports(path)
        }) else {
            return Vec::new();
        };
        let named = context
            .diagnostics
            .iter()
            .any(|(code, range)| code == CHECK_CODE && *range == diagnostic.range);
        if !named && !(at <= end && start <= to) {
            return Vec::new();
        }
        // Preferred only when the compiler knows of one module that declares the name.
        let unambiguous = imports.len() == 1;
        planned(&doc.text, &imports)
            .into_iter()
            .filter_map(|(statement, offset, new_text)| {
                let range = self.range_of(uri, Span::new(offset, offset))?;
                Some(CodeAction {
                    title: format!("Add `{statement}`"),
                    kind: Some("quickfix"),
                    edit: Some((
                        uri.as_str().into(),
                        vec![TextEdit {
                            range,
                            new_text,
                        }],
                    )),
                    diagnostics: vec![diagnostic.clone()],
                    is_preferred: unambiguous,
                    ..CodeAction::default()
                })
            })
            .collect()
    }
}

/// Each of `imports` that `text` does not already have (at its top level), as the statement,
/// the byte offset to insert at and the text inserted.
fn planned(text: &str, imports: &[ImportSuggestion]) -> Vec<(String, usize, String)> {
    let Ok(file) = jaic::parser::parse_file(FileId(0), text) else {
        return Vec::new();
    };
    let existing: Vec<(&str, Option<&str>, usize)> = file
        .stmts
        .iter()
        .filter_map(|stmt| match &stmt.kind {
            StmtKind::Import(import) => match &import.source {
                ImportSource::Module(module) => Some((
                    &**module,
                    import.name.as_ref().map(|n| n.name.as_str()),
                    stmt.span.end as usize,
                )),
                _ => None,
            },
            _ => None,
        })
        .collect();
    let after_imports = existing.iter().map(|(_, _, end)| *end).max();
    imports
        .iter()
        .filter(|i| {
            !existing
                .iter()
                .any(|(module, name, _)| *module == i.module && *name == i.name.as_deref())
        })
        .map(|i| {
            let statement = i.statement();
            let (offset, new_text) = insertion(text, after_imports, &statement);
            (statement, offset, new_text)
        })
        .collect()
}

/// Where `statement` goes in `text` and the text inserted there: on the line after the last
/// import (which ends at `after_imports`), else above the first line of code with a blank line
/// after it.
pub(crate) fn insertion(
    text: &str,
    after_imports: Option<usize>,
    statement: &str,
) -> (usize, String) {
    if let Some(end) = after_imports {
        return match text[end.min(text.len())..].find('\n') {
            Some(newline) => (end + newline + 1, format!("{statement}\n")),
            None => (text.len(), format!("\n{statement}")),
        };
    }
    let first_code = jaic::lexer::lex(FileId(0), text)
        .ok()
        .and_then(|tokens| tokens.first().map(|t| t.span.start as usize));
    match first_code {
        Some(code) => {
            let line = text[..code].rfind('\n').map_or(0, |n| n + 1);
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
            insertion(text, Some(end), "#import \"File\";"),
            (
                text.find("\n\nmain").unwrap() + 1,
                "#import \"File\";\n".into()
            )
        );
    }

    #[test]
    fn goes_below_leading_comments() {
        let text = "// A program.\n/* more */\n\nmain :: () {}\n";
        assert_eq!(
            insertion(text, None, "#import \"Basic\";"),
            (text.find("main").unwrap(), "#import \"Basic\";\n\n".into())
        );
        assert_eq!(
            insertion("main :: () {}", None, "#import \"Basic\";"),
            (0, "#import \"Basic\";\n\n".into())
        );
    }
}
