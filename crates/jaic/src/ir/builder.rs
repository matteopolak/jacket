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
    /// Bytes remaining in a known live stack allocation. Loads, casts and raw
    /// arithmetic deliberately do not propagate this proof.
    object_bytes: crate::fxhash::HashMap<Val, u64>,
    /// Entry copies of aggregate parameters, made by `param_copy`.
    param_copies: Vec<ParamCopy>,
}

/// The procedure's own copy of an aggregate parameter, kept only if the body takes the
/// parameter's address.
struct ParamCopy {
    addr: Val,
    incoming: Val,
    address_taken: bool,
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
            cold: false,
            target_features: None,
            debug: None,
        };
        Self {
            func,
            current: BlockId(0),
            terminated: crate::fxhash::HashSet::default(),
            last_loc: None,
            object_bytes: crate::fxhash::HashMap::default(),
            param_copies: Vec::new(),
        }
    }

    /// The place of an aggregate parameter that arrives as the address `incoming` of the
    /// caller's value. The procedure reads the caller's value in place, so a pointer to a
    /// member (`*p.field`, `p.items.data`) points into it, unless the body takes the address of
    /// the parameter itself (`*p`, see `note_address_taken`). Then it works on its own copy, so
    /// `advance(*s, 1)` does not move the caller's string. Which one is settled in `finish`.
    pub fn param_copy(&mut self, incoming: Val, size: u64, align: u64) -> Val {
        let addr = self.alloca(size.max(1), align);
        self.copy(addr, incoming, size);
        self.param_copies.push(ParamCopy {
            addr,
            incoming,
            address_taken: false,
        });
        addr
    }

    /// The program takes `addr`, the address of a whole variable, as a pointer value: if it is
    /// a parameter's place, that parameter needs its own copy.
    pub fn note_address_taken(&mut self, addr: Val) {
        if let Some(copy) = self.param_copies.iter_mut().find(|copy| copy.addr == addr) {
            copy.address_taken = true;
        }
    }

    /// Drop the copy of each parameter whose address the body never took: its place becomes
    /// the caller's value.
    fn elide_param_copies(&mut self) {
        let elided: crate::fxhash::HashMap<Val, Val> = std::mem::take(&mut self.param_copies)
            .into_iter()
            .filter(|copy| !copy.address_taken)
            .map(|copy| (copy.addr, copy.incoming))
            .collect();
        if elided.is_empty() {
            return;
        }
        let zeros: Vec<Val> = (0..elided.len()).map(|_| self.new_val(Ty::I64)).collect();
        let mut zeros = zeros.into_iter();
        let old = std::mem::take(&mut self.func.blocks[0].insts);
        let insts = &mut self.func.blocks[0].insts;
        insts.reserve(old.len() + elided.len());
        for inst in old {
            match inst {
                Inst::Copy {
                    dst,
                    src,
                    ..
                } if elided.get(&dst) == Some(&src) => {}
                Inst::SlotAddr {
                    dst, ..
                } if elided.contains_key(&dst) => {
                    let zero = zeros.next().expect("one zero per elided copy");
                    insts.push(Inst::IConst {
                        dst: zero,
                        ty: Ty::I64,
                        value: 0,
                    });
                    insts.push(Inst::PtrAdd {
                        dst,
                        base: elided[&dst],
                        offset: zero,
                        inbounds: true,
                    });
                }
                inst => insts.push(inst),
            }
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
        self.object_bytes.insert(addr, size);
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
        self.object_bytes
            .insert(dst, self.func.slots[slot.0 as usize].size);
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
        self.ptr_add_with_proof(base, offset, None)
    }

    fn ptr_add_with_proof(&mut self, base: Val, offset: Val, remaining: Option<u64>) -> Val {
        let dst = self.new_val(Ty::Ptr);
        self.push(Inst::PtrAdd {
            dst,
            base,
            offset,
            inbounds: remaining.is_some(),
        });
        if let Some(bytes) = remaining {
            self.object_bytes.insert(dst, bytes);
        }
        dst
    }

    pub fn ptr_offset(&mut self, base: Val, offset: u64) -> Val {
        if offset == 0 {
            return base;
        }
        let off = self.iconst(Ty::I64, offset);
        let remaining = self
            .object_bytes
            .get(&base)
            .and_then(|n| n.checked_sub(offset));
        self.ptr_add_with_proof(base, off, remaining)
    }

    /// Called only after a fixed-array bounds check. The allocation must also
    /// cover the entire declared array, so a forged/nullable array place is plain.
    pub fn checked_array_addr(
        &mut self,
        base: Val,
        offset: Val,
        bytes: u64,
        elem_size: u64,
    ) -> Val {
        let remaining = self
            .object_bytes
            .get(&base)
            .filter(|&&n| n >= bytes)
            .map(|_| elem_size);
        self.ptr_add_with_proof(base, offset, remaining)
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
        self.elide_param_copies();
        // Blocks that were never terminated fall off the end: void return.
        for (i, block) in self.func.blocks.iter_mut().enumerate() {
            if !self.terminated.contains(&BlockId(i as u32)) && self.func.sig.returns.is_empty() {
                block.term = Term::Ret(Vec::new());
            }
        }
        self.func
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_proofs_require_allocation_extent_and_do_not_cross_raw_operations() {
        let mut b = Builder::new(
            "provenance".into(),
            Sig {
                params: vec![Ty::Ptr],
                returns: vec![],
                conv: super::super::Conv::Jai,
                c_varargs: false,
                c_fixed: 0,
                c_abi: None,
            },
        );
        let base = b.alloca(24, 8);
        let field = b.ptr_offset(base, 8);
        b.ptr_offset(field, 8);
        b.ptr_offset(base, 25);
        let null = b.iconst(Ty::Ptr, 0);
        b.ptr_offset(null, 8);
        b.ptr_offset(b.param(0), 8);
        let off = b.iconst(Ty::I64, 8);
        let raw = b.ptr_add(base, off);
        b.ptr_offset(raw, 8);
        let int = b.conv(ConvOp::Bitcast, Ty::Ptr, Ty::I64, base);
        let cast = b.conv(ConvOp::Bitcast, Ty::I64, Ty::Ptr, int);
        b.ptr_offset(cast, 8);
        b.checked_array_addr(base, off, 24, 8);
        b.checked_array_addr(base, off, 32, 8);
        b.checked_array_addr(null, off, 24, 8);
        let hints: Vec<_> = b
            .finish()
            .blocks
            .iter()
            .flat_map(|block| &block.insts)
            .filter_map(|inst| match inst {
                Inst::PtrAdd {
                    inbounds, ..
                } => Some(*inbounds),
                _ => None,
            })
            .collect();
        assert_eq!(
            hints,
            [
                true, true, false, false, false, false, false, false, true, false, false
            ]
        );
    }
}
