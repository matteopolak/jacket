//! The command-line flags that `jaic`, `jailint` and `jacket` take with one spelling and one
//! meaning: what they are called, what they take, how a value is checked, and the help line and
//! error text for each. Each tool keeps its own storage and its own other flags; it looks a flag
//! up here ([`find`]), reads its value ([`SharedFlag::take`]) and folds the result ([`Shared`])
//! into what it keeps. Adding a flag the tools share means adding it to [`ALL`].
use crate::render::ColorChoice;
use crate::target::Selection;
use crate::target::{TargetCpu, TargetOs};
use std::path::PathBuf;

/// A mistake in a flag or its value: the message, then help lines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlagError {
    pub message: String,
    pub help: Vec<String>,
}

impl FlagError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            help: Vec::new(),
        }
    }

    #[must_use]
    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help.push(help.into());
        self
    }

    /// The text `jaic::render::Report::from_text` renders: the message, then a `help:` line
    /// for each help.
    pub fn into_text(self) -> String {
        let mut text = self.message;
        for help in self.help {
            text.push_str("\nhelp: ");
            text.push_str(&help);
        }
        text
    }
}

/// A shared flag, read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shared {
    Os(TargetOs),
    Cpu(TargetCpu),
    /// An LLVM target triple, which overrides the OS and CPU.
    Triple(String),
    ImportDir(PathBuf),
    /// Source added to the program's scope.
    Add(String),
    /// Programs (or tests) handled at once.
    Jobs(usize),
    Color(ColorChoice),
}

/// One flag the tools share.
pub struct SharedFlag {
    /// Its spellings; a mistyped flag suggests the nearest of these.
    pub names: &'static [&'static str],
    /// What it takes, for the "needs ..." error.
    pub what: &'static str,
    /// The flag as the help spells it, with its value (`-os <os>`).
    pub synopsis: &'static str,
    /// What it does.
    pub help: &'static str,
    parse: fn(&str, &str) -> Result<Shared, FlagError>,
}

impl SharedFlag {
    /// The flag spelt `name`, given `value`.
    pub fn parse(&self, name: &str, value: &str) -> Result<Shared, FlagError> {
        (self.parse)(name, value)
    }

    /// The error for `name` given without a value.
    pub fn needs_value(&self, name: &str) -> FlagError {
        FlagError::new(format!("`{name}` needs {}", self.what))
    }

    /// Read the flag spelt `name`: its value is `inline` (`name=value`) or what `next` gives.
    pub fn take(
        &self,
        name: &str,
        inline: Option<String>,
        next: impl FnOnce() -> Option<String>,
    ) -> Result<Shared, FlagError> {
        let value = inline.or_else(next).ok_or_else(|| self.needs_value(name))?;
        self.parse(name, &value)
    }
}

/// `-os`: the target operating system.
pub const OS: SharedFlag = SharedFlag {
    names: &["-os"],
    what: "an OS name: linux, windows, macos or wasm",
    synopsis: "-os <os>",
    help: "target OS: linux, windows, macos or wasm",
    parse: |_, os| {
        crate::target::parse_os(os).map(Shared::Os).ok_or_else(|| {
            let mut error = FlagError::new(format!("unknown OS `{os}` for `-os`"));
            if let Some(near) = crate::suggest::closest(os, crate::target::OS_NAMES) {
                error = error.help(format!("did you mean `-os {near}`?"));
            }
            error.help("the OS names are linux, windows, macos and wasm")
        })
    },
};

/// `-cpu`: the target processor.
pub const CPU: SharedFlag = SharedFlag {
    names: &["-cpu"],
    what: "a CPU name: x64 or arm64",
    synopsis: "-cpu <cpu>",
    help: "target CPU: x64 or arm64",
    parse: |_, cpu| match crate::target::parse_cpu(cpu) {
        Some(parsed) if parsed != TargetCpu::Wasm => Ok(Shared::Cpu(parsed)),
        _ => Err(FlagError::new(format!("unknown CPU `{cpu}` for `-cpu`"))
            .help("the CPU names are x64 (or x86_64) and arm64 (or aarch64)")),
    },
};

/// `-target`: an LLVM target triple.
pub const TARGET: SharedFlag = SharedFlag {
    names: &["-target"],
    what: "an LLVM target triple, such as x86_64-pc-windows-gnu",
    synopsis: "-target <triple>",
    help: "build for an LLVM target triple (overrides -os and -cpu)",
    parse: |_, triple| Ok(Shared::Triple(triple.to_string())),
};

/// `-I`, `-import_dir`: a folder to look for modules in.
pub const IMPORT_DIR: SharedFlag = SharedFlag {
    names: &["-import_dir", "-I"],
    what: "a directory",
    synopsis: "-import_dir <dir>",
    help: "also look for modules in <dir> (repeatable; short form -I)",
    parse: |_, dir| Ok(Shared::ImportDir(PathBuf::from(dir))),
};

