//! Native backend acceptance: every corpus case with a runtime expectation
//! that passes under `jaic run` must also pass when built with `jaic build`
//! and executed as a native program.
use std::path::{Path, PathBuf};
use std::process::Command;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

struct Case {
    id: String,
    source: PathBuf,
    exit_code: i32,
    stdout: String,
}

fn corpus_cases() -> Vec<Case> {
    let corpus = repo_root().join("tests/corpus");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(corpus.join("manifest.json")).unwrap()).unwrap();
    manifest["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| {
            let runtime = c.get("runtime")?;
            Some(Case {
                id: c["id"].as_str()?.to_string(),
                source: corpus.join(c["source"].as_str()?),
                exit_code: runtime["exit_code"].as_i64().unwrap_or(0) as i32,
                stdout: runtime["stdout"].as_str().unwrap_or("").to_string(),
            })
        })
        .collect()
}

fn matches(case: &Case, output: &std::process::Output) -> bool {
    output.status.code() == Some(case.exit_code)
        && String::from_utf8_lossy(&output.stdout) == case.stdout
}

/// Build `source` natively into `dir` and run it.
fn build_and_run(source: &Path, dir: &Path, name: &str) -> Result<std::process::Output, String> {
    let exe = dir.join(name);
    let build = Command::new(JAIC)
        .arg("build")
        .arg(source)
        .arg("-o")
        .arg(&exe)
        .current_dir(source.parent().unwrap())
        .output()
        .map_err(|e| e.to_string())?;
    if !build.status.success() {
        return Err(format!(
            "build failed: {}",
            String::from_utf8_lossy(&build.stderr)
        ));
    }
    Command::new(&exe).output().map_err(|e| e.to_string())
}

#[test]
fn corpus_runs_natively_like_the_interpreter() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-corpus");
    std::fs::create_dir_all(&dir).unwrap();
    let (mut checked, mut failures) = (0, Vec::new());
    for case in corpus_cases() {
        // Only cases the interpreter already passes are meaningful for the backend.
        let interp = Command::new(JAIC)
            .arg("run")
            .arg(&case.source)
            .current_dir(case.source.parent().unwrap())
            .output()
            .unwrap();
        if !matches(&case, &interp) {
            continue;
        }
        checked += 1;
        match build_and_run(&case.source, &dir, &case.id) {
            Ok(native) if matches(&case, &native) => {}
            Ok(native) => failures.push(format!(
                "{}: native exit {:?}, stdout {:?} (expected exit {}, stdout {:?})",
                case.id,
                native.status.code(),
                String::from_utf8_lossy(&native.stdout),
                case.exit_code,
                case.stdout
            )),
            Err(e) => failures.push(format!("{}: {e}", case.id)),
        }
    }
    assert!(checked > 0, "no corpus case passes under the interpreter");
    assert!(
        failures.is_empty(),
        "{} of {checked} cases differ natively:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn hello_world_builds_and_prints() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-hello");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("hello.jai");
    std::fs::write(
        &source,
        "#import \"Basic\";\nmain :: () { print(\"hello %!\\n\", 42); }\n",
    )
    .unwrap();
    let output = build_and_run(&source, &dir, "hello").unwrap();
    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello 42!\n");
    assert_eq!(output.status.code(), Some(0));
}
