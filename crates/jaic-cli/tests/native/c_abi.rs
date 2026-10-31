//! C ABI layouts, by-value structs, callbacks and sanitized builds.
use super::{JAIC, abi_layout, build_and_run, exe_path, jaic, read_ir_parts, repo_root};
use std::path::Path;
use std::process::Command;
use test_support::{TempDir, toolchain};

fn abi_manifest(target: &str) -> abi_layout::Manifest {
    let text =
        std::fs::read_to_string(repo_root().join("tests/compiler/abi/manifest.txt")).unwrap();
    abi_layout::parse(&text, target).unwrap_or_else(|e| panic!("{e}"))
}

/// The stdlib's hand-written C declarations (`tests/compiler/abi/manifest.txt`) against the host's own
/// headers: sizes, alignments, field offsets, integer signedness and constants. Every CI host
/// runs this, so a layout copied from another CPU or OS fails on the host it is wrong for.
#[test]
fn stdlib_c_abi_matches_host_headers() {
    let os = match std::env::consts::OS {
        "linux" => "linux",
        "macos" => "macos",
        "windows" => "windows",
        other => panic!(
            "the stdlib ABI manifest has sections for linux, macos and windows only, not {other}"
        ),
    };
    let cpu = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    };
    let target = format!("{os}-{cpu}");
    let dir = TempDir::new("native-abi-layout");
    if toolchain::host_cc().is_none() {
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
/// CI sets it on the macOS legs (`macos-x64`, whose headers come with the SDK). Skipped, with a
/// note, when the variable is unset or empty.
#[test]
fn stdlib_c_abi_matches_cross_target_headers() {
    let specs = jacket_config::env::ABI_CROSS.get().unwrap_or_default();
    if specs.trim().is_empty() {
        toolchain::skip("JAIC_ABI_CROSS names no target to check");
        return;
    }
    let clang = jacket_config::env::ABI_CLANG
        .get()
        .unwrap_or_else(|| "clang".into());
    let mut failures = Vec::new();
    for spec in specs.split(';').filter(|s| !s.trim().is_empty()) {
        let target = abi_layout::cross_target(spec.trim()).unwrap_or_else(|e| panic!("{e}"));
        let dir = TempDir::new(&target.name);
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
    let fixture = repo_root().join("tests/compiler/native/c-structs-by-value");
    let dir = TempDir::new("native-c-structs");
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
    let mut steps = vec![Command::new(
        toolchain::host_cc().expect("a C compiler is installed"),
    )];
    if cfg!(windows) {
        steps[0].args(["-c", "structs.c", "-o", "structs.o"]);
        let mut archive = toolchain::command("llvm-ar");
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
    let calls = "{11, 22} {2, 4, 6} {5, 6, 7, 8} 10 {-7, 9} {99, 2.5} {11, 22, 33} {11, 22}\n832\n8940414\n";
    let callbacks = "{111, 47} {10, 20, 30, 40} {8, 4}\n832\n{12, 10.25} {7.5, 5.25}\n8940414\n";
    let run_interp = |name: &str| {
        let output = jaic()
            .args(["-interp", &format!("{name}.jai")])
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
        let mut link = toolchain::command("clang");
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
/// starts, and called back through the stored pointer by Jai code. `jaic -interp` gives such
/// procedures native thunk addresses (docs/compiler/interpreter.md). Skipped when no C compiler
/// is installed.
#[test]
fn c_call_procedures_stored_in_memory() {
    let fixture = repo_root().join("tests/compiler/native/c-call-stored");
    let dir = TempDir::new("native-c-call-stored");
    for name in ["stored.c", "stored.jai"] {
        std::fs::copy(fixture.join(name), dir.join(name)).unwrap();
    }
    let compile = |args: &[&str]| {
        let mut cc = Command::new(toolchain::host_cc().expect("a C compiler is installed"));
        cc.args(args).current_dir(&dir);
        let output = cc
            .output()
            .expect("native callback regressions require a C compiler");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    if cfg!(target_os = "macos") {
        compile(&[
            "-shared",
            "-o",
            "libstored.dylib",
            "stored.c",
            "-Wl,-install_name,@rpath/libstored.dylib",
        ]);
    } else if cfg!(windows) {
        // One DLL (and its import library) serves both the interpreter and the native build.
        compile(&["-shared", "stored.c", "-o", "libstored.dll"]);
    } else {
        compile(&[
            "-shared",
            "-fPIC",
            "-pthread",
            "-o",
            "libstored.so",
            "stored.c",
        ]);
    }
    let expected = "115 1006 -9\n21 -4 true true\n[9, 7, 5, 3, 1]\n133\n7 1018 8\n";
    let output = jaic()
        .args(["-interp", "stored.jai"])
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

/// A `#c_call` procedure that a thread C started calls may block on Jai threads under `jaic -interp`
/// (a mutex, a condition variable, a join, a sleep), callbacks nest in C calls on every kind of
/// thread, and a Jai thread keeps running while another one waits in a long C call; the output
/// matches the native build (docs/compiler/interpreter-threads.md). Deadlocks among Jai threads,
/// and between a C thread's callback and a Jai thread, are reported. Skipped when no C compiler
/// is installed.
#[test]
fn c_thread_callbacks_block_on_jai_threads() {
    let fixture = repo_root().join("tests/compiler/native/c-callback-threads");
    let dir = TempDir::new("native-c-callback-threads");
    for name in ["blocking.c", "blocking.jai", "deadlock.jai"] {
        std::fs::copy(fixture.join(name), dir.join(name)).unwrap();
    }
    let compile = |args: &[&str]| {
        let mut cc = Command::new(toolchain::host_cc().expect("a C compiler is installed"));
        cc.args(args).current_dir(&dir);
        let output = cc
            .output()
            .expect("native callback regressions require a C compiler");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    if cfg!(target_os = "macos") {
        compile(&[
            "-shared",
            "-o",
            "libblocking.dylib",
            "blocking.c",
            "-Wl,-install_name,@rpath/libblocking.dylib",
        ]);
    } else if cfg!(windows) {
        compile(&["-shared", "blocking.c", "-o", "libblocking.dll"]);
    } else {
        compile(&[
            "-shared",
            "-fPIC",
            "-pthread",
            "-o",
            "libblocking.so",
            "blocking.c",
        ]);
    }
    let expected = "mutex: 42\ncondition: 42\njoin: 42\nsleep: 10\nnested: 61\n\
                    nested on a thread of C: 61\nnested on a Jai thread: 31\n\
                    progress while a thread waits in C: 5000050000 1\n";
    let output = jaic()
        .args(["-interp", "blocking.jai"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "jaic -interp blocking.jai exited with {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
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
        let mut child = jaic()
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
        let stderr = BufReader::new(child.stderr.take().unwrap());
        let errors = std::thread::spawn(move || {
            let mut started = None;
            let mut text = String::new();
            for line in stderr.lines() {
                let line = line.unwrap();
                if line.trim_end() == "started" && started.is_none() {
                    started = Some(std::time::Instant::now());
                } else {
                    text.push_str(&line);
                    text.push('\n');
                }
            }
            (started, text)
        });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        let (status, timed_out) = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break (status, false);
            }
            if std::time::Instant::now() >= deadline {
                child.kill().unwrap();
                break (child.wait().unwrap(), true);
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        };
        let (started, text) = errors.join().unwrap();
        assert!(!timed_out, "{args:?}: deadlock report timed out: {text}");
        let took = started
            .unwrap_or_else(|| panic!("{args:?} never started: {text}"))
            .elapsed();
        assert!(!status.success(), "{args:?} did not fail");
        assert!(
            text.contains("deadlock: every thread is blocked"),
            "{args:?} ({status}): {text}"
        );
        assert!(out.join().unwrap().is_empty(), "{args:?}");
        took
    };
    let jai_only = report(&["-interp", "deadlock.jai"]);
    let callback = report(&["-interp", "deadlock.jai", "--", "callback"]);
    assert!(callback >= grace, "the callback case took {callback:?}");
    assert!(
        jai_only < grace,
        "the Jai-only case took {jai_only:?}, the callback case {callback:?}"
    );
}

/// A `#c_call` procedure called from the top of a stack under `jaic -interp`: its thunk reads only
/// the stack arguments the signature has, so it does not fault on the unmapped page just above
/// (a thread C starts calls its callback from about there). Unix only: the C side switches
/// stacks with `ucontext`.
#[cfg(unix)]
#[test]
fn c_callbacks_from_the_top_of_a_stack() {
    let fixture = repo_root().join("tests/compiler/native/c-callback-threads");
    let dir = TempDir::new("native-c-callback-stack-top");
    for name in ["stack_top.c", "stack_top.jai"] {
        std::fs::copy(fixture.join(name), dir.join(name)).unwrap();
    }
    let library = if cfg!(target_os = "macos") {
        "libstack_top.dylib"
    } else {
        "libstack_top.so"
    };
    let mut cc = toolchain::command("cc");
    cc.args([
        "-shared",
        "-fPIC",
        "-Wno-deprecated-declarations",
        "-o",
        library,
        "stack_top.c",
    ])
    .current_dir(&dir);
    if cfg!(target_os = "macos") {
        cc.arg("-Wl,-install_name,@rpath/libstack_top.dylib");
    }
    let output = cc
        .output()
        .expect("native callback regressions require a C compiler");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = jaic()
        .args(["-interp", "stack_top.jai"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "jaic -interp stack_top.jai exited with {}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "from the stack top: 42\n"
    );
}

/// The interpreter's crash report on Windows (its vectored exception handler; see
/// `crash_in_native_code_names_the_foreign_call` in diagnostics.rs for Unix). It is here because
/// this suite is the one that runs on native Windows hosts. A crash on a thread C started is
/// not blamed on the foreign call the interpreter is making meanwhile.
#[cfg(windows)]
#[test]
fn crash_in_native_code_is_reported_on_windows() {
    let dir = TempDir::new("native-crash-windows");
    let run = |name: &str, source: &str| {
        std::fs::write(dir.join(name), source).unwrap();
        jaic()
            .args(["-interp", name])
            .current_dir(&dir)
            .output()
            .unwrap()
    };
    // A library name the interpreter resolves to the C runtime DLL.
    let runtime = jaic::toolchain::Toolchain::interpreter().runtime_names[0].name;
    let output = run(
        "crash.jai",
        &format!(
            "crt :: #system_library \"{runtime}\";\nstrlen :: (s: *u8) -> u64 #foreign crt;\nmeasure :: (p: *u8) -> u64 {{\n    return strlen(p);\n}}\nmain :: () {{\n    n := measure(cast(*u8) 16);\n}}\n"
        ),
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
    let dir = TempDir::new("native-sanitizers");
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
        let output = jaic()
            .args(["errors.jai", "-sanitize", sanitize, "-exe"])
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

    let cross = jaic()
        .args(["errors.jai", "-sanitize", "address", "-os", "windows"])
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
    let dir = TempDir::new("native-cpp-non-pod");
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
    let interp = jaic()
        .args(["-interp", "non_pod.jai"])
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

/// `#cpp_return_type_is_non_pod` on a `#c_call` procedure C++ calls through a function pointer: the
/// result goes through the hidden pointer (`x8` on `AArch64`) whatever its size, in the interpreter's
/// callback thunks (`AArch64` used to refuse them) and in native builds. Skipped without a C++
/// compiler; not run on Windows.
#[test]
fn cpp_non_pod_callback_results() {
    if cfg!(windows) {
        return;
    }
    let dir = TempDir::new("native-cpp-non-pod-callbacks");
    std::fs::write(
        dir.join("counter.cpp"),
        "struct Counter {\n\
             int value;\n\
             Counter(int v) : value(v) {}\n\
             Counter(const Counter &o) : value(o.value) {}\n\
             ~Counter() {}\n\
         };\n\
         struct Wide {\n\
             long long a, b, c;\n\
             Wide(long long v) : a(v), b(v + 1), c(v + 2) {}\n\
             Wide(const Wide &o) : a(o.a), b(o.b), c(o.c) {}\n\
             ~Wide() {}\n\
         };\n\
         extern \"C\" int run_counter(Counter (*cb)(int), int v) { Counter c = cb(v); return c.value; }\n\
         extern \"C\" long long run_wide(Wide (*cb)(long long), long long v) {\n\
             Wide w = cb(v);\n\
             return w.a * 100 + w.b * 10 + w.c;\n\
         }\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("non_pod_callbacks.jai"),
        "#import \"Basic\";\n\
         counter :: #library \"libcounter\";\n\
         Counter :: struct { value: s32; }\n\
         Wide :: struct { a, b, c: s64; }\n\
         run_counter :: (cb: (v: s32) -> Counter #c_call #cpp_return_type_is_non_pod, v: s32) -> s32 #foreign counter;\n\
         run_wide :: (cb: (v: s64) -> Wide #c_call #cpp_return_type_is_non_pod, v: s64) -> s64 #foreign counter;\n\
         make_counter :: (v: s32) -> Counter #c_call #cpp_return_type_is_non_pod {\n\
             c: Counter;\n\
             c.value = v * 2 + 1;\n\
             return c;\n\
         }\n\
         make_wide :: (v: s64) -> Wide #c_call #cpp_return_type_is_non_pod {\n\
             w: Wide;\n\
             w.a = v;\n\
             w.b = v + 1;\n\
             w.c = v + 2;\n\
             return w;\n\
         }\n\
         main :: () { print(\"% %\\n\", run_counter(make_counter, 20), run_wide(make_wide, 1)); }\n",
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
    let interp = jaic()
        .args(["-interp", "non_pod_callbacks.jai"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&interp.stdout),
        "41 123\n",
        "{}",
        String::from_utf8_lossy(&interp.stderr)
    );
    let native = build_and_run(
        &dir.join("non_pod_callbacks.jai"),
        &dir,
        "non_pod_callbacks",
    )
    .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&native.stdout),
        "41 123\n",
        "{}",
        String::from_utf8_lossy(&native.stderr)
    );
}

/// C `long double` (the `Long_Double` extension) across the C ABI: arguments, results and struct
/// members through `#foreign` in the interpreter and natively. On Apple arm64 hosts that can run
/// x86-64 code (Rosetta), the fixture is also built for x86_64-apple-darwin, where long double is
/// the 80-bit x87 format, and the interpreter's soft-float arithmetic is compared bit for bit with
/// the hardware's. Variadic `long double` arguments (`ld_vsum`, `ld_vmix`) ride the same checks. Skipped when no C compiler is installed; not run on Windows.
// rules: ext.2 ext.16 ext.17
#[test]
fn c_long_double() {
    if cfg!(windows) {
        return;
    }
    let fixture = repo_root().join("tests/compiler/native/c-long-double");
    let dir = TempDir::new("native-c-long-double");
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
        let mut cc = toolchain::command("cc");
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
    let calls = "third 0.333333 wide true\nadd true mix 10.5\nto_double true\nbox 6 tagged 6 true pair true 1\nspill 46.25\nvsum true true\nvmix true 4\n";
    let callbacks = "apply 7.5 true\nbox_apply 4.5\n";
    let jaic = |args: &[&str]| {
        let output = jaic().args(args).current_dir(&dir).output().unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).into_owned()
    };
    assert_eq!(jaic(&["-interp", "foreign_calls.jai"]), calls);
    // C calling a Jai procedure that takes a wide long double needs a native build; where long
    // double is float64 the interpreter's callbacks work as usual.
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        assert_eq!(jaic(&["-interp", "callbacks.jai"]), callbacks);
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
    let x86 = |name: &str| {
        jaic(&[
            &format!("{name}.jai"),
            "-target",
            "x86_64-apple-darwin",
            "-exe",
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
    let soft = jaic(&["-interp", "precision.jai", "-target", "x86_64-apple-darwin"]);
    let hard = x86("precision");
    assert!(soft.len() > 10_000, "{soft}");
    assert!(soft == hard, "soft-float x87 differs from the hardware");
}

/// C variadic foreign calls in a native build (Apple arm64 passes variadic arguments on the stack).
#[test]
fn c_variadic_calls() {
    let dir = TempDir::new("native-varargs");
    let source = repo_root().join("tests/compiler/regressions/c-variadic-foreign-calls.jai");
    let output = build_and_run(&source, &dir, "varargs").unwrap();
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "ok\n",
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// A static library compiled against the DLL CRT (`/MD`, as Visual Studio builds them) links into
/// a native build: its calls go through `__imp_` pointers that only the DLL CRT's import
/// libraries define, so the link must not fall back to the static CRT Clang's driver names.
#[cfg(windows)]
#[test]
fn dll_crt_static_library_links_on_windows() {
    let dir = TempDir::new("native-dll-crt");
    std::fs::write(
        dir.join("crtuser.c"),
        "#include <stdio.h>\n#include <stdlib.h>\n\
         int crtuser_lines(const char *path) {\n\
             FILE *f = fopen(path, \"r\");\n\
             char line[128];\n\
             int n = 0;\n\
             if (!f) return -1;\n\
             while (!feof(f) && fgets(line, sizeof line, f)) n++;\n\
             fclose(f);\n\
             if (n < 0) system(\"exit\");\n\
             return n;\n\
         }\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("main.jai"),
        "#import \"Basic\";\n\
         crtuser :: #library,no_dll \"crtuser\";\n\
         crtuser_lines :: (path: *u8) -> s32 #foreign crtuser;\n\
         main :: () {\n    print(\"lines: %\\n\", crtuser_lines(temp_c_string(#file)));\n}\n",
    )
    .unwrap();
    for (program, args) in [
        (
            "clang",
            &["-c", "-fms-runtime-lib=dll", "crtuser.c", "-o", "crtuser.o"][..],
        ),
        ("llvm-ar", &["rcs", "crtuser.lib", "crtuser.o"][..]),
    ] {
        let output = Command::new(program)
            .args(args)
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let output =
        build_and_run(&dir.join("main.jai"), &dir, "main").unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "lines: 6\n");
    // The DLL CRT, not a static copy: the executable imports the UCRT's stdio set.
    let imports = toolchain::command("llvm-readobj")
        .args(["--coff-imports"])
        .arg(exe_path(&dir, "main"))
        .output()
        .unwrap();
    let imports = String::from_utf8_lossy(&imports.stdout).to_ascii_lowercase();
    assert!(imports.contains("api-ms-win-crt-stdio"), "{imports}");
}

/// An MSVC C runtime exports no `long double` math (`sqrtl` is an inline call of `sqrt` in its
/// headers, `long double` being `double`), so an MSVC build calls the `double` functions.
#[test]
fn msvc_long_double_math_calls_the_double_functions() {
    let dir = TempDir::new("native-msvc-long-double");
    let msvc = jaic::target::Target::parse("x86_64-pc-windows-msvc")
        .unwrap()
        .toolchain();
    let double = msvc.rename("sqrtl").expect("sqrtl is a renamed symbol");
    assert!(msvc.is_runtime("libm"));
    std::fs::write(
        dir.join("main.jai"),
        "#import \"Extensions/Long_Double\";\n\
         libm :: #system_library \"libm\";\n\
         sqrtl :: (x: Long_Double) -> Long_Double #foreign libm;\n\
         sqrt :: (x: float64) -> float64 #foreign libm;\n\
         main :: () {\n    x := sqrtl(2) + sqrt(2);\n}\n",
    )
    .unwrap();
    // Only the IR matters: the link needs a Windows toolchain this host may lack.
    jaic()
        .args([
            "main.jai",
            "-target",
            "x86_64-pc-windows-msvc",
            "-emit_ir",
            "main.ll",
            "-exe",
            "main",
        ])
        .current_dir(&dir)
        .output()
        .unwrap();
    let ir = read_ir_parts(&dir.join("main.ll"));
    assert!(
        ir.contains(&format!("declare double @{double}(double)")),
        "{ir}"
    );
    assert!(!ir.contains("@sqrtl"), "{ir}");
}

/// Every MSVC link uses the DLL C runtime, which exports a variable only as `__imp_<name>`, so
/// a C-runtime variable is `dllimport` there, on both Windows CPUs; MinGW links its runtime
/// statically, and functions stay plain declarations.
#[test]
fn msvc_c_runtime_variables_are_dllimport() {
    let dir = TempDir::new("native-msvc-crt-data");
    // The runtime library that promises DLL data, and a generic name that does not.
    let msvc = jaic::target::Target::parse("x86_64-pc-windows-msvc")
        .unwrap()
        .toolchain();
    let jaic::toolchain::DataImport::DllImport {
        only,
    } = msvc.runtime_data
    else {
        panic!("the MSVC runtime is a DLL");
    };
    let dll = only[0];
    let generic = msvc
        .runtime_names
        .iter()
        .map(|row| row.name)
        .find(|name| !only.contains(name))
        .unwrap();
    std::fs::write(
        dir.join("main.jai"),
        format!(
            "crt :: #system_library \"{dll}\";\n\
             k32 :: #system_library \"kernel32\";\n\
             _fmode: s32 #elsewhere crt;\n\
             other_data: s32 #elsewhere k32;\n\
             generic :: #system_library \"{generic}\";\n\
             _HUGE: float64 #elsewhere generic;\n\
             sqrt :: (x: float64) -> float64 #foreign crt;\n\
             main :: () {{\n    x := _fmode + other_data;\n    y := sqrt(2) + _HUGE;\n}}\n"
        ),
    )
    .unwrap();
    let emit = |target: &str| {
        let name = format!("{target}.ll");
        // Only the IR matters: the link needs a toolchain this host may lack.
        jaic()
            .args([
                "main.jai", "-target", target, "-emit_ir", &name, "-exe", "main",
            ])
            .current_dir(&dir)
            .output()
            .unwrap();
        read_ir_parts(&dir.join(name))
    };
    for target in ["x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc"] {
        let ir = emit(target);
        assert!(
            ir.contains("@_fmode = external dllimport global"),
            "{target}: {ir}"
        );
        assert!(
            ir.contains("@other_data = external dllimport global"),
            "{target}: {ir}"
        );
        assert!(ir.contains("@_HUGE = external global"), "{target}: {ir}");
        assert!(
            ir.contains("declare double @sqrt(double)"),
            "{target}: {ir}"
        );
    }
    let ir = emit("x86_64-pc-windows-gnu");
    assert!(ir.contains("@_fmode = external global"), "{ir}");
    assert!(
        ir.contains("@other_data = external dllimport global"),
        "{ir}"
    );
}
