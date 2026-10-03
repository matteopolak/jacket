//! Independently authored Thread-shaped field overlays, with VM/native agreement.
#![cfg(target_pointer_width = "64")]

#[path = "support/native_tools.rs"]
mod native_tools;

use jai_codegen::{
    optimization::Optimization,
    target::{NativeTarget, TargetOptions},
    types::TypeLowerer,
};
use jai_types::{BitcodeOptimization, TypeKind};
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
            "jai-thread-placement-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn check(&self, source: &str, expected_layout: (u64, u32, &[u64])) {
        let input = self.0.join("main.jai");
        fs::write(&input, source).unwrap();
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
            let execution = jai_vm::execute(&program, jai_vm::Limits::default());
            assert!(
                matches!(execution.outcome, jai_vm::Outcome::Complete(ref values)
                    if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 42)),
                "VM placement result: {execution:?}"
            );
            let context = jai_codegen::Context::create();
            let mut types = TypeLowerer::with_target(&context, program.types(), &target.data);
            let mut matching_layouts = 0;
            for (id, kind) in program.types().iter() {
                if !matches!(kind, TypeKind::Record(_)) {
                    continue;
                }
                let layout = types.verify_layout(id, &target.data).unwrap();
                if (layout.size, layout.alignment, layout.field_offsets.as_ref()) == expected_layout
                {
                    matching_layouts += 1;
                }
            }
            assert_eq!(
                matching_layouts, 1,
                "missing exact Thread-shaped physical layout"
            );
            let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
            let object = self.0.join("program.o");
            let executable = self.0.join("program");
            target.write_object(&module, &object).unwrap();
            let mut command = native_tools::clang_command();
            command.arg(&object).arg("-o").arg(&executable);
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
                .current_dir(&self.0)
                .spawn()
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                if let Some(status) = child.try_wait().unwrap() {
                    assert_eq!(status.code(), Some(42), "{bitcode:?}\n{source}");
                    break;
                }
                if Instant::now() >= deadline {
                    child.kill().unwrap();
                    child.wait().unwrap();
                    panic!("fresh placement executable exceeded deadline");
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
fn thread_worker_padding_overlays_info_and_keeps_slice_after_cache_line() {
    Fixture::new().check(
        r#"
Unpadded :: struct { current:s64; next:s64; }
Worker :: struct {
    using info:Unpadded;
    #place info;
    padding:[64]u8=---;
    work_steal_indices:[]s32;
}
main :: ()->int {
    worker:Worker=---;
    worker.current=20; worker.next=22;
    worker.padding[63]=9;
    indices:[2]s32=.[3,5];
    worker.work_steal_indices=indices;
    copied:Worker=worker;
    worker.current=1;
    view:=cast(*Unpadded) *copied.padding;
    if view.current!=20 || view.next!=22 || copied.padding[63]!=9 return 1;
    if copied.work_steal_indices.count!=2 || copied.work_steal_indices[1]!=5 return 2;
    view.current=21; copied.next=21;
    if size_of(Worker)!=80 return 3;
    return copied.current+view.next;
}
"#,
        (80, 8, &[0, 0, 64]),
    );
}

#[test]
fn placement_cursor_rewinds_but_preserves_full_record_extent_on_copy() {
    Fixture::new().check(
        r#"
Owner :: struct {
    original:[79]u8;
    #place original;
    overlay:[16]u8=---;
    tail:u64;
}
main :: ()->int {
    owner:Owner=---;
    owner.original[78]=22;
    owner.overlay[0]=20;
    owner.tail=77;
    copied:Owner=owner;
    owner.original[78]=1;
    if copied.original[0]!=20 || copied.overlay[0]!=20 || copied.tail!=77 return 1;
    if size_of(Owner)!=80 return 2;
    return cast(int) copied.original[0]+cast(int) copied.original[78];
}
"#,
        (80, 8, &[0, 0, 16]),
    );
}
