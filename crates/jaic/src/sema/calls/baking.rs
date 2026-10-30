//! Values for baked (`$x`) parameters: defaults, `Code` arguments and constant packs.

use super::{
    ArrayKind, CallArg, Compiler, Diagnostic, E, Operand, PackKey, Rc, Result, ScopeId, Span, Sym,
    TypeId, Value, ast, err, ir, procs, value,
};

use crate::layout;

impl Compiler {
    /// The value of a defaulted baked parameter. A declared type naming earlier bindings
    /// (`$compare: (T, T) -> bool = ...`) is read in a scope holding them.
    pub(crate) fn baked_default(
        &mut self,
        def_scope: ScopeId,
        param: &ast::Param,
        d: &ast::Expr,
        bindings: &[(Sym, Value, TypeId)],
        span: Span,
    ) -> Result<(Value, TypeId)> {
        // `$loc := #caller_location` is the place of the call, as for a runtime parameter:
        // generic code can hand it to `compiler_report` so an error points at the user's call.
        if matches!(d.kind, E::CallerLocation) {
            let file = self.scope_file(def_scope);
            let mut f = self.thunk_ctx(ir::FuncOrigin::ConstInit, file);
            if let Operand::Const {
                ty,
                value,
                ..
            } = self.location_operand(&mut f, span)?
            {
                return Ok((value, ty));
            }
        }
        let mut scope = def_scope;
        if !bindings.is_empty() {
            // One scope per set of bindings, so a lambda default
            // (`$compare := (a, b) => a == b`) is the same procedure on every
            // call and the instance is found again.
            let key = (def_scope, bindings.iter().map(|b| b.1.clone()).collect());
            scope = if let Some(&s) = self.ns.default_scopes.get(&key) {
                s
            } else {
                let s = self.new_hidden_scope(def_scope);
                // Not written at the call: recording them there would make the call's
                // text look like a declaration of `T` to hover and go to definition.
                for (n, v, t) in bindings {
                    self.new_const(s, *n, span, v.clone(), *t);
                }
                self.ns.default_scopes.insert(key, s);
                s
            };
        }
        let declared = match &param.ty {
            Some(t) if !procs::has_poly(t) => Some(self.eval_type(scope, t)?),
            _ => None,
        };
        let (v, ty) = if let Some(t) = declared {
            (self.const_value_of_type(scope, d, t)?, t)
        } else {
            let ty = self.eval_const(scope, d, None)?.ty();
            (self.const_value_of_type(scope, d, ty)?, ty)
        };
        Ok((v, ty))
    }

    /// The code of a `Code` parameter's argument: the argument expression itself, unevaluated,
    /// or the code a `Code` constant names (another `$c: Code` passes its code on). `None`
    /// when the argument already is a constant `Code` value.
    pub(super) fn quote_code_arg(&mut self, arg: &CallArg) -> Option<value::CodeId> {
        if matches!(
            arg.op,
            Some(Operand::Const {
                value: Value::Code(_),
                ..
            })
        ) {
            return None;
        }
        let expr = arg.expr.as_ref()?;
        let named = match &expr.kind {
            ast::ExprKind::Ident(_) | ast::ExprKind::Member(..) => {
                match self.check_expr_no_emit(arg.scope, expr) {
                    Ok(Operand::Const {
                        value: Value::Code(code),
                        ..
                    }) => Some(code),
                    _ => None,
                }
            }
            _ => None,
        };
        Some(match named {
            Some(code) => code,
            None => self.add_code(Rc::new(ast::CodeBody::Expr(expr.clone())), arg.scope),
        })
    }

    /// A baked variadic parameter given `..view`: the view must be a compile-time constant
    /// of the parameter's array type.
    pub(super) fn baked_spread(
        &mut self,
        def_scope: ScopeId,
        param: &ast::Param,
        arg: &CallArg,
    ) -> Result<(Value, TypeId)> {
        let elem = match &param.ty {
            Some(t) if !procs::has_poly(t) => self.eval_type(def_scope, t)?,
            _ => TypeId::TYPE,
        };
        let ty = self.ty.types.array(elem, ArrayKind::View);
        let op = match (&arg.op, &arg.expr) {
            (Some(op), _) => op.clone(),
            (None, Some(e)) => self.eval_const(arg.scope, e, Some(ty))?,
            (None, None) => return err(arg.span, "missing value"),
        };
        let value = self.const_value_of_operand(arg.scope, op, ty, arg.span)?;
        Ok((value, ty))
    }

