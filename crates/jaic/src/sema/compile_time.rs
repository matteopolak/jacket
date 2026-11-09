//! What compile-time code has done in this compile: the interpreter it runs in, the top-level
//! `#run`s and `#assert`s and how many have run, and the values frozen out of it.
use super::body_probe::{Probed, W};
use crate::ast;
use crate::fxhash::HashMap;
use crate::interp::Interp;
use crate::ir;
use crate::sema::{Compiler, ScopeId};

/// The bytes a build's `#Context` may grow to (`context_size_max`'s default). The compile-time
/// context is allocated this large, so an `#add_context` met while it is in use fits in place.
pub const CONTEXT_CAPACITY: u64 = 4096;

/// A top-level `#assert`: condition, message, extra arguments, and the scope it is checked in.
pub type TopLevelAssert = (ast::Expr, Option<ast::Expr>, Vec<ast::Expr>, ScopeId);

#[derive(Default)]

pub struct CompileTime {
    pub interp: Probed<Interp, { W::CtInterp as usize }>,
    /// Interpreter address of the compile-time `#Context`.
    pub context: Option<u64>,
    /// The global behind `context`, which has room for `#add_context`s met after it was made.
    pub context_global: Option<ir::GlobalId>,
    /// The global behind compile-time temporary storage. It starts empty in the program, so a
    /// pointer into it is frozen by copying what it points at, not by relocating to the global.
    pub(crate) temporary_buffer: Option<ir::GlobalId>,
    /// Compile-time code run so far, of any kind.
    pub(crate) executions: u64,
    /// Compile-time runs that ended in a trap so far.
    pub traps: u64,
    pub top_level_runs: Probed<Vec<(ast::Expr, ScopeId)>, { W::CtTopLevelRuns as usize }>,
    pub asserts: Probed<Vec<TopLevelAssert>, { W::CtAsserts as usize }>,
    /// How many of `top_level_runs`/`asserts` already executed.
    pub(crate) runs_done: usize,
    pub(crate) asserts_done: usize,
    /// Interpreter blocks already copied into globals while freezing one compile-time value,
    /// by `(address, bytes)`: pointer cycles (`n.next = n`) refer back instead of recursing.
    pub(crate) frozen: Probed<HashMap<(u64, u64), ir::GlobalId>, { W::CtFrozen as usize }>,
    /// The compiler plain `#run` executes in when this one cross-compiles (`sema::host_twin`).
    pub(crate) twin: Probed<super::host_twin::TwinSlot, { W::CtTwin as usize }>,
    /// Expressions left to check until the check's time limit is asked again
    /// (`Compiler::check_expr`).
    pub(crate) expr_polls: u8,
}

impl Drop for Compiler {
    /// Jai threads started by compile-time code may still run, in frames that borrow the
    /// program, sources and types: stop and join them before any field is dropped.
    fn drop(&mut self) {
        self.ct.interp.shutdown();
    }
}
