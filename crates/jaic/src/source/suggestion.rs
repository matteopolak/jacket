//! Suggested changes a diagnostic carries, in a form the CLI shows and editors apply.
use super::Span;

/// How far a suggestion can be trusted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Applicability {
    /// Applying it leaves a program that means what the author wrote.
    MachineApplicable,
    /// It is a likely guess (a similar name, a cast): the author should look at the result.
    MaybeIncorrect,
}

/// One change of a suggestion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Edit {
    /// Replace the text of `span` with `text`. An empty span inserts, an empty `text` deletes.
    Replace { span: Span, text: String },
    /// Add this `#import` to the file the diagnostic is in. Where it goes (after the last
    /// import, else above the first code) is the applier's choice, so the edit has no span.
    Import(ImportSuggestion),
}

/// An `#import` that would bring an unknown name into scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportSuggestion {
    /// The module to import, as written in `#import "module"`.
    pub module: String,
    /// The name to bind the module to (`name :: #import "module";`) when the unknown name is
    /// the module itself, used as a namespace (`Math.sqrt`); `None` imports its names.
    pub name: Option<String>,
}

impl ImportSuggestion {
    /// The import statement, as it would be written: `#import "Basic";`.
    pub fn statement(&self) -> String {
        match &self.name {
            Some(name) => format!("{name} :: #import \"{}\";", self.module),
            None => format!("#import \"{}\";", self.module),
        }
    }
}

/// The suggestions of a diagnostic, boxed in it to keep `Result<_, Diagnostic>` small.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Suggestions(pub(super) Vec<Suggestion>);

/// A way to fix a diagnostic: a title for the `help:` line and the edits that carry it out
/// together (possibly in several places).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suggestion {
    /// What the change does, as the `help:` line says it: "a similar name exists: `count`".
    pub title: String,
    pub edits: Vec<Edit>,
    pub applicability: Applicability,
    /// What an editor's quick-fix menu says instead of `title` ("Replace with `count`"):
    /// `title` with its first letter capitalised when `None`.
    pub action: Option<String>,
    /// Whether the CLI prints the `title` as a `help:` line. Alternatives that an earlier
    /// help line already names (the other modules that declare a name) are only for editors.
    pub listed: bool,
}

impl Suggestion {
    /// What the quick fix of this suggestion is called in an editor.
    pub fn action_title(&self) -> String {
        if let Some(action) = &self.action {
            return action.clone();
        }
        let mut chars = self.title.chars();
        chars
            .next()
            .map(|c| c.to_uppercase().chain(chars).collect())
            .unwrap_or_default()
    }

    /// Name the editor's quick fix (see [`Suggestion::action`]).
    #[must_use]
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Trust the suggestion as `applicability` says.
    #[must_use]
    pub fn with_applicability(mut self, applicability: Applicability) -> Self {
        self.applicability = applicability;
        self
    }

    /// Keep the CLI from printing this suggestion (see [`Suggestion::listed`]).
    #[must_use]
    pub fn unlisted(mut self) -> Self {
        self.listed = false;
        self
    }

    /// A suggestion that replaces `span` with `text`.
    pub fn replace(
        title: impl Into<String>,
        span: Span,
        text: impl Into<String>,
        applicability: Applicability,
    ) -> Self {
        Self::with_edits(
            title,
            vec![Edit::Replace {
                span,
                text: text.into(),
            }],
            applicability,
        )
    }

    /// A suggestion that inserts `text` at the start of `at`.
    pub fn insert(
        title: impl Into<String>,
        at: Span,
        text: impl Into<String>,
        applicability: Applicability,
    ) -> Self {
        let point = Span {
            end: at.start,
            ..at
        };
        Self::replace(title, point, text, applicability)
    }

    /// A suggestion made of several `edits`.
    pub fn with_edits(
        title: impl Into<String>,
        edits: Vec<Edit>,
        applicability: Applicability,
    ) -> Self {
        Self {
            title: title.into(),
            edits,
            applicability,
            action: None,
            listed: true,
        }
    }

    /// A suggestion to add `import` to the file. It is never machine-applicable: the module
    /// is a choice about the program.
    pub fn import(title: impl Into<String>, import: ImportSuggestion) -> Self {
        Self::with_edits(
            title,
            vec![Edit::Import(import)],
            Applicability::MaybeIncorrect,
        )
    }

    /// The replacements of the suggestion as (span, text), without its imports.
    pub fn replacements(&self) -> impl Iterator<Item = (Span, &str)> {
        self.edits.iter().filter_map(|e| match e {
            Edit::Replace {
                span,
                text,
            } => Some((*span, text.as_str())),
            Edit::Import(_) => None,
        })
    }

    /// The imports of the suggestion.
    pub fn imports(&self) -> impl Iterator<Item = &ImportSuggestion> {
        self.edits.iter().filter_map(|e| match e {
            Edit::Import(i) => Some(i),
            Edit::Replace {
                ..
            } => None,
        })
    }
}
