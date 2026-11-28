//! Per-function lowering state.
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue, PointerValue};
use jaic::abi::Piece;
use jaic::ir::NullChecks;

use crate::debuginfo::FnDebug;

/// Per-function lowering state.
pub(super) struct FnState<'ctx> {
    pub(super) function: FunctionValue<'ctx>,
    pub(super) vals: Vec<Option<BasicValueEnum<'ctx>>>,
    pub(super) slots: Vec<PointerValue<'ctx>>,
    pub(super) blocks: Vec<Option<BasicBlock<'ctx>>>,
    /// Builder positioned in the dedicated alloca block.
    pub(super) allocas: Builder<'ctx>,
    /// A C aggregate returned in registers: its pieces and the memory the IR writes it to.
    pub(super) reg_ret: Option<(Vec<Piece>, PointerValue<'ctx>)>,
    /// Debug locations and variables of this function, with debug info on.
    pub(super) dbg: Option<FnDebug<'ctx>>,
    /// (file, line) of the last `Inst::Loc`: where a failed check reports itself.
    pub(super) loc: (u32, u32),
    /// Which accesses need a null check (`Func::null_checks`).
    pub(super) checks: NullChecks,
    /// The `i1` a comparison produced, for the `Val` holding it widened to a byte: a branch on
    /// that byte tests the `i1` itself, which keeps the compare and the branch adjacent for
    /// instruction selection.
    pub(super) flags: Vec<Option<IntValue<'ctx>>>,
}
