//! The link driver and its inputs.
use super::link_options;
use super::linker::{
    check_sanitizer_target, library_args, link_failure, linker_not_run, pdb_args, render_link_arg,
};
use super::options::Sanitize;
use super::toolchain::{command, lookup, report};
use jaic::ir::Library;
use jaic::target::Target as BuildTarget;
use jaic::toolchain::{Driver, Linker, Toolchain};
use jaic_toolchain::linker::{find_linker, find_sanitizer_driver};
use jaic_toolchain::{Origin, Overrides};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The toolchain of `target` (`None`: the host).
pub(super) fn toolchain_for(target: Option<BuildTarget>) -> &'static Toolchain {
    target.unwrap_or_else(BuildTarget::host).toolchain()
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

/// Whether a build for `target` (`None`: the host) links the helper libraries the compiler
/// carries (`native/portable_watch`), which are found by file name in its library folders.
/// They are built for the host, and on a Mac as universal binaries, so any Mac target can use
/// them (crates/jaic-cli/build.rs).
pub(crate) fn uses_compiler_helpers(target: Option<BuildTarget>, host: BuildTarget) -> bool {
    target.is_none_or(|t| t == host || (t.is_apple() && host.is_apple()))
}

/// `library_args`, kept in the link when nothing references the library: with `--as-needed`
/// (Ubuntu's default) the linker drops a shared library no symbol needs, and a
/// `#library,link_always` one exists for its constructor.
fn library_group(
    lib: &Library,
    toolchain: &Toolchain,
    cross: bool,
    helpers: bool,
    cross_dirs: &[PathBuf],
) -> Result<Vec<LinkArg>, String> {
    let mut args = library_args(lib, toolchain, cross, helpers, cross_dirs)?;
    if lib.link_always
        && toolchain.linker == Linker::Unix
        && !args.is_empty()
        && !cfg!(target_os = "macos")
    {
        args.insert(0, LinkArg::Option("-Wl,--push-state,--no-as-needed".into()));
        args.push(LinkArg::Option("-Wl,--pop-state".into()));
    }
    Ok(args)
}

/// What [`link`] joins into one output.
#[derive(Clone, Copy)]
pub struct LinkRequest<'a> {
    pub objects: &'a [PathBuf],
    pub libraries: &'a [Library],
    pub output: &'a Path,
    /// Produce a shared library rather than an executable.
    pub dynamic_library: bool,
    pub extra_args: &'a [String],
    /// `None`: the host.
    pub target: Option<BuildTarget>,
    pub sanitize: Sanitize,
    pub debug_info: bool,
    /// Where libraries built for `target` are, for a cross build.
    pub cross_library_dirs: &'a [PathBuf],
}

