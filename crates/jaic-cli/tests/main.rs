//! `jaic-cli`'s integration tests, built as one binary so the compiler is linked once.
//! Every file in this folder is a module here, its own `[[test]]` in `Cargo.toml`, or a
//! support module another file declares; `every_test_file_is_built` fails otherwise.

#[path = "argument_order.rs"]
mod argument_order;
#[path = "asm_syscall.rs"]
mod asm_syscall;
#[path = "avr.rs"]
mod avr;
#[path = "browser_guard.rs"]
mod browser_guard;
#[path = "build_status.rs"]
mod build_status;
#[path = "callable_provenance.rs"]
mod callable_provenance;
#[path = "cli.rs"]
mod cli;
#[path = "code_roundtrips.rs"]
mod code_roundtrips;
#[path = "compile_time_services.rs"]
mod compile_time_services;
#[path = "compiler_catalogue.rs"]
mod compiler_catalogue;
#[path = "compiler_primitives.rs"]
mod compiler_primitives;
#[path = "deferred_calls.rs"]
mod deferred_calls;
#[path = "deps.rs"]
mod deps;
#[path = "diagnostic_codes.rs"]
mod diagnostic_codes;
#[path = "diagnostic_conversion_hints.rs"]
mod diagnostic_conversion_hints;
#[path = "diagnostic_name_hints.rs"]
mod diagnostic_name_hints;
#[path = "diagnostic_suggestions.rs"]
mod diagnostic_suggestions;
#[path = "diagnostics.rs"]
mod diagnostics;
#[path = "doc_import_dir.rs"]
mod doc_import_dir;
#[path = "doc_model_inserts.rs"]
mod doc_model_inserts;
#[path = "examples.rs"]
mod examples;
#[path = "failed_run_not_replayed.rs"]
mod failed_run_not_replayed;
#[path = "freestanding.rs"]
mod freestanding;
#[path = "host_runs.rs"]
mod host_runs;
#[path = "http_routes.rs"]
mod http_routes;
#[path = "insert_scope.rs"]
mod insert_scope;
#[path = "interp_counters.rs"]
mod interp_counters;
#[path = "interpreter_assertions.rs"]
mod interpreter_assertions;
#[path = "llvm_inbounds.rs"]
mod llvm_inbounds;
#[path = "llvm_pointer_facts.rs"]
mod llvm_pointer_facts;
#[path = "loaded_import_scope.rs"]
mod loaded_import_scope;
#[path = "missing_prerequisites.rs"]
mod missing_prerequisites;
#[path = "module_compat.rs"]
mod module_compat;
#[path = "module_replacement.rs"]
mod module_replacement;
#[path = "module_roots.rs"]
mod module_roots;
#[path = "n_bit_targets.rs"]
mod n_bit_targets;
#[path = "named_import_run_context.rs"]
mod named_import_run_context;
#[path = "native_link_always.rs"]
mod native_link_always;
#[path = "overloads.rs"]
mod overloads;
#[path = "plugin_build_options.rs"]
mod plugin_build_options;
#[path = "plugin_interp.rs"]
mod plugin_interp;
#[path = "plugin_registration.rs"]
mod plugin_registration;
#[path = "plugins.rs"]
mod plugins;
#[path = "run_budget.rs"]
mod run_budget;
#[path = "run_executable.rs"]
mod run_executable;
#[path = "runtime_config.rs"]
mod runtime_config;
#[path = "sandbox_host.rs"]
mod sandbox_host;
#[path = "stack_capture.rs"]
mod stack_capture;
#[path = "struct_argument_conversion.rs"]
mod struct_argument_conversion;
#[path = "symbol_order.rs"]
mod symbol_order;
#[path = "void_ifx.rs"]
mod void_ifx;
#[path = "wild_pointers.rs"]
mod wild_pointers;
#[path = "workspace_options.rs"]
mod workspace_options;
#[path = "workspace_targets.rs"]
mod workspace_targets;

#[test]
fn every_test_file_is_built() {
    test_support::layout::assert_every_test_file_is_built(env!("CARGO_MANIFEST_DIR"));
}
