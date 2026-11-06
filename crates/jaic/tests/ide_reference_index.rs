//! Owned reference queries must remain equivalent to live compiler resolution.
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::{Compiler, Options, VirtualFs};
use jaic::source::FileId;
use std::path::Path;
use std::rc::Rc;

#[test]
fn owned_reference_index_matches_every_cursor_in_checked_sources() {
    let programs = [
        "global := 1; main :: () { local := global; { local := 2; local += 1; } local += global; }",
        "f :: (x: int) -> int { return x; } f :: (x: float) -> float { return x; } alias :: f; main :: () { a := f(1); b := alias(2.0); }",
        "S :: struct { field: int; } E :: enum { a; b :: a; } main :: () { s: S; s.field = 1; e := E.b; }",
        "twice :: (x: $T) -> T { return x + x; } main :: () { x := twice(1); y := twice(2.0); }",
        "#load \"other.jai\"; main :: () { x := supplied; nested :: () { y := supplied; } nested(); }",
        "f :: (first: int, second: int) -> int { return first + second; } main :: () { x := f(second = 2, first = 1); }",
        "#insert \"generated :: () -> int { return 7; }\"; main :: () { x := generated(); }",
        "unused :: (x: $T) -> T { return x; } main :: () {}",
        "main :: () { // 😀 UTF-16\n größe := 1; größe += 2; broken: int = \"text\"; }",
    ];
    for text in programs {
        let mut fs = VirtualFs::default();
        fs.insert("/references/main.jai", text.as_bytes());
        fs.insert("/references/other.jai", "supplied :: 42;".as_bytes());
        let mut options = Options::host();
        options.runtime_support = false;
        options.stack_trace = false;
        let mut compiler = Compiler::new(options, Rc::new(fs));
        compiler
            .ed
            .record(IdeFacts::new(vec![IdeRoot::Under("/references".into())]));
        let compiled = compiler.compile_program(Path::new("/references/main.jai"));
        if !text.contains("broken:") {
            assert!(
                compiled.is_ok(),
                "valid oracle fixture must compile: {text}"
            );
        }
        let _ = compiler.ide_check_all();
        compiler.ide_sketch_polymorphs();
        let index = compiler.ide_reference_index();
        assert!(index.retained_bytes() > 0);
        let mut queries = Vec::new();
        let mut nonempty = 0;
        for file in (0..compiler.src.sources.len() as u32).map(FileId) {
            let source = &compiler.src.sources.get(file).text;
            for offset in 0..=source.len() as u32 {
                let expected = compiler.ide_references(file, offset);
                nonempty += usize::from(!expected.is_empty());
                assert_eq!(
                    index.references(file, offset),
                    expected,
                    "{text:?}, file {file:?}, byte {offset}"
                );
                queries.push((file, offset, expected));
            }
        }
        assert!(
            nonempty > 0,
            "fixture must record useful reference facts: {text}"
        );
        drop(compiler);
        for (file, offset, expected) in queries {
            assert_eq!(index.references(file, offset), expected);
        }
    }
}
