//! Authored complete multi-file applications exercise the actual graph, binder and VM.
use jai_driver::{CompilerSession, PlatformCompilerSession};
use jai_modules::{GraphOptions, ModuleGraph};
use jai_platform::{HostServices, SharedVfs, SourceProvider, VfsHost, VfsHostLimits, VfsLimits};
use jai_sema::{BoundLibrary, BoundLibraryReadiness, PreparedLibrarySession, RuntimeInvocation};
use jai_types::{
    Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem, ScalarLayout,
};
use jai_vm::{CompilerEffects, ExecutionPhase, Outcome};
use std::path::Path;

const MAIN: &str = r#"
#import "POSIX";
#load "lib/measure.jai";
write_string::(s:string,to_standard_error:bool) #no_context #compiler;
MEASURED :: #run measure();
main :: () -> int #no_context { if #compile_time return -9; return MEASURED + measure(); }
"#;
const MEASURE: &str = r#"
measure :: () -> int #no_context {
    path: [5] u8 = .[100,97,116,97,0];
    mode: [3] u8 = .[114,98,0];
    stream := fopen(*path[0], *mode[0]);
    if stream == null return -1;
    if fseek(stream,0,2) != 0 return -2;
    length := ftello(stream);
    if fclose(stream) != 0 return -3;
    if #compile_time write_string("compiled\n",false);
    return length;
}
"#;
const STDIO: &str = r#"
FILE :: struct {};
fopen :: (path:*u8, mode:*u8)->*FILE #foreign libc;
fseek :: (stream:*FILE, offset:s64, origin:s32)->s32 #foreign libc;
ftello :: (stream:*FILE)->s64 #foreign libc;
fclose :: (stream:*FILE)->s32 #foreign libc;
libc :: #system_library "authored-virtual-stdio";
"#;
fn files() -> [(&'static str, Vec<u8>); 5] {
    [
        ("main.jai", MAIN.as_bytes().to_vec()),
        ("lib/measure.jai", MEASURE.as_bytes().to_vec()),
        ("modules/POSIX/module.jai", b"#load \"stdio.jai\";".to_vec()),
        ("modules/POSIX/stdio.jai", STDIO.as_bytes().to_vec()),
        ("data", vec![b'x'; 21]),
    ]
}
fn lp64() -> BuildTarget {
    BuildTarget {
        operating_system: OperatingSystem::MacOS,
        architecture: Architecture::Arm64,
        layout: LayoutPolicy::lp64(),
        byte_order: ByteOrder::Little,
    }
}
fn wasm32() -> BuildTarget {
    BuildTarget {
        operating_system: OperatingSystem::WebAssembly,
        architecture: Architecture::WebAssembly32,
        layout: LayoutPolicy::new(
            ScalarLayout::new(4, 4),
            [
                ScalarLayout::new(1, 1),
                ScalarLayout::new(2, 2),
                ScalarLayout::new(4, 4),
                ScalarLayout::new(8, 8),
            ],
            [ScalarLayout::new(4, 4), ScalarLayout::new(8, 8)],
            ScalarLayout::new(1, 1),
        )
        .unwrap(),
        byte_order: ByteOrder::Little,
    }
}
fn prepare(
    graph: &ModuleGraph,
    provider: &dyn SourceProvider,
    modules: &Path,
    target: BuildTarget,
    effects: &mut PlatformCompilerSession<'_>,
) -> BoundLibrary {
    let entry_path = provider
        .canonicalize(&modules.join("POSIX/module.jai"))
        .unwrap();
    let stdio_path = provider
        .canonicalize(&modules.join("POSIX/stdio.jai"))
        .unwrap();
    let entry = graph
        .modules()
        .iter()
        .find(|module| {
            graph
                .sources()
                .get(graph.file(module.entry()).unwrap().source())
                .unwrap()
                .path()
                == entry_path
        })
        .unwrap()
        .entry();
    let stdio = graph
        .files()
        .iter()
        .find(|file| graph.sources().get(file.source()).unwrap().path() == stdio_path)
        .unwrap()
        .id();
    let options = jai_sema::ResolveOptions {
        compiler: Some(jai_sema::CompilerBindingContext::from_graph_with_provider(
            graph,
            &[modules.to_owned()],
            effects.compiler().root(),
            provider,
        )),
        file_abi: Some(
            jai_sema::FileAbiBindingContext::from_selected_sources(
                graph,
                entry,
                stdio,
                None,
                target.clone(),
            )
            .unwrap(),
        ),
        target: Some(target),
        ..Default::default()
    };
    match PreparedLibrarySession::new(graph, &options)
        .unwrap()
        .drive_bound(effects)
    {
        BoundLibraryReadiness::Complete(bound) => *bound,
        BoundLibraryReadiness::Failed(error) => panic!("{}", error.render(graph.sources())),
        BoundLibraryReadiness::Pending(pending) => panic!(
            "actual source graph remains pending: {:?}",
            pending.dependencies
        ),
    }
}
fn run(
    graph: &ModuleGraph,
    bound: &BoundLibrary,
    target: &BuildTarget,
    effects: &mut PlatformCompilerSession<'_>,
) -> i128 {
    let declaration = graph
        .declarations()
        .iter()
        .find(|decl| graph.symbols().name(decl.name()) == "main")
        .unwrap();
    let entry = bound.library().procedure(declaration.id()).unwrap().id;
    let origin = bound
        .runtime_origin(
            effects.compiler().root(),
            entry,
            RuntimeInvocation::allocate(),
        )
        .unwrap();
    effects.set_source_origin(origin);
    let execution = jai_vm::Vm::new_with_execution_phase(
        bound,
        &mut *effects,
        Default::default(),
        jai_vm::ByteTarget::from(target),
        ExecutionPhase::Runtime,
    )
    .unwrap()
    .execute(entry, vec![]);
    match execution.outcome {
        Outcome::Complete(values) => values[0].integer().unwrap().value(),
        other => panic!("runtime host execution failed: {other:?}"),
    }
}
#[test]
fn authored_virtual_file_graph_runs_with_real_32_and_64_bit_targets_and_console_commit() {
    for target in [lp64(), wasm32()] {
        let vfs = SharedVfs::new("/jai-script", VfsLimits::default()).unwrap();
        for (name, bytes) in files() {
            vfs.insert(name, bytes).unwrap();
        }
        let snapshot = vfs.snapshot().unwrap();
        let graph = ModuleGraph::load_with_target(
            Path::new("/jai-script/main.jai"),
            GraphOptions {
                import_dirs: vec!["/jai-script/modules".into()],
            },
            &snapshot,
            target.clone(),
        )
        .unwrap();
        let mut host =
            VfsHost::new(vfs, Path::new(""), false, true, VfsHostLimits::default()).unwrap();
        {
            let mut effects = PlatformCompilerSession::new(CompilerSession::new(), &mut host);
            let bound = prepare(
                &graph,
                &snapshot,
                Path::new("/jai-script/modules"),
                target.clone(),
                &mut effects,
            );
            assert_eq!(run(&graph, &bound, &target, &mut effects), 42);
            assert_eq!(run(&graph, &bound, &target, &mut effects), 42);
        }
        let output = host.take_console();
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].bytes, b"compiled\n");
    }
}
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_adapter_and_vfs_execute_the_same_authored_application() {
    use jai_driver::host_io::{HostIo, NativeHostServices, OriginalInputPolicy};
    use jai_platform::native::NativeSourceSnapshot;
    let directory = std::env::temp_dir().join(format!(
        "jai-platform-source-{:?}",
        jai_vm::host_effects::HostRequestKey::allocate()
    ));
    std::fs::create_dir(&directory).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(directory.clone());
    for (name, bytes) in files() {
        let path = directory.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }
    let source = NativeSourceSnapshot::default();
    let modules = directory.join("modules");
    let target = lp64();
    let graph = ModuleGraph::load_with_target(
        &directory.join("main.jai"),
        GraphOptions {
            import_dirs: vec![modules.clone()],
        },
        &source,
        target.clone(),
    )
    .unwrap();
    let mut native = HostIo::default();
    OriginalInputPolicy::from_trusted_inventory(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
    )
    .unwrap()
    .apply(&mut native)
    .unwrap();
    let root = native.register_root(&directory, false).unwrap();
    let mut host =
        NativeHostServices::new(native, root, Path::new(""), VfsHostLimits::default()).unwrap();
    {
        let mut effects = PlatformCompilerSession::new(CompilerSession::new(), &mut host);
        let bound = prepare(&graph, &source, &modules, target.clone(), &mut effects);
        assert_eq!(run(&graph, &bound, &target, &mut effects), 42);
        assert_eq!(run(&graph, &bound, &target, &mut effects), 42);
    }
    assert_eq!(host.take_console()[0].bytes, b"compiled\n");
}
