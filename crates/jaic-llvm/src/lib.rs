//! LLVM backend for the jaic compiler core.
//!
//! [`emit_object`] lowers an `ir::Program` to LLVM IR and writes a native
//! object file ([`emit_objects`] splits large unoptimized builds across threads). Linking, the
//! build options and the loader for this backend as a library live in `jaic-native`, which this
//! crate re-exports so a caller that links LLVM directly needs only this one.
mod debuginfo;
mod emit;
mod green;
mod lower;
mod machine;
mod partition;
mod simd;
mod split;

pub use jaic_native::{
    Codegen, LinkInputs, LinkRequest, OptLevel, Options, OutputKind, SANCOV_PASS, Sanitize,
    WasmLink, archive, check_sanitizer_target, find_wasm_ld, fix_malloc_zone_for_fork, link,
    link_failure, link_inputs, link_wasm, linker_not_run, output_extension, used_libraries,
    write_dsym,
};

pub use self::emit::{emit_object, emit_objects};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emit::{apply_abi_unwind_attributes, apply_codegen_attributes, unit_path};
    use crate::machine::{baseline_cpu, cpu_and_features, target_machine};
    use inkwell::attributes::{Attribute, AttributeLoc};
    use inkwell::context::Context;
    use inkwell::targets::TargetMachine;
    use jaic::target::Target as BuildTarget;
    use std::path::Path;

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
}
