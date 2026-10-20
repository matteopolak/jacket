//! Expression checking and lowering.
mod idents;
mod ifx;
mod indexing;
mod unary;

use super::binary::Binary;
use super::lower::{FnCtx, Operand};
use super::{Compiler, Rc, Result, ScopeId, Sym, TypeId, Value, ast, err, ir};
use crate::ast::{BinOp, ExprKind as E};
use crate::ir::Ty;
use crate::types::{ArrayKind, TypeKind};

/// An expression with no type of its own, which takes the type of context: the other branch of an
/// `ifx`, the other operand of an operator, the parameter of a candidate overload or the target of
/// a cast. `.NAME`, `.{...}` and `.[...]` are these; everything else is typed from its own parts.
pub(super) fn takes_context_type(e: &ast::Expr) -> bool {
    matches!(
        e.kind,
        E::InferredMember(_)
            | E::StructLit {
                ty: None,
                ..
            }
            | E::ArrayLit {
                ty: None,
                ..
            }
    )
}

/// Whether `e` waits for its sibling: it takes the sibling's type from context while the sibling
/// is typed on its own, so the sibling is checked first and `e` is checked with its type.
pub(super) fn takes_sibling_type(e: &ast::Expr, sibling: &ast::Expr) -> bool {
    takes_context_type(e) && !takes_context_type(sibling)
}

