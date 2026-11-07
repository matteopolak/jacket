//! A controlled compiler fixture proves selection, build arguments and execution routing
//! together, without compiling or executing a foreign native binary.
#![cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use test_support::TempDir;

const COMPILER: &str = r"#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
with open(os.environ['TARGET_LOG'], 'a') as out:
    out.write(json.dumps(args) + '\n')
if '-check' in args and '-interp' in args:
    sys.stderr.write('conflicting compiler modes\n')
    raise SystemExit(1)
if '-doc_comments' in args:
    print(os.environ.get('TARGET_DOC_RECORD', ''))
    raise SystemExit(0)
if '-check' in args and os.environ.get('TARGET_CHECK_SUCCESS') == '1':
    raise SystemExit(0)
if '-check' in args:
    sys.stderr.write('known compile error\n')
    raise SystemExit(1)
def record(*fields):
    print('\x02' + '\x1f'.join(fields) + '\x03', end='', flush=True, file=sys.stderr if '--run' in args else sys.stdout)
if '--run' in args:
    for name in args[args.index('--run')+1:args.index('--temp')]:
        record('start', name)
        record('end', name, 'pass')
    raise SystemExit(0)
for name, note in [('wasm_only', 'only(.WASM)'), ('native_only', 'skip_on(.WASM)'), ('native_cpu', 'only(.ARM64,.X64)')]:
    record('manifest', 'test', name, os.environ['TARGET_TEST'], '1', note)
record('discovered')
if '-exe' in args:
    path = pathlib.Path(args[args.index('-exe')+1])
    path.write_text(pathlib.Path(__file__).read_text())
    path.chmod(0o755)
";

fn fixture() -> (TempDir, std::path::PathBuf) {
    let root = TempDir::new("forwarded-target");
    std::fs::create_dir_all(root.join("tests")).unwrap();
    let compiler = root.join("jaic");
    std::fs::write(&compiler, COMPILER).unwrap();
    std::fs::set_permissions(&compiler, std::fs::Permissions::from_mode(0o755)).unwrap();
    let jacket = root.join("jacket");
    std::fs::copy(env!("CARGO_BIN_EXE_jacket"), &jacket).unwrap();
    (root, jacket)
}

#[test]
fn forwarded_targets_drive_selection_compilation_and_runner() {
    let host = format!("-os={}", std::env::consts::OS);
    for (flags, wasm, interp) in [
        (vec!["-os", "wasm"], true, false),
        (vec!["-os=wasm"], true, false),
        (
            vec!["-os=linux", "-target=wasm64-unknown-wasi", "-os=windows"],
            true,
            false,
        ),
        (vec!["-os=wasm", "-os=wasm", "-cpu=aarch64"], true, false),
        (vec!["-os=wasm", host.as_str()], false, false),
        (vec!["-os=wasm", "-cpu=aarch64", "-interp"], true, true),
    ] {
        let (root, jacket) = fixture();
        let test = root.join("tests/target.jai");
        std::fs::write(&test, "placeholder :: () {} @test\n").unwrap();
        let log = root.join("args.jsonl");
        let output = Command::new(&jacket)
            .current_dir(&root)
            .args([
                "test", "--os", "windows", "--format", "json", "--runner", "python3", "--",
            ])
            .args(&flags)
            .env("JAIC_STDLIB", root.join("stdlib"))
            .env("TARGET_LOG", &log)
            .env("TARGET_TEST", &test)
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "{flags:?}: {text}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let outcomes: Vec<serde_json::Value> = text
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();
        let expected = if wasm {
            [
                ("wasm_only", "passed"),
                ("native_only", "skipped"),
                ("native_cpu", "skipped"),
            ]
        } else {
            [
                ("wasm_only", "skipped"),
                ("native_only", "passed"),
                ("native_cpu", "passed"),
            ]
        };
        for (name, status) in expected {
            assert!(
                outcomes
                    .iter()
                    .any(|o| o["name"] == name && o["event"] == status),
                "{flags:?}: {text}"
            );
        }
        let invocations: Vec<Vec<String>> = std::fs::read_to_string(&log)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let target_flags: Vec<_> = flags.iter().copied().filter(|s| *s != "-interp").collect();
        assert!(
            invocations.iter().any(|args| args
                .windows(target_flags.len())
                .any(|slice| slice == target_flags)),
            "{invocations:?}"
        );
        if interp {
            assert!(
                !invocations
                    .iter()
                    .any(|args| args.iter().any(|s| s == "-exe"))
            );
        } else {
            let build = invocations
                .iter()
                .find(|args| args.iter().any(|s| s == "-exe"))
                .unwrap();
            assert_eq!(
                build.iter().any(|arg| arg.ends_with("tests.wasm")),
                wasm,
                "{build:?}"
            );
        }
        let listed = Command::new(&jacket)
            .current_dir(&root)
            .args(["test", "--list", "--"])
            .args(&flags)
            .args(["-add", "-os"])
            .env("JAIC_STDLIB", root.join("stdlib"))
            .env("TARGET_LOG", &log)
            .env("TARGET_TEST", &test)
            .output()
            .unwrap();
        assert!(listed.status.success(), "{listed:?}");
        let logs = std::fs::read_to_string(&log).unwrap();
        let discovery: Vec<String> = serde_json::from_str(logs.lines().last().unwrap()).unwrap();
        assert!(
            discovery.windows(2).any(|pair| pair == ["-add", "-os"]),
            "{discovery:?}"
        );
        assert!(
            !discovery.iter().any(|arg| arg.starts_with("-os=")
                || arg.starts_with("-target")
                || arg.starts_with("-cpu")),
            "{discovery:?}"
        );
    }
}

