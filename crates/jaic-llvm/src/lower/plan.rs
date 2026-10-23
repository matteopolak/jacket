//! Lowering a whole program: the plan, its shards and how signatures are passed.
use super::Backend;
use crate::debuginfo::{DebugFormat, DebugInfo};
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::FunctionType;
use jaic::abi::{Arch, Passing, Piece};
use jaic::fxhash::HashMap;
use jaic::ir::{AggLayout, Linkage as IrLinkage, Program, Ty};
use std::cell::{OnceCell, RefCell};

/// How the program is divided among modules when codegen is split (`partition.rs`).
pub struct Plan {
    /// The module that defines each function.
    pub func_owner: Vec<u32>,
    /// The module that defines each global.
    pub global_owner: Vec<u32>,
    /// Whether a function is referred to from a module other than its owner (or from outside
    /// the program): only those need a name the linker can see.
    pub func_shared: Vec<bool>,
    pub global_shared: Vec<bool>,
    /// Per module, the functions of other modules it holds a copy of for the optimizer to
    /// inline (`available_externally`: never emitted).
    pub imports: Vec<Vec<u32>>,
    /// The same for read-only globals, which the optimizer may fold loads from.
    pub global_imports: Vec<Vec<u32>>,
}

/// Which part of the program a module holds when codegen is split across modules.
#[derive(Clone, Copy)]
pub struct Shard<'p> {
    pub plan: &'p Plan,
    pub index: u32,
}

/// What `lower_program` needs to know about the target and the unit being lowered.
pub struct LowerOptions<'a> {
    pub arch: Arch,
    pub target: jaic::target::Target,
    /// The part of the program this module holds, `None` for the whole of it.
    pub shard: Option<Shard<'a>>,
    /// The debug-info format and whether the build is optimized, `None` without `-g`.
    pub debug: Option<(DebugFormat, bool)>,
    /// An `-O0` build, which copies small aggregates inline.
    pub unoptimized: bool,
    /// Per function: never inline it.
    pub no_inline: &'a [bool],
    pub simd: crate::simd::Target,
    /// The LLVM feature string the target machine was made with: what `#target_features` adds to.
    pub features: &'a str,
}

/// Lower `program` into `module`. With a shard, the module defines only that shard's
/// functions (and, in shard 0, the globals); everything else is declared, and internal
/// symbols become hidden externals so the modules link together.
pub fn lower_program<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    program: &Program,
    options: &LowerOptions,
) -> Result<(), String> {
    let LowerOptions {
        arch,
        target,
        shard,
        debug,
        unoptimized,
        no_inline,
        simd,
        features,
    } = *options;
    let debug = debug
        .map(|(format, optimized)| DebugInfo::new(context, module, program, format, optimized));
    let mut backend = Backend {
        ctx: context,
        module,
        builder: context.create_builder(),
        program,
        arch,
        target,
        shard,
        imported: shard.map_or_else(Vec::new, |s| {
            let mut flags = vec![false; program.funcs.len()];
            for &i in &s.plan.imports[s.index as usize] {
                flags[i as usize] = true;
            }
            flags
        }),
        imported_globals: shard.map_or_else(Vec::new, |s| {
            let mut flags = vec![false; program.globals.len()];
            for &i in &s.plan.global_imports[s.index as usize] {
                flags[i as usize] = true;
            }
            flags
        }),
        funcs: cells(program.funcs.len()),
        globals: cells(program.globals.len()),
        foreigns: cells(program.foreigns.len()),
        foreign_symbols: RefCell::new(HashMap::default()),
        exports: export_names(program),
        foreign_ids: foreign_ids(program),
        null_fail: OnceCell::new(),
        check_files: RefCell::new(HashMap::default()),
        debug,
        inline_small_copies: unoptimized && !arch.is_wasm(),
        no_inline: no_inline.to_vec(),
        simd,
        features: features.to_string(),
    };
    backend.run().map_err(|e| e.0)
}

/// How one IR parameter maps onto LLVM parameters.
#[derive(Clone, Debug)]
pub(super) enum ParamPlan {
    Scalar(Ty),
    /// A by-value C aggregate; the IR passes a pointer to it.
    Agg(AggLayout, Passing),
    /// A by-value C aggregate of several register pieces on `AArch64`, passed as one array
    /// parameter (`[2 x i64]`, `[4 x float]`) like Clang does: the backend then gives the whole
    /// array registers or none, where separate scalars would be split between the two.
    AggArray(AggLayout, Vec<Piece>),
    /// The IR out-pointer of a C aggregate return that LLVM returns in registers.
    Dropped,
}

#[derive(Clone, Debug)]
pub(super) enum RetPlan {
    Scalars(Vec<Ty>),
    /// C aggregate returned in register chunks; stored through the IR out-pointer.
    Registers(AggLayout, Vec<Piece>),
    /// C aggregate returned through a hidden first `sret` pointer.
    Sret,
}

/// Attributes attached to an LLVM parameter.
#[derive(Clone, Copy, Debug)]
pub(super) enum ParamAttr {
    Sret(u64),
    ByVal(u64, u64),
    InReg,
}

pub(super) struct Lowered<'ctx> {
    pub(super) fn_ty: FunctionType<'ctx>,
    pub(super) params: Vec<ParamPlan>,
    pub(super) ret: RetPlan,
    pub(super) attrs: Vec<(u32, ParamAttr)>,
}

/// `n` empty slots for declarations made on first use.
fn cells<T>(n: usize) -> Vec<OnceCell<T>> {
    (0..n).map(|_| OnceCell::new()).collect()
}

/// Program function ids by exported symbol name (the first of a repeated name).
fn export_names(program: &Program) -> HashMap<&str, usize> {
    let mut names = HashMap::default();
    for (i, func) in program.funcs.iter().enumerate() {
        if let Some(func) = func
            && let IrLinkage::Export(name) = &func.linkage
        {
            names.entry(name.as_str()).or_insert(i);
        }
    }
    names
}

/// The first foreign id of each symbol.
pub(super) fn foreign_ids(program: &Program) -> HashMap<&str, usize> {
    let mut ids = HashMap::default();
    for (i, foreign) in program.foreigns.iter().enumerate() {
        ids.entry(foreign.symbol.as_str()).or_insert(i);
    }
    ids
}
