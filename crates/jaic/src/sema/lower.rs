//! Operands and the per-procedure lowering context.
use super::scope::BuiltinProc;
use super::value::{LibraryId, PolyStructId};
use super::*;
use crate::ir::{Builder, Ty, Val};
use crate::types::{ArrayKind, TypeKind};

/// Result of checking an expression.
#[derive(Clone, Debug)]
pub enum Operand {
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
    },
    Type(TypeId),
    /// Candidates for a call (overload set).
    Procs(Vec<ProcId>),
    PolyStruct(PolyStructId),
    Module(ModuleId),
    Builtin(BuiltinProc),
    Library(LibraryId),
    /// Results of a multi-value call.
    Multi(Vec<(TypeId, Val)>),
    Void,
}

impl Operand {
    pub fn ty(&self) -> TypeId {
        match self {
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
            Operand::Multi(v) => v.first().map_or(TypeId::VOID, |(t, _)| *t),
            Operand::Void => TypeId::VOID,
            _ => TypeId::COMPILE_TIME,
        }
    }

    pub fn is_const(&self) -> bool {
        matches!(self, Operand::Const { .. } | Operand::Type(_))
    }

    pub fn const_value(&self) -> Option<Value> {
        match self {
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
        }
    }

    pub fn untyped_int(value: i128) -> Operand {
        Operand::Const {
            ty: TypeId::S64,
            value: Value::Int(value),
            untyped: true,
        }
    }

    pub fn bool(value: bool) -> Operand {
        Operand::Const {
            ty: TypeId::BOOL,
            value: Value::Bool(value),
            untyped: false,
        }
    }
}

#[derive(Clone)]
pub struct LoopFrame {
    pub label: Option<Sym>,
    pub break_block: ir::BlockId,
    pub continue_block: ir::BlockId,
    /// Defer depth at loop entry.
    pub defer_depth: usize,
    /// For `remove`: (container place, index slot, element type, reverse loop).
    pub remove: Option<(Operand, Val, TypeId, bool)>,
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
pub struct DeferEntry {
    pub stmt: ast::Stmt,
    pub scope: ScopeId,
    /// For a `` `defer `` in a macro: the caller's scope, where backtick names in the
    /// deferred code resolve once the macro has returned.
    pub caller_scope: Option<ScopeId>,
}

/// State for lowering one procedure body (or a compile-time thunk).
pub struct FnCtx {
    pub b: Builder,
    pub proc: Option<ProcId>,
    /// Current `context` pointer (absent in #c_call/#no_context code).
    pub context: Option<Val>,
    pub return_types: Vec<TypeId>,
    /// Out-pointers for aggregate results, by result index.
    pub return_outs: Vec<Option<Val>>,
    /// Named result slots (`-> (x: int)`), by result index.
    pub named_results: Vec<Option<Val>>,
    pub defers: Vec<DeferEntry>,
    pub loops: Vec<LoopFrame>,
    pub file: FileId,
    /// Set when the body runs only at compile time.
    pub compile_time: bool,
    /// Macro expansion stack: (caller scope, return block, result slots, defer depth).
    pub macros: Vec<MacroFrame>,
    /// `#insert (break=..., continue=..., remove=...) body` of for_expansion macros being
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
    /// Array bounds checks are off (`#no_abc` on the procedure or an enclosing `for`).
    pub no_abc: bool,
    /// Arithmetic overflow checks are off (`#no_aoc` on the procedure, a block or a loop).
    pub no_aoc: bool,
    /// Debug scope index of each sema scope seen so far (`debug_info.rs`).
    pub debug_scopes: crate::fxhash::HashMap<ScopeId, u32>,
}

#[derive(Clone)]
pub struct MacroFrame {
    pub caller_scope: ScopeId,
    pub exit_block: ir::BlockId,
    pub result_slots: Vec<(TypeId, Val)>,
    pub defer_depth: usize,
    pub loop_depth: usize,
    /// Caller's macro depth for nested backticks.
    pub caller_macro_depth: usize,
    /// For loop bodies inserted from `for_expansion`: name bindings of the loop.
    pub for_body: Option<ForBody>,
}

#[derive(Clone)]
pub struct ForBody {
    /// The `Code` value standing for the loop body.
    pub code: super::value::CodeId,
    pub body: Rc<ast::Stmt>,
    pub scope: ScopeId,
    pub it_name: Sym,
    pub index_name: Sym,
    pub label: Option<Sym>,
}

impl FnCtx {
    pub fn new(name: String, sig: ir::Sig, file: FileId) -> Self {
        Self {
            b: Builder::new(name, sig),
            proc: None,
            context: None,
            return_types: Vec::new(),
            return_outs: Vec::new(),
            named_results: Vec::new(),
            defers: Vec::new(),
            loops: Vec::new(),
            file,
            compile_time: false,
            macros: Vec::new(),
            insert_replacements: Vec::new(),
            pending_for_body: None,
            type_only: false,
            backtick_scope: None,
            hoisted_consts: Default::default(),
            no_abc: false,
            no_aoc: false,
            debug_scopes: Default::default(),
        }
    }
}

impl Compiler {
    /// Scalar register class of a type, or `None` for memory-class types.
    pub fn ir_ty(&self, ty: TypeId) -> Option<Ty> {
        match self.types.kind(ty) {
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
                let base = self.types.enum_info(*e).base;
                self.ir_ty(base)
            }
            TypeKind::Distinct(d) => self.ir_ty(self.types.distincts[d.0 as usize].base),
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
        Ok(self.types.size_of(ty))
    }

