//! `jacket`'s integration tests, built as one binary so the compiler is linked once.
//! Every file in this folder is a module here, its own `[[test]]` in `Cargo.toml`, or a
//! support module another file declares; `every_test_file_is_built` fails otherwise.

#[path = "assert_output.rs"]
mod assert_output;
#[path = "board_tests.rs"]
mod board_tests;
#[path = "compile_coverage.rs"]
mod compile_coverage;
#[path = "completions.rs"]
mod completions;
#[path = "crash_output.rs"]
mod crash_output;
#[path = "dep_dedupe.rs"]
mod dep_dedupe;
#[path = "dep_direct.rs"]
mod dep_direct;
#[path = "dep_package.rs"]
mod dep_package;
#[path = "dep_package_roots.rs"]
mod dep_package_roots;
#[path = "dep_policy.rs"]
mod dep_policy;
#[path = "dep_preconditions.rs"]
mod dep_preconditions;
#[path = "dep_submodules.rs"]
mod dep_submodules;
mod dep_support;
#[path = "dep_versions.rs"]
mod dep_versions;
#[path = "doc_files.rs"]
mod doc_files;
#[path = "doc_generated.rs"]
mod doc_generated;
#[path = "doc_import_dir.rs"]
mod doc_import_dir;
#[path = "doc_module_identity.rs"]
mod doc_module_identity;
#[path = "doc_theme.rs"]
mod doc_theme;
#[path = "explain.rs"]
mod explain;
#[path = "fmt_passthrough.rs"]
mod fmt_passthrough;
#[path = "forwarded_targets.rs"]
mod forwarded_targets;
#[path = "support/installed.rs"]
mod installed;
#[path = "leak_report.rs"]
mod leak_report;
#[path = "lint_notes.rs"]
mod lint_notes;
#[path = "list_compile_time.rs"]
mod list_compile_time;
#[path = "migrate.rs"]
mod migrate;
#[path = "named_targets.rs"]
mod named_targets;
#[path = "new_golden.rs"]
mod new_golden;
#[path = "plain_timeouts.rs"]
mod plain_timeouts;
#[path = "prerequisites.rs"]
mod prerequisites;
#[path = "run_installed.rs"]
mod run_installed;
#[path = "target_catalogue.rs"]
mod target_catalogue;

#[test]
fn every_test_file_is_built() {
    test_support::layout::assert_every_test_file_is_built(env!("CARGO_MANIFEST_DIR"));
}
