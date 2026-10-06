//! Command-line behavior of the `jaic` binary that is not about native code generation.
use std::path::Path;
use std::process::Command;

mod common;

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

/// The `Long_Double` extension on targets whose C `long double` is wider than float64: the
/// interpreter's soft-float runs x87 (`x86_64-linux-gnu`, `x86_64-apple-darwin`) and binary128
/// (`aarch64-linux-gnu`, `wasm32`) arithmetic whatever the host. The name only exists after
/// `#import "Jaic_Extensions"`, and `#jaic_type` rejects names it does not know.
// rules: ext.1 ext.3 ext.4 ext.8 ext.9 ext.17
#[test]
fn long_double_extension_on_wide_targets() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/stdlib/jaic-extensions-long-double.jai");
    for target in [
        "x86_64-linux-gnu",
        "x86_64-apple-darwin",
        "aarch64-linux-gnu",
        "wasm32-unknown-unknown",
    ] {
        let output = Command::new(JAIC)
            .arg("run")
            .arg(&source)
            .args(["-target", target])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{target}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "ok wide\n",
            "{target}"
        );
    }

    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-long-double");
    std::fs::create_dir_all(&dir).unwrap();
    for (name, text, message) in [
        (
            "no_import",
            "main :: () { x: Long_Double; }\n",
            "unknown identifier 'Long_Double'",
        ),
        (
            "unknown_name",
            "T :: #jaic_type quad_float;\nmain :: () { x: T; }\n",
            "unknown jaic extension type 'quad_float'",
        ),
        (
            "no_name",
            "T :: #jaic_type;\nmain :: () { x: T; }\n",
            "#jaic_type expects a name",
        ),
    ] {
        let source = dir.join(format!("{name}.jai"));
        std::fs::write(&source, text).unwrap();
        let output = Command::new(JAIC)
            .arg("check")
            .arg(&source)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{name}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{name}: {stderr}");
    }
    // A wide long double through C varargs is rejected rather than passed wrongly.
    let source = dir.join("varargs.jai");
    std::fs::write(
        &source,
        "#import \"Jaic_Extensions\";\n\
         libc :: #system_library \"libc\";\n\
         printf :: (fmt: *u8, args: ..Any) -> s32 #foreign libc;\n\
         main :: () { x: Long_Double = 2.5; printf(\"%Lf\\n\", x); }\n",
    )
    .unwrap();
    let output = Command::new(JAIC)
        .arg("run")
        .arg(&source)
        .args(["-target", "x86_64-linux-gnu"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cannot be passed to a C variadic procedure"),
        "{stderr}"
    );
    // On a wide target, narrowing to float64 needs a cast and `%` / bit operations are not
    // defined.
    for (name, body, message) in [
        (
            "narrowing",
            "x: Long_Double = 7; f: float64 = x;",
            "expected float64, found Long_Double",
        ),
        (
            "remainder",
            "x: Long_Double = 7; y := x % 2;",
            "operator % is not defined for Long_Double",
        ),
        (
            "bit_and",
            "x: Long_Double = 7; y := x & x;",
            "operator & is not defined for Long_Double",
        ),
    ] {
        let source = dir.join(format!("wide_{name}.jai"));
        std::fs::write(
            &source,
            format!("#import \"Jaic_Extensions\";\nmain :: () {{ {body} }}\n"),
        )
        .unwrap();
        let output = Command::new(JAIC)
            .arg("check")
            .arg(&source)
            .args(["-target", "x86_64-linux-gnu"])
            .output()
            .unwrap();
        assert!(!output.status.success(), "{name}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{name}: {stderr}");
    }
}

