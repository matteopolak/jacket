//! Validate staging graphs once before exposing an immutable compilation.
mod context;
mod control;
mod expression_bindings;
mod expressions;
mod floats;
mod pointers;
mod procedures;
mod sequences;
mod static_closures;
use crate::*;
pub(crate) use expressions::constant;
use jai_types::{IntegerType, ScalarType, TypeId, TypeKind, TypeView};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    rc::Rc,
};

/// Public staging graphs can be deeper than source-generated expressions.
const MAX_VERIFICATION_DEPTH: usize = 128;
const MAX_CONSTANT_DEPTH: usize = 256;
struct DepthGuard(Rc<Cell<usize>>);
impl Drop for DepthGuard {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1);
    }
}

fn unknown(kind: &'static str, index: usize) -> IrError {
    IrError::UnknownIdentity {
        kind,
        index,
    }
}
fn arity(kind: &'static str, expected: usize, actual: usize) -> Result<(), IrError> {
    if expected == actual {
        Ok(())
    } else {
        Err(IrError::Arity {
            kind,
            expected,
            actual,
        })
    }
}
fn same_type(expected: TypeId, actual: TypeId) -> Result<(), IrError> {
    if expected == actual {
        Ok(())
    } else {
        Err(IrError::TypeMismatch {
            expected,
            actual,
        })
    }
}
fn same_integer(expected: IntegerType, actual: IntegerType) -> Result<(), IrError> {
    if expected == actual {
        Ok(())
    } else {
        Err(IrError::IntegerMismatch {
            expected,
            actual,
        })
    }
}

fn global_identities(types: &dyn TypeView, globals: &[Global]) -> Result<(), IrError> {
    for (index, global) in globals.iter().enumerate() {
        if global.id().index() != index {
            return Err(unknown("global", global.id().index()));
        }
        if matches!(types.kind(global.ty())?, TypeKind::Void | TypeKind::Code) {
            return Err(IrError::InvalidValue(global.ty()));
        }
    }
    Ok(())
}
fn global_initializer(
    types: &dyn TypeView,
    global: &Global,
    signatures: &HashMap<ProcedureId, TypeId>,
    closures: &mut static_closures::StaticClosures,
) -> Result<(), IrError> {
    let scalar;
    let value = match global.initializer() {
        GlobalInitializer::External(data) => {
            same_type(global.ty(), data.ty())?;
            return data.validate(types).map_err(IrError::from);
        }
        GlobalInitializer::Int(value) => {
            scalar = ConstantValue {
                ty: global.ty(),
                kind: ConstantKind::Int(*value),
            };
            &scalar
        }
        GlobalInitializer::Bool(value) => {
            scalar = ConstantValue {
                ty: global.ty(),
                kind: ConstantKind::Bool(*value),
            };
            &scalar
        }
        GlobalInitializer::Value(value) => {
            same_type(global.ty(), value.ty)?;
            value
        }
    };
    expressions::constant_with_closures(types, value, closures)?;
    expressions::constant_procedures_with_closures(types, value, signatures, closures)
}
fn globals(
    types: &dyn TypeView,
    globals: &[Global],
    signatures: &HashMap<ProcedureId, TypeId>,
) -> Result<(), IrError> {
    global_identities(types, globals)?;
    let mut closures = static_closures::StaticClosures::default();
    for global in globals {
        global_initializer(types, global, signatures, &mut closures)?;
    }
    Ok(())
}

/// Validate only constant procedure leaves against an immutable signature ledger.
/// This does not demand unrelated nominal definitions or prove the rest of the value.
pub fn verify_constant_procedures(
    types: &dyn TypeView,
    value: &ConstantValue,
    signatures: &HashMap<ProcedureId, TypeId>,
) -> Result<(), IrError> {
    expressions::constant_procedures(types, value, signatures)
}

pub(super) fn library(library: &Library) -> Result<(), IrError> {
    crate::program_exports::validate(library, &library.program_exports)
        .map_err(IrError::ProgramExport)?;
    if let Some(context) = &library.context {
        context::definition(context, &library.types, &library.signatures)?;
    }
    globals(&library.types, &library.globals, &library.signatures)?;
    library
        .source_procedure_owners
        .validate(
            &library.types,
            &library.signatures,
            &library.globals,
            library.context.as_ref(),
        )
        .map_err(|error| IrError::SourceProcedureOwner(Box::new(error)))?;
    procedures::declarations(library)?;
    crate::external_data::verify::verify_library(library).map_err(IrError::from)?;
    for procedure in &library.procedures {
        procedure_body(
            &library.types,
            procedure,
            &library.signatures,
            &library.globals,
            &library.places,
            library.context.as_ref(),
        )?;
    }
    Ok(())
}

