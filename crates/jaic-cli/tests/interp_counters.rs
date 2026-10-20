//! The interpreter's opt-in counters: `-profile`, `-profile_ops` and `-coverage_file`.
use std::path::Path;
use std::process::Command;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

/// A program with a hot loop and a few calls, interpreted with the counters on.
fn counting_program(dir: &Path, name: &str, end: &str) -> std::path::PathBuf {
    std::fs::create_dir_all(dir).unwrap();
    let source = dir.join(name);
    std::fs::write(
        &source,
        format!(
            "#import \"Basic\";\n\
             square :: (x: int) -> int {{ return x * x; }}\n\
             unused :: () -> int {{ return 99; }}\n\
             main :: () {{\n\
             \x20   total := 0;\n\
             \x20   for 0..49 total += square(it);\n\
             \x20   assert(total == 40425);\n\
             {end}\
             }}\n"
        ),
    )
    .unwrap();
    source
}

/// `-profile` prints the procedures that ran most, with the instruction kinds and op pairs;
/// `-profile_ops` counts the straightened code instead.
#[test]
fn profile_reports_counts_per_procedure() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-profile");
    let source = counting_program(&dir, "profile.jai", "");
    for flag in ["-profile", "-profile_ops"] {
        let output = Command::new(JAIC)
            .arg("-interp")
            .arg(flag)
            .arg(&source)
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{flag}: {stderr}");
        assert!(stderr.contains("interpreter profile: "), "{flag}: {stderr}");
        assert!(stderr.contains("procedure"), "{flag}: {stderr}");
        let square = stderr
            .lines()
            .find(|line| line.trim_end().ends_with("square"))
            .unwrap_or_else(|| panic!("{flag}: no row for `square`:\n{stderr}"));
        assert!(
            square.split_whitespace().any(|field| field == "50"),
            "{flag}: `square` ran 50 times: {square}"
        );
        assert!(stderr.contains("instructions by kind:"), "{flag}: {stderr}");
        assert!(
            stderr.contains("interpreter ops by kind:"),
            "{flag}: {stderr}"
        );
        assert!(
            stderr.contains("op pairs within blocks:"),
            "{flag}: {stderr}"
        );
    }
    let plain = Command::new(JAIC)
        .arg("-interp")
        .arg(&source)
        .output()
        .unwrap();
    assert!(
        !String::from_utf8_lossy(&plain.stderr).contains("interpreter profile"),
        "the counters are opt-in"
    );
}

/// `-coverage_file` appends one `path:line name` line for each procedure that ran, including
/// when the program ends the process with `exit`.
#[test]
fn coverage_file_lists_the_procedures_that_ran() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-coverage-file");
    for (name, end) in [("returns.jai", ""), ("exits.jai", "    exit(0);\n")] {
        let source = counting_program(&dir, name, end);
        let coverage = dir.join(format!("{name}.coverage"));
        let _ = std::fs::remove_file(&coverage);
        let output = Command::new(JAIC)
            .arg("-interp")
            .arg("-coverage_file")
            .arg(&coverage)
            .arg(&source)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let text = std::fs::read_to_string(&coverage).unwrap();
        assert!(
            text.lines()
                .any(|line| line.ends_with(":2 square") && line.contains(name)),
            "{name}: {text}"
        );
        assert!(
            text.lines().any(|line| line.ends_with(" main")),
            "{name}: {text}"
        );
        assert!(!text.contains(" unused"), "{name}: {text}");
    }
}
