//! IR signatures and aggregate layouts for procedures.
use crate::ir::{Conv, Sig, Ty};
use crate::sema::*;
use crate::types::{ArrayKind, TypeKind};

impl Compiler {
    /// Lowered IR signature for a procedure type.
    pub fn ir_sig(&mut self, ty: TypeId, span: Span) -> Result<Sig> {
        let TypeKind::Proc(p) = self.types.kind(ty).clone() else {
            return err(
                span,
                format!("{} is not a procedure type", self.types.name(ty)),
            );
        };
        let mut params = Vec::new();
        if !p.no_context {
            params.push(Ty::Ptr);
        }
        let mut c_abi = ir::CAbi::default();
        if !p.no_context {
            c_abi.params.push(None);
        }
        for &t in &p.params {
            // Aggregates have no scalar type: they are passed by address.
            params.push(self.ir_ty(t).unwrap_or(Ty::Ptr));
            c_abi.params.push(if p.c_call && self.is_memory_type(t) {
                Some(self.agg_layout(t, span)?)
            } else {
                None
            });
        }
        let mut returns = Vec::new();
        for (i, &t) in p.returns.iter().enumerate() {
            if let Some(rt) = self.ir_ty(t) {
                returns.push(rt);
            } else {
                params.push(Ty::Ptr);
                if p.c_call && i == 0 {
                    c_abi.ret = Some(self.agg_layout(t, span)?);
                    c_abi.ret_indirect = p.non_pod_return;
                }
                c_abi.params.push(None);
            }
        }
        let needs_abi =
            p.c_call && (c_abi.ret.is_some() || c_abi.params.iter().any(Option::is_some));
        let c_fixed = params.len() as u32;
        Ok(Sig {
            params,
            returns,
            conv: if p.c_call {
                Conv::C
            } else {
                Conv::Jai
            },
            c_varargs: p.c_varargs,
            c_fixed,
            c_abi: needs_abi.then(|| Box::new(c_abi)),
        })
    }

    /// Flattened scalar layout of an aggregate for C ABI classification.
    pub fn agg_layout(&mut self, ty: TypeId, span: Span) -> Result<ir::AggLayout> {
        let size = self.size_of(ty, span)?;
        let align = self.align_of(ty, span)?;
        let mut fields = Vec::new();
        self.flatten_fields(ty, 0, &mut fields, span)?;
        // Members placed over each other (`Vector2`'s `component: [2] float` over `x, y`) are one
        // set of bytes: classifying each view as a field of its own turns a two-float
        // aggregate into four and loses the register class C expects.
        let mut seen = Vec::new();
        fields.retain(|field| {
            let new = !seen.contains(field);
            seen.push(*field);
            new
        });
        Ok(ir::AggLayout {
            size,
            align,
            fields,
        })
    }

    fn flatten_fields(
        &mut self,
        ty: TypeId,
        base: u64,
        out: &mut Vec<(u64, Ty)>,
        span: Span,
    ) -> Result<()> {
        if let Some(t) = self.ir_ty(ty).or_else(|| self.wide_abi_class(ty)) {
            out.push((base, t));
            return Ok(());
        }
        match self.types.kind(ty).clone() {
            TypeKind::Struct(s) => {
                self.layout_struct(s, span)?;
                let fields = self.types.struct_info(s).fields.clone();
                for field in fields.iter() {
                    self.flatten_fields(field.ty, base + field.offset, out, span)?;
                }
            }
            TypeKind::Array {
                elem,
                kind: ArrayKind::Fixed(n),
            } => {
                let size = self.size_of(elem, span)?;
                for i in 0..n {
                    self.flatten_fields(elem, base + i * size, out, span)?;
                }
            }
            TypeKind::String
            | TypeKind::Array {
                kind: ArrayKind::View,
                ..
            } => {
                out.push((base, Ty::I64));
                out.push((base + 8, Ty::Ptr));
            }
            TypeKind::Any => {
                out.push((base, Ty::Ptr));
                out.push((base + 8, Ty::Ptr));
            }
            TypeKind::Array {
                kind: ArrayKind::Resizable,
                ..
            } => {
                for i in 0..5 {
                    out.push((base + i * 8, Ty::I64));
                }
            }
            _ => {}
        }
        Ok(())
    }
}
