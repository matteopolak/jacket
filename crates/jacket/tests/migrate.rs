//! `jacket migrate`: a dry run prints the diff and writes nothing; `--apply` writes only what
//! git could give back.
use std::path::Path;
use std::process::{Command, Output};

use test_support::repo::stdlib;

const OLD: &str = "#import \"Basic\";\nmain :: () {\n    c: Context;\n    print(\"%\\n\", size_of(Context));\n}\n";
const NEW: &str = "#import \"Basic\";\nmain :: () {\n    c: #Context;\n    print(\"%\\n\", size_of(#Context));\n}\n";


/// A scratch directory of its own, so tests do not share files.
fn scratch(name: &str) -> test_support::TempDir {
    let dir = test_support::TempDir::new(&format!("jacket-test-{name}"));

    std::fs::create_dir_all(&dir).expect("create the scratch directory");
    dir
}

fn migrate(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jacket"))
        .arg("migrate")
        .args(args)
        .current_dir(dir)
        .env("JAIC_STDLIB", stdlib())
        .output()
        .expect("run jacket")
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args([
            "-c",
            "user.name=test",
            "-c",
            "user.email=test@example.com",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(dir)
        .status()
        .expect("run git");

    assert!(status.success(), "git {args:?}");
}

/// A git repository holding `main.jai` with `text`, committed.
fn repository(name: &str, text: &str) -> test_support::TempDir {
    let dir = scratch(name);

    std::fs::write(dir.join("main.jai"), text).unwrap();
    git(&dir, &["init", "-q"]);
    git(&dir, &["add", "main.jai"]);
    git(&dir, &["commit", "-q", "-m", "init"]);
    dir
}

#[test]
fn a_dry_run_prints_the_diff_and_writes_nothing() {
    let dir = repository("migrate-dry", OLD);
    let out = migrate(&dir, &[]);

    assert!(out.status.success(), "{}", text(&out));
    let shown = text(&out);

    assert!(shown.contains("\n-    c: Context;\n"), "{shown}");
    assert!(shown.contains("\n+    c: #Context;\n"), "{shown}");
    assert!(
        shown.contains("main.jai:3:8: context-type-directive"),
        "{shown}"
    );
    assert!(shown.contains("nothing written"), "{shown}");
    assert_eq!(std::fs::read_to_string(dir.join("main.jai")).unwrap(), OLD);
}

#[test]
fn apply_writes_files_git_can_give_back() {
    let dir = repository("migrate-apply", OLD);
    let out = migrate(&dir, &["--apply"]);

    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(std::fs::read_to_string(dir.join("main.jai")).unwrap(), NEW);
    let again = migrate(&dir, &[]);

    assert!(
        text(&again).contains("nothing to migrate"),
        "{}",
        text(&again)
    );
}

#[test]
fn apply_refuses_uncommitted_changes_unless_allowed() {
    let dir = repository("migrate-dirty", OLD);
    let edited = format!("{OLD}// mine\n");

    std::fs::write(dir.join("main.jai"), &edited).unwrap();
    let out = migrate(&dir, &["--apply"]);

    assert_eq!(out.status.code(), Some(2), "{}", text(&out));
    assert!(
        text(&out).contains("commit or stash them first"),
        "{}",
        text(&out)
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("main.jai")).unwrap(),
        edited
    );
    // Staged counts too: git could not tell the staged text from the migration's.
    git(&dir, &["add", "main.jai"]);
    assert_eq!(migrate(&dir, &["--apply"]).status.code(), Some(2));
    let out = migrate(&dir, &["--apply", "--allow-dirty"]);

    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(
        std::fs::read_to_string(dir.join("main.jai")).unwrap(),
        format!("{NEW}// mine\n")
    );
}

#[test]
fn apply_refuses_untracked_files() {
    let dir = repository("migrate-untracked", "main :: () {}\n");

    std::fs::write(dir.join("old.jai"), OLD).unwrap();
    let out = migrate(&dir, &["old.jai", "--apply"]);

    assert_eq!(out.status.code(), Some(2), "{}", text(&out));
    assert_eq!(std::fs::read_to_string(dir.join("old.jai")).unwrap(), OLD);
}

#[test]
fn apply_outside_git_needs_allow_no_vcs() {
    let dir = scratch("migrate-no-vcs");

    std::fs::write(dir.join("main.jai"), OLD).unwrap();
    let out = migrate(&dir, &["--apply"]);

    assert_eq!(out.status.code(), Some(2), "{}", text(&out));
    assert!(text(&out).contains("--allow-no-vcs"), "{}", text(&out));
    let out = migrate(&dir, &["--apply", "--allow-no-vcs"]);

    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(std::fs::read_to_string(dir.join("main.jai")).unwrap(), NEW);
}

#[test]
fn a_mistyped_migration_name_gets_a_suggestion() {
    let dir = scratch("migrate-only");

    std::fs::write(dir.join("main.jai"), OLD).unwrap();
    let out = migrate(&dir, &["--only", "context-type-directiv"]);

    assert_eq!(out.status.code(), Some(2));
    assert!(
        text(&out).contains("did you mean `context-type-directive`?"),
        "{}",
        text(&out)
    );
}

#[test]
fn list_names_every_migration() {
    let out = migrate(Path::new("."), &["--list"]);

    assert!(out.status.success());
    assert!(
        text(&out).contains("context-type-directive"),
        "{}",
        text(&out)
    );
}

/// `old` migrated with `--apply --allow-no-vcs`, with what the command printed.
fn migrated(name: &str, old: &str) -> (String, String) {
    let dir = scratch(name);

    std::fs::write(dir.join("main.jai"), old).unwrap();
    let out = migrate(&dir, &["--apply", "--allow-no-vcs"]);

    assert!(out.status.success(), "{}", text(&out));
    (
        std::fs::read_to_string(dir.join("main.jai")).unwrap(),
        text(&out),
    )
}

#[test]
fn each_migration_rewrites_its_old_form() {
    let old = r#"#import "Basic";
#import "Math";
M :: #import "Math";
libc :: #foreign_system_library "libc";
main :: () {
    v := make_vector2(1, 2);
    w := M.make_vector3(1, 2, 3);
    p := New(int, allocator = temp);
    s := copy_string("x", allocator = temp);
}
"#;
    let new = r#"#import "Basic";
#import "Math";
M :: #import "Math";
libc :: #system_library "libc";
main :: () {
    v := Vector2.{1, 2};
    w := M.Vector3.{1, 2, 3};
    p := New(int,, allocator = temp);
    s := copy_string("x",, allocator = temp);
}
"#;
    let (after, shown) = migrated("migrate-each", old);

    assert_eq!(after, new, "{shown}");
}

#[test]
fn code_that_is_valid_today_is_left_alone() {
    // Its own `Context`, a `make_vector3` overload that still exists, and an allocator that is
    // already a context argument.
    let old = r#"#import "Basic";
#import "Math";
Context :: struct { depth: int; }
main :: () {
    c: Context;
    v := make_vector3(Vector2.{1, 2}, 5);
    p := New(int,, allocator = temp);
}
"#;
    let (after, shown) = migrated("migrate-valid", old);

    assert_eq!(after, old, "{shown}");
    assert!(shown.contains("nothing to migrate"), "{shown}");
}

/// `old` migrates to `new`, and `new` is then left alone.
fn assert_migrates(name: &str, old: &str, new: &str) {
    let (after, shown) = migrated(name, old);

    assert_eq!(after, new, "{shown}");
    let (again, shown) = migrated(&format!("{name}-again"), new);

    assert_eq!(again, new, "second run changed the code: {shown}");
    assert!(shown.contains("nothing to migrate"), "{shown}");
}

#[test]
fn renamed_basic_procedures_take_their_new_names() {
    assert_migrates(
        "migrate-renamed",
        r#"#import "Basic";
#import "Sort";
main :: () {
    a := 1;
    b := 2;
    Swap(*a, *b);
    t := get_time();
    words := string.["b", "a"];
    bubble_sort(words, compare_strings);
    context.allocator = __temporary_allocator;
    args := compiler_get_command_line_arguments();
}
"#,
        r#"#import "Basic";
#import "Sort";
main :: () {
    a := 1;
    b := 2;
    swap(*a, *b);
    t := seconds_since_init();
    words := string.["b", "a"];
    quick_sort(words, compare_strings);
    context.allocator = temp;
    args := get_command_line_arguments();
}
"#,
    );
}

#[test]
fn a_procedure_the_program_declares_keeps_its_name() {
    let old = r#"#import "Basic";
Swap :: (a: *int, b: *int) { t := a.*; a.* = b.*; b.* = t; }
get_time :: () -> float64 { return 1; }
main :: () {
    a := 1;
    b := 2;
    Swap(*a, *b);
    t := get_time();
}
"#;
    let (after, shown) = migrated("migrate-own-names", old);

    assert_eq!(after, old, "{shown}");
}

#[test]
fn the_build_options_output_type_and_message_kind_are_renamed() {
    assert_migrates(
        "migrate-compiler-names",
        r#"#import "Compiler";
main :: () {
    options := get_build_options();
    options.output_type = Build_Output_Type.NO_OUTPUT;
}
handle :: (message: *Message) {
    if message.kind == .COMPILATION_PHASE return;
}
"#,
        r#"#import "Compiler";
main :: () {
    options := get_build_options();
    options.output_type = Build_Options.Output_Type.NO_OUTPUT;
}
handle :: (message: *Message) {
    if message.kind == .PHASE return;
}
"#,
    );
}

#[test]
fn file_list_follows_directory_symlinks_by_its_new_name() {
    assert_migrates(
        "migrate-file-list",
        "#import \"File_Utilities\";\nmain :: () {\n    files := file_list(\".\", recursive = true, follow_symlinks = false);\n}\n",
        "#import \"File_Utilities\";\nmain :: () {\n    files := file_list(\".\", recursive = true, follow_directory_symlinks = false);\n}\n",
    );
}

#[test]
fn an_assert_message_is_positional() {
    assert_migrates(
        "migrate-assert-message",
        "#assert(OS == .MACOS || OS != .MACOS, message = \"unreachable\");\nmain :: () {}\n",
        "#assert(OS == .MACOS || OS != .MACOS, \"unreachable\");\nmain :: () {}\n",
    );
}

#[test]
fn vector_arguments_that_start_with_a_cast_are_rewritten() {
    assert_migrates(
        "migrate-vector-cast",
        r#"#import "Math";
main :: () {
    w := 640;
    a := make_vector2(cast(float) w / 2, cast(float) w / 2);
    b := make_vector2(xx (w / 2), xx (w / 2));
    f := (v: Vector2) -> Vector2 { return v; };
    c := f(make_vector2(cast(float) w, 1));
}
"#,
        r#"#import "Math";
main :: () {
    w := 640;
    a := Vector2.{cast(float) w / 2, cast(float) w / 2};
    b := Vector2.{xx (w / 2), xx (w / 2)};
    f := (v: Vector2) -> Vector2 { return v; };
    c := f(Vector2.{cast(float) w, 1});
}
"#,
    );
}

#[test]
fn a_removed_module_gets_a_note_and_no_rewrite() {
    let old = "#import \"Basic\";\n#import \"Render\";\nmain :: () {}\n";
    let (after, shown) = migrated("migrate-removed-module", old);

    assert_eq!(after, old);
    assert!(
        shown.contains("main.jai:2:1: removed-modules: the `Render` module was removed"),
        "{shown}"
    );
}

#[test]
fn a_file_that_is_not_utf8_is_left_alone_with_a_note() {
    let dir = scratch("migrate-not-utf8");

    std::fs::write(dir.join("main.jai"), format!("#load \"other.jai\";\n{OLD}")).unwrap();
    std::fs::write(dir.join("other.jai"), b"x :: \"\xff\";\n").unwrap();
    let out = migrate(&dir, &["--apply", "--allow-no-vcs", "main.jai"]);

    assert!(out.status.success(), "{}", text(&out));
    assert!(
        text(&out).contains("note: `other.jai` is not UTF-8 text"),
        "{}",
        text(&out)
    );
    let main = std::fs::read_to_string(dir.join("main.jai")).unwrap();

    assert!(main.contains("c: #Context;"), "{main}");
}

#[test]
fn an_allocator_that_is_the_first_argument_moves_to_the_context() {
    assert_migrates(
        "migrate-allocator-first",
        r#"#import "Basic";
main :: () {
    a := make_text(allocator = temp);
    g(allocator = temp);
    c := f(allocator = temp, n = 2);
}
make_text :: () -> string { return ""; }
g :: () {}
f :: (n := 1) -> int { return n; }
"#,
        r#"#import "Basic";
main :: () {
    a := make_text(,, allocator = temp);
    g(,, allocator = temp);
    c := f(n = 2,, allocator = temp);
}
make_text :: () -> string { return ""; }
g :: () {}
f :: (n := 1) -> int { return n; }
"#,
    );
}

#[test]
fn arguments_after_the_allocator_stay_ordinary_arguments() {
    assert_migrates(
        "migrate-allocator-order",
        r#"#import "Basic";
main :: () {
    p := New(int, allocator = temp, initialized = false);
}
"#,
        r#"#import "Basic";
main :: () {
    p := New(int, initialized = false,, allocator = temp);
}
"#,
    );
}
