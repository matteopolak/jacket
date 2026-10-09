//! Expression checking and lowering.
use super::lower::{FnCtx, Operand};
use super::scope::{EntityKind, Found, Resolved, ScopeKind};
use super::*;
use crate::ast::{BinOp, ExprKind as E, UnOp};
use crate::ir::{CmpOp, Ty};
use crate::types::{ArrayKind, ProcType, TypeKind};

/// `ifx cond else e` without a `then`: the interesting part of the condition is the value.
/// `!x` and `!f(x)` give `x`, `x >= y` gives `x` and a call `f(x, ...)` gives `x`.
fn implicit_then_expr(cond: &ast::Expr) -> Option<&ast::Expr> {
    match &cond.kind {
        E::Unary(ast::UnOp::Not, inner) => Some(implicit_then_expr(inner).unwrap_or(inner)),
        E::Binary(
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge,
            lhs,
            _,
        ) => Some(lhs),
        E::Call {
            args, ..
        } => args.first().map(|a| &a.value),
        _ => None,
    }
}

impl Compiler {
    pub fn check_expr(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        expr: &ast::Expr,
        expected: Option<TypeId>,
    ) -> Result<Operand> {
        let result = self.check_expr_kind(f, scope, expr, expected);
        if self.ide.is_some()
            && let Ok(op) = &result
        {
            self.ide_note_expr(expr, op);
        }
        result
    }

    fn check_expr_kind(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        expr: &ast::Expr,
        expected: Option<TypeId>,
    ) -> Result<Operand> {
        let span = expr.span;
        match &expr.kind {
            E::Ident(name) => self.check_ident(f, scope, *name, span),
            E::PolyVar {
                name, ..
            } => self.check_ident(f, scope, *name, span),
            E::PolyRestricted {
                name, ..
            } => self.check_ident(f, scope, *name, span),
            E::Int(v) => {
                let v = *v as i128;
                match expected.map(|t| self.types.repr(t)) {
                    Some(t) if self.types.is_float(t) => Ok(Operand::Const {
                        ty: t,
                        value: Value::Float(v as f64),
                        untyped: true,
                    }),
                    _ => Ok(Operand::untyped_int(v)),
                }
            }
            E::Float(v) => Ok(Operand::Const {
                ty: self.float_literal_type(*v, expr.span),
                value: Value::Float(*v),
                untyped: true,
            }),
            E::Char(c) => Ok(Operand::Const {
                ty: TypeId::U8,
                value: Value::Int(*c as i128),
                untyped: true,
            }),
            E::Str(s) => Ok(Operand::Const {
                ty: TypeId::STRING,
                value: Value::String(s.clone()),
                untyped: false,
            }),
            E::Bool(b) => Ok(Operand::bool(*b)),
            E::Null => Ok(Operand::Const {
                ty: TypeId::NULL,
                value: Value::Null,
                untyped: true,
            }),
            E::Uninit => err(span, "`---` is only allowed as a declaration initializer"),
            E::Context => {
                let ty = self.context_type(span)?;
                let Some(ctx) = f.context else {
                    return err(
                        span,
                        "`context` is not available here (procedure is #c_call or #no_context; use push_context)",
                    );
                };
                Ok(Operand::Place {
                    ty,
                    addr: ctx,
                })
            }
            E::Binary(op, a, b) => self.check_binary(f, scope, *op, a, b, expected, span),
            E::Unary(op, a) => self.check_unary(f, scope, *op, a, expected, span),
            E::Call {
                callee,
                args,
                ..
            } => self.check_call(f, scope, callee, args, expected, span),
            E::Member(base, member) => {
                if let Some(op) = self.enclosing_local_constant(f, scope, base, member)? {
                    return Ok(op);
                }
                if !self.const_macro_params.is_empty()
                    && member.name.as_str() == "count"
                    && let E::Ident(name) = &base.kind
                    && let Some(&id) = self.lookup(scope, *name)?.first()
                    && let Some((Value::String(s), _)) = self.const_macro_params.get(&id)
                {
                    let count = s.len();
                    return Ok(Operand::Const {
                        ty: TypeId::S64,
                        value: Value::Int(count as i128),
                        untyped: true,
                    });
                }
                // Compile-time code may read type-level constants through a runtime local
                // (`#if table.FLAG`): only the local's type is needed.
                if f.compile_time
                    && let Some(ty) = self.outer_local_type(scope, base)?
                    && !self.names_field(self.types.pointee(ty).unwrap_or(ty), member.name)?
                    && let Ok(op) = self.member_access(
                        f,
                        scope,
                        Operand::Type(self.types.pointee(ty).unwrap_or(ty)),
                        member.name,
                        member.span,
                    )
                {
                    return Ok(op);
                }
                let base_op = self.check_expr(f, scope, base, None)?;
                if self.ide.is_some() {
                    self.ide_note_receiver(member.span, &base_op);
                }
                let note = self.ide.is_some().then(|| match &base_op {
                    Operand::Type(t) => *t,
                    other => other.ty(),
                });
                let result = self.member_access(f, scope, base_op, member.name, member.span);
                if let (Some(ty), Ok(_)) = (note, &result) {
                    self.ide_note_member_use(member.span, ty, member.name);
                }
                result
            }
            E::InferredMember(name) => self.check_inferred_member(f, scope, name, expected),
            E::Index(base, index) => self.check_index(f, scope, base, index, span),
            E::Cast {
                ty,
                value,
                flags,
            } => {
                let target = match ty {
                    Some(t) => self.eval_type_in(f, scope, t)?,
                    None => match expected {
                        Some(t) => t,
                        None => {
                            // `xx` without a target: leave the value as-is (the caller converts).
                            return self.check_expr(f, scope, value, None);
                        }
                    },
                };
                // Types flow up through a cast: the operand is typed on its own and then
                // converted, so `cast(float32) (0 - w)` subtracts in `w`'s type. Only literals that
                // have no type of their own (`.NAME`, `.{...}`, `.[...]`) are matched against
                // the target {#cast.29}.
                let operand_expected = cast_operand_needs_target(value).then_some(target);
                let op = self.check_expr(f, scope, value, operand_expected)?;
                if self.ide.is_some() {
                    self.ide_note_cast(span, target, &op);
                }
                let op = if flags.no_check || flags.truncate || flags.force {
                    op
                } else {
                    self.check_constant_cast(&op, target, value.span)?;
                    self.emit_cast_check(f, op, target, span)?
                };
                self.explicit_cast(f, op, target, *flags, span)
            }
            E::Ifx {
                cond,
                then_value,
                else_value,
                ..
            } => match else_value {
                Some(e) => self.check_ifx(
                    f,
                    scope,
                    cond,
                    then_value.as_deref(),
                    Some(e),
                    expected,
                    span,
                ),
                // `ifx c then a` / `ifx c`: the value, or zero when the condition is false.
                None => self.check_ifx(f, scope, cond, then_value.as_deref(), None, expected, span),
            },
            E::StructLit {
                ty,
                fields,
            } => self.check_struct_literal(f, scope, ty.as_deref(), fields, expected, span),
            E::ArrayLit {
                ty,
                elems,
            } => self.check_array_literal(f, scope, ty.as_deref(), elems, expected, span),
            E::ArrayType {
                size,
                elem,
            } => {
                let elem = self.eval_type_in(f, scope, elem)?;
                let kind = match size {
                    ast::ArraySize::View => ArrayKind::View,
                    ast::ArraySize::Resizable => ArrayKind::Resizable,
                    ast::ArraySize::Fixed(n) => {
                        let n = self.check_expr(f, scope, n, Some(TypeId::S64))?;
                        match n {
                            Operand::Const {
                                value: Value::Int(n),
                                ..
                            } if n >= 0 => ArrayKind::Fixed(n as u64),
                            other => {
                                return err(
                                    span,
                                    format!(
                                        "array size must be a non-negative constant, found {}",
                                        self.describe(&other)
                                    ),
                                );
                            }
                        }
                    }
                };
                Ok(Operand::Type(self.types.array(elem, kind)))
            }
            E::ProcType(header) => Ok(Operand::Type(self.proc_type_from_header(f, scope, header)?)),
            E::Proc(lit) => {
                let name = Sym::intern("anonymous_procedure");
                let key = lit.header.id;
                let proc = match self.anonymous_procs.get(&(key, scope)) {
                    Some(&p) => p,
                    None => {
                        let p = self.new_proc(name, lit.clone(), scope, span);
                        self.anonymous_procs.insert((key, scope), p);
                        p
                    }
                };
                Ok(Operand::Procs(vec![proc]))
            }
            E::Struct(lit) => {
                if let Some(&t) = self.anonymous_types.get(&(lit.id, scope)) {
                    return Ok(Operand::Type(t));
                }
                self.check_struct_modify(Sym::intern("struct"), lit)?;
                let t = self.new_struct_type(
                    Sym::intern("struct"),
                    lit.clone(),
                    scope,
                    Vec::new(),
                    None,
                );
                self.anonymous_types.insert((lit.id, scope), t);
                Ok(Operand::Type(t))
            }
            E::Enum(lit) => {
                if let Some(&t) = self.anonymous_types.get(&(lit.id, scope)) {
                    return Ok(Operand::Type(t));
                }
                let t = self.new_enum_type(Sym::intern("enum"), lit, scope)?;
                self.anonymous_types.insert((lit.id, scope), t);
                Ok(Operand::Type(t))
            }
            E::TypeDirective {
                modifier,
                ty,
            } => {
                let base = self.eval_type_in(f, scope, ty)?;
                match modifier {
                    ast::TypeModifier::Plain => Ok(Operand::Type(base)),
                    _ => {
                        let isa = *modifier == ast::TypeModifier::Isa;
                        let t = self.types.new_distinct(crate::types::DistinctInfo {
                            name: Sym::intern("distinct"),
                            base,
                            isa,
                        });
                        Ok(Operand::Type(t))
                    }
                }
            }
            E::Run {
                body, ..
            } => self.check_run(scope, body, expected, span),
            E::Code(body) => {
                let id = self.add_code(body.clone(), scope);
                Ok(Operand::Const {
                    ty: TypeId::CODE,
                    value: Value::Code(id),
                    untyped: false,
                })
            }
            E::Insert {
                value,
                flags,
                scope: target,
                ..
            } => {
                let (inserted, at) = self.eval_insert_expr(scope, value)?;
                // `#insert,scope() code` resolves names at the insertion site.
                let at = if target.is_none() && flags.iter().any(|fl| fl.name.as_str() == "scope") {
                    scope
                } else {
                    at
                };
                self.check_expr(f, at, &inserted, expected)
            }
            E::Location(target) => {
                let loc_span = match target {
                    Some(e) => e.span,
                    None => span,
                };
                self.location_operand(f, loc_span)
            }
            E::CallerLocation => self.location_operand(f, span),
            // `#file` is the full path of the file, as loaded.
            E::File => {
                let path = forward_slashes(&self.sources.get(span.file).path);
                Ok(Operand::Const {
                    ty: TypeId::STRING,
                    value: Value::String(path.as_bytes().into()),
                    untyped: false,
                })
            }
            E::Filepath => {
                let path = self.sources.get(span.file).path.clone();
                let dir = std::path::Path::new(&path)
                    .parent()
                    .map(|p| format!("{}/", forward_slashes(&p.display().to_string())))
                    .unwrap_or_default();
                Ok(Operand::Const {
                    ty: TypeId::STRING,
                    value: Value::String(dir.as_bytes().into()),
                    untyped: false,
                })
            }
            E::Line => {
                let (line, _) = self.sources.get(span.file).line_col(span.start);
                Ok(Operand::untyped_int(line as i128))
            }
            E::ProcedureName(_) => {
                let name = f
                    .proc
                    .map(|p| self.proc(p).name.to_string())
                    .unwrap_or_default();
                Ok(Operand::Const {
                    ty: TypeId::STRING,
                    value: Value::String(name.as_bytes().into()),
                    untyped: false,
                })
            }
            E::This => {
                // In a macro body `#this` is the procedure the macro expanded into.
                let mut result = self.check_this(scope, span);
                for frame in f.macros.iter().rev() {
                    if result.is_ok() {
                        break;
                    }
                    result = self.check_this(frame.caller_scope, span);
                }
                if let (Err(_), Some(caller)) = (&result, f.backtick_scope) {
                    // A deferred macro statement runs after its frame is gone.
                    result = self.check_this(caller, span);
                }
                result
            }
            E::CompileTime => {
                if f.compile_time {
                    return Ok(Operand::bool(true));
                }
                let v =
                    f.b.intrinsic(ir::Intrinsic::IsCompileTime, Vec::new(), &[Ty::I8]);
                Ok(Operand::Value {
                    ty: TypeId::BOOL,
                    val: v[0],
                })
            }
            E::Exists(e) => {
                let exists = match &e.kind {
                    E::Ident(name) => !self.lookup(scope, *name)?.is_empty(),
                    _ => self.check_expr(f, scope, e, None).is_ok(),
                };
                Ok(Operand::bool(exists))
            }
            E::Bytes(e) => {
                // Raw machine code cannot run here; the debug-trap encodings (x64 `int3`,
                // arm64 `brk #0`) still trap.
                if let E::ArrayLit {
                    elems, ..
                } = &e.kind
                {
                    let bytes: Vec<u128> = elems
                        .iter()
                        .filter_map(|el| match el.kind {
                            E::Int(v) => Some(v),
                            _ => None,
                        })
                        .collect();
                    if bytes == [0xCC] || bytes == [0x20, 0x00, 0x20, 0xD4] {
                        f.b.intrinsic(ir::Intrinsic::DebugBreak, Vec::new(), &[]);
                        return Ok(Operand::Void);
                    }
                }
                self.check_expr(f, scope, e, expected)
            }
            E::Backtick(inner) => {
                let caller = match (f.macros.last(), f.backtick_scope) {
                    (_, Some(s)) => s,
                    (Some(frame), None) => frame.caller_scope,
                    (None, None) => {
                        return err(span, "backtick names are only valid inside macros");
                    }
                };
                self.check_expr(f, caller, inner, expected)
            }
            E::Bake {
                callee,
                args,
                ..
            } => self.check_bake(scope, callee, args, span),
            E::ProcedureOfCall(call) => self.check_procedure_of_call(f, scope, call),
            E::CallerCode => {
                let Some((call, caller_scope)) = self.calls_in_flight.last().cloned() else {
                    return err(span, "#caller_code is only valid as a parameter default");
                };
                let id = self.add_code(Rc::new(ast::CodeBody::Expr((*call).clone())), caller_scope);
                Ok(Operand::Const {
                    ty: TypeId::CODE,
                    value: Value::Code(id),
                    untyped: false,
                })
            }
            E::UnknownDirective {
                name, ..
            } if name.name.as_str() == "Context" => Ok(Operand::Type(self.context_type(span)?)),
            E::UnknownDirective {
                name,
                operand,
                ..
            } if name.name.as_str() == "jaic_type" => {
                Ok(Operand::Type(self.jaic_type(operand.as_deref(), span)?))
            }
            E::UnknownDirective {
                name, ..
            } => err(
                name.span,
                format!(
                    "directive `#{}` is not supported in this position",
                    name.name
                ),
            ),
            E::Asm(block) => self.check_asm(f, scope, block),
            E::Lambda {
                header,
                body,
            } => self.check_lambda(scope, header, body, expected, span),
            E::Block(block) => self.check_block_value(f, scope, block, expected, span),
        }
    }

