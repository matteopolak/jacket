//! Editor features built on the compiler's metaprogramming and call facts
//! (`jaic::sema::ide_meta`) and on the syntax layer: expansion hovers and documents, inlay
//! hints, code actions, format strings, references, signature help, folding, code lenses and
//! workspace symbols.
mod actions;
mod formats;
mod hovers;
mod links;
mod navigation;
mod signature;
mod symbols;

use crate::analysis::Span;
use crate::semantic;
use crate::session::{Session, contains};
use crate::{DocumentUri, Location, Range, TextEdit};
use jaic::fxhash::HashSet;
use jaic::parser::ide_facts;
use jaic::sema::ide_meta::{IdeCallInfo, IdeExpansion, IdeExpansionKind};
use jaic::source::FileId;
use std::path::Path;
use std::rc::Rc;

/// URI scheme of expansion documents.
pub const EXPANSION_SCHEME: &str = "jai-expansion://";

/// Longest inlay hint label.
const HINT_CHARS: usize = 40;

fn kind_name(kind: IdeExpansionKind) -> &'static str {
    match kind {
        IdeExpansionKind::Insert => "insert",
        IdeExpansionKind::Run => "run",
        IdeExpansionKind::If => "if",
        IdeExpansionKind::Macro => "macro",
    }
}

fn span_of(e: &IdeExpansion) -> Span {
    Span::new(e.span.start as usize, e.span.end as usize)
}

/// The directive word at `at` of `text` (`insert` for `#insert`), if one starts there.
fn directive_at(text: &str, at: usize) -> Option<&str> {
    let rest = text.get(at..)?.strip_prefix('#')?;
    let end = rest
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    Some(&rest[..end])
}

fn clip(text: &str, chars: usize) -> String {
    let one_line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if one_line.chars().count() > chars {
        let cut: String = one_line.chars().take(chars).collect();
        format!("{cut}…")
    } else {
        one_line
    }
}

/// Rename edits grouped by document URI.
pub type RenameEdits = Vec<(String, Vec<TextEdit>)>;

impl Session {
    /// Run `query` on the type-checked program, the document's text.
    pub(crate) fn checked<T>(
        &self,
        uri: &DocumentUri,
        query: impl FnOnce(&mut semantic::Analysis, FileId) -> T,
    ) -> Option<T> {
        let doc = self.document(uri).ok()?;
        self.with_semantic(uri, &doc.text, |a, path| {
            let file = a.file(path)?;
            Some(query(a, file))
        })
    }

    /// `span` of an open document as a range, if it lies on character boundaries.
    pub(crate) fn range_of(&self, uri: &DocumentUri, span: Span) -> Option<Range> {
        let doc = self.document(uri).ok()?;
        if span.end > doc.text.len() || span.start > span.end {
            return None;
        }
        doc.index.range(&doc.text, span).ok()
    }

    /// A location in any compiled file (an open document's own index when it is open).
    pub(crate) fn location_in(
        &self,
        path: &str,
        text: &Rc<str>,
        start: usize,
        end: usize,
    ) -> Option<Location> {
        let target = DocumentUri::from_path(Path::new(path)).ok()?;
        let span = Span::new(start, end);
        let range = match self.documents.get(&target) {
            Some(open) if open.same_positions(text) => open.index.range(&open.text, span).ok()?,
            _ => crate::position::LineIndex::new(text)
                .range(text, span)
                .ok()?,
        };
        Some(Location {
            uri: target.as_str().into(),
            range,
        })
    }

    fn expansions(&self, uri: &DocumentUri) -> Vec<IdeExpansion> {
        self.checked(uri, |a, f| a.compiler.ide_expansions(f))
            .unwrap_or_default()
    }

    fn calls(&self, uri: &DocumentUri) -> Vec<IdeCallInfo> {
        self.checked(uri, |a, f| a.compiler.ide_calls(f))
            .unwrap_or_default()
    }

