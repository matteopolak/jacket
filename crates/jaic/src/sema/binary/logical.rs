//! Logical operators and string comparison.
use crate::ast::BinOp;
use crate::ir::{CmpOp, Ty};
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::*;

impl Compiler {
    pub(in crate::sema) fn check_logical(
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

    pub(in crate::sema) fn string_compare(
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
}
