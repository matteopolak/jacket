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

/// Self-checking stdlib tests whose bugs showed only in compiled code; each prints "ok".
#[test]
fn stdlib_tests_run_natively() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-stdlib-tests");
    std::fs::create_dir_all(&dir).unwrap();
    for name in [
        "struct-literal-overrides-default-string",
        "array-literal-view-lifetime",
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
    assert!(stderr.contains("Stack trace:"), "{stderr}");
    assert!(stderr.contains("trace.jai:6: main"), "{stderr}");
    assert!(!output.status.success());
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
    let mut cc = Command::new("cc");
    cc.args(["-shared", "-fPIC", "-o", lib, "structs.c"]);
    if cfg!(target_os = "macos") {
        // Found through the executable's rpath rather than relative to the working directory.
        cc.arg("-Wl,-install_name,@rpath/libstructs.dylib");
    }
    let Ok(cc) = cc.current_dir(&dir).output() else {
        eprintln!("skipping: no C compiler");
        return;
    };
    assert!(
        cc.status.success(),
        "{}",
        String::from_utf8_lossy(&cc.stderr)
    );
    let calls = "{11, 22} {2, 4, 6} {5, 6, 7, 8} 10 {-7, 9} {99, 2.5} {11, 22, 33}\n832\n";
    let callbacks = "{111, 47} {10, 20, 30, 40} {8, 4}\n832\n";
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
    assert_eq!(run_interp("foreign_calls"), calls);
    assert_eq!(run_interp("callbacks"), callbacks);
    let run_native = |name: &str| {
        let output = build_and_run(&dir.join(format!("{name}.jai")), &dir, name).unwrap();
        String::from_utf8_lossy(&output.stdout).into_owned()
    };
    assert_eq!(run_native("foreign_calls"), calls);
    assert_eq!(run_native("callbacks"), callbacks);
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
        let run = Command::new(dir.join(format!("{name}-prog")))
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

/// `tools/jaifmt` builds natively and behaves as documented: `--stdin` formats to stdout,
/// `--check` lists files that would change (exit 1) without writing, a plain run rewrites them,
/// ignore globs from the nearest jaifmt.toml apply, and malformed files are refused (exit 2).
#[test]
fn jaifmt_builds_and_formats() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-jaifmt");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/skipped")).unwrap();
    let exe = dir.join("jaifmt");
    let build = Command::new(JAIC)
        .arg("build")
        .arg(repo_root().join("tools/jaifmt/main.jai"))
        .args(["-O2", "-o"])
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
    assert!(String::from_utf8_lossy(&check.stdout).contains("src/a.jai:2"));
    assert_eq!(read("src/a.jai"), messy);

    let rewrite = fmt(&["src"]);
    assert_eq!(rewrite.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&rewrite.stderr).contains("unbalanced"));
    assert_eq!(read("src/a.jai"), "main :: () {\n  x := 1;\n}\n");
    assert_eq!(read("src/skipped/b.jai"), messy);
    assert_eq!(read("src/bad.jai"), bad_source);

    std::fs::remove_file(dir.join("src/bad.jai")).unwrap();
    assert_eq!(fmt(&["--check", "src"]).status.code(), Some(0));
}
