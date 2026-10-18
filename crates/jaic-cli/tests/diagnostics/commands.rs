//! Input file, build and command line mistakes and rich layouts.
use super::{JAIC, assert_in_order, jaic, jaic_on, scratch, stderr};
use std::process::Command;

#[test]
fn input_file_problems() {
    let dir = scratch("input");
    std::fs::write(dir.join("game.jai"), "main :: () {}\n").unwrap();
    std::fs::create_dir_all(dir.join("proj")).unwrap();
    std::fs::write(dir.join("proj/first.jai"), "main :: () {}\n").unwrap();
    let cases: &[(&[&str], &[&str])] = &[
        (
            &["-interp", "gmae.jai"],
            &[
                "error: file `gmae.jai` does not exist",
                "help: did you mean `game.jai`?",
            ],
        ),
        (
            &["-interp", "game"],
            &[
                "error: file `game` does not exist",
                "help: did you mean `game.jai`?",
            ],
        ),
        (
            &["-interp", "nothing_like_it.jai"],
            &[
                "error: file `nothing_like_it.jai` does not exist",
                "help: relative paths start from the current directory",
            ],
        ),
        (
            &["-check", "proj"],
            &[
                "error: `proj` is a directory, not a .jai file",
                "help: did you mean `proj/first.jai`?",
            ],
        ),
    ];
    for (args, expected) in cases {
        let output = jaic(&dir, args, &[]);
        assert_eq!(output.status.code(), Some(1), "{args:?}");
        let text = stderr(&output);
        assert_in_order(&text, expected);
        // No snippet of some unrelated file (the stdlib's Preload used to be shown).
        assert!(!text.contains("Preload"), "{text}");
    }
}

#[test]
fn build_problems_say_what_to_change() {
    let dir = scratch("build");
    std::fs::write(dir.join("ok.jai"), "main :: () {}\n").unwrap();
    std::fs::write(dir.join("lib.jai"), "f :: () {}\n").unwrap();
    std::fs::create_dir_all(dir.join("out")).unwrap();
    let cases: &[(&[&str], Option<i32>, &[&str])] = &[
        (
            &["lib.jai"],
            Some(3),
            &[
                "error: `lib.jai` has no `main` procedure, so there is no program to write",
                "help: add `main :: () { ... }`",
            ],
        ),
        (
            &["ok.jai", "-exe", "out"],
            Some(1),
            &[
                "error: the output path `out` is a directory",
                "help: name the file to write inside it, as in `-exe out/program`",
            ],
        ),
        (
            &["ok.jai", "-sanitize", "adress"],
            Some(2),
            &[
                "error: unknown sanitizer `adress` for `-sanitize`",
                "help: use address, undefined, fuzzer or a list: `-sanitize address,fuzzer`",
            ],
        ),
    ];
    for (args, code, expected) in cases {
        let output = jaic(&dir, args, &[]);
        let text = stderr(&output);
        assert_eq!(output.status.code(), *code, "{args:?}: {text}");
        assert_in_order(&text, expected);
    }
    // A linker that is not installed: how to get one, and that `-interp` needs none.
    let output = Command::new(JAIC)
        .args(["ok.jai", "-exe", "prog"])
        .current_dir(&dir)
        .env("JAIC_LINKER", "no-such-linker-for-jaic-tests")
        .env_remove("JAIC_DIAGNOSTICS")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_in_order(
        &stderr(&output),
        &[
            "error: could not find the linker `no-such-linker-for-jaic-tests`",
            "help: ",
            "JAIC_LINKER",
            "help: `jaic file.jai -interp` needs no linker",
        ],
    );
    // An undeclared `#foreign` library.
    let output = jaic_on(
        &dir,
        "f.jai",
        "f :: () #foreign nothere;\nmain :: () { f(); }\n",
        "-check",
        &[],
    );
    assert_in_order(
        &stderr(&output),
        &[
            "f.jai:1:18: error: unknown library `nothere`",
            "help: declare it with `nothere :: #library",
        ],
    );
}