/// `-plug Name` compiles the program in a workspace with the plugin's hooks; options jaic does
/// not know go to the plugins, and `run` refuses plugins.
// rules: plugin.1 plugin.2 plugin.7 plugin.9 plugin.10
#[test]
fn plug_hooks_a_plugin_into_check() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-plug");
    let source = common::write_plugin_program(&dir);
    let check = |extra: &[&str]| {
        Command::new(JAIC)
            .arg("check")
            .arg(&source)
            .args(extra)
            .output()
            .unwrap()
    };
    let output = check(&["-plug", "Echo_Plugin", "-shout"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "LOUD finished, typechecked: true\n"
    );
    // Without the plugin the program does not compile; a refused option fails the build.
    assert!(!check(&[]).status.success());
    assert!(!check(&["-plug", "Echo_Plugin", "-refuse"]).status.success());
    // Unknown options without plugins, and plugins with `run`, are usage errors.
    assert_eq!(check(&["-shout"]).status.code(), Some(2));
    let run = Command::new(JAIC)
        .arg("run")
        .arg(&source)
        .args(["-plug", "Echo_Plugin"])
        .output()
        .unwrap();
    assert_eq!(run.status.code(), Some(2));
}

/// `--timings` prints one `jaic-timing: <phase> <seconds> <calls>` line per phase (read by
/// `tools/compile_bench.py`); without it nothing is printed.
#[test]
fn timings_report_phases() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-timings");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("timed.jai");
    std::fs::write(&source, "main :: () {}\n").unwrap();
    let stderr = |extra: &[&str]| {
        let output = Command::new(JAIC)
            .arg("check")
            .arg(&source)
            .args(extra)
            .output()
            .unwrap();
        assert!(output.status.success());
        String::from_utf8_lossy(&output.stderr).into_owned()
    };
    let report = stderr(&["--timings"]);
    for phase in ["front end", "workspaces", "total"] {
        let line = report
            .lines()
            .find(|l| l.starts_with(&format!("jaic-timing: {phase} ")))
            .unwrap_or_else(|| panic!("no {phase} line in {report}"));
        let fields: Vec<&str> = line.rsplitn(3, ' ').collect();
        assert!(
            fields[1].parse::<f64>().is_ok() && fields[0] == "1",
            "{line}"
        );
    }
    assert!(!stderr(&[]).contains("jaic-timing"));
}

/// `JAIC_MEMORY_LIMIT` stops a run whose allocations cross it, whether the memory is the
/// program's heap at run time (`alloc` reaches C `malloc`), the interpreter's at compile time
/// (`#run`) or the sandbox heap (`-os wasm`), and leaves a program under it alone.
#[test]
fn memory_limit_stops_unbounded_allocation() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-memory-limit");
    std::fs::create_dir_all(&dir).unwrap();
    let grow = "grow :: () -> int {\n\
                \x20   total := 0;\n\
                \x20   while true { p := alloc(1 << 20); memset(p, 1, 1 << 20); total += 1; }\n\
                \x20   return total;\n\
                }\n";
    let runtime = dir.join("runtime.jai");
    std::fs::write(
        &runtime,
        format!("#import \"Basic\";\n{grow}main :: () {{ print(\"%\\n\", grow()); }}\n"),
    )
    .unwrap();
    let compile_time = dir.join("compile_time.jai");
    std::fs::write(
        &compile_time,
        format!(
            "#import \"Basic\";\n{grow}N :: #run grow();\nmain :: () {{ print(\"%\\n\", N); }}\n"
        ),
    )
    .unwrap();
    let small = dir.join("small.jai");
    std::fs::write(
        &small,
        "#import \"Basic\";\n\
         main :: () { a: [..] int; for 1..10000 array_add(*a, it); print(\"%\\n\", a.count); }\n",
    )
    .unwrap();
    let run = |source: &Path, extra: &[&str]| {
        Command::new(JAIC)
            .arg("run")
            .arg(source)
            .args(extra)
            .env("JAIC_MEMORY_LIMIT", "128M")
            .output()
            .unwrap()
    };
    for (source, extra) in [
        (&runtime, &[][..]),
        (&compile_time, &[][..]),
        (&runtime, &["-os", "wasm"][..]),
    ] {
        let output = run(source, extra);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            output.status.code(),
            Some(120),
            "{} {extra:?}: {stderr}",
            source.display()
        );
        assert!(
            stderr.contains("error: memory limit of 128 MiB exceeded"),
            "{stderr}"
        );
    }
    let output = run(&small, &[]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "10000\n");
}

