//! Resolve documentation type references in source scope and preserve ordered alias hops.
use super::super::refs::{BUILTIN_TYPES, TypeRef, type_paths};
use super::{Item, MAX_HOPS, Reader, RefChain, TopIndex, members, slice};
use crate::ast::{Decl, DeclKind, Expr, ExprKind, StmtKind, UnOp};
use crate::docs::{self, Resolver};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

impl Reader<'_> {
    /// The declarations the names written in `ty` (in `file`) stand for. A name that is not
    /// declared anywhere the reader can see (a type parameter, a builtin) has none. For
    /// `Kind.MEMBER` that is the declaration of `Kind`.
    pub(super) fn type_refs(&mut self, file: &Path, ty: &str) -> Vec<TypeRef> {
        let mut out: Vec<TypeRef> = Vec::new();
        for path in type_paths(ty) {
            if path.len() == 1 && BUILTIN_TYPES.contains(&path[0].as_str()) {
                continue;
            }
            for len in (1..=path.len()).rev() {
                let written = path[..len].join(".");
                if out.iter().any(|r| r.path == written) {
                    break;
                }
                let key = (file.to_path_buf(), written.clone());
                let found = if let Some(found) = self.refs.get(&key) {
                    found.clone()
                } else {
                    let found = self.declaration_of(file, &path[..len]);
                    self.refs.insert(key, found.clone());
                    found
                };
                if !found.is_empty() {
                    for (file, line, private, at) in found {
                        out.push(TypeRef {
                            path: written.clone(),
                            private,
                            file,
                            line,
                            at,
                        });
                    }
                    break;
                }
            }
        }
        out
    }

    /// Where `path` is declared, looking from `file`, then from the library's entry, then in
    /// every file of the library.
    fn locate(&mut self, file: &Path, path: &[String]) -> Option<(PathBuf, usize)> {
        let entry = self.module_file.clone();
        // A re-export's signature is written in its provider module's scope. Files loaded
        // there can use imports declared in that module's entry, not just our own entry.
        let provider = file.parent().and_then(|parent| {
            parent
                .ancestors()
                .map(crate::paths::module_entry)
                .find(|entry| entry.is_file() && entry != file)
        });
        if let Some(found) = std::iter::once(file)
            .chain(provider.as_deref())
            .chain(std::iter::once(entry.as_path()))
            .find_map(|from| self.files.type_site(from, path))
        {
            return Some(found);
        }
        let key = path.join(".");
        if let Some(found) = self.anywhere.get(&key) {
            return found.clone();
        }
        let all = self.library.clone();
        let found = all
            .iter()
            .find_map(|from| self.files.type_site(from, path))
            .or_else(|| self.files.implicit_site(path));
        self.anywhere.insert(key, found.clone());
        found
    }

    /// Where `path`, written in `file`, is declared, after following aliases. A library's files
    /// share their names, so an alias whose target is named in another file is followed from
    /// the library's other files.
    fn declaration_of(&mut self, file: &Path, path: &[String]) -> RefChain {
        let Some(mut site) = self.locate(file, path) else {
            return Vec::new();
        };
        let mut seen = BTreeSet::new();
        let mut chain = Vec::new();
        for _ in 0..MAX_HOPS {
            if !seen.insert(site.clone()) {
                break;
            }
            let Some(text) = self.files.text(&site.0) else {
                break;
            };
            // Prelude declarations are language-provided, and the site deliberately has no
            // prelude module. Private declarations likewise need no public page.
            let private = self.files.unpublished(&site.0)
                || self
                    .parsed(&site.0)
                    .is_some_and(|p| p.private.contains(&site.1));
            chain.push((site.0.clone(), self.line_of(&text, site.1), private, site.1));
            let Some(next) =
                docs::alias_target(&text, site.1).and_then(|path| self.locate(&site.0, &path))
            else {
                break;
            };
            site = next;
        }
        chain
    }

    /// Classify a value by the declaration it names, including imported polymorphic types.
    pub(super) fn value_is_type(&mut self, file: &Path, expr: &Expr, depth: usize) -> bool {
        if depth == 0 {
            return false;
        }
        match &expr.kind {
            ExprKind::Struct(_)
            | ExprKind::Enum(_)
            | ExprKind::ProcType(_)
            | ExprKind::ArrayType {
                ..
            }
            | ExprKind::TypeDirective {
                ..
            } => true,
            ExprKind::UnknownDirective(directive)
                if matches!(
                    directive.kind,
                    crate::ast::DirectiveKind::Context | crate::ast::DirectiveKind::JaicType
                ) =>
            {
                true
            }
            ExprKind::Unary(UnOp::Star, inner) => self.value_is_type(file, inner, depth - 1),
            ExprKind::Call {
                callee, ..
            } => self.value_is_type(file, callee, depth - 1),
            ExprKind::Ident(name) if BUILTIN_TYPES.contains(&name.as_str()) => true,
            ExprKind::Ident(_) | ExprKind::Member(..) => {
                let Some(parsed) = self.parsed(file) else {
                    return false;
                };
                let written = slice(&parsed.text, expr.span.start, expr.span.end);
                let path = written
                    .split('.')
                    .map(|s| s.trim().to_owned())
                    .collect::<Vec<_>>();
                let Some((owner, at)) = self.locate(file, &path) else {
                    return false;
                };
                let Some(target) = self.parsed(&owner) else {
                    return false;
                };
                let index = TopIndex::new(&target);
                index
                    .at
                    .get(&at)
                    .and_then(|decl| decl.value.as_ref())
                    .is_some_and(|value| self.value_is_type(&owner, value, depth - 1))
            }
            _ => false,
        }
    }

    /// Resolve member types in their containing declaration's scope. A nested type's
    /// declaration has no separate item page, so link to the containing type's page.
    pub(super) fn item_members(&mut self, file: &Path, text: &str, decl: &Decl, item: &mut Item) {
        item.members = members(decl, text);
        for member in &mut item.members {
            member.refs = self.type_refs(file, &member.ty);
        }
        let Some(Expr {
            kind: ExprKind::Struct(lit),
            ..
        }) = &decl.value
        else {
            return;
        };
        for member in &mut item.members {
            for path in type_paths(&member.ty) {
                let nested = lit.body.iter().find_map(|s| match &s.kind {
                    StmtKind::Decl(d)
                        if d.kind == DeclKind::Const
                            && d.names.iter().any(|n| n.name.as_str() == path[0]) =>
                    {
                        Some(d)
                    }
                    _ => None,
                });
                if nested
                    .and_then(|d| d.value.as_ref())
                    .is_some_and(|v| self.value_is_type(file, v, MAX_HOPS))
                {
                    member.refs.insert(
                        0,
                        TypeRef {
                            path: path.join("."),
                            file: item.file.clone(),
                            line: item.line,
                            at: 0,
                            private: false,
                        },
                    );
                }
            }
        }
    }

    /// The declaration an alias at `at` of `file` finally names (itself when it names nothing
    /// that can be found).
    pub(super) fn target(&mut self, file: &Path, at: usize) -> (PathBuf, usize) {
        let (mut file, mut at) = (file.to_path_buf(), at);
        for _ in 0..MAX_HOPS {
            let Some(text) = self.files.text(&file) else {
                break;
            };
            let Some(path) = docs::alias_target(&text, at) else {
                break;
            };
            match self.locate(&file, &path) {
                Some(next) if next != (file.clone(), at) => (file, at) = next,
                _ => break,
            }
        }
        (file, at)
    }
}
