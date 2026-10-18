//! Private namespaces of loaded files and the arguments bundled modules accept.
use super::{compiler, namespace_source, rejects, succeeds};
use std::path::Path;

#[test]
fn loaded_pointer_constructor_keeps_public_type_and_storage_identity() {
    let mut files = namespace_source::FILES.to_vec();
    let main = format!("{}\n#assert(check()==0);", namespace_source::SOURCE);
    files.push(("/app/main.jai", &main));
    let mut c = compiler(&files, false);
    succeeds(&mut c);
}

#[test]
fn module_interface_constraints_use_the_declaring_private_file_namespace() {
    for (arguments, valid) in [
        ("", true),
        ("(HOOKS = Good)", true),
        ("(HOOKS = Broken)", false),
    ] {
        let main = format!(
            "Good :: struct {{ run :: inline (value: int) {{}} }} Broken :: struct {{}} M :: #import \"Contract\"{arguments}; #assert !#exists(M.Required); main :: () {{ result := M.touch(); }}"
        );
        let mut c = compiler(
            &[
                (
                    "/lib/Contract/module.jai",
                    "#module_parameters(HOOKS: $I/interface Required = Required) { #scope_file Required :: struct { run :: inline (value: int) {} } } touch :: () -> int { return 7; }",
                ),
                ("/app/main.jai", &main),
            ],
            false,
        );
        if valid {
            succeeds(&mut c);
        } else {
            rejects(&mut c, "run");
        }
    }
}

#[test]
fn module_parameter_defaults_use_the_declaring_private_file_namespace() {
    let mut c = compiler(
        &[
            (
                "/lib/Config/module.jai",
                "#module_parameters(value: int = hidden_default, Record: Type = Hidden_Record) { #scope_file hidden_default :: 37; Hidden_Record :: struct { value: u64; } } exposed :: value; record_size :: size_of(Record);",
            ),
            (
                "/app/main.jai",
                "M :: #import \"Config\"; #assert(M.exposed == 37); #assert(M.record_size == 8); #assert !#exists(M.hidden_default); #assert !#exists(M.Hidden_Record);",
            ),
        ],
        false,
    );
    succeeds(&mut c);
}

#[test]
fn bundled_modules_reject_unknown_and_excess_positional_arguments() {
    for (arguments, category) in [
        ("misspelled = true", "not a parameter"),
        ("true, false", "Too many module parameters"),
    ] {
        let main = format!("M :: #import \"Config\"({arguments});");
        let mut c = compiler(
            &[
                ("/lib/prelude/module.jai", ""),
                (
                    "/lib/Config/module.jai",
                    "#module_parameters(enabled: bool);",
                ),
                ("/app/main.jai", &main),
            ],
            true,
        );
        rejects(&mut c, category);
    }
}

#[test]
fn implicit_runtime_does_not_ignore_explicit_import_arguments() {
    let mut c = compiler(
        &[
            ("/lib/prelude/module.jai", ""),
            (
                "/lib/Runtime_Support/module.jai",
                "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT: bool, DEFINE_INITIALIZATION: bool, ENABLE_BACKTRACE_ON_CRASH: bool);",
            ),
            (
                "/app/main.jai",
                "R :: #import \"Runtime_Support\"(unknown = true);",
            ),
        ],
        true,
    );
    c.options.runtime_support = true;
    rejects(&mut c, "not a parameter");
}

#[test]
fn ide_selected_internal_entry_is_admitted_without_opening_application_imports() {
    let files = [
        ("/lib/prelude/module.jai", ""),
        ("/lib/internal/helper/module.jai", "value :: 4;"),
        ("/app/main.jai", "H :: #import \"internal/helper\";"),
    ];
    let mut c = compiler(&files, true);
    c.ide = Some(Box::default());
    c.compile_program(Path::new("/lib/internal/helper/module.jai"))
        .unwrap_or_else(|e| panic!("{}", e.render(&c.sources)));

    let mut c = compiler(&files, true);
    c.ide = Some(Box::default());
    rejects(&mut c, "private");
    let mut c = compiler(&files, true);
    assert!(
        c.compile_program(Path::new("/lib/internal/helper/module.jai"))
            .is_err()
    );
}

#[test]
fn bootstrap_runtime_passes_only_reviewed_parameters() {
    let mut c = compiler(
        &[
            ("/lib/prelude/module.jai", ""),
            (
                "/lib/Runtime_Support/module.jai",
                "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT: bool, DEFINE_INITIALIZATION: bool, ENABLE_BACKTRACE_ON_CRASH: bool);",
            ),
            ("/app/main.jai", ""),
        ],
        true,
    );
    c.options.runtime_support = true;
    succeeds(&mut c);
    assert_eq!(
        c.modules[c.runtime_support.unwrap().0 as usize]
            .params
            .len(),
        3
    );
}
