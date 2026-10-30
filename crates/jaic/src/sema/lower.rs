//! Operands and the per-procedure lowering context.
use super::scope::BuiltinProc;
use super::value::{LibraryId, PolyStructId};
use super::{
    Compiler, FileId, HashMap, HashSet, ModuleId, ProcId, ProcTypeParams, Rc, Result, ScopeId,
    Span, Sym, TypeId, Value, ast, err, ir, procs,
};
use crate::diagnostics::Code;
use crate::ir::{Builder, Ty, Val};
use crate::sema::Output;
use crate::sema::TypeState;
use crate::types::{ArrayKind, TypeKind};

/// Where a numeric constant came from: what is known about how it was written. Reading the
/// source text for this would break on macro output and on constants with no text, so the
/// fact travels with the value, and folding two values yields `Computed`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Origin {
    /// Folded or synthesised: nothing is known about its spelling.
    #[default]
    Computed,
    /// A literal, with the spelling it was parsed from.
    Literal(ast::Spelling),
    /// The complement of a constant (`~0`).
    Complement,
    /// A `#char` literal.
    Char,
    /// A declared constant, read by name.
    Named,
}

impl Origin {
    /// The constant may stand for a bit pattern of the type it meets: a hex or binary literal, a
    /// complement or a named constant, which are all written to say which bits are wanted.
    pub fn bit_pattern(self) -> bool {
        match self {
            Origin::Literal(spelling) => spelling == ast::Spelling::Radix,
            Origin::Complement | Origin::Char | Origin::Named => true,
            Origin::Computed => false,
        }
    }

    /// A decimal literal with a point (`16777217.0`): the digits written are the digits meant.
    pub fn decimal_fraction(self) -> bool {
        self == Origin::Literal(ast::Spelling::Fractional)
    }
}

/// Result of checking an expression.
#[derive(Clone, Debug)]
pub enum Operand {
    /// A callable value/place paired with its selected lexical header. This
    /// annotation is compile-time information and does not change the ABI.
    Callable {
        operand: Box<Operand>,
        parameters: Rc<ProcTypeParams>,
    },
    /// A runtime value. Scalars live in the register; memory-class types
    /// (structs, arrays, strings, Any) are represented by the address of a
    /// read-only copy.
    Value {
        ty: TypeId,
        val: Val,
    },
    /// An addressable location.
    Place {
        ty: TypeId,
        addr: Val,
    },
    /// A compile-time constant. `untyped` literals adapt to their context.
    Const {
        ty: TypeId,
        value: Value,
        untyped: bool,
        origin: Origin,
    },
    Type(TypeId),
    /// Candidates for a call (overload set).
    Procs(Vec<ProcId>),
    PolyStruct(PolyStructId),
    Module(ModuleId),
    Builtin(BuiltinProc),
    Library(LibraryId),
    /// Results of a multi-value call.
    Multi(Vec<Operand>),
    Void,
}

impl Operand {
    pub(super) fn with_callable(self, parameters: Option<Rc<ProcTypeParams>>) -> Self {
        match parameters {
            Some(parameters) => Self::Callable {
                operand: Box::new(self.without_callable()),
                parameters,
            },
            None => self,
        }
    }

    pub(crate) fn without_callable(self) -> Self {
        match self {
            Self::Callable {
                operand, ..
            } => *operand,
            other => other,
        }
    }

    pub fn ty(&self) -> TypeId {
        match self {
            Operand::Callable {
                operand, ..
            } => operand.ty(),
            Operand::Value {
                ty, ..
            }
            | Operand::Place {
                ty, ..
            }
            | Operand::Const {
                ty, ..
            } => *ty,
            Operand::Type(_) => TypeId::TYPE,
            Operand::Multi(v) => v.first().map_or(TypeId::VOID, Operand::ty),
            Operand::Void => TypeId::VOID,
            _ => TypeId::COMPILE_TIME,
        }
    }

    pub fn is_const(&self) -> bool {
        if let Operand::Callable {
            operand, ..
        } = self
        {
            return operand.is_const();
        }
        matches!(self, Operand::Const { .. } | Operand::Type(_))
    }

