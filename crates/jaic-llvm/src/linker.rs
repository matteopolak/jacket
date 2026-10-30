//! Finding and running linkers, archives, output names and library arguments.
use super::link::{LinkArg, toolchain_for};
use super::{link_options, wasm};
use jaic::ir::Library;
use jaic::msvc;
use jaic::target::Target as BuildTarget;
use jaic::toolchain::{Linker, Toolchain};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The error for a linker that could not be started, with how to get one.
/// The error for a linker that could not be started; `install` says how to get one.
pub fn linker_not_run(program: &str, e: &std::io::Error, install: &str) -> String {
    if e.kind() != std::io::ErrorKind::NotFound {
        return format!("could not run the linker `{program}`: {e}");
    }
    format!(
        "could not find the linker `{program}`, which jaic uses to write executables\n\
         help: {install}\n\
         help: `jaic file.jai -interp` needs no linker: it runs the program in the interpreter"
    )
}

/// The error for a link that failed: the linker's own output, then what usually causes it.
pub fn link_failure(program: &str, out: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let log = format!("{stdout}{stderr}");
    let mut text = format!(
        "linking failed: `{program}` stopped with {}\n{}",
        out.status,
        log.trim_end()
    );
    let missing_symbol = [
        "Undefined symbols",
        "undefined reference",
        "unresolved external",
    ]
    .iter()
    .any(|m| log.contains(m));
    let missing_library = [
        "library not found",
        "cannot find -l",
        "unable to find library",
    ]
    .iter()
    .any(|m| log.contains(m));
    if missing_library {
        text += "\nhelp: a `#library` or `#system_library` names a library the linker cannot find: \
                 check its path, or install the library";
    } else if missing_symbol {
        text += "\nhelp: a `#foreign` procedure is not in any linked library: check its name and \
                 that its `#library` is the one that defines it";
    }
    text
}

/// `link.exe`/`lld-link` arguments that keep the objects' `CodeView` in a PDB named after the
/// output. The objects are deleted after linking, so without `/DEBUG` the debug information is
/// lost. `/DEBUG` would otherwise also turn on incremental linking (an `.ilk` file and padded
/// code) and keep unreferenced functions; both are turned back off to match a build without it.
pub(super) fn pdb_args(output: &Path) -> [String; 4] {
    [
        "/DEBUG".to_string(),
        format!("/PDB:{}", output.with_extension("pdb").display()),
        "/INCREMENTAL:NO".to_string(),
        "/OPT:REF".to_string(),
    ]
}

