use super::*;
use jai_modules::GraphOptions;

mod messages;
mod runtime_info;

fn signature(parameters: &[TypeId], results: &[TypeId]) -> ProcedureType {
    ProcedureType {
        parameters: parameters.into(),
        results: results.into(),
        return_abi: jai_types::ForeignReturnAbi::Natural,
        convention: CallingConvention::Jai,
        context: ContextMode::Implicit,
        variadic: jai_types::Variadic::None,
    }
}
fn workspace() -> WorkspaceId {
    WorkspaceId::from_raw(7).unwrap()
}
fn runtime_target() -> jai_types::BuildTarget {
    jai_types::BuildTarget {
        operating_system: jai_types::OperatingSystem::MacOS,
        architecture: jai_types::Architecture::Arm64,
        layout: jai_types::LayoutPolicy::lp64(),
        byte_order: jai_types::ByteOrder::Little,
    }
}

#[test]
fn source_string_precedes_signed_workspace_and_internal_order_is_rejected() {
    let types = TypeRegistry::new();
    let string = types.string();
    let signed = types.scalar(ScalarType::Int(IntegerType::S64));
    let unsigned = types.scalar(ScalarType::Int(IntegerType::U64));
    let accepted = bind_signature(
        SourceIntrinsic::AddString,
        &signature(&[string, signed], &[]),
        &types,
        workspace(),
    )
    .unwrap();
    assert_eq!(
        accepted,
        CompilerIntrinsic::SourceAddString {
            current_workspace: workspace()
        }
    );
    assert!(
        bind_signature(
            SourceIntrinsic::AddString,
            &signature(&[signed, string], &[]),
            &types,
            workspace()
        )
        .is_err()
    );
    assert!(
        bind_signature(
            SourceIntrinsic::AddString,
            &signature(&[string, unsigned], &[]),
            &types,
            workspace()
        )
        .is_err()
    );
}

#[test]
fn workspace_result_and_report_semantics_are_verified() {
    let types = TypeRegistry::new();
    let string = types.string();
    let signed = types.scalar(ScalarType::Int(IntegerType::S64));
    let unsigned = types.scalar(ScalarType::Int(IntegerType::U64));
    assert_eq!(
        bind_signature(
            SourceIntrinsic::CreateWorkspace,
            &signature(&[string], &[signed]),
            &types,
            workspace()
        )
        .unwrap(),
        CompilerIntrinsic::SourceCreateWorkspace
    );
    assert!(
        bind_signature(
            SourceIntrinsic::CreateWorkspace,
            &signature(&[string], &[unsigned]),
            &types,
            workspace()
        )
        .is_err()
    );
    assert_eq!(
        bind_signature(
            SourceIntrinsic::CurrentWorkspace,
            &signature(&[], &[signed]),
            &types,
            workspace()
        )
        .unwrap(),
        CompilerIntrinsic::SourceCurrentWorkspace {
            current_workspace: workspace()
        }
    );
    assert_eq!(
        bind_signature(
            SourceIntrinsic::Report,
            &signature(&[string], &[]),
            &types,
            workspace()
        )
        .unwrap(),
        CompilerIntrinsic::SourceReport {
            level: jai_vm::MessageLevel::Error
        }
    );
    assert!(
        bind_signature(
            SourceIntrinsic::Report,
            &signature(&[string, signed], &[]),
            &types,
            workspace()
        )
        .is_err()
    );
}

#[test]
fn unknown_catalog_entries_and_invalid_options_signatures_are_rejected() {
    let types = TypeRegistry::new();
    assert!(SourceIntrinsic::parse("compiler_invented_effect").is_none());
    for source in [
        SourceIntrinsic::SetBuildOptions,
        SourceIntrinsic::GetBuildOptions,
    ] {
        let error = bind_signature(source, &signature(&[], &[]), &types, workspace()).unwrap_err();
        assert!(error.contains("requires"));
        assert!(error.contains("Build_Options"));
    }
}

