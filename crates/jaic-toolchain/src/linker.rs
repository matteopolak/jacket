//! The linker, archiver and sysroot for a target, in a fixed order. Each step finds the tool or
//! ends in a [`Missing`] that says how to get it; nothing switches to another toolchain quietly.
//!
//! 1. An override: [`Overrides`].
//! 2. The system tool that handles the target: `cc` for a host program, `wasm-ld`, `ld.lld`, the
//!    MinGW driver, Microsoft's linker.
//! 3. Another installed tool that does: the GNU cross driver named for the triple, else `clang`
//!    with `ld.lld` against a sysroot.
//! 4. The sysroot `jacket toolchain add` fetched.
//! 5. An error with a note.
use std::ffi::OsString;
use std::path::PathBuf;

use jaic::target::{Target, TargetAbi, TargetCpu, TargetOs};
use jaic::toolchain::{Linker, discover};

use crate::found::{Found, Missing, Origin};
use crate::host::{Host, Os};
use crate::lookup::Lookup;
use crate::overrides::{Given, Overrides};
use crate::package::{Need, PackageManager};
use crate::{cache, msvc, names, pins};

/// The target the programs of `host` run as (`aarch64-apple-darwin`): the MSVC environment on
/// Windows, glibc on Linux.
pub fn host_target(host: &dyn Host) -> Target {
    let os = match host.os() {
        Os::MacOs => TargetOs::MacOS,
        Os::Linux => TargetOs::Linux,
        Os::Windows => TargetOs::Windows,
    };
    Target::new(os, host.cpu())
}

fn missing(lookup: &Lookup, what: String, needs: &[Need]) -> Missing {
    let manager = PackageManager::detect(lookup);
    let os = lookup.host.os();

    Missing::new(
        what,
        needs.iter().map(|need| need.note(os, manager)).collect(),
    )
}

fn overridden(target: Target, given: &Given<OsString>) -> Found {
    let program = given.value.to_string_lossy().into_owned();
    let mut found = Found::new(PathBuf::from(&given.value), Origin::Override(given.from));

    // A linker named by hand gets the arguments its style expects, as one found by the lookup does.
    match target.toolchain().linker {
        Linker::Msvc if !names::is_msvc_linker(&program) => {
            found = found.with_args([format!("--target={target}")]);
        }
        Linker::Msvc => {
            if let Some(lib) = discover::lib_env(discover::arch_dir(target.cpu)) {
                found.env.push(("LIB".into(), lib));
            }
        }
        Linker::MinGw if names::is_clang(&program) => {
            found = found.with_args([format!("--target={target}")]);
        }
        _ => {}
    }

    found
}

/// The program that links an executable or a library for `target` (`None`: the host).
pub fn find_linker(
    lookup: &Lookup,
    target: Option<Target>,
    overrides: &Overrides,
) -> Result<Found, Missing> {
    let target = target.unwrap_or_else(|| host_target(lookup.host));

    if let Some(given) = &overrides.linker {
        return Ok(overridden(target, given));
    }

    let no_linker = || format!("no linker for {target}");

    if target.is_wasm() {
        return lookup
            .llvm_tool("wasm-ld")
            .map(|path| Found::new(path, Origin::System))
            .ok_or_else(|| missing(lookup, no_linker(), &[Need::Lld]));
    }

    if target.is_freestanding() {
        return lookup
            .llvm_tool("ld.lld")
            .map(|path| Found::new(path, Origin::System))
            .ok_or_else(|| missing(lookup, no_linker(), &[Need::Lld]));
    }

    match target.toolchain().linker {
        Linker::Unix => unix(lookup, target, overrides),
        Linker::MinGw => mingw(lookup, target),
        Linker::Msvc => msvc(lookup, target),
    }
}

fn unix(lookup: &Lookup, target: Target, overrides: &Overrides) -> Result<Found, Missing> {
    let host = host_target(lookup.host);
    let no_linker = || format!("no linker for {target}");

    if target.is_apple() {
        if lookup.host.os() != Os::MacOs {
            return Err(missing(lookup, no_linker(), &[Need::AppleSdk]));
        }

        let cc = lookup
            .which("cc")
            .ok_or_else(|| missing(lookup, no_linker(), &[Need::CDriver]))?;
        let found = Found::new(cc, Origin::System);

        // The other architecture of a Mac (run through Rosetta, and the reverse): Apple's `cc`
        // is clang and links either.
        return Ok(if target == host {
            found
        } else {
            found.with_args([format!("--target={target}")])
        });
    }

    if target.os != TargetOs::Linux {
        return Err(Missing::new(
            no_linker(),
            vec![format!(
                "jaic has no linker for {target} on {}",
                match lookup.host.os() {
                    Os::MacOs => "macOS",
                    Os::Linux => "Linux",
                    Os::Windows => "Windows",
                }
            )],
        ));
    }

    // The system C driver, when it builds for the target's CPU.
    if lookup.host.os() == Os::Linux
        && target.abi != TargetAbi::Musl
        && let Some(cc) = lookup.which("cc")
    {
        if target.cpu == host.cpu {
            return Ok(Found::new(cc, Origin::System));
        }

        // 32-bit x86 on a 64-bit machine: the C driver's multilib mode.
        if target.cpu == TargetCpu::X86 && host.cpu == TargetCpu::X64 {
            return Ok(Found::new(cc, Origin::System).with_args(["-m32"]));
        }
    }

    cross_linux(lookup, target, overrides)
}

