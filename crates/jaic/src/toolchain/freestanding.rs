//! Freestanding targets (`*-none-elf`): no C runtime and no libc. Programs link with `ld.lld` and
//! a linker script, plus the compiler-rt builtins for the exact target (64-bit division, soft
//! float and the like). See `docs/native/embedded.md`.
use super::{DataImport, Linker, Runtime, Toolchain};
use crate::target::Target;
use std::path::{Path, PathBuf};

pub(super) static FREESTANDING: Toolchain = Toolchain {
    linker: Linker::Unix,
    runtime: Runtime::NONE,
    // No library name means "the C runtime": there is none.
    runtime_names: &[],
    names_fold_case: false,
    implicit_libraries: &[],
    renames: |_| None,
    runtime_data: DataImport::Direct,
    main_stack: None,
    foreign_modules: None,
};

/// The file name compiler-rt gives its builtins for `target`: `libclang_rt.builtins-<arch>.a`.
fn builtins_file(target: Target) -> String {
    let arch = match target.cpu {
        crate::target::TargetCpu::Arm64 => "aarch64",
        crate::target::TargetCpu::Arm => match target.abi {
            crate::target::TargetAbi::CortexM(core) => core.compiler_rt_arch(),
            _ => "armhf",
        },
        crate::target::TargetCpu::X64 => "x86_64",
        crate::target::TargetCpu::X86 => "i386",
        crate::target::TargetCpu::S390x => "s390x",
        crate::target::TargetCpu::Wasm => "wasm",
    };
    format!("libclang_rt.builtins-{arch}.a")
}

/// The compiler-rt builtins archive for the exact freestanding `target`.
///
/// `JAIC_COMPILER_RT` names the file, or a folder holding `<triple>/libclang_rt.builtins.a` for
/// each target. Otherwise each LLVM install in `prefixes` is searched under
/// `lib/clang/<version>/lib/`: the target's own directory (`aarch64-none-elf/libclang_rt.builtins.a`,
/// where a `-DCOMPILER_RT_BAREMETAL_BUILD` build installs it) then `baremetal/`. A builtins
/// archive built for an operating system (`linux/`, `darwin/`) is never used: its code may call
/// into that OS.
pub fn compiler_rt_builtins(
    target: Target,
    explicit: Option<&Path>,
    prefixes: &[PathBuf],
) -> Option<PathBuf> {
    let own = target.to_string();
    if let Some(path) = explicit {
        if !path.is_dir() {
            return Some(path.to_path_buf());
        }
        // A folder of builtins, one per target as `tools/embedded/build-compiler-rt.sh` leaves
        // them: it serves every target it has an archive for.
        let file = path.join(&own).join("libclang_rt.builtins.a");
        if file.is_file() {
            return Some(file);
        }
    }
    for prefix in prefixes {
        let Ok(versions) = std::fs::read_dir(prefix.join("lib/clang")) else {
            continue;
        };
        let mut versions: Vec<PathBuf> = versions.flatten().map(|e| e.path()).collect();
        versions.sort();
        for version in versions.iter().rev() {
            let lib = version.join("lib");
            for candidate in [
                lib.join(&own).join("libclang_rt.builtins.a"),
                lib.join("baremetal").join(builtins_file(target)),
            ] {
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

/// How to get the builtins for `target`.
pub fn compiler_rt_install(target: Target) -> String {
    format!(
        "build compiler-rt's builtins for `{target}` (`tools/embedded/build-compiler-rt.sh`, or \
         LLVM's `-DCOMPILER_RT_BAREMETAL_BUILD=ON`) and install them under the LLVM's \
         `lib/clang/<version>/lib/{target}/`, or set JAIC_COMPILER_RT to the \
         `libclang_rt.builtins` archive"
    )
}

/// Whether linker output names an undefined compiler-rt builtin (`__udivti3`, `__addtf3`,
/// `__aarch64_ldadd4_acq_rel`): those come from the builtins archive, not from the program.
pub fn undefined_builtin(log: &str) -> bool {
    log.lines()
        .filter(|line| line.contains("undefined symbol"))
        .any(|line| {
            line.split("undefined symbol:")
                .nth(1)
                .is_some_and(|name| name.trim_start().starts_with("__"))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh directory removed when dropped (a test in jaic's `src/` cannot link `test_support`).
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Scratch {
            let dir = std::env::temp_dir()
                .join(format!("jaic-freestanding-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            Scratch(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn target() -> Target {
        Target::parse("aarch64-none-elf").unwrap()
    }

    fn install(root: &Path, relative: &str) -> PathBuf {
        let file = root.join("lib/clang/22/lib").join(relative);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, b"!<arch>\n").unwrap();
        file
    }

    #[test]
    fn builtins_come_from_the_targets_own_directory() {
        let dir = Scratch::new("own");
        let own = install(dir.path(), "aarch64-none-elf/libclang_rt.builtins.a");
        install(dir.path(), "baremetal/libclang_rt.builtins-aarch64.a");
        let found = compiler_rt_builtins(target(), None, &[dir.path().to_path_buf()]);
        assert_eq!(found, Some(own));
    }

    #[test]
    fn baremetal_builtins_are_found_by_architecture() {
        let dir = Scratch::new("baremetal");
        let file = install(dir.path(), "baremetal/libclang_rt.builtins-aarch64.a");
        let found = compiler_rt_builtins(target(), None, &[dir.path().to_path_buf()]);
        assert_eq!(found, Some(file));
    }

    #[test]
    fn an_explicit_folder_holds_one_archive_per_target() {
        let dir = Scratch::new("folder");
        let file = dir.path().join("aarch64-none-elf/libclang_rt.builtins.a");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, b"!<arch>\n").unwrap();
        assert_eq!(
            compiler_rt_builtins(target(), Some(dir.path()), &[]),
            Some(file)
        );
        let other = Target::parse("thumbv7em-none-eabihf").unwrap();
        assert_eq!(compiler_rt_builtins(other, Some(dir.path()), &[]), None);
    }

    #[test]
    fn builtins_built_for_an_operating_system_are_never_used() {
        let dir = Scratch::new("os");
        install(dir.path(), "linux/libclang_rt.builtins-aarch64.a");
        install(dir.path(), "darwin/libclang_rt.builtins_osx.a");
        assert_eq!(
            compiler_rt_builtins(target(), None, &[dir.path().to_path_buf()]),
            None
        );
    }

    #[test]
    fn an_explicit_archive_wins() {
        let dir = Scratch::new("explicit");
        install(dir.path(), "aarch64-none-elf/libclang_rt.builtins.a");
        let explicit = Path::new("/elsewhere/builtins.a");
        assert_eq!(
            compiler_rt_builtins(target(), Some(explicit), &[dir.path().to_path_buf()]),
            Some(explicit.to_path_buf())
        );
    }

    #[test]
    fn undefined_compiler_rt_symbols_are_told_apart_from_the_programs() {
        assert!(undefined_builtin(
            "ld.lld: error: undefined symbol: __udivti3\n>>> referenced by a.o"
        ));
        assert!(!undefined_builtin(
            "ld.lld: error: undefined symbol: uart_write"
        ));
    }
}
