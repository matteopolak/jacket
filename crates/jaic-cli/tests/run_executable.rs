//! `jaic -interp` answers the program's executable-path query with the executable `jaic`
//! would write next to the main file, not `jaic`'s own path.
use std::process::Command;
use test_support::TempDir;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

#[test]
fn run_reports_the_built_executable_path() {
    let dir = TempDir::new("run-executable-path");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("src/game.jai"),
        "#import \"Basic\";\n#import \"System\";\n\
         main :: () { print(\"%\\n\", get_path_of_running_executable()); }\n",
    )
    .unwrap();
    let output = Command::new(JAIC)
        .args(["-interp", "src/game.jai"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let printed = jacket_config::to_slash(String::from_utf8_lossy(&output.stdout).trim());
    let name = if cfg!(windows) {
        "game.exe"
    } else {
        "game"
    };
    let expected = std::fs::canonicalize(dir.join("src")).unwrap().join(name);
    let expected = jacket_config::slashed(&expected);
    assert_eq!(
        printed.trim_start_matches("//?/"),
        expected.trim_start_matches("//?/")
    );
}
