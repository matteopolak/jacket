//! Link flavors, the link driver and its inputs.
use super::link_options;
use super::linker::{
    check_sanitizer_target, library_args, link_failure, linker_command, linker_not_run, pdb_args,
    render_link_arg, sanitizer_driver,
};
use super::machine::fuzzer_runtime;
use super::options::Sanitize;
use jaic::ir::Library;
use jaic::target::{Target as BuildTarget, TargetAbi};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The linker family a target uses.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LinkFlavor {
    /// macOS and Linux: the system `cc` driver (Clang or GCC).
    Unix,
    /// Windows with the MinGW-w64 runtime (`x86_64-pc-windows-gnu`): a MinGW GCC or Clang
    /// driver. This is what cross builds from macOS and Linux use.
    MinGw,
    /// Windows with the Microsoft toolchain (`x86_64-pc-windows-msvc`, the default on a
    /// Windows host): Clang's driver, `lld-link` or `link.exe`.
    Msvc,
}

impl LinkFlavor {
    /// The flavor for a target (`None`: the host).
    pub fn for_target(target: Option<BuildTarget>) -> LinkFlavor {
        let target = target.unwrap_or_else(BuildTarget::host);
        match (target.os, target.abi) {
            (jaic::sema::TargetOs::Windows, TargetAbi::Gnu) => LinkFlavor::MinGw,
            (jaic::sema::TargetOs::Windows, _) => LinkFlavor::Msvc,
            _ => LinkFlavor::Unix,
        }
    }

    pub fn is_windows(self) -> bool {
        self != LinkFlavor::Unix
    }
}

/// One linker input, rendered per linker style.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) enum LinkArg {
    /// A library by name (`-lname`, `name.lib`).
    Lib(String),
    /// A library file by path.
    File(String),
    /// An Apple framework.
    Framework(String),
    /// Runtime search directory for a shared library next to the source.
    Rpath(String),
    /// Link-time search directory.
    SearchDir(String),
    /// An option for the linker driver, passed as it is.
    Option(String),
}

/// Puts the library files (built archives such as `libstb_vorbis.a`) ahead of libraries linked
/// by name, keeping each kind's order. GNU `ld` resolves an archive's references only against
/// the inputs after it, and with `--as-needed` (Ubuntu's default) a `-lm` that comes first is
/// dropped before the archive needs it; system libraries never need our archives.
pub(super) fn order_link_groups(groups: &mut [Vec<LinkArg>]) {
    groups.sort_by_key(|group| !group.iter().any(|arg| matches!(arg, LinkArg::File(_))));
}

/// `library_args`, kept in the link when nothing references the library: with `--as-needed`
/// (Ubuntu's default) the linker drops a shared library no symbol needs, and a
/// `#library,link_always` one exists for its constructor.
fn library_group(lib: &Library, flavor: LinkFlavor, cross: bool) -> Result<Vec<LinkArg>, String> {
    let mut args = library_args(lib, flavor, cross)?;
    if lib.link_always
        && flavor == LinkFlavor::Unix
        && !args.is_empty()
        && !cfg!(target_os = "macos")
    {
        args.insert(0, LinkArg::Option("-Wl,--push-state,--no-as-needed".into()));
        args.push(LinkArg::Option("-Wl,--pop-state".into()));
    }
    Ok(args)
}

