//! The conversion cost of an argument against a parameter.
use super::{CallArg, inferred_flags, is_deferred};
use crate::ast::ExprKind as E;
use crate::sema::convert;
use crate::sema::lower::Operand;
use crate::sema::{Compiler, Diagnostic, Result, ScopeId, Span, Sym, TypeId, Value, ast, err};
use crate::types::TypeKind;

impl Compiler {
    /// The operand an `ifx` argument offers to polymorphic inference: its first branch with a
    /// type of its own (`ifx c then x else y` with `x: s16` binds `$T` to `s16`), else its first
    /// branch that checks at all (an untyped constant, defaulted like a plain literal argument).
    pub(super) fn ifx_binding_operand(&mut self, arg: &CallArg) -> Option<Operand> {
        let expr = arg.expr.as_ref()?;
        let mut fallback = None;
        let typed = self.ifx_branch_operand(arg.scope, expr, &mut fallback);
        typed.or(fallback)
    }

    /// [`Self::ifx_binding_operand`] for one `ifx` expression. A branch that is itself an `ifx`
    /// (`ifx a then (ifx b then x else y) else z`) offers its own branches, so nesting binds too.
    fn ifx_branch_operand(
        &mut self,
        scope: ScopeId,
        expr: &ast::Expr,
        fallback: &mut Option<Operand>,
    ) -> Option<Operand> {
        let E::Ifx {
            then_value,
            else_value,
            ..
        } = &expr.kind
        else {
            return None;
        };
        for branch in [then_value, else_value].into_iter().flatten() {
            if matches!(branch.kind, E::Ifx { .. }) {
                if let Some(op) = self.ifx_branch_operand(scope, branch, fallback) {
                    return Some(op);
                }
                continue;
            }
            if is_deferred(branch) {
                continue;
            }
            let Ok(op) = self.check_expr_no_emit(scope, branch) else {
                continue;
            };
            match op {
                Operand::Const {
                    untyped: true, ..
                } => {
                    fallback.get_or_insert(op);
                }
                Operand::Value {
                    ..
                }
                | Operand::Place {
                    ..
                }
                | Operand::Const {
                    ..
                } => {
                    return Some(op);
                }
                _ => {}
            }
        }
        None
    }

    /// How well the arguments of an untyped `.{...}` fit the members of `param`: `None` when
    /// they cannot fill it, else the number of members left to their defaults (a literal with no
    /// values says nothing about the shape, so it fits every type alike). A named member must
    /// exist; positional values must not outnumber the members that take them, must reach the
    /// last member without an initializer, and an array literal needs an array member.
    pub(super) fn struct_lit_fit(
        &mut self,
        param: TypeId,
        fields: &[ast::Arg],
        scope: ScopeId,
        span: Span,
    ) -> Result<Option<u32>> {
        if self.size_of(param, span).is_err() || fields.is_empty() {
            return Ok(Some(0));
        }
        let members = self.positional_members(param, span)?;
        let mut positional = 0;
        let mut named = 0;
        for field in fields {
            if let Some(name) = field.name {
                let Some((_, member)) = self.find_member(param, name.name, name.span)? else {
                    return Ok(None);
                };
                if !self.value_fits_member(scope, &field.value, member) {
                    return Ok(None);
                }
                named += 1;
            } else if field.target.is_none() {
                let Some(&(_, member, _)) = members.get(positional) else {
                    return Ok(None);
                };
                if matches!(field.value.kind, E::ArrayLit { .. })
                    && !matches!(
                        self.ty.types.kind(self.ty.types.repr(member)),
                        TypeKind::Array { .. }
                    )
                {
                    return Ok(None);
                }
                if !self.value_fits_member(scope, &field.value, member) {
                    return Ok(None);
                }
                positional += 1;
            }
        }
        // All values positional and fewer than the fields without initializers: the literal
        // itself would be rejected, so it is not this type.
        if positional > 0
            && positional == fields.len()
            && positional < self.positional_values_needed(param)
        {
            return Ok(None);
        }
        // Positional values fill members from the front, so a type with exactly that many
        // members is closer. Named values pick their members, and a member left to its
        // initializer is not a shortfall.
        let unfilled = if named == 0 {
            members.len().saturating_sub(positional)
        } else {
            let defaults = self.positional_member_defaults(param);
            let named_here = |name: Option<Sym>| {
                fields
                    .iter()
                    .any(|f| f.name.is_some_and(|n| Some(n.name) == name))
            };
            members
                .iter()
                .enumerate()
                .skip(positional)
                .filter(|&(i, &(name, ..))| {
                    !named_here(name) && !defaults.get(i).copied().unwrap_or(false)
                })
                .count()
        };
        Ok(Some(u32::try_from(unfilled).unwrap_or(u32::MAX)))
    }

