//! `jaic run` answers the program's executable-path query with the executable `jaic build`
//! would write next to the main file, not `jaic`'s own path.
use std::path::Path;
use std::process::Command;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

#[test]
fn run_reports_the_built_executable_path() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("run-executable-path");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("src/game.jai"),
        "#import \"Basic\";\n#import \"System\";\n\
         main :: () { print(\"%\\n\", get_path_of_running_executable()); }\n",
    )
    .unwrap();
    let output = Command::new(JAIC)
        .args(["run", "src/game.jai"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let printed = String::from_utf8_lossy(&output.stdout)
        .trim()
        .replace('\\', "/");
    let name = if cfg!(windows) {
        "game.exe"
    } else {
        "game"
    };
    let expected = std::fs::canonicalize(dir.join("src")).unwrap().join(name);
    let expected = expected.display().to_string().replace('\\', "/");
    assert_eq!(
        printed.trim_start_matches("//?/"),
        expected.trim_start_matches("//?/")
    );
}