#[test]
fn origins_are_module_identities_from_selected_configured_paths() {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "jai-compiler-origins-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(root.join("modules/Compiler")).unwrap();
    std::fs::write(
        root.join("main.jai"),
        "C :: #import \"Compiler\";\nmain :: () {}\n",
    )
    .unwrap();
    std::fs::write(
        root.join("modules/Compiler/module.jai"),
        "compiler_create_workspace :: (name: string) -> s64 #compiler;\n",
    )
    .unwrap();
    let directories = vec![root.join("modules")];
    let graph = ModuleGraph::load(
        &root.join("main.jai"),
        jai_modules::GraphOptions {
            import_dirs: directories.clone(),
        },
    )
    .unwrap();
    let compiler = graph.imports()[0].module();
    let origins = CompilerModuleOrigins::from_graph(&graph, &directories);
    assert_eq!(
        origins.origin(graph.root()),
        Some(CompilerModuleOrigin::Application)
    );
    assert_eq!(
        origins.origin(compiler),
        Some(CompilerModuleOrigin::Compiler)
    );
    let other = CompilerModuleOrigins::from_graph(&graph, &[root.join("unselected")]);
    assert_eq!(other.origin(compiler), None);
    std::fs::remove_dir_all(root).unwrap();
}

fn with_graph<T>(source: &str, action: impl FnOnce(&ModuleGraph) -> T) -> T {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "jai-source-intrinsics-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("main.jai"), source).unwrap();
    let graph = ModuleGraph::load(
        &root.join("main.jai"),
        jai_modules::GraphOptions {
            import_dirs: vec![],
        },
    )
    .unwrap();
    let result = action(&graph);
    std::fs::remove_dir_all(root).unwrap();
    result
}

#[test]
fn same_named_ordinary_body_stays_a_body_and_unknown_mark_gets_precise_error() {
    with_graph("add_build_string :: () -> s32 { return 11; }", |graph| {
        let library = crate::resolve_library(graph).unwrap();
        assert_eq!(library.procedures().len(), 1);
        assert!(library.prototypes().is_empty());
    });
    with_graph(
        "danger :: () #compiler \"compiler_invented_effect\";",
        |graph| {
            let error = crate::resolve_library(graph).unwrap_err();
            assert!(
                error
                    .message
                    .contains("unsupported #compiler intrinsic `compiler_invented_effect`")
            );
        },
    );
}

#[test]
fn selected_options_schema_is_verified_and_extra_fields_are_rejected() {
    const OPTIONS: &str = "Llvm_Bitcode_Optimization_Setting :: enum u8 { UNSET :: 0; O0 :: 1; O1 :: 2; O2 :: 3; O3 :: 4; OS :: 5; OZ :: 6; }\nLlvm_Machine_Code_Optimization_Setting :: enum u8 { UNSET :: 0; NONE :: 1; LESS :: 2; DEFAULT :: 3; AGGRESSIVE :: 4; }\nLlvm_Options :: struct { bitcode_optimization_setting: Llvm_Bitcode_Optimization_Setting; machine_code_optimization_setting: Llvm_Machine_Code_Optimization_Setting; target_system_triple: string; }\nBuild_Options :: struct { output_path: string; llvm_options: Llvm_Options; }\nset_build_options :: (options: Build_Options, w: s64 = -1) #compiler;\n";
    let check = |graph: &ModuleGraph| {
        let options = crate::ResolveOptions {
            compiler: Some(CompilerBindingContext::from_graph(graph, &[], workspace())),
            ..crate::ResolveOptions::default()
        };
        crate::resolve_library_with_options(graph, &options, &mut jai_vm::NoEffects)
    };
    with_graph(OPTIONS, |graph| {
        let library = check(graph).unwrap();
        assert_eq!(library.prototypes().len(), 1);
        assert!(library.procedures().is_empty());
    });
    with_graph(
        &OPTIONS.replace(
            "output_path: string;",
            "output_path: string; imaginary_option: bool;",
        ),
        |graph| {
            let error = check(graph).unwrap_err();
            assert!(
                error
                    .message
                    .contains("unsupported Build_Options field `imaginary_option`")
            );
        },
    );
    with_graph(&OPTIONS.replace("OS :: 5", "OS :: 19"), |graph| {
        let error = check(graph).unwrap_err();
        assert!(error.message.contains("incompatible source enum value"));
    });
}

