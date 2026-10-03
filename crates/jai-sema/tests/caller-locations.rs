//! Independently authored source defaults retain the actual caller's provenance.
use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_types::LayoutPolicy;
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

const LOCATION: &str = "Source_Code_Location :: struct { fully_pathed_filename: string; line_number: s64; character_number: s64; }\n";
struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "jai-caller-location-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("main.jai"), source).unwrap();
        Self(root)
    }
    fn graph(&self) -> ModuleGraph {
        ModuleGraph::load(&self.0.join("main.jai"), GraphOptions::default()).unwrap()
    }
    fn run(&self, target: bool) -> Result<i128, String> {
        let graph = self.graph();
        let program = resolve_graph_with_options(
            &graph,
            &ResolveOptions {
                layout: target.then_some(LayoutPolicy::lp64()),
                ..ResolveOptions::default()
            },
            &mut NoEffects,
        )
        .map_err(|error| error.render(graph.sources()))?;
        match jai_vm::execute(&program, Limits::default()).outcome {
            Outcome::Complete(values) => match values.as_slice() {
                [Value::Int(value)] => Ok(value.value()),
                _ => Err(format!("unexpected VM result: {values:?}")),
            },
            outcome => Err(format!("unexpected VM outcome: {outcome:?}")),
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn encoded_position(source: &str, call: &str) -> i128 {
    let position = source.rfind(call).unwrap();
    let prefix = &source[..position];
    let line = prefix.bytes().filter(|&byte| byte == b'\n').count() + 1;
    let column = prefix.rsplit('\n').next().unwrap().chars().count() + 1;
    (line * 1000 + column) as i128
}
fn assert_caller(header: &str, main: &str, call: &str) {
    let source = format!("{LOCATION}{header}\n{main}");
    let expected = encoded_position(&source, call);
    assert_eq!(
        Fixture::new(&source).run(true).unwrap(),
        expected,
        "{source}"
    );
}

#[test]
fn inferred_and_typed_defaults_measure_unicode_caller_columns() {
    for parameter in [
        "loc := #caller_location",
        "loc: Source_Code_Location = #caller_location",
    ] {
        assert_caller(
            &format!(
                "probe :: ({parameter})->int {{ return loc.line_number*1000+loc.character_number; }}"
            ),
            "main :: ()->int { text := \"é🦀\"; return probe(); }",
            "probe()",
        );
    }
}

#[test]
fn named_indirect_and_generic_calls_use_their_own_call_spans() {
    assert_caller(
        "probe :: (value: int = 1, loc := #caller_location)->int { return loc.line_number*1000+loc.character_number; }",
        "main :: ()->int { callback := probe; return callback(value=9); }",
        "callback(value=9)",
    );
    assert_caller(
        "probe :: (value: $T, loc := #caller_location)->int { return loc.line_number*1000+loc.character_number; }",
        "main :: ()->int { return probe(9); }",
        "probe(9)",
    );
}

#[test]
fn nested_defaults_use_the_nested_calls_source_span() {
    assert_caller(
        "",
        "main :: ()->int { probe :: (loc := #caller_location)->int { return loc.line_number*1000+loc.character_number; } return probe(); }",
        "probe()",
    );
}

#[test]
fn literal_source_location_reports_its_own_unicode_span() {
    let source = format!(
        "{LOCATION}main :: ()->int {{ text := \"é🦀\"; loc := #location(); return loc.line_number*1000+loc.character_number; }}"
    );
    assert_eq!(
        Fixture::new(&source).run(true).unwrap(),
        encoded_position(&source, "#location()")
    );
    assert!(
        Fixture::new(&source)
            .run(false)
            .unwrap_err()
            .contains("explicit target layout")
    );
    for main in [
        "main :: ()->int { return probe(#location()); }",
        "main :: ()->int { callback := probe; return callback(loc=#location()); }",
    ] {
        let source = format!(
            "{LOCATION}probe :: (loc: Source_Code_Location)->int {{ return loc.line_number*1000+loc.character_number; }} {main}"
        );
        assert_eq!(
            Fixture::new(&source).run(true).unwrap(),
            encoded_position(&source, "#location()")
        );
    }
}

#[test]
fn source_file_and_line_do_not_require_a_location_nominal() {
    let source = "main :: ()->int {\nfilename := #file;\nline := #line;\nif filename.count < 9 return 0;\nreturn line; }";
    let fixture = Fixture::new(source);
    let path = fs::canonicalize(fixture.0.join("main.jai")).unwrap();
    let source = format!(
        "main :: ()->int {{ expected := \"{}\"; filename := #file; if filename.count != expected.count return 0; for i: 0..expected.count-1 {{ if filename[i] != expected[i] return 0; }}\nreturn #line; }}",
        path.to_str().unwrap()
    );
    fs::write(&path, &source).unwrap();
    assert_eq!(fixture.run(false).unwrap(), 2);
}

#[test]
fn filepath_is_the_retained_directory_in_runtime_and_definition_defaults() {
    let parent = Fixture::new("");
    let directory = parent.0.join("dossier-é🦀");
    fs::create_dir(&directory).unwrap();
    let fixture = Fixture(directory);
    let expected = fs::canonicalize(&fixture.0).unwrap();
    let source = format!(
        "directory :: #filepath; probe :: (value: $T, path := #filepath)->string {{ return path; }} main :: ()->int {{ expected := \"{}\"; local :: #filepath; runtime := #filepath; baked := #run #filepath; supplied := probe(9, #filepath); defaulted := probe(9); if directory.count != expected.count || local.count != expected.count || runtime.count != expected.count || baked.count != expected.count || supplied.count != expected.count || defaulted.count != expected.count return 0; for i: 0..expected.count-1 {{ if directory[i] != expected[i] || local[i] != expected[i] || runtime[i] != expected[i] || baked[i] != expected[i] || supplied[i] != expected[i] || defaulted[i] != expected[i] return 0; }} return 42; }}",
        expected.to_str().unwrap()
    );
    fs::write(fixture.0.join("main.jai"), source).unwrap();
    assert_eq!(fixture.run(false).unwrap(), 42);
}

#[test]
fn quoted_filepath_keeps_its_original_directory_in_both_scopes() {
    for insertion in ["#insert quoted", "#insert,scope() quoted"] {
        let fixture = Fixture::new("");
        let directory = fixture.0.join("quoted-é🦀");
        fs::create_dir(&directory).unwrap();
        let expected = fs::canonicalize(&directory).unwrap();
        fs::write(
            directory.join("quote.jai"),
            format!(
                "emit :: (target: Code) #expand {{ quoted :: #code #filepath; (#insert target) = {insertion}; }}"
            ),
        )
        .unwrap();
        let source = format!(
            "#load \"quoted-é🦀/quote.jai\"; main :: ()->int {{ expected := \"{}\"; actual: string = \"\"; emit(actual); if actual.count != expected.count return 0; for i: 0..expected.count-1 {{ if actual[i] != expected[i] return 0; }} return 42; }}",
            expected.to_str().unwrap()
        );
        fs::write(fixture.0.join("main.jai"), source).unwrap();
        assert_eq!(fixture.run(false).unwrap(), 42);
    }
}

#[test]
fn literal_location_requires_the_genuine_source_nominal_shape() {
    for declaration in [
        "",
        "Source_Code_Location :: struct { fully_pathed_filename: string; line_number: u64; character_number: s64; }",
        "Source_Code_Location :: struct { line_number: s64; fully_pathed_filename: string; character_number: s64; }",
    ] {
        let source = format!("{declaration} main :: ()->int {{ loc := #location(); return 0; }}");
        assert!(
            Fixture::new(&source)
                .run(true)
                .unwrap_err()
                .contains("Source_Code_Location")
        );
    }
}

#[test]
fn compile_time_source_location_retains_the_directive_position() {
    let source = format!(
        "{LOCATION}main :: ()->int {{ loc := #run #location(); return loc.line_number*1000+loc.character_number; }}"
    );
    assert_eq!(
        Fixture::new(&source).run(true).unwrap(),
        encoded_position(&source, "#location()")
    );
}

#[test]
fn global_source_literals_keep_the_definition_phase_and_source() {
    let source = format!(
        "{LOCATION}origin :: #location();\nfilename :: #file;\nsource_line :: #line;\nmain :: ()->int {{ if source_line != 4 return 0; if origin.fully_pathed_filename.count != filename.count return 0; for i: 0..filename.count-1 {{ if origin.fully_pathed_filename[i] != filename[i] return 0; }} return origin.line_number*1000+origin.character_number; }}"
    );
    assert_eq!(
        Fixture::new(&source).run(true).unwrap(),
        encoded_position(&source, "#location()")
    );
}

#[test]
fn local_source_literals_remain_immutable_semantic_constants() {
    let source = format!(
        "{LOCATION}main :: ()->int {{ origin :: #location(); filename :: #file; source_line :: #line+1; if source_line != 3 return 0; if origin.fully_pathed_filename.count != filename.count return 0; return origin.line_number*1000+origin.character_number; }}"
    );
    assert_eq!(
        Fixture::new(&source).run(true).unwrap(),
        encoded_position(&source, "#location()")
    );
}

#[test]
fn explicit_location_defaults_are_pinned_at_definition() {
    for definition in [
        "probe :: (loc := #location())->int { return loc.line_number*1000+loc.character_number; } main :: ()->int { callback := probe; return callback(); }",
        "probe :: (value: $T, loc := #location())->int { return loc.line_number*1000+loc.character_number; } main :: ()->int { return probe(9); }",
        "main :: ()->int { probe :: (loc := #location())->int { return loc.line_number*1000+loc.character_number; } return probe(); }",
    ] {
        let source = format!("{LOCATION}{definition}");
        assert_eq!(
            Fixture::new(&source).run(true).unwrap(),
            encoded_position(&source, "#location()")
        );
    }
}

#[test]
fn macro_caller_defaults_and_literal_locations_keep_distinct_origins() {
    let source = format!(
        "{LOCATION}probe :: (loc := #caller_location)->int {{ return loc.line_number*1000+loc.character_number; }}\nemit :: (target: Code) #expand {{ (#insert target) = probe(); }}\nliteral :: (target: Code) #expand {{ loc := #location(); (#insert target) = loc.line_number*1000+loc.character_number; }}\nmain :: ()->int {{ result := 0; emit(result); return result; }}"
    );
    assert_eq!(
        Fixture::new(&source).run(true).unwrap(),
        encoded_position(&source, "emit(result)")
    );
    let source = source.replace("emit(result)", "literal(result)");
    assert_eq!(
        Fixture::new(&source).run(true).unwrap(),
        encoded_position(&source, "#location()")
    );
}

#[test]
fn nested_macros_use_immediate_invocations_and_preserve_forwarded_locations() {
    let source = format!(
        "{LOCATION}probe :: (loc := #caller_location)->int {{ return loc.line_number*1000+loc.character_number; }}\ninner :: (target: Code) #expand {{ (#insert target) = probe(); }}\nouter :: (target: Code) #expand {{ inner(target); }}\nmain :: ()->int {{ result := 0; outer(result); return result; }}"
    );
    assert_eq!(
        Fixture::new(&source).run(true).unwrap(),
        encoded_position(&source, "inner(target)")
    );
    for invocation in ["inner(target)", "inner(target, loc)"] {
        let source = format!(
            "{LOCATION}probe :: (loc := #caller_location)->int {{ return loc.line_number*1000+loc.character_number; }}\ninner :: (target: Code, loc := #caller_location) #expand {{ (#insert target) = probe(loc); }}\nouter :: (target: Code, loc := #caller_location) #expand {{ {invocation}; }}\nmain :: ()->int {{ result := 0; outer(result); return result; }}"
        );
        assert_eq!(
            Fixture::new(&source).run(true).unwrap(),
            encoded_position(&source, "outer(result)")
        );
    }
}

#[test]
fn explicit_location_overrides_are_preserved() {
    let source = format!(
        "{LOCATION}probe :: (loc := #caller_location)->int {{ return loc.line_number; }} main :: ()->int {{ return probe(Source_Code_Location.{{fully_pathed_filename=\"explicit.jai\", line_number=42, character_number=9}}); }}"
    );
    assert_eq!(Fixture::new(&source).run(false).unwrap(), 42);
}

#[test]
fn immutable_explicit_location_defaults_survive_callbacks_and_specialization() {
    let literal = "Source_Code_Location.{fully_pathed_filename=\"explicit.jai\", line_number=42, character_number=9}";
    for source in [
        format!(
            "probe :: (loc: Source_Code_Location = {literal})->int {{ return loc.line_number; }} main :: ()->int {{ callback:=probe; return callback(); }}"
        ),
        format!(
            "probe :: (value: $T, loc: Source_Code_Location = {literal})->int {{ return loc.line_number; }} main :: ()->int {{ return probe(9); }}"
        ),
        format!(
            "main :: ()->int {{ probe :: (loc: Source_Code_Location = {literal})->int {{ return loc.line_number; }} return probe(); }}"
        ),
    ] {
        assert_eq!(
            Fixture::new(&format!("{LOCATION}{source}"))
                .run(true)
                .unwrap(),
            42
        );
    }
}

#[test]
fn quoted_call_keeps_its_source_under_both_insertion_scopes() {
    for insertion in ["#insert quoted", "#insert,scope() quoted"] {
        let source = format!(
            "{LOCATION}probe :: (loc := #caller_location)->int {{ return loc.line_number*1000+loc.character_number; }}\nmain :: ()->int {{ quoted :: #code probe();\nreturn {insertion}; }}"
        );
        assert_eq!(
            Fixture::new(&source).run(true).unwrap(),
            encoded_position(&source, "probe()")
        );
    }
}

#[test]
fn quoted_call_from_another_file_keeps_that_files_exact_name_and_position() {
    for insertion in ["#insert quoted", "#insert,scope() quoted"] {
        let quote = format!(
            "emit :: (target: Code) #expand {{ quoted :: #code probe(); (#insert target) = {insertion}; }}"
        );
        let fixture = Fixture::new("");
        let quote_path = fixture.0.join("quote.jai");
        fs::write(&quote_path, &quote).unwrap();
        let quote_path = fs::canonicalize(quote_path).unwrap();
        let source = format!(
            "{LOCATION}#load \"quote.jai\";\nprobe :: (loc := #caller_location)->int {{ expected := \"{}\"; if loc.fully_pathed_filename.count != expected.count return 0; for i: 0..expected.count-1 {{ if loc.fully_pathed_filename[i] != expected[i] return 0; }} return loc.line_number*1000+loc.character_number; }}\nmain :: ()->int {{ result := 0; emit(result); return result; }}",
            quote_path.to_str().unwrap()
        );
        fs::write(fixture.0.join("main.jai"), source).unwrap();
        assert_eq!(
            fixture.run(true).unwrap(),
            encoded_position(&quote, "probe()")
        );
    }
}

#[test]
fn compile_time_calls_materialize_the_source_location_before_execution() {
    assert_caller(
        "probe :: (loc := #caller_location)->int { return loc.line_number*1000+loc.character_number; }",
        "main :: ()->int { return #run probe(); }",
        "probe()",
    );
}

#[test]
fn malformed_source_schema_and_missing_target_are_diagnostics() {
    let source = format!(
        "{LOCATION}probe :: (loc := #caller_location)->int {{ return 42; }} main :: ()->int {{ return probe(); }}"
    );
    assert!(
        Fixture::new(&source)
            .run(false)
            .unwrap_err()
            .contains("explicit target layout")
    );
    for location in [
        "Source_Code_Location :: struct { fully_pathed_filename: string; character_number: s64; line_number: s64; }",
        "Source_Code_Location :: struct { fully_pathed_filename: string; line_number: u64; character_number: s64; }",
        "Other_Location :: struct { fully_pathed_filename: string; line_number: s64; character_number: s64; }",
    ] {
        let source = format!(
            "{location} probe :: (loc := #caller_location)->int {{ return 42; }} main :: ()->int {{ return probe(); }}"
        );
        assert!(
            Fixture::new(&source)
                .run(true)
                .unwrap_err()
                .contains("Source_Code_Location")
        );
    }
}

#[test]
fn marker_cannot_be_used_as_an_ordinary_expression_or_scalar_default() {
    for source in [
        "main :: ()->int { value := #caller_location; return 0; }",
        "probe :: (loc: s64 = #caller_location)->int { return loc; } main :: ()->int { return probe(); }",
        "emit :: (target: Code, loc: s64 = #caller_location) #expand { (#insert target) = 42; } main :: ()->int { value := 0; emit(value); return value; }",
    ] {
        assert!(
            Fixture::new(&format!("{LOCATION}{source}"))
                .run(true)
                .unwrap_err()
                .contains("#caller_location")
        );
    }
    let source = format!(
        "{LOCATION}emit :: (target: Code, loc := #caller_location) #expand {{ (#insert target) = 42; }} main :: ()->int {{ value := 0; emit(value, 9); return value; }}"
    );
    assert!(Fixture::new(&source).run(true).is_err());
}

#[test]
fn caller_defaults_do_not_require_implicit_context() {
    assert_caller(
        "probe :: (loc := #caller_location)->int #no_context { return loc.line_number*1000+loc.character_number; }",
        "main :: ()->int #no_context { return probe(); }",
        "probe()",
    );
}

#[test]
fn inferred_location_adopts_the_complete_compiler_prelude_nominal() {
    use jai_modules::{PreludeSource, SourceOverlay};
    use jai_types::{Architecture, BuildTarget, ByteOrder, OperatingSystem};
    use std::path::Path;
    let source = "probe :: (loc := #caller_location)->s64 { return loc.line_number*1000+loc.character_number; }\nmain :: ()->s64 { literal := #location(); filename := #file; if literal.line_number != #line return 0; if literal.fully_pathed_filename.count != filename.count return 0; for i: 0..filename.count-1 { if literal.fully_pathed_filename[i] != filename[i] return 0; } return probe(); }";
    let mut provider = SourceOverlay::new();
    provider
        .insert(
            Path::new("/jai-caller-preload/main.jai"),
            source.as_bytes().to_vec(),
        )
        .unwrap();
    provider
        .insert(
            Path::new("/jai-caller-preload/modules/module.jai"),
            jai_modules::compiler_prelude_source().as_bytes().to_vec(),
        )
        .unwrap();
    let target = BuildTarget {
        operating_system: OperatingSystem::Linux,
        architecture: Architecture::X86_64,
        layout: LayoutPolicy::lp64(),
        byte_order: ByteOrder::Little,
    };
    let roots = vec!["/jai-caller-preload/modules".into()];
    let graph = ModuleGraph::load_with_bootstrap(
        Path::new("/jai-caller-preload/main.jai"),
        GraphOptions {
            import_dirs: roots.clone(),
        },
        PreludeSource::Search,
        &provider,
        Some(target.clone()),
    )
    .unwrap();
    let compiler = jai_sema::CompilerBindingContext::from_graph(
        &graph,
        &roots,
        jai_vm::WorkspaceId::from_raw(1).unwrap(),
    );
    let program = resolve_graph_with_options(
        &graph,
        &ResolveOptions {
            target: Some(target),
            compiler: Some(compiler),
            ..ResolveOptions::default()
        },
        &mut NoEffects,
    )
    .unwrap_or_else(|error| panic!("{}", error.render(graph.sources())));
    let result = jai_vm::execute(&program, Limits::default());
    assert!(
        matches!(result.outcome, Outcome::Complete(ref values) if matches!(values.as_slice(), [Value::Int(value)] if value.value() == encoded_position(source, "probe()"))),
        "{result:?}"
    );
}

#[test]
fn inferred_location_follows_the_selected_alias_defining_file() {
    for alias in [
        "Source_Code_Location :: Provider.Source_Code_Location;",
        "Source_Code_Location :: #type Provider.Source_Code_Location;",
    ] {
        let source = "#load \"facade.jai\";\nmain::()->s64 { return probe(); }";
        let fixture = Fixture::new(source);
        let provider = format!(
            "{LOCATION}take::(value:Source_Code_Location)->s64{{return value.line_number*1000+value.character_number;}}"
        );
        let facade = format!(
            "#scope_file Provider::#import,file \"provider.jai\";#scope_export {alias}probe::(loc:=#caller_location)->s64{{return Provider.take(loc);}}"
        );
        fs::write(fixture.0.join("provider.jai"), provider).unwrap();
        fs::write(fixture.0.join("facade.jai"), facade).unwrap();
        assert_eq!(
            fixture.run(true).unwrap(),
            encoded_position(source, "probe()")
        );
    }
}

#[test]
fn literal_location_follows_the_selected_source_alias() {
    let facade = "#scope_file Provider::#import,file \"provider.jai\";#scope_export Source_Code_Location::Provider.Source_Code_Location;probe::()->s64{location:=#location();return Provider.take(location);}";
    let fixture = Fixture::new("#load \"facade.jai\";main::()->s64{return probe();}");
    let provider = format!(
        "{LOCATION}take::(value:Source_Code_Location)->s64{{return value.line_number*1000+value.character_number;}}"
    );
    fs::write(fixture.0.join("provider.jai"), provider).unwrap();
    fs::write(fixture.0.join("facade.jai"), facade).unwrap();
    assert_eq!(
        fixture.run(true).unwrap(),
        encoded_position(facade, "#location()")
    );
}

#[test]
fn caller_location_aliases_preserve_nominal_validation_and_cycle_rejection() {
    for (target, provider) in [
        (
            "Other_Location",
            "Other_Location::struct{fully_pathed_filename:string;line_number:s64;character_number:s64;}",
        ),
        (
            "Source_Code_Location",
            "Source_Code_Location::struct{fully_pathed_filename:string;line_number:u64;character_number:s64;}",
        ),
    ] {
        let source = format!(
            "Provider::#import,file \"provider.jai\";Source_Code_Location::Provider.{target};probe::(loc:=#caller_location)->s64{{return 42;}}main::()->s64{{return probe();}}"
        );
        let fixture = Fixture::new(&source);
        fs::write(fixture.0.join("provider.jai"), provider).unwrap();
        assert!(
            fixture
                .run(true)
                .unwrap_err()
                .contains("Source_Code_Location")
        );
    }
    let fixture = Fixture::new(
        "Source_Code_Location::Cycle;Cycle::Source_Code_Location;probe::(loc:=#caller_location)->s64{return 42;}main::()->s64{return probe();}",
    );
    // Source discovery may reject a cycle before semantic inference reaches it.
    if let Ok(graph) = ModuleGraph::load(&fixture.0.join("main.jai"), GraphOptions::default()) {
        assert!(
            resolve_graph_with_options(
                &graph,
                &ResolveOptions {
                    layout: Some(LayoutPolicy::lp64()),
                    ..Default::default()
                },
                &mut NoEffects,
            )
            .is_err()
        );
    }
}

#[test]
fn exported_facade_adopts_the_selected_physical_preload_location() {
    use jai_modules::{PreludeSource, SourceOverlay};
    use jai_types::{Architecture, BuildTarget, ByteOrder, OperatingSystem};
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let source = "#load \"facade.jai\";\nmain::()->s64 { return probe(); }";
    let fixture = Fixture::new(source);
    let facade = "#scope_file Protocol::#import \"Preload\";#scope_export Source_Code_Location::Protocol.Source_Code_Location;accept::(location:Protocol.Source_Code_Location)->s64{return location.line_number*1000+location.character_number;}probe::(loc:=#caller_location)->s64{return accept(loc);}";
    fs::write(fixture.0.join("facade.jai"), facade).unwrap();
    let target = BuildTarget {
        operating_system: OperatingSystem::Linux,
        architecture: Architecture::X86_64,
        layout: LayoutPolicy::lp64(),
        byte_order: ByteOrder::Little,
    };
    let roots = vec![repository.join("stdlib")];
    let provider = SourceOverlay::new();
    let graph = ModuleGraph::load_with_bootstrap(
        &fixture.0.join("main.jai"),
        GraphOptions {
            import_dirs: roots.clone(),
        },
        PreludeSource::File(repository.join("stdlib/prelude/module.jai")),
        &provider,
        Some(target.clone()),
    )
    .unwrap();
    let compiler = jai_sema::CompilerBindingContext::from_graph(
        &graph,
        &roots,
        jai_vm::WorkspaceId::from_raw(1).unwrap(),
    );
    let program = resolve_graph_with_options(
        &graph,
        &ResolveOptions {
            target: Some(target),
            compiler: Some(compiler),
            ..Default::default()
        },
        &mut NoEffects,
    )
    .unwrap_or_else(|e| panic!("{}", e.render(graph.sources())));
    let outcome = jai_vm::execute(&program, Limits::default()).outcome;
    assert!(
        matches!(outcome,Outcome::Complete(ref values) if matches!(values.as_slice(),[Value::Int(value)] if value.value()==encoded_position(source,"probe()"))),
        "{outcome:?}"
    );
}
