//! The values each instruction defines and uses.
use std::fmt;

use super::Val;
use super::func::Func;
use super::insts::{Callee, Inst, Term};

impl fmt::Display for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "func {} {:?} -> {:?}",
            self.name, self.sig.params, self.sig.returns
        )?;

        for (i, slot) in self.slots.iter().enumerate() {
            writeln!(f, "  slot{i}: {} align {}", slot.size, slot.align)?;
        }

        for (i, block) in self.blocks.iter().enumerate() {
            writeln!(f, " b{i}:")?;

            for inst in &block.insts {
                writeln!(f, "    {inst:?}")?;
            }

            writeln!(f, "    {:?}", block.term)?;
        }

        Ok(())
    }
}

/// Calls `def` for each value `inst` defines and `use_` for each it reads.
pub fn inst_vals(inst: &Inst, def: &mut impl FnMut(Val), use_: &mut impl FnMut(Val)) {
    match inst {
        Inst::StackAlloc {
            dst,
            size,
            ..
        } => {
            def(*dst);
            use_(*size);
        }
        Inst::IConst {
            dst, ..
        }
        | Inst::FConst {
            dst, ..
        }
        | Inst::SlotAddr {
            dst, ..
        }
        | Inst::GlobalAddr {
            dst, ..
        }
        | Inst::FuncAddr {
            dst, ..
        }
        | Inst::ForeignAddr {
            dst, ..
        } => def(*dst),
        Inst::Bin {
            dst,
            a,
            b,
            ..
        }
        | Inst::Cmp {
            dst,
            a,
            b,
            ..
        } => {
            use_(*a);
            use_(*b);
            def(*dst);
        }
        Inst::Un {
            dst,
            a,
            ..
        } => {
            use_(*a);
            def(*dst);
        }
        Inst::Conv {
            dst,
            src,
            ..
        } => {
            use_(*src);
            def(*dst);
        }
        Inst::Load {
            dst,
            addr,
            ..
        } => {
            use_(*addr);
            def(*dst);
        }
        Inst::Store {
            addr,
            value,
            ..
        } => {
            use_(*addr);
            use_(*value);
        }
        Inst::PtrAdd {
            dst,
            base,
            offset,
            ..
        } => {
            use_(*base);
            use_(*offset);
            def(*dst);
        }
        Inst::Copy {
            dst,
            src,
            ..
        } => {
            use_(*dst);
            use_(*src);
        }
        Inst::Zero {
            dst, ..
        } => use_(*dst),
        Inst::Call(call) => {
            if let Callee::Indirect(target, _) = &call.callee {
                use_(*target);
            }

            call.args.iter().copied().for_each(&mut *use_);
            call.results.iter().copied().for_each(def);
        }
        Inst::Intrinsic(call) => {
            call.args.iter().copied().for_each(&mut *use_);
            call.results.iter().copied().for_each(def);
        }
        Inst::Loc {
            ..
        } => {}
    }
}

/// Calls `use_` for each value the terminator reads.
pub fn term_uses(term: &Term, use_: &mut impl FnMut(Val)) {
    match term {
        Term::Branch {
            cond, ..
        } => use_(*cond),
        Term::Switch {
            value, ..
        } => use_(*value),
        Term::Ret(values) => values.iter().copied().for_each(use_),
        Term::Jump(_) | Term::Unreachable => {}
    }
}
