//! Built-in procedures (`size_of`, `type_of`, `is_constant`, ...) and procedure-type helpers.

use super::{
    ArrayKind, BuiltinProc, CallArg, Compiler, Diagnostic, E, FnCtx, Operand, PreludeType, ProcId,
    Rc, Result, ScopeId, ScopeKind, Span, Ty, TypeId, TypeKind, Value, ast, err, ir, procs,
};

impl Compiler {
    pub(super) fn check_builtin(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        b: BuiltinProc,
        args: &[ast::Arg],
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        let _ = expected;
        let Some(arg) = args.first() else {
            return err(span, "builtin needs an argument");
        };
        match b {
            BuiltinProc::SizeOf | BuiltinProc::AlignOf => {
                let op = match self.type_field_type(scope, &arg.value)? {
                    Some(t) => Operand::Type(t),
                    None => self.check_expr_no_emit(scope, &arg.value)?,
                };
                let ty = match op {
                    Operand::Type(t) => t,
                    Operand::Const {
                        value: Value::Type(t),
                        ..
                    } => t,
                    Operand::Value {
                        ty: TypeId::TYPE, ..
                    }
                    | Operand::Place {
                        ty: TypeId::TYPE, ..
                    } => {
                        let name = if b == BuiltinProc::SizeOf {
                            "size_of"
                        } else {
                            "align_of"
                        };
                        return Err(Box::new(
                            Diagnostic::error(
                                arg.value.span,
                                format!(
                                    "`{name}` needs a constant type, but this is a `Type` variable"
                                ),
                            )
                            .with_label("not a compile-time constant")
                            .with_help(
                                "a variable's value is only known at run time; use \
                                 `(cast(*Type_Info) t).runtime_size` (see `type_info(...)`) \
                                 to read a type's size from a variable",
                            ),
                        ));
                    }
                    other => other.ty(),
                };
                let v = if b == BuiltinProc::SizeOf {
                    self.size_of(ty, span)?
                } else {
                    self.align_of(ty, span)?
                };
                Ok(Operand::Const {
                    ty: TypeId::S64,
                    value: Value::Int(v as i128),
                    untyped: true,
                })
            }
            BuiltinProc::CodeOf => {
                // The code of a procedure (its header and body), else of the expression.
                let proc = match self.check_expr_no_emit(scope, &arg.value) {
                    Ok(Operand::Procs(p)) if p.len() == 1 => Some(p[0]),
                    Ok(Operand::Const {
                        value: Value::Proc(p),
                        ..
                    }) => Some(p),
                    _ => None,
                };
                let id = match proc {
                    Some(p) => {
                        let (lit, span, pscope) = (
                            self.proc(p).lit.clone(),
                            self.proc(p).span,
                            self.proc(p).scope,
                        );
                        let expr = ast::Expr {
                            kind: ast::ExprKind::Proc(lit),
                            span,
                        };
                        let id = self.add_code(Rc::new(ast::CodeBody::Expr(expr)), pscope);
                        self.pr.code_procs.insert(id.0 as usize, p);
                        id
                    }
                    None => self.add_code(Rc::new(ast::CodeBody::Expr(arg.value.clone())), scope),
                };
                Ok(Operand::Const {
                    ty: TypeId::CODE,
                    value: Value::Code(id),
                    untyped: false,
                })
            }
            BuiltinProc::TypeOf => {
                if let Some(t) = self.type_field_type(scope, &arg.value)? {
                    return Ok(Operand::Type(t));
                }
                // Only the type is needed, so an enclosing procedure's local is fine
                // (`#run f(type_of(local))` inside a body).
                if let Some(t) = self.outer_local_type(scope, &arg.value)? {
                    return Ok(Operand::Type(t));
                }
                let op = match self.check_expr_no_emit(scope, &arg.value) {
                    Ok(op) => op,
                    Err(e) => match self.enclosing_struct_field_type(scope, &arg.value)? {
                        Some(t) => return Ok(Operand::Type(t)),
                        None => return Err(e),
                    },
                };
                let ty = match op {
                    Operand::Type(_) | Operand::PolyStruct(_) => TypeId::TYPE,
                    Operand::Const {
                        ty,
                        value,
                        untyped: true,
                    } => self.ty.default_untyped(ty, &value),
                    Operand::Procs(p) if p.len() == 1 && self.proc(p[0]).is_poly => {
                        self.poly_proc_type(p[0])
                    }
                    Operand::Procs(p) if p.len() == 1 => self.proc_type(p[0], span)?,
                    other => other.ty(),
                };
                Ok(Operand::Type(ty))
            }
            BuiltinProc::TypeInfo => {
                let op = self.check_expr(f, scope, &arg.value, Some(TypeId::TYPE))?;
                if matches!(
                    op,
                    Operand::Value {
                        ty: TypeId::TYPE,
                        ..
                    } | Operand::Place {
                        ty: TypeId::TYPE,
                        ..
                    }
                ) {
                    // A runtime `Type` is already a `*Type_Info`.
                    let (_, v) = self.rvalue(f, op, span)?;
                    let info = self.preload_type(PreludeType::TypeInfo, span)?;
                    return Ok(Operand::Value {
                        ty: self.ty.types.pointer(info),
                        val: v,
                    });
                }
                // A generic struct named without arguments has the descriptor its instances
                // carry as `polymorph_source_struct`.
                if let Operand::PolyStruct(ps) = op
                    && !self.ty.poly_structs[ps.0 as usize]
                        .lit
                        .params
                        .iter()
                        .all(|p| p.default.is_some())
                {
                    let desc = self.preload_type(PreludeType::TypeInfoStruct, span)?;
                    let (key, name) = {
                        let p = &self.ty.poly_structs[ps.0 as usize];
                        (p.lit.id, p.name)
                    };
                    let global = self.generic_struct_info_global(key, name, desc, span)?;
                    let v = f.b.global_addr(global);
                    return Ok(Operand::Value {
                        ty: self.ty.types.pointer(desc),
                        val: v,
                    });
                }
                let ty = self.operand_as_type(op, arg.value.span)?;
                let info_ty = self.type_info_struct_type(ty, span)?;
                let ptr = self.ty.types.pointer(info_ty);
                let global = self.type_info_global(ty, span)?;
                let v = f.b.global_addr(global);
                Ok(Operand::Value {
                    ty: ptr,
                    val: v,
                })
            }
            BuiltinProc::IsConstant => {
                let op = self.check_expr_no_emit(scope, &arg.value)?;
                let constant = op.is_const()
                    || matches!(op, Operand::Procs(_))
                    || self.is_constant_pointer(scope, &arg.value, &op);
                Ok(Operand::bool(constant))
            }
            BuiltinProc::InitializerOf => {
                let ty = self.eval_type_in(f, scope, &arg.value)?;
                let proc = self.initializer_proc(ty, span)?;
                // A type that is all zeroes by default has no initializer (`#if initializer_of(T)`).
                if matches!(self.default_initializer(ty, span), Ok(None)) {
                    return Ok(Operand::Const {
                        ty: proc.1,
                        value: Value::Null,
                        untyped: false,
                    });
                }
                Ok(Operand::Value {
                    ty: proc.1,
                    val: f.b.func_addr(proc.0),
                })
            }
            BuiltinProc::OffsetOf => {
                // `offset_of(T, "member")` or `offset_of(T.member)`; the member may be a dotted
                // path (`"inner.x"`) and may come from a `using` member.
                let (ty, path) = if let Some(second) = args.get(1) {
                    let ty = self.eval_type_in(f, scope, &arg.value)?;
                    let name = match self.check_expr_no_emit(scope, &second.value)? {
                        Operand::Const {
                            value: Value::String(s),
                            ..
                        } => String::from_utf8_lossy(&s).into_owned(),
                        _ => {
                            return err(
                                second.value.span,
                                "the member name of `offset_of` must be a constant string",
                            );
                        }
                    };
                    (ty, name)
                } else {
                    let E::Member(base, field) = &arg.value.kind else {
                        return err(
                            arg.value.span,
                            "`offset_of` takes `(Type, \"member\")` or `Type.member`",
                        );
                    };
                    let ty = self.eval_type_in(f, scope, base)?;
                    (ty, field.name.as_str().to_string())
                };
                let mut offset = 0u64;
                let mut cur = ty;
                for part in path.split('.') {
                    if let Some((o, t)) = self.member_offset(cur, part, span)? {
                        offset += o;
                        cur = t;
                    } else {
                        let tn = self.ty.types.name(cur);
                        return err(
                            span,
                            format!("`{tn}` has no member `{part}` at a fixed offset"),
                        );
                    }
                }
                Ok(Operand::Const {
                    ty: TypeId::S64,
                    value: Value::Int(offset as i128),
                    untyped: true,
                })
            }
            BuiltinProc::IsValueType => {
                let ty = self.eval_type_in(f, scope, &arg.value)?;
                let v = self.is_value_type(ty, span)?;
                Ok(Operand::bool(v))
            }
        }
    }

