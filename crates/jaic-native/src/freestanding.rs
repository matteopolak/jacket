//! Linking freestanding programs (`*-none-elf`) with `ld.lld` (see `docs/native/embedded.md`).
//!
//! There is no C driver, libc or startup file: the program defines its own entry point, and a
//! linker script places its sections. `ld.lld` is the only linker used; if it is missing the
//! build fails with a note that says how to get it.
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use jacket_config::os_text;
use jaic::ir::Library;
use jaic::target::{Target, TargetCpu};
use jaic_toolchain::builtins::find_builtins;
use jaic_toolchain::linker::find_linker;
use jaic_toolchain::{Found, Overrides};

use crate::toolchain::{command, lookup, report};

/// How a freestanding program is linked.
pub struct FreestandingLink<'a> {
    pub objects: &'a [PathBuf],
    pub libraries: &'a [Library],
    pub output: &'a Path,
    pub target: Target,
    /// Drop DWARF from the output.
    pub strip_debug: bool,
    /// `additional_linker_arguments`, after jaic's own. One of them must name the linker script
    /// (`-T script.ld`), as `Embedded.link_for` does.
    pub extra_args: &'a [OsString],
}

/// The linker script `args` name (`-T script.ld`, `-Tscript.ld`, `--script=script.ld`).
fn script_file(args: &[OsString]) -> Option<OsString> {
    let mut args = args.iter();

    while let Some(arg) = args.next() {
        if matches!(arg.to_str(), Some("-T" | "--script")) {
            return args.next().cloned();
        }

        if let Some(file) =
            os_text::strip_prefix(arg, "--script=").or_else(|| os_text::strip_prefix(arg, "-T"))
        {
            return Some(file);
        }
    }

    None
}

/// The `-m` emulation of `ld.lld` for a freestanding CPU; `None` for AVR, which has no name
/// and is read from the objects.
fn emulation(cpu: TargetCpu) -> Result<Option<&'static str>, String> {
    match cpu {
        TargetCpu::Arm64 => Ok(Some("aarch64elf")),
        TargetCpu::Arm => Ok(Some("armelf")),
        TargetCpu::Avr => Ok(None),
        other => Err(format!("jaic cannot link a freestanding {other:?} program")),
    }
}

/// The linker script `link` names, or how to add one.
fn required_script(link: &FreestandingLink) -> Result<OsString, String> {
    script_file(link.extra_args).ok_or_else(|| {
        "a freestanding program needs a linker script\n\
         help: call `Embedded.link_for(*options, memory_map)` from the metaprogram, or add \
         `-T <script>` to `additional_linker_arguments`"
            .into()
    })
}

/// The `ld.lld` command for `link`, with the compiler-rt `builtins` archive.
fn lld_command(lld: &Found, link: &FreestandingLink, builtins: &Path) -> Result<Command, String> {
    let mut cmd = command(lld);

    if let Some(emulation) = emulation(link.target.cpu)? {
        cmd.args(["-m", emulation]);
    }

    cmd.args(["-static", "-nostdlib", "--gc-sections"])
        .arg("-o")
        .arg(link.output)
        .args(link.objects);

    if link.strip_debug {
        cmd.arg("--strip-debug");
    }

    for lib in link.libraries {
        if let Some(file) = crate::linker::static_library_file(lib) {
            cmd.arg(file);
        } else {
            let kind = if lib.system {
                "a system library"
            } else {
                "a library"
            };

            return Err(format!(
                "`#library \"{}\"` names {kind}, which a freestanding target cannot link: there \
                 is no libc or dynamic linking\n\
                 help: put a static archive `{0}.a` beside the declaring file, or bind the \
                 symbols with `#elsewhere`",
                lib.name
            ));
        }
    }

    cmd.arg(builtins);
    cmd.args(link.extra_args);
    Ok(cmd)
}

