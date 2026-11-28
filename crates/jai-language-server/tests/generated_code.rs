//! Code a build metaprogram generates (`add_build_string`) has no file: go to definition lands
//! in a `jai-generated:` document the server serves through `jai/source`.
use test_support::lsp::{at, open_file, project};

const BUILD: &str = "#import \"Basic\";\n#import \"Compiler\";\n#run build();\nbuild :: () {\n    \
    set_build_options_dc(.{do_output = false});\n    w := compiler_create_workspace(\"app\");\n    \
    options := get_build_options(w);\n    options.output_type = .NO_OUTPUT;\n    \
    set_build_options(options, w);\n    add_build_string(\"made_here :: () -> int { return 7; }\\n\", w);\n    \
    add_build_file(\"main.jai\", w);\n}\n";

const MAIN: &str =
    "#import \"Basic\";\nmain :: () {\n    value := made_here();\n    print(\"%\\n\", value);\n}\n";

#[test]
fn definition_into_a_generated_string_points_at_a_served_document() {
    let dir = project(
        "generated-code",
        &[
            ("jacket.toml", "[project]\nentry = [\"build.jai\"]\n"),
            ("build.jai", BUILD),
            ("main.jai", MAIN),
        ],
    );
    let (s, uri, text) = open_file(&dir, "main.jai");
    let found = s.definition(&uri, at(&text, "made_here()")).unwrap();
    let generated = found
        .iter()
        .find(|l| l.uri.as_str().starts_with("jai-generated:"))
        .unwrap_or_else(|| panic!("{found:?}"));
    let served = s.generated_source(generated.uri.as_str()).expect("served");

    assert!(served.contains("made_here :: () -> int"), "{served}");
    assert_eq!(generated.range.start.line, 0);
}

#[test]
fn an_error_that_names_generated_code_points_into_the_served_document() {
    let dir = project(
        "generated-note",
        &[
            ("jacket.toml", "[project]\nentry = [\"build.jai\"]\n"),
            ("build.jai", BUILD),
            (
                "main.jai",
                "#import \"Basic\";\nmain :: () {\n    value := made_here(1, 2);\n    print(\"%\\n\", value);\n}\n",
            ),
        ],
    );
    let (s, uri, _) = open_file(&dir, "main.jai");
    let diagnostics = s.diagnostics(&uri).unwrap();
    let related: Vec<_> = diagnostics
        .iter()
        .flat_map(|d| d.related.iter())
        .filter(|r| r.uri.starts_with("jai-generated:"))
        .collect();

    assert!(!related.is_empty(), "{diagnostics:?}");
    assert!(s.generated_source(&related[0].uri).is_some());
}
