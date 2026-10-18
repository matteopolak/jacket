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
        if let Some(ide) = self.ide.as_mut()
            && ide.sketching
        {
            ide.sketch_tried.insert(expr.span);
        }
        let mut result = self.check_expr_kind(f, scope, expr, expected);
        if result.is_err()
            && self.sketching()
            && let Some(op) = self.ide_sketch_recover(f, scope, expr)
        {
            result = Ok(op);
        }
        if !f.type_only
            && !self.sketching()
            && let Some(procedure) = f.proc
            && let Ok(op) = &result
            && op.ty() != TypeId::COMPILE_TIME
        {
            let ty = op.ty();
            self.export
                .expression_types
                .entry(procedure)
                .or_default()
                .entry(expr.span)
                .and_modify(|known| {
                    if *known != Some(ty) {
                        *known = None;
                    }
                })
                .or_insert(Some(ty));
        }
        if self.ide.is_some()
            && let Ok(op) = &result
        {
            self.ide_note_expr(expr, op);
            if self.sketching() {
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
                match expected.map(|t| self.types.repr(t)) {
                    Some(t) if self.types.is_float(t) => Ok(Operand::Const {
                        ty: t,
                        value: Value::float(v as f64),
                        untyped: true,
                    }),
                    _ => Ok(Operand::untyped_int(v)),
                }
            }
            E::Float(v) => Ok(Operand::Const {
                ty: self.float_literal_type(*v, expr.span),
                value: Value::Float(
                    *v,
                    crate::lexer::float_literal_precision(self.sources.snippet(span), *v),
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
                        } = self.types.kind(self.types.repr(op.ty()))
                    {
                        return Ok(Operand::Const {
                            ty: TypeId::S64,
                            value: Value::Int(*n as i128),
                            untyped: true,
                        });
                    }
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
                let result = self.explicit_cast(f, op, target, *flags, span)?;
                let parameters = if matches!(self.types.kind(target), TypeKind::Proc(_)) {
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
                    expected,
                    span,
                ),
                // `ifx c then a` / `ifx c`: the value, or zero when the condition is false.
                None => self.check_ifx(
                    f,
                    scope,
                    cond,
                    (then_value.as_deref(), None),
                    expected,
                    span,
                ),
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
            E::UnknownDirective(d) if d.name.name.as_str() == "Context" => {
                Ok(Operand::Type(self.context_type(span)?))
            }
            E::UnknownDirective(d) if d.name.name.as_str() == "jaic_type" => {
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
        let depth = f.mark_defers();
        self.check_block_stmts(f, inner, rest)?;
        let mut op = self.check_expr(f, inner, last, expected)?;
        if f.defers.len() > depth.position() {
            // Deferred code runs after the value is computed.
            if !matches!(op, Operand::Void) {
                let op_settled = self.settle_untyped(op, expected);
                let ty = op_settled.ty();
                let (_, v) = self.rvalue(f, op_settled, span)?;
                op = Operand::Value {
                    ty,
                    val: v,
                };
            }
            self.emit_defers(f, depth.position(), span)?;
        }
        f.truncate_defers(depth.position());
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
        if f.type_only
            && let Some(ty) = self.struct_field_type(scope, name, span)?
        {
            let addr = f.b.iconst(Ty::Ptr, 0);
            return Ok(Operand::Place {
                ty,
                addr,
            });
        }
        match self.lookup_full(scope, name)? {
            Found::Using(entry, member) => {
                let place = match &entry {
                    scope::UsingEntry::Place {
                        entity,
                        ty,
                    } => {
                        self.ide_note_use(*entity);
                        Some(*ty)
                    }
                    _ => None,
                };
                let op = self.using_member(f, entry, member, span)?;
                // `name` stands for `value.name`: hover and go to definition treat it as the field.
                if let Some(ty) = place {
                    self.ide_note_member_use(span, ty, member);
                    if let Some(ide) = self.ide.as_mut() {
                        ide.last_using_member = Some(member);
                    }
                }
                Ok(op)
            }
            Found::Entities(ids) => {
                if ids.is_empty() {
                    if f.type_only
                        && let Some(ty) = self.struct_field_type(scope, name, span)?
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
                    ide.last_entity = Some((span, ids[0]));
                    if ide.lint {
                        ide.used.extend(ids.iter().copied());
                    }
                }
                op
            }
        }
    }

    /// Resolve a lexical field type without requesting the containing struct's offsets.
    pub(super) fn struct_field_type(
        &mut self,
        scope: ScopeId,
        name: Sym,
        use_span: Span,
    ) -> Result<Option<TypeId>> {
        let mut s = Some(scope);
        while let Some(sid) = s {
            if self.scope(sid).names.contains_key(&name) {
                return Ok(None);
            }
            if let Some(decl) = self
                .field_declarations
                .get(&sid)
                .and_then(|fields| fields.get(&name))
                .cloned()
            {
                let ty = self.resolve_field_decl_type(sid, &decl)?;
                let declared = decl.names.iter().find(|n| n.name == name).unwrap().span;
                self.ide_note_named_field(use_span, name, declared, ty);
                return Ok(Some(ty));
            }
            if let Some(ty) = self.declared_promoted_field_type(sid, name, use_span)? {
                return Ok(Some(ty));
            }
            s = self.scope(sid).parent;
        }
        Ok(None)
    }

    pub fn entities_operand(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        ids: &[EntityId],
        span: Span,
    ) -> Result<Operand> {
        // A macro's backtick declaration reusing a parameter's name shadows the parameter: the
        // newest local is the one the rest of the body means.
        if ids.len() > 1
            && ids
                .iter()
                .all(|&id| matches!(self.entity(id).kind, EntityKind::Local { .. }))
        {
            return self.entity_operand(f, scope, ids[ids.len() - 1], span);
        }
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
                    }
                    .with_callable(self.callable_entities.get(&id).cloned()));
                }
                if depth != self.scope(scope).proc_depth {
                    if let Some((value, ty)) = self.local_consts.get(&id).cloned() {
                        return Ok(Operand::Const {
                            ty,
                            value,
                            untyped: false,
                        }
                        .with_callable(self.callable_entities.get(&id).cloned()));
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
                }
                .with_callable(self.callable_entities.get(&id).cloned()));
            }
            EntityKind::Builtin(scope::Builtin::Proc(p)) => return Ok(Operand::Builtin(p)),
            _ => {}
        }
        let resolved = self.resolve_entity(id)?;
        self.use_runtime_storage_entity(f, id);
        Ok(match resolved {
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
                if let Some(what) = f.constant_only
                    && !f.type_only
                {
                    let name = self.entity(id).name;
                    return err(span, format!("{what}, but `{name}` is a variable"));
                }
                let addr = f.b.storage_addr(storage);
                let parameters = if matches!(self.types.kind(ty), TypeKind::Proc(_)) {
                    self.callable_entity_parameters(id)?
                } else {
                    None
                };
                Operand::Place {
                    ty,
                    addr,
                }
                .with_callable(parameters)
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
        procs::validate_c_variadic_header(header)?;
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
            if let Some(name) = &p.name
                && self.ide.is_some()
            {
                self.ide_note_param_name(name.span, name.name, ty);
            }
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
            if let Some(name) = r.name {
                self.ide_note_param_name(name.span, name.name, t);
            }
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
                // Address-taking selects the pointer getter even when an ordinary
                // getter is available. Reuse its place normalization, and evaluate
                // each operand only once if builtin indexing is the fallback.
                let inner = if let E::Index(base, index) = &a.kind
                    && !self.operator_candidates(scope, "*[]")?.is_empty()
                {
                    let base_op = self.check_expr(f, scope, base, None)?;
                    let index_op = self.check_expr(f, scope, index, Some(TypeId::S64))?;
                    match self.try_pointer_index_operator(f, scope, &base_op, &index_op, span)? {
                        Some(place) => place,
                        None => {
                            self.index_operand(f, scope, base_op, index_op, index.span, a.span)?
                        }
                    }
                } else {
                    // The address of a variable (`*global`, `*global.field`) is constant even
                    // though the variable is not: naming it reads no value.
                    let constant_only = f.constant_only;
                    if names_variable_path(a) {
                        f.constant_only = None;
                    }
                    let inner = self.check_expr(f, scope, a, None);
                    f.constant_only = constant_only;
                    inner?
                };
                match inner {
                    Operand::Type(t) => Ok(Operand::Type(self.types.pointer(t))),
                    // Concrete default pointees share the explicit instance's identity.
                    // Required arguments still leave a source type for generic patterns.
                    Operand::PolyStruct(ps) => {
                        let t = if self.poly_struct_has_defaults(ps) {
                            self.instantiate_struct(ps, Vec::new(), span)?
                        } else {
                            self.poly_struct_type(ps)
                        };
                        Ok(Operand::Type(self.types.pointer(t)))
                    }
                    Operand::Place {
                        ty,
                        addr,
                    } => Ok(Operand::Value {
                        ty: self.types.pointer(ty),
                        val: addr,
                    }),
                    Operand::Procs(p) if p.len() == 1 => {
                        let op = Operand::Procs(p);
                        let parameters = self.callable_operand_parameters(f, &op)?;
                        let (ty, v) = self.rvalue(f, op, span)?;
                        Ok(Operand::Value {
                            ty,
                            val: v,
                        }
                        .with_callable(parameters))
                    }
                    Operand::Const {
                        value: Value::Int(_) | Value::Float(_, _) | Value::Bool(_),
                        ..
                    } => err(
                        span,
                        "cannot take the address of a number or bool constant: it has no storage (assign it to a variable first)",
                    ),
                    // An aggregate constant lives in static data, so its address outlives the
                    // procedure and is the same on every evaluation.
                    Operand::Const {
                        value: Value::Bytes(agg),
                        ty,
                        ..
                    } if self.is_memory_type(ty) => {
                        let key = (ty, agg.bytes.clone(), agg.relocs.clone());
                        let global = match self.const_addr_globals.get(&key) {
                            Some(&g) => g,
                            None => {
                                let size = self.size_of(ty, span)?;
                                let align = self.align_of(ty, span)?;
                                let g = self.program.add_global(ir::Global {
                                    name: format!("const.{}", self.program.globals.len()),
                                    size,
                                    align,
                                    init: agg.bytes.clone(),
                                    relocs: agg.relocs.clone(),
                                    read_only: false,
                                    export: None,
                                });
                                self.const_addr_globals.insert(key, g);
                                g
                            }
                        };
                        let addr = f.b.global_addr(global);
                        Ok(Operand::Value {
                            ty: self.types.pointer(ty),
                            val: addr,
                        })
                    }
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
                        (UnOp::Neg, Value::Float(v, required)) => Some(Value::Float(-v, *required)),
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

    /// Default precision follows `lexer::float_literal_precision`: the fractional-digit count for
    /// decimal spellings and the digit width for `0h` bit patterns.
    fn float_literal_type(&self, v: f64, span: Span) -> TypeId {
        let precision = crate::lexer::float_literal_precision(self.sources.snippet(span), v);
        if precision.defaults_to_float64 || precision.requires_float64 {
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
                        Value::Float(_, p) => {
                            self.types.is_float(r) && !(p.requires_float64 && r == TypeId::F32)
                        }
                        Value::Null => {
                            self.types.is_pointer(r)
                                || matches!(self.types.kind(r), TypeKind::Proc(_))
                        }
                        _ => false,
                    }
                });
                let ty = target.unwrap_or_else(|| self.default_untyped(ty, &value));
                let value = match value {
                    Value::Int(i) if self.types.is_float(ty) => Value::float(i as f64),
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
        if self.is_stand_in(ty) {
            return Ok(f.b.iconst(Ty::I8, 1));
        }
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

    /// Resolve the compiler-private native error reporter from the runtime's own
    /// file scopes. This transport hook is not a public library export.
    pub(crate) fn note_check_handler(&mut self, span: Span) -> Result<()> {
        if self.check_handler_resolved {
            return Ok(());
        }
        self.check_handler_resolved = true;
        let result = (|| {
            let Some(proc) = self.private_runtime_procedure("__jaic_check_failed", span)? else {
                return Ok(());
            };
            let signature = self.signature(proc, span)?;
            let expected = [
                TypeId::S64,
                TypeId::S64,
                TypeId::S64,
                TypeId::BOOL,
                TypeId::S64,
            ];
            if signature.has_context
                || signature.c_call
                || signature.c_varargs
                || !signature.returns.is_empty()
                || signature.params.len() != 6
                || signature.params.iter().any(|p| p.variadic || p.discard)
                || signature.params[..5]
                    .iter()
                    .zip(expected)
                    .any(|(p, ty)| p.ty != ty)
                || !matches!(self.types.kind(signature.params[5].ty), TypeKind::Pointer(ty) if *ty == TypeId::U8)
            {
                return err(
                    span,
                    "private native check reporter must take (s64, s64, s64, bool, s64, *u8), return nothing, and have no context",
                );
            }
            let super::procs::ProcTarget::Func(func) = self.proc_func(proc, span)? else {
                return err(span, "private native check reporter must be defined in Jai");
            };
            self.program.check_failed = Some(func);
            Ok(())
        })();
        if result.is_err() {
            self.check_handler_resolved = false;
        }
        result
    }

    /// Stop the program at a failed check that has no details (`ir::Intrinsic::Trap` with an
    /// `ir::CheckReason` reason). Every trap with a reason goes through here or `emit_check_failed`,
    /// so native code always has the reporting procedure.
    pub(crate) fn emit_trap(
        &mut self,
        f: &mut FnCtx,
        reason: ir::CheckReason,
        span: Span,
    ) -> Result<()> {
        self.note_check_handler(span)?;
        let reason = f.b.iconst(Ty::I64, reason as u64);
        f.b.intrinsic(ir::Intrinsic::Trap, vec![reason], &[]);
        Ok(())
    }

    /// Report a failed check (`ir::Intrinsic::CheckFailed`): `reason` is an `ir::CheckReason`, `a`
    /// and `b` its `I64` details. A fatal failure stops the program.
    pub(crate) fn emit_check_failed(
        &mut self,
        f: &mut FnCtx,
        reason: ir::CheckReason,
        a: ir::Val,
        b: ir::Val,
        fatal: bool,
        span: Span,
    ) -> Result<()> {
        self.note_check_handler(span)?;
        f.b.repeat_loc();
        let reason = f.b.iconst(Ty::I64, reason as u64);
        let fatal = f.b.iconst(Ty::I8, u64::from(fatal));
        f.b.intrinsic(ir::Intrinsic::CheckFailed, vec![reason, a, b, fatal], &[]);
        Ok(())
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
        if self.operator_candidates(scope, "*[]")?.is_empty() {
            return Ok(None);
        }
        let base = match base.clone().without_callable() {
            // A pointer stored in a variable denotes the receiver, not the
            // address of the variable containing that pointer.
            operand if self.types.pointee(operand.ty()).is_some() => {
                let (ty, val) = self.rvalue(f, operand, span)?;
                Operand::Value {
                    ty,
                    val,
                }
            }
            Operand::Place {
                ty,
                addr,
            } => Operand::Value {
                ty: self.types.pointer(ty),
                val: addr,
            },
            // By-value receivers also accept computed aggregate values. The
            // normal call binder retains their type and conversion rules.
            operand => operand,
        };
        let Some(result) = self.try_index_operator_overload(f, scope, "*[]", &base, index, span)?
        else {
            return Ok(None);
        };
        let (ty, val) = self.rvalue(f, result, span)?;
        let Some(elem) = self.types.pointee(ty) else {
            return err(span, "operator *[] must return a pointer");
        };
        Ok(Some(Operand::Place {
            ty: elem,
            addr: val,
        }))
    }

    #[allow(clippy::too_many_arguments)]
    /// The type an `ifx` branch contributes, or `None` when it adapts to the other branch
    /// (an untyped constant) or is not a plain value.
    fn ifx_branch_type(op: &Operand) -> Option<TypeId> {
        match op {
            Operand::Const {
                untyped: true, ..
            }
            | Operand::Procs(_)
            | Operand::Void => None,
            other => Some(other.ty()),
        }
    }

    /// Both arms of an `ifx` must meet in one type: `ifx c then 1 else "a"` has none.
    fn ifx_branch_agreement(
        &mut self,
        then_op: &Operand,
        else_op: &Operand,
        then_e: &ast::Expr,
        else_e: &ast::Expr,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<()> {
        if matches!(
            then_op,
            Operand::Procs(_) | Operand::Void | Operand::Type(_)
        ) || matches!(
            else_op,
            Operand::Procs(_) | Operand::Void | Operand::Type(_)
        ) {
            return Ok(());
        }
        let (tt, et) = (
            Self::ifx_branch_type(then_op),
            Self::ifx_branch_type(else_op),
        );
        let number = |c: &Self, o: &Operand| {
            matches!(
                o,
                Operand::Const {
                    value: Value::Int(_) | Value::Float(_, _),
                    untyped: true,
                    ..
                }
            ) && !c.types.is_pointer(o.ty())
        };
        let agree = match (tt, et) {
            (Some(a), Some(b)) if a != b => {
                let to = |c: &mut Self, from: TypeId, into: TypeId| {
                    c.implicit_cost(from, false, into).is_some()
                };
                to(self, a, b)
                    || to(self, b, a)
                    || expected.is_some_and(|e| to(self, a, e) && to(self, b, e))
            }
            (None, Some(b)) => !(number(self, then_op) && self.types.repr(b) == TypeId::STRING),
            (Some(a), None) => !(number(self, else_op) && self.types.repr(a) == TypeId::STRING),
            _ => true,
        };
        if agree {
            return Ok(());
        }
        let describe = |c: &mut Self, o: &Operand| match Self::ifx_branch_type(o) {
            Some(t) => c.types.name(t).to_string(),
            None => {
                let d = c.settle_untyped(o.clone(), None).ty();
                format!("{} (untyped constant)", c.types.name(d))
            }
        };
        let (a, b) = (describe(self, then_op), describe(self, else_op));
        Err(Box::new(
            Diagnostic::error(span, "the branches of `ifx` have different types")
                .with_note(then_e.span, format!("the `then` value is `{a}`"))
                .with_note(else_e.span, format!("the `else` value is `{b}`")),
        ))
    }

    /// `ifx` with a constant condition: the taken branch is the value, so it stays constant,
    /// but the other branch is still checked, and the arms must still agree on a type.
    #[allow(clippy::too_many_arguments)]
    fn fold_constant_ifx(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        cond_holds: bool,
        then_e: &ast::Expr,
        else_e: &ast::Expr,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        let (taken_e, other_e) = if cond_holds {
            (then_e, else_e)
        } else {
            (else_e, then_e)
        };
        let taken = self.check_expr(f, scope, taken_e, expected)?;
        // The other arm is lowered into a block nothing reaches.
        let dead = f.b.new_block();
        let after = f.b.new_block();
        let never = f.b.iconst(Ty::I8, 0);
        f.b.branch(never, dead, after);
        f.b.switch_to(dead);
        let other = self.check_expr(f, scope, other_e, expected);
        f.b.jump(after);
        f.b.switch_to(after);
        let other = other?;
        let (then_op, else_op) = if cond_holds {
            (&taken, &other)
        } else {
            (&other, &taken)
        };
        self.ifx_branch_agreement(then_op, else_op, then_e, else_e, expected, span)?;
        // A one-character string only becomes a byte where a string is converted directly,
        // not through `ifx`.
        if let (
            Operand::Const {
                value: Value::String(_),
                ..
            },
            Some(e),
        ) = (&taken, expected)
            && self.types.is_integer(self.types.repr(e))
        {
            let (ty, val) = self.rvalue(f, taken.clone(), span)?;
            self.convert(
                f,
                Operand::Value {
                    ty,
                    val,
                },
                e,
                span,
            )?;
        }
        Ok(taken)
    }

    fn check_ifx(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        cond: &ast::Expr,
        (then_value, else_value): (Option<&ast::Expr>, Option<&ast::Expr>),
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
                if let (Some(then_e), Some(else_e)) = (then_value, else_value) {
                    return self.fold_constant_ifx(f, scope, t, then_e, else_e, expected, span);
                }
                if t {
                    return self.check_expr(f, scope, then_value.unwrap(), expected);
                } else if let Some(else_value) = else_value {
                    return self.check_expr(f, scope, else_value, expected);
                }
                // A false condition without else still needs the then type.
                // Lower it into the untaken block below, so discovering that
                // type does not execute the arm's effects.
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
        if let (Some(then_e), Some(else_e), Some(else_op)) = (then_value, else_value, &else_op) {
            self.ifx_branch_agreement(&then_op, else_op, then_e, else_e, expected, span)?;
        }
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
        if ty == TypeId::VOID {
            // A discarded conditional can join void arms. Their effects are
            // already emitted in their own blocks; there is no result storage
            // to load. Conversion still rejects an incompatible value arm.
            if let Some(else_op) = else_op {
                self.convert(f, else_op, ty, span)?;
            }
            f.b.jump(done);
            f.b.switch_to(then_end);
            self.convert(f, then_op, ty, span)?;
            f.b.jump(done);
            f.b.switch_to(done);
            return Ok(Operand::Void);
        }
        let size = self.size_of(ty, span)?;
        let align = self.align_of(ty, span)?;
        // The result slot must be allocated before both branches use it; slots are function-wide.
        let slot = f.b.slot(size.max(1), align);
        let addr = f.b.slot_addr(slot);
        let mut callable_parameters = None;
        match (else_op, else_value) {
            (Some(else_op), Some(else_value)) => {
                let else_converted = self.convert(f, else_op, ty, else_value.span)?;
                // Runtime selection changes the callee, not its lexical call
                // header: inference retains the else arm's names/defaults.
                callable_parameters = self.callable_operand_parameters(f, &else_converted)?;
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
        .map(|operand| operand.with_callable(callable_parameters))
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
        // A field named through its owning type has no instance storage. In
        // type_of, represent its array as a type-only place so indexing finds
        // the element type through the ordinary array rules, without a load.
        let base_op = match base_op {
            Operand::Type(ty)
                if f.type_only
                    && matches!(self.types.kind(self.types.repr(ty)), TypeKind::Array { .. }) =>
            {
                Operand::Place {
                    ty,
                    addr: f.b.iconst(Ty::Ptr, 0),
                }
            }
            other => other,
        };
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
            self.note_check_handler(span)?;
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