#[test]
fn compile_fail_cases_use_checking_targets_without_native_link_restrictions() {
    let (root, jacket) = fixture();
    let tests = root.join("tests/compile_fail");
    std::fs::create_dir_all(&tests).unwrap();
    let source = tests.join("cross.jai");
    std::fs::write(&source, "// test: only(.X64)\ninvalid source\n").unwrap();
    std::fs::write(tests.join("cross.stderr"), "known compile error\n").unwrap();
    let foreign = if cfg!(target_os = "linux") {
        "macos"
    } else {
        "linux"
    };
    let log = root.join("args.jsonl");
    for modes in [vec![], vec!["--interp"], vec!["--", "-interp"]] {
        let output = Command::new(&jacket)
            .current_dir(&root)
            .args(["test", "--format", "json", "--os", foreign])
            .args(modes)
            .env("JAIC_STDLIB", root.join("stdlib"))
            .env("TARGET_LOG", &log)
            .env("TARGET_TEST", &source)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let invocations: Vec<Vec<String>> = std::fs::read_to_string(&log)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert!(
            invocations
                .iter()
                .any(|args| args.iter().any(|s| s == "-check"))
        );
        assert!(
            !invocations
                .iter()
                .any(|args| args.iter().any(|s| s == "-exe"))
        );
    }
}


#[test]
fn forwarded_mode_conflicts_and_dependencies_are_rejected_before_compiling() {
    let (root, jacket) = fixture();
    for flags in [
        vec!["-check", "-interp"],
        vec!["-deps", "-interp"],
        vec!["-deps"],
    ] {
        let output = Command::new(&jacket)
            .current_dir(&root)
            .args(["test", "--doc", "--check-only", "--"])
            .args(&flags)
            .output()
            .unwrap();
        assert!(!output.status.success());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains(if flags.len() == 2 {
                "cannot be combined"
            } else {
                "--list --deps"
            }),
            "{error}"
        );
    }
}

#[test]
fn forwarded_benchmark_modes_reject_before_invoking_the_compiler() {
    for flags in [
        vec!["-os", "wasm"],
        vec!["-os=wasm"],
        vec!["-target=wasm64-unknown-wasi"],
        vec!["-interp"],
    ] {
        let (root, jacket) = fixture();
        let log = root.join("args.jsonl");
        let output = Command::new(&jacket)
            .current_dir(&root)
            .args(["bench", "--"])
            .args(&flags)
            .env("JAIC_STDLIB", root.join("stdlib"))
            .env("TARGET_LOG", &log)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{flags:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("measures native builds"),
            "{output:?}"
        );
        assert!(
            !log.exists(),
            "foreign or interpreted benchmark must not invoke the compiler"
        );
    }
}

#[test]
fn forwarded_check_docs_use_the_effective_checking_target_without_running() {
    let (root, jacket) = fixture();
    let module = root.join("module.jai");
    std::fs::write(&module, "// controlled documented library\n").unwrap();
    let log = root.join("args.jsonl");
    let foreign = if cfg!(target_os = "linux") {
        "macos"
    } else {
        "linux"
    };
    let record = format!(
        "{}\t1\t\t```jai only(.X64)\\nvalue :: 1;\\n```",
        module.display()
    );
    let output = Command::new(&jacket)
        .current_dir(&root)
        .args([
            "test",
            "--doc",
            "--check-only",
            "--format",
            "json",
            "--os",
            "wasm",
            "--",
            "-check",
            "-os",
            foreign,
            "-cpu=x64",
        ])
        .env("JAIC_STDLIB", root.join("stdlib"))
        .env("TARGET_LOG", &log)
        .env("TARGET_DOC_RECORD", record)
        .env("TARGET_CHECK_SUCCESS", "1")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "{text}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let outcomes: Vec<serde_json::Value> = text
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| outcome["kind"] == "doc" && outcome["event"] == "passed")
            .count(),
        1,
        "{text}"
    );
    let invocations: Vec<Vec<String>> = std::fs::read_to_string(&log)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let checks: Vec<_> = invocations
        .iter()
        .filter(|args| args.iter().any(|arg| arg == "-check"))
        .collect();
    assert_eq!(checks.len(), 1, "{invocations:?}");
    assert_eq!(
        checks[0]
            .iter()
            .filter(|arg| arg.as_str() == "-check")
            .count(),
        1
    );
    assert!(
        checks[0]
            .windows(3)
            .any(|args| args == ["-os", foreign, "-cpu=x64"]),
        "{checks:?}"
    );
    assert!(
        !invocations
            .iter()
            .flatten()
            .any(|arg| arg == "-interp" || arg == "-exe"),
        "{invocations:?}"
    );
}
