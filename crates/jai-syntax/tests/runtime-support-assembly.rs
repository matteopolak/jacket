//! Whole original source parsing retains unsupported inactive assembly profiles.
use jai_source::{SourceMap, Symbols};
#[path = "support/optional_sources.rs"]
mod optional_sources;

fn assert_runtime_support_parses_completely(relative: &str, items: usize) {
    let Some(text) = optional_sources::read_optional(relative) else {
        return;
    };
    let mut sources = SourceMap::default();
    let source = sources.insert(relative.into(), text.clone());
    let mut symbols = Symbols::default();
    // Parse every original token in ordinary file mode, including inactive branches.
    let file = jai_syntax::parse_file(sources.get(source).unwrap(), &mut symbols)
        .unwrap_or_else(|diagnostic| panic!("{}", diagnostic.render(&sources)));
    assert_eq!(file.source(), source);
    assert_eq!(file.items().len(), items);
    for statement in [
        "#asm { int 0x41; }",
        "#asm { int3; }",
        "#bytes .[0x20, 0x00, 0b001_0_0000, 0b1101_0100];",
    ] {
        assert!(text.contains(statement));
    }
    assert!(symbols.find("debug_break").is_some());
    assert!(symbols.find("futex").is_some());
    assert!(symbols.find("SYSCALL_SYSRET").is_some());
    assert_eq!(sources.get(source).unwrap().text(), text);
}

#[test]
fn complete_original_runtime_support_parses_all_platform_assembly_branches() {
}

#[test]
fn complete_pinned_focus_runtime_support_parses_all_platform_assembly_branches() {
    assert_runtime_support_parses_completely(
        "corpus/upstream/focus-editor--focus/modules/Runtime_Support.jai",
        46,
    );
}