/// Link freestanding objects into an ELF executable with `ld.lld`. The linker script and the
/// compiler builtins are required before the linker starts: every freestanding program links
/// against the builtins, so a missing archive is an error with the command that builds it.
pub fn link_freestanding(link: &FreestandingLink) -> Result<(), String> {
    let script = required_script(link)?;
    let overrides = Overrides::from_env();
    let lookup = lookup()?;
    let lld = find_linker(&lookup, Some(link.target), &overrides)
        .map_err(|missing| missing.to_string())?;
    let builtins =
        find_builtins(&lookup, link.target, &overrides).map_err(|missing| missing.to_string())?;

    report("Linker", &lld, &lookup, Some(link.target));
    report("Builtins", &builtins, &lookup, Some(link.target));
    jaic::progress::tool("Script", &jaic::display_path(Path::new(&script)), true);
    let mut cmd = lld_command(&lld, link, &builtins.program)?;
    let out = crate::linker::run_captured(&mut cmd).map_err(|e| crate::linker_not_run(&lld, &e))?;

    if out.status.success() {
        Ok(())
    } else {
        Err(crate::link_failure(&lld.program, &out))
    }
}

#[cfg(test)]
mod tests {
    use jaic_toolchain::Origin;

    use super::*;

    fn words(args: &[&str]) -> Vec<OsString> {
        args.iter().map(OsString::from).collect()
    }

    #[test]
    fn the_script_is_found_in_each_spelling() {
        let script = |args: &[&str]| script_file(&words(args));

        assert_eq!(script(&["-T", "board.ld"]), Some("board.ld".into()));
        assert_eq!(script(&["-Tboard.ld"]), Some("board.ld".into()));
        assert_eq!(script(&["-x", "--script=b.ld"]), Some("b.ld".into()));
        assert_eq!(script(&["-x"]), None);
        // `-T` must have its operand; the one at the end of the list names no script.
        assert_eq!(script(&["-x", "-T"]), None);
    }

    fn link<'a>(objects: &'a [PathBuf], extra: &'a [OsString]) -> FreestandingLink<'a> {
        FreestandingLink {
            objects,
            libraries: &[],
            output: Path::new("out.elf"),
            target: Target::parse("aarch64-none-elf").unwrap(),
            strip_debug: false,
            extra_args: extra,
        }
    }

    fn lld() -> Found {
        Found::new("ld.lld", Origin::System)
    }

    fn args(cmd: &Command) -> Vec<String> {
        cmd.get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn a_link_without_a_script_says_how_to_add_one() {
        let error = required_script(&link(&[], &[])).unwrap_err();

        assert!(error.contains("linker script"), "{error}");
        assert!(error.contains("link_for"), "{error}");
    }

    #[test]
    fn the_link_line_is_static_and_has_no_c_runtime() {
        let objects = [PathBuf::from("a.o")];
        let extra = words(&["-T", "board.ld"]);
        let builtins = Path::new("rt.a");
        let cmd = lld_command(&lld(), &link(&objects, &extra), builtins).unwrap();

        assert_eq!(
            args(&cmd),
            [
                "-m",
                "aarch64elf",
                "-static",
                "-nostdlib",
                "--gc-sections",
                "-o",
                "out.elf",
                "a.o",
                "rt.a",
                "-T",
                "board.ld"
            ]
        );
    }

    /// `ld.lld` has no `-m` name for AVR: it takes the machine from the objects.
    #[test]
    fn an_avr_link_names_no_emulation() {
        let extra = words(&["-T", "board.ld"]);
        let request = FreestandingLink {
            target: Target::parse("avr-none-elf").unwrap(),
            ..link(&[], &extra)
        };
        let cmd = lld_command(&lld(), &request, Path::new("rt.a")).unwrap();

        assert!(!args(&cmd).contains(&"-m".to_string()), "{:?}", args(&cmd));
    }

    #[test]
    fn a_library_that_needs_an_os_is_an_error_naming_it() {
        let extra = words(&["-Tboard.ld"]);
        let libraries = [Library {
            name: "libc".into(),
            system: true,
            link_always: false,
            no_dll: false,
            base_dir: String::new(),
        }];
        let request = FreestandingLink {
            libraries: &libraries,
            ..link(&[], &extra)
        };
        let error = lld_command(&lld(), &request, Path::new("rt.a")).unwrap_err();

        assert!(error.contains("`#library \"libc\"`"), "{error}");
        assert!(error.contains("no libc"), "{error}");
    }
}
