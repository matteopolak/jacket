//! Go to definition, path resolution and struct layout notes.
use super::facts::{IdeFieldSite, IdeReceiver, IdeWhat};
use crate::sema::scope::{EntityId, EntityKind, Found, Resolved, ScopeId, ScopeKind};
use crate::sema::*;
use crate::types::{StructId, TypeKind};

impl Compiler {
    /// Struct `s` was laid out: record where its named fields are, and move the fields of its
    /// anonymous `struct { ... }` / `union { ... }` members into it. The fields of a
    /// polymorphic struct's instance are recorded as having no one layout: their source is the
    /// polymorphic definition.
    pub(in crate::sema) fn ide_note_layout(&mut self, s: StructId) {
        let info = self.types.struct_info(s);
        let instance = !info.poly_args.is_empty();
        let mut sites = Vec::new();
        let mut absorbed = Vec::new();
        for (field, f) in info.fields.iter().enumerate() {
            if f.name.is_some() {
                let site = IdeFieldSite {
                    owner: s,
                    declaring: s,
                    field,
                    offset: f.offset,
                };
                sites.push((f.span, site));
            } else if f.using
                && let TypeKind::Struct(inner) = *self.types.kind(f.ty)
                && self.types.struct_info(inner).name.as_str() == "anonymous"
            {
                absorbed.push((inner, f.offset));
            }
        }
        let sites: Vec<_> = sites
            .into_iter()
            .filter(|(span, _)| span.end > span.start && self.ide_wants(span.file))
            .collect();
        let names: Vec<Span> = sites
            .iter()
            .filter_map(|(span, site)| {
                let name = self.types.struct_info(site.declaring).fields[site.field].name?;
                Some(self.ide_name_span(*span, name))
            })
            .collect();
        let Some(ide) = self.ide.as_mut() else {
            return;
        };
        for name in names {
            ide.member_uses.insert((name, name));
        }
        for (span, site) in sites {
            let site = (!instance).then_some(site);
            ide.fields
                .entry(span)
                .and_modify(|old| {
                    if *old != site {
                        *old = None;
                    }
                })
                .or_insert(site);
        }
        for (inner, base) in absorbed {
            for entry in ide.fields.values_mut() {
                if let Some(site) = entry
                    && site.owner == inner
                {
                    site.owner = s;
                    site.offset += base;
                    if instance {
                        *entry = None;
                    }
                }
            }
        }
    }