    pub fn align_of(&mut self, ty: TypeId, span: Span) -> Result<u64> {
        self.ensure_complete(ty, span)?;
        Ok(self.types.align_of(ty))
    }

    /// Make sure every struct contained by value in `ty` is laid out.
    pub fn ensure_complete(&mut self, ty: TypeId, span: Span) -> Result<()> {
        match self.types.kind(ty).clone() {
            TypeKind::Struct(s) => self.layout_struct(s, span),
            TypeKind::Array {
                elem,
                kind: ArrayKind::Fixed(n),
            } => {
                self.ensure_complete(elem, span)?;
                let size = self.types.size_of(elem).checked_mul(n);
                if size.is_none_or(|size| size > crate::types::MAX_SIZE) {
                    return err(
                        span,
                        format!(
                            "array type {} is too large (the limit is {} bytes)",
                            self.types.name(ty),
                            crate::types::MAX_SIZE
                        ),
                    );
                }
                Ok(())
            }
            TypeKind::Distinct(d) => {
                let base = self.types.distincts[d.0 as usize].base;
                self.ensure_complete(base, span)
            }
            _ => Ok(()),
        }
    }

    /// Turn an operand into a runtime value (loading places, materializing constants).
    pub fn rvalue(&mut self, f: &mut FnCtx, op: Operand, span: Span) -> Result<(TypeId, Val)> {
        match op {
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
            } => {
                let ty = if untyped {
                    self.default_untyped(ty, &value)
                } else {
                    ty
                };
                let val = self.materialize(f, &value, ty, span)?;
                Ok((ty, val))
            }
            Operand::Type(t) => {
                let val = self.materialize(f, &Value::Type(t), TypeId::TYPE, span)?;
                Ok((TypeId::TYPE, val))
            }
            Operand::Procs(procs) => {
                if procs.len() != 1 {
                    return err(span, "ambiguous overloaded procedure used as a value");
                }
                let proc = procs[0];
                let ty = self.proc_type(proc, span)?;
                let val = self.materialize(f, &Value::Proc(proc), ty, span)?;
                Ok((ty, val))
            }
            Operand::Multi(values) => Ok(values[0]),
            Operand::Void => err(span, "expression has no value"),
            Operand::Module(_) => err(span, "a module is not a value"),
            Operand::PolyStruct(ps) => {
                let t = self.poly_struct_type(ps);
                let val = self.materialize(f, &Value::Type(t), TypeId::TYPE, span)?;
                Ok((TypeId::TYPE, val))
            }
            Operand::Builtin(_) => err(span, "builtin procedure is not a value"),
            Operand::Library(_) => err(span, "a library is not a value"),
        }
    }

    /// The natural type of an untyped literal.
    pub fn default_untyped(&self, ty: TypeId, value: &Value) -> TypeId {
        match value {
            // `float64` only for literals too precise for `float32` (`float_literal_type`).
            Value::Float(_) if ty == TypeId::F64 => ty,
            Value::Float(_) => TypeId::F32,
            Value::Int(_) if self.types.is_float(ty) => ty,
            Value::Int(_) => TypeId::S64,
            _ => ty,
        }
    }

    /// Emit IR producing `value` of type `ty`.
    pub fn materialize(
        &mut self,
        f: &mut FnCtx,
        value: &Value,
        ty: TypeId,
        span: Span,
    ) -> Result<Val> {
        if let Some(fmt) = self.wide_float(ty)
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
                    format!("integer constant cannot have type {}", self.types.name(ty)),
                )
            }
            Value::Float(v) => {
                let t = self.ir_ty(ty).filter(|t| t.is_float()).unwrap_or(Ty::F64);
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
                    let data = self.string_global(bytes);
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
                let global = self.program.add_global(ir::Global {
                    name: format!("const.{}", self.program.globals.len()),
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

    /// Store a runtime value of `ty` into memory at `addr`.
    pub fn store_value(
        &mut self,
        f: &mut FnCtx,
        ty: TypeId,
        addr: Val,
        val: Val,
        span: Span,
    ) -> Result<()> {
        match self.ir_ty(ty) {
            Some(t) => f.b.store(t, addr, val),
            None => {
                let size = self.size_of(ty, span)?;
                if addr != val {
                    f.b.copy(addr, val, size);
                }
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
