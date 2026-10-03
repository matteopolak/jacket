//! Discarded using owners keep actual declaration identities without name bindings.
use jai_modules::{
    GraphDiscovery, GraphOptions, LookupError, SourceOverlay, UsingDeclarationSource,
};
use jai_syntax::NamePath;
use std::path::Path;

#[test]
fn repeated_discarded_nominal_owners_are_distinct_original_declarations() {
    let path = Path::new("/discarded-using/main.jai");
    let mut provider = SourceOverlay::new();
    provider
        .insert(
            path,
            b"using _ :: struct { FIRST :: 20; }\nusing _ :: struct { SECOND :: 22; }".to_vec(),
        )
        .unwrap();
    let mut discovery = GraphDiscovery::new(path, GraphOptions::default(), &provider).unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    let graph = discovery.graph();
    let file = graph.module(graph.root()).unwrap().entry();
    let discarded = graph.symbols().find("_").unwrap();
    assert!(matches!(
        graph.lookup(file, &NamePath { root: discarded, members: vec![] }),
        Err(LookupError::UnknownName(name)) if name == discarded
    ));
    assert!(
        !graph
            .module(graph.root())
            .unwrap()
            .bindings()
            .contains_key(&discarded)
    );
    assert!(
        !graph
            .module(graph.root())
            .unwrap()
            .exports()
            .contains_key(&discarded)
    );
    let requests = discovery.pending_using_requests();
    assert_eq!(requests.len(), 2);
    let owners = requests
        .iter()
        .map(|request| {
            let Some(UsingDeclarationSource::File(child)) = &request.declaration else {
                panic!("original file child is required");
            };
            let owner = graph.declaration_at(request.file, child.location).unwrap();
            assert_eq!(owner.location(), child.location);
            assert_eq!(owner.name(), discarded);
            assert!(child.location.span.start > request.location.span.start);
            owner.id()
        })
        .collect::<Vec<_>>();
    assert_ne!(owners[0], owners[1]);
    assert_eq!(graph.file(file).unwrap().declarations(), owners.as_slice());
}
