//! What a Jai file declares at its top level and what it brings in, read from tokens alone (no
//! parse), so a module or project file that does not parse yet still has names. This is the
//! raw material of auto-import completion (`auto_import.rs`).
//!
//! The scan follows top-level structure only: a `{` after `#if` (or after the `else` of one)
//! keeps the top level open, every other `{` (a procedure or struct body) closes it until its
//! `}`. `#scope_file`, `#scope_module` and `#scope_export` set the visibility of what follows.
use crate::model::CompletionKind;
use jaic::lexer::{P, Tok, Token};
use jaic::source::FileId;

/// Who can see a top-level declaration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    /// Importers of the module (and the rest of the program).
    Export,
    /// The files of its module or program only (`#scope_module`).
    Module,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decl {
    pub name: String,
    /// `name` in lower case, for case-insensitive prefix matching.
    pub lower: String,
    pub kind: CompletionKind,
    pub visibility: Visibility,
    /// The declaration's header on one line: `print :: (format: string, args: ..Any) -> s64`.
    pub signature: String,
    /// The `//` comment lines right above it.
    pub doc: String,
}

/// A top-level `#import "Name"` (bound to `name` for `Name :: #import`), or `#load`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Directive {
    pub path: String,
    pub name: Option<String>,
    /// Byte offset just past the statement (its `;`, else its string).
    pub end: usize,
    /// Inside an `#if` block rather than directly at the top level.
    pub nested: bool,
}

/// A top-level name and where it is declared, whatever its visibility (workspace symbols).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub kind: CompletionKind,
    /// Byte offset of the name.
    pub offset: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileScan {
    /// Every top-level declaration, `#scope_file` ones included.
    pub symbols: Vec<Symbol>,
    /// Top-level declarations outside `#scope_file`.
    pub decls: Vec<Decl>,
    pub imports: Vec<Directive>,
    pub loads: Vec<Directive>,
    /// The targets a top-level `#assert OS == .A || OS == .B` allows (`["A", "B"]`).
    pub only_os: Option<Vec<String>>,
    /// The file declares `main` at its top level (it is a program's entry, not a part).
    pub declares_main: bool,
}

/// The longest signature kept, in bytes.
const SIGNATURE_BYTES: usize = 200;

/// The most comment lines kept as a declaration's documentation.
const DOC_LINES: usize = 12;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Frame {
    /// A `{` that keeps the top level open (`#if`'s block).
    Open,
    /// Any other `{`, `(` or `[`.
    Closed,
}

pub fn scan(text: &str) -> FileScan {
    let Ok(tokens) = jaic::lexer::lex(FileId(0), text) else {
        return FileScan::default();
    };
    scan_tokens(text, &tokens)
}

