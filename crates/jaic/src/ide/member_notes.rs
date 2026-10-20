//! Recording member uses, labels, named fields, polymorphic headers and inferred members.
use super::facts::{IdeName, IdeReceiver, IdeRef, IdeWhat};
use crate::ast;
use crate::intern::Sym;
use crate::sema::Compiler;
use crate::sema::lower::Operand;
use crate::sema::scope::{EntityId, EntityKind, ScopeId};
use crate::sema::value::ProcId;
use crate::source::FileId;
use crate::source::Span;
use crate::types::TypeId;
use crate::types::{StructId, TypeKind};

impl Compiler {
    /// `b` of `a.b` was checked with `a` being `base`.
    pub(crate) fn ide_note_receiver(&mut self, span: Span, base: &Operand) {
        if matches!(
            base,
            Operand::Type(_) | Operand::Module(_) | Operand::Procs(_)
        ) || !self.ide_wants(span.file)
        {
            return;
        }
        let ty = base.ty();
        if let Some(ide) = self.ed.facts_mut() {
            ide.receivers
                .entry(span)
                .and_modify(|t| {
                    if *t != Some(ty) {
                        *t = None;
                    }
                })
                .or_insert(Some(ty));
        }
    }

    /// The name of `name` within declaration `span` (a field's span can cover its type and
    /// initializer).
    pub(super) fn ide_name_span(&self, span: Span, name: Sym) -> Span {
        let text = self.src.sources.snippet(span);
        let word = name.as_str();
        let boundary = |c: Option<char>| !c.is_some_and(|c| c.is_alphanumeric() || c == '_');
        let mut from = 0;
        while let Some(at) = text[from..].find(word) {
            let start = from + at;
            let end = start + word.len();
            if boundary(text[..start].chars().next_back()) && boundary(text[end..].chars().next()) {
                return Span {
                    file: span.file,
                    start: span.start + start as u32,
                    end: span.start + end as u32,
                };
            }
            from = start + 1;
        }
        span
    }

    /// The declaration of the field or enum member `name` of `ty`'s struct or enum, through
    /// pointers and `using` members.
    pub(super) fn ide_member_decl(&self, ty: TypeId, name: Sym) -> Option<Span> {
        let mut ty = ty;
        while let Some(p) = self.types.pointee(ty) {
            ty = p;
        }
        let ty = self.types.repr_struct(ty);
        match self.types.kind(ty) {
            TypeKind::Struct(s) => {
                let (s, field) = self.ide_find_field_decl(*s, name, &mut Vec::new())?;
                let span = self.types.struct_info(s).fields[field].span;
                (span.end > span.start).then(|| self.ide_name_span(span, name))
            }
            TypeKind::Enum(e) => {
                let e =
                    self.types
                        .enum_info(*e)
                        .loose_of
                        .map_or(*e, |t| match self.types.kind(t) {
                            TypeKind::Enum(strict) => *strict,
                            _ => *e,
                        });
                self.ed.facts()?.enum_decls.get(&(e, name)).copied()
            }
            _ => None,
        }
    }

    fn ide_find_field_decl(
        &self,
        s: StructId,
        name: Sym,
        searched: &mut Vec<StructId>,
    ) -> Option<(StructId, usize)> {
        if searched.contains(&s) {
            return None;
        }
        searched.push(s);
        let fields = &self.types.struct_info(s).fields;
        if let Some(at) = fields.iter().position(|f| f.name == Some(name)) {
            return Some((s, at));
        }
        fields.iter().filter(|f| f.using).find_map(|f| {
            let mut ty = f.ty;
            while let Some(p) = self.types.pointee(ty) {
                ty = p;
            }
            let TypeKind::Struct(inner) = *self.types.kind(self.types.repr_struct(ty)) else {
                return None;
            };
            self.ide_find_field_decl(inner, name, searched)
        })
    }

