//! A backward slice of integer branch operands, excluding aliased memory.
use super::{HashMap, HashSet, Val, ir};

#[derive(Clone, Debug, Default)]
pub struct LifetimeControls {
    pub slots: Vec<Val>,
    pub values: Vec<Val>,
    pub constants: Vec<(Val, u64)>,
    pub blocks: Vec<Vec<ir::Inst>>,
}

pub(super) fn capture(func: &ir::Func) -> LifetimeControls {
    let mut addresses = HashMap::<ir::SlotId, Vec<Val>>::default();
    for inst in func.blocks.iter().flat_map(|b| &b.insts) {
        if let ir::Inst::SlotAddr {
            dst,
            slot,
        } = inst
            && func.slots[slot.0 as usize].size == 8
        {
            addresses.entry(*slot).or_default().push(*dst);
        }
    }
    let mut private: HashSet<_> = addresses
        .values()
        .filter_map(|v| match v.as_slice() {
            [v] => Some(*v),
            _ => None,
        })
        .collect();
    for block in &func.blocks {
        for inst in &block.insts {
            match inst {
                ir::Inst::Load {
                    ty: ir::Ty::I64, ..
                } => {}
                ir::Inst::Store {
                    ty: ir::Ty::I64,
                    value,
                    ..
                } => {
                    private.remove(value);
                }
                _ => ir::inst_vals(inst, &mut |_| {}, &mut |v| {
                    private.remove(&v);
                }),
            }
        }
        ir::term_uses(&block.term, &mut |v| {
            private.remove(&v);
        });
    }
    let mut needed: HashSet<_> = func
        .blocks
        .iter()
        .filter_map(|b| match b.term {
            ir::Term::Branch {
                cond, ..
            } => Some(cond),
            _ => None,
        })
        .collect();
    let mut slots = HashSet::default();
    loop {
        let old = (needed.len(), slots.len());
        for inst in func.blocks.iter().flat_map(|b| &b.insts) {
            match inst {
                ir::Inst::Load {
                    dst,
                    addr,
                    ty: ir::Ty::I64,
                } if needed.contains(dst) && private.contains(addr) => {
                    slots.insert(*addr);
                }
                ir::Inst::Store {
                    addr,
                    value,
                    ty: ir::Ty::I64,
                } if slots.contains(addr) => {
                    needed.insert(*value);
                }
                ir::Inst::Bin {
                    dst,
                    a,
                    b,
                    ty: ir::Ty::I64,
                    ..
                }
                | ir::Inst::Cmp {
                    dst,
                    a,
                    b,
                    ty: ir::Ty::I64,
                    ..
                } if needed.contains(dst) => {
                    needed.insert(*a);
                    needed.insert(*b);
                }
                _ => {}
            }
        }
        if old == (needed.len(), slots.len()) {
            break;
        }
    }
    if slots.is_empty() || needed.len() > 64 {
        return LifetimeControls::default();
    }
    let mut out = LifetimeControls {
        slots: slots.iter().copied().collect(),
        values: needed.iter().copied().collect(),
        ..LifetimeControls::default()
    };
    out.slots.sort_unstable_by_key(|v| v.0);
    out.values.sort_unstable_by_key(|v| v.0);
    for block in &func.blocks {
        let mut insts = Vec::new();
        for inst in &block.insts {
            let keep = match inst {
                ir::Inst::IConst {
                    dst,
                    value,
                    ty: ir::Ty::I64 | ir::Ty::I8,
                } if needed.contains(dst) => {
                    out.constants.push((*dst, *value));
                    false
                }
                ir::Inst::Load {
                    dst,
                    addr,
                    ty: ir::Ty::I64,
                } => needed.contains(dst) && slots.contains(addr),
                ir::Inst::Store {
                    addr,
                    ty: ir::Ty::I64,
                    ..
                } => slots.contains(addr),
                ir::Inst::Bin {
                    dst,
                    ty: ir::Ty::I64,
                    ..
                }
                | ir::Inst::Cmp {
                    dst,
                    ty: ir::Ty::I64,
                    ..
                } => needed.contains(dst),
                _ => false,
            };
            if keep {
                insts.push(inst.clone());
            }
        }
        out.blocks.push(insts);
    }
    out
}

impl LifetimeControls {
    pub(super) fn vector_bytes(&self) -> usize {
        (self.slots.capacity() + self.values.capacity()) * std::mem::size_of::<Val>()
            + self.constants.capacity() * std::mem::size_of::<(Val, u64)>()
            + self.blocks.capacity() * std::mem::size_of::<Vec<ir::Inst>>()
            + self
                .blocks
                .iter()
                .map(|b| b.capacity() * std::mem::size_of::<ir::Inst>())
                .sum::<usize>()
    }
}
