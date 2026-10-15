//! Which identifiers of a project get no hover or no go to definition from jailsp.
//!
//! `cargo run --locked -p jai-language-server --example ide_coverage -- --json --strict --redacted DIR...`
//!
//! Each DIR is opened as a workspace folder with all of its `.jai` files open, as an editor
//! would. Every identifier and directive outside comments and strings is asked for hover, and
//! every identifier (including declarations) for go to definition. A hover that is only the syntax layer's
//! fallback counts as missing. Typecheck failures never waive gaps. Original corpus bodies
//! are opaque coordinator-only inputs. See docs/tools/ide-coverage.md.
use jai_language_server::{Environment, Position};
use jaic::lexer::{P, Tok};
use std::path::PathBuf;

const KEYWORDS: &[&str] = &[
    "if",
    "else",
    "then",
    "ifx",
    "for",
    "while",
    "break",
    "continue",
    "return",
    "defer",
    "using",
    "struct",
    "union",
    "enum",
    "enum_flags",
    "case",
    "cast",
    "xx",
    "inline",
    "no_inline",
    "push_context",
    "remove",
    "null",
    "true",
    "false",
];

/// What an identifier is doing, from its neighbours.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Role {
    /// `name ::`, `name :=`, `name:` (and parameters).
    Declaration,
    /// `f(name = 1)`, `.{name = 1}`.
    Named,
    /// `a.name`, `.NAME`.
    Member,
    Use,
    /// `#name` (hover only).
    Directive,
}

fn environment(stdlib: PathBuf, runtime_support: bool) -> Environment {
    Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |_| {
            let mut options = jaic::sema::Options::host();
            options.import_paths = vec![stdlib.clone()];
            options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
            options.runtime_support = runtime_support;
            options.stack_trace = runtime_support;
            options
        }),
    }
}

/// Byte offsets where each line starts, for `position`.
fn line_starts(text: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(text.match_indices('\n').map(|(i, _)| i + 1))
        .collect()
}

fn position(text: &str, starts: &[usize], byte: usize) -> Position {
    let line = starts.partition_point(|&s| s <= byte) - 1;
    let character = text[starts[line]..byte].encode_utf16().count();
    Position {
        line: line as u32,
        character: character as u32,
    }
}

fn role(tokens: &[jaic::lexer::Token], i: usize) -> Role {
    let next = tokens.get(i + 1).map(|t| &t.tok);
    let prev = i.checked_sub(1).map(|j| &tokens[j].tok);
    if matches!(prev, Some(Tok::Punct(P::Dot))) {
        return Role::Member;
    }
    match next {
        Some(Tok::Punct(P::ColonColon | P::Colon | P::ColonEq)) => Role::Declaration,
        Some(Tok::Punct(P::Eq))
            if matches!(
                prev,
                Some(Tok::Punct(P::LParen | P::Comma | P::LBrace | P::DotBrace))
            ) =>
        {
            Role::Named
        }
        _ => Role::Use,
    }
}

fn using_modifier(tokens: &[jaic::lexer::Token], i: usize) -> bool {
    i >= 2
        && matches!(&tokens[i].tok,Tok::Ident(name) if matches!(name.as_str(),"only"|"except"|"map"))
        && matches!(tokens[i - 1].tok, Tok::Punct(P::Comma))
        && matches!(&tokens[i-2].tok,Tok::Ident(name) if name.as_str()=="using")
}

#[path = "ide_coverage/report.rs"]
mod report;
#[path = "ide_coverage/runner.rs"]
mod runner;

fn main() {
    runner::main();
}
