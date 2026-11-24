//! Linking freestanding programs (`*-none-elf`) with `ld.lld` (see `docs/native/embedded.md`).
//!
//! There is no C driver, libc or startup file: the program defines its own entry point, and a
//! linker script places its sections. `ld.lld` is the only linker used; if it is missing the
//! build fails with a note that says how to get it.
use crate::toolchain::{command, lookup};
use jaic::ir::Library;
use jaic::target::{Target, TargetCpu};
use jaic::toolchain::freestanding;
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
    let lld = find_linker(&lookup(), Some(link.target), &Overrides::from_env())
        .map_err(|missing| missing.to_string())?;
    let explicit = jacket_config::env::COMPILER_RT.os().map(PathBuf::from);
    if let Some(path) = explicit.as_deref().filter(|path| !path.exists()) {
        return Err(format!(
            "JAIC_COMPILER_RT names `{}`, which does not exist\nhelp: {}",
            path.display(),
            freestanding::compiler_rt_install(link.target)
        ));
    }
    let prefixes = jacket_config::llvm::prefixes(crate::BAKED_LLVM_PREFIX);
    let builtins = freestanding::compiler_rt_builtins(link.target, explicit.as_deref(), &prefixes);
    let mut cmd = lld_command(&lld, link, builtins.as_deref())?;
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
    if builtins.is_none() && freestanding::undefined_builtin(&log) {
        message += &format!(
            "\nhelp: the program needs compiler-rt builtins: {}",
            freestanding::compiler_rt_install(link.target)
        );
    }
    Err(message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jaic_toolchain::Origin;

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