    /// The value of a baked variadic parameter (`$types: ..Type`): a constant `[] T`
    /// view over the arguments, each of which must be a compile-time constant.
    pub(super) fn baked_pack(
        &mut self,
        def_scope: ScopeId,
        param: &ast::Param,
        args: &[&CallArg],
    ) -> Result<(Value, TypeId)> {
        let span = param.span;
        let elem = match &param.ty {
            Some(t) if !procs::has_poly(t) => self.eval_type(def_scope, t)?,
            _ => TypeId::TYPE,
        };
        let esize = self.size_of(elem, span)?;
        let mut array = value::Aggregate::zeroed(0, span)?;
        let mut count = 0u64;
        for arg in args {
            if arg.spread {
                let (view, _) = self.baked_spread(def_scope, param, arg)?;
                self.append_constant_spread(&mut array, &mut count, elem, view, arg.span)?;
                continue;
            }
            let size = (count + 1)
                .checked_mul(esize)
                .filter(|&n| n <= value::MAX_IMAGE)
                .ok_or_else(|| {
                    Box::new(Diagnostic::error(
                        arg.span,
                        "baked variadic pack is too large",
                    ))
                })?;
            array.bytes.resize(size as usize, 0);
            let value = if elem == TypeId::CODE
                && let Some(id) = self.quote_code_arg(arg)
            {
                Value::Code(id)
            } else {
                let op = match (&arg.op, &arg.expr) {
                    (Some(op), _) => op.clone(),
                    (None, Some(e)) => self.eval_const(arg.scope, e, Some(elem))?,
                    (None, None) => return err(arg.span, "missing value"),
                };
                self.const_value_of_operand(arg.scope, op, elem, arg.span)?
            };
            self.write_value(&mut array, count * esize, &value, elem, arg.span)?;
            count += 1;
        }
        let view = self.const_pack(elem, array, count, span)?;
        Ok((view, self.ty.types.array(elem, ArrayKind::View)))
    }

    /// Snapshot a constant view through interpreter memory so changed compile-time
    /// globals and pointer/type relocations are retained, rather than reading stale initializers.
    pub(super) fn append_constant_spread(
        &mut self,
        array: &mut value::Aggregate,
        count: &mut u64,
        elem: TypeId,
        view: Value,
        span: Span,
    ) -> Result<()> {
        let ty = self.ty.types.array(elem, ArrayKind::View);
        let mut image = value::Aggregate::zeroed(16, span)?;
        self.write_value(&mut image, 0, &view, ty, span)?;
        let g = self.out.program.add_global(ir::Global {
            name: "baked.spread.view".into(),
            size: 16,
            align: 8,
            init: image.bytes,
            relocs: image.relocs,
            read_only: true,
            export: None,
        });
        let addr = self
            .ct
            .interp
            .global_addr(&self.out.program, g)
            .map_err(|t| Box::new(Diagnostic::error(span, t.into_message())))?;
        let n = self.ct.program_u64(addr + layout::view::COUNT, span)?;
        let data = self.ct.program_u64(addr + layout::view::DATA, span)?;
        let esize = self.size_of(elem, span)?;
        let total = count
            .checked_add(n)
            .filter(|&n| n <= value::MAX_IMAGE)
            .ok_or_else(|| Box::new(Diagnostic::error(span, "baked variadic pack is too large")))?;
        let size = total
            .checked_mul(esize)
            .filter(|&n| n <= value::MAX_IMAGE)
            .ok_or_else(|| Box::new(Diagnostic::error(span, "baked variadic pack is too large")))?;
        array.bytes.resize(size as usize, 0);
        for i in 0..n {
            let at = data.checked_add(i * esize).ok_or_else(|| {
                Box::new(Diagnostic::error(span, "baked spread pointer overflow"))
            })?;
            let value = self.read_value(at, elem, span)?;
            self.write_value(array, (*count + i) * esize, &value, elem, span)?;
        }
        *count = total;
        Ok(())
    }

    /// A constant `[] elem` view over `count` elements laid out in `array`. Equal contents
    /// share one read-only global, so equal argument lists key the same instance.
    pub(in crate::sema) fn const_pack(
        &mut self,
        elem: TypeId,
        array: value::Aggregate,
        count: u64,
        span: Span,
    ) -> Result<Value> {
        let align = self.align_of(elem, span)?;
        let key: PackKey = (
            align,
            array.bytes.clone(),
            array
                .relocs
                .iter()
                .map(|r| (r.offset, r.target, r.addend))
                .collect(),
        );
        let data = if let Some(&g) = self.out.packs.get(&key) {
            g
        } else {
            let g = self.out.program.add_global(ir::Global {
                name: "baked.pack".into(),
                size: array.bytes.len().max(1) as u64,
                align,
                init: array.bytes,
                relocs: array.relocs,
                read_only: true,
                export: None,
            });
            self.out.packs.insert(key, g);
            g
        };
        let mut view = value::Aggregate {
            bytes: vec![0; 16],
            relocs: vec![ir::Reloc {
                offset: 8,
                target: ir::RelocTarget::Global(data),
                addend: 0,
            }],
        };
        view.bytes[..8].copy_from_slice(&count.to_le_bytes());
        Ok(Value::Bytes(Rc::new(view)))
    }

    /// Whether a constant of `ty` is a bool, integer, float or enum value, which the argument
    /// of a constant parameter must convert to.
    /// Whether `ty` is a `[] T` view.
    pub(super) fn is_view_type(&self, ty: TypeId) -> bool {
        matches!(
            self.ty.types.kind(ty),
            crate::types::TypeKind::Array {
                kind: ArrayKind::View,
                ..
            }
        )
    }

    pub(super) fn is_scalar_constant_type(&self, ty: TypeId) -> bool {
        let repr = self.ty.types.repr(ty);
        repr == TypeId::BOOL
            || self.ty.types.is_integer(repr)
            || self.ty.types.is_float(repr)
            || matches!(self.ty.types.kind(repr), crate::types::TypeKind::Enum(_))
    }
}