/// Collect the DWARF of a linked macOS executable or library into `output.dSYM`.
///
/// Apple's linker does not copy debug information into its output: the binary only
/// records which object files hold it, and the CLI deletes those objects. `dsymutil`
/// gathers it into a bundle next to the binary, where lldb finds it by UUID.
pub fn write_dsym(output: &Path) -> Result<(), String> {
    let mut bundle = output.to_path_buf().into_os_string();
    bundle.push(".dSYM");
    let out = Command::new("dsymutil")
        .arg(output)
        .arg("-o")
        .arg(&bundle)
        .output()
        .map_err(|e| format!("could not run 'dsymutil' for debug information: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "dsymutil failed ({}):\n{}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

/// Archive object files into a static library for `target`: the system `ar` on macOS and
/// Linux; for Windows a MinGW `ar`, `llvm-ar` or `lib.exe` (`JAIC_AR` overrides the choice).
pub fn archive(
    objects: &[PathBuf],
    output: &Path,
    target: Option<BuildTarget>,
) -> Result<(), String> {
    let program = match jacket_config::env::AR.get() {
        Some(program) => program,
        // The system `ar` on macOS writes no symbol index for wasm objects.
        None if target.is_some_and(jaic::target::Target::is_wasm) => {
            wasm::find_llvm_tool("llvm-ar")
                .ok_or("no llvm-ar found for a WebAssembly archive; set JAIC_AR")?
        }
        None if toolchain_for(target).linker.is_windows() => {
            let mingw_ar = format!("{}-w64-mingw32-ar", mingw_cpu(target));
            let arch = msvc::arch_dir(target.unwrap_or_else(BuildTarget::host).cpu);
            find_program(&[&mingw_ar, "llvm-ar", "llvm-lib", "lib"])
                // Visual Studio's lib.exe when no developer prompt put it on PATH.
                .or_else(|| {
                    msvc::find_tool("lib.exe", arch)
                        .map(|tool| tool.program.to_string_lossy().into_owned())
                })
                .ok_or("no archiver for Windows libraries found: install LLVM or mingw-w64")?
        }
        None => "ar".to_string(),
    };
    let stem = Path::new(&program)
        .file_stem()
        .map(|s| s.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    let mut cmd = Command::new(&program);
    if matches!(stem.as_str(), "lib" | "llvm-lib") {
        cmd.arg("/NOLOGO").arg(format!("/OUT:{}", output.display()));
    } else {
        // `ar` appends to an existing archive; start from an empty one.
        let _ = std::fs::remove_file(output);
        cmd.arg("rcs").arg(output);
    }
    let status = cmd
        .args(objects)
        .status()
        .map_err(|e| format!("could not run '{program}': {e}"))?;
    status
        .success()
        .then_some(())
        .ok_or(format!("{program} failed ({status})"))
}

pub(super) fn render_link_arg(cmd: &mut Command, arg: &LinkArg, msvc_style: bool) {
    match (arg, msvc_style) {
        (LinkArg::Lib(name), false) => cmd.arg(format!("-l{name}")),
        (LinkArg::Lib(name), true) => cmd.arg(format!("{name}.lib")),
        (LinkArg::File(path), _) => cmd.arg(path),
        (LinkArg::Framework(name), _) => cmd.arg("-framework").arg(name),
        (LinkArg::Rpath(dir), _) => cmd.arg(format!("-Wl,-rpath,{dir}")),
        (LinkArg::SearchDir(dir), false) => cmd.arg(format!("-L{dir}")),
        (LinkArg::SearchDir(dir), true) => cmd.arg(format!("/LIBPATH:{dir}")),
        (LinkArg::Option(option), _) => cmd.arg(option),
    };
}

/// Whether `program` takes `link.exe`-style arguments rather than a C compiler driver's.
fn is_msvc_linker(program: &str) -> bool {
    let stem = Path::new(program)
        .file_stem()
        .map(|s| s.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    matches!(stem.as_str(), "link" | "lld-link")
}

/// Sanitized builds run only on the host, and only on macOS and Linux: the runtimes come from
/// the host's LLVM install, and jaic does not drive Clang's Windows or wasm sanitizer setups.
pub fn check_sanitizer_target(target: Option<BuildTarget>) -> Result<(), String> {
    if target.is_some() {
        return Err("-sanitize supports only native builds for the host, not cross builds".into());
    }
    if !cfg!(any(target_os = "macos", target_os = "linux")) {
        return Err("-sanitize is supported on macOS and Linux hosts only".into());
    }
    Ok(())
}

/// The Clang driver that links a sanitized build. The instrumentation comes from the LLVM jaic
/// is built with and calls into a runtime of the same version, so prefer that install's
/// `clang` over the system `cc` (Apple's Clang ships an older runtime; GCC's `libasan` is a
/// different one). `JAIC_SANITIZER_CC` overrides the choice.
pub(super) fn sanitizer_driver() -> String {
    if let Some(program) = jacket_config::env::SANITIZER_CC.get() {
        return program;
    }
    let prefixes = [
        jacket_config::env::LLVM_SYS_231_PREFIX.get(),
        option_env!("LLVM_SYS_231_PREFIX").map(str::to_string),
    ];
    for prefix in prefixes.into_iter().flatten() {
        let clang = Path::new(&prefix).join("bin/clang");
        if clang.is_file() {
            return clang.to_string_lossy().into_owned();
        }
    }
    for config in ["llvm-config-23", "llvm-config"] {
        let Ok(out) = Command::new(config).arg("--bindir").output() else {
            continue;
        };
        let clang = Path::new(String::from_utf8_lossy(&out.stdout).trim()).join("clang");
        if out.status.success() && clang.is_file() {
            return clang.to_string_lossy().into_owned();
        }
    }
    find_program(&["clang-23", "clang"]).unwrap_or_else(|| "clang".into())
}

/// The first of `names` found on `PATH`.
fn find_program(names: &[&str]) -> Option<String> {
    let path = jacket_config::env::PATH.os()?;
    for name in names {
        for dir in std::env::split_paths(&path) {
            if dir.join(name).is_file() || dir.join(format!("{name}.exe")).is_file() {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// The linker program and its base command. `JAIC_LINKER` overrides the choice: a program
/// named `link` or `lld-link` gets MSVC-style arguments, anything else C-driver arguments.
pub(super) fn linker_command(
    linker: Linker,
    target: Option<BuildTarget>,
) -> Result<(String, Command), String> {
    let resolved = target.unwrap_or_else(BuildTarget::host);
    let msvc_triple = resolved.to_string();
    // An empty value (PowerShell 7.5 passes `$env:JAIC_LINKER = ''` on) means unset.
    if let Some(program) = jacket_config::env::LINKER.os().filter(|v| !v.is_empty()) {
        let program = program.to_string_lossy().into_owned();
        let mut cmd = Command::new(&program);
        if linker == Linker::Msvc {
            if !is_msvc_linker(&program) {
                cmd.arg(format!("--target={msvc_triple}"));
            } else if let Some(lib) = msvc::lib_env(msvc::arch_dir(resolved.cpu)) {
                // As for a `link.exe` jaic finds itself: without a developer prompt nothing
                // else tells it where `user32.lib` and the C runtime are.
                cmd.env("LIB", lib);
            }
        } else if linker == Linker::MinGw && link_options::is_clang(&program) {
            cmd.arg(format!("--target={msvc_triple}"));
        }
        return Ok((program, cmd));
    }
    match linker {
        Linker::Unix => {
            let mut cmd = Command::new("cc");
            // The other architecture of a Mac (`x86_64-apple-darwin` on arm64, run through
            // Rosetta, and the reverse): Apple's `cc` is clang and links either.
            if let Some(target) = target
                && cfg!(target_os = "macos")
                && target.is_apple()
            {
                cmd.arg(format!("--target={target}"));
            }
            Ok(("cc".into(), cmd))
        }
        Linker::MinGw => {
            // GCC only targets x64; llvm-mingw provides `-clang` (and a `-gcc` alias of it)
            // for both CPUs.
            let cpu = mingw_cpu(target);
            let gcc = format!("{cpu}-w64-mingw32-gcc");
            let clang = format!("{cpu}-w64-mingw32-clang");
            let mut names = vec![gcc.as_str(), clang.as_str()];
            if cfg!(windows) {
                names.extend(["gcc", "clang"]);
            }
            let program = find_program(&names).ok_or_else(|| {
                let install = if cpu == "aarch64" {
                    "install llvm-mingw (it provides aarch64-w64-mingw32-clang)"
                } else {
                    "install mingw-w64 (it provides x86_64-w64-mingw32-gcc)"
                };
                format!(
                    "no MinGW-w64 linker found to link for Windows\nhelp: {install}, or name a linker with JAIC_LINKER"
                )
            })?;
            let mut cmd = Command::new(&program);
            if program.ends_with("clang") {
                cmd.arg(format!("--target={resolved}"));
            }
            Ok((program, cmd))
        }
        Linker::Msvc => msvc_linker_command(resolved),
    }
}

/// The linker for an MSVC target. Microsoft's `link.exe` comes first: it is the fastest of the
/// three on small debug builds (Clang's driver searches for Visual Studio and the SDK on every
/// run and then starts `link.exe` itself; `lld-link` is slower too). It is taken from the
/// Visual Studio installation (or the developer prompt) with `LIB` filled in when no prompt set
/// it, then from `PATH`, where a `link` that is not Microsoft's (the coreutils `link` of Git for
/// Windows and MSYS2, which stops with `extra operand`) is skipped. Without Visual Studio: Clang's
/// driver on `PATH`, `lld-link` on `PATH`, then a Clang installed elsewhere. See `msvc.rs` and
/// `docs/native/windows.md`.
fn msvc_linker_command(target: BuildTarget) -> Result<(String, Command), String> {
    let arch = msvc::arch_dir(target.cpu);
    let triple = target.to_string();
    let clang = |program: String| {
        let mut cmd = Command::new(&program);
        cmd.arg(format!("--target={triple}"));
        (program, cmd)
    };
    let with_env = |program: &Path, env: Vec<(std::ffi::OsString, std::ffi::OsString)>| {
        let program = program.to_string_lossy().into_owned();
        let mut cmd = Command::new(&program);
        cmd.envs(env);
        (program, cmd)
    };
    let with_lib = |program: &Path, lib: Option<std::ffi::OsString>| {
        with_env(
            program,
            lib.map(|lib| ("LIB".into(), lib)).into_iter().collect(),
        )
    };
    // No process is started to find this one; the `PATH` check below runs `link /?`.
    if let Some(tool) = msvc::find_tool("link.exe", arch) {
        return Ok(with_env(&tool.program, tool.env));
    }
    let mut skipped = Vec::new();
    for program in msvc::on_path("link") {
        if msvc::is_microsoft_linker(&program) {
            return Ok(with_lib(&program, msvc::lib_env(arch)));
        }
        skipped.push(program);
    }
    if let Some(program) = find_program(&["clang"]) {
        return Ok(clang(program));
    }
    if let Some(program) = msvc::on_path("lld-link").first() {
        return Ok(with_lib(program, msvc::lib_env(arch)));
    }
    if let Some(program) = msvc::find_clang() {
        return Ok(clang(program.to_string_lossy().into_owned()));
    }
    let mut message = String::from("no MSVC linker found to link for Windows");
    for program in skipped {
        message += &format!(
            "\nnote: `{}` is not Microsoft's linker but the coreutils `link` (Git for Windows, MSYS2), so it was skipped",
            program.display()
        );
    }
    message += "\nhelp: install LLVM (clang), or the Visual Studio build tools with the \"Desktop development with C++\" workload, or name a linker with JAIC_LINKER";
    Err(message)
}

/// The CPU prefix of the MinGW-w64 tool names for `target` (`None`: the host).
pub(super) fn mingw_cpu(target: Option<BuildTarget>) -> &'static str {
    match target.unwrap_or_else(BuildTarget::host).cpu {
        jaic::target::TargetCpu::Arm64 => "aarch64",
        _ => "x86_64",
    }
}

/// What a native build writes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OutputKind {
    Executable,
    DynamicLibrary,
    StaticLibrary,
}

/// The file name extension an output gets on `target` when its name has none, where the
/// platform expects one: `exe`, `dll` and `lib` on Windows, `wasm` (or `a`) for WebAssembly.
pub fn output_extension(target: Option<BuildTarget>, kind: OutputKind) -> Option<&'static str> {
    if target.is_some_and(jaic::target::Target::is_wasm) {
        return Some(match kind {
            OutputKind::StaticLibrary => "a",
            _ => "wasm",
        });
    }
    if !toolchain_for(target).linker.is_windows() {
        return None;
    }
    Some(match kind {
        OutputKind::Executable => "exe",
        OutputKind::DynamicLibrary => "dll",
        OutputKind::StaticLibrary => "lib",
    })
}

/// Linker inputs for one Jai library reference. `cross`: the target is not the host, so the
/// host's library directories (Homebrew, native-libs builds, frameworks) do not apply.
pub(super) fn library_args(
    lib: &Library,
    toolchain: &Toolchain,
    cross: bool,
) -> Result<Vec<LinkArg>, String> {
    let name = lib.name.as_str();
    // The C runtime is linked by the toolchain itself (see `link`); on Windows the POSIX-only
    // names are not libraries either.
    if toolchain.is_runtime(name) {
        return Ok(Vec::new());
    }
    let linker = toolchain.linker;
    if !lib.system {
        // A library shipped next to the source: link it by path when found.
        let path = Path::new(name);
        let file = path
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default();
        let dir = Path::new(&lib.base_dir).join(path.parent().unwrap_or(Path::new("")));
        // Like `jai`, a static archive wins over a shared library.
        let candidates: Vec<String> = match linker {
            Linker::Unix => {
                let ext = if cfg!(target_os = "macos") {
                    "dylib"
                } else {
                    "so"
                };
                vec![
                    format!("{file}.a"),
                    format!("lib{file}.a"),
                    format!("{file}.{ext}"),
                    format!("lib{file}.{ext}"),
                ]
            }
            // `name.lib` is a static library or a DLL's import library; MinGW's `ld` also
            // reads GNU archives and links against a DLL directly.
            Linker::MinGw => vec![
                format!("{file}.lib"),
                format!("{file}.a"),
                format!("lib{file}.a"),
                format!("{file}.dll.a"),
                format!("lib{file}.dll.a"),
                format!("{file}.dll"),
            ],
            Linker::Msvc => vec![format!("{file}.lib"), format!("lib{file}.lib")],
        };
        for candidate in &candidates {
            let full = dir.join(candidate);
            if !full.exists() {
                continue;
            }
            let mut args = vec![LinkArg::File(full.display().to_string())];
            if candidate.ends_with(".dylib") || candidate.ends_with(".so") {
                let parent = full.parent().unwrap_or(Path::new("."));
                args.push(LinkArg::Rpath(parent.display().to_string()));
            }
            return Ok(args);
        }
        // Not beside the source: a helper the compiler provides (the crash runtime) is found
        // by file name in the library directories, as the interpreter does.
        if name.contains('/') && !cross && linker == Linker::Unix {
            for lib_dir in jaic::interp::library_dirs() {
                for candidate in &candidates {
                    let full = lib_dir.join(candidate);
                    if !full.exists() {
                        continue;
                    }
                    let mut args = vec![LinkArg::File(full.display().to_string())];
                    if candidate.ends_with(".dylib") || candidate.ends_with(".so") {
                        args.push(LinkArg::Rpath(lib_dir.display().to_string()));
                    }
                    return Ok(args);
                }
            }
        }
        // A path is never a system library name: report it instead of a confusing `-l`.
        if name.contains('/') {
            let searched = jaic::interp::library_dirs();
            let elsewhere = if searched.is_empty() {
                String::new()
            } else {
                format!(
                    " and in {}",
                    searched
                        .iter()
                        .map(|dir| dir.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            return Err(format!(
                "library '{name}' not found: looked for {} in {}{elsewhere}",
                candidates.join(", "),
                dir.display()
            ));
        }
    }
    // Built third-party libraries (`tools/native/build_native_libs.py`: `freetype.lib`...) link
    // statically, as on macOS and Linux below.
    if linker == Linker::Msvc && !cross {
        let bare = name.strip_prefix("lib").unwrap_or(name);
        for dir in jaic::interp::library_dirs() {
            let archive = dir.join(format!("{bare}.lib"));
            if archive.exists() {
                return Ok(vec![LinkArg::File(archive.display().to_string())]);
            }
        }
    }
    // A MinGW cross build links third-party libraries cross-built for it
    // (`tools/native/build_native_libs.py --platform windows-<cpu>-mingw`) from the directories in
    // `JAIC_CROSS_LIBS`, not from the host's native-libs directory, whose archives are for the host.
    if linker == Linker::MinGw && cross {
        let bare = name.strip_prefix("lib").unwrap_or(name);
        if let Some(dirs) = jacket_config::env::CROSS_LIBS.os() {
            for dir in std::env::split_paths(&dirs) {
                let archive = dir.join(format!("lib{bare}.a"));
                if archive.exists() {
                    return Ok(vec![LinkArg::File(archive.display().to_string())]);
                }
            }
        }
    }
    if linker.is_windows() {
        // Import libraries of system DLLs (`kernel32`, `user32`...) come with the toolchain.
        // Windows file names ignore case, but a MinGW toolchain on Linux or macOS has
        // `libgdi32.a`, not `libGdi32.a`, and its names are all lowercase.
        let mut args: Vec<LinkArg> = jaic::interp::extra_library_dirs()
            .iter()
            .map(|d| LinkArg::SearchDir(d.display().to_string()))
            .collect();
        args.push(LinkArg::Lib(name.to_ascii_lowercase()));
        return Ok(args);
    }
    // Apple frameworks (`AppKit`, `Metal`...) link with `-framework`; their directories exist on
    // disk even though the binaries live in the shared cache.
    if cfg!(target_os = "macos")
        && !cross
        && Path::new(&format!("/System/Library/Frameworks/{name}.framework")).exists()
    {
        return Ok(vec![LinkArg::Framework(name.to_string())]);
    }
    // Jai names libraries either way (`"libobjc"` / `"objc"`); `-l` wants the bare name.
    let name = name.strip_prefix("lib").unwrap_or(name);
    // Built third-party libraries link statically, so the executable is self-contained.
    if !cross {
        for dir in jaic::interp::library_dirs() {
            let archive = dir.join(format!("lib{name}.a"));
            if archive.exists() {
                return Ok(vec![LinkArg::File(archive.display().to_string())]);
            }
        }
    }
    // Directories a metaprogram added (`compiler_add_library_search_directory`).
    let mut args: Vec<LinkArg> = jaic::interp::extra_library_dirs()
        .iter()
        .map(|d| LinkArg::SearchDir(d.display().to_string()))
        .collect();
    let homebrew = jaic::interp::homebrew_lib_dir();
    if cfg!(target_os = "macos") && !cross && Path::new(homebrew).exists() {
        args.push(LinkArg::SearchDir(homebrew.to_string()));
    }
    args.push(LinkArg::Lib(name.to_string()));
    Ok(args)
}