    pub fn const_value(&self) -> Option<Value> {
        match self {
            Operand::Callable {
                operand, ..
            } => operand.const_value(),
            Operand::Const {
                value, ..
            } => Some(value.clone()),
            Operand::Type(t) => Some(Value::Type(*t)),
            _ => None,
        }
    }

    pub fn int(value: i128, ty: TypeId) -> Operand {
        Operand::Const {
            ty,
            value: Value::Int(value),
            untyped: false,
            origin: Origin::Computed,
        }
    }

    /// A constant integer read through a name keeps no spelling, but a named constant may be
    /// written to say which bits it holds.
    pub(super) fn named_constant(self) -> Operand {
        match self {
            Operand::Const {
                ty,
                value: value @ Value::Int(_),
                untyped,
                origin: Origin::Computed,
            } => Operand::Const {
                ty,
                value,
                untyped,
                origin: Origin::Named,
            },
            other => other,
        }
    }

    /// The operand is a constant that may stand for a bit pattern (see [`Origin::bit_pattern`]).
    pub fn bit_pattern(&self) -> bool {
        match self {
            Operand::Callable {
                operand, ..
            } => operand.bit_pattern(),
            Operand::Const {
                origin, ..
            } => origin.bit_pattern(),
            _ => false,
        }
    }

    pub fn untyped_int(value: i128) -> Operand {
        Operand::Const {
            ty: TypeId::S64,
            value: Value::Int(value),
            untyped: true,
            origin: Origin::Computed,
        }
    }

    pub fn bool(value: bool) -> Operand {
        Operand::Const {
            ty: TypeId::BOOL,
            value: Value::Bool(value),
            untyped: false,
            origin: Origin::Computed,
        }
    }
}

/// Why a local cannot be assigned to.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Immutable {
    /// An aggregate procedure argument.
    Argument,
    /// The iterator of a by-value `for` over aggregate elements.
    Iterator,
}

#[derive(Clone)]
pub struct LoopFrame {
    pub label: Option<Sym>,
    pub break_block: ir::BlockId,
    pub continue_block: ir::BlockId,
    /// Defer depth at loop entry.
    pub defer_depth: CleanupMark,
    /// What `remove` does in this loop.
    pub remove: LoopRemove,
}

#[derive(Clone)]
pub enum LoopRemove {
    /// A `for` over a resizable array or an array view: the container place, the index slot,
    /// the element type and whether the loop runs in reverse.
    Array(Operand, Val, TypeId, bool),
    /// `remove` is an error in this loop; the message says why.
    Unsupported(String),
}

/// What `break`, `continue` and `remove` aimed at an inserted for-loop body become.
#[derive(Clone)]
pub struct InsertReplacements {
    /// Index in `FnCtx::loops` of the body's loop frame.
    pub loop_index: usize,
    /// Scope of the `#insert` (the macro), where replacements are checked.
    pub scope: ScopeId,
    pub break_: Option<ast::Expr>,
    pub continue_: Option<ast::Expr>,
    pub remove: Option<ast::Expr>,
}

#[derive(Clone)]
pub struct CleanupMark {
    position: Rc<std::cell::Cell<usize>>,
    serial: usize,
}

impl CleanupMark {
    pub fn position(&self) -> usize {
        self.position.get()
    }
}

#[derive(Clone)]
pub enum DeferAction {
    Statement(ast::Stmt),
    RestoreContext {
        previous: Option<Val>,
        saved_slot: Val,
    },
}

#[derive(Clone)]
pub struct DeferEntry {
    pub action: DeferAction,
    pub active: Option<Val>,
    pub scope: ScopeId,
    /// For a `` `defer `` in a macro: the caller's scope, where backtick names in the
    /// deferred code resolve once the macro has returned.
    pub caller_scope: Option<ScopeId>,
}

