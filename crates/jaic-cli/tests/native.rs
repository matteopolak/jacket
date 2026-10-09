//! Native backend acceptance: every corpus case with a runtime expectation
//! that passes under `jaic run` must also pass when built with `jaic build`
//! and executed as a native program.
use std::path::{Path, PathBuf};
use std::process::Command;

mod abi_layout;
mod common;
mod wasm_target;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
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

/// The path of executable `name` in `dir` (`name.exe` on Windows, where `jaic build` adds it).
fn exe_path(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
}

/// Build `source` natively into `dir` and run it.
fn build_and_run(source: &Path, dir: &Path, name: &str) -> Result<std::process::Output, String> {
    let exe = exe_path(dir, name);
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
    let build = Command::new(JAIC)
        .arg("build")
        .arg(&source)
        .arg("-o")
        .arg(&exe)
        .arg("--emit-ir")
        .arg(&ir)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let ir = std::fs::read_to_string(&ir).unwrap();
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
    let build = Command::new(JAIC)
        .arg("build")
        .arg(&source)
        .arg("-o")
        .arg(&exe)
        .arg("--emit-ir")
        .arg(&ir)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let ir = std::fs::read_to_string(&ir).unwrap();
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
    let mut sizes = std::collections::HashMap::new();
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

/// Self-checking stdlib tests whose bugs showed only in compiled code; each prints "ok".
#[test]
fn stdlib_tests_run_natively() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-stdlib-tests");
    std::fs::create_dir_all(&dir).unwrap();
    for name in [
        "struct-literal-overrides-default-string",
        "array-literal-view-lifetime",
        "over-aligned-allocation",
        "proc-sentinel-constant",
        "add-context-constant",
        "process-stdin-socket",
        "posix-stat-and-mutex",
        "no-reset-globals-baked",
    ] {
        // These two import POSIX, which does not build for Windows.
        if cfg!(windows) && matches!(name, "proc-sentinel-constant" | "process-stdin-socket") {
            continue;
        }
        let source = repo_root().join(format!("tests/stdlib/{name}.jai"));
        let output = build_and_run(&source, &dir, name).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "ok\n",
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// `read_stdin_line()` reads piped standard input in a compiled program, and an
/// exhausted pipe ends the loop.
#[test]
fn stdin_lines_are_read_natively() {
    use std::io::Write;
    use std::process::Stdio;
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-stdin-lines");
    std::fs::create_dir_all(&dir).unwrap();
    let source = repo_root().join("tests/native/stdin-lines/main.jai");
    let exe = exe_path(&dir, "stdin-lines");
    let build = Command::new(JAIC)
        .arg("build")
        .arg(&source)
        .arg("-o")
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
    ] {
        let source = repo_root().join(format!("tests/stdlib/{name}.jai"));
        let output = build_and_run(&source, &dir, name).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "ok\n",
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// Workspaces a `#run` creates without adding any source have nothing to compile: `jaic build`
/// used to link each one as an executable without `main` and fail.
#[test]
fn empty_workspaces_write_no_output() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-empty-workspaces");
    std::fs::create_dir_all(&dir).unwrap();
    let source = repo_root().join("tests/stdlib/compiler-workspace-ids.jai");
    let output = build_and_run(&source, &dir, "workspace-ids").unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Compiled code maintains `context.stack_trace` (`jaic::stack_trace::instrument`): call lines, depth,
/// a trace through an inline procedure, and the trace an assertion prints.
#[test]
fn stack_traces() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-stack-trace");
    std::fs::create_dir_all(&dir).unwrap();
    let ok = repo_root().join("tests/stdlib/stack-trace-line-through-inline.jai");
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
        stderr
            .contains("error: assertion failed: boom\ncall stack (innermost first):\n    main at "),
        "{stderr}"
    );
    assert!(
        stderr.contains("trace.jai:6\n") && !stderr.contains("assert_helper"),
        "{stderr}"
    );
    // Paths under the directory the build ran in are shown relative to it, as `jaic run` does.
    assert!(
        stderr.starts_with("trace.jai:6:") && stderr.contains("    main at trace.jai:6\n"),
        "{stderr}"
    );
    assert!(!output.status.success());
}

fn abi_manifest(target: &str) -> abi_layout::Manifest {
    let text = std::fs::read_to_string(repo_root().join("tests/abi/manifest.txt")).unwrap();
    abi_layout::parse(&text, target).unwrap_or_else(|e| panic!("{e}"))
}

/// The stdlib's hand-written C declarations (`tests/abi/manifest.txt`) against the host's own
/// headers: sizes, alignments, field offsets, integer signedness and constants. Every CI host
/// runs this, so a layout copied from another CPU or OS fails on the host it is wrong for.
#[test]
fn stdlib_c_abi_matches_host_headers() {
    let os = match std::env::consts::OS {
        "linux" => "linux",
        "macos" => "macos",
        "windows" => "windows",
        other => {
            eprintln!("skipping: no ABI manifest section for {other}");
            return;
        }
    };
    let cpu = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    };
    let target = format!("{os}-{cpu}");
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-abi-layout");
    std::fs::create_dir_all(&dir).unwrap();
    let compiler = abi_layout::host_c_compiler();
    if Command::new(&compiler).arg("--version").output().is_err() {
        // CI hosts always have one; a missing compiler there must not pass silently.
        assert!(
            std::env::var_os("CI").is_none(),
            "no C compiler `{compiler}`"
        );
        eprintln!("skipping: no C compiler `{compiler}`");
        return;
    }
    let mismatches = abi_layout::check_host(JAIC, &abi_manifest(&target), &target, &dir)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        mismatches.is_empty(),
        "{} stdlib declarations differ from the C headers:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}

/// The same check for targets this host cannot run, from `JAIC_ABI_CROSS` (`;`-separated
/// `os-cpu[:clang args]`, e.g. `macos-x64;windows-x64`); clang must have that target's headers.
/// Does nothing when the variable is unset. See docs/stdlib/native-bindings.md.
#[test]
fn stdlib_c_abi_matches_cross_target_headers() {
    let Ok(specs) = std::env::var("JAIC_ABI_CROSS") else {
        return;
    };
    let clang = std::env::var("JAIC_ABI_CLANG").unwrap_or_else(|_| "clang".into());
    let mut failures = Vec::new();
    for spec in specs.split(';').filter(|s| !s.trim().is_empty()) {
        let target = abi_layout::cross_target(spec.trim()).unwrap_or_else(|e| panic!("{e}"));
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("native-abi-cross")
            .join(&target.name);
        std::fs::create_dir_all(&dir).unwrap();
        match abi_layout::check_cross(JAIC, &clang, &abi_manifest(&target.name), &target, &dir) {
            Ok(mismatches) => failures.extend(mismatches),
            Err(e) => failures.push(format!("[{}] {e}", target.name)),
        }
    }
    assert!(
        failures.is_empty(),
        "{} problems:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// C structs by value across the C ABI: foreign calls from the interpreter and from native code, and
/// native `#c_call` definitions called back from C. Skipped when no C compiler is installed.
#[test]
fn c_structs_by_value() {
    let fixture = repo_root().join("tests/native/c-structs-by-value");
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-c-structs");
    std::fs::create_dir_all(&dir).unwrap();
    for name in [
        "structs.c",
        "types.jai",
        "foreign_calls.jai",
        "callbacks.jai",
    ] {
        std::fs::copy(fixture.join(name), dir.join(name)).unwrap();
    }
    let lib = if cfg!(target_os = "macos") {
        "libstructs.dylib"
    } else {
        "libstructs.so"
    };
    // Windows: a static `libstructs.lib` from Clang for the native build; the interpreter is
    // checked against a DLL below.
    let mut steps = vec![Command::new(if cfg!(windows) {
        "clang"
    } else {
        "cc"
    })];
    if cfg!(windows) {
        steps[0].args(["-c", "structs.c", "-o", "structs.o"]);
        let mut archive = Command::new("llvm-ar");
        archive.args(["rcs", "libstructs.lib", "structs.o"]);
        steps.push(archive);
    } else {
        steps[0].args(["-shared", "-fPIC", "-o", lib, "structs.c"]);
    }
    if cfg!(target_os = "macos") {
        // Found through the executable's rpath rather than relative to the working directory.
        steps[0].arg("-Wl,-install_name,@rpath/libstructs.dylib");
    }
    for step in &mut steps {
        let Ok(output) = step.current_dir(&dir).output() else {
            eprintln!("skipping: no C compiler");
            return;
        };
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let calls = "{11, 22} {2, 4, 6} {5, 6, 7, 8} 10 {-7, 9} {99, 2.5} {11, 22, 33}\n832\n8940414\n";
    let callbacks = "{111, 47} {10, 20, 30, 40} {8, 4}\n832\n{12, 10.25} {7.5, 5.25}\n8940414\n";
    let run_interp = |name: &str| {
        let output = Command::new(JAIC)
            .args(["run", &format!("{name}.jai")])
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).into_owned()
    };
    if !cfg!(windows) {
        assert_eq!(run_interp("foreign_calls"), calls);
        assert_eq!(run_interp("callbacks"), callbacks);
    }
    let run_native = |name: &str| {
        let output = build_and_run(&dir.join(format!("{name}.jai")), &dir, name).unwrap();
        String::from_utf8_lossy(&output.stdout).into_owned()
    };
    assert_eq!(run_native("foreign_calls"), calls);
    assert_eq!(run_native("callbacks"), callbacks);
    // Windows: the interpreter loads the fixture as `libstructs.dll` (built after the native
    // runs, since its import library replaces the static `libstructs.lib`). An MSVC DLL exports
    // only what it is told to, so every function defined in `structs.c` is named. On x64 the
    // callbacks go through the Microsoft x64 thunks (`callbacks/win64.rs`).
    if cfg!(windows) {
        let source = std::fs::read_to_string(dir.join("structs.c")).unwrap();
        let mut link = Command::new("clang");
        link.args(["-shared", "structs.c", "-o", "libstructs.dll"]);
        for line in source.lines() {
            let starts_definition = line.chars().next().is_some_and(|c| c.is_ascii_alphabetic());
            if !starts_definition || line.starts_with("typedef") {
                continue;
            }
            let Some(head) = line.split('(').next() else {
                continue;
            };
            if let Some(name) = head.split_whitespace().last() {
                link.arg(format!("-Wl,/EXPORT:{}", name.trim_start_matches('*')));
            }
        }
        let output = link.current_dir(&dir).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(run_interp("foreign_calls"), calls);
        assert_eq!(run_interp("callbacks"), callbacks);
    }
}

/// `#c_call` procedures C reads from memory instead of its arguments (a struct field, a global, a
/// `qsort` comparator in a struct), called by C on the interpreter's thread and on threads C
/// starts, and called back through the stored pointer by Jai code. `jaic run` gives such
/// procedures native thunk addresses (docs/compiler/interpreter.md). Skipped when no C compiler
/// is installed.
#[test]
fn c_call_procedures_stored_in_memory() {
    let fixture = repo_root().join("tests/native/c-call-stored");
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-c-call-stored");
    std::fs::create_dir_all(&dir).unwrap();
    for name in ["stored.c", "stored.jai"] {
        std::fs::copy(fixture.join(name), dir.join(name)).unwrap();
    }
    let compile = |args: &[&str]| {
        let mut cc = Command::new(if cfg!(windows) {
            "clang"
        } else {
            "cc"
        });
        cc.args(args).current_dir(&dir);
        cc.output().ok().map(|output| {
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        })
    };
    let built = if cfg!(target_os = "macos") {
        compile(&[
            "-shared",
            "-o",
            "libstored.dylib",
            "stored.c",
            "-Wl,-install_name,@rpath/libstored.dylib",
        ])
    } else if cfg!(windows) {
        // One DLL (and its import library) serves both the interpreter and the native build.
        compile(&["-shared", "stored.c", "-o", "libstored.dll"])
    } else {
        compile(&[
            "-shared",
            "-fPIC",
            "-pthread",
            "-o",
            "libstored.so",
            "stored.c",
        ])
    };
    if built.is_none() {
        // CI hosts always have one; a missing compiler there must not pass silently.
        assert!(std::env::var_os("CI").is_none(), "no C compiler");
        eprintln!("skipping: no C compiler");
        return;
    }
    let expected = "115 1006 -9\n21 -4 true true\n[9, 7, 5, 3, 1]\n133\n7 1018 8\n";
    let output = Command::new(JAIC)
        .args(["run", "stored.jai"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
    let output = build_and_run(&dir.join("stored.jai"), &dir, "stored").unwrap();
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
}

/// A `#c_call` procedure that a thread C started calls may block on Jai threads under `jaic run`
/// (a mutex, a condition variable, a join, a sleep), callbacks nest in C calls on every kind of
/// thread, and a Jai thread keeps running while another one waits in a long C call; the output
/// matches the native build (docs/compiler/interpreter-threads.md). Deadlocks among Jai threads,
/// and between a C thread's callback and a Jai thread, are reported. Skipped when no C compiler
/// is installed.
#[test]
fn c_thread_callbacks_block_on_jai_threads() {
    let fixture = repo_root().join("tests/native/c-callback-threads");
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-c-callback-threads");
    std::fs::create_dir_all(&dir).unwrap();
    for name in ["blocking.c", "blocking.jai", "deadlock.jai"] {
        std::fs::copy(fixture.join(name), dir.join(name)).unwrap();
    }
    let compile = |args: &[&str]| {
        let mut cc = Command::new(if cfg!(windows) {
            "clang"
        } else {
            "cc"
        });
        cc.args(args).current_dir(&dir);
        cc.output().ok().map(|output| {
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        })
    };
    let built = if cfg!(target_os = "macos") {
        compile(&[
            "-shared",
            "-o",
            "libblocking.dylib",
            "blocking.c",
            "-Wl,-install_name,@rpath/libblocking.dylib",
        ])
    } else if cfg!(windows) {
        compile(&["-shared", "blocking.c", "-o", "libblocking.dll"])
    } else {
        compile(&[
            "-shared",
            "-fPIC",
            "-pthread",
            "-o",
            "libblocking.so",
            "blocking.c",
        ])
    };
    if built.is_none() {
        eprintln!("skipping: no C compiler");
        return;
    }
    let expected = "mutex: 42\ncondition: 42\njoin: 42\nsleep: 10\nnested: 61\n\
                    nested on a thread of C: 61\nnested on a Jai thread: 31\n\
                    progress while a thread waits in C: 5000050000 1\n";
    let output = Command::new(JAIC)
        .args(["run", "blocking.jai"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).replace('\r', ""),
        expected
    );
    let output = build_and_run(&dir.join("blocking.jai"), &dir, "blocking").unwrap();
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).replace('\r', ""),
        expected
    );
    // Only a thunk C may hold delays the report by the scheduler's grace period: the thunk
    // `Thread` starts its threads through is run by the scheduler, not C. The program says when
    // it starts (on stderr), so the compile time, long and noisy on a loaded host, is not part
    // of the wait that is measured.
    let grace = jaic::interp::DEADLOCK_GRACE;
    let report = |args: &[&str]| {
        use std::io::{BufRead, BufReader, Read};
        let mut child = Command::new(JAIC)
            .args(args)
            .current_dir(&dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let out = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            stdout.read_to_end(&mut bytes).ok();
            bytes
        });
        let mut stderr = BufReader::new(child.stderr.take().unwrap());
        let mut started = None;
        let mut text = String::new();
        let mut line = String::new();
        while stderr.read_line(&mut line).unwrap() > 0 {
            if line.trim_end() == "started" && started.is_none() {
                started = Some(std::time::Instant::now());
            } else {
                text.push_str(&line);
            }
            line.clear();
        }
        let status = child.wait().unwrap();
        let took = started
            .unwrap_or_else(|| panic!("{args:?} never started: {text}"))
            .elapsed();
        assert!(!status.success(), "{args:?} did not fail");
        assert!(
            text.contains("deadlock: every thread is blocked"),
            "{args:?}: {text}"
        );
        assert!(out.join().unwrap().is_empty(), "{args:?}");
        took
    };
    let jai_only = report(&["run", "deadlock.jai"]);
    let callback = report(&["run", "deadlock.jai", "--", "callback"]);
    assert!(callback >= grace, "the callback case took {callback:?}");
    assert!(
        jai_only < grace,
        "the Jai-only case took {jai_only:?}, the callback case {callback:?}"
    );
}

/// The interpreter's crash report on Windows (its vectored exception handler; see
/// `crash_in_native_code_names_the_foreign_call` in diagnostics.rs for Unix). It is here because
/// this suite is the one that runs on native Windows hosts. A crash on a thread C started is
/// not blamed on the foreign call the interpreter is making meanwhile.
#[cfg(windows)]
#[test]
fn crash_in_native_code_is_reported_on_windows() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-crash-windows");
    std::fs::create_dir_all(&dir).unwrap();
    let run = |name: &str, source: &str| {
        std::fs::write(dir.join(name), source).unwrap();
        Command::new(JAIC)
            .args(["run", name])
            .current_dir(&dir)
            .output()
            .unwrap()
    };
    let output = run(
        "crash.jai",
        "crt :: #system_library \"msvcrt\";\nstrlen :: (s: *u8) -> u64 #foreign crt;\nmeasure :: (p: *u8) -> u64 {\n    return strlen(p);\n}\nmain :: () {\n    n := measure(cast(*u8) 16);\n}\n",
    );
    let err = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(121), "{err}");
    assert!(
        err.contains("crash.jai:4:5: error: native code crashed (access violation at address 0x10) while calling foreign procedure `strlen`"),
        "{err}"
    );
    assert!(err.contains("    `measure` at crash.jai:4"), "{err}");

    let output = run(
        "thread.jai",
        "kernel32 :: #system_library \"kernel32\";\nCreateThread :: (attributes: *void, stack: u64, start: *void, parameter: *void, flags: u32, id: *u32) -> *void #foreign kernel32;\nWaitForSingleObject :: (handle: *void, milliseconds: u32) -> u32 #foreign kernel32;\nmain :: () {\n    thread := CreateThread(null, 0, cast(*void) 16, null, 0, null);\n    WaitForSingleObject(thread, 0xffff_ffff);\n}\n",
    );
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{err}");
    assert_ne!(output.status.code(), Some(121), "{err}");
    assert!(!err.contains("native code crashed"), "{err}");
}

