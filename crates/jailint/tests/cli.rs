//! Mistakes on jailint's command line and in `jailint.toml`: each says what is wrong and how to
//! fix it, and exits with status 2.
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
