//! Pointer arithmetic and pointer masking.
use crate::ast::BinOp;
use crate::ir::Ty;
use crate::sema::convert::Site;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::{Compiler, Result, Span, TypeId, ast, ir};

impl Compiler {
    /// `p + n`, `p - n` and `p - q` on a pointer `p`.
    pub(super) fn pointer_offset(
        &mut self,
        f: &mut FnCtx,
        op: BinOp,
        lhs: Operand,
        rhs: Operand,
        span: Span,
    ) -> Result<Operand> {
        let (lty, rty) = (lhs.ty(), rhs.ty());
        let (_, base) = self.rvalue(f, lhs, span)?;
        if op == BinOp::Sub && self.ty.types.is_pointer(rty) {
            let (_, other) = self.rvalue(f, rhs, span)?;
            let pointee = self.ty.types.pointee(lty).unwrap();
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
        let rhs = if self.ty.types.is_integer(rty) && !matches!(rhs, Operand::Const { .. }) {
            self.explicit_cast(f, rhs, TypeId::S64, ast::CastFlags::default(), span)?
        } else {
            self.convert(f, rhs, TypeId::S64, Site::opaque(span))?
        };
        let (_, idx) = self.rvalue(f, rhs, span)?;
        let pointee = self.ty.types.pointee(lty).unwrap();
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
        Ok(Operand::Value {
            ty: lty,
            val: f.b.ptr_add(base, off),
        })
    }

    /// Masking an address (`p & (alignment - 1)`; Jai reads `cast(u64) p & MASK` as
    /// `cast(u64) (p & MASK)`): the integer meets the pointer's bits, and the result keeps the
    /// pointer's type.
    pub(super) fn mask_pointer(
        &mut self,
        f: &mut FnCtx,
        op: BinOp,
        lhs: Operand,
        rhs: Operand,
        span: Span,
    ) -> Result<Operand> {
        let (ptr, int) = if self.ty.types.is_pointer(lhs.ty()) {
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
        Ok(Operand::Value {
            ty: ptr_ty,
            val: f.b.conv(ir::ConvOp::Bitcast, Ty::I64, Ty::Ptr, combined),
        })
    }
}
