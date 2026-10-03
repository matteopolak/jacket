//! Source schema checks use an independently authored bootstrap and API declarations.
use super::*;
use jai_modules::{GraphOptions, PreludeSource, SourceOverlay};
use jai_types::{Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem};
use std::path::Path;

const SOURCE: &str = r#"
Global_Data_Segment_Info :: struct {
    segment_tag: enum u16 { BSS::0; DATA::1; RDATA::2; NO_RESET::3; USER::5; };
    data: []u8;
}
Global_Data_Info :: struct { version_stamp: u64; segment_info: []Global_Data_Segment_Info; }
Runtime_Info :: struct { type_table: []*Type_Info; global_data_info: *Global_Data_Info; }
get_runtime_info :: (w:s64 = -1) -> Runtime_Info #compiler;
"#;

fn check(source: &str) -> Result<Library, String> {
    let root = Path::new("/jai-runtime-info-source/main.jai");
    let mut provider = SourceOverlay::new();
    provider.insert(root, source.as_bytes().to_vec()).unwrap();
    provider
        .insert(
            Path::new("/jai-runtime-info-source/modules/module.jai"),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../stdlib/prelude/module.jai"
            ))
            .to_vec(),
        )
        .unwrap();
    let target = BuildTarget {
        operating_system: OperatingSystem::Linux,
        architecture: Architecture::X86_64,
        layout: LayoutPolicy::lp64(),
        byte_order: ByteOrder::Little,
    };
    let roots = vec!["/jai-runtime-info-source/modules".into()];
    let graph = ModuleGraph::load_with_bootstrap(
        root,
        GraphOptions {
            import_dirs: roots.clone(),
        },
        PreludeSource::Search,
        &provider,
        Some(target.clone()),
    )
    .unwrap();
    let options = crate::ResolveOptions {
        target: Some(target),
        compiler: Some(CompilerBindingContext::from_graph(
            &graph,
            &roots,
            workspace(),
        )),
        ..Default::default()
    };
    crate::resolve_library_with_options(&graph, &options, &mut jai_vm::NoEffects)
        .map_err(|error| error.render(graph.sources()))
}

#[test]
fn actual_runtime_info_shape_binds_without_an_invented_body_or_table() {
    let library = check(SOURCE).unwrap();
    assert!(
        library
            .prototypes()
            .iter()
            .any(|prototype| { matches!(prototype.origin, PrototypeOrigin::Compiler) })
    );
    assert!(library.procedures().is_empty());
}

#[test]
fn incompatible_source_runtime_global_and_segment_schemas_reject() {
    for source in [
        SOURCE.replace("type_table: []*Type_Info", "type_table: []Type_Info"),
        SOURCE.replace("version_stamp: u64", "version_stamp: s64"),
        SOURCE.replace("USER::5", "USER::4"),
        SOURCE.replace("segment_tag: enum u16", "segment_tag: enum u32"),
        SOURCE.replace("data: []u8", "data: []s8"),
        SOURCE.replace("w:s64 = -1", "w:u64 = 0"),
    ] {
        let error = check(&source).unwrap_err();
        assert!(
            error.contains("#compiler") || error.contains("default"),
            "{error}"
        );
    }
}
