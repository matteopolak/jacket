//! Authored source proves the source VM table and genuine native publication.
#[path = "support/native_tools.rs"]
mod native_tools;
use inkwell::values::AnyValue;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

const COMPILER: &str = r#"
Global_Data_Segment_Info :: struct {
    segment_tag: enum u16 { BSS::0; DATA::1; RDATA::2; NO_RESET::3; USER::5; };
    data: []u8;
}
Global_Data_Info :: struct { version_stamp:u64; segment_info:[]Global_Data_Segment_Info; }
Runtime_Info :: struct { type_table:[]*Type_Info; global_data_info:*Global_Data_Info; }
runtime_catalog :: (w:s64=-1) -> Runtime_Info #compiler "get_runtime_info" {
    fresh_program_catalog: Runtime_Info #elsewhere;
    return fresh_program_catalog;
}
"#;

const SOURCE: &str = r#"
#import "Compiler";
Pair :: struct { byte:u8; value:s64; }
owned: u64 = 37;
unprovided_foreign: u64 #elsewhere;
verify :: () -> int {
    info := runtime_catalog();
    pair := type_info(Pair);
    pointer := type_info(*Pair);
    integer := type_info(s32);
    if pair.runtime_size != 16 return 1;
    if pair.members.count != 2 return 2;
    if pair.members[1].offset_in_bytes != 8 return 3;
    if pointer.pointer_to != cast(*Type_Info)pair return 4;
    if !integer.signed return 5;
    found_pair := false;
    found_pointer := false;
    found_integer := false;
    for row: info.type_table {
        if row == cast(*Type_Info)pair found_pair = true;
        if row == cast(*Type_Info)pointer found_pointer = true;
        if row == cast(*Type_Info)integer found_integer = true;
    }
    if !found_pair || !found_pointer || !found_integer return 6;
    if #compile_time {
        if info.global_data_info != null return 7;
    } else {
        if info.global_data_info == null return 8;
        if info.global_data_info.version_stamp != 1 return 9;
        found_owned := false;
        for segment: info.global_data_info.segment_info {
            if cast(*void)segment.data.data == cast(*void)*owned {
                if segment.data.count != 8 return 10;
                if segment.segment_tag != .DATA return 11;
                found_owned = true;
            }
        }
        if !found_owned return 12;
    }
    return 42;
}
OBSERVED :: #run verify();
#assert OBSERVED == 42;
main :: () -> int { return verify(); }
"#;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-authored-runtime-info-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join("modules")).unwrap();
        fs::write(
            path.join("modules/module.jai"),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../stdlib/prelude/module.jai"
            )),
        )
        .unwrap();
        fs::write(path.join("modules/Compiler.jai"), COMPILER).unwrap();
        fs::write(path.join("main.jai"), SOURCE).unwrap();
        Self(path)
    }
    fn resolve(
        &self,
        target: &jai_codegen::target::NativeTarget,
    ) -> Result<jai_ir::Program, String> {
        let roots = vec![self.0.join("modules")];
        let graph = jai_modules::ModuleGraph::load_with_bootstrap(
            &self.0.join("main.jai"),
            jai_modules::GraphOptions {
                import_dirs: roots.clone(),
            },
            jai_modules::PreludeSource::Search,
            &jai_modules::Filesystem,
            Some(target.build_target().unwrap()),
        )
        .unwrap();
        let options = jai_sema::ResolveOptions {
            target: Some(target.build_target().unwrap()),
            layout: Some(target.layout_policy().unwrap()),
            compiler: Some(jai_sema::CompilerBindingContext::from_graph(
                &graph,
                &roots,
                jai_vm::WorkspaceId::from_raw(1).unwrap(),
            )),
            ..Default::default()
        };
        jai_sema::resolve_graph_with_options(&graph, &options, &mut jai_vm::NoEffects)
            .map_err(|error| error.render(graph.sources()))
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn source_vm_and_native_table_share_typed_descriptor_identity_and_owned_ranges() {
    let fixture = Fixture::new();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let program = fixture.resolve(&target).unwrap();
    assert_eq!(program.library().native_runtime_info().len(), 1);
    let publication = &program.library().native_runtime_info()[0];
    assert_eq!(publication.data().symbol(), "fresh_program_catalog");
    assert!(publication.snapshot().represented_types().len() > 3);
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
    let binding = module.get_global("fresh_program_catalog").unwrap();
    assert!(binding.get_initializer().is_some());
    assert!(
        module
            .get_global("unprovided_foreign")
            .unwrap()
            .get_initializer()
            .is_none()
    );
    let native_info = module
        .get_global(&format!(
            "jai.runtime.global-data.{}",
            publication.global().index()
        ))
        .unwrap();
    assert!(native_info.get_initializer().is_some());
    let native_segments = module
        .get_global(&format!(
            "jai.runtime.global-segments.{}",
            publication.global().index()
        ))
        .unwrap();
    let owned_count = module
        .get_globals()
        .filter(|global| global.get_initializer().is_some())
        .count();
    assert_eq!(
        native_segments.get_value_type().into_array_type().len() as usize,
        owned_count
    );
    assert!(
        !native_segments
            .get_initializer()
            .unwrap()
            .print_to_string()
            .to_string()
            .contains("@unprovided_foreign")
    );
    let llvm = fixture.0.join("program.ll");
    fs::write(&llvm, module.print_to_string().to_bytes()).unwrap();
    for optimization in ["-O0", "-O2"] {
        let executable = fixture.0.join(format!("program{}", optimization));
        let linked = native_tools::clang_command()
            .arg(&llvm)
            .arg(optimization)
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            linked.status.success(),
            "{}",
            String::from_utf8_lossy(&linked.stderr)
        );
        let mut child = Command::new(&executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert_eq!(status.code(), Some(42), "{optimization}");
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("authored runtime-info fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

#[test]
fn runtime_catalog_cannot_adopt_owned_zero_file_external_or_extra_read() {
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    for invalid in [
        COMPILER.replace(
            "fresh_program_catalog: Runtime_Info #elsewhere;",
            "fresh_program_catalog: Runtime_Info;",
        ),
        COMPILER
            .replace("    fresh_program_catalog: Runtime_Info #elsewhere;", "")
            .replace(
                "runtime_catalog ::",
                "fresh_program_catalog: Runtime_Info #elsewhere;\nruntime_catalog ::",
            ),
        COMPILER.replace(
            "    return fresh_program_catalog;",
            "    ignored := fresh_program_catalog;\n    return fresh_program_catalog;",
        ),
    ] {
        let fixture = Fixture::new();
        fs::write(fixture.0.join("modules/Compiler.jai"), invalid).unwrap();
        let error = fixture.resolve(&target).unwrap_err();
        assert!(
            error.contains("runtime-info")
                || error.contains("runtime info")
                || error.contains("fallback"),
            "{error}"
        );
    }
}
