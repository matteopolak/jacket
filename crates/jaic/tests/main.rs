//! `jaic`'s integration tests, built as one binary so the compiler is linked once.
//! Every file in this folder is a module here, its own `[[test]]` in `Cargo.toml`, or a
//! support module another file declares; `every_test_file_is_built` fails otherwise.

#[path = "aggregate_dependencies.rs"]
mod aggregate_dependencies;
#[path = "assignment_count.rs"]
mod assignment_count;
#[path = "atomic_permissions.rs"]
mod atomic_permissions;
#[path = "atomics.rs"]
mod atomics;
#[path = "black_box.rs"]
mod black_box;
#[path = "call_declaration_hints.rs"]
mod call_declaration_hints;
#[path = "call_diagnostics.rs"]
mod call_diagnostics;
#[path = "callable_provenance.rs"]
mod callable_provenance;
#[path = "cast_semantics.rs"]
mod cast_semantics;
#[path = "cast_shifts.rs"]
mod cast_shifts;
#[path = "check_only_output.rs"]
mod check_only_output;
#[path = "checked_operand_types.rs"]
mod checked_operand_types;
#[path = "cleanroom_variable_constness.rs"]
mod cleanroom_variable_constness;
#[path = "code_flags.rs"]
mod code_flags;
#[path = "compile_time_threads.rs"]
mod compile_time_threads;
#[path = "constant_parameter_types.rs"]
mod constant_parameter_types;
#[path = "consteval_diagnostics.rs"]
mod consteval_diagnostics;
#[path = "context.rs"]
mod context;
#[path = "conversion_hints.rs"]
mod conversion_hints;
#[path = "corpus.rs"]
mod corpus;
#[path = "default_type.rs"]
mod default_type;
#[path = "enum_dependencies.rs"]
mod enum_dependencies;
#[path = "evaluation_budgets.rs"]
mod evaluation_budgets;
#[path = "expansion_phases.rs"]
mod expansion_phases;
#[path = "foreign_c_library.rs"]
mod foreign_c_library;
#[path = "foreign_call_plans.rs"]
mod foreign_call_plans;
#[path = "formatter_lexer.rs"]
mod formatter_lexer;
#[path = "freestanding_runtime.rs"]
mod freestanding_runtime;
#[path = "frozen_determinism.rs"]
mod frozen_determinism;
#[path = "generic_pointers.rs"]
mod generic_pointers;
#[path = "getrect_clipboard_dependency.rs"]
mod getrect_clipboard_dependency;
#[path = "host_long_double.rs"]
mod host_long_double;
#[path = "host_runs.rs"]
mod host_runs;
#[path = "host_target.rs"]
mod host_target;
#[path = "ide_deadline.rs"]
mod ide_deadline;
#[path = "ide_entry.rs"]
mod ide_entry;
#[path = "ide_facts_golden.rs"]
mod ide_facts_golden;
#[path = "ide_layout.rs"]
mod ide_layout;
#[path = "ide_lifetimes.rs"]
mod ide_lifetimes;
#[path = "ide_local_scopes.rs"]
mod ide_local_scopes;
#[path = "ide_span_index.rs"]
mod ide_span_index;
#[path = "ifx_callable_headers.rs"]
mod ifx_callable_headers;
#[path = "import_groups.rs"]
mod import_groups;
#[path = "insert_scope.rs"]
mod insert_scope;
#[path = "integer_constant_arithmetic.rs"]
mod integer_constant_arithmetic;
#[path = "interception.rs"]
mod interception;
#[path = "interfaces.rs"]
mod interfaces;
#[path = "interp_division.rs"]
mod interp_division;
#[path = "interp_stack_trace.rs"]
mod interp_stack_trace;
#[path = "interpolates.rs"]
mod interpolates;
#[path = "interpreter_hook.rs"]
mod interpreter_hook;
#[path = "keyword_declarations.rs"]
mod keyword_declarations;
#[path = "lang_items.rs"]
mod lang_items;
#[path = "layout.rs"]
mod layout;
#[path = "leniency_warnings.rs"]
mod leniency_warnings;
#[path = "lexer_target_tags.rs"]
mod lexer_target_tags;
#[path = "lexicon_json.rs"]
mod lexicon_json;
#[path = "library_phases.rs"]
mod library_phases;
#[path = "literals.rs"]
mod literals;
#[path = "loaded_import_scope.rs"]
mod loaded_import_scope;
#[path = "machine_intrinsics.rs"]
mod machine_intrinsics;
#[path = "meta_op_names.rs"]
mod meta_op_names;
#[path = "module_written_defaults.rs"]
mod module_written_defaults;
#[path = "modules.rs"]
mod modules;
#[path = "named_while.rs"]
mod named_while;
#[path = "narrow_filters.rs"]
mod narrow_filters;
#[path = "native_check_hook.rs"]
mod native_check_hook;
#[path = "never_returns.rs"]
mod never_returns;
#[path = "nominal_comparisons.rs"]
mod nominal_comparisons;
#[path = "operator_filters.rs"]
mod operator_filters;
#[path = "operator_home_module.rs"]
mod operator_home_module;
#[path = "operator_literal_operands.rs"]
mod operator_literal_operands;
#[path = "operator_visibility.rs"]
mod operator_visibility;
#[path = "overload_literal_fit.rs"]
mod overload_literal_fit;
#[path = "parser_hints.rs"]
mod parser_hints;
#[path = "placeholder_dependencies.rs"]
mod placeholder_dependencies;
#[path = "pointer_subscript.rs"]
mod pointer_subscript;
#[path = "poly_inference_errors.rs"]
mod poly_inference_errors;
#[path = "polymorph_hints.rs"]
mod polymorph_hints;
#[path = "polymorphic_type_values.rs"]
mod polymorphic_type_values;
#[path = "prelude_entry.rs"]
mod prelude_entry;
#[path = "program_file_checks.rs"]
mod program_file_checks;
#[path = "program_parameter_origin.rs"]
mod program_parameter_origin;
#[path = "pthread_names.rs"]
mod pthread_names;
#[path = "raw_pointers.rs"]
mod raw_pointers;
#[path = "readiness_preservation.rs"]
mod readiness_preservation;
#[path = "recursive_variadic_struct.rs"]
mod recursive_variadic_struct;
#[path = "reflection.rs"]
mod reflection;
#[path = "support/reflection_bootstrap.rs"]
mod reflection_bootstrap;
#[path = "rejected_forms.rs"]
mod rejected_forms;
#[path = "run_result_constants.rs"]
mod run_result_constants;
#[path = "runtime_config.rs"]
mod runtime_config;
#[path = "runtime_contract.rs"]
mod runtime_contract;
#[path = "same_named_types.rs"]
mod same_named_types;
#[path = "sema_version_drift.rs"]
mod sema_version_drift;
#[path = "storage_aliases.rs"]
mod storage_aliases;
#[path = "struct_arguments.rs"]
mod struct_arguments;
#[path = "struct_default_overrides.rs"]
mod struct_default_overrides;
#[path = "symmetric_procedures.rs"]
mod symmetric_procedures;
#[path = "target_architecture.rs"]
mod target_architecture;
#[path = "thread_entry_abi.rs"]
mod thread_entry_abi;
#[path = "toolchain_policy.rs"]
mod toolchain_policy;
#[path = "typed_builtin_keys.rs"]
mod typed_builtin_keys;
#[path = "typed_nodes.rs"]
mod typed_nodes;
#[path = "typed_program_config.rs"]
mod typed_program_config;
#[path = "unit_test_policy.rs"]
mod unit_test_policy;
#[path = "unused_struct_defaults.rs"]
mod unused_struct_defaults;
#[path = "use_before_declaration.rs"]
mod use_before_declaration;
#[path = "using_parameter_writes.rs"]
mod using_parameter_writes;
#[path = "using_redeclaration.rs"]
mod using_redeclaration;
#[path = "variadics.rs"]
mod variadics;
#[path = "void_ifx.rs"]
mod void_ifx;
#[path = "warning_hints.rs"]
mod warning_hints;
#[path = "windows_cross_files.rs"]
mod windows_cross_files;
#[path = "windows_imports.rs"]
mod windows_imports;
#[path = "windows_threads.rs"]
mod windows_threads;

#[test]
fn every_test_file_is_built() {
    test_support::layout::assert_every_test_file_is_built(env!("CARGO_MANIFEST_DIR"));
}
