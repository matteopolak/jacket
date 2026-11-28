//! The options `jaic` and `jailint` take with one spelling and one meaning: the target, the
//! import folders, added source, the number of jobs and the colour choice. A tool declares them
//! once by flattening [`SharedFlags`] into its own options.
use std::path::PathBuf;

use argdef::{Args, Error};
use jaic::render::ColorChoice;
use jaic::target::{Selection, TargetCpu, TargetOs};

/// The spellings of `--color`, for reading the choice before the options are parsed.
pub const COLOR_NAMES: &[&str] = &["--color"];

/// The shared options, as written on the command line.
// The doc comments are `-help` text: names and variables stay unquoted.
#[allow(clippy::doc_markdown, clippy::duplicated_attributes)]
#[derive(Clone, Debug, Default, Args)]
pub struct SharedFlags {
    /// target OS: linux, windows, macos or wasm
    #[arg(name = "-os", value = "os", needs = "an OS name: linux, windows, macos or wasm", parse = read_os, tag = "target", section = "Program")]
    pub os: Option<TargetOs>,
    /// target CPU: x64 or arm64
    #[arg(name = "-cpu", value = "cpu", needs = "a CPU name: x64 or arm64", parse = read_cpu, tag = "target")]
    pub cpu: Option<TargetCpu>,
    /// build for an LLVM target triple (overrides -os and -cpu)
    #[arg(
        name = "-target",
        value = "triple",
        needs = "an LLVM target triple, such as x86_64-pc-windows-gnu",
        tag = "target"
    )]
    pub target: Option<String>,
    /// also look for modules in <dir> (repeatable; short form -I)
    #[arg(
        name = "-import_dir",
        name = "-I",
        synopsis = "-import_dir <dir>",
        value = "dir",
        needs = "a directory",
        hint = "dir",
        tag = "import"
    )]
    pub imports: Vec<PathBuf>,
    /// also look for modules in <dir>, for the files under <package> only (repeatable)
    #[arg(name = "-package_import_dir", synopsis = "-package_import_dir <package>=<dir>", needs = "a package folder and a directory, as <package>=<dir>", parse = read_package_import)]
    pub package_imports: Vec<(PathBuf, PathBuf)>,
    /// add the code to the target program (repeatable)
    #[arg(
        name = "-add",
        alias = "--add",
        synopsis = "-add \"code\"",
        needs = "the code to add to the program"
    )]
    pub add: Vec<String>,
    /// run n compiles (or tests) at once (default: one per CPU)
    #[arg(name = "-j", alias = "--jobs", synopsis = "-j, --jobs <n>", value = "n", needs = "a number of jobs", parse = read_jobs, section = "Performance")]
    pub jobs: Option<usize>,
    /// colour in diagnostics: auto, always or never
    #[arg(name = "--color", value = "when", needs = "auto, always or never", parse = read_color, section = "Diagnostics and output")]
    pub color: Option<ColorChoice>,
}

impl SharedFlags {
    /// How many jobs `-j` asks for; `None` for no flag or `-j 0`: one per core.
    pub fn jobs(&self) -> Option<usize> {
        self.jobs.filter(|&n| n > 0)
    }

    /// What `-os`, `-cpu` and `-target` select; `None` fields are the host's.
    pub fn selection(&self) -> Selection {
        Selection {
            os: self.os,
            cpu: self.cpu,
            triple: self.target.clone(),
        }
    }
}

/// Reads the `-os` value; the error lists the valid names.
pub fn read_os(name: &str, text: &str) -> Result<TargetOs, Error> {
    jaic::target::parse_os(text).ok_or_else(|| {
        let mut error = Error::invalid(format!("unknown OS `{text}` for `{name}`"));

        if let Some(near) = jaic::suggest::closest(text, jaic::target::OS_NAMES) {
            error = error.help(format!("did you mean `{name} {near}`?"));
        }

        error.help("the OS names are linux, windows, macos and wasm")
    })
}

/// Reads the `-cpu` value, `x64` or `arm64` (or their long names).
pub fn read_cpu(name: &str, text: &str) -> Result<TargetCpu, Error> {
    match jaic::target::parse_cpu(text) {
        Some(cpu) if cpu != TargetCpu::Wasm => Ok(cpu),
        _ => Err(Error::invalid(format!("unknown CPU `{text}` for `{name}`"))
            .help("the CPU names are x64 (or x86_64) and arm64 (or aarch64)")),
    }
}

/// Reads a `-package_import_dir` value, `<package>=<dir>`, into the package folder and the
/// folder to search for it; both halves must be non-empty.
pub fn read_package_import(name: &str, text: &str) -> Result<(PathBuf, PathBuf), Error> {
    match text.split_once('=') {
        Some((package, dir)) if !package.is_empty() && !dir.is_empty() => {
            Ok((PathBuf::from(package), PathBuf::from(dir)))
        }
        _ => Err(Error::invalid(format!(
            "`{name}` needs <package>=<dir>, not `{text}`"
        ))),
    }
}

/// How many jobs `-j` asks for.
pub fn read_jobs(name: &str, text: &str) -> Result<usize, Error> {
    text.parse().map_err(|_| {
        Error::invalid(format!(
            "`{name}` takes a number of jobs, such as `{name} 4`"
        ))
    })
}

/// Reads a `--color` value: `auto`, `always` or `never`.
pub fn read_color(name: &str, text: &str) -> Result<ColorChoice, Error> {
    ColorChoice::parse(text).ok_or_else(|| {
        Error::invalid(format!("unknown `{name}` value `{text}`")).help("use auto, always or never")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<SharedFlags, Error> {
        SharedFlags::parse(&args.iter().map(ToString::to_string).collect::<Vec<_>>())
    }

    #[test]
    fn every_spelling_reads_its_value() {
        let flags = parse(&[
            "-os=linux",
            "-cpu",
            "arm64",
            "--jobs",
            "4",
            "-I",
            "a",
            "-import_dir=b",
            "--add",
            "x :: 1;",
            "--color=never",
            "-package_import_dir",
            "p=q",
        ])
        .unwrap();

        assert_eq!(flags.os, Some(TargetOs::Linux));
        assert_eq!(flags.cpu, Some(TargetCpu::Arm64));
        assert_eq!(flags.jobs, Some(4));
        assert_eq!(flags.imports, [PathBuf::from("a"), PathBuf::from("b")]);
        assert_eq!(flags.add, ["x :: 1;"]);
        assert_eq!(flags.color, Some(ColorChoice::Never));
        assert_eq!(flags.package_imports, [("p".into(), "q".into())]);
        assert_eq!(parse(&["-j", "0"]).unwrap().jobs(), None);
    }

    #[test]
    fn values_are_checked_and_errors_say_what_is_wrong() {
        let error = parse(&["-os", "linx"]).unwrap_err();

        assert!(error.help[0].contains("did you mean `-os linux`"));
        assert!(parse(&["-cpu", "wasm"]).is_err());
        assert!(parse(&["--jobs", "many"]).is_err());
        assert!(parse(&["--color", "sometimes"]).is_err());
        assert_eq!(
            parse(&["-os"]).unwrap_err().message,
            "`-os` needs an OS name: linux, windows, macos or wasm"
        );
    }

    #[test]
    fn spellings_do_not_repeat_across_flags() {
        let spec = SharedFlags::spec();
        let mut all: Vec<&str> = spec
            .opts
            .iter()
            .flat_map(|opt| opt.names.iter().chain(opt.aliases).copied())
            .collect();
        let before = all.len();

        all.sort_unstable();
        all.dedup();
        assert_eq!(all.len(), before);
    }
}
