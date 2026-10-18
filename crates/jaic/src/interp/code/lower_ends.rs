//! Lowering of block terminators, including fused compare-and-branch.
use super::build::{Builder, Rhs, op_dst};
use super::op::Op;
use crate::ir::{self, BinOp, CmpOp, Term, Ty};

impl Builder<'_> {
    /// Emit the ending of block `bi`. A jump to the block that follows is no op at all.
    pub(super) fn end(&mut self, bi: usize, block: &ir::Block) {
        let next = bi as u32 + 1;
        match block.term {
            Term::Jump(t) => {
                if t.0 != next {
                    self.push(
                        Op::Jump {
                            target: t.0,
                        },
                        None,
                    );
                }
            }
            Term::Branch {
                cond,
                then_block,
                else_block,
            } => {
                // Checks the front end already decided (`if false` around a trap).
                if let Some(x) = self.int(cond) {
                    self.fold(cond);
                    let t = if x & 0xff != 0 {
                        then_block.0
                    } else {
                        else_block.0
                    };
                    if t != next {
                        self.push(
                            Op::Jump {
                                target: t,
                            },
                            None,
                        );
                    }
                    return;
                }
                // Fall through to whichever successor comes next, if either does.
                let (sense, target, after) = if then_block.0 == next {
                    (false, else_block.0, None)
                } else if else_block.0 == next {
                    (true, then_block.0, None)
                } else {
                    (true, then_block.0, Some(else_block.0))
                };
                let cond = self.v(cond);
                let fused = self.fuse_compare(cond);
                let test = match fused {
                    Some((op @ (CmpOp::Ne | CmpOp::Eq), 0, a, Rhs::Imm(0))) => {
                        self.fuse_test(a).map(|(x, mask)| (op, x, mask))
                    }
                    _ => None,
                };
                let op = match fused {
                    Some(_) if test.is_some() => {
                        let (op, a, mask) = test.unwrap();
                        Op::BranchTest {
                            sense: sense == (op == CmpOp::Ne),
                            a,
                            mask,
                            target,
                        }
                    }
                    Some((op, shift, a, Rhs::Val(b))) => Op::BranchCmp {
                        op,
                        shift,
                        sense,
                        a,
                        b,
                        target,
                    },
                    Some((op, shift, a, Rhs::Imm(imm))) => Op::BranchCmpImm {
                        op,
                        shift,
                        sense,
                        a,
                        imm,
                        target,
                    },
                    None => Op::Branch {
                        cond,
                        sense,
                        target,
                    },
                };
                self.push(op, None);
                if let Some(other) = after {
                    self.push(
                        Op::Jump {
                            target: other,
                        },
                        None,
                    );
                }
            }
            Term::Switch {
                value, ..
            } => {
                self.v(value);
                self.push(
                    Op::Term {
                        block: bi as u32,
                    },
                    None,
                );
            }
            Term::Ret(ref values) => {
                for &v in values {
                    self.v(v);
                }
                let op = match values[..] {
                    [] => Op::Ret0,
                    [v] => Op::Ret1 {
                        src: v.0,
                    },
                    _ => {
                        let at = self.pool.len() as u32;
                        self.pool.extend(values.iter().map(|v| v.0));
                        Op::RetN {
                            at,
                            n: values.len() as u32,
                        }
                    }
                };
                self.push(op, None);
            }
            Term::Unreachable => self.push(
                Op::Term {
                    block: bi as u32,
                },
                None,
            ),
        }
    }

    /// The comparison `cond` is, when this block made it for its branch alone: its operands,
    /// to compare in the branch itself.
    fn fuse_compare(&mut self, cond: u32) -> Option<(CmpOp, u8, u32, Rhs)> {
        if self.uses[cond as usize] != 1 || self.defs[cond as usize] != 1 {
            return None;
        }
        let i = self.ops[self.block_start_mark..]
            .iter()
            .rposition(|op| op_dst(op) == Some(cond))?;
        let at = self.block_start_mark + i;
        let fused = match self.ops[at] {
            Op::Cmp {
                op,
                shift,
                a,
                b,
                ..
            } if self.defs[a as usize] == 1 && self.defs[b as usize] == 1 => {
                self.uses[a as usize] += 1;
                self.uses[b as usize] += 1;
                (op, shift, a, Rhs::Val(b))
            }
            Op::CmpImm {
                op,
                shift,
                a,
                imm,
                ..
            } if self.defs[a as usize] == 1 => {
                self.uses[a as usize] += 1;
                (op, shift, a, Rhs::Imm(imm))
            }
            // `!flag` of a byte: unequal to the 1 `flag` is compared with.
            Op::BinImm {
                op: BinOp::Xor,
                ty: Ty::I8,
                a,
                imm: 1,
                ..
            } if self.defs[a as usize] == 1 => {
                self.uses[a as usize] += 1;
                (CmpOp::Ne, 56, a, Rhs::Imm(1))
            }
            _ => return None,
        };
        // The comparison's own operand uses stay counted (it is dropped below only if nothing
        // else reads its result).
        self.uses[cond as usize] = 0;
        Some(fused)
    }
}

impl Builder<'_> {
    /// The `x & mask` that `reg` is, when this block made it for a fused compare against zero
    /// alone (`fuse_compare` counted that compare's and the branch's use of it).
    fn fuse_test(&mut self, reg: u32) -> Option<(u32, i32)> {
        if self.uses[reg as usize] != 2 || self.defs[reg as usize] != 1 {
            return None;
        }
        let i = self.ops[self.block_start_mark..]
            .iter()
            .rposition(|op| op_dst(op) == Some(reg))?;
        let Op::BinImm {
            op: BinOp::And,
            a,
            imm,
            ..
        } = self.ops[self.block_start_mark + i]
        else {
            return None;
        };
        if self.defs[a as usize] != 1 {
            return None;
        }
        self.uses[reg as usize] -= 1;
        self.uses[a as usize] += 1;
        Some((a, imm))
    }
}
