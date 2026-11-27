//! Input file, build and command line mistakes and rich layouts.
use super::{JAIC, assert_in_order, jaic, jaic_on, stderr};
use std::process::Command;
use test_support::TempDir;

#[test]
fn input_file_problems() {
    let dir = TempDir::new("input");
    std::fs::write(dir.join("game.jai"), "main :: () {}\n").unwrap();
    std::fs::create_dir_all(dir.join("proj")).unwrap();
    std::fs::write(dir.join("proj/first.jai"), "main :: () {}\n").unwrap();
    let cases: &[(&[&str], &[&str])] = &[
        (
            &["-interp", "gmae.jai"],
            &[
                "error[cli-usage]: file `gmae.jai` does not exist",
                "help: did you mean `game.jai`?",
            ],
        ),
        (
            &["-interp", "game"],
            &[
                "error[cli-usage]: file `game` does not exist",
                "help: did you mean `game.jai`?",
            ],
        ),
        (
            &["-interp", "nothing_like_it.jai"],
            &[
                "error[cli-usage]: file `nothing_like_it.jai` does not exist",
                "help: relative paths start from the current directory",
            ],
        ),
        (
            &["-check", "proj"],
            &[
                "error[cli-usage]: `proj` is a directory, not a .jai file",
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
    let dir = TempDir::new("build");
    std::fs::write(dir.join("ok.jai"), "main :: () {}\n").unwrap();
    std::fs::write(dir.join("lib.jai"), "f :: () {}\n").unwrap();
    // The runtime's own `main` is no program: a file of only compile-time code still has none.
    std::fs::write(
        dir.join("only_run.jai"),
        "#import \"Basic\";\n#run print(\"computed at compile time\\n\");\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("out")).unwrap();
    let cases: &[(&[&str], Option<i32>, &[&str])] = &[
        (
            &["lib.jai"],
            Some(3),
            &[
                "error[cli-failure]: `lib.jai` has no `main` procedure, so there is no program to write",
                "help: add `main :: () { ... }`",
            ],
        ),
        (
            &["only_run.jai"],
            Some(3),
            &[
                "error[cli-failure]: `only_run.jai` has no `main` procedure, so there is no program to write",
            ],
        ),
        (
            &["ok.jai", "-exe", "out"],
            Some(1),
            &[
                "error[cli-failure]: the output path `out` is a directory",
                "help: name the file to write inside it, as in `-exe out/program`",
            ],
        ),
        (
            &["ok.jai", "-sanitize", "adress"],
            Some(2),
            &[
                "error[cli-usage]: unknown sanitizer `adress` for `-sanitize`",
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
    // A build file that made its own workspace but left its own output on.
    std::fs::write(
        dir.join("first.jai"),
        "#run {\n    w := compiler_create_workspace(\"app\");\n    add_build_file(\"ok.jai\", w);\n}\n#import \"Compiler\";\n",
    )
    .unwrap();
    let output = jaic(&dir, &["first.jai"], &[]);
    let text = stderr(&output);
    assert_eq!(output.status.code(), Some(3), "{text}");
    assert_in_order(
        &text,
        &[
            "error[cli-failure]: `first.jai` has no `main` procedure, so there is no program to write",
            "help: `first.jai` built another workspace, so it is a build file: add `set_build_options_dc(.{do_output = false});` to its `#run` so it writes no program of its own",
        ],
    );
    // A created workspace with no `main` is refused before anything links it.
    std::fs::write(
        dir.join("build.jai"),
        "#run {\n    set_build_options_dc(.{do_output = false});\n    w := compiler_create_workspace(\"lib\");\n    add_build_string(\"f :: () {}\", w);\n}\n#import \"Compiler\";\n",
    )
    .unwrap();
    for mode in [None, Some("-check")] {
        let mut args = vec!["build.jai"];
        args.extend(mode);
        let output = jaic(&dir, &args, &[]);
        let text = stderr(&output);
        assert_eq!(output.status.code(), Some(1), "{mode:?}: {text}");
        assert_in_order(
            &text,
            &[
                "error: workspace `lib` has no `main` procedure, so there is no executable to write",
                "help: add `main :: () { ... }` to its sources, or set its `output_type` to a library",
            ],
        );
        assert!(!text.contains("link"), "{mode:?}: {text}");
    }
    // A linker that is not installed: what named it, and that `-interp` needs none.
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
            "error[cli-failure]: could not run the linker `no-such-linker-for-jaic-tests`",
            "note: JAIC_LINKER names it",
            "note: `jaic file.jai -interp` needs no linker",
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
            "f.jai:1:18: error[unknown-library]: unknown library `nothere`",
            "help: declare it with `nothere :: #library",
        ],
    );
}

#[test]
fn command_line_mistakes_explain_themselves() {
    let dir = TempDir::new("cli");
    std::fs::write(dir.join("ok.jai"), "main :: () {}\n").unwrap();
    std::fs::write(dir.join("two.jai"), "main :: () {}\n").unwrap();
    let cases: &[(&[&str], &[&str])] = &[
        (
            &[],
            &[
                "error[cli-usage]: no file to compile",
                "help: usage: jaic [options] file.jai [options]",
                "help: `jaic -help` lists the options",
            ],
        ),
        (&["-check"], &["error[cli-usage]: no file to compile"]),
        (
            &["ok.jai", "-os"],
            &["error[cli-usage]: `-os` needs an OS name: linux, windows, macos or wasm"],
        ),
        (
            &["ok.jai", "-os", "linus"],
            &[
                "error[cli-usage]: unknown OS `linus` for `-os`",
                "help: did you mean `-os linux`?",
            ],
        ),
        (
            &["ok.jai", "-cpu", "z80"],
            &["error[cli-usage]: unknown CPU `z80` for `-cpu`"],
        ),
        (
            &["ok.jai", "-I"],
            &["error[cli-usage]: `-I` needs a directory"],
        ),
        (
            &["ok.jai", "-timing"],
            &[
                "error[cli-usage]: unknown option `-timing`",
                "help: did you mean `-timings`?",
            ],
        ),
        (
            &["ok.jai", "-O7"],
            &[
                "error[cli-usage]: unknown optimization level `-O7`",
                "help: use -O0, -O1, -O2 or -O3",
            ],
        ),
        (
            &["ok.jai", "-o", "-O2"],
            &[
                "error[cli-usage]: `-O0`..`-O3` and `-o`, `-od` or `-very_debug` both set the optimization",
            ],
        ),
        (
            &["ok.jai", "-interp", "extra"],
            &[
                "error[cli-usage]: unexpected argument `extra`",
                "help: arguments for the program go after `--`: `jaic ok.jai -interp -- extra`",
            ],
        ),
        (
            &["ok.jai", "two.jai"],
            &[
                "error[cli-usage]: jaic compiles one file, but got `ok.jai` and `two.jai`",
                "help: `#load` or `#import` the others from the first",
            ],
        ),
        (
            &["ok.jai", "-check", "-interp"],
            &["error[cli-usage]: `-check` and `-interp` cannot be combined"],
        ),
        (
            &["ok.jai", "--", "a"],
            &[
                "error[cli-usage]: `--` hands arguments to the program, which only `-interp` runs",
                "help: use `jaic ok.jai -interp -- ...`",
            ],
        ),
        (
            &["ok.jai", "-check", "-", "--", "a"],
            &["error[cli-usage]: `--` hands arguments to the program, which only `-interp` runs"],
        ),
        (
            &["ok.jai", "-no_workspace_output"],
            &["error[cli-usage]: `-no_workspace_output` only applies to `-interp`"],
        ),
        (
            &["ok.jai", "-quiet", "-verbose"],
            &["error[cli-usage]: `-quiet` and `-verbose` contradict each other"],
        ),
    ];
    for (args, expected) in cases {
        let output = jaic(&dir, args, &[]);
        let text = stderr(&output);
        assert_eq!(output.status.code(), Some(2), "{args:?}: {text}");
        assert_in_order(&text, expected);
        // The full help text is not dumped on a mistake.
        assert!(!text.contains("Exit status:"), "{args:?}: {text}");
    }
}

#[test]
fn plugin_arguments_help_version_and_colour() {
    let dir = TempDir::new("cli-help");
    std::fs::write(dir.join("ok.jai"), "main :: () {}\n").unwrap();
    // Options after a `+Plugin` are the plugin's own arguments, not mistakes.
    let output = jaic(
        &dir,
        &["ok.jai", "-check", "+NoSuchPlugin", "--timings"],
        &[],
    );
    assert!(
        !stderr(&output).contains("unknown option"),
        "{}",
        stderr(&output)
    );
    // A file called `build` is compiled like any other.
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
                "Usage: jaic [options] file.jai",
                "-check",
                "-interp",
                "+Name",
                "Exit status:",
            ],
        );
    }
    // Options that shape a build's output are accepted and ignored by the modes that write none.
    for args in [
        &["ok.jai", "-check", "-exe", "x", "-O2"][..],
        &["ok.jai", "-interp", "-o", "-emit_ir", "x.ll"][..],
    ] {
        assert_eq!(jaic(&dir, args, &[]).status.code(), Some(0), "{args:?}");
    }
    let printed = jaic(&dir, &["-print_stdlib"], &[]);
    assert_eq!(printed.status.code(), Some(0));
    assert!(
        String::from_utf8_lossy(&printed.stdout)
            .trim()
            .ends_with("stdlib")
    );
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
    assert!(colored(&["-bogus"], true).starts_with("\x1b[1;31merror["));
    assert!(colored(&["-bogus", "-no_color"], true).starts_with("error[cli-usage]: "));
    // After a lone `-` or a `+Plugin`, `-no_color` is not jaic's.
    assert!(colored(&["-bogus", "-", "-no_color"], true).starts_with("\x1b[1;31merror"));
}

#[test]
fn help_is_sectioned_and_completions_cover_the_options() {
    let dir = TempDir::new("cli-completions");
    let help = String::from_utf8_lossy(&jaic(&dir, &["-help"], &[]).stdout).into_owned();
    assert_in_order(
        &help,
        &[
            "Usage: jaic [options] file.jai",
            "Modes:",
            "-check",
            "Program:",
            "-import_dir <dir>",
            "Output (building only):",
            "-exe <name>",
            "Plugins:",
            "+Name",
            "Diagnostics and output:",
            "-timings",
            "Environment:",
            "JAIC_MEMORY_LIMIT",
            "Exit status:",
        ],
    );
    // The flags for studying the compiler are accepted, not listed.
    assert!(!help.contains("-dev_"), "{help}");
    assert!(!help.contains('\x1b'), "{help}");
    for (shell, expected) in [
        ("bash", "complete -o filenames -F _jaic jaic"),
        ("zsh", "#compdef jaic"),
        ("fish", "complete -c jaic -o check -d "),
    ] {
        let output = jaic(&dir, &["-completions", shell], &[]);
        assert_eq!(output.status.code(), Some(0), "{shell}");
        let script = String::from_utf8_lossy(&output.stdout);
        assert!(script.contains(expected), "{shell}: {script}");
        assert!(
            script.contains("-interp") && !script.contains("-dev_"),
            "{shell}"
        );
    }
    let output = jaic(&dir, &["-completions", "csh"], &[]);
    assert_eq!(output.status.code(), Some(2));
    assert_in_order(
        &stderr(&output),
        &[
            "error[cli-usage]: unknown shell `csh` for `-completions`",
            "help: use bash, zsh or fish",
        ],
    );
    // Colour follows `FORCE_COLOR` and `-no_color`, like the rest of the output.
    let colored = |args: &[&str]| {
        let output = Command::new(JAIC)
            .args(args)
            .env_remove("NO_COLOR")
            .env_remove("CLICOLOR_FORCE")
            .env("FORCE_COLOR", "1")
            .output()
            .unwrap();
        String::from_utf8_lossy(&output.stdout).into_owned()
    };
    assert!(colored(&["-help"]).contains("\x1b[1;4mModes:\x1b[0m"));
    assert!(!colored(&["-help", "-no_color"]).contains('\x1b'));
    // `NO_COLOR` wins over `FORCE_COLOR`.
    let no_color = Command::new(JAIC)
        .arg("-help")
        .env("NO_COLOR", "1")
        .env("FORCE_COLOR", "1")
        .output()
        .unwrap();
    assert!(!String::from_utf8_lossy(&no_color.stdout).contains('\x1b'));
    // A glued value is not split at an `=`.
    let output = jaic(&dir, &["-Dwarnings=x"], &[]);
    assert_in_order(
        &stderr(&output),
        &["error[cli-usage]: unknown option `-Dwarnings=x`"],
    );
}

#[test]
fn rich_layouts_on_request() {
    let dir = TempDir::new("rich");
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
            "error[unknown-identifier]: unknown identifier",
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
    assert!(colored.contains("\x1b[1;31merror["), "{colored}");
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
            "u.jai(4,18): error[unknown-identifier]: unknown identifier `countr`\n",
            "u.jai(4,18): note: help: a similar name exists: `counter`\n",
        ],
    );
    assert!(!msvc.contains('\x1b') && !msvc.contains('│'), "{msvc}");
}
