//! Target metadata and host discovery arguments, using the compiler's option table.
use std::path::PathBuf;

use argdef::Args;
use jaic::target::Selection;

use crate::cli::Cli;
use crate::shared::{read_cpu, read_os};

/// Target selection and host arguments with compiler target/mode options removed.
#[derive(Debug)]
pub struct TargetFlags {
    /// The target `-os`, `-cpu` and `-target` choose; a later flag overrides an earlier one.
    pub selection: Selection,
    /// The arguments for the host side, in order: every argument but the target flags and the
    /// mode switches, then everything from the first boundary on.
    pub host: Vec<String>,
    /// Whether `-check` was given.
    pub check: bool,
    /// Whether `-interp` was given.
    pub interp: bool,
    /// Whether `-deps` was given.
    pub deps: bool,
    mode_indices: Vec<usize>,
    /// Every option value before the first boundary, target flags included.
    pub values: Vec<ArgumentValue>,
}

impl TargetFlags {
    /// Remove only actual compiler mode switches, leaving option values and boundaries intact.
    pub fn without_modes(&self, mut args: Vec<String>) -> Vec<String> {
        let mut modes = self.mode_indices.iter().copied().peekable();
        let mut index = 0;

        args.retain(|_| {
            let keep = modes.peek() != Some(&index);

            if !keep {
                modes.next();
            }

            index += 1;
            keep
        });
        args
    }
}

/// One value consumed by a compiler option, before plugin or program arguments.
#[derive(Debug)]
pub struct ArgumentValue {
    /// The option as written, without an inline value.
    pub name: String,
    pub value: String,
    index: usize,
    inline: bool,
}

impl ArgumentValue {
    /// Replace this value in the original argument sequence, keeping spaced/inline syntax.
    pub fn replace(&self, args: &mut [String], value: String) {
        if self.inline {
            args[self.index] = format!("{}={value}", self.name);
        } else {
            args[self.index + 1] = value;
        }
    }
}

/// The text `jaic::render::Report::from_text` renders: the message, then a `help:` line for each
/// help.
fn error_text(error: argdef::Error) -> String {
    error.help.iter().fold(error.message, |mut text, help| {
        text.push_str("\nhelp: ");
        text.push_str(help);
        text
    })
}

/// Inspect compiler arguments without interpreting option values as new options.
/// Plugins and the `-`/`--` argument boundaries have the same meaning as in `jaic`.
///
/// ```
/// let flags = jaic_cli_args::arguments::target_flags(&["-os=wasm".into()]).unwrap();
/// assert!(flags.host.is_empty());
/// ```
pub fn target_flags(args: &[String]) -> Result<TargetFlags, String> {
    let scan = Cli::spec().scan(args);

    if let Some(error) = scan.error {
        return Err(error.message);
    }

    let mut selection = Selection::default();
    let mut host = Vec::new();
    let mut values = Vec::new();
    let (mut check, mut interp, mut deps) = (false, false, false);
    let mut mode_indices = Vec::new();

    for token in &scan.tokens {
        let tag = token.opt.map_or("", |opt| opt.tag);

        match tag {
            "check" => check = true,
            "interp" => interp = true,
            "deps" => deps = true,
            "target" => {
                // Only these three options select the target; their values are checked here.
                let value = token.value.as_deref().unwrap_or_default();

                match token.name {
                    "-os" => selection.os = Some(read_os(token.name, value).map_err(error_text)?),
                    "-cpu" => {
                        selection.cpu = Some(read_cpu(token.name, value).map_err(error_text)?)
                    }
                    _ => selection.triple = Some(value.to_string()),
                }
            }
            _ => {}
        }

        if matches!(tag, "check" | "interp" | "deps") {
            mode_indices.push(token.index);
        } else if tag != "target" {
            host.push(args[token.index].clone());

            if let Some(value) = &token.value
                && !token.inline
            {
                host.push(value.clone());
            }
        }

        if let Some(value) = &token.value {
            values.push(ArgumentValue {
                name: token.name.into(),
                value: value.clone(),
                index: token.index,
                inline: token.inline,
            });
        }
    }

    if let Some(boundary) = scan.boundary {
        host.extend(args[boundary..].iter().cloned());
    }

    Ok(TargetFlags {
        selection,
        host,
        check,
        interp,
        deps,
        mode_indices,
        values,
    })
}

