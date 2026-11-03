//! Integer representations of pointers remain available after executable IR is released.
use crate::ir::{self, Val};

#[derive(Clone, Copy, Debug)]
pub struct LifetimeAddressCast {
    pub value: Val,
    pub source: Val,
    pub to_pointer: bool,
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
            let integer = |ty| matches!(ty, ir::Ty::I32 | ir::Ty::I64);
            ((from == ir::Ty::Ptr && integer(to)) || (to == ir::Ty::Ptr && integer(from)))
                .then_some(LifetimeAddressCast {
                    value: dst,
                    source: src,
                    to_pointer: to == ir::Ty::Ptr,
                })
        })
        .collect()
}
