//! Immutable private predicates shared by repeated lifetime branch tests.
use super::{HashMap, LifetimeBlock, LifetimeEvent, Val, ir, private_byte_flags};

pub(super) fn immutable_boolean_branches(
    func: &ir::Func,
    blocks: &[LifetimeBlock],
    constants: &HashMap<Val, u64>,
) -> Vec<(Val, Val, bool)> {
    let mut flags = private_byte_flags(func);
    flags.retain(|flag| {
        func.blocks
            .iter()
            .flat_map(|b| &b.insts)
            .filter(|i| matches!(i, ir::Inst::Store { addr, .. } if addr == flag))
            .count()
            == 1
    });
    let slots: HashMap<_, _> = flags
        .iter()
        .copied()
        .enumerate()
        .map(|(i, v)| (v, i))
        .collect();
    let mut out = Vec::new();
    for (block, ir) in blocks.iter().zip(&func.blocks) {
        let origins = boolean_load_origins(&ir.insts, &slots, constants);
        for event in &block.events {
            if let LifetimeEvent::Branch {
                condition, ..
            } = event
                && let Some(&(slot, inverted)) = origins.get(condition)
            {
                out.push((*condition, flags[slot], inverted));
            }
        }
    }
    out
}

/// The private slot and optional negation tested by a lowered boolean branch.
fn boolean_load_origins(
    insts: &[ir::Inst],
    slots: &HashMap<Val, usize>,
    constants: &HashMap<Val, u64>,
) -> HashMap<Val, (usize, bool)> {
    let mut origins = HashMap::default();
    for inst in insts {
        match inst {
            ir::Inst::Store {
                addr, ..
            } => {
                if let Some(slot) = slots.get(addr) {
                    origins.retain(|_, (s, _)| s != slot);
                }
            }
            ir::Inst::Load {
                addr,
                dst,
                ..
            } => {
                if let Some(&slot) = slots.get(addr) {
                    origins.insert(*dst, (slot, false));
                }
            }
            ir::Inst::Bin {
                dst,
                op: ir::BinOp::Xor,
                ty: ir::Ty::I8,
                a,
                b,
            } => {
                let input = if constants.get(a) == Some(&1) {
                    origins.get(b)
                } else if constants.get(b) == Some(&1) {
                    origins.get(a)
                } else {
                    None
                };
                if let Some(&(slot, inverted)) = input {
                    origins.insert(*dst, (slot, !inverted));
                }
            }
            ir::Inst::Cmp {
                dst,
                op: ir::CmpOp::Eq | ir::CmpOp::Ne,
                a,
                b,
                ..
            } => {
                let input = if constants.get(a) == Some(&0) {
                    origins.get(b)
                } else if constants.get(b) == Some(&0) {
                    origins.get(a)
                } else {
                    None
                };
                if let Some(&(slot, inverted)) = input {
                    let negated = matches!(
                        inst,
                        ir::Inst::Cmp {
                            op: ir::CmpOp::Eq,
                            ..
                        }
                    );
                    origins.insert(*dst, (slot, inverted ^ negated));
                }
            }
            _ => {}
        }
    }
    origins
}
