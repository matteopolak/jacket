//! The help on name-resolution errors: a name a namespaced import provides, members a module
//! lacks, members a type lacks and named arguments no parameter matches. Each help is a
//! suggestion (`jaic::source::Suggestion`), so the rich layouts show the changed code under it.
//! See docs/compiler/diagnostics.md.
use std::path::Path;
use std::process::Command;
use test_support::TempDir;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

fn scratch(name: &str) -> TempDir {
    TempDir::new(name)
}

/// `jaic -check` on `source` in `dir` (as `t.jai`), with the diagnostics layout `layout` (empty
/// for the plain one).
fn check_in(dir: &Path, source: &str, layout: &str) -> String {
    std::fs::write(dir.join("t.jai"), source).unwrap();
    let mut command = Command::new(JAIC);
    command
        .args(["-check", "t.jai", "-no_color"])
        .current_dir(dir)
        .env_remove("FORCE_COLOR")
        .env_remove("CLICOLOR_FORCE")
        .env_remove("NO_COLOR")
        .env_remove("JAIC_DIAGNOSTICS");
    if !layout.is_empty() {
        command.env("JAIC_DIAGNOSTICS", layout);
    }
    String::from_utf8_lossy(&command.output().unwrap().stderr).into_owned()
}

fn check(name: &str, source: &str, layout: &str) -> String {
    check_in(&scratch(name), source, layout)
}

#[track_caller]
fn assert_in_order(text: &str, needles: &[&str]) {
    let mut rest = text;
    for needle in needles {
        match rest.find(needle) {
            Some(at) => rest = &rest[at + needle.len()..],
            None => panic!("expected {needle:?} (in this order) in:\n{text}"),
        }
    }
}

const NAMESPACED: &str =
    "#import \"Basic\";\nM :: #import \"Math\";\nmain :: () {\n    x := sqrt(2.0);\n}\n";

#[test]
fn a_name_a_namespaced_import_exports_is_written_through_it() {
    let plain = check("namespaced-plain", NAMESPACED, "");
    assert_in_order(
        &plain,
        &[
            "t.jai:4:10: error[unknown-identifier]: unknown identifier `sqrt`",
            "help: `sqrt` is in the namespaced import `M`; write `M.sqrt`",
            "help: or bring every name of `Math` into scope with `using M;`",
        ],
    );
    // The module is imported already: no second `#import` and no other module.
    assert!(!plain.contains("add `#import"), "{plain}");
    assert!(!plain.contains("other modules"), "{plain}");
}

#[test]
fn the_namespaced_fixes_show_the_changed_code() {
    let rich = check("namespaced-rich", NAMESPACED, "unicode");
    assert_in_order(
        &rich,
        &[
            "help: `sqrt` is in the namespaced import `M`; write `M.sqrt`",
            " 4 ~     x := M.sqrt(2.0);",
            "help: or bring every name of `Math` into scope with `using M;`",
            " 3 + using M;",
        ],
    );
}

#[test]
fn a_name_no_namespaced_import_exports_still_suggests_the_import() {
    let output = check(
        "namespaced-other",
        "M :: #import \"Math\";\nmain :: () {\n    print(\"x\");\n}\n",
        "",
    );
    assert_in_order(
        &output,
        &["help: `print` is declared in the `Basic` module: add `#import \"Basic\";`"],
    );
    assert!(!output.contains("namespaced import"), "{output}");
}

#[test]
fn a_misspelled_module_member_suggests_the_closest_export() {
    let source = "B :: #import \"Basic\";\nmain :: () {\n    B.printt(\"x\");\n}\n";
    let plain = check("member-typo-plain", source, "");
    assert_in_order(
        &plain,
        &[
            "error[unknown-module-member]: module `Basic` has no exported member `printt`",
            "help: a member with a similar name exists: `print`",
        ],
    );
    let rich = check("member-typo-rich", source, "unicode");
    assert_in_order(&rich, &[" 3 ~     B.print(\"x\");"]);
}

