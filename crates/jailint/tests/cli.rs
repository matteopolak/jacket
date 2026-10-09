//! jailint from the command line: which files it compiles as what, and mistakes on its command
//! line and in `jailint.toml`, each of which says what is wrong and how to fix it, and exits
//! with status 2.
use std::path::PathBuf;
use std::process::Command;

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("jailint-cli")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("game.jai"), "main :: () {}\n").unwrap();
    dir
}

/// jailint's stderr and exit status for `args`, run in `dir`.
fn jailint(dir: &PathBuf, args: &[&str]) -> (String, Option<i32>) {
    let output = Command::new(env!("CARGO_BIN_EXE_jailint"))
        .args(args)
        .current_dir(dir)
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("CLICOLOR_FORCE")
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code(),
    )
}

#[test]
fn command_line_mistakes_explain_themselves() {
    let dir = scratch("args");
    let cases: &[(&[&str], &str)] = &[
        (
            &["--fx"],
            "error: unknown option `--fx`\nhelp: did you mean `--fix`?",
        ),
        (
            &["-D", "float_equalty"],
            "error: unknown rule `float_equalty`\nhelp: did you mean `float_equality`?",
        ),
        (
            &["--color", "sometimes"],
            "error: unknown `--color` value `sometimes`\nhelp: use auto, always or never",
        ),
        (
            &["-j", "x"],
            "error: `-j` takes a number of jobs, such as `-j 4`",
        ),
        (&["--config"], "error: `--config` needs a value"),
        (
            &["gmae.jai"],
            "error: `gmae.jai` does not exist\nhelp: did you mean `game.jai`?",
        ),
        (
            &["nothing_like_it.jai"],
            "error: `nothing_like_it.jai` does not exist\nhelp: pass .jai files",
        ),
    ];
    for (args, expected) in cases {
        let (text, code) = jailint(&dir, args);
        assert_eq!(code, Some(2), "{args:?}: {text}");
        assert!(text.contains(expected), "{args:?}: {text}");
    }
}

#[test]
fn config_mistakes_name_the_line_and_the_fix() {
    let dir = scratch("config");
    let cases = [
        (
            "exclude = [\n",
            "line 1: this array is never closed\nhelp: end it with `]`",
        ),
        (
            "[rules]\nfloat_equalty = \"deny\"\n",
            "line 2: unknown rule `float_equalty`\nhelp: did you mean `float_equality`?",
        ),
        (
            "[rules]\nfloat_equality = \"loud\"\n",
            "line 2: unknown level \"loud\" for `float_equality`\nhelp: use \"allow\", \"warn\" or \"deny\"",
        ),
        (
            "[lints]\n",
            "line 1: unknown table `[lints]`\nhelp: the only table is `[rules]`",
        ),
        ("verbose = true\n", "line 1: unknown setting `verbose`"),
    ];
    for (config, expected) in cases {
        std::fs::write(dir.join("jailint.toml"), config).unwrap();
        let (text, code) = jailint(&dir, &["game.jai"]);
        assert_eq!(code, Some(2), "{config:?}: {text}");
        assert!(text.starts_with("error: in `"), "{config:?}: {text}");
        assert!(text.contains(expected), "{config:?}: {text}");
    }
    let (text, code) = jailint(&dir, &["--config", "missing.toml", "game.jai"]);
    assert_eq!(code, Some(2));
    assert!(
        text.contains("error: could not read config `missing.toml`: no such file or directory"),
        "{text}"
    );
}

#[test]
fn a_file_a_module_loads_is_checked_as_part_of_the_module() {
    let dir = scratch("module-part");
    let module = dir.join("Thing");
    std::fs::create_dir_all(&module).unwrap();
    std::fs::write(module.join("module.jai"), "#load \"part.jai\";\n").unwrap();
    // `get` is exported: its parameter belongs to the module's interface even when unused.
    std::fs::write(
        module.join("part.jai"),
        "get :: (w: int = -1) -> int { return 4; }\n\
         twice :: () -> int { unused := 1; return get() * 2; }\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jailint"))
        .args(["-D", "warnings", "Thing/part.jai"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(!text.contains("unused parameter"), "{text}");
    assert!(text.contains("unused variable `unused`"), "{text}");
    assert_eq!(output.status.code(), Some(1), "{text}");
}

#[test]
fn a_file_a_nearby_program_loads_is_checked_as_part_of_that_program() {
    let dir = scratch("loader-root");
    let part = dir.join("tour");
    std::fs::create_dir_all(part.join("basics")).unwrap();
    // `basics.jai` has no entry point of its own: the program that loads it has to be the root.
    std::fs::write(
        part.join("main.jai"),
        "#load \"basics/basics.jai\";\nmain :: () { run_basics(); }\n",
    )
    .unwrap();
    std::fs::write(
        part.join("basics/basics.jai"),
        "run_basics :: () { unused := helper(1); }\nhelper :: (x: int) -> int { return x; }\n",
    )
    .unwrap();
    let run = |args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_jailint"))
            .args(args)
            .current_dir(&dir)
            .output()
            .unwrap();
        (
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
            output.status.code(),
        )
    };
    let (out, err, code) = run(&["-D", "warnings", "-v", "tour/basics/basics.jai"]);
    assert!(err.contains("main.jai"), "root should be main.jai: {err}");
    assert!(!err.contains("did not compile completely"), "{err}");
    assert!(out.contains("unused variable `unused`"), "{out}");
    assert_eq!(code, Some(1), "{out}{err}");
    // Only the listed file is reported, not the loader's other findings.
    std::fs::write(
        part.join("main.jai"),
        "#load \"basics/basics.jai\";\nmain :: () { run_basics(); stray := 2; }\n",
    )
    .unwrap();
    let (out, _, _) = run(&["-D", "warnings", "tour/basics/basics.jai"]);
    assert!(!out.contains("stray"), "{out}");
}