    /// Whether copying a value of `ty` byte for byte copies all of it: no pointers, strings,
    /// array views, resizable arrays, procedures, `Any`, `Type` or `Code` inside.
    pub(super) fn is_value_type(&mut self, ty: TypeId, span: Span) -> Result<bool> {
        let ty = self.ty.types.repr(ty);
        Ok(match self.ty.types.kind(ty).clone() {
            TypeKind::Void
            | TypeKind::Bool
            | TypeKind::Int {
                ..
            }
            | TypeKind::Float {
                ..
            }
            | TypeKind::WideFloat(_) => true,
            TypeKind::Array {
                elem,
                kind: ArrayKind::Fixed(_),
            } => self.is_value_type(elem, span)?,
            TypeKind::Struct(s) => {
                self.layout_struct(s, span)?;
                let fields: Vec<TypeId> = self
                    .ty
                    .types
                    .struct_info(s)
                    .fields
                    .iter()
                    .map(|f| f.ty)
                    .collect();
                for t in fields {
                    if !self.is_value_type(t, span)? {
                        return Ok(false);
                    }
                }
                true
            }
            _ => false,
        })
    }

    /// Check an expression only for its type; any IR it emits is discarded.
    /// Argument of `type_of`/`size_of`: `Struct.field` names the field's type
    /// there, without a value.
    pub(super) fn type_field_type(
        &mut self,
        scope: ScopeId,
        expr: &ast::Expr,
    ) -> Result<Option<TypeId>> {
        if let E::Member(base, field) = &expr.kind
            && let Ok(Operand::Type(t)) = self.check_expr_no_emit(scope, base)
            && self
                .ty
                .types
                .as_struct(self.ty.types.repr_struct(t))
                .is_some()
            && self.struct_constant(t, field.name)?.is_none()
        {
            if let Some(ty) = self.pending_struct_field_type(t, field.name, expr.span)? {
                return Ok(Some(ty));
            }
            if let Some((_, fty)) = self.find_member(t, field.name, expr.span)? {
                return Ok(Some(fty));
            }
        }
        Ok(None)
    }

