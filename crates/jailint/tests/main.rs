//! `jailint`'s integration tests, built as one binary so the compiler is linked once.
//! Every file in this folder is a module here, its own `[[test]]` in `Cargo.toml`, or a
//! support module another file declares; `every_test_file_is_built` fails otherwise.

#[path = "branch_errors.rs"]
mod branch_errors;
#[path = "cli.rs"]
mod cli;
#[path = "compile_time_limits.rs"]
mod compile_time_limits;
#[path = "declaration_notes.rs"]
mod declaration_notes;
#[path = "generator_context_roots.rs"]
mod generator_context_roots;
#[path = "library_roots.rs"]
mod library_roots;
#[path = "lifetime_scaling.rs"]
mod lifetime_scaling;
#[path = "lint_each.rs"]
mod lint_each;
#[path = "nested_config.rs"]
mod nested_config;
#[path = "overloads.rs"]
mod overloads;
#[path = "platform_leak.rs"]
mod platform_leak;
#[path = "rules.rs"]
mod rules;
#[path = "shared_note_reads.rs"]
mod shared_note_reads;
#[path = "shared_parses.rs"]
mod shared_parses;
#[path = "storage_aliases.rs"]
mod storage_aliases;
#[path = "targets.rs"]
mod targets;

#[test]
fn every_test_file_is_built() {
    test_support::layout::assert_every_test_file_is_built(env!("CARGO_MANIFEST_DIR"));
}

#[path = "unreachable_declaration.rs"]
mod unreachable_declaration;