/// `-sanitize address` reports a use after free with the Jai source line, `-sanitize undefined`
/// an out-of-bounds stack access, and a correct program runs cleanly under both. Not supported
/// on Windows.
#[test]
fn sanitized_builds_report_memory_errors() {
    if cfg!(windows) {
        return;
    }
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-sanitizers");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("errors.jai"),
        "#import \"Basic\";\n\
         main :: () {\n\
             args := get_command_line_arguments();\n\
             a := NewArray(4, int);\n\
             a[1] = 5;\n\
             print(\"%\\n\", a[1]);\n\
             if args.count == 2 && args[1] == \"heap\" {\n\
                 array_free(a);\n\
                 print(\"%\\n\", a[2]);\n\
             }\n\
             local: [4] int;\n\
             p := local.data;\n\
             if args.count == 2 && args[1] == \"stack\" p[args.count + 2] = 1;\n\
             print(\"%\\n\", local[0]);\n\
         }\n",
    )
    .unwrap();
    let build = |name: &str, sanitize: &str| {
        let exe = exe_path(&dir, name);
        let output = Command::new(JAIC)
            .args(["build", "errors.jai", "-sanitize", sanitize, "-o"])
            .arg(&exe)
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        exe
    };
    let run = |exe: &Path, arg: &str| {
        Command::new(exe)
            .arg(arg)
            .env("ASAN_OPTIONS", "detect_leaks=0")
            .output()
            .unwrap()
    };
    let both = build("both", "address,undefined");
    let clean = run(&both, "none");
    assert_eq!(String::from_utf8_lossy(&clean.stdout), "5\n0\n");
    assert!(clean.status.success());

    let asan = build("asan", "address");
    let heap = run(&asan, "heap");
    let report = String::from_utf8_lossy(&heap.stderr);
    assert!(!heap.status.success());
    assert!(
        report.contains("AddressSanitizer: heap-use-after-free") && report.contains("errors.jai:9"),
        "{report}"
    );

    let ubsan = build("ubsan", "undefined");
    let stack = run(&ubsan, "stack");
    let report = String::from_utf8_lossy(&stack.stderr);
    assert!(!stack.status.success());
    assert!(
        report.contains("errors.jai:13") && report.contains("runtime error: access out of bounds"),
        "{report}"
    );

    let cross = Command::new(JAIC)
        .args([
            "build",
            "errors.jai",
            "-sanitize",
            "address",
            "-os",
            "windows",
        ])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(!cross.status.success());
    assert!(String::from_utf8_lossy(&cross.stderr).contains("cross builds"));
}

