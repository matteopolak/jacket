//! The build status (`jaic::progress`): `Compiling`/`Finished` lines and, on a terminal, a status
//! line redrawn in place. `JAIC_PROGRESS` stands in for the terminal test, since a test's stderr
//! is a pipe.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

fn scratch(name: &str, source: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("build-status-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("hello.jai"), source).unwrap();
    dir
}

const HELLO: &str = "#import \"Basic\";\nmain :: () { print(\"hi\\n\"); }\n";

fn jaic(dir: &Path, progress: Option<&str>, args: &[&str]) -> Output {
    let mut command = Command::new(JAIC);
    command.args(args).current_dir(dir);
    command
        .env_remove("JAIC_PROGRESS")
        .env_remove("JACKET_PROJECT");
    if let Some(mode) = progress {
        command.env("JAIC_PROGRESS", mode);
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// stderr with the times replaced by `<t>`.
fn stderr(output: &Output) -> String {
    let text = String::from_utf8_lossy(&output.stderr);
    text.lines()
        .map(|line| match line.rfind(" in ") {
            Some(at) if line.ends_with('s') && line.contains("Finished") => {
                format!("{} in <t>", &line[..at])
            }
            _ => line.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn a_pipe_gets_no_status() {
    let dir = scratch("pipe", HELLO);
    for args in [
        &["hello.jai", "-check"][..],
        &["hello.jai"],
        &["hello.jai", "-interp"],
    ] {
        assert_eq!(stderr(&jaic(&dir, None, args)), "", "{args:?}");
    }
}

#[test]
fn plain_lines_have_no_control_codes() {
    let dir = scratch("plain", HELLO);
    let cases: [(&[&str], &str); 4] = [
        (
            &["hello.jai", "-check"],
            "    Checking hello (hello.jai)\n    Finished check in <t>",
        ),
        (
            &["hello.jai"],
            "   Compiling hello (hello.jai)\n    Finished debug [unoptimized] in <t>",
        ),
        (
            &["hello.jai", "-o"],
            "   Compiling hello (hello.jai)\n    Finished optimized in <t>",
        ),
        (
            &["hello.jai", "-interp"],
            "   Compiling hello (hello.jai)\n    Finished interp in <t>\n     Running hello.jai",
        ),
    ];
    for (args, expected) in cases {
        let output = jaic(&dir, Some("plain"), args);
        assert_eq!(stderr(&output), expected, "{args:?}");
        assert!(!output.stderr.contains(&0x1b), "{args:?}");
    }
}

#[test]
fn quiet_turns_the_status_off() {
    let dir = scratch("quiet", HELLO);
    for flag in ["-quiet", "--quiet"] {
        let output = jaic(&dir, Some("live"), &["hello.jai", "-check", flag]);
        assert_eq!(stderr(&output), "", "{flag}");
    }
}

#[test]
fn a_failed_compile_has_no_finished_line() {
    let dir = scratch("failed", "main :: () { x: int = \"no\"; }\n");
    let output = Command::new(JAIC)
        .args(["hello.jai", "-check"])
        .current_dir(&dir)
        .env("JAIC_PROGRESS", "plain")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let text = String::from_utf8_lossy(&output.stderr);
    assert!(
        text.starts_with("    Checking hello (hello.jai)\n"),
        "{text}"
    );
    assert!(!text.contains("Finished"), "{text}");
    assert!(text.contains("error"), "{text}");
}

/// A slow `#run` shows on the status line, and what it prints is never on the same line: as
/// a terminal shows it, the text after the last carriage return of its line is the text alone.
#[test]
fn live_status_steps_aside_for_output() {
    let dir = scratch(
        "live",
        "#import \"Basic\";\n\
         main :: () {}\n\
         #run { sleep_milliseconds(700); print(\"from run\\n\", to_standard_error = true); sleep_milliseconds(700); }\n",
    );
    let output = jaic(&dir, Some("live"), &["hello.jai", "-check"]);
    let text = String::from_utf8_lossy(&output.stderr);
    assert!(
        text.starts_with("\x1b[1;32m    Checking\x1b[0m hello (hello.jai)\n"),
        "{text:?}"
    );
    assert!(
        text.contains("\r\x1b[1;36m     Running\x1b[0m #run"),
        "{text:?}"
    );
    let printed = text
        .split('\n')
        .find(|line| line.contains("from run"))
        .expect("the #run's output");
    let shown = printed.rsplit('\r').next().unwrap();
    assert_eq!(shown.trim_start_matches("\x1b[K"), "from run", "{text:?}");
    let last = text.trim_end_matches('\n').rsplit('\n').next().unwrap();
    let last = last
        .rsplit('\r')
        .next()
        .unwrap()
        .trim_start_matches("\x1b[K");
    assert!(
        last.starts_with("\x1b[1;32m    Finished\x1b[0m check in "),
        "{text:?}"
    );
}

/// Output from a foreign call that is not a known writer (here a shell the `#run` starts) steps
/// aside for the status line too.
#[cfg(unix)]
#[test]
fn live_status_steps_aside_for_a_child_process() {
    let dir = scratch(
        "child",
        "#import \"Basic\";\n\
         libc :: #system_library \"libc\";\n\
         system :: (command: *u8) -> s32 #foreign libc;\n\
         main :: () {}\n\
         #run { sleep_milliseconds(700); system(\"echo from child 1>&2\"); sleep_milliseconds(700); }\n",
    );
    let output = jaic(&dir, Some("live"), &["hello.jai", "-check"]);
    let text = String::from_utf8_lossy(&output.stderr);
    let printed = text
        .split('\n')
        .find(|line| line.contains("from child"))
        .expect("the child's output");
    let shown = printed.rsplit('\r').next().unwrap();
    assert_eq!(shown.trim_start_matches("\x1b[K"), "from child", "{text:?}");
}
