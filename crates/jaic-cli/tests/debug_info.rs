//! Native debug information (`docs/native/debug-info.md`): the DWARF `jaic build` emits
//! is valid and describes the program, and a debugger can use it. Tests skip when
//! `llvm-dwarfdump` or `lldb` is not installed.
use std::path::{Path, PathBuf};
use std::process::Command;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/native/debug-info/main.jai")
}

/// Build the fixture into `dir/name` with extra arguments; returns the executable.
fn build(dir: &str, name: &str, args: &[&str], units: Option<&str>) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(dir);
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join(name);
    let _ = std::fs::remove_dir_all(dsym(&exe));
    let source = fixture();
    let mut cmd = Command::new(JAIC);
    cmd.arg("build")
        .arg(&source)
        .arg("-o")
        .arg(&exe)
        .args(args)
        .current_dir(source.parent().unwrap());
    if let Some(units) = units {
        cmd.env("JAIC_CODEGEN_UNITS", units);
    }
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    exe
}

fn dsym(exe: &Path) -> PathBuf {
    let mut name = exe.as_os_str().to_owned();
    name.push(".dSYM");
    PathBuf::from(name)
}

/// Where the debug information of `exe` lives: the `.dSYM` bundle on macOS.
fn debug_file(exe: &Path) -> PathBuf {
    if cfg!(target_os = "macos") {
        dsym(exe)
    } else {
        exe.to_path_buf()
    }
}

fn dwarfdump() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(prefix) = std::env::var("LLVM_SYS_221_PREFIX") {
        candidates.push(Path::new(&prefix).join("bin/llvm-dwarfdump"));
    }
    candidates.push(PathBuf::from("/opt/homebrew/opt/llvm/bin/llvm-dwarfdump"));
    candidates.push(PathBuf::from("llvm-dwarfdump"));
    candidates.into_iter().find(|c| {
        Command::new(c)
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success())
    })
}

fn run_dwarfdump(tool: &Path, args: &[&str], file: &Path) -> (bool, String) {
    let out = Command::new(tool).args(args).arg(file).output().unwrap();
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.success(), text)
}

#[test]
fn dwarf_describes_procedures_variables_and_types() {
    let Some(tool) = dwarfdump() else {
        eprintln!("skipped: llvm-dwarfdump not found");
        return;
    };
    let exe = build("debug-info-dwarf", "prog", &[], None);
    let file = debug_file(&exe);
    let (ok, verify) = run_dwarfdump(&tool, &["--verify"], &file);
    assert!(ok, "llvm-dwarfdump --verify failed:\n{verify}");
    let (_, info) = run_dwarfdump(&tool, &["--debug-info", "--debug-line"], &file);
    for expected in [
        // Procedures, with polymorph instances under their written name.
        "DW_AT_name\t(\"main\")",
        "DW_AT_name\t(\"scale\")",
        "DW_AT_name\t(\"total_of\")",
        // Both source files of a multi-file program.
        "main.jai",
        "shapes.jai",
        // Parameters and locals with Jai type names.
        "DW_TAG_formal_parameter",
        "DW_AT_name\t(\"inner\")",
        "DW_AT_name\t(\"it\")",
        "DW_AT_name\t(\"s64\")",
        "DW_AT_name\t(\"float32\")",
        "DW_AT_name\t(\"string\")",
        "DW_AT_name\t(\"[] s64\")",
        "DW_AT_name\t(\"[..] s64\")",
        "DW_AT_name\t(\"Vec2\")",
        "DW_TAG_enumeration_type",
        "DW_AT_name\t(\"GREEN\")",
        "DW_TAG_lexical_block",
        // Globals.
        "DW_AT_name\t(\"counter\")",
    ] {
        assert!(info.contains(expected), "missing {expected:?} in:\n{info}");
    }
}

/// Split codegen writes one compile unit per object; each must be complete on its own.
#[test]
fn dwarf_is_valid_with_split_codegen() {
    let Some(tool) = dwarfdump() else {
        eprintln!("skipped: llvm-dwarfdump not found");
        return;
    };
    let exe = build("debug-info-split", "prog", &[], Some("3"));
    let file = debug_file(&exe);
    let (ok, verify) = run_dwarfdump(&tool, &["--verify"], &file);
    assert!(ok, "llvm-dwarfdump --verify failed:\n{verify}");
    let (_, info) = run_dwarfdump(&tool, &["--debug-info"], &file);
    assert_eq!(info.matches("DW_TAG_compile_unit").count(), 3, "{info}");
}

#[test]
fn no_debug_info_flag_omits_it() {
    let exe = build("debug-info-off", "prog", &["--no-debug-info"], None);
    if cfg!(target_os = "macos") {
        assert!(!dsym(&exe).exists());
    } else if let Some(tool) = dwarfdump() {
        let (_, info) = run_dwarfdump(&tool, &["--debug-info"], &exe);
        assert!(!info.contains("DW_TAG_compile_unit"), "{info}");
    }
}

fn lldb_available() -> bool {
    Command::new("lldb")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Breakpoints by `file:line` in both files, backtraces with Jai names and locations,
/// and variables of many types, through a real debugger.
#[test]
fn lldb_breakpoints_backtraces_and_variables() {
    if !lldb_available() {
        eprintln!("skipped: lldb not found");
        return;
    }
    let exe = build("debug-info-lldb", "prog", &[], None);
    let out = Command::new("lldb")
        .arg("--batch")
        .args(["-o", "b shapes.jai:25", "-o", "b main.jai:27", "-o", "run"])
        .args(["-o", "bt", "-o", "frame variable"])
        .args(["-o", "breakpoint delete 1", "-o", "continue"])
        .args(["-o", "frame variable", "-o", "p counter", "-o", "p *p"])
        .arg(&exe)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    if text.contains("error: process launch failed") {
        eprintln!("skipped: lldb cannot launch processes here:\n{text}");
        return;
    }
    // In the polymorphic procedure, called from main (multi-file backtrace).
    let at_first = text
        .split("breakpoint delete")
        .next()
        .unwrap_or_default()
        .to_string();
    for expected in [
        "total_of(xs=",
        "at shapes.jai:25",
        "main at main.jai:23",
        "(s64) it = 10",
        "(s64) it_index = 0",
        "(s64) total = 0",
    ] {
        assert!(
            at_first.contains(expected),
            "missing {expected:?} in:\n{text}"
        );
    }
    let at_second = text.split("breakpoint delete").nth(1).unwrap_or_default();
    for expected in [
        "at main.jai:27",
        "(s64) a = 42",
        "(float64) b = 3.5",
        "(bool) flag = true",
        "\"hello\"",
        "(Vec2) v = (x = 1, y = 2)",
        "(Color) c = GREEN",
        "(s64) inner = 43",
        "(s64) s = 60",
        "(float32) f = 4",
        "(s64) 7",
        "(s64) 42",
    ] {
        assert!(
            at_second.contains(expected),
            "missing {expected:?} in:\n{text}"
        );
    }
}
