//! What bounds compile-time code under jailint: a block budget for each evaluation, and a
//! wall-clock limit for each program checked (`--compile-time-budget` and `--check-time-limit`;
//! a project's `jacket.toml` has no say).
use std::path::PathBuf;
use std::process::Command;

const SOURCE: &str = r"spin :: () { while true {} }
#run spin();

Info :: ($T: Type) -> *Type_Info {
    TI :: type_info(T);
    return TI;
}

Point :: struct { x: int; }

main :: () {
    info := Info(Point);
    sum := #run 1 + 2;
}
";

fn project(name: &str, config: Option<&str>) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("jailint-limits")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("main.jai"), SOURCE).unwrap();
    if let Some(config) = config {
        std::fs::write(dir.join("jacket.toml"), config).unwrap();
    }
    dir
}

fn lint(dir: &PathBuf, flags: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_jailint"))
        .args(flags)
        .arg("main.jai")
        .current_dir(dir)
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn a_runaway_run_does_not_hide_the_code_after_it() {
    // The budget must run out first; on a slow machine 200M blocks outlast the default time limit.
    let text = lint(&project("budget", None), &["--check-time-limit", "0"]);
    assert!(
        text.contains("execution budget exhausted (200000000 basic blocks)"),
        "{text}"
    );
    // The bodies after the runaway were checked: their findings are there.
    assert!(text.contains("unused variable `info`"), "{text}");
    assert!(text.contains("unused variable `sum`"), "{text}");
}

#[test]
fn the_budget_comes_from_the_flag() {
    let text = lint(&project("blocks", None), &["--compile-time-budget", "5000"]);
    assert!(text.contains("(5000 basic blocks)"), "{text}");
    assert!(
        text.contains("raise the compile-time evaluation budget"),
        "{text}"
    );
}

#[test]
fn a_project_cannot_raise_the_limits() {
    // A `[check]` table is not a setting: it is reported like any unknown key, and the default
    // budget stays.
    let text = lint(
        &project("raised", Some("[check]\nmax_blocks = 0\nmax_seconds = 0\n")),
        &["--check-time-limit", "0.3"],
    );
    assert!(text.contains("checking stopped after 0.3 s"), "{text}");
}

#[test]
fn a_flag_with_a_bad_value_is_refused() {
    let output = Command::new(env!("CARGO_BIN_EXE_jailint"))
        .args(["--compile-time-budget", "many", "main.jai"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let text = String::from_utf8_lossy(&output.stderr);
    assert!(
        text.contains("`--compile-time-budget` takes a whole number"),
        "{text}"
    );
}

#[test]
fn the_wall_clock_limit_gives_one_notice() {
    let text = lint(
        &project("seconds", None),
        &["--compile-time-budget", "0", "--check-time-limit", "0.3"],
    );
    assert_eq!(
        text.matches("checking stopped after 0.3 s; results are partial")
            .count(),
        1,
        "{text}"
    );
    assert!(!text.contains("execution budget"), "{text}");
}

/// The time limit belongs to each program checked, not to the run: two runaway programs one
/// after the other each get the whole limit.
#[test]
fn the_wall_clock_limit_is_each_programs_own() {
    let dir = project("seconds-per-program", None);
    std::fs::write(dir.join("other.jai"), SOURCE).unwrap();
    let started = std::time::Instant::now();
    let output = Command::new(env!("CARGO_BIN_EXE_jailint"))
        .args([
            "--compile-time-budget",
            "0",
            "--check-time-limit",
            "0.3",
            "-j",
            "1",
            "main.jai",
            "other.jai",
        ])
        .current_dir(&dir)
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        text.matches("checking stopped after 0.3 s").count(),
        2,
        "{text}"
    );
    assert!(started.elapsed().as_secs_f64() >= 0.55, "{text}");
}
