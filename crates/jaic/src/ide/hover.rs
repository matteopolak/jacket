//! Hover text and struct layouts at a position.
use super::completion::clip;
use super::definition::IdeModuleName;
use super::facts::{IdeFieldSite, IdeHover, IdeLayout, IdeRef, IdeWhat, same_what};
use crate::intern::Sym;
use crate::sema::scope::{EntityId, EntityKind, EntityState, Resolved, ScopeId, ScopeKind};
use crate::sema::value::ProcId;
use crate::sema::{Compiler, value};
use crate::source::FileId;
use crate::source::Span;
use crate::types::TypeId;
use crate::types::{StructId, TypeKind};

impl Compiler {
    /// The innermost recorded scope at `offset` of `file` (else the file's own scope).
    pub fn ide_scope_at(&self, file: FileId, offset: u32) -> Option<ScopeId> {
        let inner = self.ed.facts().and_then(|ide| {
            ide.scopes
                .iter()
                .filter(|(s, _)| s.file == file && s.start <= offset && offset <= s.end)
                .min_by_key(|(s, _)| s.end - s.start)
                .map(|(_, id)| *id)
        });
        inner.or_else(|| {
            (0..self.scopes.len())
                .map(|i| ScopeId(i as u32))
                .find(|&s| {
                    self.scope(s).kind == ScopeKind::File && self.scope(s).file == Some(file)
                })
        })
    }

    /// The innermost recorded name at `offset` of `file`, and whether every record of that
    /// exact span agrees (a polymorphic body is checked once per instance).
    fn ide_hover_ref(&self, file: FileId, offset: u32) -> Option<(IdeRef, bool)> {
        let facts = self.ed.facts()?;
        let mut here = facts.refs_at(file, offset);
        // A polymorph whose instances failed is also sketched with stand-in types: what an
        // instance recorded is what the program means.
        if here.iter().any(|r| !facts.sketched(r)) {
            here.retain(|r| !facts.sketched(r));
        }
        let r = here
            .iter()
            .copied()
            .min_by_key(|r| r.span.end - r.span.start)?;
        let same = here.iter().filter(|o| o.span == r.span).all(|o| {
            o.ty == r.ty && (same_what(&o.what, &r.what) || self.same_local(&o.what, &r.what))
        });
        Some((r.clone(), same))
    }

    /// Two locals for one declaration: a body checked again (after a statement in it failed)
    /// declares its parameters and locals afresh.
    fn same_local(&self, a: &IdeWhat, b: &IdeWhat) -> bool {
        let (IdeWhat::Entity(x), IdeWhat::Entity(y)) = (a, b) else {
            return false;
        };
        let (x, y) = (self.entity(*x), self.entity(*y));
        matches!(x.kind, EntityKind::Local { .. })
            && matches!(y.kind, EntityKind::Local { .. })
            && x.span == y.span
            && x.name == y.name
    }

    pub fn ide_hover(&mut self, file: FileId, offset: u32) -> Option<IdeHover> {
        self.ide_note_callee_at(file, offset);
        if let Some((span, ids)) = self.ide_filter_targets(file, offset) {
            let text = ids
                .iter()
                .map(|&id| self.ide_entity_hover(id, None))
                .collect::<Vec<_>>()
                .join("\n");
            return Some(IdeHover {
                span,
                text,
                layout: None,
            });
        }
        let Some((r, same)) = self.ide_hover_ref(file, offset) else {
            if let Some(hover) = self.ide_field_hover(file, offset) {
                return Some(hover);
            }
            return match self.ide_module_name_at(file, offset)? {
                IdeModuleName::Alias(span, module) => Some(IdeHover {
                    span,
                    text: format!(
                        "{} :: #import \"{}\"",
                        self.sources.snippet(span),
                        self.modules[module.0 as usize].name
                    ),
                    layout: None,
                }),
                IdeModuleName::Member(span, ids) => {
                    let text = ids
                        .iter()
                        .map(|&id| self.ide_entity_hover(id, None))
                        .collect::<Vec<_>>()
                        .join("\n");
                    Some(IdeHover {
                        span,
                        text,
                        layout: None,
                    })
                }
            };
        };
        let name = self.sources.snippet(r.span).to_string();
        // `Poly(args)`: the instance, not the polymorphic struct.
        if let IdeWhat::Entity(e) = r.what
            && matches!(
                self.entity(e).state,
                EntityState::Done(Resolved::PolyStruct(_))
            )
            && let Some(&Some(t)) = self.ed.facts()?.instances.get(&r.span)
        {
            let name = self.types.name(t);
            let text = self.ide_type_hover(&name, t);
            let layout = if same {
                self.ide_type_layout(t)
            } else {
                None
            };
            return Some(IdeHover {
                span: r.span,
                text,
                layout,
            });
        }
        let text = match &r.what {
            IdeWhat::Entity(e) => self.ide_entity_hover(*e, Some(r.ty)),
            IdeWhat::Member(_) => format!("{name}: {}", self.types.name(r.ty)),
            IdeWhat::Type(t) => self.ide_type_hover(&name, *t),
            // An overload set may include aliases (`print :: print_to_builder;`): show each
            // under the name used.
            IdeWhat::Procs(ps) => ps
                .iter()
                .map(|&p| format!("{name} :: {}", clip(&self.ide_header_text(p))))
                .collect::<Vec<_>>()
                .join("\n"),
            IdeWhat::Module(m) => {
                format!("{name} :: #import \"{}\"", self.modules[m.0 as usize].name)
            }
        };
        let layout = if same {
            match &r.what {
                IdeWhat::Entity(e) => self.ide_entity_layout(*e),
                IdeWhat::Member(m) => self.ide_member_layout(r.span, *m),
                IdeWhat::Type(t) => self.ide_type_layout(*t),
                IdeWhat::Procs(_) | IdeWhat::Module(_) => None,
            }
        } else {
            None
        };
        Some(IdeHover {
            span: r.span,
            text,
            layout,
        })
    }

