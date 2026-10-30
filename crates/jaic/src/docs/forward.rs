//! Docs forwarded through aliases: `hello :: Impl.hello;` with no comment of its own shows the
//! docs of what it names. The rule lives here and nothing else decides it: the language server
//! and `jaic -doc_comments` both ask [`docs_site`] where a declaration's docs are, and only
//! differ in how they find what a path names (the type checker, or the library's own files).
use super::{P, Tok, trivia, valid};
use crate::lexer::Token;

/// How many aliases are followed before giving up, so a cycle ends.
const MAX_HOPS: usize = 8;

/// What a path names, for [`docs_site`]. `F` identifies a file.
pub trait Resolver<F> {
    /// The text of `file`.
    fn text(&mut self, file: &F) -> Option<std::rc::Rc<str>>;

    /// The declaration the names `path` lead to from the declaration at `at` of `file`: its file
    /// and the offset of its name.
    fn resolve(&mut self, file: &F, at: usize, path: &[String]) -> Option<(F, usize)>;
}

/// The names of what the constant declared at `at` only renames (`Impl.hello` for
/// `hello :: Impl.hello;`), or `None` when it is anything else.
pub fn alias_target(text: &str, at: usize) -> Option<Vec<String>> {
    if !valid(text, at) {
        return None;
    }
    let t = trivia(text);
    let tokens: &[Token] = &t.tokens;
    let first = tokens.partition_point(|k| (k.span.start as usize) < at);
    let name = tokens.get(first)?;
    if name.span.start as usize != at || !matches!(name.tok, Tok::Ident(_)) {
        return None;
    }
    if tokens.get(first + 1)?.tok != Tok::Punct(P::ColonColon) {
        return None;
    }
    let mut path = Vec::new();
    let mut i = first + 2;
    loop {
        let Tok::Ident(part) = &tokens.get(i)?.tok else {
            return None;
        };
        path.push(part.as_str().to_owned());
        match &tokens.get(i + 1)?.tok {
            Tok::Punct(P::Dot) => i += 2,
            Tok::Punct(P::Semi) => return Some(path),
            _ => return None,
        }
    }
}

/// Where the docs of the declaration at `at` of `file` are written: there when it has docs of its
/// own, else at what it renames (and so on along a chain of aliases), else there all the same.
pub fn docs_site<F: Clone + PartialEq>(
    resolver: &mut dyn Resolver<F>,
    file: F,
    at: usize,
) -> (F, usize) {
    let start = (file.clone(), at);
    let (mut file, mut at) = (file, at);
    for _ in 0..MAX_HOPS {
        let Some(text) = resolver.text(&file) else {
            break;
        };
        if super::declaration(&text, at).is_some() {
            return (file, at);
        }
        let Some(path) = alias_target(&text, at) else {
            break;
        };
        let Some(next) = resolver.resolve(&file, at, &path) else {
            break;
        };
        if next == (file.clone(), at) {
            break;
        }
        (file, at) = next;
    }
    start
}
