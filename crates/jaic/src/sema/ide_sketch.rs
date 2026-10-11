//! Editor facts for polymorphic procedures nothing calls. Such a procedure is never
//! instantiated, so its body is never checked and has no hover or go to definition. For an
//! editor, each one is checked once more as a sketch: an instance whose `$T`s are stand-in
//! types named after them (an empty struct `T`), whose `$N` array sizes are 1 and whose other
//! baked parameters take their default or a zero. Anything may be done with a stand-in: a
//! member `x.name` of a `T` is a stand-in `T.name`, a comparison is a `bool`, it converts to
//! and from every type and iterates like an array. A statement that fails anyway is skipped.
//! The sketch is then forgotten: its errors are dropped and nothing can call it.
use super::lower::{FnCtx, Operand};
use super::procs::BodyState;
use super::value::ProcId;
use super::*;
use crate::types::TypeKind;

/// The most facts one sketch may add.
const MAX_SKETCH_REFS: usize = 20_000;

impl Compiler {
    /// Check a sketch of every polymorphic procedure of the recorded files that has no
    /// instance. Call after `ide_check_all`, only for an editor: the facts name stand-in types,
    /// which a linter should not see.
    pub fn ide_sketch_polymorphs(&mut self) {
        let (entities, procs) = (self.entities.len(), self.procs.len());
        let Some(ide) = self.ide.as_mut() else {
            return;
        };
        ide.sketch_from.get_or_insert(super::ide::SketchMark {
            refs: ide.refs.len(),
            entities,
            procs,
        });
        // Nor do the lint-only facts (expression types, casts, uses) get the stand-ins.
        let lint = std::mem::replace(&mut ide.lint, false);
        ide.sketching = true;
        let mut i = 0;
        let count = self.procs.len();
        while i < count {
            let id = ProcId(i as u32);
            i += 1;
            let p = self.proc(id);
            let (file, scope) = (p.span.file, p.scope);
            let expanded = |c: &Self, i: &ProcId| {
                c.ide
                    .as_ref()
                    .is_some_and(|ide| ide.expanded_macros.contains(i))
            };
            if !(p.is_poly || p.is_macro)
                || p.bindings.is_some()
                || p.instances.values().any(|&i| self.ide_body_checked(i))
                || (p.is_macro
                    && (expanded(self, &id) || p.instances.values().any(|i| expanded(self, i))))
                || p.lit.body.is_none()
                || self.inside_instance(scope)
                || !self.ide_wants_pub(file)
            {
                continue;
            }
            let ok = self.ide_sketch(id);
            if !ok {
                break;
            }
        }
        if let Some(ide) = self.ide.as_mut() {
            ide.lint = lint;
            ide.sketching = false;
            ide.sketch_types.clear();
            ide.sketch_tried.clear();
        }
    }

    /// The body of instance `id` was checked to its end. Overload resolution makes instances
    /// whose bodies are never needed, and a body may have failed.
    fn ide_body_checked(&self, id: ProcId) -> bool {
        self.proc(id).body_state == BodyState::Done && !self.lenient_failures.contains_key(&id)
    }