/// Link object files into an executable (or shared library) for `target` (`None`: the
/// host). macOS and Linux use the system `cc`; Windows uses a MinGW or MSVC toolchain,
/// see [`LinkFlavor`] and `docs/native/windows.md`. With `debug_info`, MSVC targets also get
/// a PDB next to the output (`foo.exe` -> `foo.pdb`).
#[allow(clippy::too_many_arguments)]
pub fn link(
    objects: &[PathBuf],
    libraries: &[Library],
    output: &Path,
    dynamic_library: bool,
    extra_args: &[String],
    target: Option<BuildTarget>,
    sanitize: Sanitize,
    debug_info: bool,
) -> Result<(), String> {
    let flavor = LinkFlavor::for_target(target);
    let cross = target.is_some() && flavor != LinkFlavor::for_target(None);
    if sanitize.any() {
        check_sanitizer_target(target)?;
    }
    // Each library's argument group is added once (`-framework X` is two arguments).
    let mut seen: Vec<Vec<LinkArg>> = Vec::new();
    for lib in libraries {
        let args = library_group(lib, flavor, cross)?;
        if !args.is_empty() && !seen.contains(&args) {
            seen.push(args);
        }
    }
    // LLVM lowers float rounding (`llvm.trunc`, `floor`, `ceil`, `round`, from `#asm` and
    // casts) to libm calls on CPUs without SSE4.1 (baseline x86-64), so libm is always linked;
    // on macOS it is part of libSystem and `-lm` is a no-op.
    let libm = vec![LinkArg::Lib("m".into())];
    if flavor == LinkFlavor::Unix && !seen.contains(&libm) {
        seen.push(libm);
    }
    order_link_groups(&mut seen);
    let (program, mut cmd) = if sanitize.any() {
        let program = sanitizer_driver();
        let mut cmd = Command::new(&program);
        cmd.args(sanitize.driver_flag());
        if sanitize.fuzzer {
            cmd.arg(fuzzer_runtime(&program)?);
            cmd.arg(if cfg!(target_os = "macos") {
                "-lc++"
            } else {
                "-lstdc++"
            });
        }
        (program, cmd)
    } else {
        linker_command(flavor, target)?
    };
    let mut coverage_source = None;
    let driver = link_options::Driver::for_command(&program, extra_args);
    let extra_args = link_options::render(extra_args, flavor, driver)?;
    let msvc_style = driver != link_options::Driver::C;
    if driver == link_options::Driver::ClangCl {
        // CL's /link consumes everything after it. Keep compiler options and
        // actual driver inputs before the delimiter; the rest uses LINK syntax.
        cmd.args(objects)
            .arg("/MD")
            .args(&extra_args.driver)
            .arg("/link");
    }
    if msvc_style {
        cmd.arg("/NOLOGO").arg(format!("/OUT:{}", output.display()));
        if driver == link_options::Driver::MsvcLink {
            cmd.args(objects);
        }
        cmd.arg(if dynamic_library {
            "/DLL"
        } else {
            "/SUBSYSTEM:CONSOLE"
        });
        // The dynamic CRT; `oldnames` maps POSIX names (`write`) to the CRT's underscored
        // ones, `legacy_stdio_definitions` keeps `printf` and friends linkable as functions.
        cmd.args([
            "/DEFAULTLIB:msvcrt",
            "/DEFAULTLIB:oldnames",
            "/DEFAULTLIB:legacy_stdio_definitions",
        ]);
    } else {
        if dynamic_library {
            cmd.arg("-shared");
        }
        cmd.args(objects).arg("-o").arg(output);
        if sanitize.coverage {
            let source =
                std::env::temp_dir().join(format!("jaic-coverage-{}.c", std::process::id()));
            std::fs::write(&source, COVERAGE_RUNTIME)
                .map_err(|e| format!("could not write `{}`: {e}", source.display()))?;
            cmd.args(["-w", "-O1"]).arg(&source);
            coverage_source = Some(source);
        }
        if flavor == LinkFlavor::Msvc {
            // Clang's MSVC driver: the dynamic CRT, as above.
            cmd.args(["-fms-runtime-lib=dll", "-llegacy_stdio_definitions"]);
        }
    }
    // Windows reserves 1 MiB for the main thread's stack (and threads created without a
    // size); reserve the 8 MiB macOS and Linux give, so deep recursion behaves the same.
    if flavor.is_windows() && !dynamic_library {
        cmd.arg(match (msvc_style, flavor) {
            (true, _) => "/STACK:8388608",
            (false, LinkFlavor::Msvc) => "-Wl,/STACK:8388608",
            _ => "-Wl,--stack,8388608",
        });
    }
    if debug_info && flavor == LinkFlavor::Msvc {
        let prefix = if msvc_style {
            ""
        } else {
            "-Wl,"
        };
        cmd.args(pdb_args(output).iter().map(|a| format!("{prefix}{a}")));
    }
    for arg in seen.concat() {
        render_link_arg(&mut cmd, &arg, msvc_style);
    }
    if driver == link_options::Driver::C {
        cmd.args(extra_args.driver);
    } else {
        cmd.args(extra_args.linker);
    }
    let install = if cfg!(target_os = "macos") {
        "install the Xcode command line tools (`xcode-select --install`), or name a linker with the JAIC_LINKER environment variable"
    } else if cfg!(windows) {
        "install LLVM (clang) or the Visual Studio build tools, or name a linker with the JAIC_LINKER environment variable"
    } else {
        "install a C compiler (clang or gcc, e.g. `apt install clang`), or name a linker with the JAIC_LINKER environment variable"
    };
    let out = cmd.output();
    if let Some(source) = coverage_source {
        let _ = std::fs::remove_file(source);
    }
    let out = out.map_err(|e| linker_not_run(&program, &e, install))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(link_failure(&program, &out))
    }
}