    /// The expansion a cursor at `byte` is on: the smallest one containing it.
    fn expansion_at(&self, uri: &DocumentUri, byte: usize) -> Option<IdeExpansion> {
        self.expansions(uri)
            .into_iter()
            .filter(|e| contains(span_of(e), byte))
            .min_by_key(|e| e.span.end - e.span.start)
    }
}

/// The open documents over the environment's file system, for resolving links.
struct OpenFiles<'a> {
    session: &'a Session,
}

impl OpenFiles<'_> {
    fn open(&self, path: &Path) -> bool {
        self.session.open_file_uri(path).is_some()
    }
}

impl jaic::sema::FileSystem for OpenFiles<'_> {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        self.session.environment.as_ref()?.fs.read(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.open(path)
            || self
                .session
                .environment
                .as_ref()
                .is_some_and(|e| e.fs.is_file(path))
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.session
            .environment
            .as_ref()
            .is_some_and(|e| e.fs.is_dir(path))
    }
}

/// Where the names end that a declaration without a written type introduces (`x := 1`, both
/// names of `a, b := f()`), read from the tokens so it holds wherever `text` stopped parsing.
fn inferred_name_ends(text: &str) -> HashSet<u32> {
    let tokens = jaic::lexer::lex_ide(FileId(0), text).0;
    ide_facts::inferred_name_spans(&tokens)
        .iter()
        .map(|s| s.end)
        .collect()
}

/// A literal argument (a parameter name hint helps there).
fn literal(source: &str) -> bool {
    let s = source.trim_start_matches('-');
    s.starts_with(|c: char| c.is_ascii_digit())
        || s.starts_with('"')
        || matches!(s, "true" | "false" | "null")
        || s.starts_with("#char")
        || (s.starts_with('.') && s[1..].chars().all(jaic::lexicon::is_identifier_char))
}

/// `#insert` at `span` replaced by its code `text`: as statements (through the `;`) in
/// statement position, else as a parenthesized expression.
fn inline_insert(text: &str, span: Span, code: &str) -> Option<(Span, String)> {
    let before = text[..span.start].trim_end();
    let statement = before.is_empty() || before.ends_with([';', '{', '}']);
    if statement {
        let rest = &text[span.end..];
        let semi = rest.trim_start_matches([' ', '\t']);
        let end = if semi.starts_with(';') {
            span.end + (rest.len() - semi.len()) + 1
        } else {
            span.end
        };
        let line_start = jaic::source::line_start(text, span.start);
        let indent: String = text[line_start..span.start]
            .chars()
            .take_while(|c| c.is_whitespace())
            .collect();
        let body = jaic::sema::ide_meta::dedent(code.trim_matches('\n'));
        let body = body
            .lines()
            .collect::<Vec<_>>()
            .join(&format!("\n{indent}"));
        return Some((Span::new(span.start, end), body));
    }
    let code = code.trim().trim_end_matches(';').trim();
    if code.is_empty() {
        return None;
    }
    Some((span, format!("({code})")))
}

