//! The function builder.
use super::func::{Block, Func, FuncOrigin, InlineHint, Linkage, Slot};
use super::insts::{CallInst, Callee, Inst, Intrinsic, IntrinsicInst, Sig, Term};
use super::program::Storage;
use super::{BinOp, BlockId, CmpOp, ConvOp, ForeignId, FuncId, GlobalId, SlotId, Ty, UnOp, Val};

// Builder

pub struct Builder {
    pub func: Func,
    pub current: BlockId,
    terminated: crate::fxhash::HashSet<BlockId>,
    /// The last `loc` marker, for `repeat_loc`.
    last_loc: Option<(u32, u32, u32, u32)>,
}

impl Builder {
    pub fn new(name: String, sig: Sig) -> Self {
        let vals = sig.params.clone();
        let func = Func {
            name,
            origin: FuncOrigin::Procedure,
            sig,
            linkage: Linkage::Internal,
            slots: Vec::new(),
            blocks: vec![Block {
                insts: Vec::new(),
                term: Term::Unreachable,
            }],
            vals,
            source_file: 0,
            trace: None,
            debug_hidden: false,
            null_checks: false,
            trusted: Vec::new(),
            inline: InlineHint::Default,
            target_features: None,
            debug: None,
        };
        Self {
            func,
            current: BlockId(0),
            terminated: crate::fxhash::HashSet::default(),
            last_loc: None,
        }
    }

    pub fn param(&self, i: usize) -> Val {
        Val(i as u32)
    }

    pub fn new_val(&mut self, ty: Ty) -> Val {
        self.func.vals.push(ty);
        Val(self.func.vals.len() as u32 - 1)
    }

    pub fn val_ty(&self, v: Val) -> Ty {
        self.func.vals[v.0 as usize]
    }

    pub fn new_block(&mut self) -> BlockId {
        self.func.blocks.push(Block {
            insts: Vec::new(),
            term: Term::Unreachable,
        });
        BlockId(self.func.blocks.len() as u32 - 1)
    }

    pub fn switch_to(&mut self, block: BlockId) {
        self.current = block;
    }

    /// True once the current block has been terminated by `terminate`.
    pub fn is_terminated(&self) -> bool {
        self.terminated.contains(&self.current)
    }

    pub fn slot(&mut self, size: u64, align: u64) -> SlotId {
        self.func.slots.push(Slot {
            size,
            align: align.max(1),
        });
        SlotId(self.func.slots.len() as u32 - 1)
    }

    /// A zero-initialized scalar slot whose address dominates every block.
    /// Used for state that can be registered conditionally and consumed after
    /// the control-flow paths join (including caller-owned macro cleanups).
    pub fn entry_zeroed_slot(&mut self, ty: Ty, size: u64, align: u64) -> Val {
        let slot = self.slot(size, align);
        let addr = self.new_val(Ty::Ptr);
        let zero = self.new_val(ty);
        self.func.blocks[0].insts.splice(
            0..0,
            [
                Inst::SlotAddr {
                    dst: addr,
                    slot,
                },
                Inst::IConst {
                    dst: zero,
                    ty,
                    value: 0,
                },
                Inst::Store {
                    ty,
                    addr,
                    value: zero,
                },
            ],
        );
        addr
    }

    pub fn push(&mut self, inst: Inst) {
        if self.is_terminated() {
            // Code after return/break is unreachable; emit into a fresh dead block.
            let dead = self.new_block();
            self.current = dead;
        }
        self.func.blocks[self.current.0 as usize].insts.push(inst);
    }

    pub fn terminate(&mut self, term: Term) {
        if self.is_terminated() {
            return;
        }
        self.func.blocks[self.current.0 as usize].term = term;
        self.terminated.insert(self.current);
    }

    pub fn iconst(&mut self, ty: Ty, value: u64) -> Val {
        let dst = self.new_val(ty);
        self.push(Inst::IConst {
            dst,
            ty,
            value,
        });
        dst
    }

    pub fn fconst(&mut self, ty: Ty, value: f64) -> Val {
        let dst = self.new_val(ty);
        self.push(Inst::FConst {
            dst,
            ty,
            value,
        });
        dst
    }

    pub fn bin(&mut self, op: BinOp, ty: Ty, a: Val, b: Val) -> Val {
        let dst = self.new_val(ty);
        self.push(Inst::Bin {
            dst,
            op,
            ty,
            a,
            b,
        });
        dst
    }

    pub fn un(&mut self, op: UnOp, ty: Ty, a: Val) -> Val {
        let dst = self.new_val(ty);
        self.push(Inst::Un {
            dst,
            op,
            ty,
            a,
        });
        dst
    }

    pub fn cmp(&mut self, op: CmpOp, ty: Ty, a: Val, b: Val) -> Val {
        let dst = self.new_val(Ty::I8);
        self.push(Inst::Cmp {
            dst,
            op,
            ty,
            a,
            b,
        });
        dst
    }