/// The C runtime a `-coverage` build links (`runtime/coverage.c`).
const COVERAGE_RUNTIME: &str = include_str!("../runtime/coverage.c");

/// The libraries a program links, as linker arguments for a metaprogram that links it itself
/// (`use_custom_link_command`, `Message_Phase.system_libraries` and `.user_libraries`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LinkInputs {
    /// Libraries linked by name: `-lm`, `-framework Metal` (two arguments), `user32.lib`.
    pub system_libraries: Vec<String>,
    /// Library files by path, with the search and run-time directories they need.
    pub user_libraries: Vec<String>,
}

/// The C runtime libraries an MSVC link names by default (`msvcrt` brings in `vcruntime` and `ucrt`
/// through its own directives, which `/NODEFAULTLIB` would drop).
const MSVC_RUNTIME_LIBRARIES: &[&str] = &[
    "msvcrt.lib",
    "vcruntime.lib",
    "ucrt.lib",
    "oldnames.lib",
    "legacy_stdio_definitions.lib",
    "legacy_stdio_wide_specifiers.lib",
];

/// [`LinkInputs`] for `libraries` on `target` (`None`: the host), rendered as [`link`] would
/// pass them: `cc`-style arguments, or `name.lib` for an MSVC target.
pub fn link_inputs(
    libraries: &[Library],
    target: Option<BuildTarget>,
) -> Result<LinkInputs, String> {
    let flavor = LinkFlavor::for_target(target);
    let cross = target.is_some() && flavor != LinkFlavor::for_target(None);
    let mut seen: Vec<Vec<LinkArg>> = Vec::new();
    for lib in libraries {
        let args = library_group(lib, flavor, cross)?;
        if !args.is_empty() && !seen.contains(&args) {
            seen.push(args);
        }
    }
    order_link_groups(&mut seen);
    let mut inputs = LinkInputs::default();
    for arg in seen.concat() {
        match arg {
            LinkArg::Lib(name) if flavor == LinkFlavor::Msvc => {
                inputs.system_libraries.push(format!("{name}.lib"))
            }
            LinkArg::Lib(name) => inputs.system_libraries.push(format!("-l{name}")),
            LinkArg::Framework(name) => {
                inputs.system_libraries.push("-framework".into());
                inputs.system_libraries.push(name);
            }
            LinkArg::File(path) => inputs.user_libraries.push(path),
            LinkArg::Rpath(dir) => inputs.user_libraries.push(format!("-Wl,-rpath,{dir}")),
            LinkArg::SearchDir(dir) if flavor == LinkFlavor::Msvc => {
                inputs.user_libraries.push(format!("/LIBPATH:{dir}"))
            }
            LinkArg::SearchDir(dir) => inputs.user_libraries.push(format!("-L{dir}")),
            LinkArg::Option(option) => inputs.user_libraries.push(option),
        }
    }
    if flavor == LinkFlavor::Msvc {
        // What `link` adds with /DEFAULTLIB. A custom link command often passes /NODEFAULTLIB
        // (to swap in the static runtime), which then needs every one of them named.
        for name in MSVC_RUNTIME_LIBRARIES {
            if !inputs.system_libraries.iter().any(|l| l == name) {
                inputs.system_libraries.push((*name).to_string());
            }
        }
    }
    Ok(inputs)
}
