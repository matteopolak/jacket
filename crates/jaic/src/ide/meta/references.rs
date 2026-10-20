//! References, member queries and polymorphic bindings.
use super::{IdeMembers, Target};
use crate::ide::IdeWhat;
use crate::intern::Sym;
use crate::sema::scope::{EntityId, EntityKind};
use crate::sema::value::ProcId;
use crate::sema::{Compiler, Value};
use crate::source::FileId;
use crate::source::Span;
use crate::types::TypeId;
use crate::types::TypeKind;

impl Compiler {
    /// The uses of the struct field or enum member named at `offset`, with its declaration.
    fn ide_member_references(&self, file: FileId, offset: u32) -> Option<Vec<(Span, bool)>> {
        let ide = self.ed.facts()?;
        let (_, decl) = ide.member_use_at(file, offset)?;
        let mut out: Vec<(Span, bool)> = ide
            .member_uses
            .iter()
            .filter(|(_, d)| *d == decl)
            .map(|(u, d)| (*u, u == d))
            .collect();
        out.sort_by_key(|(s, _)| (s.file.0, s.start));
        Some(out)
    }

    /// For a struct field or enum member named at `offset`: whether its declaration is in the
    /// recorded files (a rename can reach it). `None` when `offset` names no member.
    pub fn ide_member_declared(&self, file: FileId, offset: u32) -> Option<bool> {
        let ide = self.ed.facts()?;
        let (_, decl) = ide.member_use_at(file, offset)?;
        Some(ide.member_uses.contains(&(decl, decl)))
    }

    /// Every recorded reference to what the identifier at `offset` names (declarations
    /// included), as (span, is the declaration). Struct fields and enum members answer from `member_uses`.
    pub fn ide_references(&self, file: FileId, offset: u32) -> Vec<(Span, bool)> {
        if let Some(members) = self.ide_member_references(file, offset) {
            return members;
        }
        let Some(at) = self.ide_ref_at(file, offset) else {
            return Vec::new();
        };
        let mut wanted = self.ide_targets(&at);
        if wanted.is_empty() {
            return Vec::new();
        }
        let Some(ide) = self.ed.facts() else {
            return Vec::new();
        };
        // A use naming an overload set reaches every overload in it, so renaming one renames the
        // name of them all: grow the targets by the sets the matching uses name.
        loop {
            let mut grew = false;
            for r in &ide.refs {
                if matches!(r.what, IdeWhat::Procs(_))
                    && self.ide_targets(r).iter().any(|t| wanted.contains(t))
                {
                    for t in self.ide_targets(r) {
                        if !wanted.contains(&t) {
                            wanted.push(t);
                            grew = true;
                        }
                    }
                }
            }
            if !grew {
                break;
            }
        }
        let mut out: Vec<(Span, bool)> = Vec::new();
        for r in &ide.refs {
            if !self.ide_targets(r).iter().any(|t| wanted.contains(t)) {
                continue;
            }
            // `$T` bound by `#bake_constants` is a use of `T` without its `$`.
            let mut span = r.span;
            let dollars = self
                .ide_snippet(span)
                .bytes()
                .take_while(|&b| b == b'$')
                .count();
            span.start = (span.start + dollars as u32).min(span.end);
            if !out.iter().any(|(s, _)| *s == span) {
                out.push((span, r.decl));
            }
        }
        // A named argument of a call or `#bake_constants` naming a declared parameter.
        for &(u, d) in &ide.member_uses {
            if u != d
                && wanted
                    .iter()
                    .any(|t| matches!(t, Target::Declared(s, _) if *s == d))
                && !out.iter().any(|(s, _)| *s == u)
            {
                out.push((u, false));
            }
        }
        out.sort_by_key(|(s, _)| (s.file.0, s.start));
        out
    }

    /// Where the type of the expression at `offset` is declared (through pointers and arrays).
    pub fn ide_type_definition(&self, file: FileId, offset: u32) -> Vec<Span> {
        let Some(r) = self.ide_ref_at(file, offset) else {
            return Vec::new();
        };
        let mut t = match &r.what {
            IdeWhat::Type(t) => *t,
            IdeWhat::Entity(e) => match &self.entity(*e).kind {
                EntityKind::Const {
                    value: Value::Type(t),
                    ..
                } => *t,
                _ => r.ty,
            },
            _ => r.ty,
        };
        for _ in 0..8 {
            t = match self.ty.types.kind(t) {
                TypeKind::Pointer(inner) => *inner,
                TypeKind::Array {
                    elem, ..
                } => *elem,
                _ => break,
            };
        }
        let span = match self.ty.types.kind(t) {
            TypeKind::Struct(s) => self.ty.types.struct_info(*s).span,
            TypeKind::Enum(e) => self.ty.types.enum_info(*e).span,
            _ => return Vec::new(),
        };
        if span.end <= span.start {
            return Vec::new();
        }
        vec![span]
    }

