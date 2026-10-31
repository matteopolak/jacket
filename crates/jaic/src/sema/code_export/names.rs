//! Resolving names, headers and callees while exporting.
use super::Exporter;
use super::constants::{inst_flags, node};
use super::procs::procedure_flags;
use crate::records::{Field, FieldRef, Item, RecordTag};
use crate::sema::scope::{EntityKind, Resolved};
use crate::sema::{ProcId, Span, Sym, TypeId, Value, ast, procs};

impl Exporter<'_> {
    /// The declaration record `name` refers to here: a parameter or local of the tree being
    /// exported, else what the scope's lookup finds (a declaration made for it once).
    pub(super) fn resolve_name(&mut self, name: Sym) -> Option<i64> {
        if let Some(&(_, decl)) = self.locals.iter().rev().find(|(n, _)| *n == name) {
            // 0: a local with no declaration record (a loop's `it`).
            return (decl != 0).then_some(decl);
        }
        let scope = self.scope;
        let c = self.c.as_deref_mut()?;
        c.meta.export.refresh_name_caches(c.ns.entities.len());
        let id = if let Some(&cached) = c.meta.export.name_lookups.get(&(scope, name)) {
            cached?
        } else {
            let found = c
                .lookup(scope, name)
                .ok()
                .and_then(|ids| ids.first().copied());
            c.meta.export.name_lookups.insert((scope, name), found);
            found?
        };
        if let Some(&decl) = c.meta.export.resolved_decls.get(&id) {
            return Some(decl);
        }
        let entity = c.entity(id);
        let span = entity.span;
        let (ty, header, flags) = match entity.kind.clone() {
            EntityKind::Local {
                ty, ..
            } => (Some(ty), None, 0),
            // Builtin procedures (`size_of`, `type_of`...) have no declaration.
            EntityKind::Builtin(_) => return None,
            _ => match c.resolve_entity(id).ok()? {
                Resolved::Proc(p)
                | Resolved::Const {
                    value: Value::Proc(p),
                    ..
                } => {
                    let ty = c.signature(p, span).ok().map(|s| s.ty);
                    (ty, Some(p), 0x1 | 0x10_0000)
                }
                Resolved::ProcSet(ps) => (None, ps.first().copied(), 0x1 | 0x10_0000),
                Resolved::Const {
                    ty, ..
                } => (Some(ty), None, 0x1 | 0x10_0000),
                Resolved::Global {
                    ty, ..
                } => (Some(ty), None, 0x10_0000),
                Resolved::PolyStruct(_) => (Some(TypeId::TYPE), None, 0x1 | 0x10_0000),
                Resolved::Module(_) | Resolved::Library(_) => (None, None, 0x1 | 0x10_0000),
            },
        };
        let expression = header.map_or(0, |p| self.resolved_header(p));
        let decl = self.reference_decl(name, ty, expression, flags, span);
        // A procedure's notes follow its body, so the declaration carries the same ones as its
        // header (`@after:a @without:B` is read from `declaration.notes`).
        let header_notes = match self.r.field(expression, "notes") {
            Some(FieldRef::List(list)) => list
                .iter()
                .filter_map(|item| match item {
                    Item::Ref(id) => Some(id),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        };
        if !header_notes.is_empty() {
            self.r.get_mut(decl).unwrap().refs("notes", header_notes);
        }
        self.c
            .as_deref_mut()?
            .meta
            .export
            .resolved_decls
            .insert(id, decl);
        Some(decl)
    }

    /// A `Code_Declaration` that only describes a name (type, value) for resolution: not part
    /// of the tree being exported, so not in `sub`.
    fn reference_decl(
        &mut self,
        name: Sym,
        ty: Option<TypeId>,
        expression: i64,
        flags: i64,
        span: Span,
    ) -> i64 {
        let ty = ty.map_or(0, |t| self.ty(t));
        let mut inst = self.node(
            RecordTag::CodeTypeInstantiation,
            node::TYPE_INSTANTIATION,
            span,
        );
        inst.ptr("result", ty).ptr("type", ty);
        let inst = self.r.add(inst);
        let mut rec = self.node(RecordTag::CodeDeclaration, node::DECLARATION, span);
        rec.str("name", name.as_str().as_bytes())
            .ptr("type", ty)
            .ptr(
                "type_inst",
                if ty != 0 {
                    inst
                } else {
                    0
                },
            )
            .ptr("expression", expression)
            .int("flags", flags);
        let id = self.r.reserve(rec.tag);
        rec.int("serial", id);
        *self.r.get_mut(id).unwrap() = rec;
        id
    }

    /// Set `.VARARGS` in the `inst_flags` of a variadic parameter's declaration (`args: ..Any`),
    /// which checkers such as Check's @`PrintLike` read; adds the instantiation when the
    /// declaration has none (a polymorphic header's unresolved type).
    pub(super) fn mark_varargs(&mut self, decl: i64, span: Span) {
        let inst =
            if let Some(FieldRef::Ref(id)) = self.r.get(decl).and_then(|d| d.field("type_inst")) {
                id
            } else {
                let rec = self.node(
                    RecordTag::CodeTypeInstantiation,
                    node::TYPE_INSTANTIATION,
                    span,
                );
                let id = self.r.reserve(rec.tag);
                let mut rec = rec;
                rec.int("serial", id);
                *self.r.get_mut(id).unwrap() = rec;
                if let Some(d) = self.r.get_mut(decl) {
                    d.set("type_inst", Field::Item(Item::Ref(id)));
                }
                id
            };
        if let Some(rec) = self.r.get_mut(inst) {
            let existing = match rec.field("inst_flags") {
                Some(FieldRef::Int(flags)) => flags,
                _ => 0,
            };
            rec.set(
                "inst_flags",
                Field::Item(Item::Int(existing | inst_flags::VARARGS)),
            );
        }
    }

    /// The header `resolved_procedure_expression` points to: name, typed arguments and
    /// returns, flags. Made once per procedure; no body.
    pub(super) fn resolved_header(&mut self, p: ProcId) -> i64 {
        let own = self.own;
        let Some(c) = self.c.as_deref_mut() else {
            return 0;
        };
        if let Some(&h) = c.meta.export.resolved_headers.get(&(p, own)) {
            return h;
        }
        let id = self.r.reserve(RecordTag::CodeProcedureHeader);
        c.meta.export.resolved_headers.insert((p, own), id);
        let info = c.proc(p);
        let (name, lit, span, is_poly) = (info.name, info.lit.clone(), info.span, info.is_poly);
        // Called ordinary procedures and polymorph instances both need a body
        // event once lowered, regardless of where their declarations live.
        if !own && !is_poly && !info.is_macro && lit.body.is_some() {
            let scope = info.bindings.unwrap_or(info.scope);
            c.meta.export.schedule_body(p, id, scope);
        }
        let sig = if is_poly {
            None
        } else {
            c.signature(p, span).ok()
        };
        let h = &lit.header;
        let mut arguments = Vec::new();
        let mut runtime_index = 0;
        for param in &h.params {
            let ty = self.parameter_type(Some(p), param, sig.as_deref(), &mut runtime_index);
            let name = param.name.map_or(Sym::intern(""), |n| n.name);
            let arg = self.reference_decl(name, ty, 0, 0, param.span);
            if param.variadic {
                self.mark_varargs(arg, param.span);
            }
            arguments.push(arg);
        }
        let mut returns = Vec::new();
        for (i, ret) in h.returns.iter().enumerate() {
            let ty = sig.as_ref().and_then(|s| s.returns.get(i)).copied();
            let name = ret.name.map_or(Sym::intern(""), |n| n.name);
            returns.push(self.reference_decl(name, ty, 0, 0, ret.span));
        }
        let flags = procedure_flags(h, is_poly);
        let ty = sig.as_ref().map_or(0, |s| self.ty(s.ty));
        // Checkers such as MetaThreadSafe read the callee's notes (`@thread`).
        let decl_notes = (self.c.as_deref())
            .and_then(|c| c.pr.proc_decl_notes.get(&p).cloned())
            .unwrap_or_default();
        let mut notes: Vec<&ast::Note> = h.notes.iter().collect();
        for n in &decl_notes {
            if !notes.iter().any(|m| m.span == n.span) {
                notes.push(n);
            }
        }
        let notes = self.notes(&notes);
        let mut rec = self.node(RecordTag::CodeProcedureHeader, node::PROCEDURE_HEADER, span);
        rec.refs("arguments", arguments)
            .refs("returns", returns)
            .refs("notes", notes)
            .str("name", name.as_str().as_bytes())
            .ptr("type", ty)
            .int("procedure_flags", flags)
            .int("serial", id);
        *self.r.get_mut(id).unwrap() = rec;
        id
    }

    /// For a call at `span` resolved to polymorphic `p`: the instance the call made, if
    /// the compiler recorded one, else `p`.
    pub(super) fn instance_called_at(&self, p: ProcId, span: Span) -> ProcId {
        let Some(c) = self.c.as_deref() else {
            return p;
        };
        if !c.proc(p).is_poly {
            return p;
        }
        let made = c.meta.export.call_instances.get(&span);
        made.and_then(|list| {
            list.iter()
                .rev()
                .find(|inst| c.proc(p).instances.values().any(|i| i == *inst))
                .copied()
        })
        .unwrap_or(p)
    }

    /// Baked parameters have bindings but occupy no runtime signature slot.
    pub(super) fn parameter_type(
        &mut self,
        procedure: Option<ProcId>,
        param: &ast::Param,
        signature: Option<&procs::Signature>,
        runtime_index: &mut usize,
    ) -> Option<TypeId> {
        if !param.baked {
            let index = *runtime_index;
            *runtime_index += 1;
            return signature.and_then(|s| s.params.get(index)).map(|p| p.ty);
        }
        let c = self.c.as_deref_mut()?;
        let bindings = c.proc(procedure?).bindings?;
        let entities = c.lookup(bindings, param.name?.name).ok()?;
        match c.resolve_entity(*entities.first()?).ok()? {
            Resolved::Const {
                ty, ..
            } => Some(ty),
            _ => None,
        }
    }

    /// The procedure a call to `callee` with `args` resolves to: the one candidate, or the
    /// first whose parameter count fits the arguments.
    pub(super) fn resolve_call(&mut self, callee: &ast::Expr, args: usize) -> Option<ProcId> {
        let ast::ExprKind::Ident(name) = &callee.kind else {
            return None;
        };
        let name = *name;
        if self.locals.iter().any(|(n, _)| *n == name) {
            return None;
        }
        let scope = self.scope;
        let c = self.c.as_deref_mut()?;
        let mut procs = Vec::new();
        for id in c.lookup(scope, name).ok()? {
            if matches!(c.entity(id).kind, EntityKind::Builtin(_)) {
                continue;
            }
            match c.resolve_entity(id) {
                Ok(
                    Resolved::Proc(p)
                    | Resolved::Const {
                        value: Value::Proc(p),
                        ..
                    },
                ) => procs.push(p),
                Ok(Resolved::ProcSet(ps)) => procs.extend(ps),
                _ => {}
            }
        }
        if procs.len() <= 1 {
            return procs.first().copied();
        }
        procs.into_iter().find(|&p| {
            let params = &c.proc(p).lit.header.params;
            let required = params
                .iter()
                .filter(|p| p.default.is_none() && !p.variadic)
                .count();
            let variadic = params.iter().any(|p| p.variadic);
            args >= required && (variadic || args <= params.len())
        })
    }
}
