//! The diagnostic every part of the toolchain shares: what the compiler, the linter, the language
//! server and the playground report. One [`Diagnostic`] holds a [`Code`] (what kind of problem),
//! a severity, a primary span with its message, and the notes, help and suggestions that go with
//! it. Renderers turn it into terminal text (`render`), LSP diagnostics, or the playground's JSON.
use crate::source::{Applicability, SourceMap, Span, Suggestion, Suggestions};
use std::fmt;

mod catalog;
mod levels;
pub use catalog::{Code, RETIRED};
pub use levels::{Level, Levels};

/// What every leniency warning says besides its own message.
pub const LENIENCY_NOTE: &str =
    "the official Jai compiler rejects this; Jacket accepts it but may stop in a future release";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        }
    }
}

/// What a diagnostic carries for the code that handles it, beyond its message: the editor's quick
/// fixes read it. Which variant a code uses is documented where the diagnostic is made.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Data {
    #[default]
    None,
    /// An unknown name: the index of the sema scope the lookup started from, for the "did you
    /// mean" help worked out when the error is rendered.
    Scope(u32),
    /// `Module.name` with no such export: the index of the sema module, for the help worked out
    /// when the error is rendered (the modules that declare the name).
    Module(u32),
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub code: Code,
    pub severity: Severity,
    pub span: Span,
    pub message: String,
    pub data: Data,
    /// Text under the primary span's carets (outside the plain layout). Boxed, as `fixes`
    /// is, to keep `Result<_, Diagnostic>` small.
    pub label: Option<Box<str>>,
    /// Related locations; a note whose span has no file (`Span::NONE`) prints as text only.
    /// Outside the plain layout, a located note is a label in the snippet.
    pub notes: Vec<(Span, String)>,
    /// `help:` lines printed last: how to fix the problem, when that is clear.
    pub help: Vec<String>,
    /// Changes that fix the problem, in a form tools can apply. Each prints as a `help:` line
    /// before the plain ones in `help`; editors offer each as a quick fix.
    pub suggestions: Option<Box<Suggestions>>,
}

impl Diagnostic {
    /// A diagnostic of `code`, an error or a warning as the code says.
    pub fn new(code: Code, span: Span, message: impl Into<String>) -> Self {
        Self {
            code,
            severity: code.severity(),
            span,
            message: message.into(),
            data: Data::None,
            label: None,
            notes: Vec::new(),
            help: Vec::new(),
            suggestions: None,
        }
    }

    /// Report the same problem under another code.
    #[must_use]
    pub fn with_code(mut self, code: Code) -> Self {
        self.code = code;
        self
    }

    #[must_use]
    pub fn with_data(mut self, data: Data) -> Self {
        self.data = data;
        self
    }

    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into().into_boxed_str());
        self
    }

    #[must_use]
    pub fn with_note(mut self, span: Span, message: impl Into<String>) -> Self {
        self.notes.push((span, message.into()));
        self
    }

    #[must_use]
    pub fn with_help(mut self, message: impl Into<String>) -> Self {
        self.help.push(message.into());
        self
    }

    /// The changes that fix the problem, in the order they were added.
    pub fn suggestions(&self) -> &[Suggestion] {
        self.suggestions.as_deref().map_or(&[], |s| &s.0)
    }

    /// A change that fixes the problem (see [`Suggestion`]).
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.get_or_insert_default().0.push(suggestion);
        self
    }

    /// A suggestion to replace `span` with `text`, titled for the `help:` line. A guess the
    /// author should check ([`Applicability::MaybeIncorrect`]).
    #[must_use]
    pub fn with_replacement(
        self,
        title: impl Into<String>,
        span: Span,
        text: impl Into<String>,
    ) -> Self {
        let text = text.into();
        let action = format!("Replace with `{text}`");
        self.with_suggestion(
            Suggestion::replace(title, span, text, Applicability::MaybeIncorrect)
                .with_action(action),
        )
    }

    /// The diagnostic as text, in the process-wide style (`crate::render::style`).
    pub fn render(&self, sources: &SourceMap) -> String {
        self.report(sources).render()
    }

    /// The diagnostic in the shared renderer's terms.
    pub fn report<'a>(&'a self, sources: &'a SourceMap) -> crate::render::Report<'a> {
        use crate::render::{Fix, Help, Label, Report};
        let label = |span: Span, message: Option<String>| {
            has_location(sources, span).then(|| {
                let file = sources.get(span.file);
                Label {
                    path: &file.path,
                    text: &file.text,
                    start: span.start as usize,
                    end: span.end as usize,
                    message,
                    lines: Some(file.lines()),
                }
            })
        };
        let mut report = Report::new(self.code, self.message.clone());
        report.severity = self.severity;
        report.primary = label(self.span, self.label.as_deref().map(String::from));
        for (span, note) in &self.notes {
            match label(*span, Some(note.clone())) {
                Some(l) => report.secondary.push(l),
                None => report.notes.push(note.clone()),
            }
        }
        for suggestion in self.suggestions().iter().filter(|s| s.listed) {
            let edits: Vec<(usize, usize, String)> = suggestion
                .replacements()
                .map(|(span, text)| (span.start as usize, span.end as usize, text.to_string()))
                .collect();
            // The changed line is shown for edits in one file, none of them an import.
            let file = suggestion.replacements().next().map(|(span, _)| span.file);
            let fix = file
                .filter(|&f| {
                    suggestion.imports().next().is_none()
                        && suggestion
                            .replacements()
                            .all(|(span, _)| span.file == f && has_location(sources, span))
                })
                .map(|f| Fix {
                    text: &sources.get(f).text,
                    edits,
                    lines: Some(sources.get(f).lines()),
                });
            report.help.push(Help {
                message: suggestion.title.clone(),
                fix,
            });
        }
        for help in &self.help {
            report.help.push(Help {
                message: help.clone(),
                fix: None,
            });
        }
        report
    }
}

/// Whether `span` names a place in a file (`Span::NONE` does not).
fn has_location(sources: &SourceMap, span: Span) -> bool {
    sources.try_get(span.file).is_some()
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[cfg(test)]
mod tests;