    /// `type_of(x)` inside a procedure nested in a struct: `x` is a field of that struct, which
    /// has a type though not a value.
    pub(super) fn enclosing_struct_field_type(
        &mut self,
        scope: ScopeId,
        expr: &ast::Expr,
    ) -> Result<Option<TypeId>> {
        if let E::Ident(name) = &expr.kind {
            if let Some(ty) = self.struct_field_type(scope, *name, expr.span)? {
                return Ok(Some(ty));
            }
            let mut s = Some(scope);
            while let Some(sid) = s {
                if let ScopeKind::Struct(t) = self.scope(sid).kind
                    && self.ty.types.as_struct(t).is_none_or(|s| {
                        self.ty.types.struct_info(s).layout != crate::types::LayoutState::InProgress
                    })
                    && let Some((_, fty)) = self.find_member(t, *name, expr.span)?
                {
                    return Ok(Some(fty));
                }
                s = self.scope(sid).parent;
            }
        }
        Ok(None)
    }

    /// A throwaway function context for checking code whose output is discarded.
    pub(in crate::sema) fn scratch_ctx(&self, scope: ScopeId) -> FnCtx {
        let file = self.scope_file(scope);
        let mut scratch = FnCtx::new(
            "scratch".into(),
            ir::Sig {
                params: vec![Ty::Ptr],
                returns: vec![],
                conv: ir::Conv::Jai,
                c_varargs: false,
                c_fixed: 0,
                c_abi: None,
            },
            file,
        );
        scratch.set_context(Some(scratch.b.param(0)));
        scratch.type_only = true;
        scratch
    }