    /// `name` at `span` named a member of `ty` (a field through `a.name`, `T.{ name = v }`, an
    /// enum member through `Enum.NAME` or `.NAME`).
    pub(crate) fn ide_note_member_use(&mut self, span: Span, ty: TypeId, name: Sym) {
        if !self.ed.is_live() || !self.ide_wants(span.file) {
            return;
        }
        let Some(decl) = self.ide_member_decl(ty, name) else {
            return;
        };
        if let Some(ide) = self.ed.facts_mut() {
            ide.member_uses.insert((span, decl));
        }
    }

    /// The label of `break label` / `continue label`: the loop's name, an entity in `scope`.
    pub(crate) fn ide_note_label(&mut self, scope: ScopeId, label: ast::Ident) {
        if !self.ide_wants(label.span.file) {
            return;
        }
        let Ok(ids) = self.lookup(scope, label.name) else {
            return;
        };
        let &[id] = ids.as_slice() else {
            return;
        };
        let ty = match &self.entity(id).kind {
            EntityKind::Local {
                ty, ..
            }
            | EntityKind::Const {
                ty, ..
            } => *ty,
            _ => return,
        };
        if let Some(ide) = self.ed.facts_mut() {
            ide.refs.push(IdeRef {
                span: label.span,
                what: IdeWhat::Entity(id),
                ty,
                decl: false,
            });
        }
    }

    /// `name` at `span` names an authored member/parameter declaration at `decl`.
    pub(crate) fn ide_note_named_field(&mut self, span: Span, name: Sym, decl: Span, ty: TypeId) {
        if !self.ide_wants(span.file) {
            return;
        }
        let decl = self.ide_name_span(decl, name);
        if let Some(ide) = self.ed.facts_mut() {
            ide.member_uses.insert((span, decl));
        }
        self.ide_note_member_ref(span, name, ty);
    }

    /// The name of a procedure type's parameter (`#type (a: int)`), once however often the
    /// type is evaluated.
    pub(crate) fn ide_note_param_name(&mut self, span: Span, name: Sym, ty: TypeId) {
        if self
            .ed
            .facts_mut()
            .is_some_and(|i| i.param_names.insert(span))
        {
            self.ide_note_named_field(span, name, span, ty);
        }
    }

    /// Named arguments of a successfully selected struct instance. Its bound constants
    /// supply the actual parameter types, including dependent and baked parameters.
    pub(crate) fn ide_note_struct_arguments(&mut self, ty: TypeId, args: &[ast::Arg]) {
        if !self.ed.is_live() {
            return;
        }
        let Some(s) = self.types.as_struct(ty) else {
            return;
        };
        let Some(source) = self.struct_asts.get(&s) else {
            return;
        };
        let (lit, scope) = (source.lit.clone(), source.scope);
        for name in args.iter().filter_map(|a| a.name) {
            let Some(param) = lit
                .params
                .iter()
                .filter_map(|p| p.name)
                .find(|p| p.name == name.name)
            else {
                continue;
            };
            let Some(ids) = self.scope(scope).names.get(&name.name) else {
                continue;
            };
            let [id] = ids.as_slice() else {
                continue;
            };
            let EntityKind::Const {
                ty, ..
            } = self.entity(*id).kind
            else {
                continue;
            };
            self.ide_note_named_field(name.span, name.name, param.span, ty);
        }
    }

    /// `name` at `span` names a member of type `ty` where nothing else records it for hover
    /// (a field of a struct literal).
    pub(crate) fn ide_note_member_ref(&mut self, span: Span, name: Sym, ty: TypeId) {
        if !self.ed.is_live() || !self.ide_wants(span.file) {
            return;
        }
        if let Some(ide) = self.ed.facts_mut() {
            ide.refs.push(IdeRef {
                span,
                what: IdeWhat::Member(name),
                ty,
                decl: false,
            });
        }
    }

    /// The `$T` binders of a procedure header: each name with the span of the name after the
    /// `$` (its first binder when there are several).
    pub(crate) fn poly_binders(&self, header: &ast::ProcHeader) -> Vec<(Sym, Span)> {
        let mut out: Vec<(Sym, Span)> = Vec::new();
        for_header_types(header, &mut |e| {
            if let ast::ExprKind::PolyVar {
                name, ..
            }
            | ast::ExprKind::PolyRestricted {
                name, ..
            } = &e.kind
                && !out.iter().any(|(n, _)| n == name)
            {
                out.push((*name, self.poly_name_span(e, *name)));
            }
        });
        out
    }

