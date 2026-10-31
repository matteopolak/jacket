//! Which struct types count as instances of a polymorphic struct or as having a given type as
//! an `#as` base, for `$T/Struct` restrictions and bare polymorphic struct parameters.

use super::{Compiler, E, Operand, Result, ScopeId, Span, TypeId, ast, value};

impl Compiler {
    /// Whether struct `ty` is `want` or reaches it through `#as` members.
    pub(super) fn has_as_base(&mut self, ty: TypeId, want: TypeId) -> bool {
        if ty == want {
            return true;
        }
        let Some(s) = self.ty.types.as_struct(ty) else {
            return false;
        };
        if self.layout_struct(s, Span::NONE).is_err() {
            return false;
        }
        let fields = self.ty.types.struct_info(s).fields.clone();
        fields
            .iter()
            .filter(|f| f.as_)
            .any(|f| self.has_as_base(f.ty, want))
            || self
                .using_constant_bases(s, true)
                .into_iter()
                .any(|base| self.has_as_base(base, want))
    }

    /// The types of the struct's `using name :: value;` constants (only the `#as` ones when
    /// `as_only`): members without storage that still lend the struct their own constants and,
    /// with `#as`, make it count as an instance of the type they hold.
    pub(crate) fn using_constant_bases(
        &mut self,
        s: crate::types::StructId,
        as_only: bool,
    ) -> Vec<TypeId> {
        let Some(src) = self.ty.struct_asts.get(&s).cloned() else {
            return Vec::new();
        };
        let mut bases = Vec::new();
        for stmt in &src.lit.body {
            if let ast::StmtKind::Decl(decl) = &stmt.kind
                && decl.kind == ast::DeclKind::Const
                && decl.using
                && (decl.as_ || !as_only)
                && let Some(value) = &decl.value
                && let Ok(operand) = self.eval_const(src.scope, value, None)
                && let Some(base) = match operand {
                    Operand::Const {
                        ty, ..
                    } => Some(ty),
                    _ => None,
                }
            {
                bases.push(base);
            }
        }
        bases
    }

    /// The polymorphic struct a type expression names, if it is a bare identifier for one.
    pub(super) fn ident_poly_struct_expr(
        &mut self,
        scope: ScopeId,
        expr: &ast::Expr,
    ) -> Result<Option<value::PolyStructId>> {
        match &expr.kind {
            E::Ident(name) => self.ident_poly_struct(scope, *name),
            _ => Ok(None),
        }
    }

    pub(super) fn instance_or_as_base(
        &mut self,
        ps: value::PolyStructId,
        ty: TypeId,
    ) -> Option<TypeId> {
        // A `#bake_arguments` struct's instances are its origin's with the baked values.
        let poly = &self.ty.poly_structs[ps.0 as usize];
        let (origin, baked) = match poly.origin {
            Some(origin) => (origin, poly.baked.clone()),
            None => (ps, Vec::new()),
        };
        if self.ty.poly_structs[origin.0 as usize]
            .instances
            .values()
            .any(|&t| t == ty)
            && (baked.is_empty() || {
                let bindings = self.poly_struct_bindings(self.ty.types.as_struct(ty)?);
                baked.iter().all(|(name, value, _)| {
                    bindings.iter().any(|(n, v, _)| n == name && v == value)
                })
            })
        {
            return Some(ty);
        }
        let s = self.ty.types.as_struct(ty)?;
        self.layout_struct(s, Span::NONE).ok()?;
        let fields = self.ty.types.struct_info(s).fields.clone();
        if let Some(found) = fields
            .iter()
            .filter(|f| f.as_)
            .find_map(|f| self.instance_or_as_base(ps, f.ty))
        {
            return Some(found);
        }
        self.using_constant_bases(s, true)
            .into_iter()
            .find_map(|base| self.instance_or_as_base(ps, base))
    }
}