    pub fn check_expr_no_emit(&mut self, scope: ScopeId, expr: &ast::Expr) -> Result<Operand> {
        let mut scratch = self.scratch_ctx(scope);
        self.check_expr(&mut scratch, scope, expr, None)
    }

    /// Instantiate polymorphic `proc` so that it has procedure type `target`, as when a
    /// polymorphic procedure is passed to a parameter of procedure type: the target's
    /// parameter types play the role of call arguments.
    pub fn instantiate_for_proc_type(
        &mut self,
        proc: ProcId,
        target: TypeId,
        span: Span,
    ) -> Option<ProcId> {
        let TypeKind::Proc(info) = self.ty.types.kind(target).clone() else {
            return None;
        };
        let inst = self.proc_for_param_types(proc, &info.params, span)?;
        let ty = self.proc_type(inst, span).ok()?;
        (ty == target || self.ty.proc_types_compatible(ty, target)).then_some(inst)
    }

    /// The type of a polymorphic procedure for compile-time inspection (`type_of(poly)`):
    /// a parameter or result that depends on a type variable is shown as `$` (inside a
    /// pointer, `*$`); the rest keeps its type. Baked parameters keep their position.
    pub(in crate::sema) fn poly_proc_type(&mut self, proc: ProcId) -> TypeId {
        let header = self.proc(proc).lit.header.clone();
        let scope = self.proc(proc).scope;
        self.poly_header_type(scope, &header)
    }

    /// The type of a header that has type variables (`type_of(poly)`, or a `#type` alias with
    /// `$T` in it), with the parts that depend on them shown as `$`.
    pub(in crate::sema) fn poly_header_type(
        &mut self,
        scope: ScopeId,
        header: &ast::ProcHeader,
    ) -> TypeId {
        let mut params = Vec::new();
        let mut variadic = false;
        for p in &header.params {
            let mut ty = match (&p.ty, &p.default) {
                (Some(t), _) => self.poly_shape_type(scope, t),
                (None, Some(d)) => {
                    self.check_expr_no_emit(scope, d)
                        .map_or(TypeId::POLY_PARAM, |op| match op {
                            Operand::Const {
                                ty,
                                value,
                                untyped: true,
                            } => self.ty.default_untyped(ty, &value),
                            other => other.ty(),
                        })
                }
                (None, None) => TypeId::POLY_PARAM,
            };
            if p.variadic {
                variadic = true;
                ty = self.ty.types.array(ty, ArrayKind::View);
            }
            params.push(ty);
        }
        let returns: Vec<TypeId> = header
            .returns
            .iter()
            .filter_map(|r| match &r.ty {
                Some(t) => Some(self.poly_shape_type(scope, t)),
                None => r.default.as_ref().map(|_| TypeId::POLY_PARAM),
            })
            .filter(|&t| t != TypeId::VOID)
            .collect();
        self.ty
            .types
            .intern(TypeKind::Proc(Rc::new(crate::types::ProcType {
                params,
                returns,
                variadic,
                c_varargs: false,
                c_call: header.flags.c_call(),
                no_context: false,
                non_pod_return: false,
            })))
    }

