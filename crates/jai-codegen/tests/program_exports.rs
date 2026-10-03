//! Emits only self-written Jai source, then links newly generated objects to own C fixtures.
#![cfg(any(target_os = "linux", target_os = "macos"))]

#[path = "support/native_tools.rs"]
mod native_tools;
use jai_codegen::{
    Context,
    native_reachability::Publication,
    optimization::{BitcodeOptimization, Optimization},
    target::{NativeTarget, TargetOptions},
};
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use std::{
    fs,
    path::Path,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

fn graph(source: &str) -> ModuleGraph {
    let mut sources = SourceOverlay::new();
    sources
        .insert(
            Path::new("/own-export-fixture/main.jai"),
            source.as_bytes().to_vec(),
        )
        .unwrap();
    ModuleGraph::load_with_provider(
        Path::new("/own-export-fixture/main.jai"),
        GraphOptions::default(),
        &sources,
    )
    .unwrap()
}

fn targets() -> impl Iterator<Item = NativeTarget> {
    [BitcodeOptimization::O0, BitcodeOptimization::O2]
        .into_iter()
        .map(|bitcode| {
            NativeTarget::select(&TargetOptions {
                optimization: Optimization {
                    bitcode,
                    ..Optimization::default()
                },
                ..TargetOptions::default()
            })
            .unwrap()
        })
}

fn execute(module: &inkwell::module::Module<'_>, target: &NativeTarget, c_fixture: Option<&str>) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "jai-own-export-native-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&directory).unwrap();
    let object = directory.join("generated.o");
    let executable = directory.join("program");
    module.verify().unwrap();
    target.write_object(module, &object).unwrap();
    let mut linker = native_tools::clang_command();
    let native_search_variables = [
        "LIBRARY_PATH",
        "LD_LIBRARY_PATH",
        "LD_PRELOAD",
        "LD_AUDIT",
        "DYLD_LIBRARY_PATH",
        "DYLD_FALLBACK_LIBRARY_PATH",
        "DYLD_FRAMEWORK_PATH",
        "DYLD_FALLBACK_FRAMEWORK_PATH",
        "DYLD_VERSIONED_LIBRARY_PATH",
        "DYLD_VERSIONED_FRAMEWORK_PATH",
        "DYLD_ROOT_PATH",
        "DYLD_INSERT_LIBRARIES",
        "SDKROOT",
        "CCC_OVERRIDE_OPTIONS",
    ];
    for variable in native_search_variables {
        linker.env_remove(variable);
    }
    linker.arg(&object);
    if let Some(c_fixture) = c_fixture {
        let source = directory.join("own.c");
        fs::write(&source, c_fixture).unwrap();
        linker.arg(source);
    }
    let build = linker.arg("-o").arg(&executable).output().unwrap();
    assert!(
        build.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&build.stderr),
        module.print_to_string()
    );
    let mut run = Command::new(&executable);
    for variable in native_search_variables {
        run.env_remove(variable);
    }
    assert!(run.status().unwrap().success());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn renamed_c_procedure_and_global_exports_interoperate_with_own_c_caller() {
    let graph = graph(
        "#program_export \"native_counter\" count:s32=7; #program_export \"native_callback\" callback := increment; #program_export \"native_increment\" increment :: (delta:s32)->s32 #c_call { count += delta; return count; } #program_export __jai_runtime_init :: (argc:s32,argv:**u8)->*s32 #c_call { count=argc; return *count; } #program_export __jai_runtime_fini :: (_context:*void) #c_call { count=0; }",
    );
    let library = jai_sema::resolve_library(&graph).unwrap();
    let context = Context::create();
    for target in targets() {
        let module = jai_codegen::lower_library_for_target(
            &context,
            &library,
            &Publication::Selected(vec![]),
            &target,
        )
        .unwrap();
        assert!(
            module
                .get_function("native_increment")
                .unwrap()
                .count_basic_blocks()
                > 0
        );
        assert!(module.get_global("native_counter").is_some());
        assert!(module.get_global("native_callback").is_some());
        assert!(module.get_function("__jai_runtime_init").is_some());
        assert!(module.get_function("__jai_runtime_fini").is_some());
        execute(
            &module,
            &target,
            Some(
                "extern int native_counter; extern int native_increment(int); extern int (*native_callback)(int); extern int *__jai_runtime_init(int,char**); extern void __jai_runtime_fini(void*); int main(void) { if(native_counter != 7) return 1; if(native_increment(35) != 42) return 2; if(native_counter != 42 || native_callback != native_increment || native_callback(0) != 42) return 3; int *state=__jai_runtime_init(5,(char**)0); if(state != &native_counter || *state != 5) return 4; __jai_runtime_fini(state); return native_counter == 0 ? 0 : 5; }",
            ),
        );
    }
}

#[test]
fn exported_c_main_replaces_synthetic_wrapper_and_receives_argc() {
    let graph = graph(
        "#add_context number:s32=40; #add_context callback:(value:s32)->s32=adjust; adjust :: (value:s32)->s32 { return value+2; } count:s32=0; first_thread_context:#Context; #program_export \"main\" system_entry :: (argc:s32,argv:**u8)->s32 #c_call { if argc > 0 { push_context first_thread_context { entry :: () #entry_point; no_inline entry(); if count == 42 return 0; } } return 1; } main :: () { count=context.callback(context.number); }",
    );
    let program = jai_sema::resolve_graph(&graph).unwrap();
    assert!(matches!(
        program.native_entry(),
        jai_ir::NativeEntryPoint::ExportedProcedure(_)
    ));
    let context = Context::create();
    for target in targets() {
        let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
        assert_eq!(module.get_function("main").unwrap().count_params(), 2);
        assert!(module.get_function("main.1").is_none());
        execute(&module, &target, None);
    }
}

#[test]
fn compatible_foreign_alias_reuses_exported_definition_symbol() {
    let graph = graph(
        "#program_export \"native_answer\" answer :: () -> s32 #c_call { return 42; } alias :: () -> s32 #foreign \"native_answer\"; main :: () -> int { if alias() == 42 return 0; return 1; }",
    );
    let program = jai_sema::resolve_graph(&graph).unwrap();
    let context = Context::create();
    for target in targets() {
        let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
        assert!(module.get_function("native_answer.1").is_none());
        execute(&module, &target, None);
    }
}

#[test]
fn source_case_selected_export_preserves_original_declarations_and_fallthrough() {
    let graph = graph(
        "Tag::enum{OTHER;CHOSEN;LAST;} selected::Tag.CHOSEN; #program_export \"case_answer\" answer::()->s32 #c_call{#if selected == {case .OTHER; value:s32=missing();case .CHOSEN;value:s32=40;#through;case .LAST;value+=2;}return value;}",
    );
    let library = jai_sema::resolve_library(&graph).unwrap();
    let context = Context::create();
    for target in targets() {
        let module = jai_codegen::lower_library_for_target(
            &context,
            &library,
            &Publication::Selected(Vec::new()),
            &target,
        )
        .unwrap();
        execute(
            &module,
            &target,
            Some(
                "extern int case_answer(void); int main(void) { return case_answer() == 42 ? 0 : 1; }",
            ),
        );
    }
}