    /// Declarations the identifier at `offset` names: an entity's declaration, each procedure
    /// of an overload set, a struct or enum. Spans are narrowed to the declared name when it
    /// starts the declaration (`name :: ...`).
    pub fn ide_definition(&mut self, file: FileId, offset: u32) -> Vec<Span> {
        if let Some((_, ids)) = self.ide_filter_targets(file, offset) {
            return self.ide_narrow_decls(
                ids.iter()
                    .map(|&id| (self.entity(id).span, self.entity(id).name.to_string()))
                    .collect(),
            );
        }
        if let Some(selection) = self.ide.as_ref().and_then(|ide| {
            (0..3).find_map(|delta| {
                let start = offset.checked_sub(delta)?;
                let end = start.checked_add(3)?;
                ide.for_expansions.get(&Span {
                    file,
                    start,
                    end,
                })
            })
        }) {
            let spans = selection
                .as_ref()
                .into_iter()
                .flatten()
                .map(|&p| (self.proc(p).span, self.proc(p).name.to_string()))
                .collect();
            let mut definitions = self.ide_narrow_decls(spans);
            definitions.sort_unstable_by_key(|s| (s.file.0, s.start, s.end));
            definitions.dedup();
            return definitions;
        }
        // A struct field or enum member: the declaration its use was matched to.
        if let Some(ide) = self.ide.as_ref()
            && let Some((_, decl)) = ide.member_use_at(file, offset)
            && decl.end > decl.start
        {
            return vec![decl];
        }
        let Some(r) = self.ide.as_ref().and_then(|ide| {
            ide.refs_at(file, offset)
                .into_iter()
                .min_by_key(|r| r.span.end - r.span.start)
                .cloned()
        }) else {
            return Vec::new();
        };
        // A module's name (`B` of `B.print`): the start of its entry file.
        if let IdeWhat::Module(m) = r.what {
            return self.modules[m.0 as usize]
                .files
                .first()
                .map(|&file| Span {
                    file,
                    start: 0,
                    end: 0,
                })
                .into_iter()
                .collect();
        }
        let name = self.sources.snippet(r.span).to_string();
        let spans: Vec<(Span, String)> = match &r.what {
            IdeWhat::Entity(e) => vec![(self.entity(*e).span, name)],
            // Each under its own name: an overload set can include aliases.
            // A call that resolved to one overload goes to that one.
            IdeWhat::Procs(ps) => match self.ide_resolved_overload(r.span, ps) {
                Some(p) => vec![(self.proc(p).span, self.proc(p).name.to_string())],
                None => ps
                    .iter()
                    .map(|&p| (self.proc(p).span, self.proc(p).name.to_string()))
                    .collect(),
            },
            IdeWhat::Type(t) => match self.types.kind(*t) {
                TypeKind::Struct(s) => vec![(self.types.struct_info(*s).span, name)],
                TypeKind::Enum(e) => vec![(self.types.enum_info(*e).span, name)],
                _ => match self.ide.as_ref().and_then(|i| i.type_names.get(&r.span)) {
                    Some(&e) => vec![(self.entity(e).span, name)],
                    None => Vec::new(),
                },
            },
            IdeWhat::Member(_) | IdeWhat::Module(_) => Vec::new(),
        };
        self.ide_narrow_decls(spans)
    }

    /// The name inside each declaration span (see `ide_definition`); spans in no file drop out.
    pub fn ide_narrow_decls(&self, spans: Vec<(Span, String)>) -> Vec<Span> {
        spans
            .into_iter()
            // Builtins such as `OS` are declared nowhere: `Span::NONE` has no file. The slicing
            // below needs a range of whole characters in it.
            .filter(|(s, _)| {
                self.sources
                    .try_get(s.file)
                    .is_some_and(|f| f.text.get(s.start as usize..s.end as usize).is_some())
            })
            .map(|(s, name)| {
                let text = self.sources.snippet(s);
                if let Some(at) = text.find(name.as_str())
                    && text[..at].trim().is_empty()
                {
                    return Span {
                        file: s.file,
                        start: s.start + at as u32,
                        end: s.start + (at + name.len()) as u32,
                    };
                }
                // A loop's span covers `for name: ...`: its `name :` inside.
                let word = |c: char| c == '_' || c.is_alphanumeric();
                if text.starts_with("for")
                    && let Some((at, _)) = text.match_indices(name.as_str()).find(|&(at, _)| {
                        text[at + name.len()..].trim_start().starts_with(':')
                            && !text[..at].ends_with(word)
                            && !text[at + name.len()..].starts_with(word)
                    })
                {
                    return Span {
                        file: s.file,
                        start: s.start + at as u32,
                        end: s.start + (at + name.len()) as u32,
                    };
                }
                // A procedure's span starts at its literal: find `name :` earlier on the line.
                let source = self.sources.get(s.file);
                let file = &source.text;
                let line = source.lines().line_start(s.start as usize);
                let found = file[line..s.start as usize]
                    .rmatch_indices(name.as_str())
                    .find(|&(at, _)| {
                        let rest = &file[line + at + name.len()..];
                        rest.trim_start().starts_with(':')
                            && !file[..line + at]
                                .ends_with(|c: char| c == '_' || c.is_alphanumeric())
                    });
                match found {
                    Some((at, _)) => Span {
                        file: s.file,
                        start: (line + at) as u32,
                        end: (line + at + name.len()) as u32,
                    },
                    None => Span {
                        end: s.start,
                        ..s
                    },
                }
            })
            .collect()
    }

