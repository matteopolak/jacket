//! The compiler builtins a freestanding program links: 64-bit division, soft float, atomics
//! outlines. The order is fixed:
//!
//! 1. An override: `JAIC_COMPILER_RT` names the archive.
//! 2. A compiler-rt archive built for the exact target in an LLVM install, under
//!    `lib/clang/<version>/lib/`: the target's own folder (where a
//!    `-DCOMPILER_RT_BAREMETAL_BUILD` build installs it), then `baremetal/`. An archive built
//!    for an operating system (`linux/`, `darwin/`) is never used: its code may call into it.
//! 3. The `libgcc.a` of an installed GNU embedded toolchain (`arm-none-eabi-gcc`, `avr-gcc`),
//!    asked for the core the program is built for.
//! 4. The builtins `jacket toolchain add` built from compiler-rt's source for the exact target.
//! 5. An error that says how to get them.
use std::path::PathBuf;

use jaic::target::{Target, TargetAbi, TargetCpu};

use crate::cache;
use crate::found::{Found, Missing, Origin};
use crate::lookup::Lookup;
use crate::overrides::Overrides;
use crate::package::{Need, PackageManager};

/// The file name compiler-rt gives its builtins for `target`: `libclang_rt.builtins-<arch>.a`.
fn builtins_file(target: Target) -> String {
    let arch = match target.cpu {
        TargetCpu::Arm64 => "aarch64",
        TargetCpu::Arm => match target.abi {
            TargetAbi::CortexM(core) => core.compiler_rt_arch(),
            _ => "armhf",
        },
        TargetCpu::X64 => "x86_64",
        TargetCpu::X86 => "i386",
        TargetCpu::Avr => "avr",
        TargetCpu::S390x => "s390x",
        TargetCpu::PowerPc64 => "powerpc64",
        TargetCpu::Wasm => "wasm",
    };
    format!("libclang_rt.builtins-{arch}.a")
}

/// The builtins archive for the freestanding `target`.
pub fn find_builtins(
    lookup: &Lookup,
    target: Target,
    overrides: &Overrides,
) -> Result<Found, Missing> {
    if let Some(given) = &overrides.compiler_rt {
        return if lookup.host.is_file(&given.value) {
            Ok(Found::new(
                given.value.clone(),
                Origin::Override(given.from),
            ))
        } else {
            Err(missing(
                lookup,
                target,
                format!(
                    "{} names `{}`, which does not exist",
                    given.from,
                    given.value.display()
                ),
            ))
        };
    }

    if let Some(found) = llvm_install(lookup, target) {
        return Ok(found);
    }

    if let Some(found) = gnu_libgcc(lookup, target) {
        return Ok(found);
    }

    if let Some((file, source)) = cache::builtins(lookup.host, target) {
        return Ok(Found::new(file, Origin::Cache).with_detail(format!("{source} (cached)")));
    }

    Err(missing(lookup, target, format!("no builtins for {target}")))
}

fn missing(lookup: &Lookup, target: Target, what: String) -> Missing {
    let note = Need::Builtins {
        target,
    }
    .note(lookup.host.os(), PackageManager::detect(lookup));
    Missing::new(what, vec![note])
}

/// compiler-rt built for `target` in one of the LLVM installs, newest version first.
fn llvm_install(lookup: &Lookup, target: Target) -> Option<Found> {
    let own = target.to_string();

    for prefix in lookup.llvm_prefixes() {
        let mut versions = lookup.host.read_dir(&prefix.join("lib/clang"));

        versions.sort();

        for version in versions.iter().rev() {
            let lib = version.join("lib");
            let candidates: [PathBuf; 2] = [
                lib.join(&own).join("libclang_rt.builtins.a"),
                lib.join("baremetal").join(builtins_file(target)),
            ];

            if let Some(file) = candidates.into_iter().find(|c| lookup.host.is_file(c)) {
                let name = version
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();

                return Some(
                    Found::new(file, Origin::System).with_detail(format!("compiler-rt {name}")),
                );
            }
        }
    }

    None
}