/// `#cpp_return_type_is_non_pod`: a C++ class with a copy constructor or destructor comes back
/// through the hidden result pointer even when it would fit in registers. Compiled code used to
/// expect it in registers, so the callee wrote through whatever the result register held (found
/// by the sanitizer sweep). Skipped without a C++ compiler; not run on Windows.
#[test]
fn cpp_non_pod_results_use_the_hidden_pointer() {
    if cfg!(windows) {
        return;
    }
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-cpp-non-pod");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("counter.cpp"),
        "struct Counter {\n\
             int value;\n\
             Counter(int v) : value(v) {}\n\
             Counter(const Counter &o) : value(o.value) {}\n\
             ~Counter() {}\n\
         };\n\
         extern \"C\" Counter make_counter(int v) { return Counter(v * 2 + 1); }\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("non_pod.jai"),
        "#import \"Basic\";\n\
         counter :: #library \"libcounter\";\n\
         Counter :: struct { value: s32; }\n\
         make_counter :: (v: s32) -> Counter #foreign counter #cpp_return_type_is_non_pod;\n\
         main :: () { c := make_counter(20); print(\"%\\n\", c.value); }\n",
    )
    .unwrap();
    let lib = if cfg!(target_os = "macos") {
        "libcounter.dylib"
    } else {
        "libcounter.so"
    };
    let mut compile = Command::new("c++");
    compile.args([
        "-shared",
        "-fPIC",
        "-Wno-return-type-c-linkage",
        "-o",
        lib,
        "counter.cpp",
    ]);
    if cfg!(target_os = "macos") {
        compile.arg("-Wl,-install_name,@rpath/libcounter.dylib");
    }
    let Ok(output) = compile.current_dir(&dir).output() else {
        eprintln!("skipping: no C++ compiler");
        return;
    };
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let interp = Command::new(JAIC)
        .args(["run", "non_pod.jai"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&interp.stdout), "41\n");
    let native = build_and_run(&dir.join("non_pod.jai"), &dir, "non_pod").unwrap();
    assert_eq!(
        String::from_utf8_lossy(&native.stdout),
        "41\n",
        "{}",
        String::from_utf8_lossy(&native.stderr)
    );
}

