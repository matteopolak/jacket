//! Code lenses that run a test: "Run test" and "Debug test" over each `@test` procedure.
//!
//! The procedures come from the parse the analysis already made, with the same note rules as
//! the Testing module's Discovery plugin: a `@test` note makes a test, and `@property`,
//! `@compile_time` and `@cases` say what kind. The lens commands are the client's
//! (`jai.runTest`, `jai.debugTest`); their argument names the test and the `jacket test`
//! flags that select exactly it.
use crate::analysis::Span;
use crate::session::Session;
use crate::{CodeLens, Command, DocumentUri, Error};
use jaic::parser::ide_facts::Procedure;
use serde_json::json;
use std::path::{Path, PathBuf};

pub(crate) const RUN_COMMAND: &str = "jai.runTest";
pub(crate) const DEBUG_COMMAND: &str = "jai.debugTest";

/// What a test procedure's notes make of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TestKind {
    Test,
    Property,
    CompileTime,
    Cases,
}

impl TestKind {
    /// The label the client shows and the lens argument carries.
    fn name(self) -> &'static str {
        match self {
            Self::Test => "unit",
            Self::Property => "property",
            Self::CompileTime => "compile_time",
            Self::Cases => "cases",
        }
    }
}

/// A procedure with a `@test` note.
#[derive(Clone, Debug)]
pub(crate) struct TestProcedure {
    pub name: String,
    pub name_span: Span,
    pub kind: TestKind,
}

impl TestProcedure {
    /// `procedure` as a test, when it has the `@test` note (a `@bench` is not run by `jacket test`).
    pub fn of(procedure: &Procedure<'_>) -> Option<Self> {
        let notes = || {
            procedure
                .header
                .notes
                .iter()
                .chain(procedure.notes)
                .map(|note| note.text.trim())
        };
        let kind_of = |text: &str| match text {
            "property" => Some(TestKind::Property),
            "compile_time" => Some(TestKind::CompileTime),
            _ if text.starts_with("property(") => Some(TestKind::Property),
            _ if text.starts_with("cases(") => Some(TestKind::Cases),
            _ => None,
        };
        // The first of `@test` and `@bench` decides, as Discovery reads them.
        if notes().find(|text| matches!(*text, "test" | "bench")) != Some("test") {
            return None;
        }
        Some(Self {
            name: procedure.name.name.as_str().to_owned(),
            name_span: Span::new(
                procedure.name.span.start as usize,
                procedure.name.span.end as usize,
            ),
            kind: notes().find_map(kind_of).unwrap_or(TestKind::Test),
        })
    }
}

/// The folder a test file's root is: the parent of the nearest `tests` folder above it.
fn test_root(file: &Path) -> Option<PathBuf> {
    file.ancestors()
        .skip(1)
        .find(|dir| dir.file_name().is_some_and(|name| name == "tests"))
        .and_then(Path::parent)
        .map(Path::to_path_buf)
}

/// The path with `/` separators, as the filter expressions' `file(..)` reads it.
fn slashed(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// The `jacket test` flags that select the test `name` of `file` and no other.
fn filter(name: &str, file: &str) -> Vec<String> {
    // Characters of a filter expression or a glob; a path holding one is left out of `-E`.
    if file.contains(['*', '?', '[', ']', '{', '}', '(', ')', '\\']) {
        return vec!["--exact".into(), name.into()];
    }
    vec!["-E".into(), format!("test(={name}) and file({file})")]
}

impl Session {
    /// Turn the "Run test" and "Debug test" lenses on or off (`tests.codeLens`).
    pub fn set_test_lenses(&mut self, on: bool) {
        self.test_lenses = on;
    }

    pub fn test_lenses_enabled(&self) -> bool {
        self.test_lenses
    }

    /// A "Run test" and a "Debug test" lens over each test procedure of the document.
    pub(crate) fn test_code_lenses(&self, uri: &DocumentUri) -> Result<Vec<CodeLens>, Error> {
        self.document(uri)?;
        if !self.test_lenses {
            return Ok(Vec::new());
        }
        let Some(analysis) = self.analysis_for(uri) else {
            return Ok(Vec::new());
        };
        let path = Path::new(uri.path());
        let root = test_root(path);
        let project = path
            .parent()
            .and_then(|dir| jacket_config::locate(&crate::config::SessionFiles(self), dir))
            .map(|location| location.dir)
            .or_else(|| root.clone())
            .unwrap_or_else(|| path.parent().unwrap_or(path).to_path_buf());
        let relative = root
            .as_deref()
            .and_then(|root| path.strip_prefix(root).ok())
            .map_or_else(|| slashed(path), slashed);
        let mut lenses = Vec::new();
        for test in &analysis.test_procedures {
            let Some(range) = self.range_of(uri, test.name_span) else {
                continue;
            };
            let argument = json!({
                "uri": uri.as_str(),
                "file": path.to_string_lossy(),
                "name": test.name,
                "kind": test.kind.name(),
                "projectRoot": project.to_string_lossy(),
                "testRoot": root.as_deref().map(Path::to_string_lossy),
                "filter": filter(&test.name, &relative),
            });
            for (title, command) in [("Run test", RUN_COMMAND), ("Debug test", DEBUG_COMMAND)] {
                lenses.push(CodeLens {
                    range,
                    command: Command {
                        title: title.into(),
                        command: command.into(),
                        arguments: vec![argument.clone()],
                    },
                });
            }
        }
        Ok(lenses)
    }
}