fn scan_tokens(text: &str, tokens: &[Token]) -> FileScan {
    let mut out = FileScan::default();
    let mut frames: Vec<Frame> = Vec::new();
    let mut visibility = Some(Visibility::Export);
    // The next `{` belongs to `#if` (or its `else`).
    let mut static_block = false;
    // The last `}` closed an `#if` block (so `else {` continues it).
    let mut closed_static = false;
    let top = |frames: &[Frame]| frames.iter().all(|f| *f == Frame::Open);
    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];
        let at_top = top(&frames);
        let after_static = closed_static;
        closed_static = false;
        match &token.tok {
            Tok::Punct(P::LBrace) => {
                frames.push(if static_block && at_top {
                    Frame::Open
                } else {
                    Frame::Closed
                });
                static_block = false;
            }
            Tok::Punct(P::DotBrace | P::DotBracket | P::LParen | P::LBracket) => {
                frames.push(Frame::Closed);
            }
            Tok::Punct(P::RBrace | P::RParen | P::RBracket) => {
                closed_static = frames.pop() == Some(Frame::Open);
            }
            Tok::Punct(P::Semi) if at_top => static_block = false,
            Tok::Ident(word) if at_top && after_static && word.as_str() == "else" => {
                static_block = true;
            }
            Tok::Directive(name) if at_top => match name.as_str() {
                "if" => static_block = true,
                "scope_file" => visibility = None,
                "scope_module" => visibility = Some(Visibility::Module),
                "scope_export" => visibility = Some(Visibility::Export),
                "load" | "import" => {
                    if let Some(directive) = directive_at(tokens, i, None, !frames.is_empty()) {
                        if name.as_str() == "load" {
                            out.loads.push(directive);
                        } else {
                            out.imports.push(directive);
                        }
                    }
                }
                "assert" => {
                    if let Some(targets) = asserted_targets(&tokens[i + 1..]) {
                        out.only_os = Some(targets);
                    }
                }
                _ => {}
            },
            Tok::Ident(name) if at_top && starts_statement(tokens, i) => {
                let next = tokens.get(i + 1).map(|t| &t.tok);
                if let Some(Tok::Punct(P::ColonColon | P::Colon | P::ColonEq)) = next {
                    let constant = matches!(next, Some(Tok::Punct(P::ColonColon)));
                    // `Name :: #import "X";` binds a module: an import, not a declaration.
                    if constant
                        && let Some(Tok::Directive(d)) = tokens.get(i + 2).map(|t| &t.tok)
                        && d.as_str() == "import"
                    {
                        if let Some(directive) =
                            directive_at(tokens, i + 2, Some(name.as_str()), !frames.is_empty())
                        {
                            out.imports.push(directive);
                        }
                        i += 2;
                    } else {
                        if name.as_str() == "main" {
                            out.declares_main = true;
                        }
                        if let Some(kind) = kind_of(tokens, i + 2, constant) {
                            out.symbols.push(Symbol {
                                name: name.as_str().into(),
                                kind,
                                offset: token.span.start as usize,
                            });
                        }
                        if let Some(visibility) = visibility
                            && let Some(kind) = kind_of(tokens, i + 2, constant)
                        {
                            out.decls.push(Decl {
                                name: name.as_str().into(),
                                lower: name.as_str().to_lowercase(),
                                kind,
                                visibility,
                                signature: signature(text, tokens, i),
                                doc: doc_above(text, token.span.start as usize),
                            });
                        }
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    out
}

/// Whether the identifier at `i` begins a statement: first in the file, after `;`, a brace or
/// a scope directive, or first on its line.
fn starts_statement(tokens: &[Token], i: usize) -> bool {
    if i == 0 || tokens[i].newline_before {
        return true;
    }
    match &tokens[i - 1].tok {
        Tok::Punct(P::Semi | P::LBrace | P::RBrace) => true,
        Tok::Directive(d) => d.as_str().starts_with("scope_"),
        _ => false,
    }
}

/// The `#load`/`#import` at `i` (flags such as `,file` make it no module import).
fn directive_at(tokens: &[Token], i: usize, name: Option<&str>, nested: bool) -> Option<Directive> {
    let Some(Token {
        tok: Tok::Str(bytes),
        span,
        ..
    }) = tokens.get(i + 1)
    else {
        return None;
    };
    let end = match tokens.get(i + 2) {
        Some(Token {
            tok: Tok::Punct(P::Semi),
            span,
            ..
        }) => span.end,
        _ => span.end,
    };
    Some(Directive {
        path: String::from_utf8_lossy(bytes).into_owned(),
        name: name.map(Into::into),
        end: end as usize,
        nested,
    })
}

/// The targets of `#assert OS == .A || OS == .B ...` (to its `;`, string or `,`); `None` when
/// the condition names no target or compares with anything but `==`.
fn asserted_targets(tokens: &[Token]) -> Option<Vec<String>> {
    let mut found = Vec::new();
    let mut i = 0;
    while let Some(token) = tokens.get(i) {
        match &token.tok {
            Tok::Punct(P::Semi | P::Comma) | Tok::Str(_) => break,
            Tok::Ident(os) if os.as_str() == "OS" => {
                let rest: Vec<&Tok> = tokens[i + 1..].iter().take(3).map(|t| &t.tok).collect();
                match rest.as_slice() {
                    [Tok::Punct(P::EqEq), Tok::Punct(P::Dot), Tok::Ident(target)] => {
                        found.push(target.as_str().to_string());
                        i += 3;
                    }
                    _ => return None,
                }
            }
            _ => {}
        }
        i += 1;
        if i > 64 {
            break;
        }
    }
    (!found.is_empty()).then_some(found)
}

/// The kind of a declaration whose value starts at `at` (after `::`, `:` or `:=`); `None` for a
/// library (`#library`, `#system_library`), which is no name to complete.
fn kind_of(tokens: &[Token], at: usize, constant: bool) -> Option<CompletionKind> {
    if !constant {
        return Some(CompletionKind::Variable);
    }
    let Some(first) = tokens.get(at) else {
        return Some(CompletionKind::Constant);
    };
    Some(match &first.tok {
        Tok::Ident(w) if matches!(w.as_str(), "struct" | "union") => CompletionKind::Struct,
        Tok::Ident(w) if matches!(w.as_str(), "enum" | "enum_flags") => CompletionKind::Enum,
        Tok::Directive(d) => match d.as_str() {
            "type" => CompletionKind::TypeAlias,
            "bake_arguments" | "bake_constants" | "procedure_of_call" => CompletionKind::Function,
            "library" | "system_library" | "foreign_library" | "foreign_system_library" => {
                return None;
            }
            _ => CompletionKind::Constant,
        },
        Tok::Punct(P::LParen) => {
            // A procedure when its parameter list is followed by `->`, a body or a directive
            // (`#expand`, `#foreign`, ...); `(1 + 2);` is a constant.
            let mut depth = 0usize;
            let mut j = at;
            while let Some(t) = tokens.get(j) {
                match t.tok {
                    Tok::Punct(P::LParen) => depth += 1,
                    Tok::Punct(P::RParen) => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            match tokens.get(j + 1).map(|t| &t.tok) {
                Some(Tok::Punct(P::Arrow | P::LBrace) | Tok::Directive(_)) => {
                    CompletionKind::Function
                }
                _ => CompletionKind::Constant,
            }
        }
        _ => CompletionKind::Constant,
    })
}

/// The declaration at `i` up to its body or `;`, on one line.
fn signature(text: &str, tokens: &[Token], i: usize) -> String {
    let start = tokens[i].span.start as usize;
    let mut end = tokens[i].span.end as usize;
    let mut depth = 0usize;
    for t in tokens[i..].iter().take(400) {
        match t.tok {
            Tok::Punct(P::LParen | P::LBracket | P::DotBrace | P::DotBracket) => depth += 1,
            Tok::Punct(P::RParen | P::RBracket | P::RBrace) if depth > 0 => depth -= 1,
            Tok::Punct(P::Semi | P::LBrace) if depth == 0 => break,
            _ => {}
        }
        end = t.span.end as usize;
    }
    let mut out = String::new();
    for word in text[start..end].split_whitespace() {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
        if out.len() > SIGNATURE_BYTES {
            let cut = out.floor_char_boundary(SIGNATURE_BYTES);
            out.truncate(cut);
            out.push_str(" ...");
            break;
        }
    }
    out
}

/// The run of `//` comment lines directly above the line holding `at`.
fn doc_above(text: &str, at: usize) -> String {
    let line_start = text[..at].rfind('\n').map_or(0, |n| n + 1);
    let mut lines = Vec::new();
    for line in text[..line_start].lines().rev() {
        let Some(comment) = line.trim().strip_prefix("//") else {
            break;
        };
        lines.push(comment.strip_prefix(' ').unwrap_or(comment).trim_end());
        if lines.len() >= DOC_LINES {
            break;
        }
    }
    lines.reverse();
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(scan: &FileScan) -> Vec<(&str, CompletionKind, Visibility)> {
        scan.decls
            .iter()
            .map(|d| (d.name.as_str(), d.kind, d.visibility))
            .collect()
    }

    #[test]
    fn top_level_declarations_by_kind_and_scope() {
        let text = "\
#import \"Basic\";
M :: #import \"Math\";
// Prints things.
// Twice.
twice :: (s: string) -> int { inner :: 1; return 0; }
Point :: struct { x: int; }
Color :: enum { RED; }
Callback :: #type (x: int);
SIX :: (1 + 5);
counter: int;
total := 0;
lib :: #library \"foo\";
#if OS == .WINDOWS {
    win_only :: () {}
} else {
    elsewhere :: () {}
}
#scope_module
internal :: () {}
#scope_file
hidden :: () {}
";
        let scan = scan(text);
        use CompletionKind::*;
        use Visibility::{Export, Module as ModuleScope};
        assert_eq!(
            names(&scan),
            [
                ("twice", Function, Export),
                ("Point", Struct, Export),
                ("Color", Enum, Export),
                ("Callback", TypeAlias, Export),
                ("SIX", Constant, Export),
                ("counter", Variable, Export),
                ("total", Variable, Export),
                ("win_only", Function, Export),
                ("elsewhere", Function, Export),
                ("internal", Function, ModuleScope),
            ]
        );
        assert_eq!(scan.decls[0].signature, "twice :: (s: string) -> int");
        assert_eq!(scan.decls[0].doc, "Prints things.\nTwice.");
        let imports: Vec<(&str, Option<&str>)> = scan
            .imports
            .iter()
            .map(|d| (d.path.as_str(), d.name.as_deref()))
            .collect();
        assert_eq!(imports, [("Basic", None), ("Math", Some("M"))]);
        assert_eq!(
            &text[..scan.imports[1].end],
            "#import \"Basic\";\nM :: #import \"Math\";"
        );
        assert!(!scan.declares_main);
    }

    #[test]
    fn loads_main_and_target_asserts() {
        let scan = scan(
            "#assert OS == .MACOS || OS == .IOS \"Apple only\";\n#load \"a/b.jai\";\nmain :: () {}\n",
        );
        assert_eq!(scan.only_os, Some(vec!["MACOS".into(), "IOS".into()]));
        assert_eq!(scan.loads[0].path, "a/b.jai");
        assert!(scan.declares_main);
        assert_eq!(super::scan("#assert(OS != .WINDOWS);").only_os, None);
    }
}