    /// The name of `$T`, `$$T` or `$T/R` at `e`, after its `$`s.
    fn poly_name_span(&self, e: &ast::Expr, name: Sym) -> Span {
        let text = self.ide_snippet(e.span);
        let skip = text.len() - text.trim_start_matches('$').len();
        let start = e.span.start + skip as u32;
        Span {
            file: e.span.file,
            start,
            end: (start + name.as_str().len() as u32).min(e.span.end.max(start)),
        }
    }

    /// The header of a polymorphic procedure was instantiated with `bound` (each `$T` and the
    /// constant it became): its `$T`s and `T`s name those constants, and the name of a
    /// restriction (`$T/Ring`) is looked up in `scope`.
    pub(crate) fn ide_note_poly_header(
        &mut self,
        header: &ast::ProcHeader,
        bound: &[(Sym, EntityId)],
        scope: ScopeId,
    ) {
        if !self.ide_wants(header.span.file) {
            return;
        }
        let mut found: Vec<(Span, Sym, bool)> = Vec::new();
        for_header_types(header, &mut |e| match &e.kind {
            ast::ExprKind::Ident(name) => found.push((e.span, *name, false)),
            ast::ExprKind::PolyVar {
                name, ..
            }
            | ast::ExprKind::PolyRestricted {
                name, ..
            } => found.push((self.poly_name_span(e, *name), *name, true)),
            _ => {}
        });
        for (span, name, binder) in found {
            let what = match bound.iter().find(|(n, _)| *n == name) {
                Some(&(_, id)) => IdeWhat::Entity(id),
                // Only restrictions and other named types: types the header names elsewhere
                // are recorded when the instance's signature is checked.
                None if !binder => match self.lookup(scope, name) {
                    Ok(ids) if ids.len() == 1 => IdeWhat::Entity(ids[0]),
                    _ => continue,
                },
                None => continue,
            };
            let ty = match &what {
                IdeWhat::Entity(e) => match &self.entity(*e).kind {
                    EntityKind::Const {
                        ty, ..
                    } => *ty,
                    _ => TypeId::TYPE,
                },
                _ => TypeId::TYPE,
            };
            if let Some(ide) = self.ed.facts_mut() {
                ide.refs.push(IdeRef {
                    span,
                    what,
                    ty,
                    decl: false,
                });
            }
        }
    }

    /// A polymorphic procedure or macro nothing used is not checked: still name the types
    /// its header mentions.
    pub(crate) fn ide_note_unchecked_header(&mut self, id: ProcId) {
        let p = self.proc(id);
        if p.instances.is_empty() {
            let (header, scope) = (p.lit.header.clone(), p.scope);
            self.ide_note_poly_header(&header, &[], scope);
        }
    }

    /// The enum members an inferred `.NAME` at `offset` can stand for, from the type its
    /// context expected (a variable's declared type, a parameter, the other side of `==`).
    pub fn ide_inferred_members(&mut self, file: FileId, offset: u32) -> Vec<IdeName> {
        let Some(types) = self.ed.facts().and_then(|ide| {
            ide.inferred_expected
                .iter()
                .find(|(s, _)| s.file == file && s.start == offset)
                .map(|(_, t)| t.clone())
        }) else {
            return Vec::new();
        };
        let mut out: Vec<IdeName> = Vec::new();
        for t in types {
            let t = self.types.repr_struct(t);
            if matches!(self.types.kind(t), TypeKind::Enum(_)) {
                for n in self.ide_members(IdeReceiver::Type(t)) {
                    if !out.iter().any(|o| o.name == n.name) {
                        out.push(n);
                    }
                }
            }
        }
        out
    }