    /// The declarations `ide_hover` describes, for reading their doc comments: one per
    /// overload, in the order of the hover's lines.
    pub fn ide_hover_decls(&mut self, file: FileId, offset: u32) -> Vec<Span> {
        if let Some((_, ids)) = self.ide_filter_targets(file, offset) {
            return self.ide_narrow_decls(
                ids.iter()
                    .map(|&id| (self.entity(id).span, self.entity(id).name.to_string()))
                    .collect(),
            );
        }
        let Some((r, _)) = self.ide_hover_ref(file, offset) else {
            let Some(site) = self.ed.facts().and_then(|ide| {
                ide.fields
                    .iter()
                    .find(|(s, _)| s.file == file && s.start <= offset && offset <= s.end)
                    .and_then(|(_, site)| *site)
            }) else {
                return Vec::new();
            };
            let f = &self.types.struct_info(site.declaring).fields[site.field];
            let named = f.name.map(|n| (f.span, n.to_string()));
            return self.ide_narrow_decls(named.into_iter().collect());
        };
        let procs: Vec<ProcId> = match &r.what {
            IdeWhat::Procs(ps) => ps.clone(),
            IdeWhat::Entity(e) => match self.entity(*e).kind {
                EntityKind::Decl {
                    ..
                }
                | EntityKind::Import(_) => match self.resolve_entity(*e).ok() {
                    Some(
                        Resolved::Proc(p)
                        | Resolved::Const {
                            value: value::Value::Proc(p),
                            ..
                        },
                    ) => vec![p],
                    Some(Resolved::ProcSet(ps)) => ps,
                    _ => Vec::new(),
                },
                _ => Vec::new(),
            },
            _ => Vec::new(),
        };
        if procs.is_empty() {
            return self.ide_definition(file, offset);
        }
        let spans = procs
            .iter()
            .map(|&p| (self.proc(p).span, self.proc(p).name.to_string()))
            .collect();
        self.ide_narrow_decls(spans)
    }

    /// The name of a field where its struct declares it. A field laid out in several structs
    /// (the instances of a polymorphic struct) has no one type and layout to show.
    fn ide_field_hover(&mut self, file: FileId, offset: u32) -> Option<IdeHover> {
        let (span, site) = self
            .ed
            .facts()?
            .fields
            .iter()
            .find(|(s, _)| s.file == file && s.start <= offset && offset <= s.end)
            .map(|(s, site)| (*s, *site))?;
        // A polymorphic struct's field: its declaration as written (`items: [N] T`).
        let Some(site) = site else {
            let file = &self.sources.try_get(span.file)?.text;
            let rest = file.get(span.start as usize..)?;
            let text = rest[..rest.find([';', '\n', '}']).unwrap_or(rest.len())].trim_end();
            return Some(IdeHover {
                span,
                text: text.to_string(),
                layout: None,
            });
        };
        let f = &self.types.struct_info(site.declaring).fields[site.field];
        let text = format!("{}: {}", f.name?, self.types.name(f.ty));
        Some(IdeHover {
            span,
            text,
            layout: Some(self.ide_field_layout(site)),
        })
    }

    /// Lay out what a hover describes (the program may not have needed its layout).
    pub(super) fn ide_laid_out(&mut self, t: TypeId) -> bool {
        let span = Span {
            file: FileId(0),
            start: 0,
            end: 0,
        };
        self.ensure_complete(t, span).is_ok()
    }

