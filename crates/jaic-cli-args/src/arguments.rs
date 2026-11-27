//! Target metadata and host discovery arguments, using the compiler's option table.
use crate::cli::Cli;
use argdef::Args;
use jaic::flags::Common;
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
    let scan = Cli::spec().scan(args);
    if let Some(error) = scan.error {
        return Err(error.message);
    }
    let mut common = Common::default();
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
                // Only these three shared flags select the target; their values are checked here.
                let value = token.value.as_deref().unwrap_or_default();
                let (flag, _) = jaic::flags::find(jaic::flags::COMPILE, token.name)
                    .expect("a `target` option is a shared flag");
                common.take(
                    flag.parse(token.name, value)
                        .map_err(jaic::flags::FlagError::into_text)?,
                );
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
        selection: common.target,
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

    /// `jailint` and `jacket` read the flags in `jaic::flags`; `Cli` declares them again with
    /// derive attributes. Every part of a declaration that a tool shows or accepts must agree.
    #[test]
    fn shared_flags_are_declared_alike_by_cli_and_jaic_flags() {
        let spec = Cli::spec();
        for flag in jaic::flags::ALL {
            let opt = spec
                .opts
                .iter()
                .find(|o| o.spelled(flag.names[0]))
                .unwrap_or_else(|| panic!("`Cli` has no `{}`", flag.names[0]));
            let mut declared: Vec<&str> = opt.names.iter().chain(opt.aliases).copied().collect();
            let mut shared: Vec<&str> = flag.names.to_vec();
            declared.sort_unstable();
            shared.sort_unstable();
            assert_eq!(declared, shared, "{}: spellings", flag.names[0]);
            assert_eq!(opt.needs, flag.what, "{}: needs", flag.names[0]);
            assert_eq!(opt.label(), flag.synopsis, "{}: synopsis", flag.names[0]);
            assert_eq!(opt.help, flag.help, "{}: help", flag.names[0]);
        }
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