    /// `#jaic_type name`: a type only jaic has, for the `Extensions/Long_Double` module to export
    /// (`docs/language/long-double.md`). `long_double` is the target C compiler's
    /// `long double`: `float64` where the two are the same, else the wide `Long_Double` type.
    fn jaic_type(&mut self, operand: Option<&ast::Expr>, span: Span) -> Result<TypeId> {
        let name = match operand.map(|e| &e.kind) {
            Some(E::Ident(name)) => name.as_str(),
            _ => {
                return err(
                    span,
                    "#jaic_type expects a name, as in '#jaic_type long_double'",
                );
            }
        };
        match name {
            "long_double" => Ok(match self.options.long_double {
                Some(fmt) => self.types.intern(TypeKind::WideFloat(fmt)),
                None => TypeId::F64,
            }),
            other => err(span, format!("unknown jaic extension type `{other}`")),
        }
    }

    /// `ifx c then a else { stmts; value }`: a block's value is its last expression.
    fn check_block_value(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        block: &ast::Block,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        let Some((
            ast::Stmt {
                kind: ast::StmtKind::Expr(last),
                ..
            },
            rest,
        )) = block.stmts.split_last()
        else {
            return err(span, "a block used as a value must end in an expression");
        };
        let inner = self.new_block_scope(scope);
        self.ide_scope_span(inner, span);
        let depth = f.defers.len();
        self.check_block_stmts(f, inner, rest)?;
        let mut op = self.check_expr(f, inner, last, expected)?;
        if f.defers.len() > depth {
            // Deferred code runs after the value is computed.
            let op_settled = self.settle_untyped(op, expected);
            let ty = op_settled.ty();
            let (_, v) = self.rvalue(f, op_settled, span)?;
            op = Operand::Value {
                ty,
                val: v,
            };
            self.emit_defers(f, depth, span)?;
        }
        f.defers.truncate(depth);
        Ok(op)
    }

    /// The type of `expr` if it names a runtime local of an enclosing procedure.
    pub(super) fn outer_local_type(
        &mut self,
        scope: ScopeId,
        expr: &ast::Expr,
    ) -> Result<Option<TypeId>> {
        let E::Ident(name) = &expr.kind else {
            return Ok(None);
        };
        Ok(match self.lookup(scope, *name)?.as_slice() {
            [id] => match self.entity(*id).kind {
                EntityKind::Local {
                    ty,
                    depth,
                    ..
                } if depth != self.scope(scope).proc_depth => Some(ty),
                _ => None,
            },
            _ => None,
        })
    }

    /// Evaluate a type expression inside a body.
    pub fn eval_type_in(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        expr: &ast::Expr,
    ) -> Result<TypeId> {
        let op = self.check_expr(f, scope, expr, Some(TypeId::TYPE))?;
        if let Operand::Value {
            ty: TypeId::TYPE, ..
        }
        | Operand::Place {
            ty: TypeId::TYPE, ..
        } = op
        {
            return err(expr.span, "type must be known at compile time");
        }
        self.operand_as_type(op, expr.span)
    }

    pub(super) fn check_ident(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        name: Sym,
        span: Span,
    ) -> Result<Operand> {
        match self.lookup_full(scope, name)? {
            Found::Using(entry, member) => {
                if let scope::UsingEntry::Place {
                    entity, ..
                } = &entry
                {
                    self.ide_note_use(*entity);
                }
                self.using_member(f, entry, member, span)
            }
            Found::Entities(ids) => {
                if ids.is_empty() {
                    if f.type_only
                        && let Some(ty) = self.struct_field_type(scope, name)
                    {
                        // Only the type is wanted, so the address is never used.
                        let addr = f.b.iconst(Ty::Ptr, 0);
                        return Ok(Operand::Place {
                            ty,
                            addr,
                        });
                    }
                    return Err(Box::new(
                        Diagnostic::error(span, format!("unknown identifier `{name}`")).with_kind(
                            DiagnosticKind::UnknownIdentifier {
                                scope: Some(scope.0),
                            },
                        ),
                    ));
                }
                let op = self.entities_operand(f, scope, &ids, span);
                if let Some(ide) = self.ide.as_mut() {
                    ide.last_entity = Some(ids[0]);
                    if ide.lint {
                        ide.used.extend(ids.iter().copied());
                    }
                }
                op
            }
        }
    }

    /// The type of field `name` of a struct whose body encloses `scope`, if already laid out.
    fn struct_field_type(&self, scope: ScopeId, name: Sym) -> Option<TypeId> {
        let mut s = Some(scope);
        while let Some(sid) = s {
            if let Some(fields) = self.field_types.get(&sid)
                && let Some(&(_, ty)) = fields.iter().find(|(n, _)| *n == name)
            {
                return Some(ty);
            }
            s = self.scope(sid).parent;
        }
        None
    }

    pub fn entities_operand(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        ids: &[EntityId],
        span: Span,
    ) -> Result<Operand> {
        if ids.len() > 1 || self.entity_is_overloadable(ids[0]) {
            let mut procs = Vec::new();
            for &id in ids {
                match self.resolve_entity(id)? {
                    Resolved::Proc(p) => procs.push(p),
                    Resolved::Const {
                        value: Value::Proc(p),
                        ..
                    } => procs.push(p),
                    Resolved::ProcSet(set) => procs.extend(set),
                    _ => {
                        if ids.len() == 1 {
                            return self.entity_operand(f, scope, id, span);
                        }
                        return err(
                            span,
                            format!("`{}` is declared more than once", self.entity(id).name),
                        );
                    }
                }
            }
            if ids.len() > 1 {
                self.check_identical_overloads(ids)?;
            }
            return Ok(Operand::Procs(procs));
        }
        self.entity_operand(f, scope, ids[0], span)
    }

    fn entity_operand(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        id: EntityId,
        span: Span,
    ) -> Result<Operand> {
        if self.discard_params.contains(&id) {
            return err(
                span,
                format!(
                    "`{}` is a #discard parameter and cannot be used in the procedure",
                    self.entity(id).name
                ),
            );
        }
        match self.entity(id).kind.clone() {
            EntityKind::Local {
                ty,
                addr,
                depth,
            } => {
                if let Some((value, ty)) = self.const_macro_params.get(&id).cloned() {
                    if let Value::Type(t) = value {
                        return Ok(Operand::Type(t));
                    }
                    return Ok(Operand::Const {
                        ty,
                        value,
                        untyped: false,
                    });
                }
                if depth != self.scope(scope).proc_depth {
                    if let Some((value, ty)) = self.local_consts.get(&id).cloned() {
                        return Ok(Operand::Const {
                            ty,
                            value,
                            untyped: false,
                        });
                    }
                    if f.type_only {
                        // `type_of(local.*)` at compile time: the address is never used.
                        let addr = f.b.iconst(Ty::Ptr, 0);
                        return Ok(Operand::Place {
                            ty,
                            addr,
                        });
                    }
                    let name = self.entity(id).name;
                    return err(
                        span,
                        if f.compile_time {
                            format!(
                                "cannot use local `{name}` in a compile-time expression: its value is only known at runtime"
                            )
                        } else {
                            format!("cannot access local `{name}` of an enclosing procedure")
                        },
                    );
                }
                return Ok(Operand::Place {
                    ty,
                    addr,
                });
            }
            EntityKind::Builtin(scope::Builtin::Proc(p)) => return Ok(Operand::Builtin(p)),
            _ => {}
        }
        Ok(match self.resolve_entity(id)? {
            Resolved::Const {
                value: Value::Type(t),
                ..
            } => Operand::Type(t),
            Resolved::Const {
                value: Value::Proc(p),
                ..
            } => Operand::Procs(vec![p]),
            Resolved::Const {
                value,
                ty,
            } => Operand::Const {
                ty,
                value,
                untyped: self.entity(id).untyped_const,
            },
            Resolved::Proc(p) => Operand::Procs(vec![p]),
            Resolved::ProcSet(set) => Operand::Procs(set),
            Resolved::Global {
                storage,
                ty,
            } => {
                let addr = f.b.storage_addr(storage);
                Operand::Place {
                    ty,
                    addr,
                }
            }
            Resolved::Module(m) => Operand::Module(m),
            Resolved::Library(l) => Operand::Library(l),
            Resolved::PolyStruct(p) => Operand::PolyStruct(p),
        })
    }

