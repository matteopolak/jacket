//! Linking freestanding programs (`*-none-elf`) with `ld.lld` (see `docs/native/embedded.md`).
//!
//! There is no C driver, libc or startup file: the program defines its own entry point, and a
//! linker script places its sections. `ld.lld` is the only linker used; if it is missing the
//! build fails with a note that says how to get it.
use crate::toolchain::{command, lookup, report};
use jaic::ir::Library;
use jaic::target::{Target, TargetCpu};
use jaic::toolchain::freestanding;
use jaic_toolchain::builtins::find_builtins;
use jaic_toolchain::linker::find_linker;
use jaic_toolchain::{Found, Overrides};
use std::path::{Path, PathBuf};
use std::process::Command;

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
    pub extra_args: &'a [String],
}

/// Whether `args` pass a linker script.
fn names_script(args: &[String]) -> bool {
    args.iter()
        .any(|a| a == "-T" || a.starts_with("-T") || a == "--script" || a.starts_with("--script="))
}

/// The linker script `args` name (`-T script.ld`, `-Tscript.ld`, `--script=script.ld`).
fn script_file(args: &[String]) -> Option<&str> {
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-T" | "--script" => return args.next().map(String::as_str),
            _ => {
                if let Some(file) = arg
                    .strip_prefix("--script=")
                    .or_else(|| arg.strip_prefix("-T"))
                {
                    return Some(file);
                }
            }
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

/// The `ld.lld` command for `link`, with the compiler-rt `builtins` archive when there is one.
fn lld_command(
    lld: &Found,
    link: &FreestandingLink,
    builtins: Option<&Path>,
) -> Result<Command, String> {
    if !names_script(link.extra_args) {
        return Err("a freestanding program needs a linker script\n\
             help: call `Embedded.link_for(*options, memory_map)` from the metaprogram, or add \
             `-T <script>` to `additional_linker_arguments`"
            .into());
    }
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
    cmd.args(builtins);
    cmd.args(link.extra_args);
    Ok(cmd)
}

/// Link freestanding objects into an ELF executable with `ld.lld`.
pub fn link_freestanding(link: &FreestandingLink) -> Result<(), String> {
    let overrides = Overrides::from_env();
    let lookup = lookup();
    let lld = find_linker(&lookup, Some(link.target), &overrides)
        .map_err(|missing| missing.to_string())?;
    // Programs that need no builtins link without them: the lookup's error is kept for a link
    // that fails on one. A builtins archive that was named and is not there is an error now.
    let builtins = find_builtins(&lookup, link.target, &overrides);
    if let (Err(missing), Some(_)) = (&builtins, &overrides.compiler_rt) {
        return Err(missing.to_string());
    }
    report("Linker", &lld, &lookup, Some(link.target));
    if let Ok(found) = &builtins {
        report("Builtins", found, &lookup, Some(link.target));
    }
    if let Some(script) = script_file(link.extra_args) {
        jaic::progress::tool("Script", script, true);
    }
    let mut cmd = lld_command(
        &lld,
        link,
        builtins.as_ref().ok().map(|b| b.program.as_path()),
    )?;
    let program = lld.program.display().to_string();
    let out = crate::linker::run_captured(&mut cmd).map_err(|e| crate::linker_not_run(&lld, &e))?;
    if out.status.success() {
        return Ok(());
    }
    let mut message = crate::link_failure(&program, &out);
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if let Err(missing) = &builtins
        && freestanding::undefined_builtin(&log)
    {
        message += &format!("\nnote: the program needs compiler builtins: {missing}");
    }
    Err(message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jaic_toolchain::Origin;

    fn words(args: &[&str]) -> Vec<String> {
        args.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn the_script_is_found_in_each_spelling() {
        assert_eq!(script_file(&words(&["-T", "board.ld"])), Some("board.ld"));
        assert_eq!(script_file(&words(&["-Tboard.ld"])), Some("board.ld"));
        assert_eq!(script_file(&words(&["-x", "--script=b.ld"])), Some("b.ld"));
        assert_eq!(script_file(&words(&["-x"])), None);
    }

    fn link<'a>(objects: &'a [PathBuf], extra: &'a [String]) -> FreestandingLink<'a> {
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
        let error = lld_command(&lld(), &link(&[], &[]), None).unwrap_err();
        assert!(error.contains("linker script"), "{error}");
        assert!(error.contains("link_for"), "{error}");
    }

    #[test]
    fn the_link_line_is_static_and_has_no_c_runtime() {
        let objects = [PathBuf::from("a.o")];
        let extra = ["-T".to_string(), "board.ld".to_string()];
        let builtins = Path::new("rt.a");
        let cmd = lld_command(&lld(), &link(&objects, &extra), Some(builtins)).unwrap();
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
        let extra = ["-T".to_string(), "board.ld".to_string()];
        let request = FreestandingLink {
            target: Target::parse("avr-none-elf").unwrap(),
            ..link(&[], &extra)
        };
        let cmd = lld_command(&lld(), &request, None).unwrap();
        assert!(!args(&cmd).contains(&"-m".to_string()), "{:?}", args(&cmd));
    }

    #[test]
    fn a_library_that_needs_an_os_is_an_error_naming_it() {
        let extra = ["-Tboard.ld".to_string()];
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
        let error = lld_command(&lld(), &request, None).unwrap_err();
        assert!(error.contains("`#library \"libc\"`"), "{error}");
        assert!(error.contains("no libc"), "{error}");
    }
}