/// State for lowering one procedure body (or a compile-time thunk).
pub struct FnCtx {
    pub b: Builder,
    pub proc: Option<ProcId>,
    pub runtime_storage_constants: Vec<i64>,
    /// Current `context` pointer (absent in #`c_call/#no_context` code).
    context: Option<Val>,
    context_slot: Option<Val>,
    pub return_types: Vec<TypeId>,
    /// Out-pointers for aggregate results, by result index.
    pub return_outs: Vec<Option<Val>>,
    /// Named result slots (`-> (x: int)`), by result index.
    pub named_results: Vec<Option<Val>>,
    pub defers: Vec<DeferEntry>,
    _cleanup_root: CleanupMark,
    cleanup_marks: Vec<(usize, std::rc::Weak<std::cell::Cell<usize>>)>,
    next_cleanup_mark: usize,
    pub loops: Vec<LoopFrame>,
    /// Locals that cannot be assigned to, by storage address.
    pub immutable: Vec<(Val, Immutable)>,
    pub file: FileId,
    /// Set when the body runs only at compile time.
    pub compile_time: bool,
    /// Set while a constant is being evaluated: reading a variable is an error, with this
    /// text as the start of the message (`#run` bodies get their own context and may read).
    pub constant_only: Option<&'static str>,
    /// Macro expansion stack: (caller scope, return block, result slots, defer depth).
    pub macros: Vec<MacroFrame>,
    /// An expansion went too deep: every expansion fails at once until the outermost one
    /// returns, so a body that expands itself twice cannot keep a recovering check busy for
    /// 2^depth expansions.
    pub macro_overflow: bool,
    /// `#insert (break=..., continue=..., remove=...) body` of `for_expansion` macros being
    /// expanded (innermost last).
    pub insert_replacements: Vec<InsertReplacements>,
    /// Loop body handed to the next `for_expansion` macro expansion.
    pub pending_for_body: Option<ForBody>,
    /// Checking only for the type of an expression (`type_of`, `size_of`): no code is kept, so
    /// names that only exist as types, such as the fields of a struct being laid out, resolve.
    pub type_only: bool,
    /// Scope backtick names resolve in outside a macro (while a `` `defer `` runs).
    pub backtick_scope: Option<ScopeId>,
    /// Block constants already declared ahead of their statement (`check_block_stmts`), per block scope (a
    /// macro body expanded twice declares its constants in each expansion).
    pub hoisted_consts: crate::fxhash::HashSet<(ScopeId, ast::AstId)>,
    /// The checks turned off by `#no_abc` / `#no_aoc` on the procedure, an enclosing block or a loop.
    pub checks_off: ast::CheckBits,
    /// Debug scope index of each sema scope seen so far (`debug_info.rs`).
    pub debug_scopes: crate::fxhash::HashMap<ScopeId, u32>,
    /// The branch each `#if` statement of the body took, by statement span (`flow.rs`).
    /// A macro expanded in the body returned from it (a backtick `return`): the shape of the
    /// body no longer says whether every path returns.
    pub macro_returned: bool,
    /// The macro depth at which code written in the macro's caller is being inserted: a
    /// `return` there leaves the caller, like a backtick `return`.
    pub caller_code_macros: Option<usize>,
    pub static_taken: crate::fxhash::HashMap<Span, bool>,
}

#[derive(Clone)]
pub struct MacroFrame {
    pub caller_scope: ScopeId,
    pub exit_block: ir::BlockId,
    pub result_slots: Vec<(TypeId, Val)>,
    pub defer_depth: CleanupMark,
    pub caller_cleanup_serial: usize,
    pub loop_depth: usize,
    /// Caller's macro depth for nested backticks.
    pub caller_macro_depth: usize,
    /// For loop bodies inserted from `for_expansion`: name bindings of the loop.
    pub for_body: Option<ForBody>,
}

#[derive(Clone)]
pub struct ForBody {
    /// The actual custom-loop call site; overload selection may record its keyword.
    pub call_span: Span,
    /// The `Code` value standing for the loop body.
    pub code: super::value::CodeId,
    pub body: Rc<ast::Stmt>,
    pub scope: ScopeId,
    pub it_name: Sym,
    pub index_name: Sym,
    /// Where the loop names `it_name` / `index_name`, when it renames them.
    pub name_spans: [Option<Span>; 2],
    pub label: Option<Sym>,
}

