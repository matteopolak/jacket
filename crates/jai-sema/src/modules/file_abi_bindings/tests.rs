use super::*;
use std::fs;
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "jai-allocator-receipt-{}-{:?}",
            std::process::id(),
            jai_vm::host_effects::HostRequestKey::allocate()
        ));
        fs::create_dir_all(root.join("modules/Default_Allocator")).unwrap();
        fs::write(
            root.join("main.jai"),
            "#import \"Default_Allocator\"; main::()->s32 #no_context{return 0;}\n",
        )
        .unwrap();
        fs::write(
            root.join("modules/Default_Allocator/module.jai"),
            "c_malloc::(size:u64)->*void #foreign crt \"malloc\"; crt::#system_library \"libc\";\n",
        )
        .unwrap();
        Self(root)
    }
    fn graph(&self) -> ModuleGraph {
        ModuleGraph::load(
            &self.0.join("main.jai"),
            jai_modules::GraphOptions {
                import_dirs: vec![self.0.join("modules")],
            },
        )
        .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn target() -> BuildTarget {
    BuildTarget {
        operating_system: OperatingSystem::MacOS,
        architecture: Architecture::Arm64,
        layout: jai_types::LayoutPolicy::lp64(),
        byte_order: ByteOrder::Little,
    }
}
#[test]
fn allocator_role_receipt_retains_actual_declaration_without_stdio() {
    let fixture = Fixture::new();
    let graph = fixture.graph();
    let context =
        FileAbiBindingContext::from_graph(&graph, &[fixture.0.join("modules")], target()).unwrap();
    assert!(context.stdio.is_none());
    let source = context.allocator.first().unwrap();
    assert!(matches_receipt(&graph, &source.entry));
    assert_eq!(source.roles.len(), 1);
    let (declaration, role) = source.roles[0];
    assert_eq!(role, jai_vm::heap_abi::HeapAbiOperation::Malloc);
    assert_eq!(
        graph.declaration(declaration).unwrap().file(),
        source.entry.file
    );
    let reloaded = fixture.graph();
    // Unit IDs belong to each graph's source registry; the retained immutable
    // source allocation distinguishes separate loads even when IDs are equal.
    assert_eq!(graph.unit(), context.unit);
    assert!(!matches_receipt(&reloaded, &source.entry));
}
#[test]
fn allocator_role_requires_selected_import_root_and_supported_target() {
    let fixture = Fixture::new();
    let graph = fixture.graph();
    assert!(FileAbiBindingContext::from_graph(&graph, &[], target()).is_none());
    let mut unsupported = target();
    unsupported.byte_order = ByteOrder::Big;
    assert!(
        FileAbiBindingContext::from_graph(&graph, &[fixture.0.join("modules")], unsupported)
            .is_none()
    );
}

#[test]
fn readiness_publishes_only_original_headers_after_they_are_checked() {
    let fixture = Fixture::new();
    let graph = fixture.graph();
    let context =
        FileAbiBindingContext::allocator_from_graph(&graph, &[fixture.0.join("modules")], target())
            .unwrap();
    let mut types = TypeRegistry::new();
    let aliases = crate::polymorphism::integration::callable_aliases(&graph);
    let source_procedures = procedure_headers::identities::reserve(&graph, &aliases).unwrap();
    let nominals = Nominals::reserve(&graph, &mut types).unwrap();
    let mut declarations = ScopedDeclarations {
        context: None,
        graph: &graph,
        values: HashMap::new(),
        signatures: HashMap::new(),
        callable_aliases: aliases,
        generics: std::cell::RefCell::new(crate::polymorphism::integration::GenericContext::new(
            source_procedures.len(),
        )),
        source_procedures,
        source_lookup: Default::default(),
        nominals,
        defaults: HashMap::new(),
    };
    assert!(
        bind_heap_ready(
            &graph,
            &types,
            &declarations,
            Some(&context),
            Some(&target())
        )
        .unwrap()
        .is_empty()
    );
    assert!(
        bind_heap(
            &graph,
            &types,
            &declarations,
            Some(&context),
            Some(&target())
        )
        .is_err()
    );
    let mut wrong_target = target();
    wrong_target.architecture = Architecture::X86_64;
    assert!(
        bind_heap_ready(
            &graph,
            &types,
            &declarations,
            Some(&context),
            Some(&wrong_target)
        )
        .unwrap_err()
        .to_string()
        .contains("receipt differs")
    );
    let mut constants = Constants::new(&graph);
    let mut meta = crate::reflection::MetaContext::default();
    procedure_headers::register(
        &graph,
        &mut types,
        &mut declarations,
        &mut constants,
        &mut meta,
        procedure_headers::HeaderPhase::TypesOnly,
    )
    .unwrap();
    let bound = bind_heap_ready(
        &graph,
        &types,
        &declarations,
        Some(&context),
        Some(&target()),
    )
    .unwrap();
    assert_eq!(bound.len(), 1);
    let original = context.allocator.first().unwrap().roles[0].0;
    let signature = &declarations.signatures[&original];
    let capability = bound[&signature.id];
    assert_eq!(capability.procedure(), signature.id);
    assert_eq!(capability.signature, signature.ty);
}

#[test]
fn stdio_readiness_waits_for_its_actual_reserved_file_definition() {
    let fixture = Fixture::new();
    let stdio = fixture.0.join("modules/POSIX/bindings/macos/arm64");
    fs::create_dir_all(&stdio).unwrap();
    fs::write(
        fixture.0.join("main.jai"),
        "#import \"POSIX\"; #import \"Default_Allocator\"; main::()->s32 #no_context{return 0;}\n",
    )
    .unwrap();
    fs::write(
        fixture.0.join("modules/POSIX/module.jai"),
        "#load \"bindings/macos/arm64/stdio.jai\";\n",
    )
    .unwrap();
    fs::write(
        stdio.join("stdio.jai"),
        "FILE::struct{opaque:s64;} fclose::(file:*FILE)->s32 #foreign stdio_crt \"fclose\"; stdio_crt::#system_library \"libc\";\n",
    )
    .unwrap();
    let graph = fixture.graph();
    let context =
        FileAbiBindingContext::from_graph(&graph, &[fixture.0.join("modules")], target()).unwrap();
    let mut types = TypeRegistry::new();
    let aliases = crate::polymorphism::integration::callable_aliases(&graph);
    let source_procedures = procedure_headers::identities::reserve(&graph, &aliases).unwrap();
    let nominals = Nominals::reserve(&graph, &mut types).unwrap();
    let mut declarations = ScopedDeclarations {
        context: None,
        graph: &graph,
        values: HashMap::new(),
        signatures: HashMap::new(),
        callable_aliases: aliases,
        generics: std::cell::RefCell::new(crate::polymorphism::integration::GenericContext::new(
            source_procedures.len(),
        )),
        source_procedures,
        source_lookup: Default::default(),
        nominals,
        defaults: HashMap::new(),
    };
    let mut constants = Constants::new(&graph);
    let mut meta = crate::reflection::MetaContext::default();
    procedure_headers::register(
        &graph,
        &mut types,
        &mut declarations,
        &mut constants,
        &mut meta,
        procedure_headers::HeaderPhase::TypesOnly,
    )
    .unwrap();
    let stdio_source = &context.stdio.as_ref().unwrap().declarations;
    let file_declaration = graph
        .declarations()
        .iter()
        .find(|declaration| {
            declaration.file() == stdio_source.file
                && graph.symbols().name(declaration.name()) == "FILE"
        })
        .unwrap();
    let original_file = declarations.nominals.declarations[&file_declaration.id()];
    assert!(matches!(
        types.record_definition(original_file),
        Err(jai_types::TypeError::Incomplete(id)) if id == original_file
    ));
    assert!(
        bind_ready(
            &graph,
            &types,
            &declarations,
            Some(&context),
            Some(&target())
        )
        .unwrap()
        .is_empty()
    );
    assert!(
        bind(
            &graph,
            &types,
            &declarations,
            Some(&context),
            Some(&target())
        )
        .unwrap_err()
        .to_string()
        .contains("Incomplete")
    );
    let mut stale = context.clone();
    let reloaded = fixture.graph();
    let reloaded_context =
        FileAbiBindingContext::from_graph(&reloaded, &[fixture.0.join("modules")], target())
            .unwrap();
    stale.allocator.first_mut().unwrap().entry =
        reloaded_context.allocator.first().unwrap().entry.clone();
    assert!(
        bind_ready(&graph, &types, &declarations, Some(&stale), Some(&target()))
            .unwrap_err()
            .to_string()
            .contains("receipt differs")
    );
    declarations
        .nominals
        .define_records(&graph, &mut types, &mut |file, expression| {
            constants.evaluate(file, expression)
        })
        .unwrap();
    let bound = bind_ready(
        &graph,
        &types,
        &declarations,
        Some(&context),
        Some(&target()),
    )
    .unwrap();
    assert_eq!(bound.len(), 1);
    assert_eq!(bound.values().next().unwrap().file_type(), original_file);
}
