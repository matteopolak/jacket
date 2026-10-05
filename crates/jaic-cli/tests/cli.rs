//! Command-line behavior of the `jaic` binary that is not about native code generation.
use std::path::Path;
use std::process::Command;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

/// `jaic run file.jai -- args` hands the arguments to the program; `-` arguments still go to
/// the metaprogram, up to a `--`.
#[test]
fn run_passes_program_arguments() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-run-args");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("args.jai");
    std::fs::write(
        &source,
        "#import \"Basic\";\n\
         main :: () {\n\
         \x20   args := get_command_line_arguments();\n\
         \x20   for args if it_index > 0 print(\"[%]\", it);\n\
         \x20   print(\"%\\n\", args.count);\n\
         }\n",
    )
    .unwrap();
    let run = |extra: &[&str]| {
        let output = Command::new(JAIC)
            .arg("run")
            .arg(&source)
            .args(extra)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).into_owned()
    };
    assert_eq!(run(&[]), "0\n");
    assert_eq!(run(&["--", "a", "b c"]), "[a][b c]3\n");
    assert_eq!(run(&["-", "meta", "--", "x"]), "[x]2\n");
}