impl FnCtx {
    pub fn context_binding(&self) -> Option<Val> {
        self.context
    }

    /// Restore lexical availability while compiling another control-flow path.
    /// Runtime restoration is emitted by the cleanup action, not this method.
    pub fn restore_context_binding(&mut self, binding: Option<Val>) {
        self.context = binding;
    }

    /// Context selection is runtime state: caller-owned macros can push a
    /// context on only one arm of a branch. SSA pointers chosen while walking
    /// the syntax cannot represent that join.
    pub fn current_context(&mut self) -> Option<Val> {
        self.context?;
        Some(
            self.b
                .load(Ty::Ptr, self.context_slot.expect("context storage")),
        )
    }

    pub fn set_context(&mut self, context: Option<Val>) {
        let slot = *self
            .context_slot
            .get_or_insert_with(|| self.b.entry_zeroed_slot(Ty::Ptr, 8, 8));
        let value = context.unwrap_or_else(|| self.b.iconst(Ty::Ptr, 0));
        self.b.store(Ty::Ptr, slot, value);
        self.context = context;
    }

    pub fn mark_defers(&mut self) -> CleanupMark {
        self.cleanup_marks
            .retain(|(_, mark)| mark.strong_count() != 0);
        let mark = CleanupMark {
            position: Rc::new(std::cell::Cell::new(self.defers.len())),
            serial: self.next_cleanup_mark,
        };
        self.next_cleanup_mark += 1;
        self.cleanup_marks
            .push((mark.serial, Rc::downgrade(&mark.position)));
        mark
    }

    pub fn current_cleanup_serial(&self) -> usize {
        self.cleanup_marks
            .iter()
            .rev()
            .find(|(_, m)| m.strong_count() != 0)
            .map_or(0, |(s, _)| *s)
    }

    /// Caller-owned cleanup is inserted before an expansion's local cleanups.
    /// All active nested boundaries move with that prefix; the caller's own
    /// boundary stays before the inserted entry. Raw depth snapshots lose this
    /// distinction when a macro inserts cleanup inside a nested block.
    pub fn insert_caller_defer(&mut self, at: usize, caller_serial: usize, entry: DeferEntry) {
        for (serial, weak) in &self.cleanup_marks {
            if *serial > caller_serial
                && let Some(mark) = weak.upgrade()
                && mark.get() >= at
            {
                mark.set(mark.get() + 1);
            }
        }
        self.defers.insert(at, entry);
    }

    /// Leave a lexical cleanup region while compiling the following source.
    /// Runtime exit paths emit the same restoration in cleanup order; this
    /// updates the compiler's current context without emitting duplicate code.
    pub fn truncate_defers(&mut self, depth: usize) {
        for entry in self.defers.iter().skip(depth).rev() {
            if let DeferAction::RestoreContext {
                previous, ..
            } = entry.action
            {
                self.context = previous;
            }
        }
        self.defers.truncate(depth);
    }

    pub fn new(name: String, sig: ir::Sig, file: FileId) -> Self {
        let root = CleanupMark {
            position: Rc::new(std::cell::Cell::new(0)),
            serial: 0,
        };
        Self {
            b: Builder::new(name, sig),
            proc: None,
            runtime_storage_constants: Vec::new(),
            context: None,
            context_slot: None,
            return_types: Vec::new(),
            return_outs: Vec::new(),
            named_results: Vec::new(),
            defers: Vec::new(),
            cleanup_marks: vec![(0, Rc::downgrade(&root.position))],
            _cleanup_root: root,
            next_cleanup_mark: 1,
            loops: Vec::new(),
            immutable: Vec::new(),
            file,
            compile_time: false,
            constant_only: None,
            macros: Vec::new(),
            macro_overflow: false,
            insert_replacements: Vec::new(),
            pending_for_body: None,
            type_only: false,
            backtick_scope: None,
            hoisted_consts: HashSet::default(),
            checks_off: ast::CheckBits::NONE,
            debug_scopes: HashMap::default(),
            macro_returned: false,
            caller_code_macros: None,
            static_taken: HashMap::default(),
        }
    }
}