/// The folders `-import_dir` and its short form `-I` name among the compiler arguments, as
/// written, in order. Values of other options (`-add "-I"`) and everything past a boundary
/// are not options. Unlike [`target_flags`] it takes a malformed argument as the end of what it
/// can read, because the compiler is the one to report it.
///
/// ```
/// let args = ["-I", "a", "-import_dir=b", "--", "-I", "c"].map(String::from);
/// let dirs = jaic_cli_args::arguments::import_dirs(&args);
/// assert_eq!(dirs, [std::path::PathBuf::from("a"), "b".into()]);
/// ```
pub fn import_dirs(args: &[impl AsRef<std::ffi::OsStr>]) -> Vec<PathBuf> {
    Cli::spec()
        .scan(args)
        .tokens
        .iter()
        .filter(|token| token.opt.is_some_and(|opt| opt.tag == "import"))
        .filter_map(|token| token.value.as_deref().map(PathBuf::from))
        .collect()
}

#[cfg(test)]
mod tests {
    use jaic::target::{TargetCpu, TargetOs};

    use super::*;

    fn inspect(args: &[&str]) -> TargetFlags {
        target_flags(&args.iter().map(ToString::to_string).collect::<Vec<_>>()).unwrap()
    }

    #[test]
    fn target_arguments_preserve_precedence_and_non_target_values() {
        let flags = inspect(&[
            "-os=wasm",
            "-cpu",
            "aarch64",
            "-os",
            "linux",
            "-target=wasm64-unknown-wasi",
            "-add",
            "-os",
            "-exe=-cpu",
        ]);

        assert_eq!(flags.selection.os, Some(TargetOs::Linux));
        assert_eq!(flags.selection.cpu, Some(TargetCpu::Arm64));
        assert_eq!(
            flags.selection.triple.as_deref(),
            Some("wasm64-unknown-wasi")
        );
        assert_eq!(flags.host, ["-add", "-os", "-exe=-cpu"]);
    }

    #[test]
    fn target_arguments_respect_plugin_and_argument_boundaries() {
        for boundary in ["-", "--", "+Plugin"] {
            assert_eq!(
                inspect(&["-os=wasm", boundary, "-cpu", "x64"]).host,
                [boundary, "-cpu", "x64"]
            );
        }
    }

    #[test]
    fn compiler_modes_keep_value_and_program_boundaries() {
        for boundary in ["-", "--", "+Plugin"] {
            let args: Vec<String> = ["-interp", "-add", "-check", "-run=-deps", boundary, "-deps"]
                .into_iter()
                .map(str::to_owned)
                .collect();
            let parsed = target_flags(&args).unwrap();

            assert!(parsed.interp);
            assert!(!parsed.check && !parsed.deps);
            assert_eq!(parsed.host, args[1..]);
            assert_eq!(parsed.without_modes(args.clone()), args[1..]);
        }

        let parsed = inspect(&["-check", "-deps"]);

        assert!(parsed.check && parsed.deps);
        assert!(parsed.host.is_empty());

        for (check, interp, deps) in [(true, true, false), (false, true, true)] {
            assert!(crate::cli::Mode::from_flags(check, interp, deps).is_err());
        }
    }

    #[test]
    fn import_folders_come_from_the_option_table() {
        let dirs =
            |args: &[&str]| import_dirs(&args.iter().map(ToString::to_string).collect::<Vec<_>>());
        let path = |text: &str| PathBuf::from(text);

        assert_eq!(
            dirs(&["-I", "a", "-O1", "-import_dir=b", "-import_dir", "c"]),
            [path("a"), path("b"), path("c")]
        );
        // A value of another option is not an option, whatever it looks like.
        assert_eq!(dirs(&["-add", "-I", "-I", "x", "-exe", "-I"]), [path("x")]);

        // Plugin and program arguments are not the compiler's.
        for boundary in ["-", "--", "+Plugin"] {
            assert_eq!(dirs(&["-I", "a", boundary, "-I", "b"]), [path("a")]);
        }

        // A flag with no value is for the compiler to report.
        assert_eq!(dirs(&["-I", "a", "-I"]), [path("a")]);
        assert!(dirs(&[]).is_empty());
    }

    #[test]
    fn malformed_target_arguments_are_errors() {
        for args in [&["-os"][..], &["-cpu=bad"], &["-os=bad"], &["-target"]] {
            assert!(
                target_flags(&args.iter().map(ToString::to_string).collect::<Vec<_>>()).is_err()
            );
        }
    }
}
