//! Runtime checks in native builds.
use super::{build_and_run, exe_path, jaic, repo_root};
use std::path::Path;
use std::process::Command;

/// Arithmetic overflow checks in a native build: `Build_Options.arithmetic_overflow_check` on a
/// workspace that writes an executable, and `#no_aoc` switching them off again.
#[test]
fn arithmetic_overflow_checks() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-overflow");
    std::fs::create_dir_all(&dir).unwrap();
    // (name, mode, body of main, run succeeds, stdout, text stderr must contain)
    let cases: [(&str, &str, &str, bool, &str, &str); 8] = [
        (
            "off",
            "OFF",
            "x: u8 = 250; y: u8 = 10; print(\"%\\n\", x + y);",
            true,
            "4\n",
            "",
        ),
        (
            "nonfatal",
            "NONFATAL",
            "x: u8 = 250; y: u8 = 10; print(\"%\\n\", x + y);",
            true,
            "4\n",
            "arithmetic overflow computing 250 + 10 as u8",
        ),
        (
            "fatal-add",
            "FATAL",
            "x: u8 = 250; y: u8 = 10; print(\"%\\n\", x + y);",
            false,
            "",
            "arithmetic overflow computing 250 + 10 as u8",
        ),
        (
            "fatal-sub",
            "FATAL",
            "x: s64 = -0x7fff_ffff_ffff_ffff; y: s64 = 5; print(\"%\\n\", x - y);",
            false,
            "",
            "as s64",
        ),
        (
            "fatal-mul",
            "FATAL",
            "x: u32 = 70000; print(\"%\\n\", x * x);",
            false,
            "",
            "70000 * 70000 as u32",
        ),
        (
            "in-range",
            "FATAL",
            "x: s16 = 32000; y: s16 = 767; a: u64 = 3; print(\"% %\\n\", x + y, a - 3);",
            true,
            "32767 0\n",
            "",
        ),
        (
            "no-aoc-block",
            "FATAL",
            "x: u8 = 250; y: u8 = 10; z: u8; #no_aoc { z = x + y; } print(\"%\\n\", z);",
            true,
            "4\n",
            "",
        ),
        (
            "no-aoc-loop",
            "FATAL",
            "x: u8 = 250; for 1..2 #no_aoc { x += 10; } print(\"%\\n\", x);",
            true,
            "14\n",
            "",
        ),
    ];
    for (name, mode, body, ok, stdout, stderr) in cases {
        let meta = dir.join(format!("{name}.jai"));
        std::fs::write(
            &meta,
            format!(
                r##"#import "Basic";
#import "Compiler";

SOURCE :: #string END
#import "Basic";
main :: () {{
    {body}
}}
END

#run {{
    set_build_options_dc(.{{do_output = false}});
    w := compiler_create_workspace("prog");
    options := get_build_options(w);
    options.output_type = .EXECUTABLE;
    options.output_executable_name = "{name}-prog";
    options.output_path = ".";
    options.arithmetic_overflow_check = .{mode};
    set_build_options(options, w);
    add_build_string(SOURCE, w);
}}
"##
            ),
        )
        .unwrap();
        let build = jaic().arg(&meta).current_dir(&dir).output().unwrap();
        assert!(
            build.status.success(),
            "{name}: build failed: {}",
            String::from_utf8_lossy(&build.stderr)
        );
        let run = Command::new(exe_path(&dir, &format!("{name}-prog")))
            .output()
            .unwrap();
        let err = String::from_utf8_lossy(&run.stderr);
        assert_eq!(
            run.status.success(),
            ok,
            "{name}: status {:?}\n{err}",
            run.status
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), stdout, "{name}");
        assert!(err.contains(stderr), "{name}: stderr was {err:?}");
        if stderr.is_empty() {
            assert!(
                !err.contains("overflow"),
                "{name}: unexpected report {err:?}"
            );
        }
    }
}

