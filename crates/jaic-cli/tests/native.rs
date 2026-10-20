//! Native backend acceptance: every corpus case with a runtime expectation
//! that passes under `jaic -interp` must also pass when built with a plain `jaic`
//! and executed as a native program.
use std::path::{Path, PathBuf};
use std::process::Command;

mod abi_layout;
#[path = "native/c_abi.rs"]
mod c_abi;
#[path = "native/checks.rs"]
mod checks;
mod common;
#[path = "native/fibers_and_options.rs"]
mod fibers_and_options;
mod formatter_fixtures;
#[path = "native/jaifmt_and_windows.rs"]
mod jaifmt_and_windows;
#[path = "native/plugins_and_links.rs"]
mod plugins_and_links;
mod wasm_target;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

/// A `jaic` command. `JACKET_TEST_CODEGEN_UNITS=N` (set by the CI sweep that divides every native
/// build into parts, docs/native/llvm-backend.md) starts it with `-codegen_units N`; a test that
/// pins its own count passes the flag again later, and the last one wins.
fn jaic() -> Command {
    let mut command = Command::new(JAIC);
    if let Some(units) = std::env::var_os("JACKET_TEST_CODEGEN_UNITS") {
        command.arg("-codegen_units").arg(units);
    }
    command
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The LLVM IR a build wrote to `path`, with the IR of the other codegen units appended. A build
/// with several units (`-codegen_units`) writes unit `n` to `x.n.ll` next to `x.ll`, so a
/// test that looks for a definition, or for the absence of one, reads all of them.
fn read_ir_parts(path: &Path) -> String {
    let mut text = std::fs::read_to_string(path).unwrap();
    let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
    let extension = path.extension().unwrap().to_string_lossy().into_owned();
    for unit in 1.. {
        let part = path.with_file_name(format!("{stem}.{unit}.{extension}"));
        match std::fs::read_to_string(&part) {
            Ok(more) => text.push_str(&more),
            Err(_) => break,
        }
    }
    text
}

struct Case {
    id: String,
    source: PathBuf,
    exit_code: i32,
    stdout: String,
    /// The runtime error the program must stop with (any failing status, the message on
    /// stderr), instead of `exit_code`.
    error: Option<String>,
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
                error: runtime["error"].as_str().map(str::to_string),
            })
        })
        .collect()
}

fn matches(case: &Case, output: &std::process::Output) -> bool {
    let status = match &case.error {
        Some(error) => {
            !output.status.success() && String::from_utf8_lossy(&output.stderr).contains(error)
        }
        None => output.status.code() == Some(case.exit_code),
    };
    status && String::from_utf8_lossy(&output.stdout) == case.stdout
}

/// The path of executable `name` in `dir` (`name.exe` on Windows, where `jaic` adds it).
fn exe_path(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
}

/// Build `source` natively into `dir` and run it.
fn build_and_run(source: &Path, dir: &Path, name: &str) -> Result<std::process::Output, String> {
    let exe = exe_path(dir, name);
    let build = jaic()
        .arg(source)
        .arg("-exe")
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
        let interp = jaic()
            .arg("-interp")
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

/// The program's own unreferenced procedures are type-checked but not compiled in, nor what
/// only they (or an unreferenced global's initializer) call.
// rules: dce.12
#[test]
fn unreferenced_code_is_not_compiled() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-unreferenced");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("unreferenced.jai");
    std::fs::write(
        &source,
        "#import \"Basic\";\n\
         unused_entry_qz :: () -> int { return only_from_unused_qz() + 1; }\n\
         only_from_unused_qz :: () -> int { return 41; }\n\
         unused_table_qz: [2] () -> int = .[only_from_unused_qz, unused_entry_qz];\n\
         used_helper_qz :: () -> int { return 7; }\n\
         main :: () { print(\"%\\n\", used_helper_qz()); }\n",
    )
    .unwrap();
    let ir = dir.join("unreferenced.ll");
    let exe = exe_path(&dir, "unreferenced");
    let build = jaic()
        .arg(&source)
        .arg("-exe")
        .arg(&exe)
        .arg("-emit_ir")
        .arg(&ir)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    // Every part, when `-codegen_units` divides the program: nothing unreferenced is compiled
    // into any of them.
    let ir = read_ir_parts(&ir);
    assert!(ir.contains("used_helper_qz"));
    assert!(
        !ir.contains("unused_entry_qz"),
        "an unreferenced procedure was compiled"
    );
    assert!(
        !ir.contains("only_from_unused_qz"),
        "a procedure only unreferenced code calls was compiled"
    );
    let run = Command::new(&exe).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&run.stdout), "7\n");
}