    /// Check a sketch of `id`; false when checking it panicked, which may have left the
    /// checker's state half-way.
    fn ide_sketch(&mut self, id: ProcId) -> bool {
        let errors = self.ide.as_ref().map_or(0, |ide| ide.errors.len());
        let (warnings, deferred) = (self.warnings.len(), self.deferred_errors.len());
        let first_new = self.procs.len();
        let asserts = self.asserts.len();
        let refs = self.ide.as_ref().map_or(0, |ide| ide.refs.len());
        // A sketch is ordinary checking, but with types nothing was written for: should it
        // trip an assumption, lose the sketch, not the editor's other facts.
        let checked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let Some(bindings) = self.ide_sketch_bindings(id) else {
                return;
            };
            let span = self.proc(id).span;
            if let Some(ide) = self.ide.as_mut() {
                ide.sketch_root = true;
            }
            let inst = self.instantiate(id, bindings, span);
            if let Some(ide) = self.ide.as_mut() {
                ide.sketch_root = false;
            }
            if let Ok(inst) = inst {
                self.procs[id.0 as usize]
                    .instances
                    .retain(|_, &mut i| i != inst);
                // A macro is sketched as a procedure: what it reaches in its caller (`x`) is
                // missing, and those statements are skipped.
                self.procs[inst.0 as usize].is_macro = false;
                if self.proc_func(inst, span).is_ok() {
                    let _ = self.drain_bodies_lenient();
                }
            }
        }));
        // Forget the sketch: what it queued, its errors, and the bodies that failed.
        let sketch = |p: &ProcId| (p.0 as usize) >= first_new;
        self.body_queue.retain(|p| !sketch(p));
        self.parked_bodies.retain(|p| !sketch(p));
        self.lenient_failures.retain(|p, _| !sketch(p));
        for p in &mut self.procs[first_new..] {
            if p.body_state == BodyState::Queued {
                p.body_state = BodyState::NotNeeded;
            }
        }
        if let Some(ide) = self.ide.as_mut() {
            ide.errors.truncate(errors);
            // A sketch that ran away (compile-time code generating without end) is dropped.
            if ide.refs.len() - refs > MAX_SKETCH_REFS {
                ide.truncate_refs(refs);
            }
        }
        self.warnings.truncate(warnings);
        self.deferred_errors.truncate(deferred);
        self.asserts.truncate(asserts);
        self.asserts_done = self.asserts_done.min(asserts);
        checked.is_ok()
    }

    /// Bindings for a sketch of `id`, or `None` when a baked parameter has no value to stand in
    /// for it (a `Code`, a struct, a variadic pack).
    fn ide_sketch_bindings(&mut self, id: ProcId) -> Option<Vec<(Sym, Value, TypeId)>> {
        let header = self.proc(id).lit.header.clone();
        let scope = self.proc(id).scope;
        let mut sizes: Vec<Sym> = Vec::new();
        super::ide::for_header_types(&header, &mut |e| {
            if let ast::ExprKind::ArrayType {
                size: ast::ArraySize::Fixed(n),
                ..
            } = &e.kind
                && let ast::ExprKind::PolyVar {
                    name, ..
                } = &n.kind
            {
                sizes.push(*name);
            }
        });
        // `Matrix($T, $M, $N)`: each binds a parameter of the struct, of its type.
        let mut struct_args: Vec<ast::Expr> = Vec::new();
        super::ide::for_header_types(&header, &mut |e| {
            if let ast::ExprKind::Call {
                args, ..
            } = &e.kind
                && args
                    .iter()
                    .any(|a| matches!(a.value.kind, ast::ExprKind::PolyVar { .. }))
            {
                struct_args.push(e.clone());
            }
        });
        // `$M/Matrix`: an instance of the struct, with stand-in parameters.
        let mut restricted: Vec<(Sym, ast::Expr)> = Vec::new();
        super::ide::for_header_types(&header, &mut |e| {
            if let ast::ExprKind::PolyRestricted {
                name,
                restriction,
                ..
            } = &e.kind
            {
                restricted.push((*name, (**restriction).clone()));
            }
        });
        let mut typed: Vec<(Sym, TypeId)> = Vec::new();
        for call in &struct_args {
            let ast::ExprKind::Call {
                callee,
                args,
                ..
            } = &call.kind
            else {
                continue;
            };
            let Ok(Operand::PolyStruct(ps)) = self.check_expr_no_emit(scope, callee) else {
                continue;
            };
            for (i, arg) in args.iter().enumerate() {
                if let ast::ExprKind::PolyVar {
                    name, ..
                } = &arg.value.kind
                    && let Some(ty) = self.poly_struct_param_type(ps, i, arg.name.map(|n| n.name))
                {
                    typed.push((*name, ty));
                }
            }
        }
        let binders = self.poly_binders(&header);
        let mut out: Vec<(Sym, Value, TypeId)> = Vec::new();
        for (name, at) in binders {
            let typed = typed
                .iter()
                .find(|(n, _)| *n == name)
                .map(|&(_, t)| (t, self.types.kind(self.types.repr(t)).clone()));
            if sizes.contains(&name) {
                out.push((name, Value::Int(1), TypeId::S64));
            } else if let Some((
                ty,
                TypeKind::Int {
                    ..
                },
            )) = typed
            {
                // Large enough for the members a small size hides (`r3c3` of a 4x4 matrix).
                out.push((name, Value::Int(4), ty));
            } else if let Some(t) = restricted
                .iter()
                .find(|(n, _)| *n == name)
                .and_then(|(_, r)| self.ide_sketch_restricted(scope, r, at))
            {
                out.push((name, Value::Type(t), TypeId::TYPE));
            } else {
                let t = self.ide_stand_in(name, at, scope);
                out.push((name, Value::Type(t), TypeId::TYPE));
            }
        }
        // `v: Vec2` with `Vec2` polymorphic: a call binds `Vec2` to the argument's instance.
        for param in &header.params {
            let Some(mut ty) = param.ty.as_ref() else {
                continue;
            };
            if super::procs::has_poly(ty) {
                continue;
            }
            while let ast::ExprKind::Unary(_, e)
            | ast::ExprKind::ArrayType {
                elem: e, ..
            }
            | ast::ExprKind::TypeDirective {
                ty: e, ..
            } = &ty.kind
            {
                ty = e;
            }
            if let ast::ExprKind::Ident(name) = ty.kind
                && !out.iter().any(|(n, ..)| *n == name)
                && let Some(t) = self.ide_sketch_restricted(scope, ty, ty.span)
            {
                out.push((name, Value::Type(t), TypeId::TYPE));
            }
        }
        for param in &header.params {
            let Some(name) = param.name.filter(|_| param.baked) else {
                continue;
            };
            if param.variadic {
                return None;
            }
            if out.iter().any(|(n, ..)| *n == name.name) {
                continue;
            }
            if let Some(d) = &param.default
                && let Ok((v, t)) = self.baked_default(scope, param, d, &out, name.span)
            {
                out.push((name.name, v, t));
                continue;
            }
            let ty = self.eval_type(scope, param.ty.as_ref()?).ok()?;
            let value = match self.types.kind(self.types.repr(ty)) {
                TypeKind::Type => Value::Type(self.ide_stand_in(name.name, name.span, scope)),
                TypeKind::Int {
                    ..
                }
                | TypeKind::Enum(_) => Value::Int(0),
                TypeKind::Float {
                    ..
                } => Value::Float(0.0),
                TypeKind::Bool => Value::Bool(false),
                TypeKind::String => Value::String(Rc::from(&b""[..])),
                _ => return None,
            };
            out.push((name.name, value, ty));
        }
        Some(out)
    }

    /// An instance of the polymorphic struct `restriction` names, its type parameters
    /// stand-ins and its integer ones 4.
    fn ide_sketch_restricted(
        &mut self,
        scope: ScopeId,
        restriction: &ast::Expr,
        span: Span,
    ) -> Option<TypeId> {
        let Ok(Operand::PolyStruct(ps)) = self.check_expr_no_emit(scope, restriction) else {
            return None;
        };
        let lit = self.poly_structs[ps.0 as usize].lit.clone();
        let mut args: Vec<(Option<Sym>, Operand)> = Vec::new();
        for (i, param) in lit.params.iter().enumerate() {
            let name = param.name?;
            if param.variadic || param.default.is_some() {
                continue;
            }
            let ty = self.poly_struct_param_type(ps, i, None)?;
            let op = match self.types.kind(self.types.repr(ty)) {
                TypeKind::Type => Operand::Type(self.ide_stand_in(name.name, name.span, scope)),
                TypeKind::Int {
                    ..
                } => Operand::Const {
                    ty,
                    value: Value::Int(4),
                    untyped: false,
                },
                _ => return None,
            };
            args.push((Some(name.name), op));
        }
        self.instantiate_struct(ps, args, span).ok()
    }

    /// An empty struct named `name`, standing in for whatever type a call would bind.
    fn ide_stand_in(&mut self, name: Sym, span: Span, scope: ScopeId) -> TypeId {
        let lit = Rc::new(ast::StructLit {
            id: ast::AstId::fresh(),
            kind: ast::StructKind::Struct,
            params: Vec::new(),
            body: Vec::new(),
            flags: Default::default(),
            tag: None,
            modify: None,
            notes: Vec::new(),
            span,
        });
        let t = self.new_struct_type(name, lit, scope, Vec::new(), None);
        if let Some(ide) = self.ide.as_mut() {
            ide.stand_ins.insert(t);
        }
        t
    }
}

