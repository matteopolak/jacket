//! Native tool configuration is tested without mutating process-wide environment.
#![cfg(unix)]
#[path = "support/native_tools.rs"]
mod native_tools;
use std::{
    ffi::OsStr,
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-native-tools-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn tool(&self, name: &str, major: u32) -> PathBuf {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            format!("#!/bin/sh\nprintf 'clang version {major}.1.1\\n'\n"),
        )
        .unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn explicit_override_wins_and_invalid_override_does_not_fall_back() {
    let fixture = Fixture::new();
    let explicit = fixture.tool("explicit tool", 22);
    fixture.tool("prefix/bin/clang", 22);
    let prefix = fixture.0.join("prefix");
    let selected =
        native_tools::resolve_clang(Some(explicit.as_os_str()), Some(&prefix), None, &fixture.0)
            .unwrap();
    assert_eq!(selected, explicit.canonicalize().unwrap());
    let missing = fixture.0.join("missing");
    assert!(
        native_tools::resolve_clang(Some(missing.as_os_str()), Some(&prefix), None, &fixture.0)
            .unwrap_err()
            .contains("cannot resolve")
    );
    assert!(
        native_tools::resolve_clang(Some(OsStr::new("")), Some(&prefix), None, &fixture.0)
            .unwrap_err()
            .contains("empty")
    );
}

#[test]
fn prefix_and_path_discovery_use_versioned_llvm_without_platform_paths() {
    let fixture = Fixture::new();
    let configured = fixture.tool("prefix/bin/clang", 22);
    let fallback = fixture.tool("path/clang-22", 22);
    fixture.tool("path/clang", 17);
    let prefix = fixture.0.join("prefix");
    let paths = std::env::join_paths([fixture.0.join("path")]).unwrap();
    assert_eq!(
        native_tools::resolve_clang(None, Some(&prefix), Some(&paths), &fixture.0).unwrap(),
        configured.canonicalize().unwrap()
    );
    assert_eq!(
        native_tools::resolve_clang(None, None, Some(&paths), &fixture.0).unwrap(),
        fallback.canonicalize().unwrap()
    );
    assert!(
        native_tools::resolve_clang(
            None,
            Some(&fixture.0.join("missing-prefix")),
            Some(&paths),
            &fixture.0
        )
        .unwrap_err()
        .contains("cannot resolve")
    );
}

#[test]
fn incompatible_or_non_executable_compilers_report_configuration_errors() {
    let fixture = Fixture::new();
    let old = fixture.tool("old-clang", 17);
    assert!(
        native_tools::resolve_clang(Some(old.as_os_str()), None, None, &fixture.0)
            .unwrap_err()
            .contains("Clang 22")
    );
    let denied = fixture.tool("non-executable", 22);
    fs::set_permissions(&denied, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(
        native_tools::resolve_clang(Some(denied.as_os_str()), None, None, &fixture.0)
            .unwrap_err()
            .contains("not executable")
    );
}

#[test]
fn protected_inputs_and_symlink_aliases_are_rejected_before_execution() {
    let fixture = Fixture::new();
    for protected in ["reference", "corpus", "vendor", ".git"] {
        let path = fixture.0.join(protected).join("clang");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let marker = fixture.0.join(format!("executed-{protected}"));
        fs::write(
            &path,
            format!(
                "#!/bin/sh\ntouch '{}'\nprintf 'clang version 22.1.1\\n'\n",
                marker.display()
            ),
        )
        .unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        let alias = fixture.0.join(format!("alias-{protected}"));
        symlink(&path, &alias).unwrap();
        for candidate in [&path, &alias] {
            assert!(
                native_tools::resolve_clang(Some(candidate.as_os_str()), None, None, &fixture.0)
                    .unwrap_err()
                    .contains("protected")
            );
            assert!(!marker.exists(), "protected tool must never be executed");
        }
    }
}

#[test]
fn selected_installed_clang_compiles_and_runs_only_a_fresh_c_fixture() {
    let fixture = Fixture::new();
    let source = fixture.0.join("main.c");
    fs::write(&source, "int main(void) { return 42; }\n").unwrap();
    let executable = fixture.0.join("program");
    let mut command = native_tools::clang_command();
    assert!(Path::new(command.get_program()).is_absolute());
    for variable in [
        "LD_PRELOAD",
        "DYLD_INSERT_LIBRARIES",
        "CCC_OVERRIDE_OPTIONS",
    ] {
        assert!(
            command
                .get_envs()
                .any(|(key, value)| key == variable && value.is_none())
        );
    }
    let output = command
        .arg(&source)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::process::Command::new(executable)
            .status()
            .unwrap()
            .code(),
        Some(42)
    );
    let cpp = fixture.0.join("main.cpp");
    fs::write(&cpp, "struct Value { virtual ~Value() = default; virtual int get() { return 42; } }; int main() { Value value; return value.get(); }\n").unwrap();
    let executable = fixture.0.join("cpp-program");
    let output = native_tools::clang_command()
        .arg("--driver-mode=g++")
        .arg("-std=c++17")
        .arg(&cpp)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::process::Command::new(executable)
            .status()
            .unwrap()
            .code(),
        Some(42)
    );
}