impl Compiler {
    /// Scalar register class of a type, or `None` for memory-class types.
    pub fn ir_ty(&self, ty: TypeId) -> Option<Ty> {
        match self.ty.types.kind(ty) {
            TypeKind::Bool => Some(Ty::I8),
            TypeKind::Int {
                bits, ..
            } => Some(Ty::int(*bits as u64 / 8)),
            TypeKind::Float {
                bits: 32,
            } => Some(Ty::F32),
            TypeKind::Float {
                ..
            } => Some(Ty::F64),
            TypeKind::Pointer(_)
            | TypeKind::Proc(_)
            | TypeKind::Null
            | TypeKind::Type
            | TypeKind::Code => Some(Ty::Ptr),
            TypeKind::Enum(e) => {
                let base = self.ty.types.enum_info(*e).base;
                self.ir_ty(base)
            }
            TypeKind::Distinct(d) => self.ir_ty(self.ty.types.distincts[d.0 as usize].base),
            TypeKind::Void
            | TypeKind::CompileTimeOnly
            | TypeKind::PolyParam
            | TypeKind::PolyStruct {
                ..
            } => None,
            TypeKind::String
            | TypeKind::WideFloat(_)
            | TypeKind::Any
            | TypeKind::Array {
                ..
            }
            | TypeKind::Struct(_) => None,
        }
    }

    pub fn is_memory_type(&self, ty: TypeId) -> bool {
        self.ir_ty(ty).is_none() && ty != TypeId::VOID
    }

    /// Size/alignment, laying out structs on demand.
    pub fn size_of(&mut self, ty: TypeId, span: Span) -> Result<u64> {
        self.ensure_complete(ty, span)?;
        Ok(self.ty.types.size_of(ty))
    }

    pub fn align_of(&mut self, ty: TypeId, span: Span) -> Result<u64> {
        self.ensure_complete(ty, span)?;
        Ok(self.ty.types.align_of(ty))
    }

    /// Make sure every struct contained by value in `ty` is laid out.
    pub fn ensure_complete(&mut self, ty: TypeId, span: Span) -> Result<()> {
        match self.ty.types.kind(ty).clone() {
            TypeKind::Struct(s) => self.layout_struct(s, span),
            TypeKind::Array {
                elem,
                kind: ArrayKind::Fixed(n),
            } => {
                self.ensure_complete(elem, span)?;
                let size = self.ty.types.size_of(elem).checked_mul(n);
                if size.is_none_or(|size| size > crate::types::MAX_SIZE) {
                    return err(
                        span,
                        format!(
                            "array type {} is too large (the limit is {} bytes)",
                            self.ty.types.name(ty),
                            crate::types::MAX_SIZE
                        ),
                    );
                }
                Ok(())
            }
            TypeKind::Distinct(d) => {
                let base = self.ty.types.distincts[d.0 as usize].base;
                self.ensure_complete(base, span)
            }
            _ => Ok(()),
        }
    }

