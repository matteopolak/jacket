//! Native output without LLVM.
//!
//! The build options code generation takes ([`Options`]), the system linker ([`link`],
//! [`archive`], [`link_wasm`]) and the loader for the code generation backend ([`backend`]).
//! `jaic-llvm` implements the backend; a driver that links it directly never uses the loader,
//! and one that does not link it opens `libjaic_backend` the first time a build needs it.
pub mod backend;
mod link;
mod link_options;
mod linker;
#[cfg(target_os = "macos")]
mod malloc_zone;
mod options;
mod wasm;

use jaic::ir::{Library, Program};

/// The LLVM prefix this build was made against (`build.rs`), searched after the environment.
pub(crate) const BAKED_LLVM_PREFIX: Option<&str> = option_env!("LLVM_PREFIX_AT_BUILD");

pub use self::link::{LinkInputs, LinkRequest, link, link_inputs};
pub use self::linker::{
    OutputKind, archive, check_sanitizer_target, link_failure, linker_not_run, output_extension,
    write_dsym,
};
pub use self::options::{Codegen, OptLevel, Options, SANCOV_PASS, Sanitize};
pub use self::wasm::{WasmLink, find_wasm_ld, link_wasm};

/// Keep `fork` working when the process's C allocator is mimalloc's macOS zone
/// (the `override` feature of mimalloc); call once at startup. Does nothing elsewhere.
pub fn fix_malloc_zone_for_fork() {
    #[cfg(target_os = "macos")]
    malloc_zone::fix_fork();
}

