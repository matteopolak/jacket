//! Source-to-VM-to-native agreement using only newly generated executables.
#[path = "native_tools.rs"]
mod native_tools;
use jai_modules::{GraphOptions, ModuleGraph};
use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

pub(super) fn check(source: &str, expected: i32) {
    check_execution(source, Some(expected));
}

pub(super) fn check_optimized(source: &str, expected: i32) {
    for optimization in ["-O0", "-O2"] {
        check_execution_vm(
            source,
            Some(expected),
            VmExpectation::NativeAgreement,
            None,
            Some(optimization),
        );
    }
}
pub(super) fn check_execution(source: &str, expected: Option<i32>) {
    check_execution_vm(source, expected, VmExpectation::NativeAgreement, None, None);
}

pub(super) fn check_vm_unsupported(source: &str, expected: i32) {
    check_execution_vm(
        source,
        Some(expected),
        VmExpectation::PointerBoundary,
        None,
        None,
    );
}

#[allow(
    dead_code,
    reason = "Shared suites select their required optimization runners."
)]
pub(super) fn check_vm_unsupported_optimized(source: &str, expected: i32) {
    for optimization in ["-O0", "-O2"] {
        check_execution_vm(
            source,
            Some(expected),
            VmExpectation::PointerBoundary,
            None,
            Some(optimization),
        );
    }
}

#[allow(
    dead_code,
    reason = "This shared harness is also included by suites without C support fixtures."
)]
pub(super) fn check_foreign_execution(source: &str, support: &str, expected: i32) {
    for optimization in ["-O0", "-O2"] {
        check_execution_vm(
            source,
            Some(expected),
            VmExpectation::ForeignBoundary,
            Some(support),
            Some(optimization),
        );
    }
}

#[derive(Clone, Copy)]
enum VmExpectation {
    NativeAgreement,
    PointerBoundary,
    ForeignBoundary,
}

fn check_execution_vm(
    source: &str,
    expected: Option<i32>,
    vm_expectation: VmExpectation,
    support: Option<&str>,
    optimization: Option<&str>,
) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Fixture(std::path::PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let fixture = Fixture(std::env::temp_dir().join(format!(
        "jai-pointer-native-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir_all(&fixture.0).unwrap();
    let path = fixture.0.join("main.jai");
    fs::write(&path, source).unwrap();
    let graph = ModuleGraph::load(&path, GraphOptions::default()).unwrap();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let options = jai_sema::ResolveOptions {
        target: Some(target.build_target().unwrap()),
        layout: Some(target.layout_policy().unwrap()),
        ..jai_sema::ResolveOptions::default()
    };
    let program =
        jai_sema::resolve_graph_with_options(&graph, &options, &mut jai_vm::NoEffects).unwrap();
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
    let llvm = module.print_to_string().to_string();
    let executable = fixture.0.join("program");
    let mut command = native_tools::clang_command();
    command.args(["-x", "ir", "-", "-o"]).arg(&executable);
    if let Some(support) = support {
        let support_path = fixture.0.join("support.c");
        fs::write(&support_path, support).unwrap();
        command.args(["-x", "c"]).arg(support_path);
    }
    if let Some(optimization) = optimization {
        command.arg(optimization);
    }
    let mut compiler = command
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    compiler
        .stdin
        .take()
        .unwrap()
        .write_all(llvm.as_bytes())
        .unwrap();
    let compiled = compiler.wait_with_output().unwrap();
    assert!(
        compiled.status.success(),
        "{}\n{llvm}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let mut native_command = Command::new(executable);
    if matches!(vm_expectation, VmExpectation::ForeignBoundary) {
        native_command.current_dir(&fixture.0);
    }
    let mut child = native_command.spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            match expected {
                Some(expected) => assert_eq!(
                    status.code(),
                    Some(expected),
                    "native pointer fixture outcome"
                ),
                None => assert!(!status.success(), "native invalid access must trap"),
            }
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("generated pointer fixture exceeded deadline");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
    match vm_expectation {
        VmExpectation::PointerBoundary => {
            assert!(
                matches!(
                    outcome,
                    jai_vm::Outcome::Failed(jai_vm::Error::UnsupportedPointerOperation(_))
                ),
                "VM must identify the pointer capability boundary: {outcome:?}"
            );
            return;
        }
        VmExpectation::ForeignBoundary => {
            assert!(
                matches!(
                    outcome,
                    jai_vm::Outcome::Failed(jai_vm::Error::UnsupportedForeignProcedure(_))
                ),
                "VM must preserve the foreign-call boundary: {outcome:?}"
            );
            return;
        }
        VmExpectation::NativeAgreement => {}
    }
    match expected {
        Some(expected) => {
            let jai_vm::Outcome::Complete(values) = outcome else {
                panic!("VM pointer fixture outcome: {outcome:?}")
            };
            assert_eq!(
                values[0].integer().unwrap().value(),
                i128::from(expected),
                "VM pointer fixture outcome after native agreement"
            );
        }
        None => assert!(
            matches!(outcome, jai_vm::Outcome::Failed(_)),
            "VM invalid access must fail: {outcome:?}"
        ),
    }
}