/// The core `avr-gcc` is asked for: the only AVR core jaic builds for.
const AVR_CORE: &str = "atmega328p";

/// The `libgcc.a` of an installed GNU embedded toolchain, in the multilib for the core `target`
/// is built for (soft float: jaic's freestanding programs call the soft-float helpers).
fn gnu_libgcc(lookup: &Lookup, target: Target) -> Option<Found> {
    let (driver, flags): (&str, Vec<String>) = match (target.cpu, target.abi) {
        (TargetCpu::Arm, TargetAbi::CortexM(core)) => (
            "arm-none-eabi-gcc",
            vec![
                format!("-mcpu={}", core.llvm_cpu()),
                "-mthumb".into(),
                "-mfloat-abi=soft".into(),
            ],
        ),
        (TargetCpu::Avr, _) => ("avr-gcc", vec![format!("-mmcu={AVR_CORE}")]),
        _ => return None,
    };
    let program = lookup.tool(driver)?;
    let mut args: Vec<&str> = flags.iter().map(String::as_str).collect();

    args.push("-print-libgcc-file-name");
    let printed = lookup.host.run(&program, &args)?;
    let file = PathBuf::from(printed.trim());

    if !lookup.host.is_file(&file) {
        return None;
    }

    let version = Found::new(&program, Origin::System).version(lookup);
    let detail = match version {
        Some(version) => format!("libgcc from {driver} {version}"),
        None => format!("libgcc from {driver}"),
    };

    Some(Found::new(file, Origin::System).with_detail(detail))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::overrides::Given;
    use crate::testing::FakeHost;

    fn target() -> Target {
        Target::parse("aarch64-none-elf").unwrap()
    }

    fn lookup(host: &FakeHost) -> Lookup<'_> {
        Lookup::new(host).with_llvm_prefixes(|| vec!["/llvm".into()])
    }

    #[test]
    fn builtins_come_from_the_targets_own_directory() {
        let mut host = FakeHost::mac();

        host.file("/llvm/lib/clang/22/lib/aarch64-none-elf/libclang_rt.builtins.a");
        host.file("/llvm/lib/clang/22/lib/baremetal/libclang_rt.builtins-aarch64.a");
        let found = find_builtins(&lookup(&host), target(), &Overrides::default()).unwrap();

        assert_eq!(
            found.program,
            PathBuf::from("/llvm/lib/clang/22/lib/aarch64-none-elf/libclang_rt.builtins.a")
        );
        assert_eq!(found.detail.as_deref(), Some("compiler-rt 22"));
    }

    #[test]
    fn baremetal_builtins_are_found_by_architecture_and_the_newest_version_wins() {
        let mut host = FakeHost::mac();

        host.file("/llvm/lib/clang/21/lib/baremetal/libclang_rt.builtins-aarch64.a");
        host.file("/llvm/lib/clang/22/lib/baremetal/libclang_rt.builtins-aarch64.a");
        let found = find_builtins(&lookup(&host), target(), &Overrides::default()).unwrap();

        assert!(found.program.starts_with("/llvm/lib/clang/22"), "{found:?}");
    }

    #[test]
    fn builtins_built_for_an_operating_system_are_never_used() {
        let mut host = FakeHost::mac();

        host.file("/llvm/lib/clang/22/lib/linux/libclang_rt.builtins-aarch64.a");
        host.file("/llvm/lib/clang/22/lib/darwin/libclang_rt.builtins_osx.a");
        let error = find_builtins(&lookup(&host), target(), &Overrides::default()).unwrap_err();

        assert_eq!(error.what, "no builtins for aarch64-none-elf");
        assert_eq!(
            error.notes,
            ["run `jacket toolchain add aarch64-none-elf` to build its builtins"]
        );
    }

    #[test]
    fn an_override_wins_and_a_missing_one_is_an_error_naming_it() {
        let mut host = FakeHost::mac();

        host.file("/llvm/lib/clang/22/lib/aarch64-none-elf/libclang_rt.builtins.a");
        host.file("/mine/rt.a");
        let given = |path: &str| Overrides {
            compiler_rt: Some(Given {
                value: path.into(),
                from: "JAIC_COMPILER_RT",
            }),
            ..Overrides::default()
        };
        let found = find_builtins(&lookup(&host), target(), &given("/mine/rt.a")).unwrap();

        assert_eq!(found.program, PathBuf::from("/mine/rt.a"));
        assert_eq!(found.origin, Origin::Override("JAIC_COMPILER_RT"));
        let error = find_builtins(&lookup(&host), target(), &given("/mine/none.a")).unwrap_err();

        assert_eq!(
            error.what,
            "JAIC_COMPILER_RT names `/mine/none.a`, which does not exist"
        );
    }

    #[test]
    fn cortex_m_archives_are_named_for_the_core() {
        let mut host = FakeHost::mac();

        host.file("/llvm/lib/clang/22/lib/baremetal/libclang_rt.builtins-armv7em.a");
        let m4 = Target::parse("thumbv7em-none-eabihf").unwrap();

        assert!(find_builtins(&lookup(&host), m4, &Overrides::default()).is_ok());
        let m33 = Target::parse("thumbv8m.main-none-eabihf").unwrap();

        assert!(find_builtins(&lookup(&host), m33, &Overrides::default()).is_err());
    }

    #[test]
    fn an_installed_gnu_toolchain_supplies_libgcc_for_the_core() {
        let mut host = FakeHost::mac();

        host.file("/opt/homebrew/bin/arm-none-eabi-gcc");
        host.file("/gnu/thumb/v7e-m/nofp/libgcc.a");
        host.prints(
            "/opt/homebrew/bin/arm-none-eabi-gcc",
            &[
                "-mcpu=cortex-m4",
                "-mthumb",
                "-mfloat-abi=soft",
                "-print-libgcc-file-name",
            ],
            "/gnu/thumb/v7e-m/nofp/libgcc.a\n",
        );
        host.prints(
            "/opt/homebrew/bin/arm-none-eabi-gcc",
            &["--version"],
            "arm-none-eabi-gcc (Arm GNU Toolchain) 14.2.1 20241119\n",
        );
        let m4 = Target::parse("thumbv7em-none-eabihf").unwrap();
        let found = find_builtins(&lookup(&host), m4, &Overrides::default()).unwrap();

        assert_eq!(
            found.program,
            PathBuf::from("/gnu/thumb/v7e-m/nofp/libgcc.a")
        );
        assert!(
            found
                .detail
                .as_deref()
                .is_some_and(|d| d.starts_with("libgcc from arm-none-eabi-gcc")),
            "{found:?}"
        );
        // The other core has no archive printed for it: nothing is guessed.
        let m33 = Target::parse("thumbv8m.main-none-eabihf").unwrap();

        assert!(find_builtins(&lookup(&host), m33, &Overrides::default()).is_err());
    }

    #[test]
    fn the_llvm_install_comes_before_the_gnu_toolchain() {
        let mut host = FakeHost::mac();

        host.file("/llvm/lib/clang/22/lib/baremetal/libclang_rt.builtins-armv7em.a");
        host.file("/opt/homebrew/bin/arm-none-eabi-gcc");
        let m4 = Target::parse("thumbv7em-none-eabihf").unwrap();
        let found = find_builtins(&lookup(&host), m4, &Overrides::default()).unwrap();

        assert!(
            found
                .detail
                .as_deref()
                .is_some_and(|d| d.starts_with("compiler-rt")),
            "{found:?}"
        );
    }

    #[test]
    fn avr_asks_avr_gcc_for_its_core() {
        let mut host = FakeHost::mac();

        host.file("/opt/homebrew/bin/avr-gcc");
        host.file("/avr/libgcc.a");
        host.prints(
            "/opt/homebrew/bin/avr-gcc",
            &["-mmcu=atmega328p", "-print-libgcc-file-name"],
            "/avr/libgcc.a\n",
        );
        let avr = Target::parse("avr-none-elf").unwrap();
        let found = find_builtins(&lookup(&host), avr, &Overrides::default()).unwrap();

        assert_eq!(found.program, PathBuf::from("/avr/libgcc.a"));
    }
}