impl Compiler {
    /// A sketch is being checked.
    pub(super) fn sketching(&self) -> bool {
        self.ide.as_ref().is_some_and(|ide| ide.sketching)
    }

    /// A call in a sketch would instantiate a polymorph with a stand-in. It is refused: each
    /// member of a stand-in is a new stand-in, so polymorphs calling each other on members
    /// would instantiate without end.
    pub(super) fn ide_sketch_refuses(&self, bindings: &[(Sym, Value, TypeId)]) -> bool {
        self.ide
            .as_ref()
            .is_some_and(|ide| ide.sketching && !ide.sketch_root)
            && bindings.iter().any(|(_, v, _)| match v {
                Value::Type(t) => self.mentions_stand_in(*t),
                _ => false,
            })
    }

    /// `ty` is a stand-in, or a pointer to or array of one.
    fn mentions_stand_in(&self, ty: TypeId) -> bool {
        let mut t = ty;
        for _ in 0..8 {
            if self.is_stand_in(t) {
                return true;
            }
            t = match self.types.kind(t) {
                TypeKind::Array {
                    elem, ..
                } => *elem,
                TypeKind::Pointer(p) => *p,
                _ => return false,
            };
        }
        false
    }

    /// `ty` (or what it points to) is a sketch's stand-in type.
    pub(super) fn is_stand_in(&self, ty: TypeId) -> bool {
        let Some(ide) = self.ide.as_ref().filter(|ide| !ide.stand_ins.is_empty()) else {
            return false;
        };
        let mut t = ty;
        for _ in 0..8 {
            if ide.stand_ins.contains(&t) {
                return true;
            }
            match self.types.pointee(t) {
                Some(p) => t = p,
                None => return false,
            }
        }
        false
    }

