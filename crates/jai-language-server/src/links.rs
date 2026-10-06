//! `#load` and `#import` strings as links to the files they bring in, found from tokens (so
//! they work while the text does not parse) and resolved with the compiler's own rules
//! (`jaic::sema::import_entry`).
use crate::analysis::Span;
use jaic::ast::ImportSource;
use jaic::lexer::{P, Tok, Token};

#[derive(Clone, Debug)]
pub enum LinkSource {
    /// `#load "path"`, relative to the loading file.
    Load(String),
    /// `#import "Name"`, `#import,file "x.jai"`, `#import,dir "x"`.
    Import(ImportSource),
}

#[derive(Clone, Debug)]
pub struct Link {
    /// From the directive to the end of the string literal.
    pub directive: Span,
    /// The string literal, quotes included.
    pub string: Span,
    pub source: LinkSource,
}

/// The `#load` and `#import` directives of a token stream.
pub fn links(tokens: &[Token]) -> Vec<Link> {
    let mut out = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        let Tok::Directive(name) = &token.tok else {
            continue;
        };
        let load = match name.as_str() {
            "load" => true,
            "import" => false,
            _ => continue,
        };
        // `#import,file "x"`: comma-separated flags before the string.
        let mut at = i + 1;
        let mut flag = None;
        while let (Some(Tok::Punct(P::Comma)), Some(Tok::Ident(f))) = (
            tokens.get(at).map(|t| &t.tok),
            tokens.get(at + 1).map(|t| &t.tok),
        ) {
            flag.get_or_insert(f.as_str().to_string());
            at += 2;
        }
        let Some(Token {
            tok: Tok::Str(bytes),
            span,
            ..
        }) = tokens.get(at)
        else {
            continue;
        };
        let path = String::from_utf8_lossy(bytes).into_owned();
        let source = if load {
            LinkSource::Load(path)
        } else {
            LinkSource::Import(match flag.as_deref() {
                Some("file") => ImportSource::File(path.into()),
                Some("dir") => ImportSource::Dir(path.into()),
                Some("string") => continue,
                _ => ImportSource::Module(path.into()),
            })
        };
        out.push(Link {
            directive: Span::new(token.span.start as usize, span.end as usize),
            string: Span::new(span.start as usize, span.end as usize),
            source,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directives_with_flags() {
        let text = "#import \"Basic\"; B :: #import,file \"b.jai\"; #load \"x/y.jai\"; #import,string \"x :: 1;\";";
        let tokens = jaic::lexer::lex(jaic::source::FileId(0), text).unwrap();
        let found = links(&tokens);
        assert_eq!(found.len(), 3);
        assert!(
            matches!(&found[0].source, LinkSource::Import(ImportSource::Module(n)) if &**n == "Basic")
        );
        assert!(
            matches!(&found[1].source, LinkSource::Import(ImportSource::File(n)) if &**n == "b.jai")
        );
        assert!(matches!(&found[2].source, LinkSource::Load(p) if p == "x/y.jai"));
        assert_eq!(
            &text[found[2].string.start..found[2].string.end],
            "\"x/y.jai\""
        );
    }
}