/// A Linux target on a machine whose `cc` does not build for it.
fn cross_linux(lookup: &Lookup, target: Target, overrides: &Overrides) -> Result<Found, Missing> {
    if let Some(gnu) = pins::gnu_triple(target)
        && let Some(driver) = lookup.tool(&format!("{gnu}-gcc"))
    {
        return Ok(Found::new(driver, Origin::System));
    }

    // Apple's `cc` is clang without the backends for s390x and powerpc64, so it cannot build the
    // C sources a link brings (the crash runtime); LLVM's own clang can.
    let apple_clang_lacks_cpu = matches!(target.cpu, TargetCpu::S390x | TargetCpu::PowerPc64);
    let driver = if lookup.host.os() == Os::MacOs && !apple_clang_lacks_cpu {
        lookup.which("cc")
    } else {
        lookup.llvm_tool("clang")
    };
    let lld = lookup.llvm_tool("ld.lld");
    let sysroot = find_sysroot(lookup, target, overrides, SysrootUse::Link)?;

    if let (Some(driver), Some(lld), Some(sysroot)) = (&driver, &lld, &sysroot) {
        return Ok(Found::new(driver.clone(), Origin::System)
            .with_args([
                OsString::from(format!("--target={target}")),
                OsString::from(format!("--sysroot={}", sysroot.program.display())),
                OsString::from(format!("--ld-path={}", lld.display())),
                OsString::from("-fuse-ld=lld"),
            ])
            .with_sysroot(sysroot.clone()));
    }

    let manager = PackageManager::detect(lookup);
    let packaged = matches!(manager, Some(PackageManager::Apt | PackageManager::Dnf));
    let mut needs = Vec::new();

    if packaged && pins::gnu_triple(target).is_some() {
        needs.push(Need::CrossLinux {
            target,
        });
    } else {
        if driver.is_none() {
            needs.push(Need::Clang);
        }

        if lld.is_none() {
            needs.push(Need::Lld);
        }

        if sysroot.is_none() {
            needs.push(Need::CrossLinux {
                target,
            });
        }
    }

    let what = if driver.is_some() && lld.is_some() {
        format!("no Linux C library for {target}")
    } else {
        format!("no linker for {target}")
    };
    Err(missing(lookup, what, &needs))
}

/// What a sysroot is wanted for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SysrootUse {
    /// `--sysroot` of a link: it must hold the C library and the compiler's runtime files.
    Link,
    /// `-L` of a user-mode emulator that runs the program: the system's cross packages work too.
    Run,
}

/// The sysroot of a Linux `target`: the override, for a run the system's cross packages
/// (`/usr/<gnu-triple>`), then the one `jacket toolchain add` fetched. The returned [`Found`]'s
/// `program` is the folder.
pub fn find_sysroot(
    lookup: &Lookup,
    target: Target,
    overrides: &Overrides,
    purpose: SysrootUse,
) -> Result<Option<Found>, Missing> {
    if let Some(given) = &overrides.sysroot {
        if !lookup.host.is_dir(&given.value) {
            return Err(Missing::new(
                format!(
                    "{} names `{}`, which is not a folder",
                    given.from,
                    given.value.display()
                ),
                Vec::new(),
            ));
        }

        return Ok(Some(Found::new(
            given.value.clone(),
            Origin::Override(given.from),
        )));
    }

    if purpose == SysrootUse::Run
        && let Some(gnu) = pins::gnu_triple(target)
    {
        let system = PathBuf::from("/usr").join(gnu);

        if lookup.host.os() == Os::Linux && lookup.host.is_dir(&system.join("lib")) {
            return Ok(Some(Found::new(system, Origin::System)));
        }
    }

    Ok(cache::sysroot(lookup.host, target).map(|dir| Found::new(dir, Origin::Cache)))
}

fn mingw_cpu(target: Target) -> &'static str {
    match target.cpu {
        TargetCpu::Arm64 => "aarch64",
        _ => "x86_64",
    }
}

/// A Windows target linked with MinGW-w64's driver: `x86_64-w64-mingw32-gcc`, or llvm-mingw's
/// `-clang` (which GCC lacks for arm64); on a Windows machine also a plain `gcc` or `clang`.
fn mingw(lookup: &Lookup, target: Target) -> Result<Found, Missing> {
    let cpu = mingw_cpu(target);
    let mut names = vec![
        format!("{cpu}-w64-mingw32-gcc"),
        format!("{cpu}-w64-mingw32-clang"),
    ];

    if lookup.host.os() == Os::Windows {
        names.extend(["gcc".to_string(), "clang".to_string()]);
    }

    let Some(program) = names.iter().find_map(|name| lookup.tool(name)) else {
        return Err(missing(
            lookup,
            format!("no linker for {target}"),
            &[Need::Mingw {
                aarch64: cpu == "aarch64",
            }],
        ));
    };
    let found = Found::new(program.clone(), Origin::System);

    Ok(if names::is_clang(&program.to_string_lossy()) {
        found.with_args([format!("--target={target}")])
    } else {
        found
    })
}

