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
}

/// `-plug Name` compiles the program in a workspace with the plugin's hooks; options jaic does
/// not know go to the plugins, and `run` refuses plugins.
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