pub(super) fn entry(library: &Library, entry: EntryPoint) -> Result<(), IrError> {
    let id = match entry {
        EntryPoint::Void(id) | EntryPoint::Int(id) => id,
    };
    let procedure = library
        .procedure_by_id(id)
        .ok_or_else(|| unknown("entry", id.index()))?;
    let signature = library.types.procedure_definition(procedure.signature)?;
    let valid = signature.parameters.is_empty()
        && match entry {
            EntryPoint::Void(_) => signature.results.is_empty(),
            EntryPoint::Int(_) => {
                signature.results.as_ref()
                    == [library.types.scalar(ScalarType::Int(IntegerType::S64))]
            }
        };
    if valid {
        Ok(())
    } else {
        Err(IrError::InvalidEntry(id))
    }
}

/// Check a ready procedure without waiting for unrelated types or callee bodies.
pub fn verify_procedure<'a>(
    types: &'a dyn TypeView,
    procedure: &'a Procedure,
    signatures: &'a HashMap<ProcedureId, TypeId>,
    global_values: &'a [Global],
    places: &'a Places,
) -> Result<CheckedProcedure<'a>, IrError> {
    verify_procedure_with_context(types, procedure, signatures, global_values, places, None)
}
pub fn verify_procedure_with_context<'a>(
    types: &'a dyn TypeView,
    procedure: &'a Procedure,
    signatures: &'a HashMap<ProcedureId, TypeId>,
    global_values: &'a [Global],
    places: &'a Places,
    context: Option<&'a ContextDefinition>,
) -> Result<CheckedProcedure<'a>, IrError> {
    if let Some(context) = context {
        context::definition(context, types, signatures)?;
    }
    global_identities(types, global_values)?;
    procedure_body(types, procedure, signatures, global_values, places, context)?;
    Ok(CheckedProcedure {
        procedure,
        types,
        signatures,
        globals: global_values,
        places,
        context,
    })
}