/// A struct or union passed by value arrives as a pointer the callee copies from. The debug
/// A procedure that only does arithmetic, or calls only such procedures, pushes no stack trace node
/// even though it keeps its context in a slot; one that reads the context, asserts or calls through a
/// procedure value still does.
#[test]
fn quiet_procedures_push_no_stack_trace_node() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-quiet-procedures");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("quiet.jai");
    std::fs::write(
        &source,
        "#import \"Basic\";\n\
         leaf_qz :: (a: int, b: int) -> int { return a - b; }\n\
         chain_qz :: (a: int) -> int { return leaf_qz(a, 1) + leaf_qz(a, 2); }\n\
         reads_qz :: () -> bool { return context.stack_trace != null; }\n\
         through_qz :: (f: (int, int) -> int) -> int { return f(3, 4); }\n\
         main :: () { print(\"% % %\\n\", chain_qz(5), reads_qz(), through_qz(leaf_qz)); }\n",
    )
    .unwrap();
    let ir = dir.join("quiet.ll");
    let exe = exe_path(&dir, "quiet");
    let build = jaic()
        .arg(&source)
        .arg("-exe")
        .arg(&exe)
        .arg("-emit_ir")
        .arg(&ir)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let ir = read_ir_parts(&ir);
    let pushes_node = |name: &str| {
        ir.split("\ndefine ")
            .skip(1)
            .find(|f| {
                f.lines()
                    .next()
                    .is_some_and(|l| l.contains(&format!("@{name}.")))
            })
            .unwrap_or_else(|| panic!("{name} is compiled"))
            .contains("@trace_info.")
    };
    assert!(!pushes_node("leaf_qz"));
    assert!(!pushes_node("chain_qz"));
    assert!(pushes_node("reads_qz"));
    assert!(pushes_node("through_qz"));
    let run = Command::new(&exe).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&run.stdout), "7 true -1\n");
}

/// info's prologue spill of parameters must store only scalars: storing that pointer into the
/// parameter's slot wrote 8 bytes into a smaller slot, which corrupted neighbouring locals
/// (crashes at -O0 on Linux) and tripped the sanitizers' bounds checks.
#[test]
fn small_aggregate_parameters_are_not_spilled_as_pointers() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-small-aggregate-params");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("small.jai");
    std::fs::write(
        &source,
        "#import \"Basic\";\n\
         Tiny :: struct { a: u8; b: u8; }\n\
         Word :: union { bits: u32; real: float32; }\n\
         sum_small_qz :: (t: Tiny, w: Word, n: s64) -> s64 { return t.a + t.b + w.bits + n; }\n\
         main :: () { print(\"%\\n\", sum_small_qz(.{ 1, 2 }, .{ bits = 1000 }, 4)); }\n",
    )
    .unwrap();
    let ir = dir.join("small.ll");
    let exe = exe_path(&dir, "small");
    let build = jaic()
        .arg(&source)
        .arg("-exe")
        .arg(&exe)
        .arg("-emit_ir")
        .arg(&ir)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let ir = read_ir_parts(&ir);
    let body = ir
        .split("\ndefine ")
        .skip(1)
        .find(|f| {
            f.lines()
                .next()
                .is_some_and(|l| l.contains("@sum_small_qz"))
        })
        .expect("sum_small_qz is compiled");
    let prologue = body.split("\nb0:").next().unwrap();
    let mut sizes = jaic::fxhash::HashMap::default();
    for line in prologue.lines() {
        if let Some((slot, rest)) = line.trim().split_once(" = alloca [")
            && let Some((bytes, _)) = rest.split_once(" x i8]")
        {
            sizes.insert(slot.to_string(), bytes.parse::<u64>().unwrap());
        }
    }
    for line in prologue.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("store ptr ")
            && let Some((_, slot)) = rest.split_once(", ptr ")
        {
            let slot = slot.split(',').next().unwrap();
            assert!(
                sizes.get(slot).is_some_and(|&n| n >= 8),
                "a pointer is stored into the {:?}-byte slot {slot}: {line}\n{prologue}",
                sizes.get(slot)
            );
        }
    }
    let run = Command::new(&exe).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&run.stdout), "1007\n");
}