    /// A call nothing accepts (typically one being typed, with arguments missing): record the
    /// types its candidates take where an argument is an inferred `.NAME`, so completion can
    /// still list the enum's members.
    pub(crate) fn ide_note_unmatched_inferred(
        &mut self,
        procs: &[ProcId],
        args: &[crate::sema::calls::CallArg],
        span: Span,
    ) {
        if !self.ide_wants(span.file) {
            return;
        }
        for (i, a) in args.iter().enumerate() {
            let Some(ast::Expr {
                kind: ast::ExprKind::InferredMember(name),
                ..
            }) = &a.expr
            else {
                continue;
            };
            for &p in procs {
                if self.proc(p).is_poly {
                    continue;
                }
                let header = self.proc(p).lit.header.clone();
                let at = match a.name {
                    Some(n) => header
                        .params
                        .iter()
                        .position(|q| q.name.map(|x| x.name) == Some(n)),
                    None => Some(i),
                };
                let Some(at) = at.filter(|&k| k < header.params.len() && !header.params[k].baked)
                else {
                    continue;
                };
                // Index among the runtime parameters.
                let k = header.params[..at].iter().filter(|q| !q.baked).count();
                let Ok(sig) = self.signature(p, span) else {
                    continue;
                };
                let Some(param) = sig.params.get(k) else {
                    continue;
                };
                let ty = param.ty;
                if let Some(ide) = self.ed.facts_mut() {
                    let seen = ide.inferred_expected.entry(name.span).or_default();
                    if !seen.contains(&ty) {
                        seen.push(ty);
                    }
                }
            }
        }
    }

    /// An enum member's declaration.
    pub(crate) fn ide_note_enum_member(&mut self, e: crate::types::EnumId, name: Sym, span: Span) {
        if !self.ed.is_live() {
            return;
        }
        // Every file's: a use in a recorded file can name a member declared in a module.
        let wanted = self.ide_wants(span.file);
        if let Some(ide) = self.ed.facts_mut() {
            ide.enum_decls.insert((e, name), span);
            if wanted {
                ide.member_uses.insert((span, span));
            }
        }
    }

    /// `Poly(args)` with `Poly` at `span` gave the struct `t`.
    pub(super) fn ide_note_instance(&mut self, span: Span, t: TypeId) {
        if let Some(ide) = self.ed.facts_mut() {
            ide.instances
                .entry(span)
                .and_modify(|old| {
                    if *old != Some(t) {
                        *old = None;
                    }
                })
                .or_insert(Some(t));
        }
    }
}

/// Every expression of the parameter and return types of `header`, outermost first.
pub(crate) fn for_header_types(header: &ast::ProcHeader, f: &mut dyn FnMut(&ast::Expr)) {
    fn walk(e: &ast::Expr, f: &mut dyn FnMut(&ast::Expr)) {
        use crate::ast::ExprKind as E;
        f(e);
        match &e.kind {
            E::Unary(_, x) => walk(x, f),
            E::PolyRestricted {
                restriction, ..
            } => walk(restriction, f),
            E::ArrayType {
                size,
                elem,
            } => {
                if let ast::ArraySize::Fixed(n) = size {
                    walk(n, f);
                }
                walk(elem, f);
            }
            E::Call {
                callee,
                args,
                ..
            } => {
                walk(callee, f);
                for a in args {
                    walk(&a.value, f);
                }
            }
            E::ProcType(h) => for_header_types(h, f),
            _ => {}
        }
    }
    for t in header
        .params
        .iter()
        .filter_map(|p| p.ty.as_ref())
        .chain(header.returns.iter().filter_map(|r| r.ty.as_ref()))
    {
        walk(t, f);
    }
}

/// Does a condition mention `OS` or `CPU`?
pub(super) fn names_target(expr: &ast::Expr) -> bool {
    use crate::ast::ExprKind as E;
    match &expr.kind {
        E::Ident(name) => matches!(name.as_str(), "OS" | "CPU"),
        E::Binary(_, a, b) => names_target(a) || names_target(b),
        E::Unary(_, e) => names_target(e),
        _ => false,
    }
}
