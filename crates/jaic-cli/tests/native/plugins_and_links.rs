//! Plugins, custom link steps and workspace output.
use super::{build_and_run, corpus_cases, exe_path, jaic, matches, repo_root};
use std::process::Command;
use test_support::TempDir;
#[cfg(unix)]
use test_support::toolchain;

/// `jaic file.jai +Name` writes the program the plugin's workspace compiled, to `-exe` or,
/// without it, next to the source under the source's name.
// rules: plugin.1 plugin.11
#[test]
fn plug_builds_the_plugin_workspace() {
    let dir = TempDir::new("native-plug");
    let source = test_support::plugin::write_program(&dir);
    let exe = dir.join("uses_plugin_exe");
    let _ = std::fs::remove_file(&exe);
    let build = jaic()
        .arg(&source)
        .args(["-exe"])
        .arg(&exe)
        .arg("+Echo_Plugin")
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
    let build = jaic().arg(&source).arg("+Echo_Plugin").output().unwrap();
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
    let dir = TempDir::new("native-link-always");
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
/// `tools/differential/jaic_diff.py`: the native build expected it in registers while the callee
/// wrote through x8.
#[test]
fn cpp_non_pod_return_uses_the_native_hidden_result_pointer() {
    let dir = TempDir::new("native-cpp-non-pod");
    let cpp = dir.join("small.cpp");
    std::fs::write(&cpp, "#ifdef _WIN32\n#define API __declspec(dllexport)\n#else\n#define API\n#endif\nstruct Small { int value; explicit Small(int v): value(v) {} Small(const Small& v): value(v.value) {} ~Small() {} };\nextern \"C\" API Small make_small(int value) { return Small(value); }\n").unwrap();
    let library = dir.join(if cfg!(windows) {
        "small.dll"
    } else if cfg!(target_os = "macos") {
        "libsmall.dylib"
    } else {
        "libsmall.so"
    });
    let mut cc = Command::new("clang++");
    cc.args(["-shared", "-Wno-return-type-c-linkage"]);
    if !cfg!(windows) {
        cc.arg("-fPIC");
    }
    let compiled = cc.arg(cpp).arg("-o").arg(&library).output().unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let source = dir.join("main.jai");
    std::fs::write(&source, "#import \"Basic\";\nlib :: #library \"small\";\nSmall :: struct { value: s32; }\nmake_small :: (value: s32) -> Small #foreign lib #cpp_return_type_is_non_pod;\nmain :: () { result := make_small(41); assert(result.value == 41); print(\"ok\\n\"); }\n").unwrap();
    for level in ["-O0", "-O2"] {
        let executable = exe_path(&dir, &format!("small{level}"));
        let build = jaic()
            .arg(&source)
            .arg(level)
            .arg("-exe")
            .arg(&executable)
            .env("JAIC_NATIVE_LIBS", &dir)
            .output()
            .unwrap();
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let output = Command::new(executable).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"ok\n");
    }
}