pub struct CheckedExpression<'a> {
    binding_owner: Option<ProcedureId>,
    expression: &'a ValueExpr,
    types: &'a dyn TypeView,
    signatures: &'a HashMap<ProcedureId, TypeId>,
    globals: &'a [Global],
    places: &'a Places,
    context: Option<&'a ContextDefinition>,
}
impl<'a> CheckedExpression<'a> {
    pub fn binding_owner(&self) -> Option<ProcedureId> {
        self.binding_owner
    }
    pub fn expression(&self) -> &'a ValueExpr {
        self.expression
    }
    pub fn types(&self) -> &'a dyn TypeView {
        self.types
    }
    pub fn signatures(&self) -> &'a HashMap<ProcedureId, TypeId> {
        self.signatures
    }
    pub fn globals(&self) -> &'a [Global] {
        self.globals
    }
    pub fn places(&self) -> &'a Places {
        self.places
    }
    pub fn context(&self) -> Option<&'a ContextDefinition> {
        self.context
    }
}
pub struct CheckedCall<'a> {
    binding_owner: Option<ProcedureId>,
    call: &'a Call,
    results: Vec<TypeId>,
    types: &'a dyn TypeView,
    signatures: &'a HashMap<ProcedureId, TypeId>,
    globals: &'a [Global],
    places: &'a Places,
    context: Option<&'a ContextDefinition>,
}
impl<'a> CheckedCall<'a> {
    pub fn binding_owner(&self) -> Option<ProcedureId> {
        self.binding_owner
    }
    pub fn call(&self) -> &'a Call {
        self.call
    }
    pub fn results(&self) -> &[TypeId] {
        &self.results
    }
    pub fn types(&self) -> &'a dyn TypeView {
        self.types
    }
    pub fn signatures(&self) -> &'a HashMap<ProcedureId, TypeId> {
        self.signatures
    }
    pub fn globals(&self) -> &'a [Global] {
        self.globals
    }
    pub fn places(&self) -> &'a Places {
        self.places
    }
    pub fn context(&self) -> Option<&'a ContextDefinition> {
        self.context
    }
}
/// Validate a compile-time root expression, with no procedure-local frame.
pub fn verify_expression<'a>(
    types: &'a dyn TypeView,
    expression: &'a ValueExpr,
    signatures: &'a HashMap<ProcedureId, TypeId>,
    globals: &'a [Global],
    places: &'a Places,
) -> Result<CheckedExpression<'a>, IrError> {
    verify_expression_with_context(types, expression, signatures, globals, places, None)
}
pub fn verify_expression_with_context<'a>(
    types: &'a dyn TypeView,
    expression: &'a ValueExpr,
    signatures: &'a HashMap<ProcedureId, TypeId>,
    globals: &'a [Global],
    places: &'a Places,
    context: Option<&'a ContextDefinition>,
) -> Result<CheckedExpression<'a>, IrError> {
    verify_root_expression(
        types, expression, signatures, globals, places, None, context,
    )
}
/// The caller selects the owner from its actual semantic compile-time context.
pub fn verify_owned_expression_with_context<'a>(
    types: &'a dyn TypeView,
    expression: &'a ValueExpr,
    signatures: &'a HashMap<ProcedureId, TypeId>,
    globals: &'a [Global],
    places: &'a Places,
    owner: ProcedureId,
    context: Option<&'a ContextDefinition>,
) -> Result<CheckedExpression<'a>, IrError> {
    verify_root_expression(
        types,
        expression,
        signatures,
        globals,
        places,
        Some(owner),
        context,
    )
}
fn verify_root_expression<'a>(
    types: &'a dyn TypeView,
    expression: &'a ValueExpr,
    signatures: &'a HashMap<ProcedureId, TypeId>,
    globals: &'a [Global],
    places: &'a Places,
    owner: Option<ProcedureId>,
    context: Option<&'a ContextDefinition>,
) -> Result<CheckedExpression<'a>, IrError> {
    if let Some(context) = context {
        context::definition(context, types, signatures)?;
    }
    global_identities(types, globals)?;
    let mut proof = Context::root(types, signatures, globals, places, context);
    proof.binding_owner = owner;
    proof.value(expression)?;
    Ok(CheckedExpression {
        binding_owner: owner,
        expression,
        types,
        signatures,
        globals,
        places,
        context,
    })
}
/// Validate ordered root-call arguments before evaluation can perform effects.
pub fn verify_call<'a>(
    types: &'a dyn TypeView,
    call: &'a Call,
    signatures: &'a HashMap<ProcedureId, TypeId>,
    globals: &'a [Global],
    places: &'a Places,
) -> Result<CheckedCall<'a>, IrError> {
    verify_call_with_context(types, call, signatures, globals, places, None)
}
pub fn verify_call_with_context<'a>(
    types: &'a dyn TypeView,
    call: &'a Call,
    signatures: &'a HashMap<ProcedureId, TypeId>,
    globals: &'a [Global],
    places: &'a Places,
    context: Option<&'a ContextDefinition>,
) -> Result<CheckedCall<'a>, IrError> {
    verify_root_call(types, call, signatures, globals, places, None, context)
}
/// Owned roots cannot use captures allocated for a different source context.
pub fn verify_owned_call_with_context<'a>(
    types: &'a dyn TypeView,
    call: &'a Call,
    signatures: &'a HashMap<ProcedureId, TypeId>,
    globals: &'a [Global],
    places: &'a Places,
    owner: ProcedureId,
    context: Option<&'a ContextDefinition>,
) -> Result<CheckedCall<'a>, IrError> {
    verify_root_call(
        types,
        call,
        signatures,
        globals,
        places,
        Some(owner),
        context,
    )
}
fn verify_root_call<'a>(
    types: &'a dyn TypeView,
    call: &'a Call,
    signatures: &'a HashMap<ProcedureId, TypeId>,
    globals: &'a [Global],
    places: &'a Places,
    owner: Option<ProcedureId>,
    context: Option<&'a ContextDefinition>,
) -> Result<CheckedCall<'a>, IrError> {
    if let Some(context) = context {
        context::definition(context, types, signatures)?;
    }
    global_identities(types, globals)?;
    let mut proof = Context::root(types, signatures, globals, places, context);
    proof.binding_owner = owner;
    let results = proof.call(call)?.to_vec();
    Ok(CheckedCall {
        binding_owner: owner,
        call,
        results,
        types,
        signatures,
        globals,
        places,
        context,
    })
}