/// C `long double` (the `Long_Double` extension) across the C ABI: arguments, results and struct
/// members through `#foreign` in the interpreter and natively. On Apple arm64 hosts that can run
/// x86-64 code (Rosetta), the fixture is also built for x86_64-apple-darwin, where long double is
/// the 80-bit x87 format, and the interpreter's soft-float arithmetic is compared bit for bit with
/// the hardware's. Skipped when no C compiler is installed; not run on Windows.
// rules: ext.2 ext.16
#[test]
fn c_long_double() {
    if cfg!(windows) {
        return;
    }
    let fixture = repo_root().join("tests/native/c-long-double");
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-c-long-double");
    std::fs::create_dir_all(&dir).unwrap();
    for name in [
        "longdouble.c",
        "longdouble.h",
        "types.jai",
        "foreign_calls.jai",
        "callbacks.jai",
        "precision.jai",
    ] {
        std::fs::copy(fixture.join(name), dir.join(name)).unwrap();
    }
    let lib = if cfg!(target_os = "macos") {
        "liblongdouble.dylib"
    } else {
        "liblongdouble.so"
    };
    let compile = |extra: &[&str]| {
        let mut cc = Command::new("cc");
        cc.args(extra)
            .args(["-shared", "-fPIC", "-o", lib, "longdouble.c"]);
        if cfg!(target_os = "macos") {
            cc.arg("-Wl,-install_name,@rpath/liblongdouble.dylib");
        }
        cc.current_dir(&dir).output()
    };
    let Ok(output) = compile(&[]) else {
        eprintln!("skipping: no C compiler");
        return;
    };
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let calls = "third 0.333333 wide true\nadd true mix 10.5\nto_double true\nbox 6 tagged 6 true pair true 1\nspill 46.25\n";
    let callbacks = "apply 7.5 true\nbox_apply 4.5\n";
    let jaic = |args: &[&str]| {
        let output = Command::new(JAIC)
            .args(args)
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).into_owned()
    };
    assert_eq!(jaic(&["run", "foreign_calls.jai"]), calls);
    // C calling a Jai procedure that takes a wide long double needs a native build; where long
    // double is float64 the interpreter's callbacks work as usual.
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        assert_eq!(jaic(&["run", "callbacks.jai"]), callbacks);
    }
    let run_native = |name: &str| {
        let output = build_and_run(&dir.join(format!("{name}.jai")), &dir, name).unwrap();
        String::from_utf8_lossy(&output.stdout).into_owned()
    };
    assert_eq!(run_native("foreign_calls"), calls);
    assert_eq!(run_native("callbacks"), callbacks);

    // x86-64 under Rosetta: hardware x87 against the interpreter's soft-float.
    let rosetta = cfg!(all(target_os = "macos", target_arch = "aarch64"))
        && Command::new("arch")
            .args(["-x86_64", "/usr/bin/true"])
            .status()
            .is_ok_and(|s| s.success());
    if !rosetta {
        return;
    }
    let output = compile(&["-arch", "x86_64"]).unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    // The copied Bindings_Generator test imports the stdlib tests' own modules.
    let test_modules = repo_root().join("tests/stdlib/modules");
    let x86 = |name: &str| {
        jaic(&[
            "build",
            &format!("{name}.jai"),
            "-target",
            "x86_64-apple-darwin",
            "-import_dir",
            test_modules.to_str().unwrap(),
            "-o",
            name,
        ]);
        let output = Command::new("arch")
            .args(["-x86_64", &format!("./{name}")])
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).into_owned()
    };
    assert_eq!(x86("foreign_calls"), calls);
    assert_eq!(x86("callbacks"), callbacks);
    let soft = jaic(&["run", "precision.jai", "-target", "x86_64-apple-darwin"]);
    let hard = x86("precision");
    assert!(soft.len() > 10_000, "{soft}");
    assert!(soft == hard, "soft-float x87 differs from the hardware");

    // Bindings_Generator output for the x87 target, called from an x86-64 build.
    let bindings = repo_root().join("tests/stdlib/bindings-generator-long-double.jai");
    let source = std::fs::read_to_string(&bindings).unwrap().replace(
        "#filepath",
        &format!("\"{}\"", repo_root().join("tests/stdlib").display()),
    );
    std::fs::write(dir.join("bindings.jai"), source).unwrap();
    assert_eq!(x86("bindings"), "ok\n");
}

#[test]
fn bindings_generator_long_double() {
    native_bindings_generator_test("bindings-generator-long-double");
}

/// C variadic foreign calls in a native build (Apple arm64 passes variadic arguments on the stack).
#[test]
fn c_variadic_calls() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-varargs");
    std::fs::create_dir_all(&dir).unwrap();
    let source = repo_root().join("tests/stdlib/c-variadic-foreign-calls.jai");
    let output = build_and_run(&source, &dir, "varargs").unwrap();
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "ok\n",
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// A program that imports the Bindings_Generator module builds natively (its libclang bridge
/// primitives are compile-time only and get trapping stubs), and the bindings it generated at
/// compile time work in the native binary.
fn native_bindings_generator_test(name: &str) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("native-{name}"));
    std::fs::create_dir_all(&dir).unwrap();
    let source = repo_root().join(format!("tests/stdlib/{name}.jai"));
    let output = build_and_run(&source, &dir, name).unwrap();
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "ok\n",
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn bindings_generator_bitfields() {
    native_bindings_generator_test("bindings-generator-bitfields");
}

#[test]
fn bindings_generator_cpp_raw() {
    native_bindings_generator_test("bindings-generator-cpp-raw");
}

#[test]
fn bindings_generator_checks() {
    native_bindings_generator_test("bindings-generator-checks");
}

#[test]
fn bindings_generator_objc() {
    native_bindings_generator_test("bindings-generator-objc");
}

#[test]
fn bindings_generator_objc_generics() {
    native_bindings_generator_test("bindings-generator-objc-generics");
}

#[test]
fn bindings_generator_objc_ivars() {
    native_bindings_generator_test("bindings-generator-objc-ivars");
}

#[test]
fn bindings_generator_objc_blocks() {
    native_bindings_generator_test("bindings-generator-objc-blocks");
}

#[test]
fn bindings_generator_bitfields_msvc() {
    native_bindings_generator_test("bindings-generator-bitfields-msvc");
}

#[test]
fn bindings_generator_cpp_raw_multi() {
    native_bindings_generator_test("bindings-generator-cpp-raw-multi");
}

#[test]
fn bindings_generator_objc_stret() {
    native_bindings_generator_test("bindings-generator-objc-stret");
}

#[test]
fn bindings_generator_parity() {
    native_bindings_generator_test("bindings-generator-parity");
}

#[test]
fn bindings_generator_cpp_virtual_bases() {
    native_bindings_generator_test("bindings-generator-cpp-virtual-bases");
}

/// Arithmetic overflow checks in a native build: `Build_Options.arithmetic_overflow_check` on a
/// workspace that writes an executable, and `#no_aoc` switching them off again.
#[test]
fn arithmetic_overflow_checks() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-overflow");
    std::fs::create_dir_all(&dir).unwrap();
    // (name, mode, body of main, run succeeds, stdout, text stderr must contain)
    let cases: [(&str, &str, &str, bool, &str, &str); 8] = [
        (
            "off",
            "OFF",
            "x: u8 = 250; y: u8 = 10; print(\"%\\n\", x + y);",
            true,
            "4\n",
            "",
        ),
        (
            "nonfatal",
            "NONFATAL",
            "x: u8 = 250; y: u8 = 10; print(\"%\\n\", x + y);",
            true,
            "4\n",
            "arithmetic overflow computing 250 + 10 as u8",
        ),
        (
            "fatal-add",
            "FATAL",
            "x: u8 = 250; y: u8 = 10; print(\"%\\n\", x + y);",
            false,
            "",
            "arithmetic overflow computing 250 + 10 as u8",
        ),
        (
            "fatal-sub",
            "FATAL",
            "x: s64 = -0x7fff_ffff_ffff_ffff; y: s64 = 5; print(\"%\\n\", x - y);",
            false,
            "",
            "as s64",
        ),
        (
            "fatal-mul",
            "FATAL",
            "x: u32 = 70000; print(\"%\\n\", x * x);",
            false,
            "",
            "70000 * 70000 as u32",
        ),
        (
            "in-range",
            "FATAL",
            "x: s16 = 32000; y: s16 = 767; a: u64 = 3; print(\"% %\\n\", x + y, a - 3);",
            true,
            "32767 0\n",
            "",
        ),
        (
            "no-aoc-block",
            "FATAL",
            "x: u8 = 250; y: u8 = 10; z: u8; #no_aoc { z = x + y; } print(\"%\\n\", z);",
            true,
            "4\n",
            "",
        ),
        (
            "no-aoc-loop",
            "FATAL",
            "x: u8 = 250; for 1..2 #no_aoc { x += 10; } print(\"%\\n\", x);",
            true,
            "14\n",
            "",
        ),
    ];
    for (name, mode, body, ok, stdout, stderr) in cases {
        let meta = dir.join(format!("{name}.jai"));
        std::fs::write(
            &meta,
            format!(
                r##"#import "Basic";
#import "Compiler";

SOURCE :: #string END
#import "Basic";
main :: () {{
    {body}
}}
END

#run {{
    set_build_options_dc(.{{do_output = false}});
    w := compiler_create_workspace("prog");
    options := get_build_options(w);
    options.output_type = .EXECUTABLE;
    options.output_executable_name = "{name}-prog";
    options.output_path = ".";
    options.arithmetic_overflow_check = .{mode};
    set_build_options(options, w);
    add_build_string(SOURCE, w);
}}
"##
            ),
        )
        .unwrap();
        let build = Command::new(JAIC)
            .arg("build")
            .arg(&meta)
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(
            build.status.success(),
            "{name}: build failed: {}",
            String::from_utf8_lossy(&build.stderr)
        );
        let run = Command::new(exe_path(&dir, &format!("{name}-prog")))
            .output()
            .unwrap();
        let err = String::from_utf8_lossy(&run.stderr);
        assert_eq!(
            run.status.success(),
            ok,
            "{name}: status {:?}\n{err}",
            run.status
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), stdout, "{name}");
        assert!(err.contains(stderr), "{name}: stderr was {err:?}");
        if stderr.is_empty() {
            assert!(
                !err.contains("overflow"),
                "{name}: unexpected report {err:?}"
            );
        }
    }
}

