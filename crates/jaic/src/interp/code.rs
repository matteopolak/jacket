//! The interpreter's own form of a procedure, made from its IR on first call.
//!
//! The IR is built for the native backend: locals live in stack slots, so most statements
//! are `SlotAddr` + `Load`/`Store`, and constants are separate `IConst` instructions. Run
//! as is, every one of those is a dispatch. Here each block becomes a run of 16-byte `Op`s
//! where
//!
//! - an address that is a constant offset from the frame (a slot, or a field of one) is
//!   folded into the load or store that uses it,
//! - a constant pointer offset (`PtrAdd` of an `IConst`) is folded into the load or store
//!   that follows it in the same block,
//! - an integer constant that fits 32 bits becomes an immediate of the arithmetic or
//!   comparison that uses it,
//! - a comparison feeding only the block's branch becomes part of the branch, and
//! - instructions whose results end up unused (the folded constants) are dropped.
//!
//! Calls and intrinsics, which are rare and slow anyway, run through `Interp::step` on the
//! original instruction. Profile counts (`-profile`) stay in IR instructions.
//!
//! Folding a value into a later instruction reads its operands later than the IR would.
//! That is only done for values defined exactly once (parameters count as a definition)
//! whose definition cannot run again in between: constants anywhere, and other values
//! within one block.
mod build;
mod flow;
mod lower_ends;
mod lower_insts;
mod op;
mod promote;
mod run;
mod sink;

pub(super) use self::build::{build, fingerprint};
pub(super) use self::op::{Code, CodeBlock, Op};
pub(super) use self::run::{load_raw, store_raw};
use super::mask;
use crate::ir::inst_vals;