fn procedure_body(
    types: &dyn TypeView,
    procedure: &Procedure,
    signatures: &HashMap<ProcedureId, TypeId>,
    globals: &[Global],
    places: &Places,
    schema: Option<&ContextDefinition>,
) -> Result<(), IrError> {
    let &declared = signatures
        .get(&procedure.id)
        .ok_or_else(|| unknown("ready procedure", procedure.id.index()))?;
    same_type(declared, procedure.signature)?;
    let signature = types.procedure_definition(procedure.signature)?;
    arity(
        "procedure parameters",
        signature.parameters.len(),
        procedure.parameters.len(),
    )?;
    for &ty in signature.parameters.iter().chain(&signature.results) {
        storage::runtime_type(types, ty)?;
    }
    for (index, local) in procedure.locals.iter().enumerate() {
        if local.id().procedure() != procedure.id {
            return Err(IrError::LocalOwner {
                expected: procedure.id,
                actual: local.id().procedure(),
            });
        }
        if local.id().index() != index {
            return Err(unknown("local", local.id().index()));
        }
        storage::runtime_type(types, local.ty())?;
    }
    control::cleanup_dependencies(procedure)?;
    let pushes = context::push_definitions(procedure)?;
    let mut context = Context::root(types, signatures, globals, places, schema);
    context.procedure = Some(procedure);
    context.binding_owner = Some(procedure.id);
    context.context_available = signature.context == jai_types::ContextMode::Implicit;
    context.push_definitions = Some(&pushes);
    let mut parameters = HashSet::new();
    for (parameter, &ty) in procedure.parameters.iter().zip(&signature.parameters) {
        context.place(parameter.place())?;
        same_type(ty, parameter.ty())?;
        if !parameters.insert(parameter.id()) {
            return Err(IrError::DuplicateIdentity {
                kind: "parameter local",
                index: parameter.id().index(),
            });
        }
    }
    context.block(&procedure.body)?;
    if !signature.results.is_empty() && procedure.body.flow != Flow::Terminates {
        return Err(IrError::InvalidFlow);
    }
    context.cleanup = true;
    for cleanup in &procedure.cleanups {
        context.active_push = context::capture_chain(cleanup.context, &pushes)?;
        context.context_available = match cleanup.context {
            CleanupContext::Procedure => signature.context == jai_types::ContextMode::Implicit,
            CleanupContext::Push(_) => true,
        };
        context.block(&cleanup.body)?;
    }
    Ok(())
}

struct Context<'a> {
    binding_owner: Option<ProcedureId>,
    expression_bindings: RefCell<HashMap<ExpressionBindingId, TypeId>>,
    types: &'a dyn TypeView,
    signatures: &'a HashMap<ProcedureId, TypeId>,
    globals: &'a [Global],
    places: &'a Places,
    procedure: Option<&'a Procedure>,
    loops: Vec<LoopId>,
    loop_ids: HashSet<LoopId>,
    cleanup: bool,
    depth: Rc<Cell<usize>>,
    context: Option<&'a ContextDefinition>,
    context_available: bool,
    push_definitions: Option<&'a HashMap<PushContextId, Option<PushContextId>>>,
    active_push: Vec<PushContextId>,
    static_closures: RefCell<static_closures::StaticClosures>,
    checked_globals: RefCell<HashSet<GlobalId>>,
}

