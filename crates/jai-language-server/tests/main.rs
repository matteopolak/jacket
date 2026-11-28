//! `jai-language-server`'s integration tests, built as one binary so the compiler is linked once.
//! Every file in this folder is a module here, its own `[[test]]` in `Cargo.toml`, or a
//! support module another file declares; `every_test_file_is_built` fails otherwise.

#[path = "alternate_targets_memory.rs"]
mod alternate_targets_memory;
#[path = "auto_import.rs"]
mod auto_import;
#[path = "auto_import_cold_index.rs"]
mod auto_import_cold_index;
#[path = "baked_default_names.rs"]
mod baked_default_names;
#[path = "build_facts.rs"]
mod build_facts;
#[path = "build_file_entries.rs"]
mod build_file_entries;
#[path = "build_metaprograms.rs"]
mod build_metaprograms;
#[path = "call_snippets.rs"]
mod call_snippets;
#[path = "callback_arguments.rs"]
mod callback_arguments;
#[path = "compile_reuse.rs"]
mod compile_reuse;
#[path = "compile_time_budget.rs"]
mod compile_time_budget;
#[path = "compile_time_faults.rs"]
mod compile_time_faults;
#[path = "compile_time_lints.rs"]
mod compile_time_lints;
#[path = "compile_time_services.rs"]
mod compile_time_services;
#[path = "completion_expected_type.rs"]
mod completion_expected_type;
#[path = "completion_member_paths.rs"]
mod completion_member_paths;
#[path = "completion_receivers.rs"]
mod completion_receivers;
#[path = "completion_struct_literal.rs"]
mod completion_struct_literal;
#[path = "compound_assignment.rs"]
mod compound_assignment;
#[path = "declaration_implementation.rs"]
mod declaration_implementation;
#[path = "deferred_library_bodies.rs"]
mod deferred_library_bodies;
#[path = "definition_generated.rs"]
mod definition_generated;
#[path = "diagnostics.rs"]
mod diagnostics;
#[path = "differential.rs"]
mod differential;
#[path = "doc_comments.rs"]
mod doc_comments;
#[path = "edit_corpus.rs"]
mod edit_corpus;
#[path = "editor_audit.rs"]
mod editor_audit;
#[path = "editor_mode_waits.rs"]
mod editor_mode_waits;
#[path = "entry_traversals.rs"]
mod entry_traversals;
#[path = "enum_using.rs"]
mod enum_using;
#[path = "features.rs"]
mod features;
#[path = "file_uris.rs"]
mod file_uris;
#[path = "for_expansion_names.rs"]
mod for_expansion_names;
#[path = "formatting.rs"]
mod formatting;
#[path = "framing.rs"]
mod framing;
#[path = "fuzz_regressions.rs"]
mod fuzz_regressions;
#[path = "generated_code.rs"]
mod generated_code;
#[path = "generated_fields.rs"]
mod generated_fields;
#[path = "hierarchy.rs"]
mod hierarchy;
#[path = "hover_baked_parameters.rs"]
mod hover_baked_parameters;
#[path = "hover_generic_fields.rs"]
mod hover_generic_fields;
#[path = "hover_modify.rs"]
mod hover_modify;
#[path = "hover_multiline_headers.rs"]
mod hover_multiline_headers;
#[path = "hover_source_only.rs"]
mod hover_source_only;
#[path = "ide_audit_regressions.rs"]
mod ide_audit_regressions;
#[path = "import_discovery.rs"]
mod import_discovery;
#[path = "import_path_agreement.rs"]
mod import_path_agreement;
#[path = "import_paths.rs"]
mod import_paths;
#[path = "inactive_arm_hover.rs"]
mod inactive_arm_hover;
#[path = "indexing_progress.rs"]
mod indexing_progress;
#[path = "interpolates.rs"]
mod interpolates;
#[path = "interrupted_check.rs"]
mod interrupted_check;
#[path = "jacket_toml.rs"]
mod jacket_toml;
#[path = "lifetime_perf.rs"]
mod lifetime_perf;
#[path = "linked_editing.rs"]
mod linked_editing;
#[path = "links.rs"]
mod links;
#[path = "loaded_fragments.rs"]
mod loaded_fragments;
#[path = "long_polymorph_names.rs"]
mod long_polymorph_names;
#[path = "macro_overflow_hang.rs"]
mod macro_overflow_hang;
#[path = "members.rs"]
mod members;
#[path = "module_alias_uses.rs"]
mod module_alias_uses;
#[path = "module_roots.rs"]
mod module_roots;
#[path = "modules.rs"]
mod modules;
#[path = "multi_entry_hover.rs"]
mod multi_entry_hover;
#[path = "named_arguments.rs"]
mod named_arguments;
#[path = "navigation_associations.rs"]
mod navigation_associations;
#[path = "no_compile_in_requests.rs"]
mod no_compile_in_requests;
#[path = "note_readers.rs"]
mod note_readers;
#[path = "note_readers_deferred.rs"]
mod note_readers_deferred;
#[path = "outline.rs"]
mod outline;
#[path = "package_versions.rs"]
mod package_versions;
#[path = "partial_edits.rs"]
mod partial_edits;
#[path = "poly_hints.rs"]
mod poly_hints;
#[path = "poly_struct_constants.rs"]
mod poly_struct_constants;
#[path = "polymorph_instances.rs"]
mod polymorph_instances;
#[path = "prelude_entry.rs"]
mod prelude_entry;
#[path = "print_like.rs"]
mod print_like;
#[path = "project_graph_changes.rs"]
mod project_graph_changes;
#[path = "project_graph_indexed.rs"]
mod project_graph_indexed;
#[path = "protocol.rs"]
mod protocol;
#[path = "protocol_robustness.rs"]
mod protocol_robustness;
#[path = "pull_cancelled.rs"]
mod pull_cancelled;
#[path = "qualified_values.rs"]
mod qualified_values;
#[path = "quoted_code_probes.rs"]
mod quoted_code_probes;
#[path = "range_length.rs"]
mod range_length;
#[path = "reexport_navigation.rs"]
mod reexport_navigation;
#[path = "refactor.rs"]
mod refactor;
#[path = "refresh_coalescing.rs"]
mod refresh_coalescing;
#[path = "rename.rs"]
mod rename;
#[path = "resilience.rs"]
mod resilience;
#[path = "sandbox_unavailable.rs"]
mod sandbox_unavailable;
#[path = "saved_configuration.rs"]
mod saved_configuration;
#[path = "semantic.rs"]
mod semantic;
#[path = "semantic_context.rs"]
mod semantic_context;
#[path = "semantic_tokens_ranges.rs"]
mod semantic_tokens_ranges;
#[path = "silent_disk_edit.rs"]
mod silent_disk_edit;
#[path = "sketch_parameters.rs"]
mod sketch_parameters;
#[path = "source.rs"]
mod source;
#[path = "staged_diagnostics.rs"]
mod staged_diagnostics;
#[path = "stdio.rs"]
mod stdio;
#[path = "suggestion_actions.rs"]
mod suggestion_actions;
#[path = "test_lenses.rs"]
mod test_lenses;
#[path = "trait_this_hang.rs"]
mod trait_this_hang;
#[path = "type_hierarchy.rs"]
mod type_hierarchy;
#[path = "../examples/support/typing_files.rs"]
mod typing_files;
#[path = "typing_regressions.rs"]
mod typing_regressions;
#[path = "typing_session.rs"]
mod typing_session;
#[path = "unreachable.rs"]
mod unreachable;
#[path = "using_filters.rs"]
mod using_filters;
#[path = "watch_registration.rs"]
mod watch_registration;
#[path = "workspace_trust.rs"]
mod workspace_trust;

#[test]
fn every_test_file_is_built() {
    test_support::layout::assert_every_test_file_is_built(env!("CARGO_MANIFEST_DIR"));
}