    /// Size and alignment of a type, and for a struct the bytes its fields leave unused.
    fn ide_type_layout(&mut self, t: TypeId) -> Option<IdeLayout> {
        if matches!(
            self.types.kind(t),
            TypeKind::Void
                | TypeKind::CompileTimeOnly
                | TypeKind::PolyParam
                | TypeKind::PolyStruct { .. }
        ) || !self.ide_laid_out(t)
        {
            return None;
        }
        let size = self.types.size_of(t);
        let padding = match self.types.kind(t) {
            TypeKind::Struct(s) => {
                let mut spans: Vec<(u64, u64)> = self
                    .types
                    .struct_info(*s)
                    .fields
                    .iter()
                    .map(|f| (f.offset, f.offset + self.types.size_of(f.ty)))
                    .collect();
                spans.sort_unstable();
                let mut covered = 0;
                let mut reached = 0;
                for (start, end) in spans {
                    let start = start.max(reached);
                    if end > start {
                        covered += end - start;
                        reached = end;
                    }
                }
                size.saturating_sub(covered)
            }
            _ => 0,
        };
        Some(IdeLayout {
            offset: None,
            size,
            align: self.types.align_of(t),
            padding,
        })
    }

    /// A named type's layout, or a variable's or constant's when its type is an aggregate.
    fn ide_entity_layout(&mut self, id: EntityId) -> Option<IdeLayout> {
        let ty = match &self.entity(id).kind {
            EntityKind::Local {
                ty, ..
            } => *ty,
            EntityKind::Const {
                value: value::Value::Type(t),
                ..
            } => return self.ide_type_layout(*t),
            EntityKind::Const {
                ty, ..
            } => *ty,
            EntityKind::Decl {
                ..
            } => match &self.entity(id).state {
                EntityState::Done(Resolved::Const {
                    value: value::Value::Type(t),
                    ..
                }) => return self.ide_type_layout(*t),
                EntityState::Done(
                    Resolved::Const {
                        ty, ..
                    }
                    | Resolved::Global {
                        ty, ..
                    },
                ) => *ty,
                _ => return None,
            },
            _ => return None,
        };
        if !self.ide_aggregate(ty) {
            return None;
        }
        self.ide_type_layout(ty)
    }

    /// Structs, unions, arrays, strings and `Any`: types whose size is worth showing on a value.
    fn ide_aggregate(&self, ty: TypeId) -> bool {
        match self.types.kind(ty) {
            TypeKind::Struct(_)
            | TypeKind::Array {
                ..
            }
            | TypeKind::String
            | TypeKind::Any => true,
            TypeKind::Distinct(d) => self.ide_aggregate(self.types.distincts[d.0 as usize].base),
            _ => false,
        }
    }

    /// The field `name` of the value left of the `.` at `span`.
    fn ide_member_layout(&mut self, span: Span, name: Sym) -> Option<IdeLayout> {
        let mut ty = (*self.ed.facts()?.receivers.get(&span)?)?;
        while let Some(p) = self.types.pointee(ty) {
            ty = p;
        }
        let TypeKind::Struct(s) = *self.types.kind(self.types.repr(ty)) else {
            return None;
        };
        if !self.ide_laid_out(ty) {
            return None;
        }
        let site = self.ide_find_field(s, name, 0, 0)?;
        Some(self.ide_field_layout(site))
    }

    /// Where `name` is in struct `s` (through `using` members held by value), `base` bytes
    /// into the struct the search started from.
    fn ide_find_field(
        &self,
        s: StructId,
        name: Sym,
        base: u64,
        depth: u32,
    ) -> Option<IdeFieldSite> {
        let fields = &self.types.struct_info(s).fields;
        if let Some(at) = fields.iter().position(|f| f.name == Some(name)) {
            return Some(IdeFieldSite {
                owner: s,
                declaring: s,
                field: at,
                offset: base + fields[at].offset,
            });
        }
        if depth > 16 {
            return None;
        }
        fields.iter().filter(|f| f.using).find_map(|f| {
            let TypeKind::Struct(inner) = *self.types.kind(self.types.repr(f.ty)) else {
                return None;
            };
            self.ide_find_field(inner, name, base + f.offset, depth + 1)
        })
    }

    /// A field's offset (in the struct it is reached through or declared in), size, alignment
    /// and the bytes skipped before it.
    fn ide_field_layout(&self, site: IdeFieldSite) -> IdeLayout {
        let info = self.types.struct_info(site.declaring);
        let f = &info.fields[site.field];
        let padding = if f.overlay || info.is_union {
            0
        } else {
            let before = info.fields[..site.field]
                .iter()
                .filter(|g| !g.overlay)
                .map(|g| g.offset + self.types.size_of(g.ty))
                .max()
                .unwrap_or(0);
            f.offset.saturating_sub(before)
        };
        IdeLayout {
            offset: Some(site.offset),
            size: self.types.size_of(f.ty),
            align: f.align,
            padding,
        }
    }
}
