//! Procedures and code bodies: their records, the queue of bodies waiting to be checked and
//! lowered, failures remembered per body, and per-body side tables.

use super::body_probe::{Probed, W};
use std::sync::Arc;

use crate::ast;
use crate::diagnostics::Diagnostic;
use crate::fxhash::{HashMap, HashSet};
use crate::intern::Sym;
use crate::ir;
use crate::source::Span;
use crate::types::{StructId, TypeId};

use super::{EntityId, ProcId, ProcTypeParams, ScopeId, Value, asm, calls, lower, procs};

#[derive(Default)]
pub struct ProcState {
    pub procs: Probed<Vec<procs::ProcInfo>, { W::PrProcs as usize }>,
    pub codes: Probed<Vec<Arc<ast::CodeBody>>, { W::PrCodes as usize }>,
    /// Runtime opt-in to compiler-host assertion faults, keyed by actual procedure identity.
    pub runtime_interpreter_assertion: Option<ProcId>,
    /// Bodies the lenient drain failed to lower, with the `lower_epoch` of the failure.
    pub(crate) lenient_failures:
        Probed<HashMap<ProcId, (u64, Box<Diagnostic>)>, { W::PrLenientFailures as usize }>,
    /// Errors an editor analysis recorded while a body failed to lower, until the body is
    /// lowered again: when a later attempt succeeds (what it waited for was declared since),
    /// they were provisional and are forgotten.
    pub(in crate::sema) provisional_errors:
        Probed<HashMap<ProcId, Vec<Diagnostic>>, { W::PrProvisionalErrors as usize }>,
    /// `$$x` variants: (procedure, which auto-bake params are baked) -> procedure.
    pub(in crate::sema) auto_bake_variants:
        Probed<HashMap<(ProcId, Vec<calls::AutoBake>), ProcId>, { W::PrAutoBakeVariants as usize }>,
    /// Every procedure whose body reached `Done`, in the order it did (a procedure can appear
    /// again if it is lowered again). Metaprogram exports read it from a cursor, so reporting
    /// finished bodies costs what is new, not every procedure of the program.
    pub(in crate::sema) done_log: Vec<ProcId>,
    /// Procedures that need their bodies lowered.
    pub body_queue: Probed<Vec<ProcId>, { W::PrBodyQueue as usize }>,
    /// Queued bodies whose last lenient attempt failed at `parked_epoch`: skipped as a group by
    /// `drain_bodies_lenient` until the epoch moves, instead of being looked at on every call.
    pub(crate) parked_bodies: Probed<Vec<ProcId>, { W::PrParkedBodies as usize }>,
    /// Bodies being lowered right now (nested through compile-time runs).
    pub(in crate::sema) lowering_depth: u32,
    /// Values of `a, b :: expr;` declarations, by declaration and scope.
    pub(in crate::sema) multi_consts:
        Probed<HashMap<(ast::AstId, ScopeId), Vec<lower::Operand>>, { W::PrMultiConsts as usize }>,
    /// Codes made by `code_of(procedure)`: their procedure, lowered before a typed export
    /// so its locals have types.
    pub(in crate::sema) code_procs: Probed<HashMap<usize, ProcId>, { W::PrCodeProcs as usize }>,
    pub(in crate::sema) parked_epoch: u64,
    /// The procedures targeting each IR function (`procs_of_func`).
    pub(in crate::sema) func_targets:
        Probed<HashMap<ir::FuncId, Vec<ProcId>>, { W::PrFuncTargets as usize }>,
    /// Procedures `compiler_make_procedure_live` asked to keep in the program though nothing
    /// calls them.
    pub live_procs: Probed<Vec<ProcId>, { W::PrLiveProcs as usize }>,
    pub entry_point: Option<ProcId>,
    pub exports: Probed<Vec<ProcId>, { W::PrExports as usize }>,
    /// Procedure literals and anonymous types, per (AST node, scope).
    pub anonymous_procs:
        Probed<HashMap<(ast::AstId, ScopeId), ProcId>, { W::PrAnonymousProcs as usize }>,
    /// Macro parameters whose argument was a compile-time constant: compile-time code in
    /// the macro (`#insert -> string { ... }`) reads them even though they are locals.
    /// User-written calls being emitted, innermost last: `#caller_code` evaluates to the
    /// innermost one. Only calls to procedures that use `#caller_code` are recorded.
    pub calls_in_flight: Probed<Vec<(Arc<ast::Expr>, ScopeId)>, { W::PrCallsInFlight as usize }>,
    pub local_consts: Probed<HashMap<EntityId, (Value, TypeId)>, { W::PrLocalConsts as usize }>,
    /// `#discard` parameters: naming one in the body is an error.
    pub discard_params: Probed<HashSet<EntityId>, { W::PrDiscardParams as usize }>,
    /// The most recently emitted call that has `#must` results: call span, procedure name, and
    /// which results are `#must`.
    pub last_call_must: Option<(Span, Sym, Vec<bool>)>,
    /// Macro parameters bound to a constant argument the macro never writes: reading them
    /// gives the constant (`#if n <= 1` in a recursive macro; `message.count` of a constant
    /// string fits a `u64` parameter).
    pub const_macro_params:
        Probed<HashMap<EntityId, (Value, TypeId)>, { W::PrConstMacroParams as usize }>,
    /// Types of local variable declarations, by declaration (first name), once lowered.
    /// Types of local declarations, by declaration and name index (`a, b := f()`).
    pub local_decl_types:
        Probed<HashMap<(ast::AstId, usize), TypeId>, { W::PrLocalDeclTypes as usize }>,
    /// Scope each `Code` value was written in (parallel to `codes`).
    pub code_scopes: Probed<Vec<ScopeId>, { W::PrCodeScopes as usize }>,
    /// Codes `compiler_get_code` made without a scope to copy: their names resolve where
    /// they are inserted.
    pub unscoped_codes: Probed<HashMap<usize, Vec<ScopeId>>, { W::PrUnscopedCodes as usize }>,
    /// Notes after a procedure declaration's body (`f :: () { ... } @thread`).
    pub proc_decl_notes: Probed<HashMap<ProcId, Vec<ast::Note>>, { W::PrProcDeclNotes as usize }>,
    /// String literals rewritten by `@Interpolates`, by span: their value is no longer the
    /// spelling in the source.
    pub interpolated: Probed<HashSet<Span>, { W::PrInterpolated as usize }>,
    /// `ifx` expressions written as an operand of a binary operator: the official compiler
    /// compares an `s64` one with a narrower integer, so they may stay constants of that type.
    pub ifx_operands: Probed<HashSet<Span>, { W::PrIfxOperands as usize }>,
    /// Calls that received their `#caller_location` by default, by that location (file,
    /// line, column): the span of their first argument. A failed `assert` names its call's
    /// location, so the report shows this as the condition.
    pub first_arguments: Probed<HashMap<(u32, u32, u32), Span>, { W::PrFirstArguments as usize }>,
    /// Interpreted function -> source procedure (reading back procedure values).
    pub func_procs: Probed<HashMap<ir::FuncId, ProcId>, { W::PrFuncProcs as usize }>,
    /// Locals declared by `#asm` register declarations (`x: gpr`).
    pub asm_regs: Probed<HashMap<EntityId, asm::AsmReg>, { W::PrAsmRegs as usize }>,
    /// Pinned general-purpose registers of the `#asm` block being lowered: (register number in
    /// x86 encoding order, address of its slot). Later pins of a register win.
    pub asm_pins: Probed<Vec<(u8, ir::Val)>, { W::PrAsmPins as usize }>,
    /// Callable metadata belongs to declarations, never structural `TypeIds`.
    pub(in crate::sema) callable_entities:
        Probed<HashMap<EntityId, Arc<ProcTypeParams>>, { W::PrCallableEntities as usize }>,
    pub(in crate::sema) callable_fields:
        Probed<HashMap<(StructId, usize), Arc<ProcTypeParams>>, { W::PrCallableFields as usize }>,
}
