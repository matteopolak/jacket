use jai_ir::{ForeignLibrarySources, ProgramBuilder};
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::NativeSourceLibraryBinding;
use jai_source::SourceTextSnapshot;
use jai_types::TypeRegistry;
use std::path::Path;

fn graph(snapshot: SourceTextSnapshot) -> ModuleGraph {
    let path = Path::new("/own-native-binding/main.jai");
    let mut provider = SourceOverlay::new();
    provider.insert_snapshot(path, snapshot).unwrap();
    ModuleGraph::load_with_provider(path, GraphOptions::default(), &provider).unwrap()
}

#[test]
fn authentic_retained_graph_source_matches_the_actual_frozen_row_only() {
    let snapshot =
        SourceTextSnapshot::new("api :: #library \"./never-open.a\"; main :: () {}".into());
    let first = graph(snapshot.clone());
    let declaration = first
        .declarations()
        .iter()
        .find(|decl| first.symbols().name(decl.name()) == "api")
        .unwrap();
    let binding = NativeSourceLibraryBinding::from_graph(&first, declaration.id()).unwrap();
    let library = jai_sema::resolve_library(&first).unwrap();
    let row = &library.foreign_libraries()[0];
    assert!(binding.matches_frozen(&library, row));
    assert!(NativeSourceLibraryBinding::from_frozen_library(&library, row).is_some());
    assert!(!binding.matches_frozen(&library, &row.clone()));
    assert!(NativeSourceLibraryBinding::from_frozen_library(&library, &row.clone()).is_none());

    let retained = graph(snapshot);
    let retained_library = jai_sema::resolve_library(&retained).unwrap();
    assert!(
        binding.can_rebase_to_frozen(&retained_library, &retained_library.foreign_libraries()[0])
    );
    let replaced = graph(SourceTextSnapshot::new(
        first
            .sources()
            .get(first.file(declaration.file()).unwrap().source())
            .unwrap()
            .text()
            .to_owned(),
    ));
    let replaced_library = jai_sema::resolve_library(&replaced).unwrap();
    assert!(
        !binding.can_rebase_to_frozen(&replaced_library, &replaced_library.foreign_libraries()[0])
    );
}

#[test]
fn source_owner_selection_never_bypasses_changed_options_or_environment() {
    let graph = graph(SourceTextSnapshot::new(
        "api :: #library \"./never-open.a\"; main :: () {}".into(),
    ));
    let original = jai_sema::resolve_library(&graph).unwrap();
    let row = &original.foreign_libraries()[0];
    let binding = NativeSourceLibraryBinding::from_frozen_library(&original, row).unwrap();
    let mut changed = row.clone();
    changed.options.no_dll = true;
    let changed_library = ProgramBuilder::new(TypeRegistry::new().freeze().unwrap())
        .foreign_libraries(vec![changed])
        .foreign_library_sources(original.foreign_library_sources().clone())
        .finish_library()
        .unwrap();
    let changed_row = &changed_library.foreign_libraries()[0];
    assert!(binding.owns_frozen_declaration(&changed_library, changed_row));
    assert!(!binding.can_rebase_to_frozen(&changed_library, changed_row));
    let altered_environment = ForeignLibrarySources::from_source_records([(
        row.id,
        original
            .foreign_library_sources()
            .get(row.id)
            .unwrap()
            .clone(),
        b"another actual reviewed environment".to_vec(),
    )])
    .unwrap();
    let changed_environment = ProgramBuilder::new(TypeRegistry::new().freeze().unwrap())
        .foreign_libraries(vec![row.clone()])
        .foreign_library_sources(altered_environment)
        .finish_library()
        .unwrap();
    assert!(binding.owns_frozen_declaration(
        &changed_environment,
        &changed_environment.foreign_libraries()[0]
    ));
    assert!(!binding.can_rebase_to_frozen(
        &changed_environment,
        &changed_environment.foreign_libraries()[0]
    ));
}
