//! The formatter build, Windows runtime programs and PDBs.
use super::{build_and_run, exe_path, formatter_fixtures, jaic, repo_root};
use std::path::Path;
use std::process::Command;

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
    let build = jaic()
        .arg(repo_root().join("jaifmt/main.jai"))
        .args(["-O2", "-exe"])
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

/// `jaic jaifmt/build.jai - -o <file>` builds the same jaifmt through the Compiler module,
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
    let build = jaic()
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

    let bad = jaic()
        .arg(repo_root().join("jaifmt/build.jai"))
        .args(["-", "bogus"])
        .output()
        .unwrap();
    assert!(!bad.status.success());
    assert!(
        String::from_utf8_lossy(&bad.stderr).contains("invalid value `bogus` for `<TARGET>`"),
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
    let build = jaic()
        .arg(repo_root().join("jaifmt/main.jai"))
        .args(["-O2", "-exe"])
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
            } else if path.extension().is_some_and(|e| e == "jai")
                && !formatter_fixtures::is_appledouble_metadata(&path)
            {
                std::fs::create_dir_all(to).unwrap();
                let destination = to.join(name);
                std::fs::copy(&path, &destination).unwrap();
                // Frozen source inputs may be read-only. This test formats its
                // own copy; the original file's protections stay untouched.
                let mut permissions = std::fs::metadata(&destination).unwrap().permissions();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    permissions.set_mode(permissions.mode() | 0o200);
                }
                #[cfg(not(unix))]
                permissions.set_readonly(false);
                std::fs::set_permissions(&destination, permissions).unwrap();
                *count += 1;
            }
        }
    }
    let tree = dir.join("tree");
    let mut count = 0;
    let protected = dir.join("protected-input");
    std::fs::create_dir_all(&protected).unwrap();
    let protected_file = protected.join("read-only.jai");
    let protected_text = "main :: () { value:=1; }\n";
    std::fs::write(&protected_file, protected_text).unwrap();
    let mut protected_permissions = std::fs::metadata(&protected_file).unwrap().permissions();
    protected_permissions.set_readonly(true);
    std::fs::set_permissions(&protected_file, protected_permissions).unwrap();
    let readonly_retained = std::fs::metadata(&protected_file)
        .unwrap()
        .permissions()
        .readonly();
    // Some transport filesystems accept chmod but discard write-bit changes.
    // CI must exercise the actual readonly premise on a capable filesystem.
    formatter_fixtures::require_readonly_capability(
        readonly_retained,
        std::env::var_os("CI").is_some() || std::env::var_os("GITHUB_ACTIONS").is_some(),
    );
    if !readonly_retained {
        eprintln!(
            "readonly capability unavailable: set_permissions succeeded but {} remains writable; copy formatting and idempotence checks still run",
            protected_file.display()
        );
    }
    copy_jai(&protected, &tree.join("protected-input"), &mut count);
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
    assert_eq!(
        std::fs::read_to_string(&protected_file).unwrap(),
        protected_text
    );
    assert_eq!(
        std::fs::metadata(protected_file)
            .unwrap()
            .permissions()
            .readonly(),
        readonly_retained,
        "formatting a copy must preserve the original permissions"
    );
}

/// The Windows runtime test program: natively wherever the tests run, and cross-built with
/// `-os windows` (x64, and `-cpu arm64`) when a MinGW-w64 toolchain for that CPU is installed
/// (the result is checked to be a PE executable for it; CI runs them on Windows, see
/// `tools/native/windows_cross.py`).
#[test]
fn windows_runtime_program() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-windows-runtime");
    std::fs::create_dir_all(&dir).unwrap();
    let source = repo_root().join("tests/compiler/native/windows/runtime.jai");
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
        let run = jaic()
            .arg("-interp")
            .arg(&source)
            .args(["--", "runtime-arg"])
            .output()
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&run.stdout).replace('\r', ""),
            "ok\n",
            "jaic -interp: {}: {}",
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
        let build = jaic()
            .arg(&source)
            .args(["-os", "windows", "-cpu", cpu, "-exe"])
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

/// The stdlib's thread tests under `jaic -interp`: on Windows they exercise the Win32 side of the
/// interpreter's thread scheduler (`CreateThread`, critical sections, condition variables,
/// `WaitForSingleObject`, `Sleep`); elsewhere the pthread side, as the sweep does.
#[test]
fn interpreted_threads() {
    let dir = repo_root().join("tests/compiler/regressions");
    let mut checked = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if !(name.starts_with("threads-") && name.ends_with(".jai")) {
            continue;
        }
        let run = jaic()
            .arg("-interp")
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
    assert!(
        checked >= 2,
        "no tests/compiler/regressions/threads-*.jai found"
    );
}

/// MSVC builds keep their `CodeView` in a PDB next to the executable (`/DEBUG /PDB:`), with line
/// information for the Jai source, and write none with `-no_debug_info`. The line check needs
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
    assert!(pdb.is_file(), "no {} after jaic", pdb.display());
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
    let build = jaic()
        .arg(&source)
        .arg("-no_debug_info")
        .arg("-exe")
        .arg(exe_path(&dir, "pdb_lines"))
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert!(!pdb.exists(), "-no_debug_info still wrote a PDB");
}