/// A plugin that prints each hook as it is called, tagged with its module parameter.
const ORDER_PLUGIN: &str = r#"#module_parameters(TAG := "default");
#import "Basic";
#import "Compiler";
get_plugin :: () -> *Metaprogram_Plugin {
    p := New(Metaprogram_Plugin);
    p.init = (p: *Metaprogram_Plugin, options: [] string) -> bool {
        print("% init %\n", TAG, options);
        return true;
    };
    p.before_intercept = (p: *Metaprogram_Plugin, flags: *Intercept_Flags) {
        print("% before_intercept\n", TAG);
    };
    p.add_source = (p: *Metaprogram_Plugin) {
        print("% add_source\n", TAG);
    };
    p.finish = (p: *Metaprogram_Plugin) {
        print("% finish\n", TAG);
    };
    p.shutdown = (p: *Metaprogram_Plugin) {
        print("% shutdown\n", TAG);
    };
    return p;
}
"#;

/// Hook order, repeated `-plug`/`-plugin` with module parameters, jaic's own options among
/// plugin options, and a failing program under a plugin.
// rules: plugin.3 plugin.4 plugin.5 plugin.6 plugin.8
#[test]
fn plug_hook_order_parameters_and_failures() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-plug-order");
    std::fs::create_dir_all(dir.join("modules/Order_Plugin")).unwrap();
    std::fs::write(dir.join("modules/Order_Plugin/module.jai"), ORDER_PLUGIN).unwrap();
    let good = dir.join("good.jai");
    std::fs::write(
        &good,
        "#import \"Basic\";\nmain :: () { print(\"ran\\n\"); }\n",
    )
    .unwrap();
    let bad = dir.join("bad.jai");
    std::fs::write(&bad, "main :: () { x: int = \"no\"; }\n").unwrap();
    let check = |source: &Path, extra: &[&str]| {
        Command::new(JAIC)
            .arg("check")
            .arg(source)
            .args(extra)
            .output()
            .unwrap()
    };
    let stdout = |output: &std::process::Output| {
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).into_owned()
    };
    assert_eq!(
        stdout(&check(&good, &["-plug", "Order_Plugin"])),
        "default init []\ndefault before_intercept\ndefault add_source\n\
         default finish\ndefault shutdown\n"
    );
    assert_eq!(
        stdout(&check(
            &good,
            &[
                "-plug",
                "Order_Plugin(TAG=\"one\")",
                "-plugin",
                "Order_Plugin(TAG=\"two\")",
                "-x",
                "y",
            ],
        )),
        "one init [\"-x\", \"y\"]\ntwo init [\"-x\", \"y\"]\n\
         one before_intercept\ntwo before_intercept\none add_source\ntwo add_source\n\
         one finish\ntwo finish\none shutdown\ntwo shutdown\n"
    );
    // `-I` after a plugin option is still jaic's, with its value.
    let with_import = stdout(&check(
        &good,
        &["-plug", "Order_Plugin", "-x", "-I", "elsewhere", "y"],
    ));
    assert!(
        with_import.starts_with("default init [\"-x\", \"y\"]\n"),
        "{with_import}"
    );
    // A program that fails to compile under a plugin still fails the command.
    let failed = check(&bad, &["-plug", "Order_Plugin"]);
    assert_eq!(failed.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&failed.stderr).contains("type mismatch"));
}

/// `print` inside `#run` writes to stdout during compilation, before anything `main` prints,
/// even when the `#run` is written after `main`.
// rules: ctexec.6
#[test]
fn compile_time_print_precedes_program_output() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-compile-time-print");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("order.jai");
    std::fs::write(
        &source,
        "#import \"Basic\";\n\
         main :: () { print(\"runtime\\n\"); }\n\
         #run print(\"compile time\\n\");\n",
    )
    .unwrap();
    let output = Command::new(JAIC).arg("run").arg(&source).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "compile time\nruntime\n"
    );
}