    /// The type of a procedure whose signature has not been resolved, from the header's shape
    /// alone (every parameter and result is `$`): evaluating the real types here could lay out
    /// or run code too early. Type info lists such a procedure by name with this type.
    pub(in crate::sema) fn header_shape_type(&mut self, proc: ProcId) -> TypeId {
        let header = self.proc(proc).lit.header.clone();
        let mut variadic = false;
        let params = header
            .params
            .iter()
            .map(|p| {
                if p.variadic {
                    variadic = true;
                    self.ty.types.array(TypeId::POLY_PARAM, ArrayKind::View)
                } else {
                    TypeId::POLY_PARAM
                }
            })
            .collect();
        let returns = header.returns.iter().map(|_| TypeId::POLY_PARAM).collect();
        self.ty
            .types
            .intern(TypeKind::Proc(Rc::new(crate::types::ProcType {
                params,
                returns,
                variadic,
                c_varargs: false,
                c_call: header.flags.c_call(),
                no_context: false,
                non_pod_return: false,
            })))
    }

    /// A type expression of a polymorphic header as far as it is known: the parts that depend
    /// on a type variable become `$`.
    pub(super) fn poly_shape_type(&mut self, scope: ScopeId, ty: &ast::Expr) -> TypeId {
        if !procs::has_poly(ty) {
            return self.eval_type(scope, ty).unwrap_or(TypeId::POLY_PARAM);
        }
        match &ty.kind {
            E::Unary(ast::UnOp::Star, inner) => {
                let inner = self.poly_shape_type(scope, inner);
                self.ty.types.pointer(inner)
            }
            E::ArrayType {
                size: ast::ArraySize::Fixed(_),
                elem,
            } => {
                let elem = self.poly_shape_type(scope, elem);
                self.ty.types.array(elem, ArrayKind::Fixed(0))
            }
            _ => TypeId::POLY_PARAM,
        }
    }

    /// The instance of `proc` (or `proc` itself) callable with arguments of the given types.
    pub fn proc_for_param_types(
        &mut self,
        proc: ProcId,
        params: &[TypeId],
        span: Span,
    ) -> Option<ProcId> {
        let scope = self.proc(proc).scope;
        let scratch = self.scratch_ctx(scope);
        let placeholder = scratch.b.param(0);
        let args: Vec<CallArg> = params
            .iter()
            .map(|&ty| CallArg {
                name: None,
                spread: false,
                expr: None,
                op: Some(Operand::Value {
                    ty,
                    val: placeholder,
                }),
                span,
                scope,
            })
            .collect();
        Some(self.match_candidate(proc, &args, span).ok()?.proc)
    }

    pub fn check_procedure_of_call(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        call: &ast::Expr,
    ) -> Result<Operand> {
        let E::Call {
            callee,
            args,
            ..
        } = &call.kind
        else {
            return err(call.span, "#procedure_of_call needs a call expression");
        };
        let callee_op = self.check_expr(f, scope, callee, None)?;
        let Operand::Procs(procs) = callee_op else {
            return err(call.span, "#procedure_of_call needs a procedure call");
        };
        // Only the argument types matter: runtime locals are fine in a constant.
        let mut scratch = self.scratch_ctx(scope);
        let call_args = self.precheck_args(&mut scratch, scope, args)?;
        for &p in &procs {
            if let Ok(c) = self.match_candidate(p, &call_args, call.span) {
                return Ok(Operand::Procs(vec![c.proc]));
            }
        }
        err(call.span, "no procedure matches this call")
    }
}