/// Link object files into an executable (or shared library) for `target` (`None`: the
/// host). macOS and Linux use the system `cc`; Windows uses a MinGW or MSVC toolchain,
/// see [`Toolchain`] and `docs/native/windows.md`. With `debug_info`, MSVC targets also get
/// a PDB next to the output (`foo.exe` -> `foo.pdb`).
pub fn link(request: &LinkRequest) -> Result<(), String> {
    let LinkRequest {
        objects,
        libraries,
        output,
        dynamic_library,
        extra_args,
        target,
        sanitize,
        debug_info,
        cross_library_dirs,
    } = *request;
    let toolchain = toolchain_for(target);
    let linker = toolchain.linker;
    let cross = target.is_some_and(|t| t != BuildTarget::host());
    let helpers = uses_compiler_helpers(target, BuildTarget::host());
    if sanitize.any() {
        check_sanitizer_target(target)?;
    }
    // Each library's argument group is added once (`-framework X` is two arguments).
    let mut seen: Vec<Vec<LinkArg>> = Vec::new();
    for lib in libraries {
        let args = library_group(lib, toolchain, cross, helpers, cross_library_dirs)?;
        if !args.is_empty() && !seen.contains(&args) {
            seen.push(args);
        }
    }
    // LLVM lowers float rounding (`llvm.trunc`, `floor`, `ceil`, `round`, from `#asm` and
    // casts) to libm calls on CPUs without SSE4.1 (baseline x86-64), so libm is always linked;
    // on macOS it is part of libSystem and `-lm` is a no-op.
    for name in toolchain.implicit_libraries {
        let implicit = vec![LinkArg::Lib((*name).into())];
        if !seen.contains(&implicit) {
            seen.push(implicit);
        }
    }
    order_link_groups(&mut seen);
    let overrides = Overrides::from_env();
    let lookup = lookup();
    let (linker_found, mut cmd) = if sanitize.any() {
        let found = find_sanitizer_driver(&lookup, &overrides).map_err(|m| m.to_string())?;
        let program = found.program.display().to_string();
        let mut cmd = command(&found);
        cmd.args(sanitize.driver_flag());
        if sanitize.fuzzer {
            cmd.arg(fuzzer_runtime(&program)?);
            cmd.arg(if cfg!(target_os = "macos") {
                "-lc++"
            } else {
                "-lstdc++"
            });
        }
        (found, cmd)
    } else {
        let found = find_linker(&lookup, target, &overrides).map_err(|m| m.to_string())?;
        let cmd = command(&found);
        (found, cmd)
    };
    report("Linker", &linker_found, &lookup, target);
    let program = linker_found.program.display().to_string();
    let mut coverage_source = None;
    let driver = link_options::driver_for_command(&program, extra_args);
    let apple = target.unwrap_or_else(BuildTarget::host).is_apple();
    let extra_args = link_options::render(
        &link_options::translate(extra_args, linker, driver, apple),
        linker,
        driver,
    )?;
    let msvc_style = driver != Driver::C;
    let runtime = toolchain.runtime_args(driver);
    if driver == Driver::ClangCl {
        // CL's /link consumes everything after it. Keep compiler options and
        // actual driver inputs before the delimiter; the rest uses LINK syntax.
        cmd.args(objects)
            .args(&runtime.compile)
            .args(&extra_args.driver)
            .arg("/link");
    }
    if msvc_style {
        cmd.arg("/NOLOGO").arg(format!("/OUT:{}", output.display()));
        if driver == Driver::MsvcLink {
            cmd.args(objects);
        }
        cmd.arg(if dynamic_library {
            "/DLL"
        } else {
            "/SUBSYSTEM:CONSOLE"
        });
        cmd.args(&runtime.link);
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
        cmd.args(&runtime.link);
    }
    if !dynamic_library && let Some(stack) = toolchain.stack_arg(driver) {
        cmd.arg(stack);
    }
    if debug_info && linker == Linker::Msvc {
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
    if driver == Driver::C {
        cmd.args(extra_args.driver);
    } else {
        cmd.args(extra_args.linker);
    }
    #[cfg(target_os = "macos")]
    if linker_found
        .program
        .file_name()
        .is_some_and(|name| name == "cc")
        && linker_found.origin == Origin::System
        && target.is_none()
        && !dynamic_library
        && !sanitize.any()
        && coverage_source.is_none()
    {
        let args: Option<Vec<String>> = cmd
            .get_args()
            .map(|arg| arg.to_str().map(String::from))
            .collect();
        // The linker alone, without the driver; on any failure `cc` runs the same link, so
        // its message and its rules decide what is an error.
        if let Some(mut direct) = args.and_then(|args| crate::apple_ld::direct_command(&args))
            && let Ok(out) = crate::linker::run_captured(&mut direct)
            && out.status.success()
        {
            return Ok(());
        }
    }
    let out = crate::linker::run_captured(&mut cmd);
    if let Some(source) = coverage_source {
        let _ = std::fs::remove_file(source);
    }
    let out = out.map_err(|e| linker_not_run(&linker_found, &e))?;
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

/// [`LinkInputs`] for `libraries` on `target` (`None`: the host), rendered as [`link`] would
/// pass them: `cc`-style arguments, or `name.lib` for an MSVC target.
pub fn link_inputs(
    libraries: &[Library],
    target: Option<BuildTarget>,
    cross_dirs: &[PathBuf],
) -> Result<LinkInputs, String> {
    let toolchain = toolchain_for(target);
    let linker = toolchain.linker;
    let cross = target.is_some_and(|t| t != BuildTarget::host());
    let helpers = uses_compiler_helpers(target, BuildTarget::host());
    let mut seen: Vec<Vec<LinkArg>> = Vec::new();
    for lib in libraries {
        let args = library_group(lib, toolchain, cross, helpers, cross_dirs)?;
        if !args.is_empty() && !seen.contains(&args) {
            seen.push(args);
        }
    }
    order_link_groups(&mut seen);
    let mut inputs = LinkInputs::default();
    for arg in seen.concat() {
        match arg {
            LinkArg::Lib(name) if linker == Linker::Msvc => {
                inputs.system_libraries.push(format!("{name}.lib"));
            }
            LinkArg::Lib(name) => inputs.system_libraries.push(format!("-l{name}")),
            LinkArg::Framework(name) => {
                inputs.system_libraries.push("-framework".into());
                inputs.system_libraries.push(name);
            }
            LinkArg::File(path) => inputs.user_libraries.push(path),
            LinkArg::Rpath(dir) => inputs.user_libraries.push(format!("-Wl,-rpath,{dir}")),
            LinkArg::SearchDir(dir) if linker == Linker::Msvc => {
                inputs.user_libraries.push(format!("/LIBPATH:{dir}"));
            }
            LinkArg::SearchDir(dir) => inputs.user_libraries.push(format!("-L{dir}")),
            LinkArg::Option(option) => inputs.user_libraries.push(option),
        }
    }
    // What `link` adds with /DEFAULTLIB. A custom link command often passes /NODEFAULTLIB
    // (to swap in the static runtime), which then needs every one of them named.
    for name in toolchain.runtime_inputs() {
        if !inputs.system_libraries.contains(&name) {
            inputs.system_libraries.push(name);
        }
    }
    Ok(inputs)
}

/// The libFuzzer engine without its `main`, found through the sanitizer driver (`clang
/// --print-file-name`), or a message saying what to install.
fn fuzzer_runtime(driver: &str) -> Result<PathBuf, String> {
    let arch = std::env::consts::ARCH;
    let name = if cfg!(target_os = "macos") {
        "libclang_rt.fuzzer_no_main_osx.a".to_string()
    } else {
        format!("libclang_rt.fuzzer_no_main-{arch}.a")
    };
    let out =
        crate::linker::run_captured(Command::new(driver).arg(format!("--print-file-name={name}")))
            .map_err(|e| format!("could not run `{driver}` to find {name}: {e}"))?;
    let path = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
    if path.is_absolute() && path.is_file() {
        Ok(path)
    } else {
        Err(format!(
            "`-sanitize fuzzer` needs {name} from the LLVM that built jaic (`{driver}` did not find it)\nhelp: install compiler-rt (Homebrew `llvm`, or `libclang-rt-23-dev` from apt.llvm.org) so that `clang` finds it"
        ))
    }
}
