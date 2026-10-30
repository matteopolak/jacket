//! `editors/lexicon.json` is generated from the compiler's lexicon.

#[test]
fn editors_lexicon_json_is_current() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../editors/lexicon.json");
    test_support::snapshot::assert_generated_file(
        &path,
        &jaic::lexicon::to_json(),
        ", then run `node editors/vscode/scripts/build-grammar.ts`",
    );
}
