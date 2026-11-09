//! Target metadata and host discovery arguments, using the compiler's option table.
use crate::options::{Action, CliOption};
use jaic::flags::{Common, Shared};
use jaic::target::Selection;
use std::path::PathBuf;

/// Target selection and host arguments with compiler target/mode options removed.
#[derive(Debug)]
pub struct TargetFlags {
    pub selection: Selection,
    pub host: Vec<String>,
    pub check: bool,
    pub interp: bool,
    pub deps: bool,
    mode_indices: Vec<usize>,
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

/// Inspect compiler arguments without interpreting option values as new options.
/// Plugins and the `-`/`--` argument boundaries have the same meaning as in `jaic`.
///
/// ```
/// let flags = jaic_cli_args::arguments::target_flags(&["-os=wasm".into()]).unwrap();
/// assert!(flags.host.is_empty());
/// ```
pub fn target_flags(args: &[String]) -> Result<TargetFlags, String> {
    let mut common = Common::default();
    let mut host = Vec::new();
    let mut values = Vec::new();
    let (mut check, mut interp, mut deps) = (false, false, false);
    let mut mode_indices = Vec::new();
    let mut index = 0;
    while let Some(arg) = args.get(index) {
        if is_boundary(arg) {
            host.extend(args[index..].iter().cloned());
            break;
        }
        let (name, inline) = split_option(arg);
        let option = CliOption::find(name);
        if let Some(option) = option {
            match option.action {
                Action::Check => check = true,
                Action::Interp => interp = true,
                Action::Deps => deps = true,
                _ => {}
            }
        }
        let mode = matches!(
            option.map(|o| o.action),
            Some(Action::Check | Action::Interp | Action::Deps)
        );
        if mode {
            mode_indices.push(index);
        }
        let takes_value = matches!(
            option.map(|o| o.action),
            Some(Action::Value { .. } | Action::Shared(_))
        );
        let value = if takes_value {
            Some(
                inline
                    .clone()
                    .or_else(|| args.get(index + 1).cloned())
                    .ok_or_else(|| match option.map(|o| o.action) {
                        Some(Action::Shared(flag)) => flag.needs_value(name).into_text(),
                        Some(Action::Value {
                            what, ..
                        }) => format!("`{name}` needs {what}"),
                        _ => unreachable!("only value-taking options are inspected here"),
                    })?,
            )
        } else {
            None
        };
        let shared = match option.map(|o| o.action) {
            Some(Action::Shared(flag)) => Some(
                flag.take(name, value.clone(), || None)
                    .map_err(jaic::flags::FlagError::into_text)?,
            ),
            _ => None,
        };
        if let Some(shared @ (Shared::Os(_) | Shared::Cpu(_) | Shared::Triple(_))) = shared {
            common.take(shared);
        } else if !mode {
            host.push(arg.clone());
            if takes_value && inline.is_none() {
                host.extend(value.iter().cloned());
            }
        }
        if let Some(value) = value {
            values.push(ArgumentValue {
                name: name.into(),
                value,
                index,
                inline: inline.is_some(),
            });
        }
        index += if takes_value && inline.is_none() {
            2
        } else {
            1
        };
    }
    Ok(TargetFlags {
        selection: common.target,
        host,
        check,
        interp,
        deps,
        mode_indices,
        values,
    })
}

/// `-`, `--` and `+Plugin` end the compiler's own options: what follows is for the plugin or the
/// program.
fn is_boundary(arg: &str) -> bool {
    arg == "-" || arg == "--" || arg.starts_with('+') && arg.len() > 1
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
pub fn import_dirs(args: &[String]) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut index = 0;
    while let Some(arg) = args.get(index) {
        if is_boundary(arg) {
            break;
        }
        let (name, inline) = split_option(arg);
        let Some(option) = CliOption::find_value(name) else {
            index += 1;
            continue;
        };
        let spaced = inline.is_none();
        if spaced && args.get(index + 1).is_none() {
            break;
        }
        if let Action::Shared(flag) = option.action
            && let Ok(Shared::ImportDir(dir)) =
                flag.take(name, inline, || args.get(index + 1).cloned())
        {
            dirs.push(dir);
        }
        index += if spaced {
            2
        } else {
            1
        };
    }
    dirs
}

/// The compiler's inline value syntax, shared by parsing and metadata inspection.
pub fn split_option(arg: &str) -> (&str, Option<String>) {
    match arg.split_once('=') {
        Some((name, value)) if CliOption::find_value(name).is_some() => (name, Some(value.into())),
        None if arg.len() > 2 && matches!(arg.get(..2), Some("-A" | "-W" | "-D")) => {
            (&arg[..2], Some(arg[2..].into()))
        }
        _ => (arg, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jaic::target::{TargetCpu, TargetOs};

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