    /// The enum or struct at `offset`: the type the name there denotes (`as_type`), or the type
    /// of the value there. Its members are the enum's members, or the struct's named fields
    /// (`using`, `#as` and overlaid ones left out) with a value that suits each field's type.
    pub fn ide_members_at(&self, file: FileId, offset: u32, as_type: bool) -> Option<IdeMembers> {
        let r = self.ide_ref_at(file, offset)?;
        let t = if as_type {
            match &r.what {
                IdeWhat::Type(t) => *t,
                IdeWhat::Entity(e) => match &self.entity(*e).kind {
                    EntityKind::Const {
                        value: Value::Type(t),
                        ..
                    } => *t,
                    _ => return None,
                },
                _ => return None,
            }
        } else {
            r.ty
        };
        match self.ty.types.kind(t) {
            TypeKind::Enum(e) => {
                let info = self.ty.types.enum_info(*e);
                Some(IdeMembers {
                    name: info.name.to_string(),
                    is_enum: true,
                    members: info
                        .members
                        .iter()
                        .map(|(n, _)| (n.to_string(), String::new()))
                        .collect(),
                })
            }
            TypeKind::Struct(s) => {
                let info = self.ty.types.struct_info(*s);
                Some(IdeMembers {
                    name: info.name.to_string(),
                    is_enum: false,
                    members: info
                        .fields
                        .iter()
                        .filter(|f| !f.using && !f.as_ && !f.overlay)
                        .filter_map(|f| Some((f.name?.to_string(), self.ide_placeholder(f.ty))))
                        .collect(),
                })
            }
            _ => None,
        }
    }

    /// A value of type `t` to write where one is wanted.
    fn ide_placeholder(&self, t: TypeId) -> String {
        match self.ty.types.kind(t) {
            TypeKind::Bool => "false".into(),
            TypeKind::Int {
                ..
            } => "0".into(),
            TypeKind::Float {
                ..
            } => "0.0".into(),
            TypeKind::String => "\"\"".into(),
            TypeKind::Pointer(_) | TypeKind::Proc(_) | TypeKind::Null => "null".into(),
            TypeKind::Struct(_) => ".{}".into(),
            TypeKind::Array {
                ..
            } => ".[]".into(),
            TypeKind::Enum(e) => match self.ty.types.enum_info(*e).members.first() {
                Some((n, _)) => format!(".{n}"),
                None => "---".into(),
            },
            _ => "---".into(),
        }
    }

    /// Polymorphic procedures declared in `file` with the instances checking created: (the
    /// procedure's span, each instance's bindings as `T = s64`).
    pub fn ide_polymorphs(&self, file: FileId) -> Vec<(Span, String, Vec<String>)> {
        let mut out = Vec::new();
        for p in &self.pr.procs {
            if !p.is_poly || p.span.file != file || p.bindings.is_some() {
                continue;
            }
            let mut instances: Vec<String> = p
                .instances
                .values()
                .map(|&inst| self.ide_bindings_text(inst))
                .collect();
            instances.sort();
            out.push((p.span, p.name.to_string(), instances));
        }
        out
    }

    fn ide_bindings_text(&self, inst: ProcId) -> String {
        let Some(scope) = self.proc(inst).bindings else {
            return String::new();
        };
        let mut names: Vec<(Sym, EntityId)> = self
            .scope(scope)
            .names
            .iter()
            .filter_map(|(n, ids)| Some((*n, *ids.first()?)))
            .collect();
        names.sort_by_key(|(_, e)| self.entity(*e).span.start);
        names
            .into_iter()
            .map(|(n, e)| match &self.entity(e).kind {
                EntityKind::Const {
                    value, ..
                } => format!(
                    "{n} = {}",
                    self.ide_literal(value)
                        .unwrap_or_else(|| value.render(&self.ty.types))
                ),
                _ => n.to_string(),
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}