/// Where an expected type comes from. A target is the type the value is for (a declaration, a
/// parameter, a cast); a hint is the type of a sibling the value is combined with, such as the
/// other operand of a binary operator. Only a target can reject a value: `c - (ifx up then 32
/// else 0)` with a `u16` `c` takes its untyped arms as `u16`, where `x: u16 = ifx up then 32
/// else 0` makes them `s64` and is an error.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Want {
    Target,
    Hint,
}

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
        self.check_expr_for(f, scope, expr, (expected, Want::Target))
    }

    /// `expr` as a sibling of another expression, whose type is `hint`.
    pub(super) fn check_expr_hinted(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        expr: &ast::Expr,
        hint: Option<TypeId>,
    ) -> Result<Operand> {
        self.check_expr_for(f, scope, expr, (hint, Want::Hint))
    }

    fn check_expr_for(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        expr: &ast::Expr,
        (expected, want): (Option<TypeId>, Want),
    ) -> Result<Operand> {
        self.ed.sketch_tried(expr.span);
        let mut result = self.check_expr_kind(f, scope, expr, (expected, want));
        self.ed.expr_checked(expr.span, result.is_ok());
        if result.is_err()
            && self.ed.recovers_exprs()
            && let Some(op) = self.ide_sketch_recover(f, scope, expr)
        {
            result = Ok(op);
        }
        if !f.type_only
            && !self.ed.sketching()
            && let Some(procedure) = f.proc
            && let Ok(op) = &result
            && op.ty() != TypeId::COMPILE_TIME
        {
            let ty = op.ty();
            self.meta
                .export
                .expression_types
                .entry(procedure)
                .or_default()
                .record(expr.span, ty);
        }
        if self.ed.is_live()
            && let Ok(op) = &result
        {
            self.ide_note_expr(expr, op);
            if self.ed.sketching() {
                self.ide_note_sketch_type(expr.span, op);
            }
        }
        result
    }

    fn check_expr_kind(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        expr: &ast::Expr,
        (expected, want): (Option<TypeId>, Want),
    ) -> Result<Operand> {
        match &expr.kind {
            E::Ident(..)
            | E::PolyVar {
                ..
            }
            | E::PolyRestricted {
                ..
            }
            | E::Int(..)
            | E::Float(..)
            | E::Char(..)
            | E::Str(..)
            | E::Bool(..)
            | E::Null
            | E::Uninit
            | E::Context => self.check_literal_expr(f, scope, expr, expected),
            E::Binary(..)
            | E::Unary(..)
            | E::Call {
                ..
            }
            | E::Member(..)
            | E::InferredMember(..)
            | E::Index(..)
            | E::Cast {
                ..
            }
            | E::Ifx {
                ..
            }
            | E::StructLit {
                ..
            }
            | E::ArrayLit {
                ..
            }
            | E::Lambda {
                ..
            }
            | E::Block(..) => self.check_operator_expr(f, scope, expr, (expected, want)),
            E::ArrayType {
                ..
            }
            | E::ProcType(..)
            | E::Proc(..)
            | E::Struct(..)
            | E::Enum(..)
            | E::TypeDirective {
                ..
            } => self.check_type_expr(f, scope, expr, expected),
            E::Run {
                ..
            }
            | E::Code(..)
            | E::Insert(..)
            | E::Location(..)
            | E::CallerLocation
            | E::File
            | E::Filepath
            | E::Line
            | E::ProcedureName(..)
            | E::This
            | E::CompileTime
            | E::Exists(..)
            | E::Bytes(..)
            | E::Backtick(..)
            | E::Bake {
                ..
            }
            | E::ProcedureOfCall(..)
            | E::CallerCode
            | E::UnknownDirective(..)
            | E::Asm(..) => self.check_directive_expr(f, scope, expr, expected),
        }
    }

    fn check_literal_expr(
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
                let v = v.get() as i128;
                match expected.map(|t| self.ty.types.repr(t)) {
                    Some(t) if self.ty.types.is_float(t) => Ok(Operand::Const {
                        ty: t,
                        value: Value::float(v as f64),
                        untyped: true,
                    }),
                    _ => Ok(Operand::untyped_int(v)),
                }
            }
            E::Float(v) => Ok(Operand::Const {
                ty: self.src.float_literal_type(*v, expr.span),
                value: Value::Float(
                    *v,
                    crate::lexer::float_literal_precision(self.src.sources.snippet(span), *v),
                ),
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
                let Some(ctx) = f.current_context() else {
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
            _ => unreachable!("dispatched only for this group's operations"),
        }
    }

    fn check_operator_expr(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        expr: &ast::Expr,
        (expected, want): (Option<TypeId>, Want),
    ) -> Result<Operand> {
        let span = expr.span;
        match &expr.kind {
            E::Binary(op, a, b) => {
                let node = Binary {
                    op: *op,
                    a,
                    b,
                };
                self.check_binary(f, scope, node, expected, span)
            }
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
                if !self.pr.const_macro_params.is_empty()
                    && member.name.as_str() == "count"
                    && let E::Ident(name) = &base.kind
                    && let Some(&id) = self.lookup(scope, *name)?.first()
                    && let Some((Value::String(s), _)) = self.pr.const_macro_params.get(&id)
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
                    && !self.names_field(self.ty.types.pointee(ty).unwrap_or(ty), member.name)?
                    && let Ok(op) = self.member_access(
                        f,
                        scope,
                        Operand::Type(self.ty.types.pointee(ty).unwrap_or(ty)),
                        member.name,
                        member.span,
                    )
                {
                    return Ok(op);
                }
                // A fixed array's count is part of its type, so `variable.count` is constant
                // although the variable is not: naming it reads no value.
                if f.constant_only.is_some()
                    && member.name.as_str() == "count"
                    && names_variable_path(base)
                {
                    let saved = f.constant_only.take();
                    let named = self.check_expr(f, scope, base, None);
                    f.constant_only = saved;
                    if let Ok(op) = named
                        && let TypeKind::Array {
                            kind: ArrayKind::Fixed(n),
                            ..
                        } = self.ty.types.kind(self.ty.types.repr(op.ty()))
                    {
                        return Ok(Operand::Const {
                            ty: TypeId::S64,
                            value: Value::Int(*n as i128),
                            untyped: true,
                        });
                    }
                }
                let base_op = match self.check_expr(f, scope, base, None) {
                    Ok(op) => op,
                    Err(e) => {
                        // A constant whose `#run` failed still has the type the run returns, so
                        // its members are colored and hovered as fields. Nothing is lowered: the
                        // expression fails either way.
                        if self.ed.is_live()
                            && let Some(ty) = self.failed_run_type(scope, base, 8)
                        {
                            let addr = f.b.iconst(Ty::Ptr, 0);
                            let stand_in = Operand::Place {
                                ty,
                                addr,
                            };
                            self.ide_note_receiver(member.span, &stand_in);
                            if let Ok(op) =
                                self.member_access(f, scope, stand_in, member.name, member.span)
                            {
                                self.ide_note_member_use(member.span, ty, member.name);
                                self.ide_note_member_ref(member.span, member.name, op.ty());
                            }
                        }
                        return Err(e);
                    }
                };
                if self.ed.is_live() {
                    self.ide_note_receiver(member.span, &base_op);
                }
                let note = self.ed.is_live().then(|| match &base_op {
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
                if self.ed.is_live() {
                    self.ide_note_cast(span, target, &op);
                }
                let op = if flags.no_check || flags.truncate || flags.force {
                    op
                } else {
                    self.check_constant_cast(&op, target, value.span)?;
                    self.emit_cast_check(f, op, target, span)?
                };
                let result = self.explicit_cast(f, op, target, *flags, span)?;
                let parameters = if matches!(self.ty.types.kind(target), TypeKind::Proc(_)) {
                    self.callable_expression_parameters(scope, ty.as_deref())?
                } else {
                    None
                };
                Ok(result.with_callable(parameters))
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
                    (then_value.as_deref(), Some(e)),
                    (expected, want),
                    span,
                ),
                // `ifx c then a` / `ifx c`: the value, or zero when the condition is false.
                None => self.check_ifx(
                    f,
                    scope,
                    cond,
                    (then_value.as_deref(), None),
                    (expected, want),
                    span,
                ),
            },
            E::StructLit {
                ty,
                fields,
            } => self.check_struct_literal(f, scope, (ty.as_deref(), fields), expected, span),
            E::ArrayLit {
                ty,
                elems,
            } => self.check_array_literal(f, scope, (ty.as_deref(), elems), expected, span),
            E::Lambda {
                header,
                body,
            } => self.check_lambda(scope, header, body, expected, span),
            E::Block(block) => self.check_block_value(f, scope, block, expected, span),
            _ => unreachable!("dispatched only for this group's operations"),
        }
    }

    fn check_type_expr(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        expr: &ast::Expr,
        _expected: Option<TypeId>,
    ) -> Result<Operand> {
        let span = expr.span;
        match &expr.kind {
            E::ArrayType {
                size,
                elem,
            } => {
                let elem = self.eval_type_in(f, scope, elem)?;
                let kind = match size {
                    ast::ArraySize::View => ArrayKind::View,
                    ast::ArraySize::Resizable => ArrayKind::Resizable,
                    ast::ArraySize::Fixed(size_expr) => {
                        let n = self.check_expr(f, scope, size_expr, Some(TypeId::S64))?;
                        let count = match &n {
                            Operand::Const {
                                value: Value::Int(v),
                                ..
                            } => Some(*v),
                            // An element of a constant array, or another constant that only
                            // evaluating the expression shows.
                            _ => match self.const_value_of_type(scope, size_expr, TypeId::S64) {
                                Ok(Value::Int(n)) => Some(n),
                                _ => None,
                            },
                        };
                        match count {
                            Some(n) if n >= 0 => ArrayKind::Fixed(n as u64),
                            _ => {
                                return err(
                                    span,
                                    format!(
                                        "array size must be a non-negative constant, found {}",
                                        self.describe(&n)
                                    ),
                                );
                            }
                        }
                    }
                };
                Ok(Operand::Type(self.ty.types.array(elem, kind)))
            }
            E::ProcType(header) => Ok(Operand::Type(self.proc_type_from_header(f, scope, header)?)),
            E::Proc(lit) => {
                let name = Sym::intern("anonymous_procedure");
                let key = lit.header.id;
                let proc = if let Some(&p) = self.pr.anonymous_procs.get(&(key, scope)) {
                    p
                } else {
                    let p = self.new_proc(name, lit.clone(), scope, span);
                    self.pr.anonymous_procs.insert((key, scope), p);
                    p
                };
                Ok(Operand::Procs(vec![proc]))
            }
            E::Struct(lit) => {
                if let Some(&t) = self.ty.anonymous.get(&(lit.id, scope)) {
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
                self.ty.anonymous.insert((lit.id, scope), t);
                Ok(Operand::Type(t))
            }
            E::Enum(lit) => {
                if let Some(&t) = self.ty.anonymous.get(&(lit.id, scope)) {
                    return Ok(Operand::Type(t));
                }
                let t = self.new_enum_type(Sym::intern("enum"), lit, scope)?;
                self.ty.anonymous.insert((lit.id, scope), t);
                Ok(Operand::Type(t))
            }
            E::TypeDirective {
                modifier,
                ty,
            } => {
                let base = self.eval_type_in(f, scope, ty)?;
                if modifier == &ast::TypeModifier::Plain {
                    Ok(Operand::Type(base))
                } else {
                    let isa = *modifier == ast::TypeModifier::Isa;
                    let t = self.ty.types.new_distinct(crate::types::DistinctInfo {
                        name: Sym::intern("distinct"),
                        base,
                        isa,
                    });
                    Ok(Operand::Type(t))
                }
            }
            _ => unreachable!("dispatched only for this group's operations"),
        }
    }

    fn check_directive_expr(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        expr: &ast::Expr,
        expected: Option<TypeId>,
    ) -> Result<Operand> {
        let span = expr.span;
        match &expr.kind {
            E::Run {
                body,
                flags,
            } => self.check_run(scope, body, flags, expected, span),
            E::Code(body) => {
                let id = self.add_code(body.clone(), scope);
                Ok(Operand::Const {
                    ty: TypeId::CODE,
                    value: Value::Code(id),
                    untyped: false,
                })
            }
            E::Insert(boxed) => {
                let crate::ast::InsertExpr {
                    value,
                    flags,
                    scope: target,
                    ..
                } = &**boxed;
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
                let path = forward_slashes(&self.src.sources.get(span.file).path.to_string());
                Ok(Operand::Const {
                    ty: TypeId::STRING,
                    value: Value::String(path.as_bytes().into()),
                    untyped: false,
                })
            }
            E::Filepath => {
                let path = self.src.sources.get(span.file).path.to_path();
                let dir = path
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
                let (line, _) = self.src.sources.get(span.file).line_col(span.start);
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
                            E::Int(v) => Some(v.get()),
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
                let Some((call, caller_scope)) = self.pr.calls_in_flight.last().cloned() else {
                    return err(span, "#caller_code is only valid as a parameter default");
                };
                let id = self.add_code(Rc::new(ast::CodeBody::Expr((*call).clone())), caller_scope);
                Ok(Operand::Const {
                    ty: TypeId::CODE,
                    value: Value::Code(id),
                    untyped: false,
                })
            }
            E::UnknownDirective(d) if d.kind == ast::DirectiveKind::Context => {
                Ok(Operand::Type(self.context_type(span)?))
            }
            E::UnknownDirective(d) if d.kind == ast::DirectiveKind::JaicType => {
                Ok(Operand::Type(self.jaic_type(d.operand.as_ref(), span)?))
            }
            E::UnknownDirective(d) => err(
                d.name.span,
                format!(
                    "directive `#{}` is not supported in this position",
                    d.name.name
                ),
            ),
            E::Asm(block) => self.check_asm(f, scope, block),
            _ => unreachable!("dispatched only for this group's operations"),
        }
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

/// Whether a cast operand has no type without context: an inferred `.NAME`, an untyped `.{...}`
/// or `.[...]`, or operators and `ifx` built only from those (`cast(Flags) (.A | .B)`).
/// Everything else is typed bottom-up before the cast converts it {#cast.29}.
fn cast_operand_needs_target(e: &ast::Expr) -> bool {
    match &e.kind {
        _ if takes_context_type(e) => true,
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

/// Paths the program sees (`#file`, `#filepath`) use `/` on Windows too, so a metaprogram can put
/// them in a string literal or splice them into `#load "..."` without escaping backslashes.
pub(crate) fn forward_slashes(path: &str) -> String {
    if cfg!(windows) {
        path.replace('\\', "/")
    } else {
        path.to_string()
    }
}

/// `name` or `name.member.member`: a path that only names storage.
fn names_variable_path(expr: &ast::Expr) -> bool {
    match &expr.kind {
        E::Ident(_) => true,
        E::Member(base, _) => names_variable_path(base),
        _ => false,
    }
}

/// The types of the bootstrap prelude that the compiler builds values of or checks against,
/// named by the prelude's own spelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreludeType {
    Allocator,
    ForFlags,
    Reg,
    SourceCodeLocation,
    StackTraceNode,
    StackTraceProcedureInfo,
    TypeInfo,
    TypeInfoArray,
    TypeInfoEnum,
    TypeInfoFloat,
    TypeInfoInteger,
    TypeInfoPointer,
    TypeInfoProcedure,
    TypeInfoString,
    TypeInfoStruct,
    TypeInfoStructMember,
    TypeInfoTaggedUnionBinding,
    TypeInfoVariant,
}

impl PreludeType {
    pub const ALL: &'static [PreludeType] = &[
        PreludeType::Allocator,
        PreludeType::ForFlags,
        PreludeType::Reg,
        PreludeType::SourceCodeLocation,
        PreludeType::StackTraceNode,
        PreludeType::StackTraceProcedureInfo,
        PreludeType::TypeInfo,
        PreludeType::TypeInfoArray,
        PreludeType::TypeInfoEnum,
        PreludeType::TypeInfoFloat,
        PreludeType::TypeInfoInteger,
        PreludeType::TypeInfoPointer,
        PreludeType::TypeInfoProcedure,
        PreludeType::TypeInfoString,
        PreludeType::TypeInfoStruct,
        PreludeType::TypeInfoStructMember,
        PreludeType::TypeInfoTaggedUnionBinding,
        PreludeType::TypeInfoVariant,
    ];

    /// The name the prelude declares the type under.
    pub const fn name(self) -> &'static str {
        match self {
            PreludeType::Allocator => "Allocator",
            PreludeType::ForFlags => "For_Flags",
            PreludeType::Reg => "__reg",
            PreludeType::SourceCodeLocation => "Source_Code_Location",
            PreludeType::StackTraceNode => "Stack_Trace_Node",
            PreludeType::StackTraceProcedureInfo => "Stack_Trace_Procedure_Info",
            PreludeType::TypeInfo => "Type_Info",
            PreludeType::TypeInfoArray => "Type_Info_Array",
            PreludeType::TypeInfoEnum => "Type_Info_Enum",
            PreludeType::TypeInfoFloat => "Type_Info_Float",
            PreludeType::TypeInfoInteger => "Type_Info_Integer",
            PreludeType::TypeInfoPointer => "Type_Info_Pointer",
            PreludeType::TypeInfoProcedure => "Type_Info_Procedure",
            PreludeType::TypeInfoString => "Type_Info_String",
            PreludeType::TypeInfoStruct => "Type_Info_Struct",
            PreludeType::TypeInfoStructMember => "Type_Info_Struct_Member",
            PreludeType::TypeInfoTaggedUnionBinding => "Type_Info_Tagged_Union_Binding",
            PreludeType::TypeInfoVariant => "Type_Info_Variant",
        }
    }
}

#[cfg(test)]
mod prelude_type_tests {
    use super::PreludeType;

    #[test]
    fn every_prelude_type_is_declared_in_the_prelude() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib/prelude");
        let mut text = String::new();
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "jai") {
                text.push_str(&std::fs::read_to_string(path).unwrap());
                text.push('\n');
            }
        }
        for ty in PreludeType::ALL {
            let declared = text.lines().any(|line| {
                line.trim_start()
                    .strip_prefix(ty.name())
                    .is_some_and(|rest| rest.trim_start().starts_with("::"))
            });
            assert!(declared, "{} is not declared in stdlib/prelude", ty.name());
        }
    }
}
