//! Opt-in parsing of pinned local source spans; no reference tool executes.
use jai_source::{SourceMap, Symbols};
use std::{fs, ops::Range, path::Path};

enum Expected {
    Parsed,
    MacroCaptureRejected,
}

struct PinnedStatement {
    path: &'static str,
    span: Range<usize>,
    source_bytes: usize,
    expected: Expected,
}

#[test]
#[ignore = "requires the separately supplied reference and pinned upstream source corpus"]
fn supplied_cast_statements_keep_their_original_source_spelling() {
    for case in [
        PinnedStatement {
            span: 8451..8497,
            source_bytes: 18636,
            expected: Expected::MacroCaptureRejected,
        },
        PinnedStatement {
            path: "corpus/upstream/ostef--Vk-Engine/Modules/Hash_Map.jai",
            span: 3302..3356,
            source_bytes: 6489,
            expected: Expected::Parsed,
        },
        PinnedStatement {
            path: "corpus/upstream/roeyb1--sgpu/shader_vfs.jai",
            span: 1969..2023,
            source_bytes: 5569,
            expected: Expected::Parsed,
        },
        PinnedStatement {
            span: 6816..6851,
            source_bytes: 7492,
            expected: Expected::Parsed,
        },
        PinnedStatement {
            path: "corpus/upstream/roeyb1--sgpu/module.jai",
            span: 19955..19989,
            source_bytes: 49986,
            expected: Expected::Parsed,
        },
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(case.path);
        let bytes = fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        assert_eq!(bytes.len(), case.source_bytes, "{} changed", case.path);
        let text = jai_lexer::decode_source(&bytes).unwrap();
        let statement = text.get(case.span).expect("pinned source span changed");
        assert!(statement.ends_with(';'));
        let fixture = format!("main::(){{{statement}}}");
        let mut sources = SourceMap::default();
        let id = sources.insert(case.path.into(), fixture);
        let result = jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default());
        match case.expected {
            Expected::MacroCaptureRejected => {
                let error = result.unwrap_err();
                assert_eq!(
                    error.location.span.text(sources.get(id).unwrap().text()),
                    "`"
                );
                assert_eq!(
                    error.message,
                    "expected expression; this syntax is not implemented yet"
                );
            }
            Expected::Parsed => assert!(result.is_ok(), "{}: {result:?}", case.path),
        }
    }
}
