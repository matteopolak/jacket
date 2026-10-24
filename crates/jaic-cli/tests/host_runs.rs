//! Isolated host values survive source checking and actual target execution.
mod common;
use common::success_silent as success;
use std::{path::PathBuf, process::Command};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/host-runs/host-result-transfer.jai")
}

#[cfg(feature = "llvm")]
#[test]
fn workspace_host_clock_produces_a_linux_x64_object() {
    let tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/host-runs");
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("host-clock-object");
    std::fs::create_dir_all(&directory).unwrap();
    success(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(tests.join("host-clock-workspace.jai"))
            .arg("-")
            .arg(&directory)
            .arg(
                tests
                    .join("isolated-host-clock-result.jai")
                    .canonicalize()
                    .unwrap(),
            ),
    );
    let object = std::fs::read(directory.join("host-clock-target")).unwrap();
    assert_eq!(&object[..4], b"\x7fELF");
    assert_eq!(object[4], 2, "64-bit object");
    assert_eq!(
        u16::from_le_bytes([object[16], object[17]]),
        1,
        "relocatable object"
    );
    assert_eq!(
        u16::from_le_bytes([object[18], object[19]]),
        62,
        "x86-64 target"
    );
}

#[test]
fn compile_time_clock_reads_the_host_for_every_target_os() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/host-runs/cross-target-compile-time-clock.jai");
    for os in ["linux", "macos", "windows"] {
        success(
            Command::new(env!("CARGO_BIN_EXE_jaic"))
                .arg(&source)
                .args(["-check", "-os", os, "-cpu", "x64"]),
        );
    }
}

/// Targets that differ from the machine running the tests, so the host twin is in play.
fn foreign_targets() -> Vec<(&'static str, &'static str)> {
    let host_os = match std::env::consts::OS {
        "macos" => "macos",
        "windows" => "windows",
        _ => "linux",
    };
    let host_cpu = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        _ => "x64",
    };
    let mut targets = Vec::new();
    for os in ["linux", "windows"] {
        for cpu in ["x64", "arm64"] {
            if (os, cpu) != (host_os, host_cpu) {
                targets.push((os, cpu));
            }
        }
    }
    targets
}

fn check_for_foreign_targets(file: &std::path::Path) {
    for (os, cpu) in foreign_targets() {
        success(
            Command::new(env!("CARGO_BIN_EXE_jaic"))
                .arg(file)
                .args(["-check", "-os", os, "-cpu", cpu]),
        );
    }
}

fn host_run_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/host-runs")
        .join(name)
}

#[test]
fn plain_run_sees_the_host_when_cross_compiling() {
    check_for_foreign_targets(&host_run_fixture("cross-target-compile-time-host.jai"));
}

#[test]
fn results_of_a_run_convert_to_the_target_program() {
    check_for_foreign_targets(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/compiler/regressions/compile-time-host-results.jai"),
    );
}

#[test]
fn build_programs_start_processes_and_drive_workspaces_when_cross_compiling() {
    check_for_foreign_targets(&host_run_fixture("cross-target-metaprogram.jai"));
}

#[test]
fn a_run_that_returns_code_stays_on_the_target_compiler() {
    check_for_foreign_targets(&host_run_fixture("cross-target-code-result.jai"));
}

#[test]
fn a_run_that_returns_a_nested_enum_stays_on_the_target_compiler() {
    check_for_foreign_targets(&host_run_fixture("cross-target-local-enum-result.jai"));
}

#[test]
fn a_result_with_a_host_dependent_layout_is_rejected() {
    // Windows is the target whose `Odd` layout differs; a Windows host checks for Linux instead,
    // where the layouts agree, so the test only applies elsewhere.
    if std::env::consts::OS == "windows" {
        return;
    }
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg(host_run_fixture("cross-target-layout-mismatch.jai"))
        .args(["-check", "-os", "windows", "-cpu", "x64"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("different layout"), "{stderr}");
}

#[test]
fn host_values_runs_in_the_interpreter() {
    success(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(fixture())
            .arg("-interp"),
    );
}

#[cfg(feature = "llvm")]
#[test]
fn host_values_runs_native_at_o0_and_o2() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("host-values-native");
    std::fs::create_dir_all(&dir).unwrap();
    for opt in ["-O0", "-O2"] {
        let exe = dir.join(format!("host{opt}"));
        success(
            Command::new(env!("CARGO_BIN_EXE_jaic"))
                .arg(fixture())
                .args([opt, "-exe"])
                .arg(&exe),
        );
        success(&mut Command::new(exe));
    }
}