#[test]
fn source_run_projects_checked_options_into_committed_typed_requests() {
    #[derive(Default)]
    struct Effects {
        staged: Vec<jai_vm::CompilerRequest>,
        committed: Vec<jai_vm::CompilerRequest>,
    }
    impl jai_vm::CompilerEffects for Effects {
        fn begin(&mut self) {
            self.staged.clear();
        }
        fn request(&mut self, request: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
            self.staged.push(request);
            jai_vm::EffectOutcome::Ready(jai_vm::CompilerResponse::Unit)
        }
        fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
            if commit {
                self.committed.append(&mut self.staged);
            }
            self.staged.clear();
            Ok(())
        }
    }
    const SOURCE: &str = "Llvm_Bitcode_Optimization_Setting :: enum u8 { UNSET :: 0; O0 :: 1; O1 :: 2; O2 :: 3; O3 :: 4; OS :: 5; OZ :: 6; }\nLlvm_Options :: struct { bitcode_optimization_setting: Llvm_Bitcode_Optimization_Setting; target_system_triple: string; }\nBuild_Options :: struct { output_path: string; llvm_options: Llvm_Options; }\nset_build_options :: (options: Build_Options, w: s64 = -1) #compiler;\n#run set_build_options(Build_Options.{output_path = \"build\", llvm_options = Llvm_Options.{bitcode_optimization_setting = Llvm_Bitcode_Optimization_Setting.OZ, target_system_triple = \"aarch64-apple-darwin\"}});\n";
    with_graph(SOURCE, |graph| {
        let options = crate::ResolveOptions {
            compiler: Some(CompilerBindingContext::from_graph(graph, &[], workspace())),
            ..crate::ResolveOptions::default()
        };
        let mut effects = Effects::default();
        crate::resolve_library_with_options(graph, &options, &mut effects).unwrap();
        assert_eq!(
            effects.committed,
            vec![
                jai_vm::CompilerRequest::SetBuildOption {
                    workspace: workspace(),
                    option: jai_vm::BuildOption::OutputPath("build".into())
                },
                jai_vm::CompilerRequest::SetBuildOption {
                    workspace: workspace(),
                    option: jai_vm::BuildOption::Target(
                        jai_vm::TargetTriple::parse("aarch64-apple-darwin").unwrap()
                    )
                },
                jai_vm::CompilerRequest::SetBuildOption {
                    workspace: workspace(),
                    option: jai_vm::BuildOption::BitcodeOptimization(
                        jai_vm::BitcodeOptimization::Oz
                    )
                },
            ]
        );
    });
}

#[test]
fn full_report_preserves_location_and_error_rolls_back_prior_requests() {
    #[derive(Default)]
    struct Effects {
        staged: Vec<jai_vm::CompilerRequest>,
        committed: Vec<jai_vm::CompilerRequest>,
    }
    impl jai_vm::CompilerEffects for Effects {
        fn begin(&mut self) {
            self.staged.clear();
        }
        fn request(&mut self, request: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
            self.staged.push(request);
            jai_vm::EffectOutcome::Ready(jai_vm::CompilerResponse::Unit)
        }
        fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
            if commit {
                self.committed.append(&mut self.staged);
            }
            self.staged.clear();
            Ok(())
        }
    }
    const SOURCE: &str = "Source_Code_Location :: struct { fully_pathed_filename: string; line_number: s64; character_number: s64; }\nReport :: enum u8 { ERROR :: 0; ERROR_CONTINUABLE :: 1; WARNING :: 2; INFO :: 3; }\ncompiler_report :: (message: string, loc: Source_Code_Location, mode: Report) #compiler;\nadd_build_string :: (data: string, w: s64) #compiler;\nrecipe :: () { add_build_string(\"answer :: 42;\", -1); compiler_report(\"note\", Source_Code_Location.{fully_pathed_filename = \"recipe.jai\", line_number = 8, character_number = 4}, Report.WARNING); }\n#run recipe();";
    with_graph(SOURCE, |graph| {
        let options = crate::ResolveOptions {
            compiler: Some(CompilerBindingContext::from_graph(graph, &[], workspace())),
            ..crate::ResolveOptions::default()
        };
        let mut effects = Effects::default();
        crate::resolve_library_with_options(graph, &options, &mut effects).unwrap();
        assert!(
            matches!(&effects.committed[1], jai_vm::CompilerRequest::Report { level: jai_vm::MessageLevel::Warning, continuation: jai_vm::ReportContinuation::Continue, location, text } if location.path == std::path::Path::new("recipe.jai") && location.line == 8 && location.column == 4 && text == "note")
        );
    });
    with_graph(&SOURCE.replace("Report.WARNING", "Report.ERROR"), |graph| {
        let options = crate::ResolveOptions {
            compiler: Some(CompilerBindingContext::from_graph(graph, &[], workspace())),
            ..crate::ResolveOptions::default()
        };
        let mut effects = Effects::default();
        let error = crate::resolve_library_with_options(graph, &options, &mut effects).unwrap_err();
        assert!(error.message.contains("recipe.jai:8:4"));
        assert!(effects.committed.is_empty());
        assert!(effects.staged.is_empty());
    });
}

