//! The interpreter's opt-in counters: `-profile`, `-profile_ops`, `-profile_runs` and `-coverage_file`.
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

/// `-profile_runs` lists the top-level compile-time runs by the blocks they ran, with their
/// share and place, and prints nothing unless asked.
#[test]
fn profile_runs_lists_compile_time_runs_by_cost() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-profile-runs");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("runs.jai");
    std::fs::write(
        &source,
        "count :: (n: int) -> int {\n\
         \x20   total := 0;\n\
         \x20   for 0..n { total += it; }\n\
         \x20   return total;\n\
         }\n\
         #run count(10);\n\
         #run count(30000);\n\
         #run count(10000);\n\
         main :: () {}\n",
    )
    .unwrap();
    let output = Command::new(JAIC)
        .args(["-check", "-profile_runs"])
        .arg(&source)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert!(stderr.contains("compile-time runs: "), "{stderr}");
    let rows: Vec<&str> = stderr
        .lines()
        .filter(|line| line.contains("runs.jai:"))
        .collect();
    assert_eq!(rows.len(), 3, "{stderr}");
    // Costliest first: the 30000 loop (line 7), then 10000 (line 8), then 10 (line 6).
    for (row, line) in rows.iter().zip([7, 8, 6]) {
        assert!(row.contains(&format!("runs.jai:{line}:")), "{row}");
    }
    assert!(rows[0].trim_start().starts_with('7'), "{}", rows[0]);
    assert!(
        rows[0].contains('%') && rows[0].contains(" ms"),
        "{}",
        rows[0]
    );
    let plain = Command::new(JAIC)
        .arg("-check")
        .arg(&source)
        .output()
        .unwrap();
    assert!(
        !String::from_utf8_lossy(&plain.stderr).contains("compile-time runs"),
        "the cost table is opt-in"
    );
}

/// The standard library's own runs are one summary line after the program's rows, not rows of
/// their own mixed in with them.
#[test]
fn profile_runs_summarizes_the_standard_librarys_runs() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-profile-runs-library");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("mine.jai");
    std::fs::write(
        &source,
        "#import \"Basic\";\n\
         #run print(\"%\\n\", 1);\n\
         main :: () { print(\"%\\n\", 2); }\n",
    )
    .unwrap();
    let output = Command::new(JAIC)
        .args(["-check", "-profile_runs"])
        .arg(&source)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    let table: Vec<&str> = stderr
        .lines()
        .skip_while(|line| !line.starts_with("compile-time runs: "))
        .collect();
    assert!(
        table.iter().any(|line| line.contains("mine.jai:2:")),
        "{stderr}"
    );
    assert!(
        table.iter().all(|line| !line.contains("stdlib/")),
        "no row names a library file: {stderr}"
    );
    let last = table.last().unwrap();
    assert!(
        last.contains("standard library (") && last.contains(" runs)"),
        "{stderr}"
    );
}