    fn check_inferred_member(
        &mut self,
        f: &mut FnCtx,
        _scope: ScopeId,
        name: &ast::Ident,
        expected: Option<TypeId>,
    ) -> Result<Operand> {
        let Some(t) = expected else {
            return err(
                name.span,
                format!("cannot infer the type of `.{}` here", name.name),
            );
        };
        if self.ide.is_some()
            && self.ide_wants_pub(name.span.file)
            && let Some(ide) = self.ide.as_mut()
        {
            let seen = ide.inferred_expected.entry(name.span).or_default();
            if !seen.contains(&t) {
                seen.push(t);
            }
        }
        // Allow pointer-to-enum targets? No: enums only, but look through distinct.
        match self.types.kind(t).clone() {
            TypeKind::Enum(_) | TypeKind::Struct(_) => {
                let result =
                    self.member_access(f, ScopeId(0), Operand::Type(t), name.name, name.span);
                if result.is_ok() {
                    self.ide_note_member_use(name.span, t, name.name);
                }
                result
            }
            _ => err(
                name.span,
                format!(
                    "cannot infer `.{}` for type {}",
                    name.name,
                    self.types.name(t)
                ),
            ),
        }
    }

    /// Is `name` a field (not a constant) of struct type `ty`? Reading one needs a value.
    fn names_field(&mut self, ty: TypeId, name: Sym) -> Result<bool> {
        Ok(self.types.as_struct(ty).is_some()
            && self.struct_constant(ty, name)?.is_none()
            && self.find_member(ty, name, Span::NONE)?.is_some())
    }

    /// `local.CONSTANT` where `local` belongs to an enclosing procedure: compile-time code
    /// (an `#insert` procedure) may read a constant member of the local's struct type,
    /// since that needs the type but not the value.
    fn enclosing_local_constant(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        base: &ast::Expr,
        member: &ast::Ident,
    ) -> Result<Option<Operand>> {
        let E::Ident(name) = &base.kind else {
            return Ok(None);
        };
        let Ok(ids) = self.lookup(scope, *name) else {
            return Ok(None);
        };
        let [id] = ids[..] else {
            return Ok(None);
        };
        let EntityKind::Local {
            ty,
            depth,
            ..
        } = self.entity(id).kind.clone()
        else {
            return Ok(None);
        };
        if depth == self.scope(scope).proc_depth {
            return Ok(None);
        }
        let ty = self.types.pointee(ty).unwrap_or(ty);
        if self.types.as_struct(ty).is_none() || self.names_field(ty, member.name)? {
            return Ok(None);
        }
        Ok(self
            .member_access(f, scope, Operand::Type(ty), member.name, member.span)
            .ok()
            .filter(|op| {
                matches!(
                    op,
                    Operand::Const { .. } | Operand::Type(_) | Operand::Procs(_)
                )
            }))
    }

    /// `#this`: the innermost enclosing struct type or, inside a procedure body, the procedure.
    fn check_this(&mut self, scope: ScopeId, span: Span) -> Result<Operand> {
        let mut s = Some(scope);
        while let Some(sid) = s {
            match self.scope(sid).kind {
                ScopeKind::Struct(t) => return Ok(Operand::Type(t)),
                ScopeKind::Proc => {
                    if let Some(p) = self.scope(sid).proc {
                        return Ok(Operand::Procs(vec![p]));
                    }
                }
                _ => {}
            }
            s = self.scope(sid).parent;
        }
        err(span, "#this used outside of a struct or procedure")
    }

    fn proc_type_from_header(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        header: &ast::ProcHeader,
    ) -> Result<TypeId> {
        // `#cpp_method` procedures use the C calling convention with the object as first argument.
        let c_call = header.flags.c_call || header.flags.cpp_method || header.foreign.is_some();
        let mut params = Vec::new();
        let mut variadic = false;
        let mut c_varargs = false;
        for p in &header.params {
            // `(s: string, start := 0) -> s64`: a parameter may take its default's type.
            let ty = match (&p.ty, &p.default) {
                (Some(t), _) => self.eval_type_in(f, scope, t)?,
                (None, Some(d)) => {
                    let op = self.check_expr_no_emit(scope, d)?;
                    self.settle_untyped(op, None).ty()
                }
                (None, None) => return err(p.span, "procedure type parameter needs a type"),
            };
            if p.variadic {
                if c_call {
                    c_varargs = true;
                    continue;
                }
                variadic = true;
                params.push(self.types.array(ty, ArrayKind::View));
            } else {
                params.push(ty);
            }
        }
        let mut returns = Vec::new();
        for r in &header.returns {
            let Some(rt) = &r.ty else {
                return err(r.span, "procedure type result needs a type");
            };
            let t = self.eval_type_in(f, scope, rt)?;
            if t != TypeId::VOID {
                returns.push(t);
            }
        }
        let ty = self.types.intern(TypeKind::Proc(Rc::new(ProcType {
            params,
            returns,
            variadic,
            c_varargs,
            c_call,
            no_context: c_call || header.flags.no_context,
            non_pod_return: header.flags.cpp_return_type_is_non_pod,
        })));
        if header.params.iter().any(|p| p.default.is_some()) {
            let info = ProcTypeParams {
                names: header
                    .params
                    .iter()
                    .map(|p| p.name.map(|n| n.name))
                    .collect(),
                defaults: header.params.iter().map(|p| p.default.clone()).collect(),
                scope,
            };
            self.proc_type_params.insert(ty, Rc::new(info));
        }
        Ok(ty)
    }

    // -----------------------------------------------------------------------
    // Operators
    // -----------------------------------------------------------------------

    fn check_unary(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        op: UnOp,
        a: &ast::Expr,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        match op {
            UnOp::Star => {
                // `*x[i]` calls `operator *[]` when the base type has one.
                if let E::Index(base, index) = &a.kind
                    && let Ok(probe) = self.check_expr_no_emit(scope, base)
                    && !self
                        .operator_candidates(scope, "*[]", &[probe.ty()])?
                        .is_empty()
                {
                    let mut base_op = self.check_expr(f, scope, base, None)?;
                    if let Operand::Place {
                        ty,
                        addr,
                    } = base_op
                    {
                        base_op = Operand::Value {
                            ty: self.types.pointer(ty),
                            val: addr,
                        };
                    }
                    let index_op = self.check_expr(f, scope, index, Some(TypeId::S64))?;
                    if let Some(result) = self
                        .try_index_operator_overload(f, scope, "*[]", &base_op, &index_op, span)?
                    {
                        return Ok(result);
                    }
                }
                let inner = self.check_expr(f, scope, a, None)?;
                match inner {
                    Operand::Type(t) => Ok(Operand::Type(self.types.pointer(t))),
                    Operand::Place {
                        ty,
                        addr,
                    } => Ok(Operand::Value {
                        ty: self.types.pointer(ty),
                        val: addr,
                    }),
                    Operand::Procs(p) if p.len() == 1 => {
                        let op = Operand::Procs(p);
                        let (ty, v) = self.rvalue(f, op, span)?;
                        Ok(Operand::Value {
                            ty,
                            val: v,
                        })
                    }
                    Operand::Const {
                        value: Value::Int(_) | Value::Float(_) | Value::Bool(_),
                        ..
                    } => err(
                        span,
                        "cannot take the address of a number or bool constant: it has no storage (assign it to a variable first)",
                    ),
                    other => {
                        // Address of a temporary.
                        let (ty, addr) = self.address_of(f, other, span)?;
                        Ok(Operand::Value {
                            ty: self.types.pointer(ty),
                            val: addr,
                        })
                    }
                }
            }
            UnOp::Deref => {
                let inner = self.check_expr(f, scope, a, None)?;
                let (ty, v) = self.rvalue(f, inner, span)?;
                let Some(pointee) = self.types.pointee(ty) else {
                    return err(
                        span,
                        format!("cannot dereference a value of type {}", self.types.name(ty)),
                    );
                };
                // `<< ptr` on a `*void` is a `void` place (it prints as `void`).
                Ok(Operand::Place {
                    ty: pointee,
                    addr: v,
                })
            }
            UnOp::Plus => self.check_expr(f, scope, a, expected),
            UnOp::Neg | UnOp::Not | UnOp::BitNot => {
                let inner = self.check_expr(
                    f,
                    scope,
                    a,
                    if op == UnOp::Not {
                        None
                    } else {
                        expected
                    },
                )?;
                if let Operand::Const {
                    ty,
                    value,
                    untyped,
                } = &inner
                {
                    let folded = match (op, value) {
                        (UnOp::Neg, Value::Int(v)) => Some(Value::Int(v.wrapping_neg())),
                        (UnOp::Neg, Value::Float(v)) => Some(Value::Float(-v)),
                        (UnOp::Not, Value::Bool(v)) => Some(Value::Bool(!v)),
                        (UnOp::Not, Value::Int(v)) => Some(Value::Bool(*v == 0)),
                        (UnOp::Not, Value::Null) => Some(Value::Bool(true)),
                        (UnOp::BitNot, Value::Int(v)) => {
                            let (bits, signed) = self.types.int_info(*ty).unwrap_or((64, true));
                            let bits = if *untyped {
                                expected
                                    .and_then(|e| self.types.int_info(e))
                                    .map_or(bits, |i| i.0)
                            } else {
                                bits
                            };
                            Some(Value::Int(wrap_int(!v, bits, signed && !*untyped)))
                        }
                        _ => None,
                    };
                    if let Some(value) = folded {
                        let ty = if op == UnOp::Not {
                            TypeId::BOOL
                        } else {
                            *ty
                        };
                        return Ok(Operand::Const {
                            ty,
                            value,
                            untyped: *untyped && op != UnOp::Not,
                        });
                    }
                }
                if let Some(result) =
                    self.try_unary_operator_overload(f, scope, op, &inner, span)?
                {
                    return Ok(result);
                }
                let inner = self.settle_untyped(inner, expected);
                let (ty, v) = self.rvalue(f, inner, span)?;
                // `-` and `~` are arithmetic: a `Type`, `bool`, pointer or other non-number has no meaning
                // under them, and a `Type` would otherwise become a garbage type id at run time.
                if op != UnOp::Not
                    && !self.types.is_integer(ty)
                    && !self.types.is_float(ty)
                    && self.wide_float(ty).is_none()
                {
                    let what = if op == UnOp::Neg {
                        "negate"
                    } else {
                        "bit-complement"
                    };
                    return Err(Box::new(Diagnostic::error(
                        span,
                        format!("cannot {what} {}", self.types.name(ty)),
                    )));
                }
                match op {
                    UnOp::Not => {
                        let b = self.truthy(f, ty, v, span)?;
                        let one = f.b.iconst(Ty::I8, 1);
                        Ok(Operand::Value {
                            ty: TypeId::BOOL,
                            val: f.b.bin(ir::BinOp::Xor, Ty::I8, b, one),
                        })
                    }
                    UnOp::Neg if self.wide_float(ty).is_some() => {
                        let fmt = self.wide_float(ty).expect("checked above");
                        Ok(self.wide_negate(f, fmt, ty, v))
                    }
                    UnOp::Neg => {
                        let t = self.ir_ty(ty).ok_or_else(|| {
                            Box::new(Diagnostic::error(
                                span,
                                format!("cannot negate {}", self.types.name(ty)),
                            ))
                        })?;
                        let op = if t.is_float() {
                            ir::UnOp::FNeg
                        } else {
                            ir::UnOp::Neg
                        };
                        Ok(Operand::Value {
                            ty,
                            val: f.b.un(op, t, v),
                        })
                    }
                    _ => {
                        let t = self.ir_ty(ty).filter(|t| !t.is_float()).ok_or_else(|| {
                            Box::new(Diagnostic::error(
                                span,
                                format!("cannot bit-complement {}", self.types.name(ty)),
                            ))
                        })?;
                        Ok(Operand::Value {
                            ty,
                            val: f.b.un(ir::UnOp::Not, t, v),
                        })
                    }
                }
            }
        }
    }

