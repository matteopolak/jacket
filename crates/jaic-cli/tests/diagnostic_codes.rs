//! Diagnostics that point at a mistake the language makes easy, and the per-code `-A`, `-W` and
//! `-D` flags that decide how loud a warning is. The codes are described by `jacket explain`.
use std::path::Path;
use std::process::{Command, Output};
use test_support::TempDir;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

/// Writes `files` (name and text) into a fresh folder and checks the first one with `args`.
fn check(name: &str, files: &[(&str, &str)], args: &[&str]) -> Output {
    let dir = TempDir::new(name);
    for (file, text) in files {
        std::fs::write(Path::new(dir.path()).join(file), text).unwrap();
    }
    Command::new(JAIC)
        .current_dir(dir.path())
        .arg(files[0].0)
        .arg("-check")
        .arg("-no_color")
        .args(args)
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

const SHIFTS: &str = "#import \"Basic\";
main :: () {
    low: u8 = 0x12;
    shifted_first := cast(u16) low << 8;
    widened_first := (cast(u16) low) << 8;
    kept_narrow := cast(u16) (low << 8);
    combined := cast(u16) low << 8 | low;
    wide := cast(u64) 1 << 40;
    print(\"%\\n\", shifted_first + widened_first + kept_narrow + combined + cast(u16) wide);
}
";

/// A cast binds more loosely than a shift, so `cast(u16) low << 8` shifts a `u8`.
#[test]
fn a_shift_before_a_widening_cast_warns() {
    let output = check("cast-shift", &[("shift.jai", SHIFTS)], &[]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let text = stderr(&output);
    assert_eq!(
        text.matches("warning[cast-shift-precedence]").count(),
        2,
        "{text}"
    );
    assert!(
        text.contains("shift.jai:4:22: warning[cast-shift-precedence]: this shifts in `u8` before casting to `u16`"),
        "{text}"
    );
    assert!(
        text.contains("shift.jai:7:17: warning[cast-shift-precedence]"),
        "{text}"
    );
    assert!(
        text.contains("help: widen first with `(cast(u16) value) << count`"),
        "{text}"
    );
}

#[test]
fn a_warning_can_be_allowed_or_denied_by_code() {
    let allowed = check(
        "cast-shift-allow",
        &[("shift.jai", SHIFTS)],
        &["-A", "cast-shift-precedence"],
    );
    assert_eq!(allowed.status.code(), Some(0));
    assert!(stderr(&allowed).is_empty(), "{}", stderr(&allowed));

    let denied = check(
        "cast-shift-deny",
        &[("shift.jai", SHIFTS)],
        &["-D", "cast-shift-precedence"],
    );
    assert_eq!(denied.status.code(), Some(1), "{}", stderr(&denied));
    assert!(
        stderr(&denied).contains("error[cast-shift-precedence]:"),
        "{}",
        stderr(&denied)
    );

    // A later flag wins, and `-Dwarnings` is the whole group.
    let later = check(
        "cast-shift-order",
        &[("shift.jai", SHIFTS)],
        &["-Dwarnings", "-A", "cast-shift-precedence"],
    );
    assert_eq!(later.status.code(), Some(0), "{}", stderr(&later));
    let group = check(
        "cast-shift-group",
        &[("shift.jai", SHIFTS)],
        &["-D", "warnings"],
    );
    assert_eq!(group.status.code(), Some(1), "{}", stderr(&group));
}

#[test]
fn the_level_flags_check_their_codes() {
    let typo = check(
        "level-typo",
        &[("shift.jai", SHIFTS)],
        &["-A", "cast-shift-precedense"],
    );
    assert_eq!(typo.status.code(), Some(2), "{}", stderr(&typo));
    assert!(
        stderr(&typo).contains("error[cli-usage]: unknown warning `cast-shift-precedense`\nhelp: did you mean `cast-shift-precedence`?"),
        "{}",
        stderr(&typo)
    );

    let error = check(
        "level-error",
        &[("shift.jai", SHIFTS)],
        &["-A", "type-mismatch"],
    );
    assert_eq!(error.status.code(), Some(2), "{}", stderr(&error));
    assert!(
        stderr(&error).contains("`type-mismatch` is an error"),
        "{}",
        stderr(&error)
    );
}

/// A name the program declares that an import also exports: the warning is at the declaration,
/// and a note points at the other one.
#[test]
fn a_declaration_that_hides_an_import_points_at_the_import() {
    let source =
        "#import \"Basic\";\n#import \"Math\";\nPI :: 3;\nmain :: () { print(\"%\\n\", PI); }\n";
    let output = check("hides-import", &[("hide.jai", source)], &[]);
    let text = stderr(&output);
    assert!(
        text.contains(
            "hide.jai:3:1: warning[declaration-hides-import]: `PI` is also exported by `Math`"
        ),
        "{text}"
    );
    assert!(text.contains("Math/module.jai:"), "{text}");
    assert!(text.contains("note: also declared here"), "{text}");
}

/// Two declarations of one name in a scope: the error names the first.
#[test]
fn a_repeated_declaration_points_at_the_first() {
    let source =
        "#import \"Basic\";\nM :: #import \"Math\";\nM :: 3;\nmain :: () { print(\"%\\n\", M); }\n";
    let output = check("declared-twice", &[("twice.jai", source)], &[]);
    let text = stderr(&output);
    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(
        text.contains(
            "twice.jai:3:1: error[already-declared-scope]: `M` is already declared in this scope"
        ),
        "{text}"
    );
    assert!(
        text.contains("twice.jai:2:1: note: `M` is first declared here"),
        "{text}"
    );
}

/// A `#scope_file` name is private to its file even when another file `#load`s that one.
#[test]
fn a_scope_file_name_used_from_a_loading_file_suggests_scope_module() {
    let helper = "#scope_file\nclamp_index :: (i: s64) -> s64 { return i; }\n";
    let main = "#import \"Basic\";\n#load \"helper.jai\";\nmain :: () { print(\"%\\n\", clamp_index(3)); }\n";
    let output = check(
        "scope-file",
        &[("main.jai", main), ("helper.jai", helper)],
        &[],
    );
    let text = stderr(&output);
    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(
        text.contains("main.jai:3:27: error[scope-file-name]: unknown identifier `clamp_index`"),
        "{text}"
    );
    assert!(
        text.contains("helper.jai:2:1: note: `clamp_index` is declared here, under `#scope_file`"),
        "{text}"
    );
    assert!(text.contains("help: `#scope_file` names are only visible in `helper.jai`: declare `clamp_index` under `#scope_module`"), "{text}");
}
