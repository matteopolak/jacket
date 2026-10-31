//! LLVM backend for the jaic compiler core.
//!
//! [`emit_object`] lowers an `ir::Program` to LLVM IR and writes a native
//! object file ([`emit_objects`] splits large unoptimized builds across threads); [`link`] turns object files into an executable with the
//! system C compiler driver.
mod debuginfo;
mod emit;
mod green;
mod link;
mod link_options;
mod linker;
mod lower;
mod machine;
#[cfg(target_os = "macos")]
mod malloc_zone;
mod options;
mod partition;
mod simd;
mod split;
mod wasm;

/// The LLVM prefix this build was made against (`build.rs`), searched after the environment.
pub(crate) const BAKED_LLVM_PREFIX: Option<&str> = option_env!("LLVM_PREFIX_AT_BUILD");

pub use wasm::{WasmLink, find_wasm_ld, link_wasm};


pub use self::emit::{emit_object, emit_objects, fix_malloc_zone_for_fork, used_libraries};
pub use self::link::{LinkInputs, LinkRequest, link, link_inputs};
pub use self::linker::{
    OutputKind, archive, check_sanitizer_target, link_failure, linker_not_run, output_extension,
    write_dsym,
};
pub use self::options::{Codegen, OptLevel, Options, SANCOV_PASS, Sanitize};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emit::{apply_abi_unwind_attributes, apply_codegen_attributes, unit_path};
    use crate::link::{LinkArg, order_link_groups, toolchain_for};
    use crate::linker::{library_args, mingw_cpu, render_link_arg};
    use crate::machine::{baseline_cpu, cpu_and_features, target_machine};
    use inkwell::attributes::{Attribute, AttributeLoc};
    use inkwell::context::Context;
    use inkwell::targets::TargetMachine;
    use jaic::ir::Library;
    use jaic::target::Target as BuildTarget;
    use jaic::target::TargetCpu;
    use jaic::toolchain::Linker;
    use std::path::Path;
    use std::process::Command;

    #[test]
    fn windows_frame_functions_keep_async_unwind_tables_even_when_nounwind() {
        for triple in [
            "x86_64-pc-windows-msvc",
            "x86_64-pc-windows-gnu",
            "aarch64-pc-windows-msvc",
            "aarch64-unknown-linux-gnu",
            "arm64-apple-macosx11.0",
        ] {
            let target = BuildTarget::parse(triple).unwrap();
            let context = Context::create();
            let module = context.create_module("native_unwind_contract");
            let function = module.add_function(
                "frame_function",
                context.void_type().fn_type(&[], false),
                None,
            );
            let declaration = module.add_function(
                "foreign_declaration",
                context.void_type().fn_type(&[], false),
                None,
            );
            let block = context.append_basic_block(function, "entry");
            let builder = context.create_builder();
            builder.position_at_end(block);
            builder.build_return(None).unwrap();
            function.add_attribute(
                AttributeLoc::Function,
                context.create_enum_attribute(Attribute::get_named_enum_kind_id("nounwind"), 0),
            );
            apply_abi_unwind_attributes(&context, &module, target.arch().unwrap()).unwrap();
            apply_codegen_attributes(&context, &module, &Codegen::default(), target);
            let kind = Attribute::get_named_enum_kind_id("uwtable");
            let actual = function
                .get_enum_attribute(AttributeLoc::Function, kind)
                .map(inkwell::attributes::Attribute::get_enum_value);
            assert_eq!(actual, target.is_windows().then_some(2), "target {target}");
            assert!(
                declaration
                    .get_enum_attribute(AttributeLoc::Function, kind)
                    .is_none()
            );
        }
    }

    #[test]
    fn other_units_write_their_own_ir_files() {
        let path = Path::new("out/prog.ll");
        assert_eq!(unit_path(path, 0), path);
        assert_eq!(unit_path(path, 2), Path::new("out/prog.2.ll"));
        assert_eq!(unit_path(Path::new("prog"), 1), Path::new("prog.1"));
        assert_eq!(
            unit_path(Path::new("a.unoptimized.bc"), 3),
            Path::new("a.unoptimized.3.bc")
        );
    }

    #[test]
    fn builds_target_the_baseline_cpu_not_the_build_machine() {
        let target = |triple: &str| BuildTarget::parse(triple).unwrap();
        let linux = target("x86_64-unknown-linux-gnu");
        assert_eq!(
            cpu_and_features(None, None, linux),
            ("x86-64".into(), String::new())
        );
        assert_eq!(cpu_and_features(Some(""), None, linux).0, "x86-64");
        assert_eq!(
            cpu_and_features(None, None, target("arm64-apple-macosx11.0")).0,
            "apple-m1"
        );
        assert_eq!(
            cpu_and_features(None, None, target("aarch64-unknown-linux-gnu")).0,
            "generic"
        );
        assert_eq!(
            cpu_and_features(Some("skylake"), Some("+avx2"), linux),
            ("skylake".into(), "+avx2".into())
        );
        assert_eq!(
            cpu_and_features(None, Some("+simd128"), target("wasm64-unknown-wasi")),
            ("generic".into(), "+simd128,+bulk-memory".into())
        );
        let (native, features) = cpu_and_features(Some("native"), None, linux);
        assert_eq!(native, TargetMachine::get_host_cpu_name().to_string());
        assert_eq!(features, TargetMachine::get_host_cpu_features().to_string());
        // A host build (no `-target`) gets the baseline too.
        let (machine, _, target, _) = target_machine(&Options::default()).unwrap();
        assert_eq!(machine.get_cpu().to_string_lossy(), baseline_cpu(target));
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
