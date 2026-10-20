//! Coercing operands to a target type.
use super::numeric_conv;
use crate::ir::Ty;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::{Compiler, Result, Span, TypeId, Value, err, ir};
use crate::types::{ArrayKind, TypeKind};

impl Compiler {
    /// Perform an allowed conversion at runtime.
    pub(super) fn coerce(
        &mut self,
        f: &mut FnCtx,
        op: Operand,
        to: TypeId,
        span: Span,
    ) -> Result<Operand> {
        let from = op.ty();
        if to == TypeId::ANY {
            return self.box_any(f, op, span);
        }
        if let Some(converted) = self.wide_cast(f, op.clone(), to, span)? {
            return Ok(converted);
        }
        let op = if matches!(
            op,
            Operand::Const {
                untyped: true,
                ..
            }
        ) {
            self.settle_untyped(op, Some(to))
        } else {
            op
        };
        let from = if op.ty() == from {
            from
        } else {
            op.ty()
        };
        let fk = self.types.kind(from).clone();
        let tk = self.types.kind(to).clone();
        match (&fk, &tk) {
            (
                TypeKind::Array {
                    kind: ArrayKind::Fixed(n),
                    ..
                },
                TypeKind::Array {
                    kind: ArrayKind::View,
                    ..
                },
            ) => {
                let n = *n;
                // A constant literal viewed as `[] T` lives in its own writable global, as in jai:
                // the view outlives the procedure (`return .["a", "b"];`) and may be written.
                let addr = match &op {
                    Operand::Const {
                        value: Value::Bytes(agg),
                        ..
                    } if n > 0 => {
                        let size = self.size_of(from, span)?;
                        let align = self.align_of(from, span)?;
                        let name = format!("array_literal.{}", self.out.program.globals.len());
                        let global = self.out.program.add_global(ir::Global {
                            name,
                            size,
                            align,
                            init: agg.bytes.clone(),
                            relocs: agg.relocs.clone(),
                            read_only: false,
                            export: None,
                        });
                        f.b.global_addr(global)
                    }
                    _ => self.address_of(f, op, span)?.1,
                };
                // An empty array views no storage (`.[]` is a null view, not a dangling address).
                let addr = if n == 0 {
                    f.b.iconst(Ty::Ptr, 0)
                } else {
                    addr
                };
                let view = f.b.alloca(16, 8);
                let count = f.b.iconst(Ty::I64, n);
                f.b.store(Ty::I64, view, count);
                let data = f.b.ptr_offset(view, 8);
                f.b.store(Ty::Ptr, data, addr);
                Ok(Operand::Value {
                    ty: to,
                    val: view,
                })
            }
            (
                TypeKind::Array {
                    kind: ArrayKind::Resizable,
                    ..
                },
                TypeKind::Array {
                    kind: ArrayKind::View,
                    ..
                },
            ) => {
                // A resizable array starts with (count, data): reuse its prefix.
                let (_, addr) = self.address_of(f, op, span)?;
                Ok(Operand::Value {
                    ty: to,
                    val: addr,
                })
            }
            (
                TypeKind::Array {
                    kind: ArrayKind::Fixed(_),
                    ..
                },
                TypeKind::Pointer(_),
            ) => {
                let (_, addr) = self.address_of(f, op, span)?;
                Ok(Operand::Value {
                    ty: to,
                    val: addr,
                })
            }
            (TypeKind::Struct(_), _) => {
                let offset = self.as_offset(from, to).unwrap();
                let (_, addr) = self.address_of(f, op, span)?;
                let field = f.b.ptr_offset(addr, offset);
                Ok(Operand::Place {
                    ty: to,
                    addr: field,
                })
            }
            (TypeKind::Pointer(inner), TypeKind::Struct(_)) => {
                let offset = if *inner == to {
                    0
                } else {
                    self.as_offset(*inner, to).unwrap()
                };
                let (_, v) = self.rvalue(f, op, span)?;
                Ok(Operand::Place {
                    ty: to,
                    addr: f.b.ptr_offset(v, offset),
                })
            }
            (TypeKind::Pointer(inner), TypeKind::Pointer(target))
                if *target != TypeId::VOID
                    && *inner != TypeId::VOID
                    && self.as_offset(*inner, *target).is_none()
                    && self.as_offset(*inner, to).is_some() =>
            {
                // `*Wrapper` → its `#as` member's pointer value: load the member.
                let offset = self.as_offset(*inner, to).unwrap();
                let (_, v) = self.rvalue(f, op, span)?;
                Ok(Operand::Place {
                    ty: to,
                    addr: f.b.ptr_offset(v, offset),
                })
            }
            (TypeKind::Pointer(inner), TypeKind::Pointer(target))
                if *target != TypeId::VOID && *inner != TypeId::VOID =>
            {
                // Unrelated pointee types have no `#as` path between them: the cast keeps the
                // address as it is.
                let offset = self.as_offset(*inner, *target).unwrap_or(0);
                let (_, v) = self.rvalue(f, op, span)?;
                Ok(Operand::Value {
                    ty: to,
                    val: f.b.ptr_offset(v, offset),
                })
            }
            // `*Wrapper` → the value of its `#as` member (`#as id: u32`).
            (TypeKind::Pointer(inner), _)
                if self.types.as_struct(*inner).is_some()
                    && self.as_offset(*inner, to).is_some() =>
            {
                let offset = self.as_offset(*inner, to).unwrap();
                let (_, v) = self.rvalue(f, op, span)?;
                Ok(Operand::Place {
                    ty: to,
                    addr: f.b.ptr_offset(v, offset),
                })
            }
            _ => self.scalar_convert(f, op, to, span),
        }
    }

    /// Numeric/pointer conversion between scalar register classes.
    pub(super) fn scalar_convert(
        &mut self,
        f: &mut FnCtx,
        op: Operand,
        to: TypeId,
        span: Span,
    ) -> Result<Operand> {
        let from = op.ty();
        let (_, v) = self.rvalue(f, op, span)?;
        let (Some(ft), Some(tt)) = (self.ir_ty(from), self.ir_ty(to)) else {
            if self.types.repr(from) == self.types.repr(to) {
                return Ok(Operand::Value {
                    ty: to,
                    val: v,
                });
            }
            return err(
                span,
                format!(
                    "cannot convert {} to {}",
                    self.types.name(from),
                    self.types.name(to)
                ),
            );
        };
        let fsigned = self.types.int_info(from).is_some_and(|(_, s)| s);
        let tsigned = self.types.int_info(to).is_some_and(|(_, s)| s);
        let val = numeric_conv(f, v, ft, tt, fsigned, tsigned);
        Ok(Operand::Value {
            ty: to,
            val,
        })
    }
}