/// A failed check in a built executable says what failed and where (through Runtime_Support's
/// `runtime_support_check_failed`), with the interpreter's wording, before it stops.
// rules: ptr.18 cast.33 flow.27
#[test]
fn failed_checks_say_what_and_where() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-checks");
    std::fs::create_dir_all(&dir).unwrap();
    // (name, body of main, the report's line, its message)
    let cases: [(&str, &str, u32, &str); 6] = [
        (
            "bounds",
            "a: [3] int;\n    i := 5 + a[0];\n    print(\"%\\n\", a[i]);",
            7,
            "array bounds check failed: index 5 is outside an array of 3 elements",
        ),
        (
            "cast",
            "w := 300 + get_command_line_arguments().count;\n    c := cast(u8) w;\n    print(\"%\\n\", c);",
            6,
            "cast of 301 to `u8` overflows",
        ),
        (
            "switch",
            "c := cast(Color) (6 + get_command_line_arguments().count);\n    if #complete c == {\n        case .RED; print(\"red\\n\");\n        case .GREEN; print(\"green\\n\");\n    }",
            6,
            "no case of the `#complete` switch matches its value, 7",
        ),
        (
            "divide",
            "z := get_command_line_arguments().count - 1;\n    print(\"%\\n\", 7 / z);",
            6,
            "integer division by zero",
        ),
        (
            "null_read",
            "p: *int;\n    v := p.*;\n    print(\"%\\n\", v);",
            6,
            "null pointer dereference: read through a null pointer",
        ),
        (
            "null_member",
            "p: *Pair;\n    v := p.b;\n    print(\"%\\n\", v);",
            6,
            "null pointer dereference: read at address 0x8, just past null (a member of a null struct pointer?)",
        ),
    ];
    for (name, body, line, message) in cases {
        let source = dir.join(format!("{name}.jai"));
        std::fs::write(
            &source,
            format!("#import \"Basic\";\nColor :: enum {{ RED; GREEN; }}\nPair :: struct {{ a: int; b: int; }}\nmain :: () {{\n    {body}\n}}\n"),
        )
        .unwrap();
        let output = build_and_run(&source, &dir, name).unwrap();
        let err = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "{name}: ran to the end\n{err}");
        assert!(
            err.contains(&format!("{name}.jai:{line}: error: {message}\n")),
            "{name}: stderr was {err:?}"
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), "", "{name}");
    }

    // An `#asm` divide fault is reported even when nothing else in the program has a check
    // (no Basic, no indexing), so it is the only code that needs the reporting procedure.
    let source = dir.join("asm_divide.jai");
    std::fs::write(
        &source,
        "divisor: u64;\nmain :: () {\n    hi: u64 = 0;\n    lo: u64 = 7;\n    d := divisor;\n    #asm { div hi, lo, d; }\n}\n",
    )
    .unwrap();
    let output = build_and_run(&source, &dir, "asm_divide").unwrap();
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "asm_divide: ran to the end\n{err}"
    );
    assert!(
        err.contains("asm_divide.jai:6: error: #asm division fault: the divisor is zero or the quotient does not fit\n"),
        "asm_divide: stderr was {err:?}"
    );

    // .NONFATAL reports a warning and goes on with the low bits.
    let meta = dir.join("nonfatal.jai");
    std::fs::write(
        &meta,
        r##"#import "Compiler";

SOURCE :: #string END
#import "Basic";
main :: () {
    w := 300 + get_command_line_arguments().count;
    print("%\n", cast(u8) w);
}
END

#run {
    set_build_options_dc(.{do_output = false});
    w := compiler_create_workspace("prog");
    options := get_build_options(w);
    options.output_type = .EXECUTABLE;
    options.output_executable_name = "nonfatal-prog";
    options.output_path = ".";
    options.cast_bounds_check = .NONFATAL;
    set_build_options(options, w);
    add_build_string(SOURCE, w);
}
"##,
    )
    .unwrap();
    let build = Command::new(JAIC)
        .arg("build")
        .arg(&meta)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "nonfatal: build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    let run = Command::new(exe_path(&dir, "nonfatal-prog"))
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&run.stderr);
    assert!(run.status.success(), "nonfatal: {err}");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "45\n");
    assert!(
        err.contains(":4: warning: cast of 301 to `u8` overflows\n"),
        "nonfatal: stderr was {err:?}"
    );
}

