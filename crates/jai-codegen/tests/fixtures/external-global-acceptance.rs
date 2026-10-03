//! Source-to-native data access, with explicit bounded VM rejection.
#![cfg(all(
    target_pointer_width = "64",
    any(target_os = "macos", target_os = "linux")
))]

#[path = "../support/native_tools.rs"]
mod native_tools;

use jai_codegen::{
    native_reachability::Reachable,
    optimization::Optimization,
    target::{NativeTarget, TargetOptions},
};
use jai_ir::{ExternalDataId, ExternalDataSource, ForeignLibraryKind, GlobalInitializer};
use jai_types::BitcodeOptimization;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-external-data-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn check(&self, source: &str, library: bool, provided: bool) {
        let input = self.0.join("main.jai");
        fs::write(&input, source).unwrap();
        let archive = self.0.join("provider.a");
        if provided {
            let provider = self.0.join("provider.c");
            let object = self.0.join("provider.o");
            fs::write(&provider, include_str!("external-global-provider.c")).unwrap();
            let output = native_tools::clang_command()
                .args(["-std=c11", "-Wall", "-Wextra", "-Werror", "-c"])
                .arg(&provider)
                .arg("-o")
                .arg(&object)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            // Fixed installed tool; the archive contains only this fixture's fresh C object.
            let output = Command::new("/usr/bin/ar")
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .arg("rcs")
                .arg(&archive)
                .arg(&object)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let graph = jai_modules::ModuleGraph::load(&input, Default::default()).unwrap();
        for bitcode in [BitcodeOptimization::O0, BitcodeOptimization::O2] {
            let target = NativeTarget::select(&TargetOptions {
                optimization: Optimization {
                    bitcode,
                    ..Default::default()
                },
                ..Default::default()
            })
            .unwrap();
            let program = jai_sema::resolve_graph_with_options(
                &graph,
                &jai_sema::ResolveOptions {
                    target: Some(target.build_target().unwrap()),
                    layout: Some(target.layout_policy().unwrap()),
                    ..Default::default()
                },
                &mut jai_vm::NoEffects,
            )
            .unwrap_or_else(|error| panic!("{}", error.render(graph.sources())));
            let reachable = Reachable::executable(program.library(), program.entry()).unwrap();
            let external: Vec<_> = program
                .globals()
                .iter()
                .filter_map(|global| {
                    if let GlobalInitializer::External(data) = global.initializer() {
                        Some((global, data))
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(
                external.len(),
                if provided {
                    2
                } else {
                    1
                }
            );
            for (global, data) in &external {
                assert!(matches!(data.id(), ExternalDataId::File(_)));
                let record = graph.sources().get(data.location().source).unwrap();
                assert_eq!(
                    record.path().canonicalize().unwrap(),
                    input.canonicalize().unwrap()
                );
                assert!(data.location().span.start < data.location().span.end);
                assert_eq!(reachable.contains_global(global.id()), provided);
                match data.source() {
                    ExternalDataSource::Program => assert!(!library),
                    ExternalDataSource::Library(binding) => {
                        assert!(library);
                        let ForeignLibraryKind::Local {
                            path,
                        } = &binding.kind
                        else {
                            panic!("authored local archive became a different provider kind");
                        };
                        assert_eq!(
                            path.canonicalize().unwrap(),
                            archive.canonicalize().unwrap()
                        );
                        assert!(
                            program
                                .library()
                                .foreign_libraries()
                                .iter()
                                .any(|registered| registered == binding)
                        );
                    }
                }
            }
            let execution = jai_vm::execute(&program, jai_vm::Limits::default());
            if provided {
                assert!(
                    matches!(execution.outcome,
                    jai_vm::Outcome::Failed(jai_vm::Error::UnsupportedExternalGlobal(id))
                    if external.iter().any(|(global, _)| global.id() == id)),
                    "{execution:?}"
                );
            } else {
                assert!(
                    matches!(execution.outcome, jai_vm::Outcome::Complete(ref values)
                    if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 42)),
                    "{execution:?}"
                );
            }
            let mut vm =
                jai_vm::Vm::new(&program, jai_vm::NoEffects, jai_vm::Limits::default()).unwrap();
            assert_eq!(
                vm.memory().allocation_count(),
                0,
                "a fresh VM eagerly allocated external storage"
            );
            if !provided {
                // Implicit entry calls lazily allocate their genuine context.
                // Establish that separate storage before measuring externs.
                let context = program.library().context().unwrap();
                let initialized = vm.evaluate(&jai_ir::ValueExpr::Context {
                    ty: context.record_type,
                });
                assert!(
                    matches!(initialized.outcome, jai_vm::Outcome::Complete(ref values)
                    if values.len() == 1),
                    "{initialized:?}"
                );
                assert!(vm.memory().allocation_count() > 0);
            }
            let before = vm.memory().allocation_count();
            let entry = match program.entry() {
                jai_ir::EntryPoint::Int(id) | jai_ir::EntryPoint::Void(id) => id,
            };
            let progress = vm.start_resumable_procedure(entry, vec![]);
            if provided {
                assert!(
                    matches!(progress.outcome,
                    jai_vm::ResumableOutcome::Failed(jai_vm::Error::UnsupportedExternalGlobal(id))
                    if external.iter().any(|(global, _)| global.id() == id)),
                    "{progress:?}"
                );
            } else {
                assert_eq!(
                    progress.outcome,
                    jai_vm::ResumableOutcome::AwaitingPublication
                );
                assert!(
                    matches!(vm.resumable_values().unwrap(), [jai_vm::Value::Int(value)] if value.value() == 42)
                );
            }
            assert_eq!(
                vm.memory().allocation_count(),
                before,
                "external storage was allocated"
            );
            let context = jai_codegen::Context::create();
            let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
            for (_, data) in &external {
                let declaration = module.get_global(data.symbol()).unwrap();
                assert!(
                    declaration.get_initializer().is_none(),
                    "external data received an initializer"
                );
            }
            let object = self.0.join("program.o");
            let executable = self.0.join("program");
            target.write_object(&module, &object).unwrap();
            let mut command = native_tools::clang_command();
            command.arg(&object);
            if provided {
                command.arg(&archive);
            }
            command.arg("-o").arg(&executable);
            if cfg!(target_os = "macos") {
                command.arg("-Wl,-no_fixup_chains");
            }
            let output = command.output().unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let mut child = Command::new(&executable)
                .env_clear()
                .current_dir(&self.0)
                .spawn()
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                if let Some(status) = child.try_wait().unwrap() {
                    assert_eq!(status.code(), Some(42), "{bitcode:?}");
                    break;
                }
                if Instant::now() >= deadline {
                    child.kill().unwrap();
                    child.wait().unwrap();
                    panic!("fresh external-data executable exceeded deadline");
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn program_external_scalar_and_aggregate_share_native_provider_storage() {
    Fixture::new().check(include_str!("external-globals-program.jai"), false, true);
}
#[test]
fn library_external_data_preserves_canonical_provider_identity() {
    Fixture::new().check(include_str!("external-globals-library.jai"), true, true);
}
#[test]
fn unused_external_data_needs_no_vm_value_or_native_provider() {
    Fixture::new().check(include_str!("external-globals-unused.jai"), false, false);
}
