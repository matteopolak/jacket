//! Controls for anonymous imports in loaded files.

use test_support::compiler::compile_virtual_rendered;

fn compile(piece: &str, main: &str) -> Result<(), String> {
    let dependency =
        "#module_parameters(N: int = 42); Token :: struct { value: int; } answer :: N;";

    compile_virtual_rendered(
        &[
            ("/lib/Dependency/module.jai", dependency),
            ("/app/dependency.jai", dependency),
            ("/app/piece.jai", piece),
            ("/app/main.jai", main),
        ],
        "/app/main.jai",
        |options| options.import_paths = vec!["/lib".into()],
    )
}

#[test]
fn anonymous_import_after_file_scope_is_visible_to_loading_file() {
    compile(
        "#scope_file #import \"Dependency\"; #assert answer == 42;",
        "#load \"piece.jai\"; #assert answer == 42; token: Token; main :: () {}",
    )
    .unwrap();
}

#[test]
fn named_import_after_file_scope_stays_private() {
    let error = compile(
        "#scope_file Local :: #import \"Dependency\"; #assert Local.answer == 42;",
        "#load \"piece.jai\"; token: Local.Token; main :: () {}",
    )
    .unwrap_err();

    assert!(error.contains("unknown identifier `Local`"), "{error}");
}

#[test]
fn ordinary_file_private_constants_stay_private() {
    let error = compile(
        "#scope_file hidden :: 42; #assert hidden == 42;",
        "#load \"piece.jai\"; #assert hidden == 42; main :: () {}",
    )
    .unwrap_err();

    assert!(error.contains("unknown identifier `hidden`"), "{error}");
}

#[test]
fn absent_import_does_not_supply_names() {
    let error = compile(
        "#scope_file unrelated :: 42;",
        "#load \"piece.jai\"; token: Token; main :: () {}",
    )
    .unwrap_err();

    assert!(error.contains("unknown identifier `Token`"), "{error}");
}

#[test]
fn anonymous_import_keeps_relative_path_and_private_parameter_context() {
    compile(
        "#scope_file seed :: 41; #import,file \"dependency.jai\"(N=seed);",
        "#load \"piece.jai\"; #assert answer == 41; main :: () {}",
    )
    .unwrap();
}

#[test]
fn named_private_using_import_does_not_expose_members_to_loading_file() {
    let error = compile(
        "#scope_file using Local :: #import \"Dependency\"; #assert answer == 42;",
        "#load \"piece.jai\"; #assert answer == 42; main :: () {}",
    )
    .unwrap_err();

    assert!(error.contains("unknown identifier `answer`"), "{error}");
}

#[test]
fn anonymous_file_scope_import_preserves_member_filter() {
    let piece = "#scope_file using,only(Token) #import \"Dependency\";";

    compile(piece, "#load \"piece.jai\"; token: Token; main :: () {}").unwrap();
    let error = compile(
        piece,
        "#load \"piece.jai\"; #assert answer == 42; main :: () {}",
    )
    .unwrap_err();

    assert!(error.contains("unknown identifier `answer`"), "{error}");
}

#[test]
fn anonymous_file_scope_using_import_does_not_become_module_reexport() {
    let error = compile(
        "#scope_file using #import \"Dependency\"; #assert answer == 42;",
        "Wrapper :: #import,file \"piece.jai\"; #assert Wrapper.answer == 42; main :: () {}",
    )
    .unwrap_err();

    assert!(error.contains("answer"), "{error}");
}