/// Window programs using Simp's automatic GL context creation type-check for every desktop
/// OS (the GLX/WGL paths cannot run here, but they must keep compiling).
#[test]
fn simp_window_program_checks_on_desktop_oses() {
    let source = repo_root().join("tests/stdlib/simp-window-program.jai");
    for os in ["linux", "windows", "macos"] {
        let output = Command::new(JAIC)
            .arg("check")
            .arg(&source)
            .args(["-os", os])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{os}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// `jaifmt/` builds natively and behaves as documented: `--stdin` formats to stdout,
/// `--check` lists files that would change (exit 1) without writing, a plain run rewrites them,
/// ignore globs from the nearest jaifmt.toml apply, and malformed files are refused (exit 2).
#[test]
fn jaifmt_builds_and_formats() {
    // jaifmt walks directories through the POSIX module, which has no Windows side.
    if cfg!(windows) {
        return;
    }
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-jaifmt");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/skipped")).unwrap();
    let exe = exe_path(&dir, "jaifmt");
    let build = Command::new(JAIC)
        .arg("build")
        .arg(repo_root().join("jaifmt/main.jai"))
        .args(["-O2", "-o"])
        .arg(&exe)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );

    // Built for baseline x86-64, not this machine's CPU: release archives are built on one CI
    // runner and run on others (the 0.4.1 jaifmt used AVX-512 and died with SIGILL elsewhere).
    if cfg!(all(target_os = "linux", target_arch = "x86_64"))
        && let Ok(dump) = Command::new("objdump").arg("-d").arg(&exe).output()
    {
        let dump = String::from_utf8_lossy(&dump.stdout);
        let beyond = dump
            .lines()
            .find(|l| l.contains("%ymm") || l.contains("%zmm") || l.contains("{%k"));
        assert!(beyond.is_none(), "beyond baseline x86-64: {beyond:?}");
    }

    let format_stdin = |input: &[u8]| {
        let mut child = Command::new(&exe)
            .arg("--stdin")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        use std::io::Write;
        // Writes everything, then closes the pipe (as Homebrew's `pipe_output` does).
        child.stdin.take().unwrap().write_all(input).unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{:?}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).into_owned()
    };
    assert_eq!(
        format_stdin(b"f :: ()\n{\n  x:=1;\n}\n"),
        "f :: () {\n    x := 1;\n}\n"
    );
    // Input that ends without a newline.
    assert_eq!(
        format_stdin(b"main::(){x:=1;}"),
        "main :: () {\n    x := 1;\n}\n"
    );

    let messy = "main :: () {\nx:=1;\n}\n";
    let bad_source = "f :: () { x := (1; }\n";
    std::fs::write(
        dir.join("jaifmt.toml"),
        "indent_width = 2\nignore = [\"src/skipped\"]\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/a.jai"), messy).unwrap();
    std::fs::write(dir.join("src/skipped/b.jai"), messy).unwrap();
    std::fs::write(dir.join("src/bad.jai"), bad_source).unwrap();
    let read = |name: &str| std::fs::read_to_string(dir.join(name)).unwrap();
    let fmt = |args: &[&str]| {
        Command::new(&exe)
            .args(args)
            .current_dir(&dir)
            .output()
            .unwrap()
    };

    let check = fmt(&["--check", "src/a.jai"]);
    assert_eq!(check.status.code(), Some(1));
    // Paths below the current directory are shown relative to it.
    assert_eq!(String::from_utf8_lossy(&check.stdout), "src/a.jai:2\n");
    assert_eq!(read("src/a.jai"), messy);

    let rewrite = fmt(&["src"]);
    assert_eq!(rewrite.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&rewrite.stderr);
    assert!(
        stderr
            .contains("src/bad.jai:1:20: error: unbalanced `}`\nhelp: the file was left unchanged"),
        "{stderr}"
    );
    // Command-line mistakes say what is wrong and what to do instead.
    let missing = fmt(&["src/nope.jai"]);
    assert_eq!(missing.status.code(), Some(2));
    assert_eq!(
        String::from_utf8_lossy(&missing.stderr),
        "error: `src/nope.jai` does not exist\nhelp: relative paths start from the current directory\n"
    );
    let unknown = fmt(&["--chek", "src"]);
    assert_eq!(unknown.status.code(), Some(2));
    assert_eq!(
        String::from_utf8_lossy(&unknown.stderr),
        "error: unknown option `--chek`\nhelp: did you mean `--check`?\nusage: jaifmt [OPTIONS] [PATHS]...\nFor more information, run `jaifmt --help`.\n"
    );
    // --help is generated from the option declarations and goes to stdout (exit 0).
    let help = fmt(&["--help"]);
    assert_eq!(help.status.code(), Some(0));
    let help = String::from_utf8_lossy(&help.stdout);
    assert!(
        help.contains("Usage: jaifmt [OPTIONS] [PATHS]..."),
        "{help}"
    );
    assert!(help.contains("--config <FILE>"), "{help}");
    // --stdin and paths exclude each other.
    let both = fmt(&["--stdin", "src"]);
    assert_eq!(both.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&both.stderr)
            .starts_with("error: `<PATHS>` cannot be used with `--stdin`\n")
    );
    assert_eq!(read("src/a.jai"), "main :: () {\n  x := 1;\n}\n");
    assert_eq!(read("src/skipped/b.jai"), messy);
    assert_eq!(read("src/bad.jai"), bad_source);

    std::fs::remove_file(dir.join("src/bad.jai")).unwrap();
    assert_eq!(fmt(&["--check", "src"]).status.code(), Some(0));
}

/// `jaic build jaifmt/build.jai - -o <file>` builds the same jaifmt through the Compiler module,
/// and an unknown metaprogram argument fails the build.
#[test]
fn jaifmt_build_metaprogram() {
    if cfg!(windows) {
        return;
    }
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-jaifmt-metaprogram");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let exe = exe_path(&dir, "jaifmt");
    let build = Command::new(JAIC)
        .arg("build")
        .arg(repo_root().join("jaifmt/build.jai"))
        .args(["-", "-o"])
        .arg(&exe)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let mut child = Command::new(&exe)
        .arg("--stdin")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"f :: ()\n{\n  x:=1;\n}\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "f :: () {\n    x := 1;\n}\n"
    );

    let bad = Command::new(JAIC)
        .arg("build")
        .arg(repo_root().join("jaifmt/build.jai"))
        .args(["-", "bogus"])
        .output()
        .unwrap();
    assert!(!bad.status.success());
    assert!(
        String::from_utf8_lossy(&bad.stderr).contains("unknown argument 'bogus'"),
        "{}",
        String::from_utf8_lossy(&bad.stderr)
    );
}

/// jaifmt formats every Jai file in a copy of the repository's stdlib, tests and tools (each one
/// passes its token-equivalence check), and formatting the result again changes nothing.
#[test]
fn jaifmt_is_idempotent_on_the_repository() {
    if cfg!(windows) {
        return;
    }
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-jaifmt-idempotence");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let exe = exe_path(&dir, "jaifmt");
    let build = Command::new(JAIC)
        .arg("build")
        .arg(repo_root().join("jaifmt/main.jai"))
        .args(["-O2", "-o"])
        .arg(&exe)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );

    fn copy_jai(from: &Path, to: &Path, count: &mut usize) {
        for entry in std::fs::read_dir(from).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap();
            if path.is_dir() {
                // tests/corpus holds deliberately malformed and byte-pinned fixtures.
                if name != "corpus" {
                    copy_jai(&path, &to.join(name), count);
                }
            } else if path.extension().is_some_and(|e| e == "jai") {
                std::fs::create_dir_all(to).unwrap();
                std::fs::copy(&path, to.join(name)).unwrap();
                *count += 1;
            }
        }
    }
    let tree = dir.join("tree");
    let mut count = 0;
    for part in [
        "stdlib",
        "tests",
        "tools",
        "jaifmt",
        "benchmarks",
        "examples",
    ] {
        let source = repo_root().join(part);
        if source.is_dir() {
            copy_jai(&source, &tree.join(part), &mut count);
        }
    }
    assert!(count > 300, "only {count} Jai files copied");
    let run = |args: &[&str]| Command::new(&exe).args(args).arg(&tree).output().unwrap();
    let first = run(&[]);
    assert_eq!(
        first.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let again = run(&["--check"]);
    assert_eq!(
        again.status.code(),
        Some(0),
        "not idempotent:\n{}",
        String::from_utf8_lossy(&again.stdout)
    );
}

/// The Windows runtime test program: natively wherever the tests run, and cross-built with
/// `-os windows` (x64, and `-cpu arm64`) when a MinGW-w64 toolchain for that CPU is installed
/// (the result is checked to be a PE executable for it; CI runs them on Windows, see
/// `tools/windows_cross.py`).
#[test]
fn windows_runtime_program() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-windows-runtime");
    std::fs::create_dir_all(&dir).unwrap();
    let source = repo_root().join("tests/native/windows/runtime.jai");
    let output = build_and_run(&source, &dir, "runtime").unwrap();
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "ok\n",
        "{}: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    if cfg!(windows) {
        // Its threads run on the interpreter's scheduler through the Win32 calls, and its
        // arguments come back through GetCommandLineW as the program's, not jaic's.
        let run = Command::new(JAIC)
            .arg("run")
            .arg(&source)
            .args(["--", "runtime-arg"])
            .output()
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&run.stdout).replace('\r', ""),
            "ok\n",
            "jaic run: {}: {}",
            run.status,
            String::from_utf8_lossy(&run.stderr)
        );
        return;
    }
    // x64 with MinGW-w64 GCC (or llvm-mingw), arm64 with llvm-mingw; each only when installed.
    // The machine field: IMAGE_FILE_MACHINE_AMD64, IMAGE_FILE_MACHINE_ARM64.
    for (cpu, toolchain, machine) in [
        ("x64", "x86_64-w64-mingw32-gcc", 0x8664),
        ("arm64", "aarch64-w64-mingw32-clang", 0xaa64),
    ] {
        if Command::new(toolchain).arg("--version").output().is_err() {
            continue;
        }
        let name = format!("cross-{cpu}");
        let build = Command::new(JAIC)
            .arg("build")
            .arg(&source)
            .args(["-os", "windows", "-cpu", cpu, "-o"])
            .arg(dir.join(&name))
            .output()
            .unwrap();
        assert!(
            build.status.success(),
            "{cpu}: {}",
            String::from_utf8_lossy(&build.stderr)
        );
        let image = std::fs::read(dir.join(format!("{name}.exe"))).unwrap();
        assert_eq!(&image[..2], b"MZ");
        let pe = u32::from_le_bytes(image[0x3c..0x40].try_into().unwrap()) as usize;
        assert_eq!(&image[pe..pe + 4], b"PE\0\0");
        assert_eq!(
            u16::from_le_bytes([image[pe + 4], image[pe + 5]]),
            machine,
            "{cpu}"
        );
    }
}