/// A `#compiler` primitive called from compiled code says why the program stops.
#[test]
fn compiler_primitive_at_run_time_explains_the_trap() {
    let dir = TempDir::new("native-compiler-primitive");
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
/// tools/differential/jaigen.py program.
#[test]
fn optimized_sub_recurrence_matches_the_interpreter() {
    let dir = TempDir::new("native-sub-recurrence");
    let case = corpus_cases()
        .into_iter()
        .find(|c| c.id == "unrolled-sub-reduction")
        .unwrap();
    let exe = exe_path(&dir, "unrolled-sub-reduction");
    let build = jaic()
        .arg(&case.source)
        .args(["-O2", "-exe"])
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

/// Builds the stdlib's native Debug helper (`libdeveloper_tools_debug`, gitignored) with
/// `tools/developer_checks/build_developer_debug.py` when it is missing; the error says why it
/// could not be built (no Python or C compiler).
fn ensure_debug_helper() -> Result<(), String> {
    let folder = repo_root().join("stdlib/internal/developer-tools/debug-native");
    if ["dylib", "so", "a"].iter().any(|ext| {
        folder
            .join(format!("libdeveloper_tools_debug.{ext}"))
            .is_file()
    }) {
        return Ok(());
    }
    let script = repo_root().join("tools/developer_checks/build_developer_debug.py");
    match Command::new("python3").arg(script).output() {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => Err(String::from_utf8_lossy(&out.stderr).trim().to_string()),
        Err(error) => Err(error.to_string()),
    }
}

/// Compiled procedures keep frame records, so a frame-pointer stack walk (macOS libc
/// `backtrace`, behind Debug's `backtrace`) sees the callers. Without them it found no frames and
/// Native assertion failures retain stack information (independent compiler regression).
#[test]
fn backtrace_sees_compiled_callers() {
    if !cfg!(unix) {
        return;
    }
    if let Err(reason) = ensure_debug_helper() {
        eprintln!("skipping: the native Debug helper is not built and {reason}");
        return;
    }
    let dir = TempDir::new("native-backtrace");
    let source = dir.join("backtrace.jai");
    std::fs::write(
        &source,
        "#import \"Basic\";\nDebug :: #import \"Debug\";\n\
         inner :: () -> int {\n    frames := Debug.backtrace();\n    n := frames.count;\n    Debug.free_backtrace(frames);\n    return n;\n}\n\
         outer :: () -> int {\n    return inner() + 1;\n}\n\
         main :: () {\n    print(\"%\\n\", ifx outer() > 3 then \"ok\" else \"short\");\n}\n",
    )
    .unwrap();
    for opt in ["-O0", "-O2"] {
        let exe = exe_path(&dir, &format!("backtrace{opt}"));
        let build = jaic()
            .arg(&source)
            .args([opt, "-exe"])
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

/// `jaic -interp` writes what a metaprogram's workspace asks for, as `jaic` does: only the
/// top-level program is interpreted instead of compiled.
// rules: ws.15
#[test]
fn run_writes_workspace_output() {
    let dir = TempDir::new("native-run-workspace-output");
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
    let output = jaic()
        .arg("-interp")
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
/// what it does on `READY_FOR_CUSTOM_LINK_COMMAND` and `@OPTIONS@` in place of more options.
#[cfg(unix)]
const CUSTOM_LINK: &str = r#"#import "Basic";
#import "Compiler";
#import "Process";

SOURCE :: #string END
#import "Basic";
P :: #import "POSIX";
libc :: #system_library "libc";
raise_signal :: (signal: s32) -> s32 #foreign libc "raise";
start :: () {
    print("started at start\n");
    if get_command_line_arguments().count > 1 {
        raise_signal(P.SIGSEGV);
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
"#;

/// With `use_custom_link_command` the metaprogram links the objects jaic wrote (to
/// `intermediate_path`, with the LLVM IR asked for) and says how its link went; a metaprogram
/// that never says, or whose linker fails, fails the build. The program starts in the procedure
/// `entry_point_name` names.
// rules: bo.6 bo.7 bo.8 bo.9 bo.12
#[cfg(unix)]
#[test]
fn custom_link_command() {
    let units = Some("1");
    let linked = "result := run_command(..args); phase.linker_exit_code = result.exit_code; compiler_custom_link_command_is_complete(w);";
    let (dir, output) = build_custom_link("linked", linked, "", units);
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

    let (_, output) = build_custom_link("unlinked", "", "", units);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success()
            && stderr.contains("error: the metaprogram did not link")
            && stderr.contains("help: call `compiler_custom_link_command_is_complete"),
        "{stderr}"
    );

    let (_, output) = build_custom_link(
        "failed",
        "result := run_command(\"sh\", \"-c\", \"exit 3\"); phase.linker_exit_code = result.exit_code; compiler_custom_link_command_is_complete(w);",
        "",
        units,
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

/// Builds `CUSTOM_LINK` as `meta.jai` in its own directory. `units` pins `-codegen_units`
/// for the build, so a test about the objects of one module does not depend on the environment.
#[cfg(unix)]
fn build_custom_link(
    name: &str,
    link: &str,
    options: &str,
    units: Option<&str>,
) -> (TempDir, std::process::Output) {
    let dir = TempDir::new(&format!("native-custom-link-{name}"));
    let source = CUSTOM_LINK
        .replace("@LINK@", link)
        .replace("@OPTIONS@", options);
    std::fs::write(dir.join("meta.jai"), source).unwrap();
    let mut command = jaic();
    command.args(["meta.jai"]).current_dir(&dir);
    if let Some(units) = units {
        command.args(["-codegen_units", units]);
    }
    let output = command.output().unwrap();
    (dir, output)
}

/// A program divided into several codegen units (`-codegen_units 3`) hands the metaprogram
/// every object, which it links into a program that runs; the IR it asked for is written once per
/// unit, each file defining its own part.
// rules: bo.6 bo.7 bo.8
#[cfg(unix)]
#[test]
fn custom_link_command_with_several_codegen_units() {
    let linked = "result := run_command(..args); phase.linker_exit_code = result.exit_code; compiler_custom_link_command_is_complete(w);";
    let (dir, output) = build_custom_link("split", linked, "", Some("3"));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "objects 3\nwritten: failed false, linker exit code 0\n"
    );
    for object in ["obj/linked.o", "obj/linked.o.1.o", "obj/linked.o.2.o"] {
        assert!(dir.join(object).exists(), "{object} is missing");
    }
    for ir in ["obj/linked.ll", "obj/linked.1.ll", "obj/linked.2.ll"] {
        let text = std::fs::read_to_string(dir.join(ir)).unwrap_or_else(|e| panic!("{ir}: {e}"));
        assert!(
            text.contains("\"frame-pointer\"=\"all\""),
            "{ir} has no function with the frame-pointer attribute"
        );
    }
    let ran = Command::new(dir.join("out/linked")).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&ran.stdout), "started at start\n");
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
        let dir = TempDir::new(&format!("native-{name}"));
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
        let output = jaic()
            .args(["meta.jai"])
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
        assert_eq!(stderr.contains("native stack:"), on, "{name}: {stderr}");
        if on {
            assert!(stderr.contains("crash signal"), "{name}: {stderr}");
            assert!(
                stderr
                    .lines()
                    .skip_while(|line| *line != "native stack:")
                    .skip(1)
                    .any(|line| line.contains("0x")),
                "{name}: no native frame addresses: {stderr}"
            );
        }
        if cfg!(target_os = "macos") {
            let load = Command::new("otool").arg("-l").arg(&exe).output().unwrap();
            let load = String::from_utf8_lossy(&load.stdout);
            assert!(load.contains("minos 12.0"), "{load}");
        }
    }
}

/// `compiler_add_library_search_directory` makes a `#library` in an unlisted directory load, for
/// compile-time calls and in `jaic -interp`. Unix only: the fixture is a shared library built with `cc`.
// rules: compiler.28
#[cfg(unix)]
#[test]
fn library_search_directory_added_by_a_metaprogram() {
    let fixture = repo_root().join("tests/compiler/native/library-search-dir");
    let dir = TempDir::new("native-library-search-dir");
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    for name in ["probe.c", "probe.jai"] {
        std::fs::copy(fixture.join(name), dir.join(name)).unwrap();
    }
    let lib = if cfg!(target_os = "macos") {
        "lib/libsearchprobe.dylib"
    } else {
        "lib/libsearchprobe.so"
    };
    let Ok(built) = toolchain::command("cc")
        .args(["-shared", "-fPIC", "-o", lib, "probe.c"])
        .current_dir(&dir)
        .output()
    else {
        eprintln!("skipping: no C compiler");
        return;
    };
    assert!(built.status.success());
    let output = jaic()
        .args(["-interp", "probe.jai"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "4242 4242\n");
}

/// `function_sections` puts each function of an ELF program in a `.text.<name>` section; an
/// object format without function sections takes the option and ignores it.
#[cfg(unix)]
#[test]
fn function_sections_name_a_section_per_function() {
    let link = "compiler_custom_link_command_is_complete(w);";
    let (dir, output) = build_custom_link(
        "sections-elf",
        link,
        "options.os_target = .LINUX; options.cpu_target = .X64; options.llvm_options.function_sections = true;",
        Some("1"),
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    let ir = std::fs::read_to_string(dir.join("obj/linked.ll")).unwrap();
    assert!(ir.contains("section \".text.start"), "{ir}");

    let (dir, output) = build_custom_link(
        "sections-plain",
        link,
        "options.os_target = .LINUX; options.cpu_target = .X64;",
        Some("1"),
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let ir = std::fs::read_to_string(dir.join("obj/linked.ll")).unwrap();
    assert!(!ir.contains("section \".text."), "{ir}");

    let (_, output) = build_custom_link(
        "sections-host",
        "result := run_command(..args); phase.linker_exit_code = result.exit_code; compiler_custom_link_command_is_complete(w);",
        "options.llvm_options.function_sections = true;",
        Some("1"),
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
