//! `jaic build -os wasm`: wasm64 modules linked by wasm-ld, run under node's WASI
//! (docs/native/wasm-target.md). Each test is skipped, with a note, when node or wasm-ld is
//! missing, unless `JAIC_REQUIRE_WASM_TESTS=1` (CI sets it).
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use super::{JAIC, corpus_cases, matches, repo_root};

fn required() -> bool {
    std::env::var("JAIC_REQUIRE_WASM_TESTS").is_ok_and(|v| v == "1")
}

/// node's major version, when node runs.
fn node_major() -> Option<u32> {
    let output = Command::new("node").arg("--version").output().ok()?;
    let version = String::from_utf8_lossy(&output.stdout);
    version
        .trim()
        .trim_start_matches('v')
        .split('.')
        .next()?
        .parse()
        .ok()
}

/// `node` with the flags our modules need: Memory64 is on by default from node 24.
fn node() -> Option<Command> {
    let major = node_major()?;
    let mut command = Command::new("node");
    command.arg("--no-warnings");
    if major < 24 {
        command.arg("--experimental-wasm-memory64");
    }
    Some(command)
}

/// A scratch directory for one test, emptied first.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Runs `jaic` with `args` in `dir`. `None` (the test is skipped) when there is no node or the
/// build cannot find wasm-ld and the tests are not required.
fn jaic(dir: &Path, args: &[&str]) -> Option<Output> {
    if node_major().is_none() {
        assert!(!required(), "node is required for the wasm tests");
        eprintln!("skipped: node not found");
        return None;
    }
    let output = Command::new(JAIC)
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("wasm-ld not found") && !required() {
        eprintln!("skipped: {stderr}");
        return None;
    }
    assert!(output.status.success(), "jaic {args:?} failed:\n{stderr}");
    Some(output)
}

