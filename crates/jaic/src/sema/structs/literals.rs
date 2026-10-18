//! Struct and array literals, and writing values into constant images.
use super::model::PathStep;
use crate::ast::ExprKind as E;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::value::Aggregate;
use crate::sema::*;
use crate::types::{ArrayKind, TypeKind};

impl Compiler {
    #[allow(clippy::too_many_arguments)]
    pub fn check_struct_literal(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        ty_expr: Option<&ast::Expr>,
        args: &[ast::Arg],
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        let ty = match ty_expr {
            Some(t) => self.eval_type_in(f, scope, t)?,
            None => match expected {
                Some(t) => t,
                None => {
                    return err(
                        span,
                        "cannot infer the type of `.{...}` here; write `Type.{...}`",
                    );
                }
            },
        };
        let size = self.size_of(ty, span)?;
        // Member list for positional arguments.
        let members: Vec<(Option<Sym>, TypeId, u64)> =
            match self.types.as_struct(self.types.repr_struct(ty)) {
                Some(s) => self
                    .types
                    .struct_info(s)
                    .fields
                    .iter()
                    .map(|f| (f.name, f.ty, f.offset))
                    .collect(),
                None => self
                    .builtin_members(self.types.repr(ty), span)?
                    .into_iter()
                    .map(|(n, t, o)| (Some(n), t, o))
                    .collect(),
            };
        if members.is_empty()
            && !args.is_empty()
            && self.types.as_struct(self.types.repr_struct(ty)).is_none()
        {
            return err(
                span,
                format!(
                    "cannot use a struct literal for type {}",
                    self.types.name(ty)
                ),
            );
        }
        // Check each argument against its member type.
        let mut values: Vec<(Vec<PathStep>, TypeId, Operand, Span)> = Vec::new();
        let mut positional = 0;
        // `.{a.b = v, xs[1] = w}`: assigned after the ordinary fields.
        let (targeted, args): (Vec<&ast::Arg>, Vec<&ast::Arg>) =
            args.iter().partition(|a| a.target.is_some());
        for arg in args {
            let (path, mty) = if let Some(n) = arg.name {
                match self.find_member(ty, n.name, n.span)? {
                    Some(m) => {
                        self.ide_note_member_use(n.span, ty, n.name);
                        self.ide_note_member_ref(n.span, n.name, m.1);
                        m
                    }
                    None => {
                        return Err(Box::new(self.no_member(ty, n.name, n.span)));
                    }
                }
            } else {
                let Some(&(_, mty, off)) = members.get(positional) else {
                    return err(
                        arg.value.span,
                        format!("too many values for {}", self.types.name(ty)),
                    );
                };
                positional += 1;
                (vec![PathStep::Offset(off)], mty)
            };
            let op = self.check_expr(f, scope, &arg.value, Some(mty))?;
            let op = self.convert(f, op, mty, arg.value.span)?;
            values.push((path, mty, op, arg.value.span));
        }
        let all_const = values.iter().all(|(path, _, op, _)| {
            path.iter().all(|s| matches!(s, PathStep::Offset(_)))
                && (op.is_const() || matches!(op, Operand::Procs(p) if p.len() == 1))
        });
        if all_const && targeted.is_empty() {
            let mut agg = match self.default_initializer(ty, span)? {
                Some(img) => (*img).clone(),
                None => Aggregate::zeroed(size, span)?,
            };
            for (path, mty, op, vspan) in &values {
                let offset: u64 = path
                    .iter()
                    .map(|s| {
                        if let PathStep::Offset(o) = s {
                            *o
                        } else {
                            0
                        }
                    })
                    .sum();
                let value = match op {
                    Operand::Procs(p) => Value::Proc(p[0]),
                    other => other.const_value().unwrap(),
                };
                self.write_value(&mut agg, offset, &value, *mty, *vspan)?;
            }
            return Ok(Operand::Const {
                ty,
                value: Value::Bytes(Rc::new(agg)),
                untyped: false,
            });
        }
        let align = self.align_of(ty, span)?;
        let tmp = f.b.alloca(size.max(1), align);
        self.init_default(f, ty, tmp, span)?;
        for (path, mty, op, vspan) in values {
            let at = self.apply_path(f, tmp, &path);
            let (_, v) = self.rvalue(f, op, vspan)?;
            self.store_value(f, mty, at, v, vspan)?;
        }
        for arg in targeted {
            let target = arg.target.as_ref().unwrap();
            let Operand::Place {
                ty: pty,
                addr,
            } = self
                .literal_target(f, scope, ty, tmp, target)?
                .without_callable()
            else {
                return err(target.span, "this field target cannot be assigned");
            };
            let op = self.check_expr(f, scope, &arg.value, Some(pty))?;
            let op = self.convert(f, op, pty, arg.value.span)?;
            let (_, v) = self.rvalue(f, op, arg.value.span)?;
            self.store_value(f, pty, addr, v, arg.value.span)?;
        }
        Ok(Operand::Value {
            ty,
            val: tmp,
        })
    }

