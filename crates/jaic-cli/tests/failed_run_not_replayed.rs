//! Compile-time code whose run failed because a procedure it called does not compile is not run
//! a second time: the failed run may have left a lock held in compile-time memory, and a replay
//! would wait for it forever instead of reporting the error.
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Run `jaic -check file` and give up on it after `limit`.
fn check(file: &Path, limit: Duration) -> Option<(bool, String)> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg("-check")
        .arg(file)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let started = Instant::now();
    loop {
        if child.try_wait().unwrap().is_some() {
            let output = child.wait_with_output().unwrap();
            return Some((
                output.status.success(),
                String::from_utf8_lossy(&output.stderr).into_owned(),
            ));
        }
        if started.elapsed() > limit {
            child.kill().unwrap();
            let _ = child.wait();
            return None;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn a_failed_insert_run_reports_its_error_instead_of_running_again() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("failed-run-not-replayed");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("main.jai");
    // `gen` takes a lock in compile-time memory, then calls `helper`, which does not compile.
    // `other` gives the compiler another body to finish, which is what made it try `main` again.
    std::fs::write(
        &file,
        r#"state: s64;

helper :: () -> s64 {
    return missing_name;
}

other :: () -> s64 { return 3; }

gen :: () -> string {
    while state == 1 {}
    state = 1;
    x := helper();
    state = 2;
    return "y := 1;";
}

main :: () {
    z := other();
    #insert #run gen();
}
"#,
    )
    .unwrap();
    let (success, stderr) = check(&file, Duration::from_secs(60))
        .expect("the check ran for a minute instead of reporting the error");
    assert!(!success, "{stderr}");
    assert!(
        stderr.contains("unknown identifier `missing_name`"),
        "{stderr}"
    );
}