/// A Windows target linked with Microsoft's toolchain. Microsoft's `link.exe` comes first: it is
/// the fastest on small debug builds (Clang's driver searches for Visual Studio and the SDK on
/// every run, then starts `link.exe` itself; `lld-link` is slower too). It is taken from the
/// Visual Studio installation (or the developer prompt) with `LIB` filled in when no prompt set
/// it, then from `PATH`, where a `link` that is not Microsoft's (the coreutils `link` of Git for
/// Windows and MSYS2, which stops with `extra operand`) is skipped. Without Visual Studio: Clang's
/// driver on `PATH`, `lld-link` on `PATH`, then a Clang installed elsewhere. See
/// `jaic/src/toolchain/discover.rs`, [`msvc`](mod@crate::msvc) and `docs/native/windows.md`.
fn msvc(lookup: &Lookup, target: Target) -> Result<Found, Missing> {
    let arch = discover::arch_dir(target.cpu);
    let triple = target.to_string();
    let clang = |program: PathBuf| {
        Found::new(program, Origin::System).with_args([format!("--target={triple}")])
    };
    let with_lib = |program: &std::path::Path| {
        let mut found = Found::new(program, Origin::System);

        if let Some(lib) = discover::lib_env(arch) {
            found.env.push(("LIB".into(), lib));
        }

        found
    };

    // No process is started to find this one; the `PATH` check below runs `link /?`.
    if let Some(tool) = discover::find_tool("link.exe", arch) {
        let mut found = Found::new(tool.program, Origin::System);

        found.env = tool.env;

        return Ok(found);
    }

    let mut skipped = Vec::new();

    for program in lookup.every_on_path("link") {
        if msvc::is_microsoft_linker(lookup.host, &program) {
            return Ok(with_lib(&program));
        }

        skipped.push(program);
    }

    if let Some(program) = lookup.which("clang") {
        return Ok(clang(program));
    }

    if let Some(program) = lookup.which("lld-link") {
        return Ok(with_lib(&program));
    }

    if let Some(program) = msvc::find_clang(lookup) {
        return Ok(clang(program));
    }

    let mut error = missing(lookup, format!("no linker for {target}"), &[Need::Msvc]);

    for program in skipped {
        error.notes.push(format!(
            "`{}` is not Microsoft's linker but the coreutils `link` (Git for Windows, MSYS2), so it was skipped",
            program.display()
        ));
    }

    Err(error)
}

/// The program that archives objects into a static library for `target` (`None`: the host): the
/// system `ar`; `llvm-ar` for WebAssembly, whose objects the system `ar` on macOS leaves without
/// a symbol index; for Windows a MinGW `ar`, `llvm-ar`, `llvm-lib`, `lib` or Visual Studio's
/// `lib.exe`.
pub fn find_archiver(
    lookup: &Lookup,
    target: Option<Target>,
    overrides: &Overrides,
) -> Result<Found, Missing> {
    let target = target.unwrap_or_else(|| host_target(lookup.host));

    if let Some(given) = &overrides.archiver {
        return Ok(Found::new(
            PathBuf::from(&given.value),
            Origin::Override(given.from),
        ));
    }

    let no_archiver = || format!("no archiver for {target}");
    let find = |names: &[String]| names.iter().find_map(|name| lookup.tool(name));
    let program = if target.is_wasm() {
        lookup.llvm_tool("llvm-ar")
    } else if target.toolchain().linker.is_windows() {
        let mingw = format!("{}-w64-mingw32-ar", mingw_cpu(target));

        find(&[mingw, "llvm-ar".into(), "llvm-lib".into(), "lib".into()]).or_else(|| {
            discover::find_tool("lib.exe", discover::arch_dir(target.cpu)).map(|tool| tool.program)
        })
    } else {
        lookup.which("ar")
    };
    let need = if target.is_wasm() || target.toolchain().linker.is_windows() {
        Need::Clang
    } else {
        Need::Archiver
    };

    program
        .map(|program| Found::new(program, Origin::System))
        .ok_or_else(|| missing(lookup, no_archiver(), &[need]))
}

/// The Clang driver that links a sanitized build. The instrumentation comes from the LLVM jaic
/// is built with and calls into a runtime of the same version, so that install's `clang` comes
/// before the system's (Apple's Clang ships an older runtime; GCC's `libasan` is a different
/// one).
pub fn find_sanitizer_driver(lookup: &Lookup, overrides: &Overrides) -> Result<Found, Missing> {
    if let Some(given) = &overrides.linker {
        return Ok(Found::new(
            PathBuf::from(&given.value),
            Origin::Override(given.from),
        ));
    }

    lookup
        .llvm_tool("clang")
        .map(|path| Found::new(path, Origin::System))
        .ok_or_else(|| {
            missing(
                lookup,
                "no clang to link a sanitized build".to_string(),
                &[Need::Clang],
            )
        })
}

#[cfg(test)]
mod tests;