    /// The declarations `names` (`print`, `Basic.print`, `Type.field`, `Enum.MEMBER`) name
    /// from `scope`, as the name spans `ide_definition` gives. Doc comment links use it.
    pub fn ide_resolve_path(&mut self, scope: ScopeId, names: &[Sym]) -> Vec<Span> {
        let Some((last, first)) = names.split_last() else {
            return Vec::new();
        };
        let found = if first.is_empty() {
            self.lookup_full(scope, *last).ok()
        } else {
            match self.ide_receiver(scope, first) {
                Some(IdeReceiver::Module(m)) => self.module_member_lookup(m, *last).ok(),
                Some(IdeReceiver::Type(t) | IdeReceiver::Value(t)) => {
                    let member = self.ide_member_decl(t, *last);
                    if member.is_some() {
                        return member.into_iter().collect();
                    }
                    return self.ide_struct_scope_decls(t, *last);
                }
                None => return self.ide_poly_struct_member(scope, first, *last),
            }
        };
        let Some(Found::Entities(ids)) = found else {
            return Vec::new();
        };
        let spans = ids
            .into_iter()
            .map(|id| (self.entity(id).span, last.to_string()))
            .collect();
        self.ide_narrow_decls(spans)
    }

    /// `[Ring.items]` where `Ring` is a polymorphic struct: a parameter or a declaration of its
    /// body, found in its source, so it resolves whether or not the program instantiates it.
    fn ide_poly_struct_member(&mut self, scope: ScopeId, path: &[Sym], name: Sym) -> Vec<Span> {
        let [only] = path else {
            return Vec::new();
        };
        let Ok(Found::Entities(ids)) = self.lookup_full(scope, *only) else {
            return Vec::new();
        };
        let Some(&id) = ids.first() else {
            return Vec::new();
        };
        let Ok(Resolved::PolyStruct(p)) = self.resolve_entity(id) else {
            return Vec::new();
        };
        let lit = self.poly_structs[p.0 as usize].lit.clone();
        let params = lit.params.iter().filter_map(|p| p.name.as_ref());
        let fields = lit.body.iter().flat_map(|stmt| match &stmt.kind {
            ast::StmtKind::Decl(decl) => decl.names.as_slice(),
            _ => &[],
        });
        params
            .chain(fields)
            .filter(|ident| ident.name == name)
            .map(|ident| ident.span)
            .take(1)
            .collect()
    }

    /// What `name` is inside the struct `t`: constants, procedures and types declared in its
    /// body (`Type :: struct { CONST :: 3; }` is `Type.CONST`).
    fn ide_struct_scope_decls(&self, t: TypeId, name: Sym) -> Vec<Span> {
        let t = self.ide_deref(t);
        let spans = self
            .struct_scope_entities(t, name)
            .into_iter()
            .map(|id| (self.entity(id).span, name.to_string()))
            .collect();
        self.ide_narrow_decls(spans)
    }

    /// The entities named `name` in the scopes of the struct `t` (a polymorphic struct has one
    /// per instance; they all declare the same names).
    pub(super) fn struct_scope_entities(&self, t: TypeId, name: Sym) -> Vec<EntityId> {
        let mut out: Vec<EntityId> = Vec::new();
        for i in 0..self.scopes.len() {
            let s = ScopeId(i as u32);
            if self.scope(s).kind != ScopeKind::Struct(t) {
                continue;
            }
            if let Some(ids) = self.scope(s).names.get(&name) {
                out.extend(ids.iter().copied().filter(|&id| {
                    matches!(
                        &self.entity(id).kind,
                        EntityKind::Decl { decl, .. } if decl.kind == ast::DeclKind::Const
                    )
                }));
            }
        }
        out
    }
}