/// Compiler and C ABI regressions whose bugs showed only in compiled code.
#[test]
fn compiler_and_abi_regressions_run_natively() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-stdlib-tests");
    std::fs::create_dir_all(&dir).unwrap();
    for name in [
        "struct-literal-overrides-default-string",
        "array-literal-view-lifetime",
        "over-aligned-allocation",
        "proc-sentinel-constant",
        "add-context-constant",
        "posix-stat-and-mutex",
        "no-reset-globals-baked",
        "intrinsic-byte-lookup",
        "poly-inference-forms",
        "run-result-frozen-values",
        "hash-table-remove-while-iterating",
    ] {
        // This tests the POSIX process sentinel ABI; Windows has its own SDK regressions.
        if cfg!(windows) && name == "proc-sentinel-constant" {
            continue;
        }
        let source = repo_root().join(match name {
            "proc-sentinel-constant" => {
                "stdlib/Compiler/tests/proc-sentinel-constant.jai".to_string()
            }
            "posix-stat-and-mutex" => "stdlib/POSIX/tests/stat-and-mutex.jai".to_string(),
            _ => format!("tests/compiler/regressions/{name}.jai"),
        });
        let output = build_and_run(&source, &dir, name).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "ok\n",
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// File reads preserve piped standard input and EOF; the fixture owns
/// line splitting because File exports no `read_stdin_line` helper.
#[test]
fn stdin_lines_are_read_natively() {
    use std::io::Write;
    use std::process::Stdio;
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-stdin-lines");
    std::fs::create_dir_all(&dir).unwrap();
    let source = repo_root().join("tests/compiler/native/stdin-lines/main.jai");
    let exe = exe_path(&dir, "stdin-lines");
    let build = jaic()
        .arg(&source)
        .arg("-exe")
        .arg(&exe)
        .current_dir(source.parent().unwrap())
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let mut child = Command::new(&exe)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"one\r\n\ntwo words\nlast")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "[one][][two words][last]\n"
    );
}

/// `#asm` lowers to portable IR, so the instruction tests (expectations recorded on x86 hardware, or
/// hand-computed for AVX-512) must also pass in compiled code on any host, arm64 included.
#[test]
fn asm_instructions_run_natively() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-asm-tests");
    std::fs::create_dir_all(&dir).unwrap();
    for name in [
        "asm-scalar-extended",
        "asm-simd-extended",
        "asm-avx512-masks",
        "asm-f16c-sha-gfni",
        "asm-string-compare",
        "asm-string-compare-operands",
    ] {
        let source = repo_root().join(format!("tests/compiler/regressions/{name}.jai"));
        let output = build_and_run(&source, &dir, name).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "ok\n",
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// The SSE4.2 string compares give the same results when interpreted, whatever the host.
#[test]
fn asm_string_compares_run_interpreted() {
    for name in ["asm-string-compare", "asm-string-compare-operands"] {
        let source = repo_root().join(format!("tests/compiler/regressions/{name}.jai"));
        let output = jaic().arg("-interp").arg(&source).output().unwrap();
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "ok\n",
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// Workspaces a `#run` creates without adding any source have nothing to compile: `jaic`
/// used to link each one as an executable without `main` and fail.
#[test]
fn empty_workspaces_write_no_output() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-empty-workspaces");
    std::fs::create_dir_all(&dir).unwrap();
    let source = repo_root().join("tests/compiler/regressions/compiler-workspace-ids.jai");
    let output = build_and_run(&source, &dir, "workspace-ids").unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let source = repo_root().join("tests/compiler/semantics/workspace-identity.jai");
    let output = build_and_run(&source, &dir, "workspace-identity").unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ok\n");
    let interpreted = jaic().arg("-interp").arg(source).output().unwrap();
    assert!(
        interpreted.status.success(),
        "{}",
        String::from_utf8_lossy(&interpreted.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&interpreted.stdout), "ok\n");
}

/// Compiled code maintains `context.stack_trace` (`jaic::stack_trace::instrument`): call lines, depth,
/// a trace through an inline procedure, and the trace an assertion prints.
#[test]
fn stack_traces() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-stack-trace");
    std::fs::create_dir_all(&dir).unwrap();
    let ok = repo_root().join("tests/compiler/regressions/stack-trace-line-through-inline.jai");
    let output = build_and_run(&ok, &dir, "through-inline").unwrap();
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ok\n");
    let source = dir.join("trace.jai");
    std::fs::write(
        &source,
        "#import \"Basic\";\n\
         depth :: () -> int { n := 0; node := context.stack_trace; while node { n += 1; node = node.next; } return n; }\n\
         inner :: () -> int { return depth(); }\n\
         main :: () {\n\
             print(\"% %\\n\", depth(), inner());\n\
             assert(false, \"boom\");\n\
         }\n",
    )
    .unwrap();
    let output = build_and_run(&source, &dir, "trace").unwrap();
    assert_eq!(String::from_utf8_lossy(&output.stdout), "2 3\n");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("assertion failed: boom\nstack trace:\n")
            && stderr.contains("    main at "),
        "{stderr}"
    );
    assert!(
        stderr.contains("trace.jai:6\n") && !stderr.contains("assert_helper"),
        "{stderr}"
    );
    // The runtime reports the caller location; absolute-versus-relative
    // presentation is not a compatibility contract.
    assert!(
        stderr.lines().next().unwrap().contains("trace.jai:6:")
            && stderr
                .lines()
                .any(|line| line.contains("main at ") && line.ends_with("trace.jai:6")),
        "{stderr}"
    );
    assert!(!output.status.success());
}
