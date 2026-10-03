//! Authored seeds use the exact same entrypoints as coverage-guided campaigns.
use jai_fuzz::*;

#[test]
fn authored_lexer_and_parser_seeds() {
    for seed in [
        include_bytes!("../seeds/lexer_utf8/unicode.jai").as_slice(),
        include_bytes!("../seeds/lexer_utf8/herestring.jai").as_slice(),
        include_bytes!("../seeds/parser/aggregates.jai").as_slice(),
        include_bytes!("../seeds/parser/missing-delimiter.jai").as_slice(),
    ] {
        lexer_utf8(seed);
        parser(seed);
    }
    lexer_utf8(b"\xff\xfea\0\n\0");
    lexer_utf8(b"\xff\xfe\x00");
}

#[test]
fn authored_closed_module_and_host_refusal_seeds() {
    module_vfs(include_bytes!("../seeds/module_vfs/loaded-record.packet"));
    module_vfs(include_bytes!("../seeds/module_vfs/traversal.jai"));
    module_vfs(include_bytes!("../seeds/module_vfs/noeffects.jai"));
    module_vfs(include_bytes!("../seeds/module_vfs/foreign-refusal.jai"));
}

#[test]
fn closed_source_paths_and_foreign_calls_never_gain_host_authority() {
    use jai_runtime::{Error, MissingHostBinding, Options, Script, SourceBundle, SourceProvider};
    use std::path::Path;
    let mut bundle = SourceBundle::new(MAX_SOURCE_BYTES);
    bundle
        .insert("main.jai", b"main::()->int{return 42;}".to_vec())
        .unwrap();
    assert!(bundle.insert("../../escape.jai", vec![]).is_err());
    assert!(bundle.read(Path::new("/etc/passwd")).is_err());
    assert!(bundle.read(Path::new("missing.jai")).is_err());
    let source =
        std::str::from_utf8(include_bytes!("../seeds/module_vfs/foreign-refusal.jai")).unwrap();
    let script = Script::from_source(
        source,
        Options {
            target: jai_runtime::browser_target(),
            limits: limits(),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(matches!(
        script.run(&[]),
        Err(Error::HostBindingRequired(
            MissingHostBinding::ForeignProcedure(_)
        ))
    ));
    assert!(script.host_capabilities().is_empty());
}

#[test]
fn compiler_effect_refusal_retains_actual_source_span() {
    let mut bundle = jai_runtime::SourceBundle::new(MAX_SOURCE_BYTES);
    let path = bundle
        .insert(
            "main.jai",
            include_bytes!("../seeds/module_vfs/noeffects.jai").to_vec(),
        )
        .unwrap();
    let target = jai_runtime::browser_target();
    let graph = jai_modules::ModuleGraph::load_with_target(
        &path,
        Default::default(),
        &bundle,
        target.clone(),
    )
    .unwrap();
    let options = jai_sema::ResolveOptions {
        target: Some(target),
        compile_time_limits: limits(),
        compiler: Some(jai_sema::CompilerBindingContext::from_graph(
            &graph,
            &[],
            jai_vm::WorkspaceId::from_raw(1).unwrap(),
        )),
        ..Default::default()
    };
    let error = jai_sema::resolve_graph_with_options(&graph, &options, &mut jai_vm::NoEffects)
        .err()
        .expect("NoEffects must refuse creating a compiler workspace");
    let source = graph
        .sources()
        .get(error.location.source)
        .expect("refusal must refer to an actual admitted source");
    let span = error.location.span;
    assert!(span.start <= span.end && span.end <= source.text().len());
    assert!(source.text().is_char_boundary(span.start) && source.text().is_char_boundary(span.end));
    assert!(
        error
            .render(graph.sources())
            .contains("compiler effects are unavailable")
    );
}

#[test]
fn structured_constant_and_checked_ir_seeds() {
    for seed in [
        include_bytes!("../seeds/constant_sema/alternating.bin").as_slice(),
        include_bytes!("../seeds/checked_ir_vm/copy-and-xor.bin").as_slice(),
        b"",
        &[255; 64],
        &[0; 64],
    ] {
        constant_sema(seed);
        checked_ir_vm(seed);
    }
}

#[test]
fn oversize_and_conservative_recursion_inputs_return_within_harness_budget() {
    let large = vec![b'x'; MAX_SOURCE_BYTES + 1];
    lexer_utf8(&large);
    parser(&large);
    module_vfs(&large);
    let nested = format!(
        "main::()->int{{return {}0{};}}",
        "(".repeat(128),
        ")".repeat(128)
    );
    parser(nested.as_bytes());
    module_vfs(nested.as_bytes());
}