    /// A runtime value of type `ty`. Only a sketch makes one up: its code never runs.
    pub(super) fn ide_sketch_value(&mut self, f: &mut FnCtx, ty: TypeId, span: Span) -> Operand {
        let size = self.size_of(ty, span).unwrap_or(8).max(1);
        Operand::Place {
            ty,
            addr: f.b.alloca(size, 8),
        }
    }

    /// Note the type of an expression a sketch checked.
    pub(super) fn ide_note_sketch_type(&mut self, span: Span, op: &Operand) {
        let ty = match op {
            Operand::Type(_) | Operand::Procs(_) | Operand::Module(_) | Operand::Void => return,
            other => other.ty(),
        };
        if let Some(ide) = self.ide.as_mut() {
            ide.sketch_types.insert(span, ty);
        }
    }

    /// In a sketch, the value of `expr`, which failed. Its operands are checked first, for
    /// their facts, if the failure came before them (once: an operand that failed is not
    /// checked again, or each enclosing expression would check it anew). When one has a stand-in type, a member of
    /// it is a stand-in named after it, a comparison is a `bool` and any other operation takes
    /// the type of an operand, preferring one that is not a stand-in.
    pub(super) fn ide_sketch_recover(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        expr: &ast::Expr,
    ) -> Option<Operand> {
        use ast::ExprKind as E;
        let children: Vec<&ast::Expr> = match &expr.kind {
            E::Member(base, _) => vec![base],
            E::Index(base, index) => vec![base, index],
            E::Call {
                callee,
                args,
                ..
            } => std::iter::once(&**callee)
                .chain(args.iter().map(|a| &a.value))
                .collect(),
            E::Binary(_, a, b) => vec![a, b],
            E::Unary(_, a) => vec![a],
            E::Cast {
                value, ..
            } => vec![value],
            E::Ifx {
                cond,
                then_value,
                else_value,
                ..
            } => std::iter::once(&**cond)
                .chain(then_value.as_deref())
                .chain(else_value.as_deref())
                .collect(),
            E::StructLit {
                fields, ..
            } => fields.iter().map(|a| &a.value).collect(),
            E::ArrayLit {
                elems, ..
            } => elems.iter().collect(),
            _ => return None,
        };
        let ty_of = |c: &Self, e: &ast::Expr| {
            c.ide
                .as_ref()
                .and_then(|ide| ide.sketch_types.get(&e.span).copied())
        };
        for child in &children {
            let tried = self
                .ide
                .as_ref()
                .is_some_and(|ide| ide.sketch_tried.contains(&child.span));
            if !tried {
                let _ = self.check_expr(f, scope, child, None);
            }
        }
        let loose = |c: &Self, e: &ast::Expr| ty_of(c, e).filter(|&t| c.is_stand_in(t));
        let span = expr.span;
        let ty = match &expr.kind {
            E::Member(base, member) => {
                let base_ty = loose(self, base)?;
                let ty = self.ide_stand_in_member(base_ty, member.name, member.span);
                self.ide_note_member_ref(member.span, member.name, ty);
                ty
            }
            E::Index(base, _) => {
                let base_ty = loose(self, base)?;
                self.ide_stand_in_member(base_ty, Sym::intern("[]"), span)
            }
            E::Call {
                callee, ..
            } => {
                let callee_ty = loose(self, callee)?;
                self.ide_stand_in_member(callee_ty, Sym::intern("()"), span)
            }
            E::Binary(op, a, b) => {
                use ast::BinOp as B;
                loose(self, a).or_else(|| loose(self, b))?;
                if matches!(
                    op,
                    B::Eq | B::Ne | B::Lt | B::Le | B::Gt | B::Ge | B::And | B::Or
                ) {
                    TypeId::BOOL
                } else {
                    [a, b]
                        .into_iter()
                        .filter_map(|e| ty_of(self, e))
                        .find(|&t| !self.is_stand_in(t))
                        .or_else(|| loose(self, a))
                        .or_else(|| loose(self, b))?
                }
            }
            E::Unary(ast::UnOp::Not, a) => {
                loose(self, a)?;
                TypeId::BOOL
            }
            E::Unary(_, a) => loose(self, a)?,
            E::Cast {
                ty: Some(ty),
                value,
                ..
            } => {
                loose(self, value)?;
                self.eval_type(scope, ty).ok()?
            }
            E::Ifx {
                then_value,
                else_value,
                ..
            } => {
                let values = then_value.iter().chain(else_value).map(|e| ty_of(self, e));
                let values: Vec<TypeId> = values.flatten().collect();
                values
                    .iter()
                    .copied()
                    .find(|&t| !self.is_stand_in(t))
                    .or_else(|| values.iter().copied().find(|&t| self.is_stand_in(t)))?
            }
            _ => return None,
        };
        Some(self.ide_sketch_value(f, ty, span))
    }