#[cfg(feature = "llvm")]
#[test]
fn host_values_runs_as_a_real_wasi_command() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("host-values-wasi");
    std::fs::create_dir_all(&dir).unwrap();
    let module = dir.join("host.wasm");
    success(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(fixture())
            .args(["-os", "wasm", "-exe"])
            .arg(&module),
    );
    let runner = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tools/browser/run_foundations_wasm.mjs");
    success(Command::new("node").arg(runner).arg(module));
}

/// Targets the interpreter can run `main` for from this machine while the build is a cross
/// build (Windows programs call foreign procedures the host does not have).
fn interpretable_foreign_targets() -> Vec<(&'static str, &'static str)> {
    let host_os = match std::env::consts::OS {
        "macos" => "macos",
        "windows" => "windows",
        _ => "linux",
    };
    let host_cpu = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        _ => "x64",
    };
    let mut targets = Vec::new();
    for os in ["linux", "macos"] {
        for cpu in ["x64", "arm64"] {
            if (os, cpu) != (host_os, host_cpu) {
                targets.push((os, cpu));
            }
        }
    }
    targets
}

/// Run `main` of a fixture in the interpreter for every foreign target. The program reads what
/// the target compiler holds, so state that compile-time code left only in the host twin shows.
fn interpret_for_foreign_targets(name: &str) {
    for (os, cpu) in interpretable_foreign_targets() {
        let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(host_run_fixture(name))
            .args(["-interp", "-os", os, "-cpu", cpu])
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{name} for {os} {cpu}: {stderr}");
        assert_eq!(String::from_utf8_lossy(&output.stdout), "ok\n", "{name}");
    }
}

#[test]
fn a_no_reset_global_written_by_a_run_reaches_the_cross_built_program() {
    interpret_for_foreign_targets("cross-target-no-reset-global.jai");
}

#[test]
fn global_data_registered_by_a_run_reaches_the_cross_built_program() {
    interpret_for_foreign_targets("cross-target-global-data.jai");
}

#[test]
fn type_info_flags_set_by_a_run_reach_the_cross_built_program() {
    interpret_for_foreign_targets("cross-target-type-info-flags.jai");
}

#[test]
fn type_descriptors_in_a_run_result_are_the_targets_own() {
    interpret_for_foreign_targets("cross-target-type-info-result.jai");
}

#[test]
fn compile_time_file_system_and_cpu_detection_read_the_host_for_every_target() {
    // The compiler runs with the program's directory as its working directory, so the program is
    // copied next to the directories it makes.
    let original = host_run_fixture("cross-target-compile-time-files.jai");
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("cross-target-files");
    let source = directory.join("files.jai");
    for (os, cpu) in foreign_targets() {
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::copy(&original, &source).unwrap();
        success(
            Command::new(env!("CARGO_BIN_EXE_jaic"))
                .arg(&source)
                .args(["-check", "-os", os, "-cpu", cpu])
                .current_dir(&directory),
        );
        assert!(directory.join("cross-target-files/a/b").is_dir());
    }
}

#[test]
fn compile_time_code_in_a_target_only_file_loads_no_target_libraries() {
    // The fixture loads a file per OS; each declares a library no machine has and runs code
    // that allocates. The file of the target OS is not part of the host program, so its run stays
    // on the target compiler, which loads no native libraries of its own.
    for (os, cpu) in foreign_targets() {
        success(
            Command::new(env!("CARGO_BIN_EXE_jaic"))
                .arg(host_run_fixture("cross-target-only-files/main.jai"))
                .args(["-check", "-os", os, "-cpu", cpu]),
        );
    }
}

#[test]
fn a_run_in_a_branch_only_the_target_takes_stays_on_the_target() {
    for (os, cpu) in foreign_targets() {
        success(
            Command::new(env!("CARGO_BIN_EXE_jaic"))
                .arg(host_run_fixture("cross-target-only-branch.jai"))
                .args(["-check", "-os", os, "-cpu", cpu]),
        );
    }
}

#[test]
fn a_run_naming_a_member_only_the_target_exports_is_checked_on_the_target() {
    // `Check` exports its porting checks on Windows only. On another host the twin's `Check`
    // lacks them, and the run falls back to the target compiler, which runs the check.
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/compiler/rejections/check-array.jai"),
        )
        .args(["-check", "-os", "windows", "-cpu", "x64"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("fixed array used as a pointer in assignment"),
        "{stderr}"
    );
}

#[test]
fn a_target_only_foreign_procedure_is_unavailable_at_compile_time() {
    for (os, cpu) in foreign_targets() {
        let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(host_run_fixture("cross-target-only-files/call.jai"))
            .args(["-check", "-os", os, "-cpu", cpu])
            .output()
            .unwrap();
        assert!(!output.status.success(), "{os} {cpu}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("foreign procedure `target_only_call` is not available here"),
            "{os} {cpu}: {stderr}"
        );
    }
}
