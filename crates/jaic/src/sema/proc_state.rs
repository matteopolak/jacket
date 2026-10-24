//! Procedures and code bodies: their records, the queue of bodies waiting to be checked and
//! lowered, failures remembered per body, and per-body side tables.

use std::rc::Rc;

use crate::ast;
use crate::fxhash::{HashMap, HashSet};
use crate::intern::Sym;
use crate::ir;
use crate::source::{Diagnostic, Span};
use crate::types::{StructId, TypeId};

use super::{EntityId, ProcId, ProcTypeParams, ScopeId, Value, asm, calls, lower, procs};

#[derive(Default)]
pub struct ProcState {
    pub procs: Vec<procs::ProcInfo>,
    pub codes: Vec<Rc<ast::CodeBody>>,
    /// Runtime opt-in to compiler-host assertion faults, keyed by actual procedure identity.
    pub runtime_interpreter_assertion: Option<ProcId>,
    /// Bodies the lenient drain failed to lower, with the `lower_epoch` of the failure.
    pub(crate) lenient_failures: HashMap<ProcId, (u64, Box<Diagnostic>)>,
    /// Errors an editor analysis recorded while a body failed to lower, until the body is
    /// lowered again: when a later attempt succeeds (what it waited for was declared since),
    /// they were provisional and are forgotten.
    pub(in crate::sema) provisional_errors: HashMap<ProcId, Vec<Diagnostic>>,
    /// `$$x` variants: (procedure, which auto-bake params are baked) -> procedure.
    pub(in crate::sema) auto_bake_variants: HashMap<(ProcId, Vec<calls::AutoBake>), ProcId>,
    /// Procedures that need their bodies lowered.
    pub body_queue: Vec<ProcId>,
    /// Queued bodies whose last lenient attempt failed at `parked_epoch`: skipped as a group by
    /// `drain_bodies_lenient` until the epoch moves, instead of being looked at on every call.
    pub(crate) parked_bodies: Vec<ProcId>,
    /// Bodies being lowered right now (nested through compile-time runs).
    pub(in crate::sema) lowering_depth: u32,
    /// Values of `a, b :: expr;` declarations, by declaration and scope.
    pub(in crate::sema) multi_consts: HashMap<(ast::AstId, ScopeId), Vec<lower::Operand>>,
    /// Codes made by `code_of(procedure)`: their procedure, lowered before a typed export
    /// so its locals have types.
    pub(in crate::sema) code_procs: HashMap<usize, ProcId>,
    pub(in crate::sema) parked_epoch: u64,
    /// The procedures targeting each IR function (`procs_of_func`).
    pub(in crate::sema) func_targets: HashMap<ir::FuncId, Vec<ProcId>>,
    /// Procedures `compiler_make_procedure_live` asked to keep in the program though nothing
    /// calls them.
    pub live_procs: Vec<ProcId>,
    pub entry_point: Option<ProcId>,
    pub exports: Vec<ProcId>,
    /// Procedure literals and anonymous types, per (AST node, scope).
    pub anonymous_procs: HashMap<(ast::AstId, ScopeId), ProcId>,
    /// Macro parameters whose argument was a compile-time constant: compile-time code in
    /// the macro (`#insert -> string { ... }`) reads them even though they are locals.
    /// User-written calls being emitted, innermost last: `#caller_code` evaluates to the
    /// innermost one. Only calls to procedures that use `#caller_code` are recorded.
    pub calls_in_flight: Vec<(Rc<ast::Expr>, ScopeId)>,
    pub local_consts: HashMap<EntityId, (Value, TypeId)>,
    /// `#discard` parameters: naming one in the body is an error.
    pub discard_params: HashSet<EntityId>,
    /// The most recently emitted call that has `#must` results: call span, procedure name, and
    /// which results are `#must`.
    pub last_call_must: Option<(Span, Sym, Vec<bool>)>,
    /// Macro parameters bound to a constant argument the macro never writes: reading them
    /// gives the constant (`#if n <= 1` in a recursive macro; `message.count` of a constant
    /// string fits a `u64` parameter).
    pub const_macro_params: HashMap<EntityId, (Value, TypeId)>,
    /// Types of local variable declarations, by declaration (first name), once lowered.
    /// Types of local declarations, by declaration and name index (`a, b := f()`).
    pub local_decl_types: HashMap<(ast::AstId, usize), TypeId>,
    /// Scope each `Code` value was written in (parallel to `codes`).
    pub code_scopes: Vec<ScopeId>,
    /// Codes `compiler_get_code` made without a scope to copy: their names resolve where
    /// they are inserted.
    pub unscoped_codes: HashMap<usize, Vec<ScopeId>>,
    /// Notes after a procedure declaration's body (`f :: () { ... } @thread`).
    pub proc_decl_notes: HashMap<ProcId, Vec<ast::Note>>,
    /// String literals rewritten by `@Interpolates`, by span: their value is no longer the
    /// spelling in the source.
    pub interpolated: HashSet<Span>,
    /// Calls that received their `#caller_location` by default, by that location (file,
    /// line, column): the span of their first argument. A failed `assert` names its call's
    /// location, so the report shows this as the condition.
    pub first_arguments: HashMap<(u32, u32, u32), Span>,
    /// Interpreted function -> source procedure (reading back procedure values).
    pub func_procs: HashMap<ir::FuncId, ProcId>,
    /// Locals declared by `#asm` register declarations (`x: gpr`).
    pub asm_regs: HashMap<EntityId, asm::AsmReg>,
    /// Pinned general-purpose registers of the `#asm` block being lowered: (register number in
    /// x86 encoding order, address of its slot). Later pins of a register win.
    pub asm_pins: Vec<(u8, ir::Val)>,
    /// Callable metadata belongs to declarations, never structural `TypeIds`.
    pub(in crate::sema) callable_entities: HashMap<EntityId, Rc<ProcTypeParams>>,
    pub(in crate::sema) callable_fields: HashMap<(StructId, usize), Rc<ProcTypeParams>>,
}
