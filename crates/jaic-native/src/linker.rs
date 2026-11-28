//! Finding and running linkers, archives, output names and library arguments.
use super::link::{LinkArg, toolchain_for};
use super::toolchain::{command, lookup};
use jaic::ir::Library;
use jaic::target::Target as BuildTarget;
use jaic::toolchain::{Linker, Toolchain};
use jaic_toolchain::{Found, Origin, Overrides};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// A non-system `#library` with a static archive or object next to the declaring file
/// (`name.a`, `libname.a`, `name.o`), for the targets that link no shared libraries: wasm and
/// freestanding.
pub(crate) fn static_library_file(lib: &Library) -> Option<PathBuf> {
    if lib.system {
        return None;
    }

    let path = Path::new(&lib.name);
    let file = path.file_name()?.to_string_lossy().into_owned();
    let dir = Path::new(&lib.base_dir).join(path.parent().unwrap_or(Path::new("")));

    [
        format!("{file}.a"),
        format!("lib{file}.a"),
        format!("{file}.o"),
    ]
    .into_iter()
    .map(|name| dir.join(name))
    .find(|p| p.is_file())
}

/// Runs a tool and collects its output. `Command::output` gives the child the null device as stdin,
/// and a Windows `AppContainer` may not open it, so the child gets an empty pipe instead.
pub(crate) fn run_captured(cmd: &mut Command) -> std::io::Result<Output> {
    cmd.stdin(Stdio::piped()).output()
}

/// The error for a linker that could not be started. The lookup only returns files it saw, so
/// this is a named linker that is gone or cannot run.
pub fn linker_not_run(linker: &Found, e: &std::io::Error) -> String {
    let mut text = format!(
        "could not run the linker `{}`: {e}",
        linker.program.display()
    );

    if let Origin::Override(from) = linker.origin {
        text += &format!("\nnote: {from} names it");
    }

    text +=
        "\nnote: `jaic file.jai -interp` needs no linker: it runs the program in the interpreter";
    text
}

/// The error for a link that failed: the linker's own output, then what usually causes it.
pub fn link_failure(program: &std::path::Path, out: &std::process::Output) -> String {
    let program = jaic::display_path(program);
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

/// The error for a build with debug information on a machine with no `dsymutil`.
fn missing_dsymutil_message(lookup: &jaic_toolchain::Lookup) -> String {
    use jaic_toolchain::package::{Need, PackageManager};
    format!(
        "`dsymutil` is not installed, and a build with debug information on macOS needs it\nnote: {}, or build with `-no_debug_info`",
        Need::Dsymutil.note(lookup.host.os(), PackageManager::detect(lookup))
    )
}

/// `dsymutil`, or the error for it missing. Checked before compiling, so a build that will write
/// a dSYM does not fail after the compile and link.
pub fn require_dsymutil() -> Result<PathBuf, String> {
    let lookup = lookup()?;

    lookup
        .llvm_tool("dsymutil")
        .ok_or_else(|| missing_dsymutil_message(&lookup))
}

/// Collect the DWARF of a linked macOS executable or library into `output.dSYM`.
///
/// Apple's linker does not copy debug information into its output: the binary only
/// records which object files hold it, and the CLI deletes those objects. `dsymutil`
/// gathers it into a bundle next to the binary, where lldb finds it by UUID.
pub fn write_dsym(output: &Path) -> Result<(), String> {
    let mut bundle = output.to_path_buf().into_os_string();

    bundle.push(".dSYM");
    let dsymutil = require_dsymutil()?;
    let out =
        run_captured(Command::new(dsymutil).arg(output).arg("-o").arg(&bundle)).map_err(|e| {
            format!(
                "could not run `dsymutil` for debug information: {}",
                jaic::io_reason(&e)
            )
        })?;

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

/// Archive object files into a static library for `target` with the archiver the lookup finds
/// (`JAIC_AR` names one).
pub fn archive(
    objects: &[PathBuf],
    output: &Path,
    target: Option<BuildTarget>,
) -> Result<(), String> {
    let found = jaic_toolchain::linker::find_archiver(&lookup()?, target, &Overrides::from_env())
        .map_err(|missing| missing.to_string())?;
    let program = found.program.display().to_string();
    let mut cmd = command(&found);

    if jaic_toolchain::names::is_lib_tool(&found.program) {
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
/// `helpers`: the compiler's own helper libraries do ([`uses_compiler_helpers`]). `cross_dirs`:
/// where libraries built for the target are.
///
/// [`uses_compiler_helpers`]: crate::link::uses_compiler_helpers
pub(super) fn library_args(
    lib: &Library,
    toolchain: &Toolchain,
    cross: bool,
    helpers: bool,
    cross_dirs: &[PathBuf],
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

        // The compiler's own copy of the crash runtime is built for the host. A cross build
        // hands the source to the target's C driver, which compiles it as it links.
        if cross && linker == Linker::Unix && file == "portable_crash" {
            let source = dir.join("runtime.c");

            if source.exists() {
                return Ok(vec![
                    LinkArg::Option("-funwind-tables".into()),
                    LinkArg::File(source.display().to_string()),
                ]);
            }
        }

        // Not beside the source: a helper the compiler provides (the crash runtime) is found
        // by file name in the library directories, as the interpreter does.
        if name.contains('/') && helpers && linker == Linker::Unix {
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
            // The driver knows why a library it supplies is missing (an unwritable cache).
            let why = jaic::interp::missing_library(name)
                .map(|note| format!("\n{note}"))
                .unwrap_or_default();

            return Err(format!(
                "library '{name}' not found: looked for {} in {}{elsewhere}{why}",
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

    // A cross build links third-party libraries built for its target
    // (`tools/native/build_native_libs.py --platform windows-x64-mingw`) from the target's own
    // `native-libs` folder, never the host's, whose archives are for the host.
    if cross {
        let bare = name.strip_prefix("lib").unwrap_or(name);

        for dir in cross_dirs {
            let archive = dir.join(format!("lib{bare}.a"));

            if archive.exists() {
                return Ok(vec![LinkArg::File(archive.display().to_string())]);
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

#[cfg(test)]
mod dsymutil_tests {
    use super::*;
    use jaic_toolchain::Lookup;
    use jaic_toolchain::testing::FakeHost;

    #[test]
    fn a_mac_without_dsymutil_is_told_what_to_install_and_how_to_skip_debug_info() {
        let host = FakeHost::mac();
        let message = missing_dsymutil_message(&Lookup::new(&host));

        assert!(
            message.starts_with("`dsymutil` is not installed"),
            "{message}"
        );
        assert!(
            message.contains("\nnote: install the Xcode command line tools"),
            "{message}"
        );
        assert!(message.contains("-no_debug_info"), "{message}");
    }
}
