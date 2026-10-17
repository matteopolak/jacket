//! Independently authored registration primitives; pending servicing stays separate.
use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/plugin-registration")
        .join(name)
}

fn jaic(name: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg(fixture(name))
        .args(args)
        .output()
        .unwrap()
}

fn expect_success(output: Output, expected: &str) {
    assert!(
        output.status.success(),
        "status: {:?}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
    assert!(output.stderr.is_empty());
}

#[test]
fn scoped_generated_imports_register_real_factories_in_one_ordered_run() {
    expect_success(
        jaic("staged.jai", &["-interp"]),
        "registered and initialized\n",
    );
}

#[test]
fn registrar_checks_order_epochs_lifetime_and_empty_requests_in_the_interpreter() {
    expect_success(
        jaic("protocol.jai", &["-interp"]),
        "registration protocol ok\n",
    );
}

#[test]
fn source_addition_does_not_allow_an_undeclared_continuation() {
    let output = jaic("undeclared-continuation.jai", &["-check"]);
    assert!(!output.status.success());
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(
        diagnostic.contains("unknown identifier `continuation`"),
        "{diagnostic}"
    );
}

#[cfg(feature = "llvm")]
#[test]
fn staged_factories_and_registrar_execute_with_native_o0_and_o2() {
    let compiler = PathBuf::from(env!("CARGO_BIN_EXE_jaic"));
    let target = compiler.parent().unwrap();
    let scratch = target.join(format!("plugin-registration-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    for optimization in ["-O0", "-O2"] {
        for (name, compile_output, run_output) in [
            ("staged.jai", "registered and initialized\n", ""),
            ("protocol.jai", "", "registration protocol ok\n"),
        ] {
            let executable = scratch.join(format!("{name}{optimization}"));
            let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
                .arg(fixture(name))
                .args([optimization, "-exe"])
                .arg(&executable)
                .output()
                .unwrap();
            expect_success(output, compile_output);
            expect_success(Command::new(&executable).output().unwrap(), run_output);
        }
    }
    std::fs::remove_dir_all(scratch).unwrap();
}