/// `jaic check` only checks workspaces: one asking for an executable compiles, writes nothing
/// and gets a warning that points to `jaic build`. `jaic run -no_workspace_output` skips the
/// output quietly. (`jaic build` and plain `jaic run` write it:
/// `run_writes_workspace_output` in native.rs.)
// rules: ws.15 ws.17
#[test]
fn check_writes_no_workspace_output() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-workspace-no-output");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("meta.jai");
    std::fs::write(&source, WORKSPACE_ASKING_FOR_OUTPUT).unwrap();
    for args in [&["check"][..], &["run", "-no_workspace_output"][..]] {
        let output = Command::new(JAIC)
            .arg(args[0])
            .arg(&source)
            .args(&args[1..])
            .current_dir(&dir)
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{args:?}: {stderr}");
        let written: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .filter(|name| name.to_string_lossy().starts_with("target-prog"))
            .collect();
        assert!(written.is_empty(), "{args:?} wrote {written:?}");
        // `check` says so, naming the command that would write it; the flag asked for silence.
        let warned = stderr.contains("target-prog") && stderr.contains("`jaic build ");
        assert_eq!(warned, args[0] == "check", "{args:?}: {stderr}");
    }
}

/// A metaprogram whose workspace asks for an executable `target-prog` in the current directory.
const WORKSPACE_ASKING_FOR_OUTPUT: &str = r##"#import "Basic";
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
"##;

/// `-no_dce` type-checks module bodies nothing calls; by default only the program's own
/// unreferenced bodies are checked.
// rules: dce.2 dce.6 dce.8
#[test]
fn dead_code_elimination_flag() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-no-dce");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("modules/Stale")).unwrap();
    std::fs::write(
        dir.join("modules/Stale/module.jai"),
        "used :: () -> int { return 1; }\nstale :: () { missing_in_stale_module(); }\n",
    )
    .unwrap();
    let clean = dir.join("clean.jai");
    std::fs::write(&clean, "#import \"Stale\";\nmain :: () { used(); }\n").unwrap();
    let dirty = dir.join("dirty.jai");
    std::fs::write(
        &dirty,
        "#import \"Stale\";\nunused :: () { missing_in_program(); }\nmain :: () { used(); }\n",
    )
    .unwrap();
    let check = |source: &Path, extra: &[&str]| {
        let output = Command::new(JAIC)
            .arg("check")
            .arg(source)
            .args(extra)
            .current_dir(&dir)
            .output()
            .unwrap();
        (
            output.status.success(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
    };
    let (ok, stderr) = check(&clean, &[]);
    assert!(ok, "{stderr}");
    let (ok, stderr) = check(&clean, &["-no_dce"]);
    assert!(
        !ok && stderr.contains("missing_in_stale_module"),
        "{stderr}"
    );
    let (ok, stderr) = check(&dirty, &[]);
    assert!(!ok && stderr.contains("missing_in_program"), "{stderr}");
}

/// A struct constant nothing uses is checked after its struct is laid out, so it can name the
/// struct's members, also in a struct declared inside another struct.
// rules: dce.13
#[test]
fn unreferenced_struct_constant_names_a_member() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-struct-constant");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("padded.jai");
    std::fs::write(
        &source,
        "Group :: struct {\n    Padded :: struct {\n        using info: Info;\n        \
         SIZE :: #run align_forward(size_of(type_of(info)), 64);\n        \
         padding: [SIZE - size_of(Info)] u8;\n    }\n    Info :: struct { a: int; }\n}\n\
         align_forward :: (n: int, a: int) -> int { return (n + a - 1) / a * a; }\n\
         main :: () {}\n",
    )
    .unwrap();
    for extra in [&[][..], &["-no_dce"][..]] {
        let output = Command::new(JAIC)
            .arg("check")
            .arg(&source)
            .args(extra)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{extra:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
