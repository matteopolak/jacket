//! Low-level IR shared by the interpreter and the LLVM backend.
//!
//! Values are scalars in virtual registers (`Val`), each defined exactly once.
//! Every aggregate lives in memory: locals are stack `Slot`s, and aggregate
//! parameters/results travel as pointers. Mutable locals are also slots, so no
//! phi nodes are needed (LLVM's mem2reg recovers SSA form).
//!
//! Jai calling convention (`Conv::Jai`): an optional leading context pointer,
//! then each parameter (aggregates by pointer to a caller-owned copy), then one
//! out-pointer per aggregate result. Scalar results are returned directly.
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct FuncId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct GlobalId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct ForeignId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Val(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BlockId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SlotId(pub u32);

/// Scalar register classes. Booleans are `I8` holding 0 or 1.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Ty {
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Ptr,
}

impl Ty {
    pub fn size(self) -> u64 {
        match self {
            Ty::I8 => 1,
            Ty::I16 => 2,
            Ty::I32 | Ty::F32 => 4,
            Ty::I64 | Ty::F64 | Ty::Ptr => 8,
        }
    }
    pub fn is_float(self) -> bool {
        matches!(self, Ty::F32 | Ty::F64)
    }
    pub fn int(bytes: u64) -> Ty {
        match bytes {
            1 => Ty::I8,
            2 => Ty::I16,
            4 => Ty::I32,
            _ => Ty::I64,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    SDiv,
    UDiv,
    SRem,
    URem,
    And,
    Or,
    Xor,
    Shl,
    LShr,
    AShr,
    Rotl,
    Rotr,
    FAdd,
    FSub,
    FMul,
    FDiv,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CmpOp {
    Eq,
    Ne,
    SLt,
    SLe,
    SGt,
    SGe,
    ULt,
    ULe,
    UGt,
    UGe,
    FEq,
    FNe,
    FLt,
    FLe,
    FGt,
    FGe,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConvOp {
    Trunc,
    ZExt,
    SExt,
    FToS,
    FToU,
    SToF,
    UToF,
    FExt,
    FTrunc,
    /// Same-size reinterpretation (int <-> float, ptr <-> int).
    Bitcast,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnOp {
    Neg,
    Not,
    FNeg,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Conv {
    Jai,
    C,
}

/// Built-in operations with backend-specific implementations.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Intrinsic {
    Memcpy,
    Memset,
    Memcmp,
    /// (ptr, old, new) -> (success, old_value); width from operand type.
    CompareAndSwap,
    DebugBreak,
    /// Abort with a runtime error (bounds check, unreachable case...).
    Trap,
    /// (ptr: *u8, count: s64, to_stderr: bool): compile-time `write_string`.
    CompilerWrite,
    Sqrt,
    Sin,
    Cos,
    Floor,
    Ceil,
    Round,
    Trunc,
    Fabs,
    /// Return address / frame queries used by stack traces.
    ReturnAddress,
    /// Read the cycle counter (`rdtsc`).
    CycleCounter,
    Pause,
    /// Bit counting used by `#asm` lowering: (x, bits) -> count, same width as `x`.
    /// `Ctlz`/`Cttz` return `bits` for a zero input.
    Popcount,
    Ctlz,
    Cttz,
    /// (x, bits) -> x with its bytes reversed.
    Bswap,
    /// 1 when executing at compile time (`#compile_time`).
    IsCompileTime,
}

#[derive(Clone, Debug)]
pub enum Callee {
    Func(FuncId),
    Foreign(ForeignId),
    /// Indirect call through a procedure pointer with the given signature.
    Indirect(Val, Sig),
}

/// A lowered call signature (scalar parameter/result classes).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Sig {
    pub params: Vec<Ty>,
    pub returns: Vec<Ty>,
    pub conv: Conv,
    /// C variadic: `params` lists only the fixed parameters.
    pub c_varargs: bool,
    /// Per-parameter C ABI aggregate descriptions (for by-value structs in C calls).
    pub c_abi: Option<Box<CAbi>>,
}

/// C ABI facts for aggregates passed by value to/from foreign code. Index i
/// describes IR parameter i: `Some(layout)` means the IR passes a pointer to
/// an aggregate that the C ABI passes by value.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct CAbi {
    pub params: Vec<Option<AggLayout>>,
    /// The single C return value is an aggregate returned through the last IR param (out-pointer).
    pub ret: Option<AggLayout>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AggLayout {
    pub size: u64,
    pub align: u64,
    /// Flattened scalar fields: (offset, class).
    pub fields: Vec<(u64, Ty)>,
}

#[derive(Clone, Debug)]
pub enum Inst {
    IConst {
        dst: Val,
        ty: Ty,
        value: u64,
    },
    FConst {
        dst: Val,
        ty: Ty,
        value: f64,
    },
    Bin {
        dst: Val,
        op: BinOp,
        ty: Ty,
        a: Val,
        b: Val,
    },
    Un {
        dst: Val,
        op: UnOp,
        ty: Ty,
        a: Val,
    },
    /// Result is `I8` 0/1.
    Cmp {
        dst: Val,
        op: CmpOp,
        ty: Ty,
        a: Val,
        b: Val,
    },
    Conv {
        dst: Val,
        op: ConvOp,
        from: Ty,
        to: Ty,
        src: Val,
    },
    SlotAddr {
        dst: Val,
        slot: SlotId,
    },
    GlobalAddr {
        dst: Val,
        global: GlobalId,
    },
    FuncAddr {
        dst: Val,
        func: FuncId,
    },
    ForeignAddr {
        dst: Val,
        foreign: ForeignId,
    },
    Load {
        dst: Val,
        ty: Ty,
        addr: Val,
    },
    Store {
        ty: Ty,
        addr: Val,
        value: Val,
    },
    /// `dst = base + offset` (bytes).
    PtrAdd {
        dst: Val,
        base: Val,
        offset: Val,
    },
    /// Copy `size` bytes (non-overlapping).
    Copy {
        dst: Val,
        src: Val,
        size: u64,
    },
    /// Zero `size` bytes.
    Zero {
        dst: Val,
        size: u64,
    },
    Call {
        results: Vec<Val>,
        callee: Callee,
        args: Vec<Val>,
    },
    Intrinsic {
        results: Vec<Val>,
        op: Intrinsic,
        args: Vec<Val>,
    },
    /// Source line marker for debug info and runtime error locations.
    Loc {
        line: u32,
        col: u32,
        file: u32,
    },
}

#[derive(Clone, Debug)]
pub enum Term {
    Jump(BlockId),
    Branch {
        cond: Val,
        then_block: BlockId,
        else_block: BlockId,
    },
    /// Integer switch: `cases` are (value, target).
    Switch {
        value: Val,
        ty: Ty,
        cases: Vec<(u64, BlockId)>,
        default: BlockId,
    },
    Ret(Vec<Val>),
    Unreachable,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub insts: Vec<Inst>,
    pub term: Term,
}

#[derive(Clone, Copy, Debug)]
pub struct Slot {
    pub size: u64,
    pub align: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Linkage {
    Internal,
    /// `#program_export "name"`
    Export(String),
}

#[derive(Clone, Debug)]
pub struct Func {
    pub name: String,
    pub sig: Sig,
    pub linkage: Linkage,
    pub slots: Vec<Slot>,
    pub blocks: Vec<Block>,
    /// Register class of each `Val`.
    pub vals: Vec<Ty>,
    /// The first `sig.params.len()` vals are the parameters, in order.
    pub source_file: u32,
}

impl Func {
    pub fn param(&self, i: usize) -> Val {
        Val(i as u32)
    }
}

/// Where a global variable lives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Storage {
    /// Program data (`x: T;`).
    Data(GlobalId),
    /// A symbol of a foreign library (`x: T #elsewhere lib;`).
    Foreign(ForeignId),
}

/// A relocation inside a global's initial bytes: an 8-byte pointer slot at
/// `offset` that must hold the address of `target` plus `addend`.
#[derive(Clone, Debug, PartialEq)]
pub struct Reloc {
    pub offset: u64,
    pub target: RelocTarget,
    pub addend: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocTarget {
    Global(GlobalId),
    Func(FuncId),
    Foreign(ForeignId),
}

#[derive(Clone, Debug)]
pub struct Global {
    pub name: String,
    pub size: u64,
    pub align: u64,
    /// Initial bytes (empty = zero-initialized).
    pub init: Vec<u8>,
    pub relocs: Vec<Reloc>,
    pub read_only: bool,
    /// `#program_export` name, if any.
    pub export: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Foreign {
    /// Symbol name in the library.
    pub symbol: String,
    /// Library handle index (`Program::libraries`); `None` for symbols found in the process.
    pub library: Option<usize>,
    pub sig: Sig,
    /// A foreign *variable* (`x: T #elsewhere lib`) rather than a function.
    pub is_data: bool,
}

#[derive(Clone, Debug)]
pub struct Library {
    /// Name as written (`"libc"`, `"SDL2"`, path...).
    pub name: String,
    pub system: bool,
    /// Directory of the declaring source file (for relative library paths).
    pub base_dir: String,
}

/// All lowered code. Functions are lowered on demand, so slots may be empty.
#[derive(Default)]
pub struct Program {
    pub funcs: Vec<Option<Func>>,
    pub func_names: Vec<String>,
    pub globals: Vec<Global>,
    pub foreigns: Vec<Foreign>,
    pub libraries: Vec<Library>,
}

impl Program {
    pub fn reserve_func(&mut self, name: String) -> FuncId {
        self.funcs.push(None);
        self.func_names.push(name);
        FuncId(self.funcs.len() as u32 - 1)
    }
    pub fn func(&self, id: FuncId) -> Option<&Func> {
        self.funcs[id.0 as usize].as_ref()
    }
    pub fn add_global(&mut self, global: Global) -> GlobalId {
        self.globals.push(global);
        GlobalId(self.globals.len() as u32 - 1)
    }
    pub fn add_foreign(&mut self, foreign: Foreign) -> ForeignId {
        self.foreigns.push(foreign);
        ForeignId(self.foreigns.len() as u32 - 1)
    }
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

pub struct Builder {
    pub func: Func,
    pub current: BlockId,
    terminated: std::collections::HashSet<BlockId>,
}

impl Builder {
    pub fn new(name: String, sig: Sig) -> Self {
        let vals = sig.params.clone();
        let func = Func {
            name,
            sig,
            linkage: Linkage::Internal,
            slots: Vec::new(),
            blocks: vec![Block {
                insts: Vec::new(),
                term: Term::Unreachable,
            }],
            vals,
            source_file: 0,
        };
        Self {
            func,
            current: BlockId(0),
            terminated: Default::default(),
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
        self.push(Inst::Call {
            results: results.clone(),
            callee,
            args,
        });
        results
    }
    pub fn intrinsic(&mut self, op: Intrinsic, args: Vec<Val>, returns: &[Ty]) -> Vec<Val> {
        let results: Vec<Val> = returns.iter().map(|&t| self.new_val(t)).collect();
        self.push(Inst::Intrinsic {
            results: results.clone(),
            op,
            args,
        });
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
    pub fn loc(&mut self, file: u32, line: u32, col: u32) {
        if !self.is_terminated() {
            self.push(Inst::Loc {
                line,
                col,
                file,
            });
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