/// The stdlib's thread tests under `jaic run`: on Windows they exercise the Win32 side of the
/// interpreter's thread scheduler (`CreateThread`, critical sections, condition variables,
/// `WaitForSingleObject`, `Sleep`); elsewhere the pthread side, as the sweep does.
#[test]
fn interpreted_threads() {
    let dir = repo_root().join("tests/stdlib");
    let mut checked = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if !(name.starts_with("threads-") && name.ends_with(".jai")) {
            continue;
        }
        let run = Command::new(JAIC)
            .arg("run")
            .arg(&path)
            .current_dir(&dir)
            .output()
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&run.stdout).replace('\r', ""),
            "ok\n",
            "{name}: {}: {}",
            run.status,
            String::from_utf8_lossy(&run.stderr)
        );
        checked += 1;
    }
    assert!(checked >= 2, "no tests/stdlib/threads-*.jai found");
}

/// MSVC builds keep their CodeView in a PDB next to the executable (`/DEBUG /PDB:`), with line
/// information for the Jai source, and write none with `--no-debug-info`. The line check needs
/// `llvm-pdbutil` (part of the LLVM release CI installs; required when `CI` is set).
#[test]
fn msvc_builds_write_a_pdb() {
    if !cfg!(all(windows, target_env = "msvc")) {
        return;
    }
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-pdb");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("pdb_lines.jai");
    std::fs::write(
        &source,
        "#import \"Basic\";\n\
         twice :: (x: int) -> int {\n\
             return x * 2;\n\
         }\n\
         main :: () {\n\
             print(\"%\\n\", twice(21));\n\
         }\n",
    )
    .unwrap();
    let pdb = dir.join("pdb_lines.pdb");
    let _ = std::fs::remove_file(&pdb);
    let output = build_and_run(&source, &dir, "pdb_lines").unwrap();
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).replace('\r', ""),
        "42\n"
    );
    assert!(pdb.is_file(), "no {} after jaic build", pdb.display());
    match Command::new("llvm-pdbutil")
        .args(["dump", "-l"])
        .arg(&pdb)
        .output()
    {
        Ok(dump) => {
            let text = String::from_utf8_lossy(&dump.stdout);
            assert!(
                dump.status.success(),
                "{}",
                String::from_utf8_lossy(&dump.stderr)
            );
            // The line table of the module names the source and the line of `return x * 2`.
            assert!(
                text.contains("pdb_lines.jai"),
                "no source file in the PDB:\n{text}"
            );
            assert!(
                text.lines().any(|l| l
                    .split_whitespace()
                    .any(|w| w.starts_with("3:") || w == "3")),
                "no line 3 in the PDB:\n{text}"
            );
        }
        // CI installs the LLVM release, which has it.
        Err(e) if std::env::var_os("CI").is_some() => panic!("llvm-pdbutil: {e}"),
        Err(_) => eprintln!("llvm-pdbutil not found: skipping the line information check"),
    }
    let _ = std::fs::remove_file(&pdb);
    let build = Command::new(JAIC)
        .arg("build")
        .arg(&source)
        .arg("--no-debug-info")
        .arg("-o")
        .arg(exe_path(&dir, "pdb_lines"))
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert!(!pdb.exists(), "--no-debug-info still wrote a PDB");
}

/// `jaic build -plug Name` writes the program the plugin's workspace compiled, to `-o` or,
/// without it, next to the source under the source's name.
// rules: plugin.1 plugin.11
#[test]
fn plug_builds_the_plugin_workspace() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-plug");
    let source = common::write_plugin_program(&dir);
    let exe = dir.join("uses_plugin_exe");
    let _ = std::fs::remove_file(&exe);
    let build = Command::new(JAIC)
        .arg("build")
        .arg(&source)
        .args(["-plug", "Echo_Plugin", "-o"])
        .arg(&exe)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&build.stdout),
        "finished, typechecked: true\n"
    );
    let run = Command::new(&exe).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&run.stdout), "42\n");

    let default_exe = dir.join(if cfg!(windows) {
        "uses_plugin.exe"
    } else {
        "uses_plugin"
    });
    let _ = std::fs::remove_file(&default_exe);
    let build = Command::new(JAIC)
        .arg("build")
        .arg(&source)
        .args(["-plug", "Echo_Plugin"])
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let run = Command::new(&default_exe).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&run.stdout), "42\n");
}

/// An unnamed `#library,system,link_always "x";` statement (in a static `#if`, as Tracy's
/// bindings write it) is linked although no foreign procedure names it.
#[cfg(target_os = "macos")]
#[test]
fn unnamed_link_always_library_is_linked() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-link-always");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("link_always.jai");
    std::fs::write(
        &source,
        "#import \"Basic\";\n#if true {\n    #library,system,link_always \"libc++\";\n}\nmain :: () { print(\"ok\\n\"); }\n",
    )
    .unwrap();
    let output = build_and_run(&source, &dir, "link_always").unwrap();
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ok\n");
    let libs = Command::new("otool")
        .arg("-L")
        .arg(dir.join("link_always"))
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&libs.stdout).contains("libc++"));
}

