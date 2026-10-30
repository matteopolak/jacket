//! Where a declaration's docs are written when it only renames another declaration.
use crate::docs::{self, Resolver};
use crate::intern::Sym;
use crate::sema::Compiler;
use crate::source::{FileId, Span};
use std::rc::Rc;

/// The type checker as the resolver of [`docs::docs_site`].
struct Checked<'a>(&'a mut Compiler);

impl Resolver<FileId> for Checked<'_> {
    fn text(&mut self, file: &FileId) -> Option<Rc<str>> {
        Some(self.0.src.sources.try_get(*file)?.text.clone())
    }

    fn resolve(&mut self, file: &FileId, at: usize, path: &[String]) -> Option<(FileId, usize)> {
        let scope = self.0.ide_scope_at(*file, at as u32)?;
        let names: Vec<Sym> = path.iter().map(|n| Sym::intern(n)).collect();
        let found = self.0.ide_resolve_path(scope, &names);
        let span = found.first()?;
        Some((span.file, span.start as usize))
    }
}

impl Compiler {
    /// The declaration whose docs `decl` (the span of a declared name) shows: itself when it has
    /// docs of its own or is not an alias, else what the alias names, along a chain of aliases
    /// (`hello :: Impl.hello;`). `Span::NONE` and files with no source stay as they are.
    pub fn ide_doc_site(&mut self, decl: Span) -> Span {
        if decl == Span::NONE || self.src.sources.try_get(decl.file).is_none() {
            return decl;
        }
        let (file, at) = docs::docs_site(&mut Checked(self), decl.file, decl.start as usize);
        if (file, at) == (decl.file, decl.start as usize) {
            return decl;
        }
        let named = self
            .src
            .sources
            .try_get(file)
            .and_then(|s| {
                s.text.get(at..).map(|rest| {
                    rest.find(|c: char| !crate::lexicon::is_identifier_char(c))
                        .unwrap_or(rest.len())
                })
            })
            .unwrap_or(0);
        let end = (at + named) as u32;
        Span {
            file,
            start: at as u32,
            end,
        }
    }
}