    /// Turn an operand into a runtime value (loading places, materializing constants).
    pub fn rvalue(&mut self, f: &mut FnCtx, op: Operand, span: Span) -> Result<(TypeId, Val)> {
        if self.is_register_operand_type(op.ty(), span) {
            return err(
                span,
                "a SIMD register operand cannot be loaded as a scalar; extract a lane in #asm",
            );
        }

        match op {
            Operand::Callable {
                operand, ..
            } => self.rvalue(f, *operand, span),
            Operand::Value {
                ty,
                val,
            } => Ok((ty, val)),
            Operand::Place {
                ty,
                addr,
            } => match self.ir_ty(ty) {
                Some(t) => Ok((ty, f.b.load(t, addr))),
                None => Ok((ty, addr)),
            },
            Operand::Const {
                ty,
                value,
                untyped,
                ..
            } => {
                let ty = if untyped {
                    self.ty.default_untyped(ty, &value)
                } else {
                    ty
                };
                let val = self.materialize(f, &value, ty, span)?;
                Ok((ty, val))
            }
            Operand::Type(t) => {
                self.require_value_type(t, span)?;
                let val = self.materialize(f, &Value::Type(t), TypeId::TYPE, span)?;
                Ok((TypeId::TYPE, val))
            }
            Operand::Procs(procs) => {
                if procs.len() != 1 {
                    return Err(Box::new(
                        crate::diagnostics::Diagnostic::error(span, "ambiguous overloaded procedure used as a value")
                            .with_code(Code::AmbiguousProcedureValue)
                            .with_label("two or more overloads of this name fit")
                            .with_help("name the overload you mean: give it its own name and use that, or declare the variable with the procedure type you want"),
                    ));
                }
                let proc = procs[0];
                let ty = self.proc_type(proc, span)?;
                let val = self.materialize(f, &Value::Proc(proc), ty, span)?;
                Ok((ty, val))
            }
            Operand::Multi(values) => self.rvalue(f, values[0].clone(), span),
            Operand::Void => err(span, "expression has no value"),
            Operand::Module(_) => err(span, "a module is not a value"),
            Operand::PolyStruct(ps) => {
                let t = self.ty.poly_struct_type(ps);
                let val = self.materialize(f, &Value::Type(t), TypeId::TYPE, span)?;
                Ok((TypeId::TYPE, val))
            }
            Operand::Builtin(_) => err(span, "builtin procedure is not a value"),
            Operand::Library(_) => err(span, "a library is not a value"),
        }
    }

    /// `#type (value: *$T)` names a shape that matches types: it is not itself a value.
    pub(in crate::sema) fn require_value_type(&self, ty: TypeId, span: Span) -> Result<()> {
        if self.ty.poly_aliases.contains(&ty) {
            return err(
                span,
                format!(
                    "the polymorphic type `{}` is a pattern, not a value: its type variables stand for any type",
                    self.ty.types.name(ty)
                ),
            );
        }
        Ok(())
    }

    /// Emit IR producing `value` of type `ty`.
    pub fn materialize(
        &mut self,
        f: &mut FnCtx,
        value: &Value,
        ty: TypeId,
        span: Span,
    ) -> Result<Val> {
        if let Some(fmt) = self.ty.types.wide_float(ty)
            && let Some(bytes) = Self::wide_const(fmt, value)
        {
            return self.materialize(f, &bytes, ty, span);
        }
        match value {
            Value::Int(v) => {
                if let Some(t) = self.ir_ty(ty) {
                    if t.is_float() {
                        return Ok(f.b.fconst(t, *v as f64));
                    }
                    return Ok(f.b.iconst(t, *v as u64));
                }
                err(
                    span,
                    format!(
                        "integer constant cannot have type {}",
                        self.ty.types.name(ty)
                    ),
                )
            }
            Value::Float(v, _) => {
                let Some(t) = self.ir_ty(ty).filter(|t| t.is_float()) else {
                    return err(
                        span,
                        format!("float constant cannot have type {}", self.ty.types.name(ty)),
                    );
                };
                Ok(f.b.fconst(t, *v))
            }
            Value::Bool(v) => Ok(f.b.iconst(Ty::I8, *v as u64)),
            Value::Null => Ok(f.b.iconst(Ty::Ptr, 0)),
            Value::String(bytes) => {
                let addr = f.b.alloca(16, 8);
                let count = f.b.iconst(Ty::I64, bytes.len() as u64);
                f.b.store(Ty::I64, addr, count);
                // `""` has a null data pointer (so e.g. `free("")` is a no-op).
                let ptr = if bytes.is_empty() {
                    f.b.iconst(Ty::Ptr, 0)
                } else {
                    let data = self.out.string_global(bytes);
                    f.b.global_addr(data)
                };
                let field = f.b.ptr_offset(addr, 8);
                f.b.store(Ty::Ptr, field, ptr);
                Ok(addr)
            }
            Value::Type(t) => {
                let global = self.type_info_global(*t, span)?;
                Ok(f.b.global_addr(global))
            }
            Value::Proc(p) => {
                let func = self.proc_func(*p, span)?;
                Ok(match func {
                    procs::ProcTarget::Func(id) => f.b.func_addr(id),
                    procs::ProcTarget::Foreign(id) => f.b.foreign_addr(id),
                })
            }
            Value::Bytes(agg) => {
                let size = self.size_of(ty, span)?;
                let align = self.align_of(ty, span)?;
                let global = self.out.program.add_global(ir::Global {
                    name: format!("const.{}", self.out.program.globals.len()),
                    size,
                    align,
                    init: agg.bytes.clone(),
                    relocs: agg.relocs.clone(),
                    read_only: true,
                    export: None,
                });
                if let Some(t) = self.ir_ty(ty) {
                    let addr = f.b.global_addr(global);
                    return Ok(f.b.load(t, addr));
                }
                Ok(f.b.global_addr(global))
            }
            Value::Code(c) => Ok(f.b.iconst(Ty::Ptr, c.0 as u64)),
            Value::Void => err(span, "void value used"),
        }
    }