/// A C++ function marked `#cpp_return_type_is_non_pod` returns even a small class (one `int`)
/// through the hidden result pointer, as C++ does for types with constructors. Found by
/// tools/jaic-diff.py: the native build expected it in registers while the callee wrote through x8.
#[test]
fn bindings_generator_cpp_classes() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-bindings-generator-cpp-classes");
    std::fs::create_dir_all(&dir).unwrap();
    let source = repo_root().join("tests/stdlib/bindings-generator-cpp-classes.jai");
    let output = build_and_run(&source, &dir, "bindings-generator-cpp-classes").unwrap();
    assert!(
        output.status.success() && String::from_utf8_lossy(&output.stdout).ends_with("ok\n"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// A `#compiler` primitive called from compiled code says why the program stops.
#[test]
fn compiler_primitive_at_run_time_explains_the_trap() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-compiler-primitive");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("primitive.jai");
    std::fs::write(
        &source,
        "#import \"Basic\";\n#import \"Compiler\";\nmain :: () {\n    print(\"before\\n\");\n    compiler_create_workspace(\"late\");\n    print(\"after\\n\");\n}\n",
    )
    .unwrap();
    let output = build_and_run(&source, &dir, "primitive").unwrap();
    assert!(!output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "before\n");
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("is a compiler primitive; it runs only at compile time"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// `a -= b` in a loop of run-time length survives `-O2`: LLVM 22's runtime unroller recombined
/// the per-copy accumulators of a `sub` recurrence wrongly (llvm/llvm-project#201065, fixed in
/// LLVM 23.1.0), which jaic worked around until it moved to LLVM 23. Reduced from a
/// tools/jaigen.py program.
#[test]
fn optimized_sub_recurrence_matches_the_interpreter() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-sub-recurrence");
    std::fs::create_dir_all(&dir).unwrap();
    let case = corpus_cases()
        .into_iter()
        .find(|c| c.id == "unrolled-sub-reduction")
        .unwrap();
    let exe = exe_path(&dir, "unrolled-sub-reduction");
    let build = Command::new(JAIC)
        .arg("build")
        .arg(&case.source)
        .args(["-O2", "-o"])
        .arg(&exe)
        .current_dir(case.source.parent().unwrap())
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let output = Command::new(&exe).output().unwrap();
    assert!(
        matches(&case, &output),
        "stdout {:?}, expected {:?}",
        String::from_utf8_lossy(&output.stdout),
        case.stdout
    );
}

/// Compiled procedures keep frame records, so a frame-pointer stack walk (macOS libc
/// `backtrace`, behind Debug's `backtrace`) sees the callers. Without them it found no frames and
/// `tests/stdlib/debug-assert-handlers.jai` failed natively (found by tools/jaic-diff.py).
#[test]
fn backtrace_sees_compiled_callers() {
    if !cfg!(unix) {
        return;
    }
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-backtrace");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("backtrace.jai");
    std::fs::write(
        &source,
        "#import \"Basic\";\nDebug :: #import \"Debug\"(USE_GRAPHICS = false);\n\
         inner :: () -> int {\n    frames := Debug.backtrace();\n    n := frames.count;\n    Debug.free_backtrace(frames);\n    return n;\n}\n\
         outer :: () -> int {\n    return inner() + 1;\n}\n\
         main :: () {\n    print(\"%\\n\", ifx outer() > 3 then \"ok\" else \"short\");\n}\n",
    )
    .unwrap();
    for opt in ["-O0", "-O2"] {
        let exe = exe_path(&dir, &format!("backtrace{opt}"));
        let build = Command::new(JAIC)
            .arg("build")
            .arg(&source)
            .args([opt, "-o"])
            .arg(&exe)
            .output()
            .unwrap();
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let output = Command::new(&exe).output().unwrap();
        assert_eq!(String::from_utf8_lossy(&output.stdout), "ok\n", "{opt}");
    }
}

/// `jaic run` writes what a metaprogram's workspace asks for, as `jaic build` does: only the
/// top-level program is interpreted instead of compiled.
// rules: ws.15
#[test]
fn run_writes_workspace_output() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-run-workspace-output");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("meta.jai");
    std::fs::write(
        &source,
        r##"#import "Basic";
#import "Compiler";
#run {
    set_build_options_dc(.{do_output = false});
    w := compiler_create_workspace("target");
    options := get_build_options(w);
    options.output_type = .EXECUTABLE;
    options.output_executable_name = "target-prog";
    options.output_path = ".";
    set_build_options(options, w);
    add_build_string("#import \"Basic\";\nmain :: () { print(\"built\\n\"); }\n", w);
}
"##,
    )
    .unwrap();
    let output = Command::new(JAIC)
        .arg("run")
        .arg(&source)
        .current_dir(&dir)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert!(!stderr.contains("does not write"), "{stderr}");
    let exe = dir.join(if cfg!(windows) {
        "target-prog.exe"
    } else {
        "target-prog"
    });
    let ran = Command::new(&exe).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&ran.stdout), "built\n");
}

/// A metaprogram that builds a workspace with `use_custom_link_command`, with `@LINK@` in place of
/// what it does on READY_FOR_CUSTOM_LINK_COMMAND and `@OPTIONS@` in place of more options.
#[cfg(unix)]
const CUSTOM_LINK: &str = r##"#import "Basic";
#import "Compiler";
#import "Process";

SOURCE :: #string END
#import "Basic";
start :: () {
    print("started at start\n");
    if get_command_line_arguments().count > 1 {
        p: *int;
        p.* = 3;
    }
}
END

#run {
    set_build_options_dc(.{do_output = false});
    w := compiler_create_workspace("prog");
    options := get_build_options(w);
    options.output_executable_name = "linked";
    options.output_path = "out";
    options.intermediate_path = "obj";
    options.use_custom_link_command = true;
    options.entry_point_name = "start";
    options.llvm_options.output_llvm_ir = true;
    @OPTIONS@
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(SOURCE, w);
    while true {
        message := compiler_wait_for_message();
        if message.kind == .PHASE {
            phase := cast(*Message_Phase) message;
            if phase.phase == .READY_FOR_CUSTOM_LINK_COMMAND {
                print("objects %\n", phase.compiler_generated_object_files.count);
                args: [..] string;
                array_add(*args, "cc", "-o", phase.executable_name);
                array_add(*args, ..phase.compiler_generated_object_files);
                array_add(*args, ..phase.system_libraries);
                array_add(*args, ..phase.user_libraries);
                @LINK@
            }
            if phase.phase == .POST_WRITE_EXECUTABLE {
                print("written: failed %, linker exit code %\n", phase.executable_write_failed, phase.linker_exit_code);
            }
        }
        if message.kind == .COMPLETE break;
    }
    compiler_end_intercept(w);
}
"##;

/// With `use_custom_link_command` the metaprogram links the objects jaic wrote (to
/// `intermediate_path`, with the LLVM IR asked for) and says how its link went; a metaprogram
/// that never says, or whose linker fails, fails the build. The program starts in the procedure
/// `entry_point_name` names.
// rules: bo.6 bo.7 bo.8 bo.9 bo.12
#[cfg(unix)]
#[test]
fn custom_link_command() {
    let build = |name: &str, link: &str, options: &str| {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("native-custom-link-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let source = CUSTOM_LINK
            .replace("@LINK@", link)
            .replace("@OPTIONS@", options);
        std::fs::write(dir.join("meta.jai"), source).unwrap();
        let output = Command::new(JAIC)
            .args(["build", "meta.jai"])
            .current_dir(&dir)
            .output()
            .unwrap();
        (dir, output)
    };
    let linked = "result := run_command(..args); compiler_custom_link_command_is_complete(w, result.exit_code);";
    let (dir, output) = build("linked", linked, "");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "objects 1\nwritten: failed false, linker exit code 0\n"
    );
    assert!(dir.join("obj/linked.o").exists() && !dir.join("out/linked.o").exists());
    let ir = std::fs::read_to_string(dir.join("obj/linked.ll")).unwrap();
    // `enable_frame_pointers` (true unless set_optimization says otherwise) keeps every frame.
    assert!(ir.contains("\"frame-pointer\"=\"all\""), "{ir}");
    let ran = Command::new(dir.join("out/linked")).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&ran.stdout), "started at start\n");

    let (_, output) = build("unlinked", "", "");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success()
            && stderr.contains("error: the metaprogram did not link")
            && stderr.contains("help: call `compiler_custom_link_command_is_complete"),
        "{stderr}"
    );

    let (_, output) = build(
        "failed",
        "compiler_custom_link_command_is_complete(w, 3);",
        "",
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success() && stderr.contains("failed with exit code 3"),
        "{stderr}"
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .ends_with("written: failed true, linker exit code 3\n")
    );
}

/// `backtrace_on_crash` decides whether the program installs the crash handler, and
/// `minimum_os_version` is the oldest macOS the program says it runs on.
// rules: bo.10 bo.11
#[cfg(unix)]
#[test]
fn backtrace_on_crash_and_minimum_os_version() {
    for on in [true, false] {
        let name = if on {
            "crash-on"
        } else {
            "crash-off"
        };
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("native-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let options = format!(
            "options.use_custom_link_command = false; options.minimum_os_version = .{{12, 0}}; \
             options.backtrace_on_crash = .{};",
            if on {
                "ON"
            } else {
                "OFF"
            }
        );
        let source = CUSTOM_LINK
            .replace("@OPTIONS@", &options)
            .replace("@LINK@", "");
        std::fs::write(dir.join("meta.jai"), source).unwrap();
        let output = Command::new(JAIC)
            .args(["build", "meta.jai"])
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let exe = dir.join("out/linked");
        let crashed = Command::new(&exe).arg("crash").output().unwrap();
        assert!(!crashed.status.success());
        let stderr = String::from_utf8_lossy(&crashed.stderr);
        assert_eq!(
            stderr.contains("fatal runtime signal"),
            on,
            "{name}: {stderr}"
        );
        if cfg!(target_os = "macos") {
            let load = Command::new("otool").arg("-l").arg(&exe).output().unwrap();
            let load = String::from_utf8_lossy(&load.stdout);
            assert!(load.contains("minos 12.0"), "{load}");
        }
    }
}