    /// The stand-in for member `name` of stand-in `base`, named `T.name`.
    pub(super) fn ide_stand_in_member(&mut self, base: TypeId, name: Sym, span: Span) -> TypeId {
        let base = self
            .types
            .pointee(base)
            .filter(|_| !self.ide_is_own_stand_in(base))
            .unwrap_or(base);
        if let Some(&t) = self
            .ide
            .as_ref()
            .and_then(|ide| ide.stand_in_members.get(&(base, name)))
        {
            return t;
        }
        let full = if name.as_str() == "[]" || name.as_str() == "()" {
            format!("{}{}", self.types.name(base), name.as_str())
        } else {
            format!("{}.{}", self.types.name(base), name.as_str())
        };
        let scope = self.root_scope;
        let t = self.ide_stand_in(Sym::intern(&full), span, scope);
        if let Some(ide) = self.ide.as_mut() {
            ide.stand_in_members.insert((base, name), t);
        }
        t
    }

    fn ide_is_own_stand_in(&self, ty: TypeId) -> bool {
        self.ide
            .as_ref()
            .is_some_and(|ide| ide.stand_ins.contains(&ty))
    }
}

impl Compiler {
    /// A local declaration of a sketch failed: declare the names it did not, as its type or a
    /// stand-in `?`, so the statements after it still know them.
    pub(super) fn ide_sketch_declare(&mut self, f: &mut FnCtx, scope: ScopeId, decl: &ast::Decl) {
        if decl.kind != ast::DeclKind::Var || decl.backtick {
            return;
        }
        let ty = match decl.ty.as_ref().map(|t| self.eval_type(scope, t)) {
            Some(Ok(ty)) => ty,
            _ => {
                let root = self.root_scope;
                self.ide_stand_in(Sym::intern("?"), decl.span, root)
            }
        };
        let depth = self.scope(scope).proc_depth;
        for name in &decl.names {
            if name.name.as_str() == "_" || self.declares_local(scope, name.name) {
                continue;
            }
            let Operand::Place {
                addr, ..
            } = self.ide_sketch_value(f, ty, name.span)
            else {
                continue;
            };
            self.add_entity(
                scope,
                name.name,
                name.span,
                scope::EntityKind::Local {
                    ty,
                    addr,
                    depth,
                },
                false,
            );
        }
    }
}