/// `-add`: source added to the program.
pub const ADD: SharedFlag = SharedFlag {
    names: &["-add", "--add"],
    what: "the code to add to the program",
    synopsis: "-add \"code\"",
    help: "add the code to the target program (repeatable)",
    parse: |_, code| Ok(Shared::Add(code.to_string())),
};

/// `-j`, `--jobs`: how many things run at once.
pub const JOBS: SharedFlag = SharedFlag {
    names: &["-j", "--jobs"],
    what: "a number of jobs",
    synopsis: "-j, --jobs <n>",
    help: "run n compiles (or tests) at once (default: one per CPU)",
    parse: |name, n| {
        n.parse().map(Shared::Jobs).map_err(|_| {
            FlagError::new(format!(
                "`{name}` takes a number of jobs, such as `{name} 4`"
            ))
        })
    },
};

/// `--color`: whether diagnostics use colour.
pub const COLOR: SharedFlag = SharedFlag {
    names: &["--color"],
    what: "auto, always or never",
    synopsis: "--color <when>",
    help: "colour in diagnostics: auto, always or never",
    parse: |_, when| {
        ColorChoice::parse(when).map(Shared::Color).ok_or_else(|| {
            FlagError::new(format!("unknown `--color` value `{when}`"))
                .help("use auto, always or never")
        })
    },
};

/// Every shared flag.
pub const ALL: &[&SharedFlag] = &[&OS, &CPU, &TARGET, &IMPORT_DIR, &ADD, &JOBS, &COLOR];

/// The flags that choose what is compiled and for which target: what a tool that compiles
/// programs takes.
pub const COMPILE: &[&SharedFlag] = &[&OS, &CPU, &TARGET, &IMPORT_DIR, &ADD];

/// The flag of `set` spelt `name`, together with the value after an `=` when it is written
/// `name=value` (any shared flag takes one).
pub fn find(
    set: &[&'static SharedFlag],
    name: &str,
) -> Option<(&'static SharedFlag, Option<String>)> {
    let named = |n: &str| set.iter().copied().find(|flag| flag.names.contains(&n));
    if let Some(flag) = named(name) {
        return Some((flag, None));
    }
    let (n, value) = name.split_once('=')?;
    named(n).map(|flag| (flag, Some(value.to_string())))
}

/// Every spelling in `set`, for "did you mean" suggestions.
pub fn spellings<'a>(set: &'a [&'static SharedFlag]) -> impl Iterator<Item = &'static str> + 'a {
    set.iter().flat_map(|flag| flag.names.iter().copied())
}

/// What the shared flags of one run came to: the folded form for a tool with no storage of
/// its own for them.
#[derive(Clone, Debug, Default)]
pub struct Common {
    pub target: Selection,
    pub imports: Vec<PathBuf>,
    pub add: Vec<String>,
    pub jobs: Option<usize>,
    pub color: Option<ColorChoice>,
}

impl Common {
    pub fn take(&mut self, flag: Shared) {
        match flag {
            Shared::Os(os) => self.target.os = Some(os),
            Shared::Cpu(cpu) => self.target.cpu = Some(cpu),
            Shared::Triple(triple) => self.target.triple = Some(triple),
            Shared::ImportDir(dir) => self.imports.push(dir),
            Shared::Add(code) => self.add.push(code),
            Shared::Jobs(n) => self.jobs = Some(n),
            Shared::Color(choice) => self.color = Some(choice),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flag_is_found_by_any_spelling_and_by_name_equals_value() {
        let (flag, inline) = find(ALL, "--jobs").unwrap();
        assert_eq!((flag.names[0], inline), ("-j", None));
        let (flag, inline) = find(ALL, "-os=linux").unwrap();
        assert_eq!((flag.names[0], inline.as_deref()), ("-os", Some("linux")));
        assert!(find(COMPILE, "--color").is_none());
        assert!(find(ALL, "-bogus").is_none());
    }

    #[test]
    fn values_are_checked_and_errors_say_what_is_wrong() {
        assert_eq!(OS.parse("-os", "linux"), Ok(Shared::Os(TargetOs::Linux)));
        let error = OS.parse("-os", "linx").unwrap_err();
        assert!(error.help[0].contains("did you mean `-os linux`"));
        assert!(CPU.parse("-cpu", "wasm").is_err());
        assert_eq!(JOBS.parse("-j", "4"), Ok(Shared::Jobs(4)));
        assert!(JOBS.parse("--jobs", "many").is_err());
        assert_eq!(
            COLOR.parse("--color", "never"),
            Ok(Shared::Color(ColorChoice::Never))
        );
        assert_eq!(
            OS.take("-os", None, || None).unwrap_err().message,
            "`-os` needs an OS name: linux, windows, macos or wasm"
        );
    }

    #[test]
    fn spellings_do_not_repeat_across_flags() {
        let mut all: Vec<_> = spellings(ALL).collect();
        let before = all.len();
        all.sort_unstable();
        all.dedup();
        assert_eq!(all.len(), before);
    }
}
