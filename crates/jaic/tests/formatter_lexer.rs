//! The formatter (`stdlib/Extensions/Jai_Format/lexer.jai`) has a lexer of its own, because it
//! keeps comments and whitespace. Its punctuation table must be jaic's, spelling for spelling and
//! in the same order (longest match first); every spelling must also lex as one token.
use jaic::lexer::{Lexed, Tok, punctuation_spellings};
use jaic::source::FileId;

const FORMATTER_LEXER: &str = include_str!("../../../stdlib/Extensions/Jai_Format/lexer.jai");

/// The strings of the formatter's `PUNCTUATION :: string.[ ... ];`.
fn formatter_table() -> Vec<String> {
    let body = FORMATTER_LEXER
        .split("PUNCTUATION :: string.[")
        .nth(1)
        .and_then(|rest| rest.split("];").next())
        .expect("the formatter's punctuation table");
    let mut spellings = Vec::new();
    let mut chars = body.chars();
    while let Some(c) = chars.next() {
        if c == '"' {
            let spelling: String = chars.by_ref().take_while(|&c| c != '"').collect();
            spellings.push(spelling);
        }
    }
    spellings
}

#[test]
fn the_formatters_punctuation_is_the_compilers() {
    let theirs = formatter_table();
    let ours: Vec<&str> = punctuation_spellings().collect();
    assert_eq!(
        theirs, ours,
        "update PUNCTUATION in the formatter's lexer.jai"
    );
}

#[test]
fn every_spelling_is_one_punctuation_token() {
    for spelling in punctuation_spellings() {
        let lexed = Lexed::new(FileId(0), spelling);
        let tokens: Vec<_> = lexed
            .tokens
            .iter()
            .filter(|t| !matches!(t.tok, Tok::Eof))
            .collect();
        assert_eq!(tokens.len(), 1, "{spelling}");
        assert!(matches!(tokens[0].tok, Tok::Punct(_)), "{spelling}");
    }
}