    /// Store a runtime value of `ty` into memory at `addr`.
    pub fn store_value(
        &mut self,
        f: &mut FnCtx,
        ty: TypeId,
        addr: Val,
        val: Val,
        span: Span,
    ) -> Result<()> {
        if let Some(t) = self.ir_ty(ty) {
            f.b.store(t, addr, val);
        } else {
            let size = self.size_of(ty, span)?;
            if addr != val {
                f.b.copy(addr, val, size);
            }
        }
        Ok(())
    }

    /// Spill a value into a fresh stack slot, returning its address.
    pub fn spill(&mut self, f: &mut FnCtx, ty: TypeId, val: Val, span: Span) -> Result<Val> {
        let size = self.size_of(ty, span)?;
        let align = self.align_of(ty, span)?;
        let addr = f.b.alloca(size.max(1), align);
        self.store_value(f, ty, addr, val, span)?;
        Ok(addr)
    }

    /// Address of an operand, spilling values into temporaries.
    pub fn address_of(&mut self, f: &mut FnCtx, op: Operand, span: Span) -> Result<(TypeId, Val)> {
        match op {
            Operand::Callable {
                operand, ..
            } => self.address_of(f, *operand, span),
            Operand::Place {
                ty,
                addr,
            } => Ok((ty, addr)),
            other => {
                let (ty, val) = self.rvalue(f, other, span)?;
                if self.is_memory_type(ty) {
                    // Copy so the callee/user may not alias a read-only constant.
                    let tmp = self.spill(f, ty, val, span)?;
                    Ok((ty, tmp))
                } else {
                    Ok((ty, self.spill(f, ty, val, span)?))
                }
            }
        }
    }
}

impl TypeState {
    /// The natural type of an untyped literal.
    pub fn default_untyped(&self, ty: TypeId, value: &Value) -> TypeId {
        match value {
            Value::Float(_, p) if p.defaults_to_float64 || p.requires_float64 => TypeId::F64,
            // `float64` only for literals too precise for `float32` (`float_literal_type`).
            Value::Float(_, _) if ty == TypeId::F64 => ty,
            Value::Float(_, _) => TypeId::F32,
            Value::Int(_) if self.types.is_float(ty) => ty,
            Value::Int(_) => TypeId::S64,
            _ => ty,
        }
    }
}

impl Output {
    /// Read-only global holding the bytes of a string literal.
    pub fn string_global(&mut self, bytes: &Rc<[u8]>) -> ir::GlobalId {
        if let Some(&g) = self.strings.get(bytes) {
            return g;
        }
        let mut init = bytes.to_vec();
        init.push(0); // keep literals C-compatible
        let g = self.program.add_global(ir::Global {
            name: format!("str.{}", self.strings.len()),
            size: init.len() as u64,
            align: 1,
            init,
            relocs: Vec::new(),
            read_only: true,
            export: None,
        });
        self.strings.insert(bytes.clone(), g);
        g
    }
}