/// A failed check in a built executable says what failed and where (through Runtime_Support's
/// `runtime_support_check_failed`), with the interpreter's wording, before it stops.
// rules: ptr.18 cast.33 flow.27
#[test]
fn failed_checks_say_what_and_where() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-checks");
    std::fs::create_dir_all(&dir).unwrap();
    // (name, body of main, the report's line, its message)
    let cases: [(&str, &str, u32, &str); 6] = [
        (
            "bounds",
            "a: [3] int;\n    i := 5 + a[0];\n    print(\"%\\n\", a[i]);",
            7,
            "array bounds check failed: index 5 is outside an array of 3 elements",
        ),
        (
            "cast",
            "w := 300 + get_command_line_arguments().count;\n    c := cast(u8) w;\n    print(\"%\\n\", c);",
            6,
            "cast of 301 to `u8` overflows",
        ),
        (
            "switch",
            "c := cast(Color) (6 + get_command_line_arguments().count);\n    if #complete c == {\n        case .RED; print(\"red\\n\");\n        case .GREEN; print(\"green\\n\");\n    }",
            6,
            "no case of the `#complete` switch matches its value, 7",
        ),
        (
            "divide",
            "z := get_command_line_arguments().count - 1;\n    print(\"%\\n\", 7 / z);",
            6,
            "integer division by zero",
        ),
        (
            "null_read",
            "p: *int;\n    v := p.*;\n    print(\"%\\n\", v);",
            6,
            "null pointer dereference: read through a null pointer",
        ),
        (
            "null_member",
            "p: *Pair;\n    v := p.b;\n    print(\"%\\n\", v);",
            6,
            "null pointer dereference: read at address 0x8, just past null (a member of a null struct pointer?)",
        ),
    ];
    for (name, body, line, message) in cases {
        let source = dir.join(format!("{name}.jai"));
        std::fs::write(
            &source,
            format!("#import \"Basic\";\nColor :: enum {{ RED; GREEN; }}\nPair :: struct {{ a: int; b: int; }}\nmain :: () {{\n    {body}\n}}\n"),
        )
        .unwrap();
        let output = build_and_run(&source, &dir, name).unwrap();
        let err = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "{name}: ran to the end\n{err}");
        assert!(
            err.contains(&format!("{name}.jai:{line}: error: {message}\n")),
            "{name}: stderr was {err:?}"
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), "", "{name}");
    }

    // An `#asm` divide fault is reported even when nothing else in the program has a check
    // (no Basic, no indexing), so it is the only code that needs the reporting procedure.
    let source = dir.join("asm_divide.jai");
    std::fs::write(
        &source,
        "divisor: u64;\nmain :: () {\n    hi: u64 = 0;\n    lo: u64 = 7;\n    d := divisor;\n    #asm { div hi, lo, d; }\n}\n",
    )
    .unwrap();
    let output = build_and_run(&source, &dir, "asm_divide").unwrap();
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "asm_divide: ran to the end\n{err}"
    );
    assert!(
        err.contains("asm_divide.jai:6: error: #asm division fault: the divisor is zero or the quotient does not fit\n"),
        "asm_divide: stderr was {err:?}"
    );

    // .NONFATAL reports a warning and goes on with the low bits.
    let meta = dir.join("nonfatal.jai");
    std::fs::write(
        &meta,
        r##"#import "Compiler";

SOURCE :: #string END
#import "Basic";
main :: () {
    w := 300 + get_command_line_arguments().count;
    print("%\n", cast(u8) w);
}
END

#run {
    set_build_options_dc(.{do_output = false});
    w := compiler_create_workspace("prog");
    options := get_build_options(w);
    options.output_type = .EXECUTABLE;
    options.output_executable_name = "nonfatal-prog";
    options.output_path = ".";
    options.cast_bounds_check = .NONFATAL;
    set_build_options(options, w);
    add_build_string(SOURCE, w);
}
"##,
    )
    .unwrap();
    let build = jaic().arg(&meta).current_dir(&dir).output().unwrap();
    assert!(
        build.status.success(),
        "nonfatal: build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    let run = Command::new(exe_path(&dir, "nonfatal-prog"))
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&run.stderr);
    assert!(run.status.success(), "nonfatal: {err}");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "45\n");
    assert!(
        err.contains(":4: warning: cast of 301 to `u8` overflows\n"),
        "nonfatal: stderr was {err:?}"
    );
}

