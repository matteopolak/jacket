//! What compile-time code has done in this compile: the interpreter it runs in, the top-level
//! `#run`s and `#assert`s and how many have run, and the values frozen out of it.
use crate::ast;
use crate::fxhash::HashMap;
use crate::interp::Interp;
use crate::ir;
use crate::sema::{Compiler, ScopeId};

/// The bytes a build's `#Context` may grow to (`context_size_max`'s default). The compile-time
/// context is allocated this large, so an `#add_context` met while it is in use fits in place.
pub const CONTEXT_CAPACITY: u64 = 4096;

#[derive(Default)]
pub struct CompileTime {
    pub interp: Interp,
    /// Interpreter address of the compile-time `#Context`.
    pub context: Option<u64>,
    /// The global behind `context`, which has room for `#add_context`s met after it was made.
    pub context_global: Option<ir::GlobalId>,
    /// Compile-time code run so far, of any kind.
    pub(crate) executions: u64,
    /// Compile-time runs that ended in a trap so far.
    pub traps: u64,
    pub top_level_runs: Vec<(ast::Expr, ScopeId)>,
    pub asserts: Vec<(ast::Expr, Option<ast::Expr>, Vec<ast::Expr>, ScopeId)>,
    /// How many of `top_level_runs`/`asserts` already executed.
    pub(crate) runs_done: usize,
    pub(crate) asserts_done: usize,
    /// Interpreter blocks already copied into globals while freezing one compile-time value,
    /// by `(address, bytes)`: pointer cycles (`n.next = n`) refer back instead of recursing.
    pub(crate) frozen: HashMap<(u64, u64), ir::GlobalId>,
}

impl Drop for Compiler {
    /// Jai threads started by compile-time code may still run, in frames that borrow the
    /// program, sources and types: stop and join them before any field is dropped.
    fn drop(&mut self) {
        self.ct.interp.shutdown();
    }
}
