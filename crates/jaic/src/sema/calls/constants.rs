//! Deciding whether an argument is a constant expression.
use super::const_marker;
use crate::ast::ExprKind as E;
use crate::sema::lower::Operand;
use crate::sema::scope::EntityKind;
use crate::sema::*;
use crate::types::{ArrayKind, TypeKind};

impl Compiler {
    /// Constant expressions that have no `Value` to bake (addresses of globals, `.data` of
    /// constants, `type_info(T)`...). `is_constant` is true for them; a `$$` parameter
    /// receiving one stays a runtime parameter, marked constant.
    pub(in crate::sema) fn is_constant_pointer(
        &mut self,
        scope: ScopeId,
        expr: &ast::Expr,
        op: &Operand,
    ) -> bool {
        match &expr.kind {
            E::Ident(name) => self
                .lookup(scope, const_marker(*name))
                .is_ok_and(|v| !v.is_empty()),
            E::Member(base, member) => match member.name.as_str() {
                "data" if self.types.is_pointer(op.ty()) => {
                    let Ok(b) = self.check_expr_no_emit(scope, base) else {
                        return false;
                    };
                    let fixed = matches!(
                        self.types.kind(self.types.repr(b.ty())),
                        TypeKind::Array {
                            kind: ArrayKind::Fixed(_),
                            ..
                        }
                    );
                    b.is_const() || (fixed && self.is_global_var(scope, base))
                }
                // `type_info(T).type` / `.runtime_size` are constant.
                "type" | "runtime_size" => {
                    matches!(&base.kind, E::Call { callee, .. }
                        if matches!(&callee.kind, E::Ident(n) if n.as_str() == "type_info"))
                }
                _ => false,
            },
            E::Unary(ast::UnOp::Star, inner) => self.is_global_var(scope, inner),
            E::Cast {
                value, ..
            } if self.types.is_pointer(op.ty()) => self.is_constant_expr(scope, value),
            E::Binary(ast::BinOp::Add | ast::BinOp::Sub, a, b)
                if self.types.is_pointer(op.ty()) =>
            {
                self.is_constant_expr(scope, a) && self.is_constant_expr(scope, b)
            }
            E::Index(base, index) => {
                let Ok(b) = self.check_expr_no_emit(scope, base) else {
                    return false;
                };
                matches!(
                    self.types.kind(self.types.repr(b.ty())),
                    TypeKind::Array { .. } | TypeKind::String
                ) && self.is_constant_expr(scope, base)
                    && self.is_constant_expr(scope, index)
            }
            E::Call {
                callee, ..
            } => matches!(&callee.kind, E::Ident(n)
                if matches!(n.as_str(), "type_info" | "initializer_of")),
            _ => false,
        }
    }

    fn is_constant_expr(&mut self, scope: ScopeId, expr: &ast::Expr) -> bool {
        let Ok(op) = self.check_expr_no_emit(scope, expr) else {
            return false;
        };
        op.is_const()
            || matches!(op, Operand::Procs(_))
            || self.is_constant_pointer(scope, expr, &op)
    }

    /// Is `expr` the name of a global variable (not a local or constant)?
    fn is_global_var(&mut self, scope: ScopeId, expr: &ast::Expr) -> bool {
        let E::Ident(name) = &expr.kind else {
            return false;
        };
        let Ok(ids) = self.lookup(scope, *name) else {
            return false;
        };
        matches!(ids[..], [id] if matches!(
            &self.entity(id).kind,
            EntityKind::Decl { decl, .. } if decl.kind == ast::DeclKind::Var
        ))
    }

    /// A `*Struct` argument passed to a by-value `Struct` parameter is dereferenced.
    pub(super) fn auto_deref_arg(&self, from: TypeId, param: TypeId) -> bool {
        self.types.pointee(from) == Some(param) && self.types.as_struct(param).is_some()
    }
}
