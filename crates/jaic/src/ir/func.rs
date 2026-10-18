//! Blocks, slots and functions, with their debug records.
use super::insts::{Callee, Inst, Sig, Term};
use super::{FuncId, GlobalId, Ty, Val};

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
    /// What the function was made for: a procedure, or compile-time code the compiler wraps
    /// in one.
    pub origin: FuncOrigin,
    pub sig: Sig,
    pub linkage: Linkage,
    pub slots: Vec<Slot>,
    pub blocks: Vec<Block>,
    /// Register class of each `Val`.
    pub vals: Vec<Ty>,
    /// The first `sig.params.len()` vals are the parameters, in order.
    pub source_file: u32,
    /// Present for procedures that take a context: what a stack trace node says about it.
    pub trace: Option<TraceInfo>,
    /// An explicit `#no_debug` header hides unwind frames in the interpreter too.
    pub debug_hidden: bool,
    /// Native code checks every load, store and copy through a pointer that may be null
    /// (`Build_Options.null_pointer_check`, off for `#no_abc` procedures).
    pub null_checks: bool,
    /// Values known to hold a real (never null) address: the context pointer, the addresses
    /// of the stack trace bookkeeping, and the incoming addresses of aggregate parameters and
    /// results. Native code does not null-check loads and stores through
    /// them.
    pub trusted: Vec<Val>,
    /// The procedure was declared `inline` or `no_inline`: native code always or never inlines
    /// it.
    pub inline: InlineHint,
    /// Native debug information (named variables, lexical scopes), recorded only when the
    /// program is built with debug info. Boxed so the interpreter's hot data stays small.
    pub debug: Option<Box<FuncDebug>>,
}

/// Whether native code inlines a function: `inline` and `no_inline` on its declaration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InlineHint {
    #[default]
    Default,
    Always,
    Never,
}

/// What a function was made for. Compile-time code outside a procedure runs in a function of
/// its own, which reports name by what it is rather than by its `name`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FuncOrigin {
    #[default]
    Procedure,
    /// A `#run` directive's code.
    Run,
    /// A constant's initializer evaluated at compile time.
    ConstInit,
}

impl Func {
    /// A readable listing of the function: signature, stack slots and each block's instructions
    /// and terminator, one per line (the text of a `DEBUG_DUMP` message). The operands are
    /// printed in the form the IR types derive `Debug` with, and the layout may change with
    /// the IR.
    pub fn listing(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let name = if self.name.is_empty() {
            self.origin.thunk_name()
        } else {
            &self.name
        };
        let _ = writeln!(
            out,
            "{name}: ({:?}) -> ({:?})",
            self.sig.params, self.sig.returns
        );
        for (i, slot) in self.slots.iter().enumerate() {
            let _ = writeln!(out, "  slot{i}: {} bytes, align {}", slot.size, slot.align);
        }
        for (i, block) in self.blocks.iter().enumerate() {
            let _ = writeln!(out, "b{i}:");
            for inst in &block.insts {
                let _ = writeln!(out, "    {inst:?}");
            }
            let _ = writeln!(out, "    {:?}", block.term);
        }
        out
    }

    /// The first function this one calls directly, in block order.
    pub fn first_direct_callee(&self) -> Option<FuncId> {
        self.blocks
            .iter()
            .flat_map(|b| &b.insts)
            .find_map(|inst| match inst {
                Inst::Call(call) => match call.callee {
                    Callee::Func(f) => Some(f),
                    _ => None,
                },
                _ => None,
            })
    }
}

impl FuncOrigin {
    /// The function name for compile-time code (IR dumps, stack trace nodes).
    pub fn thunk_name(self) -> &'static str {
        match self {
            FuncOrigin::Procedure => "",
            FuncOrigin::Run => "#run",
            FuncOrigin::ConstInit => "#const",
        }
    }
}

/// Debug information of one procedure, for the native backend (`docs/native/debug-info.md`).
#[derive(Clone, Debug, Default)]
pub struct FuncDebug {
    /// Procedure name as written (polymorph instances share it).
    pub name: String,
    pub file: u32,
    pub line: u32,
    /// Lexical scopes. Entry 0 is the procedure body itself; `Inst::Loc::scope` and
    /// `DebugVar::scope` index this.
    pub scopes: Vec<DebugScope>,
    pub vars: Vec<DebugVar>,
}

#[derive(Clone, Copy, Debug)]
pub struct DebugScope {
    /// Enclosing scope (entry 0 is its own parent).
    pub parent: u32,
    pub line: u32,
    pub col: u32,
}

/// A named local variable or parameter.
#[derive(Clone, Debug)]
pub struct DebugVar {
    pub name: String,
    /// Key into `Program::debug_types` (a `TypeId`).
    pub ty: u32,
    /// The variable's address: usually a `SlotAddr` result, or a parameter (aggregates
    /// arrive by pointer).
    pub addr: Val,
    /// 1-based parameter position; 0 for a local.
    pub arg: u32,
    pub scope: u32,
    pub line: u32,
    pub col: u32,
}

/// A global variable with debug information.
#[derive(Clone, Debug)]
pub struct DebugGlobal {
    pub global: GlobalId,
    pub name: String,
    pub ty: u32,
    pub file: u32,
    pub line: u32,
}

/// Debug description of a Jai type, keyed by `TypeId` in `Program::debug_types`
/// (plus the synthetic keys [`DEBUG_CHAR`] and [`DEBUG_CHAR_PTR`]).
#[derive(Clone, Debug)]
pub struct DebugType {
    pub name: String,
    pub size: u64,
    pub align: u64,
    pub kind: DebugTypeKind,
}

/// `u8` shown as a character: the pointee of `string.data`, so debuggers print text.
pub const DEBUG_CHAR: u32 = u32::MAX - 1;

pub const DEBUG_CHAR_PTR: u32 = u32::MAX - 2;

#[derive(Clone, Debug)]
pub enum DebugTypeKind {
    Void,
    Bool,
    Int {
        signed: bool,
    },
    Char,
    Float,
    /// Pointee key (a `Void` pointee is `*void`).
    Pointer(u32),
    /// Structs, unions and the built-in aggregates (`string`, `[] T`, `[..] T`, `Any`).
    Struct {
        fields: Vec<DebugField>,
        union: bool,
    },
    Array {
        elem: u32,
        count: u64,
    },
    Enum {
        base: u32,
        members: Vec<(String, i64)>,
    },
    /// Another name for a type (`#type,distinct`, `Type`, procedure types).
    Typedef(u32),
}

#[derive(Clone, Debug)]
pub struct DebugField {
    pub name: String,
    pub ty: u32,
    pub offset: u64,
}

/// Name and declaration site of a procedure, for `context.stack_trace` nodes.
#[derive(Clone, Debug)]
pub struct TraceInfo {
    /// Empty for an anonymous procedure.
    pub name: String,
    pub file: u32,
    pub line: u32,
    pub col: u32,
}

impl Func {
    pub fn param(&self, i: usize) -> Val {
        Val(i as u32)
    }
}