    /// The default type of a float literal: `float32`, unless the literal has 8 or more
    /// significant digits (`16777216.0`) or lies outside float32's normal range (`1.0e39`),
    /// which makes it `float64` as in Jai.
    fn float_literal_type(&self, v: f64, span: Span) -> TypeId {
        let text = self.sources.snippet(span);
        // A `0h` bit pattern is float32 with up to 8 hex digits and float64 with more
        // (`0h7FEFFFFF_FFFFFFFF`), whatever value it spells.
        if let Some(hex) = text.strip_prefix("0h").or_else(|| text.strip_prefix("0H")) {
            let digits = hex.bytes().filter(u8::is_ascii_hexdigit).count();
            return if digits > 8 {
                TypeId::F64
            } else {
                TypeId::F32
            };
        }
        // Outside float32's normal range (`1.0e39`, `1.0e-40`) the literal is float64.
        let magnitude = v.abs();
        if magnitude != 0.0 && (magnitude > f32::MAX as f64 || magnitude < f32::MIN_POSITIVE as f64)
        {
            return TypeId::F64;
        }
        // Significant digits run from the first nonzero digit to the end of the whole part, then
        // up to the last nonzero fraction digit: `16777216.0` has 8, `0.30000000` has 1. Eight or
        // more make the literal float64, even when float32 happens to hold the value exactly.
        let mantissa = text.split(['e', 'E']).next().unwrap_or("").replace('_', "");
        let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa.as_str(), ""));
        let digits = format!("{whole}{}", fraction.trim_end_matches('0'));
        let significant = digits.trim_start_matches('0').len();
        if significant >= 8 {
            TypeId::F64
        } else {
            TypeId::F32
        }
    }

    /// Give an untyped constant a concrete type from context (or its default).
    pub fn settle_untyped(&self, op: Operand, expected: Option<TypeId>) -> Operand {
        match op {
            Operand::Const {
                ty,
                value,
                untyped: true,
            } => {
                let target = expected.filter(|&e| {
                    let r = self.types.repr(e);
                    match &value {
                        Value::Int(_) => self.types.is_integer(r) || self.types.is_float(r),
                        Value::Float(_) => self.types.is_float(r),
                        Value::Null => {
                            self.types.is_pointer(r)
                                || matches!(self.types.kind(r), TypeKind::Proc(_))
                        }
                        _ => false,
                    }
                });
                let ty = target.unwrap_or_else(|| self.default_untyped(ty, &value));
                let value = match value {
                    Value::Int(i) if self.types.is_float(ty) => Value::Float(i as f64),
                    v => v,
                };
                Operand::Const {
                    ty,
                    value,
                    untyped: false,
                }
            }
            other => other,
        }
    }

    /// Convert a scalar value to a 0/1 `I8` condition.
    pub fn truthy(&mut self, f: &mut FnCtx, ty: TypeId, v: ir::Val, span: Span) -> Result<ir::Val> {
        let Some(t) = self.ir_ty(ty) else {
            if ty == TypeId::STRING {
                let count = f.b.load(Ty::I64, v);
                let zero = f.b.iconst(Ty::I64, 0);
                return Ok(f.b.cmp(CmpOp::Ne, Ty::I64, count, zero));
            }
            if let TypeKind::Array {
                kind: ArrayKind::Fixed(n),
                ..
            } = self.types.kind(ty)
            {
                // A fixed array's count is part of its type.
                return Ok(f.b.iconst(Ty::I8, (*n != 0) as u64));
            }
            if let TypeKind::Array {
                ..
            } = self.types.kind(ty)
            {
                let count = f.b.load(Ty::I64, v);
                let zero = f.b.iconst(Ty::I64, 0);
                return Ok(f.b.cmp(CmpOp::Ne, Ty::I64, count, zero));
            }
            return err(
                span,
                format!(
                    "a value of type {} cannot be used as a condition",
                    self.types.name(ty)
                ),
            );
        };
        if ty == TypeId::BOOL {
            return Ok(v);
        }
        if t.is_float() {
            let zero = f.b.fconst(t, 0.0);
            return Ok(f.b.cmp(CmpOp::FNe, t, v, zero));
        }
        let zero = f.b.iconst(t, 0);
        Ok(f.b.cmp(CmpOp::Ne, t, v, zero))
    }

    /// Check an expression used as a condition, producing an `I8`.
    pub fn check_condition(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        expr: &ast::Expr,
    ) -> Result<ir::Val> {
        let op = self.check_expr(f, scope, expr, Some(TypeId::BOOL))?;
        let op = self.settle_untyped(op, None);
        let (ty, v) = self.rvalue(f, op, expr.span)?;
        self.truthy(f, ty, v, expr.span)
    }

    #[allow(clippy::too_many_arguments)]
    fn check_binary(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        op: BinOp,
        a: &ast::Expr,
        b: &ast::Expr,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        if matches!(op, BinOp::And | BinOp::Or) {
            return self.check_logical(f, scope, op, a, b, span);
        }
        let is_cmp = matches!(
            op,
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge
        );
        let is_shift = matches!(op, BinOp::Shl | BinOp::Shr | BinOp::Rotl | BinOp::Rotr);
        // `cast(float32) ((ifx c then a else b) & mask)`: a float target cannot be the type of an
        // operand of an operator that floats lack, so it does not reach the operands (an `ifx`
        // would widen to it, and the operator would then fail on a float).
        let no_float_operands = matches!(
            op,
            BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Rem
        );
        let expected = if no_float_operands {
            expected.filter(|&t| !self.types.is_float(self.types.repr(t)))
        } else {
            expected
        };
        let lhs_expected = if is_cmp {
            None
        } else if is_shift {
            // `cast(float) (1 << n)` shifts an integer.
            expected.filter(|&t| self.types.is_integer(self.types.repr(t)))
        } else {
            expected
        };
        // `.FIRST == x`: an inferred member on the left takes the right operand's type, so
        // the right side is checked first.
        let mut early_rhs = None;
        let is_xx = |e: &ast::Expr| {
            matches!(
                e.kind,
                E::Cast {
                    ty: None,
                    ..
                }
            )
        };
        // `.FLAG & x.flags` too: the member takes the flags type, not an expected `int`.
        // An expected enum type already names the member (`.A & ~.B` as a flags argument).
        let bitwise = matches!(op, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor)
            && !lhs_expected.is_some_and(|t| {
                matches!(
                    self.types.kind(self.types.repr_struct(t)),
                    TypeKind::Enum(_)
                )
            });
        let mut lhs = if ((is_cmp
            && (matches!(a.kind, E::InferredMember(_)) || (is_xx(a) && !is_xx(b))))
            || (bitwise && matches!(a.kind, E::InferredMember(_))))
            && !matches!(b.kind, E::InferredMember(_))
        {
            // `xx err == GL_FALSE` casts to the other operand's type too.
            let r = self.check_expr(f, scope, b, None)?;
            let l = self.check_expr(f, scope, a, Some(r.ty()))?;
            early_rhs = Some(r);
            l
        } else if lhs_expected.is_some() && is_number_literal(a) {
            // `x: float32 = 0 - w`: the literal is matched with `w`, not with the declaration, so
            // the subtraction is in `u16` and only its result converts. It stays untyped here and
            // takes the right operand's type below; with an untyped right operand, both fold
            // under the expected type as before {#num.16}.
            self.check_expr(f, scope, a, None)?
        } else {
            self.check_expr(f, scope, a, lhs_expected)?
        };
        if let Operand::PolyStruct(ps) = lhs {
            lhs = Operand::Type(self.poly_struct_type(ps));
        }
        // Inferred enum members on the right take the left operand's type.
        let pointer_lhs = self.types.is_pointer(self.types.repr(lhs.ty()));
        let pointer_offset = matches!(op, BinOp::Add | BinOp::Sub) && pointer_lhs;
        let pointer_bits =
            matches!(op, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor) && pointer_lhs;
        let rhs_expected = if is_shift || pointer_bits {
            None
        } else if pointer_offset {
            // `p += ifx c then 3 else 1`: the offset is an integer.
            Some(TypeId::S64)
        } else {
            (!matches!(
                lhs,
                Operand::Const {
                    untyped: true,
                    ..
                }
            ))
            .then(|| lhs.ty())
        };
        let mut rhs = match early_rhs {
            Some(r) => r,
            None => self.check_expr(
                f,
                scope,
                b,
                rhs_expected.or(if is_cmp || is_shift {
                    None
                } else {
                    expected
                }),
            )?,
        };
        if let Operand::PolyStruct(ps) = rhs {
            rhs = Operand::Type(self.poly_struct_type(ps));
        }
        // `s[0] == "-"`, `c - "0"`: a one-byte string constant next to an integer compares
        // and offsets as that byte.
        if is_cmp || matches!(op, BinOp::Add | BinOp::Sub) {
            let byte_of = |c: &Compiler, op: &Operand, other: &Operand| match op {
                Operand::Const {
                    value: Value::String(bytes),
                    ..
                } if bytes.len() == 1 && c.types.is_integer(other.ty()) => {
                    Some(Operand::untyped_int(bytes[0] as i128))
                }
                _ => None,
            };
            if let Some(b) = byte_of(self, &rhs, &lhs) {
                rhs = b;
            } else if let Some(b) = byte_of(self, &lhs, &rhs) {
                lhs = b;
            }
        }
        if matches!(
            lhs,
            Operand::Const {
                untyped: true,
                ..
            }
        ) && !matches!(
            rhs,
            Operand::Const {
                untyped: true,
                ..
            }
        ) && !is_shift
        {
            // Re-check the left side with the right side's type for inferred members.
            if matches!(a.kind, E::InferredMember(_)) {
                lhs = self.check_expr(f, scope, a, Some(rhs.ty()))?;
            }
        }
        if matches!(a.kind, E::InferredMember(_)) && !matches!(b.kind, E::InferredMember(_)) {
            lhs = self.check_expr(f, scope, a, Some(rhs.ty()))?;
        }
        if matches!(b.kind, E::InferredMember(_)) {
            rhs = self.check_expr(f, scope, b, Some(lhs.ty()))?;
        }
        // Type comparisons fold at compile time.
        if let (Operand::Type(x), Operand::Type(y)) = (&lhs, &rhs) {
            return match op {
                BinOp::Eq => Ok(Operand::bool(x == y)),
                BinOp::Ne => Ok(Operand::bool(x != y)),
                _ => err(span, "types can only be compared with == and !="),
            };
        }
        if let Some(result) = self.try_binary_operator_overload(f, scope, op, &lhs, &rhs, span)? {
            return Ok(result);
        }
        if let (
            Operand::Const {
                ..
            },
            Operand::Const {
                ..
            },
        ) = (&lhs, &rhs)
            && let Some(folded) = self.fold_binary(op, &lhs, &rhs, expected, span)?
        {
            return Ok(folded);
        }
        // Two procedure names compare as their addresses.
        if let (Operand::Procs(l), Operand::Procs(r)) = (&lhs, &rhs)
            && l.len() == 1
            && r.len() == 1
        {
            let (lt, rt) = (self.proc_type(l[0], span)?, self.proc_type(r[0], span)?);
            let converted = self.convert(f, lhs, lt, span)?;
            let (ty, val) = self.rvalue(f, converted, span)?;
            lhs = Operand::Value {
                ty,
                val,
            };
            let converted = self.convert(f, rhs, rt, span)?;
            let (ty, val) = self.rvalue(f, converted, span)?;
            rhs = Operand::Value {
                ty,
                val,
            };
        }
        // Procedure names compared with procedure values take the value's type.
        if matches!(rhs, Operand::Procs(_)) && !matches!(lhs, Operand::Procs(_)) {
            let converted = self.convert(f, rhs, lhs.ty(), span)?;
            let (ty, val) = self.rvalue(f, converted, span)?;
            rhs = Operand::Value {
                ty,
                val,
            };
        } else if matches!(lhs, Operand::Procs(_)) && !matches!(rhs, Operand::Procs(_)) {
            let converted = self.convert(f, lhs, rhs.ty(), span)?;
            let (ty, val) = self.rvalue(f, converted, span)?;
            lhs = Operand::Value {
                ty,
                val,
            };
        }
        let (lty, rty) = (lhs.ty(), rhs.ty());
        // Pointer arithmetic.
        if !is_cmp && self.types.is_pointer(lty) && matches!(op, BinOp::Add | BinOp::Sub) {
            let (_, base) = self.rvalue(f, lhs, span)?;
            if op == BinOp::Sub && self.types.is_pointer(rty) {
                let (_, other) = self.rvalue(f, rhs, span)?;
                let pointee = self.types.pointee(lty).unwrap();
                let size = if pointee == TypeId::VOID {
                    1
                } else {
                    self.size_of(pointee, span)?.max(1)
                };
                let x = f.b.conv(ir::ConvOp::Bitcast, Ty::Ptr, Ty::I64, base);
                let y = f.b.conv(ir::ConvOp::Bitcast, Ty::Ptr, Ty::I64, other);
                let diff = f.b.bin(ir::BinOp::Sub, Ty::I64, x, y);
                let s = f.b.iconst(Ty::I64, size);
                return Ok(Operand::Value {
                    ty: TypeId::S64,
                    val: f.b.bin(ir::BinOp::SDiv, Ty::I64, diff, s),
                });
            }
            // Any runtime integer offsets a pointer (`data + total_read` with a u64).
            let rhs = if self.types.is_integer(rty) && !matches!(rhs, Operand::Const { .. }) {
                self.explicit_cast(f, rhs, TypeId::S64, ast::CastFlags::default(), span)?
            } else {
                self.convert(f, rhs, TypeId::S64, span)?
            };
            let (_, idx) = self.rvalue(f, rhs, span)?;
            let pointee = self.types.pointee(lty).unwrap();
            let size = if pointee == TypeId::VOID {
                1
            } else {
                self.size_of(pointee, span)?
            };
            let s = f.b.iconst(Ty::I64, size);
            let mut off = f.b.bin(ir::BinOp::Mul, Ty::I64, idx, s);
            if op == BinOp::Sub {
                off = f.b.un(ir::UnOp::Neg, Ty::I64, off);
            }
            return Ok(Operand::Value {
                ty: lty,
                val: f.b.ptr_add(base, off),
            });
        }
        if !is_cmp && self.types.is_pointer(rty) && op == BinOp::Add && self.types.is_integer(lty) {
            return self.check_binary(f, scope, op, b, a, expected, span);
        }
        // Masking an address (`p & (alignment - 1)`; Jai reads `cast(u64) p & MASK` as
        // `cast(u64) (p & MASK)`): the integer meets the pointer's bits, and the result
        // keeps the pointer's type.
        let ptr_on_left = self.types.is_pointer(lty) && self.types.is_integer(rty);
        let ptr_on_right = self.types.is_pointer(rty) && self.types.is_integer(lty);
        if matches!(op, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor)
            && (ptr_on_left || ptr_on_right)
        {
            let (ptr, int) = if ptr_on_left {
                (lhs, rhs)
            } else {
                (rhs, lhs)
            };
            let ptr_ty = ptr.ty();
            let (_, address) = self.rvalue(f, ptr, span)?;
            let bits = ast::CastFlags {
                no_check: true,
                ..Default::default()
            };
            let int = self.explicit_cast(f, int, TypeId::U64, bits, span)?;
            let (_, mask) = self.rvalue(f, int, span)?;
            let address = f.b.conv(ir::ConvOp::Bitcast, Ty::Ptr, Ty::I64, address);
            let ir_op = match op {
                BinOp::BitAnd => ir::BinOp::And,
                BinOp::BitOr => ir::BinOp::Or,
                _ => ir::BinOp::Xor,
            };
            let combined = f.b.bin(ir_op, Ty::I64, address, mask);
            return Ok(Operand::Value {
                ty: ptr_ty,
                val: f.b.conv(ir::ConvOp::Bitcast, Ty::I64, Ty::Ptr, combined),
            });
        }
        // An integer variable meeting an untyped float constant (`n * 0.5`) is converted to float.
        let is_float_lit = |o: &Operand| {
            matches!(
                o,
                Operand::Const {
                    value: Value::Float(_),
                    untyped: true,
                    ..
                }
            )
        };
        let is_int_value = |s: &Self, o: &Operand| {
            !matches!(
                o,
                Operand::Const {
                    untyped: true,
                    ..
                }
            ) && s.types.is_integer(o.ty())
                && !matches!(s.types.kind(o.ty()), TypeKind::Enum(_))
        };
        if !is_shift && is_float_lit(&rhs) && is_int_value(self, &lhs) {
            lhs = self.explicit_cast(f, lhs, TypeId::F32, ast::CastFlags::default(), span)?;
        } else if !is_shift && is_float_lit(&lhs) && is_int_value(self, &rhs) {
            rhs = self.explicit_cast(f, rhs, TypeId::F32, ast::CastFlags::default(), span)?;
        }
        // Unify operand types.
        let patterns = (self.bit_pattern(a.span), self.bit_pattern(b.span));
        let ty = self.binary_operand_type(&lhs, &rhs, expected, is_shift, patterns, span)?;
        if self.types.repr(ty) == TypeId::BOOL
            && matches!(
                op,
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem
            )
        {
            let symbol = match op {
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Mul => "*",
                BinOp::Div => "/",
                _ => "%",
            };
            return err(span, format!("operator `{symbol}` is not defined for bool"));
        }
        // A literal that met the other operand's type takes it as a bit pattern
        // (`mask & ~0x7` with a `u32` mask).
        if !is_shift {
            lhs = self.literal_as_bits(lhs, ty);
            rhs = self.literal_as_bits(rhs, ty);
        }
        let rhs_ty = if is_shift {
            self.shift_amount_type(&rhs, ty)
        } else {
            ty
        };
        // An integer value meeting a float converts like `cast(float) x`.
        let int_to_float =
            |s: &Self, o: &Operand| !is_shift && s.types.is_float(ty) && s.types.is_integer(o.ty());
        if int_to_float(self, &lhs) {
            lhs = self.explicit_cast(f, lhs, ty, ast::CastFlags::default(), span)?;
        }
        if int_to_float(self, &rhs) {
            rhs = self.explicit_cast(f, rhs, ty, ast::CastFlags::default(), span)?;
        }
        if ty == TypeId::VOID_PTR && matches!(self.types.kind(lhs.ty()), TypeKind::Proc(_)) {
            lhs = self.explicit_cast(f, lhs, ty, ast::CastFlags::default(), span)?;
        }
        if ty == TypeId::VOID_PTR && matches!(self.types.kind(rhs.ty()), TypeKind::Proc(_)) {
            rhs = self.explicit_cast(f, rhs, ty, ast::CastFlags::default(), span)?;
        }
        let lhs = self.convert(f, lhs, ty, span)?;
        let rhs = self.convert(f, rhs, rhs_ty, span)?;
        let (_, x) = self.rvalue(f, lhs, span)?;
        let (_, y) = self.rvalue(f, rhs, span)?;
        if ty == TypeId::STRING && is_cmp {
            return self.string_compare(f, op, x, y, span);
        }
        if self.wide_float(ty).is_some() && !is_shift {
            return self.wide_binary(f, ty, op, x, y, span);
        }
        let Some(t) = self.ir_ty(ty) else {
            if is_cmp && matches!(op, BinOp::Eq | BinOp::Ne) {
                // Aggregate equality: bytewise.
                let size = self.size_of(ty, span)?;
                let n = f.b.iconst(Ty::I64, size);
                let r =
                    f.b.intrinsic(ir::Intrinsic::Memcmp, vec![x, y, n], &[Ty::I16]);
                let zero = f.b.iconst(Ty::I16, 0);
                let cmp = if op == BinOp::Eq {
                    CmpOp::Eq
                } else {
                    CmpOp::Ne
                };
                return Ok(Operand::Value {
                    ty: TypeId::BOOL,
                    val: f.b.cmp(cmp, Ty::I16, r[0], zero),
                });
            }
            return err(
                span,
                format!(
                    "operator {:?} is not defined for {}",
                    op,
                    self.types.name(ty)
                ),
            );
        };
        let signed = self.types.int_info(ty).is_some_and(|(_, s)| s);
        if is_cmp {
            let cmp = cmp_op(op, t.is_float(), signed);
            return Ok(Operand::Value {
                ty: TypeId::BOOL,
                val: f.b.cmp(cmp, t, x, y),
            });
        }
        if matches!(op, BinOp::Div | BinOp::Rem) && !t.is_float() && !f.type_only {
            // Native code reports a division by zero through Runtime_Support.
            self.note_check_handler(span);
        }
        let ir_op = arith_op(op, t.is_float(), signed).ok_or_else(|| {
            Box::new(Diagnostic::error(
                span,
                format!(
                    "operator {:?} is not defined for {}",
                    op,
                    self.types.name(ty)
                ),
            ))
        })?;
        if t == Ty::Ptr {
            return err(
                span,
                format!(
                    "arithmetic operator {:?} is not defined for {}",
                    op,
                    self.types.name(ty)
                ),
            );
        }
        let y = if is_shift {
            let rt = self.ir_ty(rhs_ty).unwrap();
            if rt.size() != t.size() {
                let conv = if rt.size() > t.size() {
                    ir::ConvOp::Trunc
                } else {
                    ir::ConvOp::ZExt
                };
                f.b.conv(conv, rt, t, y)
            } else {
                y
            }
        } else {
            y
        };
        let val = f.b.bin(ir_op, t, x, y);
        if self.options.arithmetic_overflow_check != 0
            && !f.no_aoc
            && !f.type_only
            && matches!(ir_op, ir::BinOp::Add | ir::BinOp::Sub | ir::BinOp::Mul)
            && self.types.is_integer(ty)
            && !matches!(self.types.kind(ty), TypeKind::Enum(_))
        {
            self.emit_overflow_check(f, ir_op, signed, t, x, y, span)?;
        }
        Ok(Operand::Value {
            ty,
            val,
        })
    }

    /// After `x op y` on integers of IR type `t`: call the runtime's `__arithmetic_overflow`
    /// when the result did not fit (`Build_Options.arithmetic_overflow_check`).
    #[allow(clippy::too_many_arguments)]
    fn emit_overflow_check(
        &mut self,
        f: &mut FnCtx,
        op: ir::BinOp,
        signed: bool,
        t: Ty,
        x: ir::Val,
        y: ir::Val,
        span: Span,
    ) -> Result<()> {
        use ir::Intrinsic as I;
        let (intrinsic, operator) = match (op, signed) {
            (ir::BinOp::Add, true) => (I::SAddOverflow, 1u64),
            (ir::BinOp::Add, false) => (I::UAddOverflow, 1),
            (ir::BinOp::Sub, true) => (I::SSubOverflow, 2),
            (ir::BinOp::Sub, false) => (I::USubOverflow, 2),
            (ir::BinOp::Mul, true) => (I::SMulOverflow, 3),
            _ => (I::UMulOverflow, 3),
        };
        let Some(runtime) = self.runtime_support else {
            return Ok(());
        };
        let handler = self.module_declarations(runtime, Sym::intern("__arithmetic_overflow"))?;
        let entity = match handler[..] {
            [entity] => entity,
            [] => {
                return err(
                    span,
                    "Runtime_Support does not define `__arithmetic_overflow`",
                );
            }
            _ => {
                return err(
                    span,
                    "Runtime_Support defines `__arithmetic_overflow` more than once",
                );
            }
        };
        let Resolved::Proc(proc) = self.resolve_entity(entity)? else {
            return err(span, "`__arithmetic_overflow` is not a procedure");
        };
        let super::procs::ProcTarget::Func(func) = self.proc_func(proc, span)? else {
            return err(span, "`__arithmetic_overflow` must be defined in Jai");
        };
        let width = f.b.iconst(Ty::I64, t.size());
        let overflowed = f.b.intrinsic(intrinsic, vec![x, y, width], &[Ty::I8])[0];
        let fail = f.b.new_block();
        let cont = f.b.new_block();
        f.b.branch(overflowed, fail, cont);
        f.b.switch_to(fail);
        let widen = if signed {
            ir::ConvOp::SExt
        } else {
            ir::ConvOp::ZExt
        };
        let (left, right) = if t == Ty::I64 {
            (x, y)
        } else {
            (
                f.b.conv(widen, t, Ty::I64, x),
                f.b.conv(widen, t, Ty::I64, y),
            )
        };
        // Same layout as the reference: bit 15 fatal, bit 14 signed, bits 7-8 operator, low bits size.
        let code = (u64::from(self.options.arithmetic_overflow_check == 2) << 15)
            | (u64::from(signed) << 14)
            | (operator << 7)
            | t.size();
        let code = f.b.iconst(Ty::I16, code);
        let file = self.sources.get(span.file);
        let (line, _) = file.line_col(span.start);
        let path: Rc<[u8]> = Rc::from(file.path.as_bytes());
        let line = f.b.iconst(Ty::I64, line as u64);
        let global = self.string_global(&path);
        let filename = f.b.global_addr(global);
        f.b.call(
            ir::Callee::Func(func),
            vec![left, right, code, line, filename],
            &[],
        );
        f.b.jump(cont);
        f.b.switch_to(cont);
        Ok(())
    }

    /// Runtime_Support's `runtime_support_check_failed`, which native code calls to report a
    /// failed check (`ir::Program::check_failed`). Resolved once, when the first check is
    /// emitted; without it native code traps without a message.
    pub(crate) fn note_check_handler(&mut self, span: Span) {
        if self.check_handler_resolved {
            return;
        }
        self.check_handler_resolved = true;
        let Some(runtime) = self.runtime_support else {
            return;
        };
        let Ok(found) =
            self.module_declarations(runtime, Sym::intern("runtime_support_check_failed"))
        else {
            return;
        };
        let Some(&entity) = found.first() else {
            return;
        };
        if let Ok(Resolved::Proc(proc)) = self.resolve_entity(entity)
            && let Ok(super::procs::ProcTarget::Func(func)) = self.proc_func(proc, span)
        {
            self.program.check_failed = Some(func);
        }
    }

    /// Stop the program at a failed check that has no details (`ir::Intrinsic::Trap` with an
    /// `ir::TRAP_*` reason). Every trap with a reason goes through here or `emit_check_failed`,
    /// so native code always has the reporting procedure.
    pub(crate) fn emit_trap(&mut self, f: &mut FnCtx, reason: u64, span: Span) {
        self.note_check_handler(span);
        let reason = f.b.iconst(Ty::I64, reason);
        f.b.intrinsic(ir::Intrinsic::Trap, vec![reason], &[]);
    }

    /// Report a failed check (`ir::Intrinsic::CheckFailed`): `reason` is an `ir::TRAP_*`, `a`
    /// and `b` its `I64` details. A fatal failure stops the program.
    pub(crate) fn emit_check_failed(
        &mut self,
        f: &mut FnCtx,
        reason: u64,
        a: ir::Val,
        b: ir::Val,
        fatal: bool,
        span: Span,
    ) {
        self.note_check_handler(span);
        f.b.repeat_loc();
        let reason = f.b.iconst(Ty::I64, reason);
        let fatal = f.b.iconst(Ty::I8, u64::from(fatal));
        f.b.intrinsic(ir::Intrinsic::CheckFailed, vec![reason, a, b, fatal], &[]);
    }

    /// The type an untyped literal takes next to a typed operand: the operand's
    /// type, or a 64-bit integer when the literal does not fit (`small + -1` and
    /// `small == -1` with a `u8` work in `s64`). A hex, binary or `~` literal
    /// (`bit_pattern`) may still fill the operand's bits.
    fn literal_meets(&self, lit: &Operand, ty: TypeId, bit_pattern: bool) -> TypeId {
        if let Operand::Const {
            value: Value::Int(v),
            ..
        } = lit
            && let Some((bits, signed)) = self.types.int_info(ty)
            && !super::convert::int_constant_fits(*v, bits, signed, bit_pattern)
            && !(bit_pattern && super::convert::int_fits(*v, bits, signed))
            && !matches!(self.types.kind(ty), TypeKind::Enum(_))
        {
            return if super::convert::int_fits(*v, 64, true) {
                TypeId::S64
            } else {
                TypeId::U64
            };
        }
        ty
    }

    /// An untyped integer literal operand of integer type `ty`, wrapped to `ty`'s bits when
    /// it only fits as a bit pattern.
    fn literal_as_bits(&self, op: Operand, ty: TypeId) -> Operand {
        match op {
            Operand::Const {
                value: Value::Int(v),
                untyped: true,
                ..
            } if !matches!(self.types.kind(ty), TypeKind::Enum(_))
                && let Some((bits, signed)) = self.types.int_info(ty)
                && bits <= 64
                && super::convert::int_fits(v, bits, signed) =>
            {
                Operand::Const {
                    ty,
                    value: Value::Int(wrap_int(v, bits, signed)),
                    untyped: false,
                }
            }
            op => op,
        }
    }

    fn shift_amount_type(&self, rhs: &Operand, lhs_ty: TypeId) -> TypeId {
        match rhs {
            Operand::Const {
                untyped: true, ..
            } => lhs_ty,
            other => {
                let t = other.ty();
                if self.types.is_integer(t) {
                    t
                } else {
                    lhs_ty
                }
            }
        }
    }

    /// The common type of a binary operation.
    /// `x[i]` through `operator *[]`: the returned pointer is the element's place.
    fn try_pointer_index_operator(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        base: &Operand,
        index: &Operand,
        span: Span,
    ) -> Result<Option<Operand>> {
        let base = match base {
            Operand::Place {
                ty,
                addr,
            } => Operand::Value {
                ty: self.types.pointer(*ty),
                val: *addr,
            },
            Operand::Value {
                ty, ..
            } if self.types.pointee(*ty).is_some() => base.clone(),
            _ => return Ok(None),
        };
        let target = self.types.pointee(base.ty()).unwrap();
        if self
            .operator_candidates(scope, "*[]", &[target])?
            .is_empty()
        {
            return Ok(None);
        }
        let Some(Operand::Value {
            ty,
            val,
        }) = self.try_index_operator_overload(f, scope, "*[]", &base, index, span)?
        else {
            return Ok(None);
        };
        Ok(self.types.pointee(ty).map(|elem| Operand::Place {
            ty: elem,
            addr: val,
        }))
    }

    fn binary_operand_type(
        &mut self,
        lhs: &Operand,
        rhs: &Operand,
        expected: Option<TypeId>,
        is_shift: bool,
        patterns: (bool, bool),
        span: Span,
    ) -> Result<TypeId> {
        let lu = matches!(
            lhs,
            Operand::Const {
                untyped: true,
                ..
            }
        );
        let ru = matches!(
            rhs,
            Operand::Const {
                untyped: true,
                ..
            }
        );
        let (lt, rt) = (lhs.ty(), rhs.ty());
        if is_shift {
            if lu {
                return Ok(expected
                    .filter(|&e| self.types.is_integer(e))
                    .unwrap_or(TypeId::S64));
            }
            return Ok(lt);
        }
        match (lu, ru) {
            (true, true) => {
                let lf = matches!(
                    lhs,
                    Operand::Const {
                        value: Value::Float(_),
                        ..
                    }
                );
                let rf = matches!(
                    rhs,
                    Operand::Const {
                        value: Value::Float(_),
                        ..
                    }
                );
                if lt == TypeId::NULL || rt == TypeId::NULL {
                    return Ok(TypeId::VOID_PTR);
                }
                Ok(if lf || rf {
                    expected
                        .filter(|&e| self.types.is_float(e))
                        .unwrap_or(TypeId::F32)
                } else {
                    expected
                        .filter(|&e| self.types.is_integer(e) || self.types.is_float(e))
                        .unwrap_or(TypeId::S64)
                })
            }
            (true, false) => Ok(self.literal_meets(lhs, rt, patterns.0)),
            (false, true) => Ok(self.literal_meets(rhs, lt, patterns.1)),
            (false, false) => {
                if lt == rt {
                    return Ok(lt);
                }
                // Widening within the same class.
                if self.implicit_cost(rt, false, lt).is_some() {
                    return Ok(lt);
                }
                if self.implicit_cost(lt, false, rt).is_some() {
                    return Ok(rt);
                }
                // An integer meeting a float in arithmetic becomes that float.
                if self.types.is_float(lt) && self.types.is_integer(rt) {
                    return Ok(lt);
                }
                if self.types.is_integer(lt) && self.types.is_float(rt) {
                    return Ok(rt);
                }
                let (li, ri) = (self.types.int_info(lt), self.types.int_info(rt));
                if let (Some((lb, _)), Some((rb, _))) = (li, ri) {
                    // Mixed signedness: Jai requires a cast, but comparisons with
                    // literals are common; pick the wider type.
                    return Ok(if lb >= rb {
                        lt
                    } else {
                        rt
                    });
                }
                // A procedure value meets a `*void` as an address (`proc == get_caller_address()`).
                let is_proc = |c: &Self, t| matches!(c.types.kind(t), TypeKind::Proc(_));
                if (is_proc(self, lt) && rt == TypeId::VOID_PTR)
                    || (is_proc(self, rt) && lt == TypeId::VOID_PTR)
                {
                    return Ok(TypeId::VOID_PTR);
                }
                let mut d = Diagnostic::error(
                    span,
                    format!(
                        "type mismatch: `{}` and `{}` cannot be combined",
                        self.types.name(lt),
                        self.types.name(rt)
                    ),
                );
                // `c * "2"`: a one-character string where a character code was meant (`+` and `-`
                // take it as the byte).
                let one_char = |o: &Operand| match o {
                    Operand::Const {
                        value: Value::String(s),
                        ..
                    } if s.len() == 1 => Some(s[0] as char),
                    _ => None,
                };
                let (int_side, text) = match (one_char(lhs), one_char(rhs)) {
                    (_, Some(c)) => (lt, Some(c)),
                    (Some(c), _) => (rt, Some(c)),
                    _ => (lt, None),
                };
                if let Some(c) = text
                    && self.types.is_integer(int_side)
                {
                    d = d.with_help(format!(
                        "`\"{c}\"` is a string; for the character's code write `#char \"{c}\"`"
                    ));
                } else if self.types.is_integer(lt) && self.types.is_float(rt)
                    || self.types.is_float(lt) && self.types.is_integer(rt)
                {
                    d = d.with_help("convert one side with `cast(float)` or `cast(int)` first");
                } else if self.types.is_integer(lt) && self.types.is_integer(rt) {
                    d = d.with_help(
                        "integers of different sizes or signedness need a `cast` to one type",
                    );
                }
                Err(Box::new(d))
            }
        }
    }

    fn fold_binary(
        &mut self,
        op: BinOp,
        lhs: &Operand,
        rhs: &Operand,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Option<Operand>> {
        let (
            Operand::Const {
                ty: lt,
                value: lv,
                untyped: lu,
            },
            Operand::Const {
                ty: rt,
                value: rv,
                untyped: ru,
            },
        ) = (lhs, rhs)
        else {
            return Ok(None);
        };
        if self.wide_float(*lt).is_some() || self.wide_float(*rt).is_some() {
            return Ok(self.wide_fold(op, (*lt, lv), (*rt, rv)));
        }
        let untyped = *lu && *ru;
        let is_shift = matches!(op, BinOp::Shl | BinOp::Shr | BinOp::Rotl | BinOp::Rotr);
        let plain_int = |c: &Self, t: TypeId| {
            c.types.int_info(t).is_some() && !matches!(c.types.kind(t), TypeKind::Enum(_))
        };
        let ty = if *lu && !*ru && !is_shift {
            *rt
        } else if !*lu
            && !*ru
            && !is_shift
            && lt != rt
            && plain_int(self, *lt)
            && plain_int(self, *rt)
        {
            // Two typed integer constants meet in the wider type, as variables do
            // (`cast(s16) 1 + cast(s32) 2` is `s32`).
            if self.implicit_cost(*rt, false, *lt).is_some() {
                *lt
            } else if self.implicit_cost(*lt, false, *rt).is_some() {
                *rt
            } else if self.types.int_info(*lt).unwrap().0 >= self.types.int_info(*rt).unwrap().0 {
                *lt
            } else {
                *rt
            }
        } else {
            *lt
        };
        let cmp = |o: std::cmp::Ordering| -> Option<Value> {
            use std::cmp::Ordering::*;
            Some(Value::Bool(match op {
                BinOp::Eq => o == Equal,
                BinOp::Ne => o != Equal,
                BinOp::Lt => o == Less,
                BinOp::Le => o != Greater,
                BinOp::Gt => o == Greater,
                BinOp::Ge => o != Less,
                _ => return None,
            }))
        };
        let result = match (lv, rv) {
            (Value::Int(a), Value::Int(b)) if !self.types.is_float(ty) => {
                let (a, b) = (*a, *b);
                let v = match op {
                    BinOp::Add => Some(Value::Int(a.wrapping_add(b))),
                    BinOp::Sub => Some(Value::Int(a.wrapping_sub(b))),
                    BinOp::Mul => Some(Value::Int(a.wrapping_mul(b))),
                    BinOp::Div => {
                        if b == 0 {
                            return err(span, "division by zero in constant expression");
                        }
                        Some(Value::Int(a.wrapping_div(b)))
                    }
                    BinOp::Rem => {
                        if b == 0 {
                            return err(span, "division by zero in constant expression");
                        }
                        Some(Value::Int(a.wrapping_rem(b)))
                    }
                    BinOp::BitAnd => Some(Value::Int(a & b)),
                    BinOp::BitOr => Some(Value::Int(a | b)),
                    BinOp::BitXor => Some(Value::Int(a ^ b)),
                    BinOp::Shl | BinOp::Shr if b < 0 => {
                        return err(
                            span,
                            format!("negative shift amount {b} in constant expression"),
                        );
                    }
                    BinOp::Shl => Some(Value::Int(if b >= 128 {
                        0
                    } else {
                        a << b
                    })),
                    BinOp::Shr => Some(Value::Int(if b >= 128 {
                        0
                    } else {
                        a >> b
                    })),
                    BinOp::Rotl | BinOp::Rotr => None,
                    _ => cmp(a.cmp(&b)),
                };
                // Wrap typed results to their width.
                match v {
                    Some(Value::Int(x)) if !untyped => {
                        let (bits, signed) = self.types.int_info(ty).unwrap_or((64, true));
                        Some(Value::Int(wrap_int(x, bits, signed)))
                    }
                    Some(Value::Int(x)) => {
                        if let Some((bits, signed)) = expected.and_then(|e| self.types.int_info(e))
                        {
                            let _ = (bits, signed);
                        }
                        Some(Value::Int(x))
                    }
                    other => other,
                }
            }
            (Value::Int(_) | Value::Float(_), Value::Int(_) | Value::Float(_)) => {
                let a = num(lv);
                let b = num(rv);
                match op {
                    BinOp::Add => Some(Value::Float(a + b)),
                    BinOp::Sub => Some(Value::Float(a - b)),
                    BinOp::Mul => Some(Value::Float(a * b)),
                    BinOp::Div => Some(Value::Float(a / b)),
                    _ => a.partial_cmp(&b).and_then(cmp),
                }
            }
            (Value::Bool(a), Value::Bool(b)) => match op {
                BinOp::Eq => Some(Value::Bool(a == b)),
                BinOp::Ne => Some(Value::Bool(a != b)),
                BinOp::BitAnd => Some(Value::Bool(*a & *b)),
                BinOp::BitOr => Some(Value::Bool(*a | *b)),
                BinOp::BitXor => Some(Value::Bool(*a ^ *b)),
                _ => None,
            },
            (Value::String(a), Value::String(b)) => match op {
                BinOp::Eq => Some(Value::Bool(a == b)),
                BinOp::Ne => Some(Value::Bool(a != b)),
                _ => None,
            },
            (Value::Null, Value::Null) => cmp(std::cmp::Ordering::Equal),
            (Value::Proc(a), Value::Proc(b)) => match op {
                BinOp::Eq => Some(Value::Bool(a == b)),
                BinOp::Ne => Some(Value::Bool(a != b)),
                _ => None,
            },
            _ => None,
        };
        let Some(value) = result else {
            return Ok(None);
        };
        let is_bool = matches!(value, Value::Bool(_));
        let rty = if is_bool
            && !matches!((lv, rv), (Value::Bool(_), Value::Bool(_)) if !matches!(op, BinOp::Eq | BinOp::Ne))
        {
            TypeId::BOOL
        } else if matches!(value, Value::Float(_)) && !self.types.is_float(ty) {
            if self.types.is_float(*rt) {
                *rt
            } else {
                TypeId::F32
            }
        } else {
            ty
        };
        Ok(Some(Operand::Const {
            ty: rty,
            value,
            untyped: untyped && !is_bool,
        }))
    }

    fn check_logical(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        op: BinOp,
        a: &ast::Expr,
        b: &ast::Expr,
        span: Span,
    ) -> Result<Operand> {
        let lhs = self.check_expr(f, scope, a, Some(TypeId::BOOL))?;
        if let Operand::Const {
            value, ..
        } = &lhs
        {
            let truth = match value {
                Value::Bool(v) => Some(*v),
                Value::Int(v) => Some(*v != 0),
                Value::Null => Some(false),
                _ => None,
            };
            if let Some(t) = truth {
                // Short-circuit at compile time.
                if (op == BinOp::And && !t) || (op == BinOp::Or && t) {
                    return Ok(Operand::bool(t));
                }
                let rhs = self.check_expr(f, scope, b, Some(TypeId::BOOL))?;
                if let Operand::Const {
                    value, ..
                } = &rhs
                {
                    let r = match value {
                        Value::Bool(v) => *v,
                        Value::Int(v) => *v != 0,
                        Value::Null => false,
                        _ => true,
                    };
                    return Ok(Operand::bool(r));
                }
                let rhs = self.settle_untyped(rhs, None);
                let (ty, v) = self.rvalue(f, rhs, span)?;
                let v = self.truthy(f, ty, v, span)?;
                return Ok(Operand::Value {
                    ty: TypeId::BOOL,
                    val: v,
                });
            }
        }
        let lhs = self.settle_untyped(lhs, None);
        let (lty, lv) = self.rvalue(f, lhs, span)?;
        let lc = self.truthy(f, lty, lv, span)?;
        let result = f.b.alloca(1, 1);
        f.b.store(Ty::I8, result, lc);
        let rhs_block = f.b.new_block();
        let done = f.b.new_block();
        if op == BinOp::And {
            f.b.branch(lc, rhs_block, done);
        } else {
            f.b.branch(lc, done, rhs_block);
        }
        f.b.switch_to(rhs_block);
        let rhs = self.check_expr(f, scope, b, Some(TypeId::BOOL))?;
        let rhs = self.settle_untyped(rhs, None);
        let (rty, rv) = self.rvalue(f, rhs, span)?;
        let rc = self.truthy(f, rty, rv, span)?;
        f.b.store(Ty::I8, result, rc);
        f.b.jump(done);
        f.b.switch_to(done);
        Ok(Operand::Value {
            ty: TypeId::BOOL,
            val: f.b.load(Ty::I8, result),
        })
    }

    fn string_compare(
        &mut self,
        f: &mut FnCtx,
        op: BinOp,
        x: ir::Val,
        y: ir::Val,
        span: Span,
    ) -> Result<Operand> {
        if !matches!(op, BinOp::Eq | BinOp::Ne) {
            return err(span, "strings can only be compared with == and !=");
        }
        let result = f.b.alloca(1, 1);
        let lc = f.b.load(Ty::I64, x);
        let rc = f.b.load(Ty::I64, y);
        let same_len = f.b.cmp(CmpOp::Eq, Ty::I64, lc, rc);
        f.b.store(Ty::I8, result, same_len);
        let cmp_block = f.b.new_block();
        let done = f.b.new_block();
        f.b.branch(same_len, cmp_block, done);
        f.b.switch_to(cmp_block);
        let xp = f.b.ptr_offset(x, 8);
        let xd = f.b.load(Ty::Ptr, xp);
        let yp = f.b.ptr_offset(y, 8);
        let yd = f.b.load(Ty::Ptr, yp);
        let r =
            f.b.intrinsic(ir::Intrinsic::Memcmp, vec![xd, yd, lc], &[Ty::I16]);
        let zero = f.b.iconst(Ty::I16, 0);
        let eq = f.b.cmp(CmpOp::Eq, Ty::I16, r[0], zero);
        f.b.store(Ty::I8, result, eq);
        f.b.jump(done);
        f.b.switch_to(done);
        let mut v = f.b.load(Ty::I8, result);
        if op == BinOp::Ne {
            let one = f.b.iconst(Ty::I8, 1);
            v = f.b.bin(ir::BinOp::Xor, Ty::I8, v, one);
        }
        Ok(Operand::Value {
            ty: TypeId::BOOL,
            val: v,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn check_ifx(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        cond: &ast::Expr,
        then_value: Option<&ast::Expr>,
        else_value: Option<&ast::Expr>,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        let cond_op = self.check_expr(
            f,
            scope,
            cond,
            then_value.map_or(expected, |_| Some(TypeId::BOOL)),
        )?;
        // Compile-time condition: only check the taken branch.
        if let Operand::Const {
            value, ..
        } = &cond_op
            && then_value.is_some()
        {
            let taken = match value {
                Value::Bool(b) => Some(*b),
                Value::Int(i) => Some(*i != 0),
                Value::Null => Some(false),
                _ => None,
            };
            if let Some(t) = taken {
                return if t {
                    self.check_expr(f, scope, then_value.unwrap(), expected)
                } else if let Some(else_value) = else_value {
                    self.check_expr(f, scope, else_value, expected)
                } else {
                    // No `else`: the zero value of the then-value's type.
                    let then_op = self.check_expr(f, scope, then_value.unwrap(), expected)?;
                    let ty = self.settle_untyped(then_op, None).ty();
                    let slot =
                        f.b.slot(self.size_of(ty, span)?.max(1), self.align_of(ty, span)?);
                    let addr = f.b.slot_addr(slot);
                    self.init_default(f, ty, addr, span)?;
                    let load = self.ir_ty(ty).map(|t| f.b.load(t, addr));
                    Ok(Operand::Value {
                        ty,
                        val: load.unwrap_or(addr),
                    })
                };
            }
        }
        let cond_op = self.settle_untyped(cond_op, None);
        let (cty, cv) = self.rvalue(f, cond_op, span)?;
        let c = self.truthy(f, cty, cv, span)?;
        let then_block = f.b.new_block();
        let else_block = f.b.new_block();
        let done = f.b.new_block();
        f.b.branch(c, then_block, else_block);
        f.b.switch_to(then_block);
        let implicit_then = match then_value {
            None => implicit_then_expr(cond),
            Some(_) => None,
        };
        let then_op = match (then_value, implicit_then) {
            (Some(e), _) | (None, Some(e)) => self.check_expr(f, scope, e, expected)?,
            (None, None) => Operand::Value {
                ty: cty,
                val: cv,
            },
        };
        let then_ty = match &then_op {
            Operand::Const {
                untyped: true, ..
            } => None,
            // An overload set takes the expected procedure type (picked by `convert`), or the
            // else-value's.
            Operand::Procs(p) if p.len() > 1 || expected.is_some() => None,
            Operand::Procs(p) => Some(self.proc_type(p[0], span)?),
            other => Some(other.ty()),
        };
        // A numeric then-value widens to an expected numeric type, so the else-value may be
        // wider (`ifx c then small_u8 else big_s32` into an `s32`).
        let numeric = |c: &Compiler, t: TypeId| {
            let r = c.types.repr(t);
            c.types.is_integer(r) || c.types.is_float(r)
        };
        let widen = match (then_ty, expected) {
            (Some(t), Some(e)) if t != e && numeric(self, t) && numeric(self, e) => {
                self.implicit_cost(t, false, e).map(|_| e)
            }
            _ => None,
        };
        let result_ty = widen.or(then_ty).or(expected);
        let then_end = f.b.current;
        // Check else first to learn its type when then is untyped.
        f.b.switch_to(else_block);
        let else_op = match else_value {
            Some(e) => Some(self.check_expr(f, scope, e, result_ty.or(expected))?),
            None => None,
        };
        // Without a target type, a narrower numeric then-value widens to the else-value's
        // type (`ifx hit then entry.score_s16 else evaluate()` is an `s64`).
        let else_wider = match (then_ty, expected, &else_op) {
            (Some(t), None, Some(op))
                if numeric(self, t)
                    && !matches!(
                        op,
                        Operand::Const {
                            untyped: true,
                            ..
                        }
                    )
                    && numeric(self, op.ty())
                    && op.ty() != t
                    && self.implicit_cost(op.ty(), false, t).is_none()
                    && self.implicit_cost(t, false, op.ty()).is_some() =>
            {
                Some(op.ty())
            }
            _ => None,
        };
        let ty = match (else_wider.or(result_ty), &else_op) {
            (Some(t), _) => t,
            (None, Some(Operand::Procs(p))) if p.len() == 1 => self.proc_type(p[0], span)?,
            (None, Some(op)) => self.settle_untyped(op.clone(), None).ty(),
            // `ifx c then 5`: an untyped then-value takes its default type.
            (None, None) => self.settle_untyped(then_op.clone(), None).ty(),
        };
        let size = self.size_of(ty, span)?;
        let align = self.align_of(ty, span)?;
        // The result slot must be allocated before both branches use it; slots are function-wide.
        let slot = f.b.slot(size.max(1), align);
        let addr = f.b.slot_addr(slot);
        match (else_op, else_value) {
            (Some(else_op), Some(else_value)) => {
                let else_converted = self.convert(f, else_op, ty, else_value.span)?;
                let (_, ev) = self.rvalue(f, else_converted, span)?;
                self.store_value(f, ty, addr, ev, span)?;
            }
            _ => self.init_default(f, ty, addr, span)?,
        }
        f.b.jump(done);
        f.b.switch_to(then_end);
        let then_converted = self.convert(f, then_op, ty, span)?;
        let (_, tv) = self.rvalue(f, then_converted, span)?;
        let addr = f.b.slot_addr(slot);
        self.store_value(f, ty, addr, tv, span)?;
        f.b.jump(done);
        f.b.switch_to(done);
        let addr = f.b.slot_addr(slot);
        match self.ir_ty(ty) {
            Some(t) => Ok(Operand::Value {
                ty,
                val: f.b.load(t, addr),
            }),
            None => Ok(Operand::Value {
                ty,
                val: addr,
            }),
        }
    }

    // -----------------------------------------------------------------------
    // Indexing
    // -----------------------------------------------------------------------

    fn check_index(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        base: &ast::Expr,
        index: &ast::Expr,
        span: Span,
    ) -> Result<Operand> {
        let base_op = self.check_expr(f, scope, base, None)?;
        let index_op = self.check_expr(f, scope, index, Some(TypeId::S64))?;
        self.index_operand(f, scope, base_op, index_op, index.span, span)
    }

    /// `base[index]` on checked operands.
    pub(super) fn index_operand(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        base_op: Operand,
        index_op: Operand,
        index_span: Span,
        span: Span,
    ) -> Result<Operand> {
        // Constant string indexing folds.
        if let (
            Operand::Const {
                value: Value::String(s),
                ..
            },
            Operand::Const {
                value: Value::Int(i),
                ..
            },
        ) = (&base_op, &index_op)
            && (*i as usize) < s.len()
        {
            return Ok(Operand::int(s[*i as usize] as i128, TypeId::U8));
        }
        let bty = base_op.ty();
        // `operator []` when one matches, else `operator *[]` dereferenced.
        let getter = self.try_index_operator_overload(f, scope, "[]", &base_op, &index_op, span);
        match getter {
            Ok(Some(result)) => return Ok(result),
            Ok(None) | Err(_) => {
                if let Some(result) =
                    self.try_pointer_index_operator(f, scope, &base_op, &index_op, span)?
                {
                    return Ok(result);
                }
                // A pointer indexes its memory when no operator for the pointee fits.
                if self.types.pointee(bty).is_none() {
                    getter?;
                }
            }
        }
        let index_op = self.settle_untyped(index_op, Some(TypeId::S64));
        let ity = index_op.ty();
        if !self.types.is_integer(ity)
            && !matches!(self.types.kind(self.types.repr(ity)), TypeKind::Enum(_))
        {
            return err(
                index_span,
                format!(
                    "array index must be an integer, found {}",
                    self.types.name(ity)
                ),
            );
        }
        let index_op = self.explicit_cast(
            f,
            index_op,
            TypeId::S64,
            ast::CastFlags {
                no_check: true,
                ..Default::default()
            },
            span,
        )?;
        let (_, idx) = self.rvalue(f, index_op, span)?;
        // Views, dynamic arrays and strings start with their count.
        let check = !f.no_abc && !f.type_only;
        if check {
            self.note_check_handler(span);
        }
        let (elem, data) = match self.types.kind(self.types.repr(bty)).clone() {
            TypeKind::Array {
                elem,
                kind: ArrayKind::Fixed(n),
            } => {
                let (_, addr) = self.address_of(f, base_op, span)?;
                if check {
                    let count = f.b.iconst(Ty::I64, n);
                    f.b.intrinsic(ir::Intrinsic::BoundsCheck, vec![idx, count], &[]);
                }
                (elem, addr)
            }
            TypeKind::Array {
                elem, ..
            } => {
                let (_, addr) = self.address_of(f, base_op, span)?;
                if check {
                    let count = f.b.load(Ty::I64, addr);
                    f.b.intrinsic(ir::Intrinsic::BoundsCheck, vec![idx, count], &[]);
                }
                let p = f.b.ptr_offset(addr, 8);
                (elem, f.b.load(Ty::Ptr, p))
            }
            TypeKind::String => {
                let (_, addr) = self.address_of(f, base_op, span)?;
                if check {
                    let count = f.b.load(Ty::I64, addr);
                    f.b.intrinsic(ir::Intrinsic::BoundsCheck, vec![idx, count], &[]);
                }
                let p = f.b.ptr_offset(addr, 8);
                (TypeId::U8, f.b.load(Ty::Ptr, p))
            }
            TypeKind::Pointer(elem) => {
                let (_, p) = self.rvalue(f, base_op, span)?;
                (elem, p)
            }
            _ => {
                return err(
                    span,
                    format!("cannot index a value of type {}", self.types.name(bty)),
                );
            }
        };
        let size = self.size_of(elem, span)?;
        let s = f.b.iconst(Ty::I64, size);
        let off = f.b.bin(ir::BinOp::Mul, Ty::I64, idx, s);
        Ok(Operand::Place {
            ty: elem,
            addr: f.b.ptr_add(data, off),
        })
    }

    /// `#location` / `#caller_location` as a `Source_Code_Location` constant.
    pub fn location_operand(&mut self, f: &mut FnCtx, span: Span) -> Result<Operand> {
        let ty = self.preload_type("Source_Code_Location", span)?;
        let file = self.sources.get(span.file);
        let (line, col) = file.line_col(span.start);
        let path = file.path.clone();
        let _ = f;
        let value = self.build_location_value(ty, &path, line as i64, col as i64, span)?;
        Ok(Operand::Const {
            ty,
            value,
            untyped: false,
        })
    }

    pub fn preload_type(&mut self, name: &str, span: Span) -> Result<TypeId> {
        let Some(preload) = self.preload else {
            return err(span, format!("`{name}` requires Preload"));
        };
        let ids = self.module_declarations(preload, Sym::intern(name))?;
        let Some(&id) = ids.first() else {
            return err(span, format!("Preload does not define `{name}`"));
        };
        match self.resolve_entity(id)? {
            Resolved::Const {
                value: Value::Type(t),
                ..
            } => Ok(t),
            _ => err(span, format!("Preload's `{name}` is not a type")),
        }
    }
}

fn num(v: &Value) -> f64 {
    match v {
        Value::Int(i) => *i as f64,
        Value::Float(f) => *f,
        _ => 0.0,
    }
}

pub fn wrap_int(v: i128, bits: u8, signed: bool) -> i128 {
    if bits >= 128 {
        return v;
    }
    let mask = (1i128 << bits) - 1;
    let x = v & mask;
    if signed && bits > 0 && (x >> (bits - 1)) & 1 == 1 {
        x - (1i128 << bits)
    } else {
        x
    }
}

fn cmp_op(op: BinOp, float: bool, signed: bool) -> CmpOp {
    match (op, float, signed) {
        (BinOp::Eq, true, _) => CmpOp::FEq,
        (BinOp::Ne, true, _) => CmpOp::FNe,
        (BinOp::Lt, true, _) => CmpOp::FLt,
        (BinOp::Le, true, _) => CmpOp::FLe,
        (BinOp::Gt, true, _) => CmpOp::FGt,
        (BinOp::Ge, true, _) => CmpOp::FGe,
        (BinOp::Eq, _, _) => CmpOp::Eq,
        (BinOp::Ne, _, _) => CmpOp::Ne,
        (BinOp::Lt, _, true) => CmpOp::SLt,
        (BinOp::Le, _, true) => CmpOp::SLe,
        (BinOp::Gt, _, true) => CmpOp::SGt,
        (BinOp::Ge, _, true) => CmpOp::SGe,
        (BinOp::Lt, _, false) => CmpOp::ULt,
        (BinOp::Le, _, false) => CmpOp::ULe,
        (BinOp::Gt, _, false) => CmpOp::UGt,
        _ => CmpOp::UGe,
    }
}

fn arith_op(op: BinOp, float: bool, signed: bool) -> Option<ir::BinOp> {
    use ir::BinOp as I;
    Some(match (op, float) {
        (BinOp::Add, true) => I::FAdd,
        (BinOp::Sub, true) => I::FSub,
        (BinOp::Mul, true) => I::FMul,
        (BinOp::Div, true) => I::FDiv,
        (_, true) => return None,
        (BinOp::Add, _) => I::Add,
        (BinOp::Sub, _) => I::Sub,
        (BinOp::Mul, _) => I::Mul,
        (BinOp::Div, _) => {
            if signed {
                I::SDiv
            } else {
                I::UDiv
            }
        }
        (BinOp::Rem, _) => {
            if signed {
                I::SRem
            } else {
                I::URem
            }
        }
        (BinOp::BitAnd, _) => I::And,
        (BinOp::BitOr, _) => I::Or,
        (BinOp::BitXor, _) => I::Xor,
        (BinOp::Shl, _) => I::Shl,
        (BinOp::Shr, _) => {
            if signed {
                I::AShr
            } else {
                I::LShr
            }
        }
        (BinOp::Rotl, _) => I::Rotl,
        (BinOp::Rotr, _) => I::Rotr,
        _ => return None,
    })
}

/// Whether a cast operand has no type without context: an inferred `.NAME`, an untyped `.{...}`
/// or `.[...]`, or operators and `ifx` built only from those (`cast(Flags) (.A | .B)`).
/// Everything else is typed bottom-up before the cast converts it {#cast.29}.
fn cast_operand_needs_target(e: &ast::Expr) -> bool {
    match &e.kind {
        E::InferredMember(_)
        | E::StructLit {
            ty: None, ..
        }
        | E::ArrayLit {
            ty: None, ..
        } => true,
        E::Unary(_, x) => cast_operand_needs_target(x),
        E::Binary(_, a, b) => cast_operand_needs_target(a) && cast_operand_needs_target(b),
        E::Ifx {
            then_value: Some(a),
            else_value: Some(b),
            ..
        } => cast_operand_needs_target(a) && cast_operand_needs_target(b),
        _ => false,
    }
}

/// A numeric literal, possibly signed or parenthesised arithmetic of literals (`-1`, `(2 * 3)`):
/// an expression that is an untyped constant whatever the context.
fn is_number_literal(e: &ast::Expr) -> bool {
    match &e.kind {
        E::Int(_) | E::Float(_) => true,
        E::Unary(ast::UnOp::Neg | ast::UnOp::Plus | ast::UnOp::BitNot, x) => is_number_literal(x),
        E::Binary(_, a, b) => is_number_literal(a) && is_number_literal(b),
        _ => false,
    }
}

/// Paths the program sees (`#file`, `#filepath`) use `/` on Windows too, so a metaprogram can put
/// them in a string literal or splice them into `#load "..."` without escaping backslashes.
pub(crate) fn forward_slashes(path: &str) -> String {
    if cfg!(windows) {
        path.replace('\\', "/")
    } else {
        path.to_string()
    }
}