impl<'a> Context<'a> {
    fn root(
        types: &'a dyn TypeView,
        signatures: &'a HashMap<ProcedureId, TypeId>,
        globals: &'a [Global],
        places: &'a Places,
        context: Option<&'a ContextDefinition>,
    ) -> Self {
        Self {
            binding_owner: None,
            expression_bindings: RefCell::default(),
            types,
            signatures,
            globals,
            places,
            procedure: None,
            loops: vec![],
            loop_ids: HashSet::new(),
            cleanup: false,
            depth: Rc::new(Cell::new(0)),
            context,
            context_available: true,
            push_definitions: None,
            active_push: vec![],
            static_closures: RefCell::default(),
            checked_globals: RefCell::default(),
        }
    }
    fn enter(&self) -> Result<DepthGuard, IrError> {
        let depth = self.depth.get();
        if depth >= MAX_VERIFICATION_DEPTH {
            return Err(IrError::VerificationDepth);
        }
        self.depth.set(depth + 1);
        Ok(DepthGuard(Rc::clone(&self.depth)))
    }
    fn place(&self, mut place: Place) -> Result<TypeId, IrError> {
        let _depth = self.enter()?;
        let result = place.ty();
        storage::runtime_type(self.types, result)?;
        loop {
            match place.kind() {
                PlaceKind::Context(ty) => {
                    same_type(self.context_type()?, ty)?;
                    same_type(ty, place.ty())?;
                    return Ok(result);
                }
                PlaceKind::Local(id) => {
                    let procedure = self
                        .procedure
                        .ok_or_else(|| unknown("expression local", id.index()))?;
                    if id.procedure() != procedure.id {
                        return Err(IrError::LocalOwner {
                            expected: procedure.id,
                            actual: id.procedure(),
                        });
                    }
                    let local = procedure
                        .locals
                        .get(id.index())
                        .ok_or_else(|| unknown("local", id.index()))?;
                    same_type(local.ty(), place.ty())?;
                    return Ok(result);
                }
                PlaceKind::Global(id) => {
                    let global = self
                        .globals
                        .get(id.index())
                        .ok_or_else(|| unknown("global", id.index()))?;
                    same_type(global.ty(), place.ty())?;
                    if !self.checked_globals.borrow().contains(&id) {
                        global_initializer(
                            self.types,
                            global,
                            self.signatures,
                            &mut self.static_closures.borrow_mut(),
                        )?;
                        self.checked_globals.borrow_mut().insert(id);
                    }
                    return Ok(result);
                }
                PlaceKind::Field(id) => {
                    let projection = self.places.projection(id)?;
                    let ty = self
                        .types
                        .validate_field(projection.base.ty(), projection.field)?;
                    same_type(ty, place.ty())?;
                    // The registry accepts only already-created bases, so ordinals descend.
                    if let PlaceKind::Field(base) = projection.base.kind()
                        && base.index() >= id.index()
                    {
                        return Err(unknown("projection dependency", base.index()));
                    }
                    place = projection.base;
                }
                PlaceKind::Dereference(id) => {
                    let projection = self.places.dereference(id)?;
                    let pointer = self.value(&projection.pointer)?;
                    let TypeKind::Pointer(ty) = *self.types.kind(pointer)? else {
                        return Err(IrError::InvalidValue(pointer));
                    };
                    same_type(ty, place.ty())?;
                    return Ok(result);
                }
                PlaceKind::Index(id) => {
                    let projection = self.places.index(id)?;
                    let base = self.place(projection.base)?;
                    self.integer(&projection.index)?;
                    same_integer(
                        crate::canonical_index_type(projection.index.ty()),
                        projection.index.ty(),
                    )?;
                    same_type(crate::sequences::element(self.types, base)?, place.ty())?;
                    return Ok(result);
                }
                PlaceKind::SequenceField(id) => {
                    let projection = self.places.sequence_field(id)?;
                    let base = self.place(projection.base)?;
                    if !matches!(
                        self.types.kind(base)?,
                        TypeKind::Slice(_) | TypeKind::DynamicArray(_) | TypeKind::String
                    ) {
                        return Err(IrError::InvalidValue(base));
                    }
                    same_type(
                        crate::sequences::field_type(self.types, base, projection.field)?,
                        place.ty(),
                    )?;
                    return Ok(result);
                }
            }
        }
    }
    fn integer_place(&self, place: IntPlace) -> Result<(), IrError> {
        let ty = self.place(place.place())?;
        same_type(self.types.scalar(ScalarType::Int(place.ty())), ty)
    }
    fn boolean_place(&self, place: BoolPlace) -> Result<(), IrError> {
        let ty = self.place(place.place())?;
        same_type(self.types.scalar(ScalarType::Bool), ty)
    }
    fn call(&self, call: &Call) -> Result<&[TypeId], IrError> {
        let _depth = self.enter()?;
        let &signature_type = self
            .signatures
            .get(&call.procedure)
            .ok_or_else(|| unknown("called procedure", call.procedure.index()))?;
        let signature = self.types.procedure_definition(signature_type)?;
        self.call_context(signature)?;
        for &ty in signature.parameters.iter().chain(&signature.results) {
            storage::runtime_type(self.types, ty)?;
        }
        self.call_arguments(signature, &call.arguments)?;
        Ok(&signature.results)
    }
    fn single_call(&self, call: &Call, expected: TypeId) -> Result<(), IrError> {
        let results = self.call(call)?;
        arity("call results", 1, results.len())?;
        same_type(expected, results[0])
    }
}

pub(crate) mod compiler_bindings;
