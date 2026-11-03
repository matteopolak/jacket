//! Execute the sidecar's private integer slice, retaining no executable compiler IR.
use super::HashMap;
use jaic::ide::lifetimes::LifetimeControls;
use jaic::ir::{BinOp, CmpOp, Inst, Val};

pub(super) struct Controls<'a> {
    code: &'a LifetimeControls,
    slots: HashMap<Val, usize>,
    values: HashMap<Val, usize>,
}

impl<'a> Controls<'a> {
    pub fn new(code: &'a LifetimeControls) -> Self {
        Self {
            code,
            slots: code
                .slots
                .iter()
                .copied()
                .enumerate()
                .map(|(i, v)| (v, i))
                .collect(),
            values: code
                .values
                .iter()
                .copied()
                .enumerate()
                .map(|(i, v)| (v, i + code.slots.len()))
                .collect(),
        }
    }

    pub fn initial(&self) -> Vec<Option<u64>> {
        let mut state = vec![None; self.slots.len() + self.values.len()];
        for &(v, n) in &self.code.constants {
            state[self.values[&v]] = Some(n);
        }
        state
    }

    pub fn condition(&self, v: Val, state: &[Option<u64>]) -> Option<bool> {
        state.get(*self.values.get(&v)?)?.map(|n| n != 0)
    }

    pub fn numeric_condition(&self, v: Val) -> bool {
        self.code
            .blocks
            .iter()
            .flatten()
            .any(|i| matches!(i, Inst::Cmp { dst, .. } if *dst == v))
    }

    pub fn apply(&self, block: usize, state: &mut [Option<u64>]) {
        let get = |v: &Val, state: &[Option<u64>]| self.values.get(v).and_then(|&i| state[i]);
        for inst in &self.code.blocks[block] {
            match inst {
                Inst::Load {
                    dst,
                    addr,
                    ..
                } => state[self.values[dst]] = state[self.slots[addr]],
                Inst::Store {
                    addr,
                    value,
                    ..
                } => state[self.slots[addr]] = get(value, state),
                Inst::Bin {
                    dst,
                    op,
                    a,
                    b,
                    ..
                } => {
                    state[self.values[dst]] =
                        get(a, state)
                            .zip(get(b, state))
                            .and_then(|(a, b)| match op {
                                BinOp::Add => Some(a.wrapping_add(b)),
                                BinOp::Sub => Some(a.wrapping_sub(b)),
                                BinOp::Mul => Some(a.wrapping_mul(b)),
                                BinOp::And => Some(a & b),
                                BinOp::Or => Some(a | b),
                                BinOp::Xor => Some(a ^ b),
                                _ => None,
                            });
                }
                Inst::Cmp {
                    dst,
                    op,
                    a,
                    b,
                    ..
                } => {
                    state[self.values[dst]] =
                        get(a, state).zip(get(b, state)).and_then(|(a, b)| {
                            Some(u64::from(match op {
                                CmpOp::Eq => a == b,
                                CmpOp::Ne => a != b,
                                CmpOp::SLt => (a as i64) < (b as i64),
                                CmpOp::SLe => (a as i64) <= (b as i64),
                                CmpOp::SGt => (a as i64) > (b as i64),
                                CmpOp::SGe => (a as i64) >= (b as i64),
                                CmpOp::ULt => a < b,
                                CmpOp::ULe => a <= b,
                                CmpOp::UGt => a > b,
                                CmpOp::UGe => a >= b,
                                _ => return None,
                            }))
                        });
                }
                _ => unreachable!(),
            }
        }
    }
}