/// A pointer that passed a null check stays checked within a block only while nothing can have
/// changed it: storing to its variable, or writing it through an address, brings the check back.
/// Everything the lowering skips here still runs (`NullFacts` in `crates/jaic-llvm/src/lower.rs`).
#[test]
fn null_checks_skipped_after_a_check_still_catch_changes() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-null-elision");
    std::fs::create_dir_all(&dir).unwrap();
    // (name, body of main, the failing line (0: runs to the end), stdout)
    let cases: [(&str, &str, u32, &str); 6] = [
        (
            "fields",
            "x := Pair.{a = 1, b = 2};\n    p := *x;\n    v := p.a + p.b + p.a + p.b;\n    print(\"%\\n\", v);",
            0,
            "6\n",
        ),
        (
            "store_nulls_the_variable",
            "x := Pair.{a = 1, b = 2};\n    p := *x;\n    v := p.a;\n    p = null;\n    w := p.b;\n    print(\"%\\n\", v + w);",
            8,
            "",
        ),
        (
            "copy_keeps_the_old_pointer",
            "x := Pair.{a = 1, b = 2};\n    p := *x;\n    q := p;\n    v := p.a;\n    p = null;\n    w := q.b;\n    z := p.a;\n    print(\"%\\n\", v + w + z);",
            10,
            "",
        ),
        (
            "written_through_its_address",
            "x := Pair.{a = 1, b = 2};\n    p := *x;\n    v := p.a;\n    pp := *p;\n    pp.* = null;\n    w := p.b;\n    print(\"%\\n\", v + w);",
            9,
            "",
        ),
        (
            "nulled_in_a_loop",
            "x := Pair.{a = 1, b = 2};\n    p := *x;\n    sum := 0;\n    for 0..2 {\n        sum += p.a;\n        if it == 1 p = null;\n    }\n    print(\"%\\n\", sum);",
            8,
            "",
        ),
        (
            "offset_from_a_nulled_pointer",
            "x := Pair.{a = 1, b = 2};\n    p := *x;\n    v := p.a;\n    p = null;\n    w := p.a + p.b;\n    print(\"%\\n\", v + w);",
            8,
            "",
        ),
    ];
    for (name, body, line, stdout) in cases {
        let source = dir.join(format!("{name}.jai"));
        std::fs::write(
            &source,
            format!("#import \"Basic\";\nPair :: struct {{ a: int; b: int; }}\nmain :: () {{\n    {body}\n}}\n"),
        )
        .unwrap();
        let output = build_and_run(&source, &dir, name).unwrap();
        let err = String::from_utf8_lossy(&output.stderr);
        assert_eq!(String::from_utf8_lossy(&output.stdout), stdout, "{name}");
        if line == 0 {
            assert!(output.status.success(), "{name}: failed\n{err}");
        } else {
            assert!(!output.status.success(), "{name}: ran to the end\n{err}");
            assert!(
                err.contains(&format!(
                    "{name}.jai:{line}: error: null pointer dereference"
                )),
                "{name}: stderr was {err:?}"
            );
        }
    }
}

/// Window programs using Simp's automatic GL context creation type-check for every desktop
/// OS (the GLX/WGL paths cannot run here, but they must keep compiling).
#[test]
fn simp_window_program_checks_on_desktop_oses() {
    let source = repo_root().join("stdlib/Simp/tests/window-program.jai");
    for os in ["linux", "windows", "macos"] {
        let output = jaic()
            .arg("-check")
            .arg(&source)
            .args(["-os", os])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{os}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