    pub fn conv(&mut self, op: ConvOp, from: Ty, to: Ty, src: Val) -> Val {
        if from == to && matches!(op, ConvOp::Bitcast) {
            return src;
        }
        let dst = self.new_val(to);
        self.push(Inst::Conv {
            dst,
            op,
            from,
            to,
            src,
        });
        dst
    }

    pub fn slot_addr(&mut self, slot: SlotId) -> Val {
        let dst = self.new_val(Ty::Ptr);
        self.push(Inst::SlotAddr {
            dst,
            slot,
        });
        dst
    }

    /// Allocate a fresh stack slot and return its address.
    pub fn alloca(&mut self, size: u64, align: u64) -> Val {
        let slot = self.slot(size, align);
        self.slot_addr(slot)
    }

    /// Address of a variable's storage: program data or a foreign symbol.
    pub fn storage_addr(&mut self, storage: Storage) -> Val {
        match storage {
            Storage::Data(global) => self.global_addr(global),
            Storage::Foreign(foreign) => self.foreign_addr(foreign),
        }
    }

    pub fn global_addr(&mut self, global: GlobalId) -> Val {
        let dst = self.new_val(Ty::Ptr);
        self.push(Inst::GlobalAddr {
            dst,
            global,
        });
        dst
    }

    pub fn func_addr(&mut self, func: FuncId) -> Val {
        let dst = self.new_val(Ty::Ptr);
        self.push(Inst::FuncAddr {
            dst,
            func,
        });
        dst
    }

    pub fn foreign_addr(&mut self, foreign: ForeignId) -> Val {
        let dst = self.new_val(Ty::Ptr);
        self.push(Inst::ForeignAddr {
            dst,
            foreign,
        });
        dst
    }

    pub fn load(&mut self, ty: Ty, addr: Val) -> Val {
        let dst = self.new_val(ty);
        self.push(Inst::Load {
            dst,
            ty,
            addr,
        });
        dst
    }

    pub fn store(&mut self, ty: Ty, addr: Val, value: Val) {
        self.push(Inst::Store {
            ty,
            addr,
            value,
        });
    }

    pub fn ptr_add(&mut self, base: Val, offset: Val) -> Val {
        let dst = self.new_val(Ty::Ptr);
        self.push(Inst::PtrAdd {
            dst,
            base,
            offset,
        });
        dst
    }

    pub fn ptr_offset(&mut self, base: Val, offset: u64) -> Val {
        if offset == 0 {
            return base;
        }
        let off = self.iconst(Ty::I64, offset);
        self.ptr_add(base, off)
    }

    pub fn copy(&mut self, dst: Val, src: Val, size: u64) {
        if size > 0 {
            self.push(Inst::Copy {
                dst,
                src,
                size,
            });
        }
    }

    pub fn zero(&mut self, dst: Val, size: u64) {
        if size > 0 {
            self.push(Inst::Zero {
                dst,
                size,
            });
        }
    }

    pub fn call(&mut self, callee: Callee, args: Vec<Val>, returns: &[Ty]) -> Vec<Val> {
        let results: Vec<Val> = returns.iter().map(|&t| self.new_val(t)).collect();
        self.push(Inst::Call(Box::new(CallInst {
            results: results.clone(),
            callee,
            args,
        })));
        results
    }

    pub fn intrinsic(&mut self, op: Intrinsic, args: Vec<Val>, returns: &[Ty]) -> Vec<Val> {
        let results: Vec<Val> = returns.iter().map(|&t| self.new_val(t)).collect();
        self.push(Inst::Intrinsic(Box::new(IntrinsicInst {
            results: results.clone(),
            op,
            args,
        })));
        results
    }

    pub fn jump(&mut self, target: BlockId) {
        self.terminate(Term::Jump(target));
    }

    pub fn branch(&mut self, cond: Val, then_block: BlockId, else_block: BlockId) {
        self.terminate(Term::Branch {
            cond,
            then_block,
            else_block,
        });
    }

    pub fn ret(&mut self, values: Vec<Val>) {
        self.terminate(Term::Ret(values));
    }

    pub fn loc(&mut self, file: u32, line: u32, col: u32, scope: u32) {
        self.last_loc = Some((file, line, col, scope));
        if !self.is_terminated() {
            self.push(Inst::Loc {
                line,
                col,
                file,
                scope,
            });
        }
    }

    /// Repeat the last `loc` marker in the current block: a block placed out of line (a failed
    /// check's) still reports the statement that branched to it.
    pub fn repeat_loc(&mut self) {
        if let Some((file, line, col, scope)) = self.last_loc {
            self.loc(file, line, col, scope);
        }
    }

    pub fn finish(mut self) -> Func {
        // Blocks that were never terminated fall off the end: void return.
        for (i, block) in self.func.blocks.iter_mut().enumerate() {
            if !self.terminated.contains(&BlockId(i as u32)) && self.func.sig.returns.is_empty() {
                block.term = Term::Ret(Vec::new());
            }
        }
        self.func
    }
}
