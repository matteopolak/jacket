//! Declared golden deadlines reach the actual runner, including interpreter execution.
#![cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use test_support::TempDir;

const COMPILER: &str = r#"#!/usr/bin/env python3
import pathlib, sys, time
args = sys.argv[1:]
source = pathlib.Path(args[0])
slow = source.stem == 'slow'
if '-exe' in args:
    binary = pathlib.Path(args[args.index('-exe') + 1])
    binary.write_text('#!/usr/bin/env python3\nimport time\n' + ('time.sleep(2)\n' if slow else "print('quick')\n"))
    binary.chmod(0o755)
elif '-interp' in args:
    if slow: time.sleep(2)
    else: print('quick')
"#;

#[test]
fn declared_golden_timeout_is_enforced_in_native_and_interpreter_modes() {
    for interpreted in [false, true] {
        let project = TempDir::new("plain-timeout-contract");
        std::fs::create_dir_all(project.join("tests")).unwrap();
        std::fs::create_dir_all(project.join("stdlib")).unwrap();
        std::fs::write(project.join("jacket.toml"), "[project]\n").unwrap();
        for (name, expected) in [("slow", ""), ("quick", "quick\n")] {
            std::fs::write(
                project.join(format!("tests/{name}.jai")),
                "// test: timeout(1)\nmain :: () {}\n",
            )
            .unwrap();
            std::fs::write(project.join(format!("tests/{name}.stdout")), expected).unwrap();
        }
        let compiler = project.join("jaic");
        std::fs::write(&compiler, COMPILER).unwrap();
        std::fs::set_permissions(&compiler, std::fs::Permissions::from_mode(0o755)).unwrap();
        let jacket = project.join("jacket");
        std::fs::copy(env!("CARGO_BIN_EXE_jacket"), &jacket).unwrap();
        let mut command = Command::new(jacket);
        command
            .args(["test", "--no-doc", "--format", "json"])
            .current_dir(&project)
            .env("JAIC_STDLIB", project.join("stdlib"));
        if interpreted {
            command.arg("--interp");
        }
        let output = command.output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!output.status.success(), "{stdout}");
        let records: Vec<serde_json::Value> = stdout
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        for (name, event) in [("slow", "timed_out"), ("quick", "passed")] {
            assert!(
                records.iter().any(|record| {
                    record["kind"] == "golden" && record["event"] == event && record["name"] == name
                }),
                "interpreted={interpreted}: {stdout}\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let summary = records.last().unwrap();
        assert_eq!(summary["event"], "finished");
        assert_eq!(summary["passed"], 1);
        assert_eq!(summary["timed_out"], 1);
    }
}