#[test]
fn a_module_member_from_another_module_suggests_that_module() {
    let output = check(
        "member-elsewhere",
        "B :: #import \"Basic\";\nmain :: () {\n    x := B.sqrt(2.0);\n}\n",
        "",
    );
    assert_in_order(
        &output,
        &[
            "error[unknown-module-member]: module `Basic` has no exported member `sqrt`",
            "help: `sqrt` is declared in the `Math` module: add `#import \"Math\";` to this file and write `sqrt` without the `Basic` qualifier",
        ],
    );
}

#[test]
fn a_private_module_member_is_not_exported() {
    let dir = scratch("member-private");
    std::fs::create_dir_all(dir.join("modules/Gadgets")).unwrap();
    std::fs::write(
        dir.join("modules/Gadgets/module.jai"),
        "shown :: 1;\n#scope_file\nhidden :: 2;\n",
    )
    .unwrap();
    let output = check_in(
        &dir,
        "G :: #import \"Gadgets\";\nmain :: () {\n    x := G.hidden;\n}\n",
        "",
    );
    assert_in_order(
        &output,
        &[
            "error[unknown-module-member]: module `Gadgets` has no exported member `hidden`",
            "help: `hidden` is declared in `Gadgets` but is not exported",
        ],
    );
}

const MEMBERS: &str = "Inner :: struct { gamma: int; }\nS :: struct {\n    using inner: Inner;\n    alpha: int;\n    LIMIT :: 3;\n    helper :: (s: S) {}\n}\nmain :: () {\n    s: S;\n    BODY\n}\n";

fn member_check(name: &str, body: &str) -> String {
    check(name, &MEMBERS.replace("BODY", body), "")
}

#[test]
fn a_type_member_suggestion_covers_using_members_constants_and_procedures() {
    assert_in_order(
        &member_check("member-using", "s.gama = 1;"),
        &[
            "error[no-member]: type `S` has no member `gama`",
            "help: a member with a similar name exists: `gamma`",
        ],
    );
    assert_in_order(
        &member_check("member-constant", "x := s.LIMTI;"),
        &["help: a member with a similar name exists: `LIMIT`"],
    );
    assert_in_order(
        &member_check("member-procedure", "s.hepler(s);"),
        &["help: a member with a similar name exists: `helper`"],
    );
}

#[test]
fn an_enum_member_suggests_the_closest_member() {
    let output = check(
        "member-enum",
        "E :: enum { first; second; }\nmain :: () {\n    e := E.frist;\n}\n",
        "",
    );
    assert_in_order(
        &output,
        &[
            "error[no-member]: enum `E` has no member `frist`",
            "help: a member with a similar name exists: `first`",
        ],
    );
}

#[test]
fn an_inferred_member_without_a_type_says_to_name_the_type() {
    let output = check("member-inferred", "main :: () {\n    g := .foo;\n}\n", "");
    assert_in_order(
        &output,
        &[
            "error[cannot-infer-type]: cannot infer the type of `.foo` here",
            "help: nothing here says which type declares `foo`: write `Type.foo`",
        ],
    );
}

const NAMED: &str =
    "f :: (count: int, name: string) {}\nmain :: () {\n    f(cuont = 1, name = \"a\");\n}\n";

#[test]
fn a_misspelled_named_argument_replaces_only_its_name() {
    let plain = check("named-plain", NAMED, "");
    assert_in_order(
        &plain,
        &[
            "error[call-mismatch]: in call to `f`: no parameter named `cuont`",
            "note: parameter `count` is declared here",
            "help: did you mean `count`?",
        ],
    );
    let rich = check("named-rich", NAMED, "unicode");
    assert_in_order(&rich, &[" 3 ~     f(count = 1, name = \"a\");"]);
}

#[test]
fn a_named_argument_far_from_every_parameter_lists_them() {
    let output = check(
        "named-far",
        "f :: (count: int, name: string) {}\nmain :: () {\n    f(qqqqqqq=1);\n}\n",
        "",
    );
    assert_in_order(
        &output,
        &[
            "no parameter named `qqqqqqq`",
            "help: its parameters are `count`, `name`",
        ],
    );
}