/// Runs a WASI command module through tools/wasi_run.mjs, with `stdin` as its input.
fn wasi_run(module: &Path, args: &[&str], stdin: &str) -> Output {
    let mut child = node()
        .unwrap()
        .arg(repo_root().join("tools/wasi_run.mjs"))
        .arg(module)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// A module's imports as `module.name`, sorted.
fn imports(module: &Path) -> Vec<String> {
    let script = "const m = new WebAssembly.Module(require('fs').readFileSync(process.argv[1]));\
                  console.log(WebAssembly.Module.imports(m).map(i => i.module + '.' + i.name).sort().join('\\n'));";
    let output = node()
        .unwrap()
        .args(["--input-type=commonjs", "-e", script])
        .arg(module)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", text(&output.stderr));
    text(&output.stdout).lines().map(str::to_string).collect()
}

/// Hello world under WASI: stdout, stderr, stdin, arguments, the environment and the exit code
/// all go through `wasi_snapshot_preview1`, the only module the program imports.
#[test]
fn wasm_hello_reads_stdin_and_exits_with_its_status() {
    let dir = scratch("wasm-hello");
    std::fs::write(
        dir.join("hello.jai"),
        r#"#import "Basic";
libc :: #system_library "libc";
c_read :: (fd: s32, buffer: *void, count: u64) -> s64 #foreign libc "read";
c_write :: (fd: s32, buffer: *void, count: u64) -> s64 #foreign libc "write";
c_getenv :: (name: *u8) -> *u8 #foreign libc "getenv";
main :: () -> s32 {
    args := get_command_line_arguments();
    print("hello % from %\n", args.count - 1, OS);
    buffer: [64] u8;
    count := c_read(0, buffer.data, buffer.count);
    print("read %\n", to_string(buffer.data, count));
    print("env %\n", to_string(c_getenv("JAIC_WASM_TEST")));
    c_write(2, "to stderr\n".data, 10);
    return cast(s32) args.count + 40;
}
"#,
    )
    .unwrap();
    if jaic(
        &dir,
        &["build", "hello.jai", "-os", "wasm", "-o", "hello.wasm"],
    )
    .is_none()
    {
        return;
    }
    let module = dir.join("hello.wasm");
    assert_eq!(&std::fs::read(&module).unwrap()[..4], b"\0asm");
    let mut run = node().unwrap();
    let output = run
        .arg(repo_root().join("tools/wasi_run.mjs"))
        .arg(&module)
        .args(["one", "two"])
        .env("JAIC_WASM_TEST", "set")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(text(&output.stdout), "hello 2 from WASM\nread \nenv set\n");
    assert_eq!(text(&output.stderr), "to stderr\n");
    assert_eq!(output.status.code(), Some(43));
    let piped = wasi_run(&module, &[], "piped");
    assert!(text(&piped.stdout).contains("read piped\n"));
    assert_eq!(piped.status.code(), Some(41));
    for import in imports(&module) {
        assert!(
            import.starts_with("wasi_snapshot_preview1."),
            "unexpected import {import}"
        );
    }
}

/// Hash tables, a million-element sort and string building: the Wasi_Runtime heap grows the
/// memory well past its first pages.
#[test]
fn wasm_allocation_heavy_program() {
    let dir = scratch("wasm-program");
    let source = repo_root().join("tests/native/wasm/program.jai");
    let source = source.to_str().unwrap();
    for opt in ["-O0", "-O2"] {
        if jaic(
            &dir,
            &["build", source, "-os", "wasm", opt, "-o", "program.wasm"],
        )
        .is_none()
        {
            return;
        }
        let output = wasi_run(&dir.join("program.wasm"), &["a", "b"], "");
        assert_eq!(
            text(&output.stdout),
            "arg 1: a\narg 2: b\nOS WASM CPU CUSTOM pointer 8 bytes\ntable 10000 149985000\n\
             sorted 1000000 0 999\ntext 1890 501\nfloat 6.5 1.414214\n",
            "{opt}: {}",
            text(&output.stderr)
        );
        assert_eq!(output.status.code(), Some(2), "{opt}");
    }
}

/// A metaprogram targets wasm the way it would in Jai: os_target .WASM, cpu_target .CUSTOM,
/// an LLVM triple and features, and extra wasm-ld arguments.
#[test]
fn wasm_build_options_from_a_metaprogram() {
    let dir = scratch("wasm-metaprogram");
    for name in ["build_wasi.jai", "program.jai"] {
        std::fs::copy(
            repo_root().join("tests/native/wasm").join(name),
            dir.join(name),
        )
        .unwrap();
    }
    if jaic(&dir, &["build", "build_wasi.jai"]).is_none() {
        return;
    }
    let output = wasi_run(&dir.join("program.wasm"), &[], "");
    assert!(
        text(&output.stdout).contains("sorted 1000000 0 999\n"),
        "{}{}",
        text(&output.stdout),
        text(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
}

/// Without "wasi" in the triple there is no runtime: `#foreign` procedures without a library
/// import from "env", a `#system_library` names its own import module, and `#program_export`
/// procedures are the module's exports, called by the host.
#[test]
fn wasm_bare_module_imports_and_exports() {
    let dir = scratch("wasm-exports");
    let source = repo_root().join("tests/native/wasm/exports.jai");
    let args = [
        "build",
        source.to_str().unwrap(),
        "-target",
        "wasm64-unknown-unknown",
        "-O2",
        "-o",
        "exports.wasm",
    ];
    if jaic(&dir, &args).is_none() {
        return;
    }
    let module = dir.join("exports.wasm");
    let imports = imports(&module);
    assert!(
        imports.contains(&"env.host_report".to_string()),
        "{imports:?}"
    );
    assert!(
        imports.contains(&"host_graphics.draw".to_string()),
        "{imports:?}"
    );
    assert!(
        !imports.iter().any(|i| i.starts_with("wasi")),
        "{imports:?}"
    );
    let output = node()
        .unwrap()
        .arg(repo_root().join("tests/native/wasm/exports.mjs"))
        .arg(&module)
        .output()
        .unwrap();
    let stdout = text(&output.stdout);
    assert!(output.status.success(), "{}", text(&output.stderr));
    assert!(
        stdout.contains("report triangle 5050\ntriangle 5050\n"),
        "{stdout}"
    );
    assert!(stdout.ends_with("count_bits 11\n"), "{stdout}");
}

/// jaifmt compiled to WebAssembly formats every golden case exactly as expected
/// (tools/check_jaifmt_wasm.mjs, which CI also runs against native jaifmt).
#[test]
fn jaifmt_wasm_formats_the_golden_cases() {
    let dir = scratch("wasm-jaifmt");
    let source = repo_root().join("tools/jaifmt/wasm.jai");
    let args = [
        "build",
        source.to_str().unwrap(),
        "-os",
        "wasm",
        "-O2",
        "--no-debug-info",
        "-o",
        "jaifmt.wasm",
    ];
    if jaic(&dir, &args).is_none() {
        return;
    }
    let output = node()
        .unwrap()
        .arg(repo_root().join("tools/check_jaifmt_wasm.mjs"))
        .arg(dir.join("jaifmt.wasm"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        text(&output.stdout),
        text(&output.stderr)
    );
    let piped = wasi_run(&dir.join("jaifmt.wasm"), &[], "f :: ()\n{\n  x:=1;\n}\n");
    assert_eq!(text(&piped.stdout), "f :: () {\n    x := 1;\n}\n");
    assert_eq!(piped.status.code(), Some(0));
}

/// Every corpus case the interpreter passes behaves the same as a WASI module.
#[test]
fn corpus_runs_as_wasm_like_the_interpreter() {
    let dir = scratch("wasm-corpus");
    let (mut checked, mut failures) = (0, Vec::new());
    for case in corpus_cases() {
        let interp = Command::new(JAIC)
            .arg("run")
            .arg(&case.source)
            .current_dir(case.source.parent().unwrap())
            .output()
            .unwrap();
        if !matches(&case, &interp) {
            continue;
        }
        let module = dir.join(format!("{}.wasm", case.id));
        let args = [
            "build",
            case.source.to_str().unwrap(),
            "-os",
            "wasm",
            "-o",
            module.to_str().unwrap(),
        ];
        let Some(_) = jaic(case.source.parent().unwrap(), &args) else {
            return;
        };
        checked += 1;
        let output = wasi_run(&module, &[], "");
        if !matches(&case, &output) {
            failures.push(format!(
                "{}: wasm exit {:?}, stdout {:?}, stderr {:?} (expected exit {}, stdout {:?})",
                case.id,
                output.status.code(),
                text(&output.stdout),
                text(&output.stderr),
                case.exit_code,
                case.stdout
            ));
        }
    }
    assert!(checked > 0, "no corpus case passes under the interpreter");
    assert!(
        failures.is_empty(),
        "{} of {checked} cases differ as wasm:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