    /// Whether a value of a literal can be stored in a member of type `member`. A value that
    /// takes its type from the member (`.A`, `.{...}`) always can.
    fn value_fits_member(&mut self, scope: ScopeId, value: &ast::Expr, member: TypeId) -> bool {
        if is_deferred(value) {
            return true;
        }
        let Ok(op) = self.check_expr_no_emit(scope, value) else {
            return true;
        };
        let probe = CallArg {
            name: None,
            spread: false,
            expr: Some(value.clone()),
            op: Some(op),
            span: value.span,
            scope,
        };
        self.arg_cost(&probe, member, false).is_ok()
    }

    /// The fit of a deferred argument to `param` that breaks ties between equally cheap
    /// candidates: see [`Self::struct_lit_fit`]. Arguments of any other shape fit alike.
    pub(super) fn arg_fit(&mut self, arg: &CallArg, param: TypeId) -> Result<u32> {
        if arg.op.is_none()
            && let Some(ast::Expr {
                kind:
                    E::StructLit {
                        ty: None,
                        fields,
                    },
                ..
            }) = &arg.expr
        {
            return Ok(self
                .struct_lit_fit(param, fields, arg.scope, arg.span)?
                .unwrap_or(0));
        }
        Ok(0)
    }

    /// `macro_call`: a Code parameter of a macro also binds a variable by name.
    pub(super) fn arg_cost(
        &mut self,
        arg: &CallArg,
        param: TypeId,
        macro_call: bool,
    ) -> Result<u32> {
        if self.is_register_operand_type(param, arg.span) {
            if !macro_call {
                return err(arg.span, "register operands require an expanded procedure");
            }
            if let Some(expr) = &arg.expr
                && self.asm_reg_named(arg.scope, expr)?.is_some()
            {
                return Ok(convert::EXACT);
            }
            return err(
                arg.span,
                "expected an #asm register operand, not a scalar or memory value",
            );
        }

        let Some(op) = &arg.op else {
            if let Some(ast::Expr {
                kind: E::Lambda {
                    header, ..
                },
                ..
            }) = &arg.expr
            {
                match self.ty.types.kind(param) {
                    TypeKind::Proc(pt) if pt.params.len() == header.params.len() => {}
                    TypeKind::Proc(pt) => {
                        return err(
                            arg.span,
                            format!(
                                "lambda takes {} parameters, but {} expects {}",
                                header.params.len(),
                                self.ty.types.name(param),
                                pt.params.len()
                            ),
                        );
                    }
                    _ => {
                        return err(
                            arg.span,
                            format!("a lambda cannot be passed as {}", self.ty.types.name(param)),
                        );
                    }
                }
            }
            // Any expression converts to a `Code` parameter (plain procedures too).
            if param == TypeId::CODE {
                return Ok(convert::LITERAL);
            }
            // `ifx c then a else b`: each branch must fit (`"-->"` does not fit a `u8`).
            if let Some(ast::Expr {
                kind:
                    E::Ifx {
                        then_value,
                        else_value,
                        ..
                    },
                ..
            }) = &arg.expr
            {
                let mut cost = convert::LITERAL;
                for branch in [then_value, else_value].into_iter().flatten() {
                    if is_deferred(branch) {
                        continue;
                    }
                    let Ok(op) = self.check_expr_no_emit(arg.scope, branch) else {
                        continue;
                    };
                    let probe = CallArg {
                        op: Some(op),
                        expr: Some((**branch).clone()),
                        span: branch.span,
                        ..arg.clone()
                    };
                    cost = cost.max(self.arg_cost(&probe, param, macro_call)?);
                }
                return Ok(cost);
            }
            // Deferred arguments fit any plausible target; prefer exact-looking ones.
            let scalar = matches!(
                self.ty.types.kind(self.ty.types.repr_struct(param)),
                TypeKind::Bool
                    | TypeKind::Int { .. }
                    | TypeKind::Float { .. }
                    | TypeKind::String
                    | TypeKind::Type
            );
            let fits = match arg.expr.as_ref().map(|e| &e.kind) {
                // `.{...}` and `.[...]` build aggregates, `.NAME` picks an enum member.
                Some(E::StructLit {
                    fields: lit_args, ..
                }) => {
                    (!scalar
                        || !self
                            .builtin_members(self.ty.types.repr(param), arg.span)?
                            .is_empty())
                        && self
                            .struct_lit_fit(param, lit_args, arg.scope, arg.span)?
                            .is_some()
                }
                Some(E::ArrayLit {
                    ..
                }) => !scalar,
                Some(_) if arg.expr.as_ref().is_some_and(inferred_flags) => {
                    matches!(self.ty.types.kind(param), TypeKind::Enum(_))
                }
                _ => true,
            };
            if !fits {
                return err(
                    arg.span,
                    format!(
                        "argument cannot be inferred as parameter type {}",
                        self.ty.types.name(param)
                    ),
                );
            }
            return Ok(convert::LITERAL);
        };
        let untyped = matches!(
            op,
            Operand::Const {
                untyped: true,
                ..
            }
        );
        match op {
            Operand::Void => return err(arg.span, "the argument has no value"),
            Operand::Type(_) | Operand::PolyStruct(_) if param == TypeId::TYPE => {
                return Ok(convert::EXACT);
            }
            Operand::PolyStruct(_) if param == TypeId::ANY => return Ok(convert::TO_ANY),
            Operand::Type(_) if param == TypeId::ANY => return Ok(convert::TO_ANY),
            Operand::Procs(procs) => {
                if let TypeKind::Proc(_) = self.ty.types.kind(param) {
                    for &p in &procs.clone() {
                        if !self.proc(p).is_poly && self.proc_type(p, arg.span)? == param {
                            return Ok(convert::EXACT);
                        }
                    }
                    for &p in &procs.clone() {
                        if self.proc(p).is_poly
                            && self.instantiate_for_proc_type(p, param, arg.span).is_some()
                        {
                            return Ok(convert::WIDEN);
                        }
                    }
                    if procs.len() == 1 && !self.proc(procs[0]).is_poly {
                        let pt = self.proc_type(procs[0], arg.span)?;
                        if let (TypeKind::Proc(a), TypeKind::Proc(b)) =
                            (self.ty.types.kind(pt), self.ty.types.kind(param))
                            && a.params == b.params
                            && a.returns == b.returns
                        {
                            return Ok(convert::WIDEN);
                        }
                    }
                    return err(
                        arg.span,
                        format!(
                            "procedure does not match parameter type {}",
                            self.ty.types.name(param)
                        ),
                    );
                }
                if param == TypeId::VOID_PTR || param == TypeId::ANY {
                    return Ok(convert::POINTER);
                }
            }
            // `split(s, ".")` into a `u8` parameter: a one-character string is that byte
            // (`convert` makes the value). A `string` overload is still the closer match.
            Operand::Const {
                value: Value::String(s),
                ..
            } if s.len() == 1 && param == TypeId::U8 => return Ok(convert::WIDEN),
            Operand::Const {
                value: Value::Int(v),
                untyped: true,
                ..
            } if self.ty.types.is_integer(param) => {
                let (bits, signed) = self.ty.types.int_info(param).unwrap();
                if !convert::int_constant_fits(*v, bits, signed, self.src.bit_pattern(arg.span)) {
                    return err(
                        arg.span,
                        format!("constant {v} does not fit in {}", self.ty.types.name(param)),
                    );
                }
                // Jai lets an untyped integer constant (`#char ","`, `3`) stand for an
                // enum value; rank it below every ordinary integer conversion.
                if matches!(self.ty.types.kind(param), TypeKind::Enum(_))
                    && !self.ty.types.is_loose_enum(param)
                {
                    return Ok(convert::SUBTYPE);
                }
                return Ok(if param == TypeId::S64 {
                    convert::EXACT
                } else {
                    convert::LITERAL
                });
            }
            Operand::Const {
                value: Value::Float(_, precision),
                untyped: true,
                ..
            } if precision.requires_float64 && self.ty.types.repr(param) == TypeId::F32 => {
                return err(
                    arg.span,
                    "this floating literal requires float64 precision (use an explicit cast to round it)",
                );
            }
            // A string literal is NUL-terminated, so it converts to a C string.
            Operand::Const {
                value: Value::String(_),
                ..
            } if self.ty.types.pointee(param) == Some(TypeId::U8) => return Ok(convert::POINTER),
            // A string literal converts to a `#type,distinct` / `#type,isa` string variant.
            Operand::Const {
                value: Value::String(_),
                ..
            } if matches!(self.ty.types.kind(param), TypeKind::Distinct(_))
                && self.ty.types.repr(param) == TypeId::STRING =>
            {
                return Ok(convert::LITERAL);
            }
            _ => {}
        }
        let from = op.ty();
        if from == TypeId::F32 && untyped && param == TypeId::F32 {
            return Ok(convert::EXACT);
        }
        if self.ty.auto_deref_arg(from, param) {
            return Ok(convert::SUBTYPE);
        }
        // Code macro parameters may quote a caller variable by name.
        if param == TypeId::CODE
            && macro_call
            && matches!(op, Operand::Place { .. })
            && matches!(arg.expr.as_ref().map(|e| &e.kind), Some(E::Ident(_)))
            && self.implicit_cost(from, untyped, param).is_none()
        {
            return Ok(convert::LITERAL);
        }
        self.implicit_cost(from, untyped, param).ok_or_else(|| {
            let mut d = Diagnostic::error(
                arg.span,
                format!(
                    "argument of type `{}` does not match parameter type `{}`",
                    self.ty.types.name(from),
                    self.ty.types.name(param)
                ),
            );
            if let Some(help) = self.ty.conversion_help(from, param) {
                d = d.with_help(help);
            }
            Box::new(d)
        })
    }
}
