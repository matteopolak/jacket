//! Integer representations of pointers remain available after executable IR is released.
use crate::ir::{self, Val};

#[derive(Clone, Copy, Debug)]
pub struct LifetimeAddressCast {
    pub value: Val,
    pub source: Val,
    pub to_pointer: bool,
    /// Integer-side width of this exact IR bit conversion, not a range proof.
    pub integer_bytes: u8,
}

pub(super) fn capture(function: &ir::Func) -> Vec<LifetimeAddressCast> {
    function
        .blocks
        .iter()
        .flat_map(|b| &b.insts)
        .filter_map(|inst| {
            let ir::Inst::Conv {
                dst,
                src,
                op: ir::ConvOp::Bitcast,
                from,
                to,
            } = *inst
            else {
                return None;
            };
            let integer_bytes = |ty| match ty {
                ir::Ty::I32 => Some(4),
                ir::Ty::I64 => Some(8),
                _ => None,
            };
            let (to_pointer, integer_bytes) = if from == ir::Ty::Ptr {
                (false, integer_bytes(to)?)
            } else if to == ir::Ty::Ptr {
                (true, integer_bytes(from)?)
            } else {
                return None;
            };
            Some(LifetimeAddressCast {
                value: dst,
                source: src,
                to_pointer,
                integer_bytes,
            })
        })
        .collect()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_cast_width_distinguishes_exact_integer_sides() {
        let mut builder = ir::Builder::new(
            "cast_widths".into(),
            ir::Sig {
                params: vec![ir::Ty::Ptr, ir::Ty::I32, ir::Ty::I64],
                returns: vec![],
                conv: ir::Conv::Jai,
                c_varargs: false,
                c_fixed: 0,
                c_abi: None,
            },
        );
        let mut expected = Vec::new();
        for (from, to, source, bytes) in [
            (ir::Ty::Ptr, ir::Ty::I32, builder.param(0), 4),
            (ir::Ty::Ptr, ir::Ty::I64, builder.param(0), 8),
            (ir::Ty::I32, ir::Ty::Ptr, builder.param(1), 4),
            (ir::Ty::I64, ir::Ty::Ptr, builder.param(2), 8),
        ] {
            let dst = builder.new_val(to);
            builder.push(ir::Inst::Conv {
                dst,
                src: source,
                op: ir::ConvOp::Bitcast,
                from,
                to,
            });
            expected.push((dst, source, to == ir::Ty::Ptr, bytes));
        }
        let retained: Vec<_> = capture(&builder.func)
            .into_iter()
            .map(|c| (c.value, c.source, c.to_pointer, c.integer_bytes))
            .collect();
        assert_eq!(retained, expected);
    }
}
