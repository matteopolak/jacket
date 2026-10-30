//! `-run_budget` bounds each top-level compile-time run, so a `#run` that never ends fails the
//! build instead of hanging it.
use std::process::Command;
use test_support::TempDir;

fn check(name: &str, source: &str, flags: &[&str]) -> (Option<i32>, String) {
    let dir = TempDir::new("run-budget");
    let file = dir.join(format!("{name}.jai"));
    std::fs::write(&file, source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg("-check")
        .args(flags)
        .arg(&file)
        .output()
        .unwrap();
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

const SPIN: &str = "spin :: () { while true {} }\n#run spin();\nmain :: () {}\n";

#[test]
fn a_runaway_run_fails_once_it_uses_its_budget() {
    let (code, err) = check("spin", SPIN, &["-run_budget", "5000"]);
    assert_eq!(code, Some(1), "{err}");
    assert!(
        err.contains("execution budget exhausted (5000 steps)"),
        "{err}"
    );
    assert!(err.contains("still running `spin`"), "{err}");
    assert!(err.contains("-run_budget <steps>"), "{err}");
}

const COUNT: &str = "count :: (n: int) -> int {\n    total := 0;\n    for 0..n { total += it; }\n    return total;\n}\nx :: #run count(100000);\nmain :: () {}\n";

#[test]
fn the_budget_is_per_run_and_zero_means_no_limit() {
    let (code, err) = check("count_small", COUNT, &["-run_budget", "10000"]);
    assert_eq!(code, Some(1), "{err}");
    assert!(err.contains("execution budget exhausted"), "{err}");
    // The default allows far more than this run needs, and so does an unlimited budget.
    let (code, err) = check("count_default", COUNT, &[]);
    assert_eq!(code, Some(0), "{err}");
    let (code, err) = check("count_unlimited", COUNT, &["-run_budget", "0"]);
    assert_eq!(code, Some(0), "{err}");
}

#[test]
fn the_flag_wants_a_number() {
    let (code, err) = check("bad", SPIN, &["-run_budget", "lots"]);
    assert_eq!(code, Some(2), "{err}");
    assert!(
        err.contains("`-run_budget` takes a whole number, not `lots`"),
        "{err}"
    );
}