#[test]
fn command_line_mistakes_explain_themselves() {
    let dir = scratch("cli");
    std::fs::write(dir.join("ok.jai"), "main :: () {}\n").unwrap();
    std::fs::write(dir.join("two.jai"), "main :: () {}\n").unwrap();
    let cases: &[(&[&str], &[&str])] = &[
        (
            &[],
            &[
                "error: no file to compile",
                "help: usage: jaic [options] file.jai [options]",
                "help: `jaic -help` lists the options",
            ],
        ),
        (&["-check"], &["error: no file to compile"]),
        (
            &["ok.jai", "-os"],
            &["error: `-os` needs an OS name: linux, windows, macos or wasm"],
        ),
        (
            &["ok.jai", "-os", "linus"],
            &[
                "error: unknown OS `linus` for `-os`",
                "help: did you mean `-os linux`?",
            ],
        ),
        (
            &["ok.jai", "-cpu", "z80"],
            &["error: unknown CPU `z80` for `-cpu`"],
        ),
        (&["ok.jai", "-I"], &["error: `-I` needs a directory"]),
        (
            &["ok.jai", "-timing"],
            &[
                "error: unknown option `-timing`",
                "help: did you mean `-timings`?",
            ],
        ),
        (
            &["ok.jai", "-interp", "-o"],
            &["error: `-o` only applies to a build, not `-interp`"],
        ),
        (
            &["ok.jai", "-check", "-exe", "x"],
            &["error: `-exe` only applies to a build, not `-check`"],
        ),
        (
            &["ok.jai", "-o", "x"],
            &[
                "error: `-o` takes no value since 0.17.0",
                "help: it asks for an optimized build; name the output with `-exe name` and `-output_path dir`",
                "help: to keep the old meaning of `-o x`, use `-exe x`",
            ],
        ),
        (
            &["ok.jai", "-O7"],
            &[
                "error: unknown optimization level `-O7`",
                "help: use -O0, -O1, -O2 or -O3",
            ],
        ),
        (
            &["ok.jai", "-o", "-O2"],
            &["error: `-O0`..`-O3` and `-o`, `-od` or `-very_debug` both set the optimization"],
        ),
        (
            &["ok.jai", "-interp", "extra"],
            &[
                "error: unexpected argument `extra`",
                "help: arguments for the program go after `--`: `jaic ok.jai -interp -- extra`",
            ],
        ),
        (
            &["ok.jai", "two.jai"],
            &[
                "error: jaic compiles one file, but got `ok.jai` and `two.jai`",
                "help: `#load` or `#import` the others from the first",
            ],
        ),
        (
            &["ok.jai", "-check", "-interp"],
            &["error: `-check` and `-interp` cannot be combined"],
        ),
        (
            &["ok.jai", "-interp", "+X"],
            &[
                "error: plugins work with a build or `-check`, not `-interp`",
                "help: use `jaic ok.jai +X ...` and run the executable",
            ],
        ),
        (
            &["ok.jai", "--", "a"],
            &[
                "error: `--` hands arguments to the program, which only `-interp` runs",
                "help: use `jaic ok.jai -interp -- ...`",
            ],
        ),
        (
            &["ok.jai", "-check", "-", "--", "a"],
            &["error: `--` hands arguments to the program, which only `-interp` runs"],
        ),
        (
            &["ok.jai", "-no_workspace_output"],
            &["error: `-no_workspace_output` only applies to `-interp`"],
        ),
        (
            &["ok.jai", "-quiet", "-verbose"],
            &["error: `-quiet` and `-verbose` contradict each other"],
        ),
        // Old spellings name their replacements, unless a plugin is given them.
        (
            &["ok.jai", "--emit-ir", "x.ll"],
            &[
                "error: `--emit-ir` is spelled differently since 0.17.0",
                "help: use -emit_ir <file.ll>",
            ],
        ),
        (
            &["ok.jai", "--timings"],
            &[
                "error: `--timings` is spelled differently",
                "help: use -timings",
            ],
        ),
        (
            &["ok.jai", "--no-debug-info"],
            &["help: use -no_debug_info"],
        ),
        (
            &["ok.jai", "--color", "never"],
            &["help: use -no_color to turn colour off"],
        ),
        (
            &["ok.jai", "-color=never"],
            &["error: `-color` is spelled differently"],
        ),
        (
            &["ok.jai", "-plug", "X"],
            &[
                "error: `-plug` is spelled differently",
                "help: use +Name after the file",
            ],
        ),
        (
            &["ok.jai", "-plugin", "X"],
            &["error: `-plugin` is spelled differently"],
        ),
        (
            &["ok.jai", "--sanitize", "address"],
            &["help: use -sanitize <list>"],
        ),
        (
            &["ok.jai", "--target", "x"],
            &["help: use -target <triple>"],
        ),
        // The old subcommands: the same command in the new form, other arguments carried over.
        (
            &["run", "ok.jai", "--", "a"],
            &[
                "error: jaic has no subcommands since 0.17.0",
                "help: use `jaic ok.jai -interp -- a`",
            ],
        ),
        (
            &["check", "ok.jai", "-I", "libs"],
            &["help: use `jaic ok.jai -I libs -check`"],
        ),
        (
            &[
                "build",
                "ok.jai",
                "-o",
                "out",
                "--emit-ir",
                "x.ll",
                "-plug",
                "Check",
            ],
            &["help: use `jaic ok.jai -exe out -emit_ir x.ll +Check`"],
        ),
        (
            &[
                "run",
                "ok.jai",
                "--color",
                "never",
                "--timings",
                "-",
                "meta",
            ],
            &["help: use `jaic ok.jai -no_color -timings -interp - meta`"],
        ),
        (&["build"], &["help: use `jaic file.jai`"]),
    ];
    for (args, expected) in cases {
        let output = jaic(&dir, args, &[]);
        let text = stderr(&output);
        assert_eq!(output.status.code(), Some(2), "{args:?}: {text}");
        assert_in_order(&text, expected);
        // The full help text is not dumped on a mistake.
        assert!(!text.contains("exit status:"), "{args:?}: {text}");
    }
    // Old spellings after a `+Plugin` are the plugin's own arguments, not mistakes.
    let output = jaic(
        &dir,
        &["ok.jai", "-check", "+NoSuchPlugin", "--timings"],
        &[],
    );
    assert!(
        !stderr(&output).contains("spelled differently"),
        "{}",
        stderr(&output)
    );
    // A file that is really called `build` is compiled, not taken for the old subcommand.
    std::fs::write(dir.join("build"), "main :: () {}\n").unwrap();
    let output = jaic(&dir, &["build", "-check"], &[]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    for args in [
        &["--help"][..],
        &["-h"],
        &["-help"],
        &["ok.jai", "--help"],
        &["ok.jai", "-O2", "-h"],
    ] {
        let output = jaic(&dir, args, &[]);
        assert_eq!(output.status.code(), Some(0), "{args:?}");
        let text = String::from_utf8_lossy(&output.stdout);
        assert_in_order(
            &text,
            &[
                "usage: jaic [options] file.jai",
                "-check",
                "-interp",
                "+Name",
                "exit status:",
            ],
        );
    }
    for flag in ["--version", "-version", "-V"] {
        let output = jaic(&dir, &[flag], &[]);
        assert_eq!(output.status.code(), Some(0), "{flag}");
        assert!(String::from_utf8_lossy(&output.stdout).starts_with("jaic "));
    }
    // `-no_color` reaches command-line errors too, `FORCE_COLOR` turns colour on.
    let colored = |args: &[&str], force: bool| {
        let mut command = Command::new(JAIC);
        command
            .args(args)
            .env("JAIC_DIAGNOSTICS", "plain")
            .env_remove("NO_COLOR")
            .env_remove("FORCE_COLOR")
            .env_remove("CLICOLOR_FORCE");
        if force {
            command.env("FORCE_COLOR", "1");
        }
        stderr(&command.output().unwrap())
    };
    assert!(colored(&["-bogus"], true).starts_with("\x1b[1;31merror\x1b[0m"));
    assert!(colored(&["-bogus", "-no_color"], true).starts_with("error: "));
    // After a lone `-` or a `+Plugin`, `-no_color` is not jaic's.
    assert!(colored(&["-bogus", "-", "-no_color"], true).starts_with("\x1b[1;31merror"));
}

#[test]
fn rich_layouts_on_request() {
    let dir = scratch("rich");
    let source =
        "#import \"Basic\";\nmain :: () {\n    counter := 1;\n    print(\"%\\n\", countr);\n}\n";
    std::fs::write(dir.join("u.jai"), source).unwrap();
    let rich = |layout: &str, color: bool| {
        let mut command = Command::new(JAIC);
        command
            .args(["-check", "u.jai"])
            .current_dir(&dir)
            .env("JAIC_DIAGNOSTICS", layout)
            .env_remove("NO_COLOR");
        if color {
            command.env("FORCE_COLOR", "1");
        } else {
            command.arg("-no_color");
        }
        stderr(&command.output().unwrap())
    };
    let unicode = rich("unicode", false);
    assert_in_order(
        &unicode,
        &[
            "error: unknown identifier",
            "╭─[",
            "u.jai:4:18]",
            " 3 │     counter := 1;",
            " 4 │     print(\"%\\n\", countr);",
            "   ·                  ━━━━━━ not found in this scope",
            " 5 │ }",
            "╰─",
            "help: a similar name exists: `counter`",
            " 4 ~     print(\"%\\n\", counter);",
        ],
    );
    let ascii = rich("ascii", false);
    assert_in_order(
        &ascii,
        &[
            "  --> ",
            " 4 |     print(",
            "   |                  ^^^^^^ not found in this scope",
        ],
    );
    assert!(ascii.is_ascii(), "{ascii}");
    let colored = rich("unicode", true);
    assert!(colored.contains("\x1b[1;31merror\x1b[0m"), "{colored}");
    // NO_COLOR is honoured when the choice is left to jaic.
    let output = Command::new(JAIC)
        .args(["-check", "u.jai"])
        .current_dir(&dir)
        .env("JAIC_DIAGNOSTICS", "unicode")
        .env("FORCE_COLOR", "1")
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    assert!(!stderr(&output).contains('\x1b'));
    // `-msvc_format`: one located line per item, no snippet and no colour, whatever the layout.
    let output = Command::new(JAIC)
        .args(["-check", "u.jai", "-msvc_format"])
        .current_dir(&dir)
        .env("JAIC_DIAGNOSTICS", "unicode")
        .env("FORCE_COLOR", "1")
        .output()
        .unwrap();
    let msvc = stderr(&output);
    assert_in_order(
        &msvc,
        &[
            "u.jai(4,18): error: unknown identifier `countr`\n",
            "u.jai(4,18): note: help: a similar name exists: `counter`\n",
        ],
    );
    assert!(!msvc.contains('\x1b') && !msvc.contains('│'), "{msvc}");
}
