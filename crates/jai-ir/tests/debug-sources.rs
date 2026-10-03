use jai_ir::*;
use jai_source::{SourceMap, SourceRecord, SourceSpan, Span};
use jai_types::{CallingConvention, ContextMode, ProcedureType, TypeRegistry, Variadic};

fn location(source: &SourceRecord) -> DebugSourceLocation {
    DebugSourceLocation::from_source(
        source,
        SourceSpan {
            source: source.id(),
            span: Span::new(0, source.text().len()),
        },
    )
    .unwrap()
}

fn publish(sources: DebugSources) -> Result<Library, IrError> {
    let mut types = TypeRegistry::new();
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    ProgramBuilder::new(types.freeze().unwrap())
        .debug_sources(sources)
        .procedures(vec![Procedure {
            id: ProcedureId::new(27),
            signature,
            parameters: vec![],
            locals: vec![],
            body: Block {
                statements: vec![],
                flow: Flow::FallsThrough,
            },
            cleanups: vec![],
        }])
        .finish_library()
}

#[test]
fn retained_source_snapshot_and_sparse_body_publish_together() {
    let mut map = SourceMap::default();
    let id = map.insert("input.jai".into(), "main :: () {}".into());
    let source = map.get(id).unwrap();
    let mut sources = DebugSources::default();
    sources.retain_source(source);
    sources.insert(
        ProcedureId::new(27),
        ProcedureSource {
            name: "main".into(),
            location: location(source),
        },
    );
    let library = publish(sources).unwrap();
    let debug = library.debug_sources().unwrap();
    assert_eq!(debug.procedure(ProcedureId::new(27)).unwrap().name, "main");
}

#[test]
fn another_source_map_cannot_substitute_same_path_and_coordinates() {
    let mut retained = SourceMap::default();
    let retained_id = retained.insert("input.jai".into(), "main :: () {}".into());
    let mut foreign = SourceMap::default();
    let foreign_id = foreign.insert("input.jai".into(), "faux :: () {}".into());
    assert_eq!(retained_id.index(), foreign_id.index());
    let mut sources = DebugSources::default();
    sources.retain_source(retained.get(retained_id).unwrap());
    sources.insert(
        ProcedureId::new(27),
        ProcedureSource {
            name: "main".into(),
            location: location(foreign.get(foreign_id).unwrap()),
        },
    );
    assert!(publish(sources).is_err());
}

#[test]
fn another_source_map_cannot_substitute_identical_text() {
    let mut retained = SourceMap::default();
    let retained_id = retained.insert("input.jai".into(), "main :: () {}".into());
    let mut foreign = SourceMap::default();
    let foreign_id = foreign.insert("input.jai".into(), "main :: () {}".into());
    let mut sources = DebugSources::default();
    sources.retain_source(retained.get(retained_id).unwrap());
    sources.insert(
        ProcedureId::new(27),
        ProcedureSource {
            name: "main".into(),
            location: location(foreign.get(foreign_id).unwrap()),
        },
    );
    assert!(publish(sources).is_err());
}

#[test]
fn suppression_keeps_original_provenance_and_checks_its_procedure_owner() {
    let mut map = SourceMap::default();
    let id = map.insert("input.jai".into(), "hidden :: () #no_debug {}".into());
    let original = map.get(id).unwrap();
    let mut sources = DebugSources::default();
    sources.retain_source(original);
    sources.insert(
        ProcedureId::new(27),
        ProcedureSource {
            name: "hidden".into(),
            location: location(original),
        },
    );
    sources.set_procedure_policy(ProcedureId::new(27), DebugPolicy::Suppress);
    let library = publish(sources.clone()).unwrap();
    let published = library.debug_sources().unwrap();
    assert_eq!(
        published.procedure_policy(ProcedureId::new(27)),
        DebugPolicy::Suppress
    );
    assert_eq!(
        published.procedure(ProcedureId::new(27)).unwrap().name,
        "hidden"
    );
    sources.set_procedure_policy(ProcedureId::new(28), DebugPolicy::Suppress);
    assert!(publish(sources).is_err());
}
