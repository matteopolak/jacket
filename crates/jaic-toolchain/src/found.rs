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
    /// What it is, when the path alone does not say (`compiler-rt 22.1.0`).
    pub detail: Option<String>,
    /// The sysroot a link against it uses, when it names one: the driver's `--sysroot`.
    pub sysroot: Option<Box<Found>>,
}

impl Found {
    pub fn new(program: impl Into<PathBuf>, origin: Origin) -> Found {
        Found {
            program: program.into(),
            args: Vec::new(),
            env: Vec::new(),
            origin,
            detail: None,
            sysroot: None,
        }
    }

    #[must_use]
    pub fn with_sysroot(mut self, sysroot: Found) -> Found {
        self.sysroot = Some(Box::new(sysroot));
        self
    }

    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<String>) -> Found {
        self.detail = Some(detail.into());
        self
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

    /// The line a status shows: its detail when it has one (`compiler-rt 21.1.8 (cached)`), else
    /// the path with `~` for the home folder and the version it reports; then how it was chosen
    /// when that was not the system.
    ///
    /// ```text
    /// /opt/homebrew/bin/ld.lld 21.1.8
    /// ~/Library/Caches/jacket/toolchains/armv7-unknown-linux-gnueabihf/sysroot (jacket toolchain)
    /// libgcc from arm-none-eabi-gcc 14.2.1
    /// ```
    pub fn describe(&self, lookup: &Lookup) -> String {
        let mut text = self.detail.clone().unwrap_or_else(|| {
            let mut text = tilde(&self.program);
            if let Some(version) = self.version(lookup) {
                text.push(' ');
                text.push_str(&version);
            }
            text
        });
        // What a cached tool says about itself already says where it came from.
        let cached_detail = self.detail.is_some() && self.origin == Origin::Cache;
        if let Some(label) = self.origin.label().filter(|_| !cached_detail) {
            text.push_str(&format!(" ({label})"));
        }
        text
    }

    /// What `--version` prints, as a version number, when the program says one.
    pub fn version(&self, lookup: &Lookup) -> Option<String> {
        lookup
            .host
            .run(&self.program, &["--version"])
            .and_then(|text| version_of(&text))
    }
}

/// `path` with the home folder written `~`, for the screen only.
pub fn tilde(path: &std::path::Path) -> String {
    let shown = path.display().to_string();
    match jacket_config::env::HOME.os().map(PathBuf::from) {
        Some(home) if !home.as_os_str().is_empty() && path.starts_with(&home) => {
            format!("~{}", &shown[home.display().to_string().len()..])
        }
        _ => shown,
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
