//! The outcome of a lookup: the tool and how it was chosen, or what is missing and how to get it.
use crate::lookup::{Lookup, version_of};
use std::ffi::OsString;
use std::path::PathBuf;

/// How a tool came to be chosen. A status line shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Origin {
    /// An option or a variable named it: `-linker`, `JAIC_LINKER`.
    Override(&'static str),
    /// The system's own: on `PATH` or in the place its installer uses.
    System,
    /// Fetched by `jacket toolchain add`.
    Cache,
}

impl Origin {
    /// What follows the path in a status line: nothing for a system tool.
    pub fn label(&self) -> Option<String> {
        match self {
            Origin::Override(name) => Some(format!("override: {name}")),
            Origin::System => None,
            Origin::Cache => Some("jacket toolchain".to_string()),
        }
    }
}

/// A program to run, with what it needs in front of the caller's own arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub program: PathBuf,
    /// Arguments that come first (`--target=...`, `--sysroot=...`).
    pub args: Vec<OsString>,
    /// Variables to set for it (MSVC's `LIB`).
    pub env: Vec<(OsString, OsString)>,
    pub origin: Origin,
}

impl Found {
    pub fn new(program: impl Into<PathBuf>, origin: Origin) -> Found {
        Found {
            program: program.into(),
            args: Vec::new(),
            env: Vec::new(),
            origin,
        }
    }

    #[must_use]
    pub fn with_args<I, S>(mut self, args: I) -> Found
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// What `--version` prints, as a version number, when the program says one.
    pub fn version(&self, lookup: &Lookup) -> Option<String> {
        lookup
            .host
            .run(&self.program, &["--version"])
            .and_then(|text| version_of(&text))
    }
}

/// A piece the machine lacks: what could not be done, and the notes that say how to fix it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Missing {
    pub what: String,
    pub notes: Vec<String>,
}

impl Missing {
    pub fn new(what: impl Into<String>, notes: Vec<String>) -> Missing {
        Missing {
            what: what.into(),
            notes,
        }
    }
}

impl std::fmt::Display for Missing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.what)?;
        for note in &self.notes {
            write!(f, "\nnote: {note}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Missing {
}