#[test]
fn constructor_default_string_is_checked_from_the_source_header() {
    with_graph(
        "Workspace :: s64;\ncompiler_create_workspace :: (name := \"\") -> Workspace #compiler;",
        |graph| {
            let options = crate::ResolveOptions {
                compiler: Some(CompilerBindingContext::from_graph(graph, &[], workspace())),
                ..crate::ResolveOptions::default()
            };
            let library =
                crate::resolve_library_with_options(graph, &options, &mut jai_vm::NoEffects)
                    .unwrap();
            assert_eq!(library.prototypes().len(), 1);
            assert!(library.procedures().is_empty());
        },
    );
}

#[test]
fn marked_fallback_body_is_preserved_while_run_uses_workspace_intrinsic() {
    with_graph(
        "get_current_workspace :: () -> s64 #compiler { return 0; }\nf :: () -> s64 { return #run get_current_workspace(); }",
        |graph| {
            let options = crate::ResolveOptions {
                compiler: Some(CompilerBindingContext::from_graph(graph, &[], workspace())),
                ..crate::ResolveOptions::default()
            };
            let library =
                crate::resolve_library_with_options(graph, &options, &mut jai_vm::NoEffects)
                    .unwrap();
            assert_eq!(library.procedures().len(), 2);
            assert!(library.prototypes().is_empty());
            let mut vm =
                jai_vm::Vm::new(&library, jai_vm::NoEffects, jai_vm::Limits::default()).unwrap();
            for (declaration, value) in [
                (graph.declarations()[0].id(), 0),
                (graph.declarations()[1].id(), 7),
            ] {
                let call = jai_ir::Call::new(library.procedure(declaration).unwrap().id, vec![]);
                assert_eq!(
                    vm.evaluate_call(&call).outcome,
                    jai_vm::Outcome::Complete(vec![jai_vm::Value::Int(
                        jai_types::Integer::checked(IntegerType::S64, value).unwrap()
                    )])
                );
            }
        },
    );
}

#[test]
fn version_catalog_requires_exact_nominal_names_types_and_order() {
    let check = |graph: &ModuleGraph| {
        let options = crate::ResolveOptions {
            compiler: Some(CompilerBindingContext::from_graph(graph, &[], workspace())),
            ..crate::ResolveOptions::default()
        };
        crate::resolve_library_with_options(graph, &options, &mut jai_vm::NoEffects)
    };
    let source = "Version_Info :: struct { major: s32; minor: s32; micro: s32; }\ncompiler_get_version_info :: (p: *Version_Info) -> string #compiler;";
    with_graph(source, |graph| {
        assert_eq!(check(graph).unwrap().prototypes().len(), 1);
    });
    for incompatible in [
        source.replace("major: s32; minor: s32", "minor: s32; major: s32"),
        source.replace("major: s32", "major: s64"),
        source.replace("Version_Info", "Impostor"),
    ] {
        with_graph(&incompatible, |graph| {
            assert!(check(graph).unwrap_err().message.contains("Version_Info"));
        });
    }
}