/// The call whose argument list the cursor at `byte` is in: the `(` offset, the callee chain
/// (`Module.name` → [Module, name]), the argument index, and the name of a `name =` argument.
fn open_call(text: &str, byte: usize) -> Option<(usize, Vec<&str>, usize, Option<String>)> {
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut commas = 0usize;
    let mut at = byte.min(bytes.len());
    let mut current_start = byte;
    let limit = byte.saturating_sub(4096);
    while at > limit {
        at -= 1;
        match bytes[at] {
            b')' | b']' | b'}' => depth += 1,
            b'(' | b'[' | b'{' if depth > 0 => depth -= 1,
            b'(' => break,
            b'[' | b'{' | b';' => return None,
            b',' if depth == 0 => {
                if commas == 0 {
                    current_start = at + 1;
                }
                commas += 1;
            }
            b'"' => {
                // Skip back over a string literal on this line.
                let line = jaic::source::line_start(text, at);
                at = text[line..at].rfind('"').map_or(at, |q| line + q);
            }
            _ => {}
        }
    }
    if bytes.get(at) != Some(&b'(') {
        return None;
    }
    if commas == 0 {
        current_start = at + 1;
    }
    let ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b >= 0x80;
    let mut chain = Vec::new();
    let mut end = at;
    loop {
        let mut s = end;
        while s > 0 && ident(bytes[s - 1]) {
            s -= 1;
        }
        if s == end || bytes[s].is_ascii_digit() {
            break;
        }
        chain.push(&text[s..end]);
        if s > 0 && bytes[s - 1] == b'.' {
            end = s - 1;
        } else {
            break;
        }
    }
    if chain.is_empty() || matches!(chain[0], "if" | "while" | "for" | "cast" | "case" | "ifx") {
        return None;
    }
    chain.reverse();
    let current = text[current_start..byte].trim_start();
    let named = current.split_once('=').and_then(|(n, rest)| {
        let n = n.trim();
        (!rest.starts_with('=')
            && !n.is_empty()
            && n.chars().all(jaic::lexicon::is_identifier_char))
        .then(|| n.to_string())
    });
    Some((at, chain, commas, named))
}

/// The type written in a parameter snippet (`name: Type = default`), or `None` for `..` varargs.
fn param_type(param: &str) -> Option<&str> {
    let ty = param.split_once(':').map_or(param, |(_, ty)| ty);
    let ty = ty.split_once('=').map_or(ty, |(ty, _)| ty).trim();
    (!ty.starts_with("..")).then_some(ty)
}

/// Which parameters share their type with another one. Only those get name hints: in
/// `print(format: string, args: ..Any)` the literal's role is obvious, in
/// `clamp(x: float, lo: float, hi: float)` it is not.
fn ambiguous_params(params: &[String]) -> Vec<bool> {
    let types: Vec<Option<&str>> = params.iter().map(|p| param_type(p)).collect();
    types
        .iter()
        .map(|ty| ty.is_some_and(|ty| types.iter().filter(|other| **other == Some(ty)).count() > 1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_calls_find_the_callee_and_argument() {
        let text = "x := Math.clamp(a, f(b, c), ";
        let (open, chain, arg, named) = open_call(text, text.len()).unwrap();
        assert_eq!(&text[open..=open], "(");
        assert_eq!(chain, ["Math", "clamp"]);
        assert_eq!(arg, 2);
        assert_eq!(named, None);
        let text = "f(1, scale = ";
        assert_eq!(
            open_call(text, text.len()).unwrap().3.as_deref(),
            Some("scale")
        );
        assert!(open_call("if (a", 5).is_none());
    }

    #[test]
    fn inline_insert_keeps_indentation() {
        let text = "f :: () {\n    #insert \"a := 1;\\nb := 2;\";\n}";
        let start = text.find("#insert").unwrap();
        let end = text.find(";\n}").unwrap();
        let (span, body) = inline_insert(text, Span::new(start, end), "a := 1;\nb := 2;").unwrap();
        assert_eq!(&text[span.end - 1..span.end], ";");
        assert_eq!(body, "a := 1;\n    b := 2;");
    }

    #[test]
    fn inferred_declarations() {
        let text = "f :: () {\n    x := 1;\n    a, b := g(\",\");\n    y : int = 1;\n    x = 2;\n}\nS :: struct { z := 3; }\n";
        let ends = inferred_name_ends(text);
        let end =
            |name: &str, n: usize| (text.match_indices(name).nth(n).unwrap().0 + name.len()) as u32;
        assert!(ends.contains(&end("x", 0)) && ends.contains(&end("a", 0)));
        assert!(ends.contains(&end("b", 0)) && ends.contains(&end("z", 0)));
        assert!(!ends.contains(&end("y", 0)));
        assert_eq!(ends.len(), 4);
    }
}