    /// The place a literal field target (`a.b`, `xs[i]`) names inside the
    /// literal being built at `base`.
    fn literal_target(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        ty: TypeId,
        base: ir::Val,
        target: &ast::Expr,
    ) -> Result<Operand> {
        match &target.kind {
            E::Ident(name) => {
                self.ide_note_member_use(target.span, ty, *name);
                let whole = Operand::Place {
                    ty,
                    addr: base,
                };
                let field = self.member_access(f, scope, whole, *name, target.span)?;
                self.ide_note_member_ref(target.span, *name, field.ty());
                Ok(field)
            }
            E::Member(inner, member) => {
                let inner = self.literal_target(f, scope, ty, base, inner)?;
                self.ide_note_member_use(member.span, inner.ty(), member.name);
                let field = self.member_access(f, scope, inner, member.name, member.span)?;
                self.ide_note_member_ref(member.span, member.name, field.ty());
                Ok(field)
            }
            E::Index(inner, index) => {
                let inner = self.literal_target(f, scope, ty, base, inner)?;
                let index_op = self.check_expr(f, scope, index, Some(TypeId::S64))?;
                self.index_operand(f, scope, inner, index_op, index.span, target.span)
            }
            _ => err(
                target.span,
                "a literal field target must be a member or index path",
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn check_array_literal(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        ty_expr: Option<&ast::Expr>,
        elems: &[ast::Expr],
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        let elem_ty = match ty_expr {
            Some(t) => Some(self.eval_type_in(f, scope, t)?),
            None => match expected.map(|e| self.types.kind(self.types.repr(e)).clone()) {
                Some(TypeKind::Array {
                    elem, ..
                }) => Some(elem),
                _ => None,
            },
        };
        let mut ops = Vec::new();
        let mut elem_ty = elem_ty;
        for e in elems {
            let op = self.check_expr(f, scope, e, elem_ty)?;
            if elem_ty.is_none() {
                let settled = self.settle_untyped(op.clone(), None);
                elem_ty = Some(match &settled {
                    Operand::Procs(p) if p.len() == 1 => self.proc_type(p[0], e.span)?,
                    other => other.ty(),
                });
            }
            let op = self.convert(f, op, elem_ty.unwrap(), e.span)?;
            ops.push((op, e.span));
        }
        let Some(elem) = elem_ty else {
            return err(
                span,
                "cannot infer the element type of an empty array literal",
            );
        };
        let n = ops.len() as u64;
        let mut ty = self.types.array(elem, ArrayKind::Fixed(n));
        // `g: Grid3i = .[1, 4, 9]`: a literal takes a `#type,distinct` variant of its own type.
        if ty_expr.is_none()
            && let Some(e) = expected
            && matches!(self.types.kind(e), TypeKind::Distinct(_))
            && self.types.repr(e) == ty
        {
            ty = e;
        }
        let esize = self.size_of(elem, span)?;
        if ops
            .iter()
            .all(|(op, _)| op.is_const() || matches!(op, Operand::Procs(p) if p.len() == 1))
        {
            let mut agg = Aggregate::zeroed(esize.saturating_mul(n), span)?;
            for (i, (op, vspan)) in ops.iter().enumerate() {
                let value = match op {
                    Operand::Procs(p) => Value::Proc(p[0]),
                    other => other.const_value().unwrap(),
                };
                self.write_value(&mut agg, i as u64 * esize, &value, elem, *vspan)?;
            }
            return Ok(Operand::Const {
                ty,
                value: Value::Bytes(Rc::new(agg)),
                untyped: false,
            });
        }
        let align = self.align_of(elem, span)?;
        let tmp = f.b.alloca((esize * n).max(1), align);
        for (i, (op, vspan)) in ops.into_iter().enumerate() {
            let at = f.b.ptr_offset(tmp, i as u64 * esize);
            let (_, v) = self.rvalue(f, op, vspan)?;
            self.store_value(f, elem, at, v, vspan)?;
        }
        Ok(Operand::Value {
            ty,
            val: tmp,
        })
    }

    // Constant images

    /// Write a constant `value` of type `ty` into `agg` at `offset`.
    pub fn write_value(
        &mut self,
        agg: &mut Aggregate,
        offset: u64,
        value: &Value,
        ty: TypeId,
        span: Span,
    ) -> Result<()> {
        let size = self.size_of(ty, span)?;
        let o = offset as usize;
        let r = self.types.repr(ty);
        // Overwriting (a literal field over the struct's default): drop the old pointers there.
        agg.relocs
            .retain(|reloc| reloc.offset < offset || reloc.offset >= offset + size);
        match value {
            Value::Int(i) => {
                if self.types.is_float(r) {
                    return self.write_value(agg, offset, &Value::float(*i as f64), ty, span);
                }
                let bytes = (*i as u128).to_le_bytes();
                agg.bytes[o..o + size as usize].copy_from_slice(&bytes[..size as usize]);
            }
            Value::Float(x, _) => {
                if r == TypeId::F32 {
                    agg.bytes[o..o + 4].copy_from_slice(&(*x as f32).to_le_bytes());
                } else {
                    agg.bytes[o..o + 8].copy_from_slice(&x.to_le_bytes());
                }
            }
            Value::Bool(b) => agg.bytes[o] = *b as u8,
            // `null` over a default (a union member sharing a string's storage) clears it.
            Value::Null => agg.bytes[o..o + size as usize].fill(0),
            Value::Void => {}
            Value::String(s) => {
                if r != TypeId::STRING {
                    return err(
                        span,
                        format!("string constant cannot initialize {}", self.types.name(ty)),
                    );
                }
                agg.bytes[o..o + 8].copy_from_slice(&(s.len() as u64).to_le_bytes());
                if !s.is_empty() {
                    let g = self.string_global(s);
                    agg.relocs.push(ir::Reloc {
                        offset: offset + 8,
                        target: ir::RelocTarget::Global(g),
                        addend: 0,
                    });
                }
            }
            Value::Type(t) => {
                let g = self.type_info_global(*t, span)?;
                agg.relocs.push(ir::Reloc {
                    offset,
                    target: ir::RelocTarget::Global(g),
                    addend: 0,
                });
            }
            Value::Proc(p) => {
                let target = match self.proc_func(*p, span)? {
                    procs::ProcTarget::Func(id) => ir::RelocTarget::Func(id),
                    procs::ProcTarget::Foreign(id) => ir::RelocTarget::Foreign(id),
                };
                agg.relocs.push(ir::Reloc {
                    offset,
                    target,
                    addend: 0,
                });
            }
            Value::Bytes(inner) => write_agg(agg, offset, inner),
            Value::Code(c) => agg.bytes[o..o + 8].copy_from_slice(&(c.0 as u64).to_le_bytes()),
        }
        Ok(())
    }
}

/// The image being built in `slot`, allocated (zeroed, `size` bytes) on first use.
pub(super) fn image_of(
    slot: &mut Option<Aggregate>,
    size: u64,
    span: Span,
) -> Result<&mut Aggregate> {
    if slot.is_none() {
        *slot = Some(Aggregate::zeroed(size, span)?);
    }
    Ok(slot.as_mut().unwrap())
}

/// Copy a nested aggregate image into `agg` at `offset`.
pub fn write_agg(agg: &mut Aggregate, offset: u64, inner: &Aggregate) {
    let o = offset as usize;
    agg.bytes[o..o + inner.bytes.len()].copy_from_slice(&inner.bytes);
    agg.relocs
        .retain(|r| r.offset < offset || r.offset >= offset + inner.bytes.len() as u64);
    for r in &inner.relocs {
        agg.relocs.push(ir::Reloc {
            offset: r.offset + offset,
            target: r.target,
            addend: r.addend,
        });
    }
}

pub(super) fn const_operand(value: Value, ty: TypeId) -> Operand {
    match value {
        Value::Type(t) => Operand::Type(t),
        value => Operand::Const {
            ty,
            value,
            untyped: false,
        },
    }
}