#[test]
fn discarded_source_parameters_do_not_create_an_accidental_catalog_match() {
    with_graph(
        "get_current_workspace :: (#discard ignored: int) -> s64 #compiler;",
        |graph| {
            let options = crate::ResolveOptions {
                compiler: Some(CompilerBindingContext::from_graph(graph, &[], workspace())),
                ..crate::ResolveOptions::default()
            };
            let error =
                crate::resolve_library_with_options(graph, &options, &mut jai_vm::NoEffects)
                    .unwrap_err();
            assert!(
                error
                    .message
                    .contains("parameter binding differs from its source catalog"),
                "{error:?}"
            );
        },
    );
}

#[test]
fn selected_runtime_source_authorizes_its_parameterized_instances_by_source_identity() {
    use jai_modules::Filesystem;
    use jai_modules::{
        BootstrapOptions, PreludeSource, RuntimeSupportOptions, RuntimeSupportParameters,
        RuntimeSupportSource,
    };
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "jai-runtime-role-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&root).unwrap();
    let source_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let prelude_dir = source_root.join("prelude");
    let stdlib = source_root.join("stdlib");
    let preload = prelude_dir.join("module.jai");
    let runtime = stdlib.join("Runtime_Support.jai");
    let main = root.join("main.jai");
    std::fs::write(
        &main,
        "Runtime_Small :: #import \"Runtime_Support\" (DEFINE_SYSTEM_ENTRY_POINT = false, DEFINE_INITIALIZATION = false, ENABLE_BACKTRACE_ON_CRASH = false, TEMPORARY_STORAGE_SIZE = 4096);\nRuntime_Large :: #import \"Runtime_Support\" (DEFINE_SYSTEM_ENTRY_POINT = false, DEFINE_INITIALIZATION = false, ENABLE_BACKTRACE_ON_CRASH = false, TEMPORARY_STORAGE_SIZE = 65536);\nmain :: () {}\n",
    )
    .unwrap();
    let import_dirs = vec![stdlib.clone(), prelude_dir.clone()];
    let graph = ModuleGraph::load_with_bootstrap_options(
        &main,
        GraphOptions {
            import_dirs: import_dirs.clone(),
        },
        BootstrapOptions {
            prelude: PreludeSource::File(preload),
            runtime_support: Some(RuntimeSupportOptions {
                source: RuntimeSupportSource::File(runtime),
                parameters: RuntimeSupportParameters {
                    define_system_entry_point: false,
                    define_initialization: false,
                    enable_backtrace_on_crash: false,
                    temporary_storage_size: 32768,
                },
            }),
        },
        &Filesystem,
        Some(runtime_target()),
    )
    .unwrap();
    let selected = graph.runtime_support().unwrap();
    let selected_source = graph
        .file(graph.module(selected).unwrap().entry())
        .unwrap()
        .source();
    let runtime_instances = graph
        .modules()
        .iter()
        .filter(|instance| {
            graph
                .file(instance.entry())
                .is_some_and(|file| file.source() == selected_source)
        })
        .collect::<Vec<_>>();
    assert!(
        runtime_instances.len() >= 3,
        "bootstrap plus two argument variants"
    );
    let context = CompilerBindingContext::from_graph_with_provider(
        &graph,
        &import_dirs,
        workspace(),
        &Filesystem,
    );
    for instance in runtime_instances {
        assert_eq!(
            context.origins.origin(instance.id()),
            Some(CompilerModuleOrigin::RuntimeSupport)
        );
    }
    let options = crate::ResolveOptions {
        target: Some(runtime_target()),
        compiler: Some(context),
        ..crate::ResolveOptions::default()
    };
    crate::resolve_library_with_options(&graph, &options, &mut jai_vm::NoEffects).unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn copied_or_loaded_runtime_writer_declarations_do_not_inherit_source_authority() {
    use jai_modules::Filesystem;
    use jai_modules::{
        BootstrapOptions, PreludeSource, RuntimeSupportOptions, RuntimeSupportParameters,
        RuntimeSupportSource,
    };
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "jai-runtime-role-negative-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let modules = root.join("modules");
    std::fs::create_dir_all(&modules).unwrap();
    let source_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let prelude_dir = source_root.join("prelude");
    let stdlib = source_root.join("stdlib");
    let preload = prelude_dir.join("module.jai");
    let runtime = modules.join("Runtime_Support.jai");
    let generated = modules.join("generated.jai");
    let main = root.join("main.jai");
    let mut runtime_source = std::fs::read_to_string(stdlib.join("Runtime_Support.jai")).unwrap();
    runtime_source.push_str("\n#load \"generated.jai\";\n");
    std::fs::write(&runtime, runtime_source).unwrap();
    std::fs::write(
        &generated,
        "inserted_writer :: (s: string, to_standard_error := false) #no_context #compiler \"write_string\";\n",
    )
    .unwrap();
    std::fs::write(&main, "main :: () {}\n").unwrap();
    let import_dirs = vec![modules.clone(), stdlib.clone(), prelude_dir.clone()];
    let graph = ModuleGraph::load_with_bootstrap_options(
        &main,
        GraphOptions {
            import_dirs: import_dirs.clone(),
        },
        BootstrapOptions {
            prelude: PreludeSource::File(preload),
            runtime_support: Some(RuntimeSupportOptions {
                source: RuntimeSupportSource::File(runtime),
                parameters: RuntimeSupportParameters {
                    define_system_entry_point: false,
                    define_initialization: false,
                    enable_backtrace_on_crash: false,
                    temporary_storage_size: 32768,
                },
            }),
        },
        &Filesystem,
        Some(runtime_target()),
    )
    .unwrap();
    let options = crate::ResolveOptions {
        target: Some(runtime_target()),
        compiler: Some(CompilerBindingContext::from_graph_with_provider(
            &graph,
            &import_dirs,
            workspace(),
            &Filesystem,
        )),
        ..crate::ResolveOptions::default()
    };
    let error =
        crate::resolve_library_with_options(&graph, &options, &mut jai_vm::NoEffects).unwrap_err();
    assert!(
        error
            .message
            .contains("not an original selected source declaration"),
        "{error:?}"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn same_named_runtime_file_from_a_different_source_provider_identity_is_not_selected() {
    use jai_modules::Filesystem;
    use jai_modules::{
        BootstrapOptions, PreludeSource, RuntimeSupportOptions, RuntimeSupportParameters,
        RuntimeSupportSource,
    };
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "jai-runtime-role-copy-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let modules = root.join("modules");
    std::fs::create_dir_all(&modules).unwrap();
    let source_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let prelude_dir = source_root.join("prelude");
    let stdlib = source_root.join("stdlib");
    let preload = prelude_dir.join("module.jai");
    let runtime = stdlib.join("Runtime_Support.jai");
    let copied = modules.join("Runtime_Support_Copy.jai");
    let main = root.join("main.jai");
    std::fs::copy(&runtime, &copied).unwrap();
    std::fs::write(
        &main,
        "Runtime_Copy :: #import \"Runtime_Support_Copy\" (DEFINE_SYSTEM_ENTRY_POINT = false, DEFINE_INITIALIZATION = false, ENABLE_BACKTRACE_ON_CRASH = false);\nmain :: () {}\n",
    )
    .unwrap();
    let import_dirs = vec![modules.clone(), stdlib.clone(), prelude_dir.clone()];
    let graph = ModuleGraph::load_with_bootstrap_options(
        &main,
        GraphOptions {
            import_dirs: import_dirs.clone(),
        },
        BootstrapOptions {
            prelude: PreludeSource::File(preload),
            runtime_support: Some(RuntimeSupportOptions {
                source: RuntimeSupportSource::File(runtime),
                parameters: RuntimeSupportParameters {
                    define_system_entry_point: false,
                    define_initialization: false,
                    enable_backtrace_on_crash: false,
                    temporary_storage_size: 32768,
                },
            }),
        },
        &Filesystem,
        Some(runtime_target()),
    )
    .unwrap();
    let options = crate::ResolveOptions {
        target: Some(runtime_target()),
        compiler: Some(CompilerBindingContext::from_graph_with_provider(
            &graph,
            &import_dirs,
            workspace(),
            &Filesystem,
        )),
        ..crate::ResolveOptions::default()
    };
    let error =
        crate::resolve_library_with_options(&graph, &options, &mut jai_vm::NoEffects).unwrap_err();
    assert!(
        error.message.contains("does not belong to the application"),
        "{error:?}"
    );
    std::fs::remove_dir_all(root).unwrap();
}