/// The libraries from `program.libraries` that some foreign symbol uses, or that are
/// `link_always`.
pub fn used_libraries(program: &Program) -> Vec<Library> {
    let mut used: Vec<bool> = program.libraries.iter().map(|l| l.link_always).collect();
    for foreign in &program.foreigns {
        if let Some(i) = foreign.library.filter(|&i| i < used.len()) {
            used[i] = true;
        }
    }
    program
        .libraries
        .iter()
        .zip(used)
        .filter(|&(_, u)| u)
        .map(|(l, _)| l.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::link::{LinkArg, order_link_groups, toolchain_for};
    use crate::linker::{library_args, mingw_cpu, render_link_arg};
    use jaic::target::Target as BuildTarget;
    use jaic::target::TargetCpu;
    use jaic::toolchain::Linker;
    use std::process::Command;

    #[test]
    fn stamp_names_the_sources_and_layouts() {
        let stamp = backend::stamp();
        assert!(stamp.starts_with(env!("CARGO_PKG_VERSION")), "{stamp}");
        assert!(stamp.contains("/rustc-"), "{stamp}");
    }

    #[test]
    fn sanitizer_lists_parse_into_flags_and_passes() {
        let both = Sanitize::parse("address,undefined").unwrap();
        assert!(both.address && both.undefined);
        assert_eq!(
            both.driver_flag().as_deref(),
            Some("-fsanitize=address,undefined")
        );
        assert_eq!(
            both.passes(OptLevel::O2),
            ["function(bounds-checking<rt-abort>)", "asan"]
        );
        // Unoptimized code needs its slots promoted before bounds checks see any object.
        assert_eq!(
            Sanitize::parse("undefined").unwrap().passes(OptLevel::O0),
            ["function(sroa,bounds-checking<rt-abort>)"]
        );
        assert_eq!(
            Sanitize::parse("address").unwrap().driver_flag().as_deref(),
            Some("-fsanitize=address")
        );
        let coverage = Sanitize::parse("coverage").unwrap();
        assert_eq!(coverage.driver_flag(), None);
        assert_eq!(coverage.passes(OptLevel::O0), ["sancov-module"]);
        let fuzz = Sanitize::parse("address,fuzzer").unwrap();
        assert!(fuzz.any() && fuzz.fuzzer);
        assert_eq!(
            fuzz.driver_flag().as_deref(),
            Some("-fsanitize=address,fuzzer-no-link")
        );
        assert_eq!(fuzz.passes(OptLevel::O2), ["asan", "sancov-module"]);
        assert_eq!(
            Sanitize::parse("fuzzer").unwrap().driver_flag().as_deref(),
            Some("-fsanitize=fuzzer-no-link")
        );
        assert!(Sanitize::parse("thread").is_err());
        assert!(check_sanitizer_target(Some(BuildTarget::windows(TargetCpu::X64, false))).is_err());
        assert!(check_sanitizer_target(Some(BuildTarget::wasm_module())).is_err());
    }

    #[test]
    fn windows_targets_pick_windows_linkers() {
        let target = |triple: &str| Some(BuildTarget::parse(triple).unwrap());
        let linker = |triple: &str| toolchain_for(target(triple)).linker;
        assert_eq!(linker("x86_64-pc-windows-gnu"), Linker::MinGw);
        assert_eq!(linker("x86_64-pc-windows-msvc"), Linker::Msvc);
        assert_eq!(linker("x86_64-unknown-linux-gnu"), Linker::Unix);
        let cross = Some(BuildTarget::windows(TargetCpu::X64, false));
        assert_eq!(output_extension(cross, OutputKind::Executable), Some("exe"));
        assert_eq!(
            output_extension(cross, OutputKind::DynamicLibrary),
            Some("dll")
        );
    }

    #[test]
    fn windows_arm64_targets_pick_windows_linkers() {
        let target = |triple: &str| Some(BuildTarget::parse(triple).unwrap());
        assert_eq!(
            toolchain_for(target("aarch64-pc-windows-msvc")).linker,
            Linker::Msvc
        );
        for mingw in ["aarch64-pc-windows-gnu", "aarch64-w64-mingw32"] {
            assert_eq!(toolchain_for(target(mingw)).linker, Linker::MinGw);
            assert_eq!(mingw_cpu(target(mingw)), "aarch64");
        }
        assert_eq!(
            mingw_cpu(Some(BuildTarget::windows(TargetCpu::X64, false))),
            "x86_64"
        );
        assert_eq!(
            output_extension(target("aarch64-pc-windows-gnu"), OutputKind::Executable),
            Some("exe")
        );
    }

    #[test]
    fn windows_system_libraries_link_by_name() {
        let lib = |name: &str| Library {
            name: name.into(),
            system: true,
            link_always: false,
            base_dir: String::new(),
        };
        let msvc = BuildTarget::windows(TargetCpu::X64, true).toolchain();
        let mingw = BuildTarget::windows(TargetCpu::X64, false).toolchain();
        let args = |name: &str, toolchain| library_args(&lib(name), toolchain, true).unwrap();
        assert_eq!(
            args("kernel32", mingw),
            vec![LinkArg::Lib("kernel32".into())]
        );
        for row in msvc.runtime_names {
            let name = row.name;
            assert!(args(name, msvc).is_empty(), "{name}");
            assert!(args(name, mingw).is_empty(), "{name}");
        }
        assert_eq!(args("Gdi32", mingw), vec![LinkArg::Lib("gdi32".into())]);
        let mut cmd = Command::new("link");
        render_link_arg(&mut cmd, &LinkArg::Lib("user32".into()), true);
        assert_eq!(cmd.get_args().next().unwrap(), "user32.lib");
    }

    #[test]
    fn library_files_link_before_named_libraries() {
        let mut groups = vec![
            vec![LinkArg::Lib("m".into())],
            vec![LinkArg::File("libstb_vorbis.a".into())],
            vec![LinkArg::Framework("AppKit".into())],
            vec![
                LinkArg::File("libone.so".into()),
                LinkArg::Rpath("/dir".into()),
            ],
        ];
        order_link_groups(&mut groups);
        assert_eq!(
            groups,
            vec![
                vec![LinkArg::File("libstb_vorbis.a".into())],
                vec![
                    LinkArg::File("libone.so".into()),
                    LinkArg::Rpath("/dir".into()),
                ],
                vec![LinkArg::Lib("m".into())],
                vec![LinkArg::Framework("AppKit".into())],
            ]
        );
    }
}
