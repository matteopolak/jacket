//! Construct and verify LLVM modules from immutable checked programs.
pub mod abi;
mod aggregates;
mod any_values;
mod arithmetic;
mod cases;
mod context;
mod cpp_methods;
pub mod debug;
mod execution_phase;
mod expression_bindings;
mod external_data;
mod floats;
pub mod foreign;
mod inline_hints;
mod memory;
mod native_intrinsics;
mod native_pointer_constants;
pub mod native_reachability;
mod native_simd;
pub mod optimization;
mod pointer_conversions;
mod pointers;
mod program_exports;
mod records;
mod sequence_pack_lifetimes;
mod sequence_packs;
mod sequences;
mod simd_dispatch;
mod static_data;
pub mod storage_alignment;
mod storage_reinterpretation;
mod string_comparison;
pub mod target;
#[cfg(test)]
#[path = "../tests/support/native_tools.rs"]
#[allow(dead_code)]
pub(crate) mod test_native_tools;
pub mod types;
pub mod unions;
pub use inkwell::context::Context;
use inkwell::{
    IntPredicate,
    basic_block::BasicBlock,
    builder::{Builder, BuilderError},
    intrinsics::Intrinsic,
    module::Module,
    support::LLVMString,
    types::IntType,
    values::{
        BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, IntValue, PointerValue,
    },
};
use jai_ir::{
    Block, BoolExpr, Call, EntryPoint, Equality, Flow, IntExpr, IntExprKind, IntLocal, IntOp,
    LoopCondition, LoopId, Program, RangeLoop, Relation, Statement, ValueExpr,
};
use jai_ir::{
    BoolPlace, CleanupId, Conditional, Exit, GlobalInitializer, IntPlace, Place, PlaceKind,
    Transfer,
};
use jai_types::{CastMode, Direction, IntegerType};
use jai_types::{TypeId, TypeKind, Types};
use std::{collections::HashMap, fmt};
pub use storage_reinterpretation::StorageValueRole;
use types::integer_type;

#[derive(Debug)]
pub enum Error {
    ExternalData(external_data::Error),
    Simd(native_simd::Error),
    StorageAlignment(storage_alignment::Error),
    Debug(debug::Error),
    Reachability(native_reachability::Error),
    RuntimeIntrinsic(jai_ir::RuntimeIntrinsicError),
    Pointer(jai_llvm::Error),
    UnsupportedPointerCast(CastMode),
    NativePointerConstant(jai_ir::NativePointerConstantError),
    StaticByteView(jai_ir::StaticByteViewError),
    UnsupportedIntegerCast(CastMode),
    StorageBitcast(jai_types::StorageBitcastError),
    UnsupportedStorageBitcast {
        procedure: jai_ir::ProcedureId,
        ty: TypeId,
        role: StorageValueRole,
    },
    Foreign(abi::Error),
    UnsupportedPrototype(jai_ir::ProcedureId),
    UnsupportedInlining {
        procedure: jai_ir::ProcedureId,
        reason: &'static str,
    },
    Build(BuilderError),
    Type(types::Error),
    Verification(LLVMString),
    Invariant,
}
impl From<abi::Error> for Error {
    fn from(error: abi::Error) -> Self {
        Self::Foreign(error)
    }
}
impl From<jai_llvm::Error> for Error {
    fn from(error: jai_llvm::Error) -> Self {
        Self::Pointer(error)
    }
}
impl From<jai_ir::NativePointerConstantError> for Error {
    fn from(error: jai_ir::NativePointerConstantError) -> Self {
        Self::NativePointerConstant(error)
    }
}
impl From<jai_types::TypeError> for Error {
    fn from(error: jai_types::TypeError) -> Self {
        Self::Type(error.into())
    }
}
impl From<types::Error> for Error {
    fn from(error: types::Error) -> Self {
        Self::Type(error)
    }
}
impl From<BuilderError> for Error {
    fn from(e: BuilderError) -> Self {
        Self::Build(e)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExternalData(error) => error.fmt(f),
            Self::UnsupportedStorageBitcast {
                procedure,
                ty,
                role,
            } => write!(
                f,
                "native storage bitcast in procedure {} cannot preserve implicit padding in {role} {ty:?}",
                procedure.index()
            ),
            Self::StorageBitcast(error) => write!(f, "storage bitcast lowering: {error}"),
            Self::UnsupportedIntegerCast(mode) => {
                write!(f, "unsupported numeric integer cast mode: {mode:?}")
            }
            Self::Simd(error) => write!(f, "SIMD assembly lowering: {error}"),
            Self::StorageAlignment(error) => write!(f, "{error}"),
            Self::Reachability(error) => write!(f, "{error}"),
            Self::RuntimeIntrinsic(error) => write!(f, "{error}"),
            Self::Debug(error) => write!(f, "LLVM debug information: {error}"),
            Self::Foreign(error) => write!(f, "foreign ABI lowering failed: {error}"),
            Self::UnsupportedPrototype(id) => write!(
                f,
                "compiler runtime prototype {id:?} has no native implementation"
            ),
            Self::UnsupportedInlining {
                procedure,
                reason,
            } => {
                write!(f, "unsupported inlining policy for {procedure:?}: {reason}")
            }
            Self::Pointer(error) => {
                write!(f, "LLVM pointer instruction construction failed: {error}")
            }
            Self::UnsupportedPointerCast(mode) => {
                write!(
                    f,
                    "numeric pointer conversion does not support {mode:?} cast mode"
                )
            }
            Self::StaticByteView(error) => write!(f, "static byte view: {error}"),
            Self::NativePointerConstant(error) => write!(f, "native pointer constant: {error}"),
            Self::Type(e) => write!(f, "LLVM type lowering failed: {e}"),
            Self::Build(e) => write!(f, "LLVM instruction construction failed: {e}"),
            Self::Verification(e) => write!(f, "LLVM module verification failed: {e}"),
            Self::Invariant => f.write_str("internal LLVM lowering invariant failed"),
        }
    }
}
impl std::error::Error for Error {
}
/// LLVM performs instruction construction and text serialization.
pub fn emit(program: &Program) -> Result<String, Error> {
    let context = Context::create();
    Ok(lower(&context, program)?.print_to_string().to_string())
}
/// Build a verified module without serialization. The context owns its lifetime.
pub fn lower<'ctx>(context: &'ctx Context, program: &Program) -> Result<Module<'ctx>, Error> {
    let target = abi::NativeTarget::new()?;
    lower_for_target(context, program, &target)
}

/// Lower against an explicitly selected target, including its actual data layout.
pub fn lower_for_target<'ctx>(
    context: &'ctx Context,
    program: &Program,
    target: &target::NativeTarget,
) -> Result<Module<'ctx>, Error> {
    let reachable = native_reachability::Reachable::executable(program.library(), program.entry())
        .map_err(Error::Reachability)?;
    lower_unit(
        context,
        program.library(),
        Some(program.entry()),
        target,
        &reachable,
    )
}
/// Publish a program object with its entry bridge and typed unresolved declarations.
/// Executable builds must use `lower_for_target` for checked provider reachability.
pub fn lower_program_object_for_target<'ctx>(
    context: &'ctx Context,
    program: &Program,
    target: &target::NativeTarget,
) -> Result<Module<'ctx>, Error> {
    let reachable = native_reachability::Reachable::object(program.library(), program.entry())
        .map_err(Error::Reachability)?;
    lower_unit(
        context,
        program.library(),
        Some(program.entry()),
        target,
        &reachable,
    )
}
/// Emit a published library without an executable wrapper. All-body publication
/// includes every checked body; selected publication follows only explicit roots.
pub fn lower_library<'ctx>(
    context: &'ctx Context,
    library: &jai_ir::Library,
    publication: &native_reachability::Publication,
) -> Result<Module<'ctx>, Error> {
    let target = target::NativeTarget::new()?;
    lower_library_for_target(context, library, publication, &target)
}
pub fn lower_library_for_target<'ctx>(
    context: &'ctx Context,
    library: &jai_ir::Library,
    publication: &native_reachability::Publication,
    target: &target::NativeTarget,
) -> Result<Module<'ctx>, Error> {
    let reachable = native_reachability::Reachable::library(library, publication)
        .map_err(Error::Reachability)?;
    lower_unit(context, library, None, target, &reachable)
}
fn lower_unit<'ctx>(
    context: &'ctx Context,
    library: &jai_ir::Library,
    entry: Option<EntryPoint>,
    target: &target::NativeTarget,
    reachable: &native_reachability::Reachable,
) -> Result<Module<'ctx>, Error> {
    let module = context.create_module("jai");
    module.set_triple(&target.triple);
    module.set_data_layout(&target.data.get_data_layout());
    let debug = match (target.debug, library.debug_sources()) {
        (jai_types::DebugInformation::Off, _) | (_, None) => None,
        (information, Some(sources)) => debug::LineTables::new_for_procedures(
            &module,
            context,
            sources,
            target.optimization,
            information,
            types::layout_policy(context, &target.data)?,
            library
                .procedures()
                .iter()
                .filter(|procedure| reachable.contains(procedure.id))
                .map(|procedure| procedure.id),
        )
        .map_err(Error::Debug)?,
    };
    let bit = context.bool_type();
    let types = library.types();
    let native_symbols = program_exports::Symbols::new(library);
    let mut lowerer = types::TypeLowerer::with_target(context, types, &target.data);
    lowerer.set_context_pointer(library.context().map(|definition| definition.pointer_type))?;
    let mut functions = HashMap::new();
    let mut foreign_functions = HashMap::new();
    for procedure in library
        .procedures()
        .iter()
        .filter(|procedure| reachable.contains(procedure.id))
    {
        let name = native_symbols.procedure(procedure.id);
        let function = if types
            .procedure_definition(procedure.signature)?
            .convention
            .uses_c_abi()
        {
            let external = foreign::declare(
                &module,
                types,
                &mut lowerer,
                target.c_platform()?,
                &target.data,
                procedure.signature,
                &name,
            )?;
            let value = external.value;
            foreign_functions.insert(procedure.id, external);
            value
        } else {
            module.add_function(&name, lowerer.function(procedure.signature)?, None)
        };
        inline_hints::apply(context, function, library.inline_hint(procedure.id));
        functions.insert(procedure.id, function);
    }
    for prototype in library
        .prototypes()
        .iter()
        .filter(|prototype| reachable.contains(prototype.id))
    {
        if let jai_ir::PrototypeOrigin::Intrinsic(operation) = prototype.origin {
            let function = native_intrinsics::define(
                &module,
                &mut lowerer,
                target,
                prototype.signature,
                operation,
                &format!("jai.p{}", prototype.id.index()),
            )?;
            functions.insert(prototype.id, function);
            continue;
        }
        if let jai_ir::PrototypeOrigin::SourceContract {
            symbol,
        } = &prototype.origin
        {
            let function =
                module.add_function(symbol, lowerer.function(prototype.signature)?, None);
            functions.insert(prototype.id, function);
            continue;
        }
        let jai_ir::PrototypeOrigin::Foreign {
            symbol, ..
        } = &prototype.origin
        else {
            return Err(Error::UnsupportedPrototype(prototype.id));
        };
        let external = foreign::declare(
            &module,
            types,
            &mut lowerer,
            target.c_platform()?,
            &target.data,
            prototype.signature,
            symbol,
        )?;
        functions.insert(prototype.id, external.value);
        foreign_functions.insert(prototype.id, external);
    }
    let mut external_declarations = external_data::Declarations::default();
    let globals: Vec<_> = library
        .globals()
        .iter()
        .map(|global| {
            let ty = lowerer.basic(global.ty())?;
            let alignment = storage_alignment::allocation(
                &target.data,
                ty,
                library.storage_alignments().global(global.id()),
            )
            .map_err(Error::StorageAlignment)?;
            if let GlobalInitializer::External(data) = global.initializer() {
                let value = external_declarations
                    .declare(&module, global.id(), data, ty, alignment)
                    .map_err(Error::ExternalData)?;
                return Ok(Slot {
                    pointer: value.as_pointer_value(),
                    ty: global.ty(),
                    alignment,
                });
            }
            let initializer: BasicValueEnum<'ctx> = match global.initializer() {
                GlobalInitializer::External(_) => return Err(Error::Invariant),
                GlobalInitializer::Int(n) => ty.into_int_type().const_int(n.bits(), false).into(),
                GlobalInitializer::Bool(b) => {
                    ty.into_int_type().const_int(u64::from(*b), false).into()
                }
                GlobalInitializer::Value(value) => aggregates::constant(
                    &mut lowerer,
                    value,
                    context,
                    &module,
                    &functions,
                    library.signatures(),
                )?,
            };
            let symbol = native_symbols.global(global.id());
            external_declarations
                .check_owned_symbol(&module, global.id(), &symbol)
                .map_err(Error::ExternalData)?;
            let value = module.add_global(initializer.get_type(), None, &symbol);
            value.set_initializer(&initializer);
            value.set_alignment(alignment);
            Ok(Slot {
                pointer: value.as_pointer_value(),
                ty: global.ty(),
                alignment,
            })
        })
        .collect::<Result<_, Error>>()?;
    let trap = Intrinsic::find("llvm.trap")
        .ok_or(Error::Invariant)?
        .get_declaration(&module, &[])
        .ok_or(Error::Invariant)?;
    for p in library
        .procedures()
        .iter()
        .filter(|procedure| reachable.contains(procedure.id))
    {
        let function = *functions.get(&p.id).ok_or(Error::Invariant)?;
        let context_mode = types.procedure_definition(p.signature)?.context;
        let active_context = context::parameter(function, context_mode)?;
        let builder = context.create_builder();
        builder.position_at_end(context.append_basic_block(function, "entry"));
        let debug_scope = if let (Some(debug), Some(sources)) = (&debug, library.debug_sources()) {
            debug
                .attach_typed_scope(
                    context,
                    &builder,
                    function,
                    p.id,
                    sources,
                    (p.signature, types),
                )
                .map_err(Error::Debug)?
        } else {
            None
        };
        let mut slots = Vec::with_capacity(p.locals.len());
        for local in &p.locals {
            let ty = lowerer.basic(local.ty())?;
            let alignment = storage_alignment::allocation(
                &target.data,
                ty,
                library.storage_alignments().local(local.id()),
            )
            .map_err(Error::StorageAlignment)?;
            let pointer = builder.build_alloca(ty, "local")?;
            pointer
                .as_instruction_value()
                .ok_or(Error::Invariant)?
                .set_alignment(alignment)
                .map_err(|_| Error::Invariant)?;
            slots.push(Slot {
                pointer,
                ty: local.ty(),
                alignment,
            });
        }
        let parameter_values = if let Some(external) = foreign_functions.get(&p.id) {
            foreign::parameters(
                &builder,
                types,
                &mut lowerer,
                &target.data,
                &external.signature,
                function,
            )?
        } else {
            function
                .get_param_iter()
                .skip(usize::from(
                    context_mode == jai_types::ContextMode::Implicit,
                ))
                .collect()
        };
        for (parameter, local) in parameter_values.into_iter().zip(&p.parameters) {
            builder.build_store(slots[local.id().index()].pointer, parameter)?;
        }
        let mut g = Generator {
            library,
            module: &module,
            context,
            context_definition: library.context(),
            active_context,
            procedure_context: active_context,
            pushed_contexts: HashMap::new(),
            expression_bindings: HashMap::new(),
            phase_bindings: HashMap::new(),
            sequence_temp_bytes: None,
            sequence_temp_allocations: None,
            types,
            places: library.places(),
            lowerer: &mut lowerer,
            procedure: p.id,
            debug: match (&debug, library.debug_sources(), debug_scope) {
                (Some(tables), Some(sources), Some(scope)) => {
                    Some(debug::State::new(tables, sources, p.id, scope).map_err(Error::Debug)?)
                }
                _ => None,
            },
            builder,
            function,
            functions: &functions,
            foreign_functions: &foreign_functions,
            signatures: library.signatures(),
            target,
            abi_signature: foreign_functions
                .get(&p.id)
                .map(|external| &external.signature),
            slots,
            globals: &globals,
            trap,
            bit,
            loops: Vec::new(),
            cleanups: &p.cleanups,
        };
        if let Some(debug) = &mut g.debug {
            debug
                .parameters(context, &g.builder, types, &g.slots)
                .map_err(Error::Debug)?;
        }
        g.block(&p.body)?;
        if p.body.flow == Flow::FallsThrough && !g.current_block_terminated()? {
            g.builder.unset_current_debug_location();
            g.return_value(None)?;
        }
    }
    if let Some(entry) = native_symbols.synthetic_entry(entry) {
        let main = module.add_function("main", context.i32_type().fn_type(&[], false), None);
        let builder = context.create_builder();
        builder.position_at_end(context.append_basic_block(main, "entry"));
        let entry_id = match entry {
            EntryPoint::Void(id) | EntryPoint::Int(id) => id,
        };
        let entry_signature = *library
            .signatures()
            .get(&entry_id)
            .ok_or(Error::Invariant)?;
        let entry_arguments: Vec<BasicMetadataValueEnum<'ctx>> =
            if types.procedure_definition(entry_signature)?.context
                == jai_types::ContextMode::Implicit
            {
                vec![
                    context::entry_context(
                        context,
                        &module,
                        &builder,
                        &mut lowerer,
                        library.context().ok_or(Error::Invariant)?,
                        &functions,
                        library.signatures(),
                    )?
                    .into(),
                ]
            } else {
                vec![]
            };
        let status = match entry {
            EntryPoint::Void(id) => {
                builder.build_call(
                    *functions.get(&id).ok_or(Error::Invariant)?,
                    &entry_arguments,
                    "",
                )?;
                context.i32_type().const_zero()
            }
            EntryPoint::Int(id) => {
                let call = builder.build_call(
                    *functions.get(&id).ok_or(Error::Invariant)?,
                    &entry_arguments,
                    "result",
                )?;
                builder.build_int_truncate(
                    int_value(call.try_as_basic_value().basic().ok_or(Error::Invariant)?)?,
                    context.i32_type(),
                    "status",
                )?
            }
        };
        builder.build_return(Some(&status))?;
    }
    // Consume the complete Option so its module borrow ends before returning it.
    let _ = debug.map(debug::LineTables::finish);
    module.verify().map_err(Error::Verification)?;
    Ok(module)
}
#[derive(Clone, Copy)]
struct LoopBlocks<'ctx> {
    id: LoopId,
    next: BasicBlock<'ctx>,
    end: BasicBlock<'ctx>,
}
// LLVM uses IntValue for integers and Booleans; keep their language categories distinct.
#[derive(Clone, Copy)]
struct Number<'ctx>(IntValue<'ctx>);
#[derive(Clone, Copy)]
struct Bit<'ctx>(IntValue<'ctx>);
#[derive(Clone, Copy)]
struct IntSlot<'ctx>(PointerValue<'ctx>, u32);
#[derive(Clone, Copy)]
struct BoolSlot<'ctx>(PointerValue<'ctx>, u32);
#[derive(Clone, Copy)]
struct Slot<'ctx> {
    pointer: PointerValue<'ctx>,
    ty: TypeId,
    alignment: u32,
}
enum Destination<'ctx> {
    ReturnVoid,
    ReturnValue(BasicValueEnum<'ctx>),
    Branch(BasicBlock<'ctx>),
}
enum Logical {
    And,
    Or,
}
fn predicate(op: Relation, ty: IntegerType) -> IntPredicate {
    match (op, ty.signed()) {
        (Relation::Equal, _) => IntPredicate::EQ,
        (Relation::NotEqual, _) => IntPredicate::NE,
        (Relation::Less, true) => IntPredicate::SLT,
        (Relation::Less, false) => IntPredicate::ULT,
        (Relation::LessEqual, true) => IntPredicate::SLE,
        (Relation::LessEqual, false) => IntPredicate::ULE,
        (Relation::Greater, true) => IntPredicate::SGT,
        (Relation::Greater, false) => IntPredicate::UGT,
        (Relation::GreaterEqual, true) => IntPredicate::SGE,
        (Relation::GreaterEqual, false) => IntPredicate::UGE,
    }
}
fn int_value(value: BasicValueEnum<'_>) -> Result<IntValue<'_>, Error> {
    match value {
        BasicValueEnum::IntValue(value) => Ok(value),
        _ => Err(Error::Invariant),
    }
}
fn call_int(call: foreign::CallResult<'_>) -> Result<IntValue<'_>, Error> {
    int_value(call.value.ok_or(Error::Invariant)?)
}
struct Generator<'ctx, 'program, 'functions> {
    library: &'program jai_ir::Library,
    module: &'functions Module<'ctx>,
    context: &'ctx Context,
    context_definition: Option<&'program jai_ir::ContextDefinition>,
    active_context: Option<PointerValue<'ctx>>,
    procedure_context: Option<PointerValue<'ctx>>,
    pushed_contexts: HashMap<jai_ir::PushContextId, PointerValue<'ctx>>,
    expression_bindings: HashMap<jai_ir::ExpressionBindingId, BasicValueEnum<'ctx>>,
    phase_bindings: execution_phase::PhaseBindings,
    sequence_temp_bytes: Option<PointerValue<'ctx>>,
    sequence_temp_allocations: Option<PointerValue<'ctx>>,
    types: &'program Types,
    places: &'program jai_ir::Places,
    lowerer: &'functions mut types::TypeLowerer<'ctx, 'program>,
    procedure: jai_ir::ProcedureId,
    debug: Option<debug::State<'ctx, 'program, 'functions>>,
    builder: Builder<'ctx>,
    function: FunctionValue<'ctx>,
    functions: &'functions HashMap<jai_ir::ProcedureId, FunctionValue<'ctx>>,
    foreign_functions: &'functions HashMap<jai_ir::ProcedureId, foreign::Function<'ctx>>,
    signatures: &'program HashMap<jai_ir::ProcedureId, TypeId>,
    target: &'program abi::NativeTarget,
    abi_signature: Option<&'functions abi::Signature<'ctx>>,
    slots: Vec<Slot<'ctx>>,
    trap: FunctionValue<'ctx>,
    bit: IntType<'ctx>,
    loops: Vec<LoopBlocks<'ctx>>,
    cleanups: &'functions [jai_ir::Cleanup],
    globals: &'functions [Slot<'ctx>],
}
impl<'ctx> Generator<'ctx, '_, '_> {
    fn label(&self, name: &str) -> BasicBlock<'ctx> {
        self.context.append_basic_block(self.function, name)
    }
    fn slot(&mut self, place: Place) -> Result<Slot<'ctx>, Error> {
        let mut root = place;
        let mut fields = Vec::new();
        while let PlaceKind::Field(id) = root.kind() {
            let projection = self.places.projection(id).map_err(|_| Error::Invariant)?;
            fields.push((projection.field, root.ty()));
            root = projection.base;
        }
        let mut slot = match root.kind() {
            PlaceKind::Context(ty) => self.context_slot(ty)?,
            PlaceKind::Local(id) if id.procedure() == self.procedure => self
                .slots
                .get(id.index())
                .copied()
                .ok_or(Error::Invariant)?,
            PlaceKind::Local(_) => return Err(Error::Invariant),
            PlaceKind::Global(id) => self
                .globals
                .get(id.index())
                .copied()
                .ok_or(Error::Invariant)?,
            PlaceKind::Dereference(id) => {
                let projection = self.places.dereference(id).map_err(|_| Error::Invariant)?;
                self.dereference_slot(&projection.pointer, root.ty())?
            }
            PlaceKind::Index(id) => {
                let projection = self.places.index(id).map_err(|_| Error::Invariant)?;
                self.index_slot(
                    projection.base,
                    &projection.index,
                    root.ty(),
                    projection.check,
                )?
            }
            PlaceKind::SequenceField(id) => {
                let projection = self
                    .places
                    .sequence_field(id)
                    .map_err(|_| Error::Invariant)?;
                self.sequence_projection(projection.base, projection.field, root.ty())?
            }
            PlaceKind::Field(_) => return Err(Error::Invariant),
        };
        if slot.ty != root.ty() {
            return Err(Error::Invariant);
        }
        for (field, projected_ty) in fields.into_iter().rev() {
            if self.types.validate_field(slot.ty, field)? != projected_ty {
                return Err(Error::Invariant);
            }
            let layout = self.lowerer.semantic_layout(slot.ty)?;
            let offset = *layout
                .field_offsets
                .get(field.index())
                .ok_or(Error::Invariant)?;
            let pointer =
                records::field_pointer(self.context, &self.builder, slot.pointer, offset)?;
            slot = Slot {
                pointer,
                ty: projected_ty,
                alignment: memory::offset_alignment(slot.alignment, offset),
            };
        }
        Ok(slot)
    }
    fn load(&mut self, place: Place) -> Result<BasicValueEnum<'ctx>, Error> {
        let slot = self.slot(place)?;
        let ty = self.lowerer.basic(slot.ty)?;
        memory::load(&self.builder, ty, slot.pointer, "load", slot.alignment)
    }
    fn int_slot(&mut self, id: IntLocal) -> Result<IntSlot<'ctx>, Error> {
        self.int_place(id.place())
    }
    fn int_place(&mut self, place: IntPlace) -> Result<IntSlot<'ctx>, Error> {
        let slot = self.slot(place.place())?;
        match self.types.kind(slot.ty).map_err(|_| Error::Invariant)? {
            TypeKind::Integer(ty) if *ty == place.ty() => Ok(IntSlot(slot.pointer, slot.alignment)),
            _ => Err(Error::Invariant),
        }
    }
    fn bool_slot(&mut self, id: jai_ir::BoolLocal) -> Result<BoolSlot<'ctx>, Error> {
        self.bool_place(id.place())
    }
    fn bool_place(&mut self, place: BoolPlace) -> Result<BoolSlot<'ctx>, Error> {
        let slot = self.slot(place.place())?;
        match self.types.kind(slot.ty).map_err(|_| Error::Invariant)? {
            TypeKind::Bool => Ok(BoolSlot(slot.pointer, slot.alignment)),
            _ => Err(Error::Invariant),
        }
    }
    fn block(&mut self, block: &Block) -> Result<(), Error> {
        for (index, statement) in block.statements.iter().enumerate() {
            if self.current_block_terminated()? {
                break;
            }
            if let Some(debug) = &mut self.debug {
                debug
                    .before_statement(self.context, &self.builder, self.types, &self.slots, index)
                    .map_err(Error::Debug)?;
            }
            match statement {
                Statement::Simd(block) => self.simd(block)?,
                Statement::PushContext {
                    id,
                    value,
                    body,
                } => self.push_context(*id, value, body)?,
                Statement::StoreInt(id, e) => {
                    let v = self.int(e)?;
                    let slot = self.int_place(*id)?;
                    memory::store(&self.builder, slot.0, v.0.into(), slot.1)?;
                }
                Statement::StoreBool(id, e) => {
                    let v = self.boolean(e)?;
                    let slot = self.bool_place(*id)?;
                    memory::store(&self.builder, slot.0, v.0.into(), slot.1)?;
                }
                Statement::Store(place, value) => self.store(*place, value)?,
                Statement::DiscardValue(value) => {
                    self.value(value)?;
                }
                Statement::IndirectCallResults {
                    inline_hint,
                    callee,
                    arguments,
                    destinations,
                } => {
                    let destinations = self.capture_call_destinations(destinations)?;
                    let result = self.indirect_call(callee, arguments, *inline_hint)?.value;
                    self.store_call_results(result, &destinations)?;
                }
                Statement::CallResults {
                    call,
                    destinations,
                } => self.call_results(call, destinations)?,
                Statement::Exit(exit) => self.exit(exit)?,
                Statement::Cleanup(id) => self.cleanup(*id)?,
                Statement::DiscardInt(e) => {
                    self.int(e)?;
                }
                Statement::DiscardBool(e) => {
                    self.boolean(e)?;
                }
                Statement::CallVoid(call) => {
                    self.call(call)?;
                }
                Statement::Cases(case) => self.cases(case)?,
                Statement::Block(block) => {
                    self.debug_child_block(jai_ir::DebugBranch::Block, block)?
                }
                Statement::If(condition, yes, no) => {
                    if let Some(selected) = self.native_condition(condition) {
                        self.boolean(condition)?;
                        let (branch, block) = if selected {
                            (jai_ir::DebugBranch::IfThen, yes)
                        } else {
                            (jai_ir::DebugBranch::IfElse, no)
                        };
                        self.debug_child_block(branch, block)?;
                        continue;
                    }
                    let c = self.boolean(condition)?;
                    let y = self.label("if.yes");
                    let n = self.label("if.no");
                    let mut join = None;
                    self.builder.build_conditional_branch(c.0, y, n)?;
                    for (label, block, branch) in [
                        (y, yes, jai_ir::DebugBranch::IfThen),
                        (n, no, jai_ir::DebugBranch::IfElse),
                    ] {
                        self.builder.position_at_end(label);
                        self.debug_child_block(branch, block)?;
                        if !self.current_block_terminated()? {
                            let join = *join.get_or_insert_with(|| self.label("if.end"));
                            self.builder.build_unconditional_branch(join)?;
                        }
                    }
                    if let Some(join) = join {
                        self.builder.position_at_end(join);
                    }
                }
                Statement::Range(range) => self.range(range)?,
                Statement::While {
                    id,
                    condition,
                    body,
                } => {
                    let selected =
                        execution_phase::native_loop_condition(condition, &self.phase_bindings);
                    if selected == Some(false) {
                        self.loop_condition(condition)?;
                        continue;
                    }
                    let terminates = execution_phase::native_while_terminates(
                        *id,
                        condition,
                        body,
                        &self.phase_bindings,
                    );
                    let test = self.label("while.test");
                    let inside = self.label("while.body");
                    let end = self.label("while.end");
                    self.builder.build_unconditional_branch(test)?;
                    self.builder.position_at_end(test);
                    let c = self.loop_condition(condition)?;
                    if selected == Some(true) {
                        self.builder.build_unconditional_branch(inside)?;
                    } else {
                        self.builder.build_conditional_branch(c.0, inside, end)?;
                    }
                    self.builder.position_at_end(inside);
                    self.loops.push(LoopBlocks {
                        id: *id,
                        next: test,
                        end,
                    });
                    self.debug_child_block(jai_ir::DebugBranch::While, body)?;
                    self.loops.pop();
                    if !self.current_block_terminated()? {
                        self.builder.build_unconditional_branch(test)?;
                    }
                    self.builder.position_at_end(end);
                    if terminates {
                        self.builder.build_unreachable()?;
                    }
                }
            }
        }
        Ok(())
    }
    fn current_block_terminated(&self) -> Result<bool, Error> {
        Ok(self
            .builder
            .get_insert_block()
            .ok_or(Error::Invariant)?
            .get_terminator()
            .is_some())
    }
    fn cleanup(&mut self, id: CleanupId) -> Result<(), Error> {
        self.context_cleanup(id)
    }
    fn exit(&mut self, exit: &Exit) -> Result<(), Error> {
        // Snapshot the return value before cleanup can mutate its source locals.
        let destination = match &exit.transfer {
            Transfer::ReturnVoid => Destination::ReturnVoid,
            Transfer::ReturnInt(e) => Destination::ReturnValue(self.int(e)?.0.into()),
            Transfer::ReturnBool(e) => Destination::ReturnValue(self.boolean(e)?.0.into()),
            Transfer::ReturnValues(values) => match self.return_values(values)? {
                Some(value) => Destination::ReturnValue(value),
                None => Destination::ReturnVoid,
            },
            Transfer::Break(id) | Transfer::Continue(id) => {
                let target = self
                    .loops
                    .iter()
                    .rev()
                    .find(|l| l.id == *id)
                    .ok_or(Error::Invariant)?;
                Destination::Branch(match exit.transfer {
                    Transfer::Break(_) => target.end,
                    Transfer::Continue(_) => target.next,
                    _ => return Err(Error::Invariant),
                })
            }
        };
        for id in &exit.cleanups {
            self.cleanup(*id)?;
        }
        match destination {
            Destination::ReturnVoid => {
                self.return_value(None)?;
            }
            Destination::ReturnValue(value) => {
                self.return_value(Some(value))?;
            }
            Destination::Branch(block) => {
                self.builder.build_unconditional_branch(block)?;
            }
        }
        Ok(())
    }
    fn return_value(&mut self, value: Option<BasicValueEnum<'ctx>>) -> Result<(), Error> {
        self.guard_sequence_pack_return(value)?;
        if let Some(signature) = self.abi_signature {
            foreign::return_value(
                &self.builder,
                self.context,
                &self.target.data,
                signature,
                self.function,
                value,
            )?;
        } else {
            match value {
                Some(value) => {
                    self.builder.build_return(Some(&value))?;
                }
                None => {
                    self.builder.build_return(None)?;
                }
            }
        }
        Ok(())
    }
    fn loop_condition(&mut self, condition: &LoopCondition) -> Result<Bit<'ctx>, Error> {
        match condition {
            LoopCondition::Value(e) => self.boolean(e),
            LoopCondition::BoundInt(id, e) => {
                let value = self.int(e)?;
                let slot = self.int_slot(*id)?;
                memory::store(&self.builder, slot.0, value.0.into(), slot.1)?;
                Ok(Bit(self.builder.build_int_compare(
                    IntPredicate::NE,
                    value.0,
                    value.0.get_type().const_zero(),
                    "while.truth",
                )?))
            }
            LoopCondition::BoundBool(id, e) => {
                let value = self.boolean(e)?;
                let slot = self.bool_slot(*id)?;
                memory::store(&self.builder, slot.0, value.0.into(), slot.1)?;
                Ok(value)
            }
        }
    }
    fn range(&mut self, range: &RangeLoop) -> Result<(), Error> {
        let start = self.int(&range.start)?.0;
        let end = self.int(&range.end)?.0;
        let slot = self.int_slot(range.iterator)?.0;
        let (first, last, done_predicate) = match range.direction {
            Direction::Forward => (
                start,
                end,
                predicate(Relation::GreaterEqual, range.start.ty()),
            ),
            Direction::Reverse => (end, start, predicate(Relation::LessEqual, range.start.ty())),
        };
        let inside = self.label("range.body");
        let step = self.label("range.step");
        let advance = self.label("range.advance");
        let after = self.label("range.end");
        self.builder.build_store(slot, first)?;
        let nonempty = self.builder.build_int_compare(
            predicate(Relation::LessEqual, range.start.ty()),
            start,
            end,
            "range.nonempty",
        )?;
        self.builder
            .build_conditional_branch(nonempty, inside, after)?;
        self.builder.position_at_end(inside);
        self.loops.push(LoopBlocks {
            id: range.id,
            next: step,
            end: after,
        });
        self.debug_child_block(jai_ir::DebugBranch::Range, &range.body)?;
        self.loops.pop();
        if !self.current_block_terminated()? {
            self.builder.build_unconditional_branch(step)?;
        }
        self.builder.position_at_end(step);
        let current = int_value(self.builder.build_load(
            integer_type(self.context, range.iterator.ty()),
            slot,
            "range.current",
        )?)?;
        // Test before advancing: inclusive MAX/MIN endpoints cannot wrap the iterator.
        let done = self
            .builder
            .build_int_compare(done_predicate, current, last, "range.done")?;
        self.builder
            .build_conditional_branch(done, after, advance)?;
        self.builder.position_at_end(advance);
        let one = current.get_type().const_int(1, false);
        let next = match range.direction {
            Direction::Forward => self.builder.build_int_add(current, one, "range.next")?,
            Direction::Reverse => self.builder.build_int_sub(current, one, "range.next")?,
        };
        self.builder.build_store(slot, next)?;
        self.builder.build_unconditional_branch(inside)?;
        self.builder.position_at_end(after);
        Ok(())
    }
    fn arguments(
        &mut self,
        arguments: &[(jai_ir::ParameterId, ValueExpr)],
    ) -> Result<Vec<(TypeId, BasicValueEnum<'ctx>)>, Error> {
        let mut evaluated = Vec::with_capacity(arguments.len());
        for (parameter, argument) in arguments {
            // Preserve source evaluation order before ordering already evaluated values by formal parameter.
            evaluated.push((
                *parameter,
                argument.type_id(self.types),
                self.value(argument)?,
            ));
        }
        evaluated.sort_by_key(|(parameter, _, _)| parameter.index());
        Ok(evaluated
            .into_iter()
            .map(|(_, ty, value)| (ty, value))
            .collect())
    }
    fn call(&mut self, call: &Call) -> Result<foreign::CallResult<'ctx>, Error> {
        inline_hints::validate(self.library, call.procedure, call.inline_hint())?;
        let arguments = self.arguments(&call.arguments)?;
        if let Some(external) = self.foreign_functions.get(&call.procedure) {
            let result = external.call(
                &self.builder,
                self.types,
                self.lowerer,
                &self.target.data,
                &arguments,
            )?;
            inline_hints::apply_call(self.context, result.site, call.inline_hint());
            return Ok(result);
        }
        let function = *self
            .functions
            .get(&call.procedure)
            .ok_or(Error::Invariant)?;
        let signature = *self
            .signatures
            .get(&call.procedure)
            .ok_or(Error::Invariant)?;
        let result =
            self.internal_call(signature, foreign::Callee::Direct(function), &arguments)?;
        inline_hints::apply_call(self.context, result.site, call.inline_hint());
        Ok(result)
    }
    fn indirect_call(
        &mut self,
        callee: &ValueExpr,
        arguments: &[(jai_ir::ParameterId, ValueExpr)],
        hint: jai_types::InlineHint,
    ) -> Result<foreign::CallResult<'ctx>, Error> {
        let signature = callee.type_id(self.types);
        if let ValueExpr::ProcedureValue {
            procedure, ..
        } = callee
        {
            inline_hints::validate(self.library, *procedure, hint)?;
        }
        let definition = self.types.procedure_definition(signature)?;
        // The checked source convention chooses the ABI. A target with no
        // foreign classifier can still lower its internal Jai callbacks.
        let foreign_signature = if definition.convention.uses_c_abi() {
            let platform = self.target.c_platform()?;
            crate::cpp_methods::validate_triple(
                self.types,
                definition,
                platform,
                &self.target.triple.as_str().to_string_lossy(),
            )?;
            Some(abi::Signature::classify(
                self.context,
                self.types,
                self.lowerer,
                platform,
                &self.target.data,
                signature,
                matches!(definition.variadic, jai_types::Variadic::C { .. }),
            )?)
        } else {
            None
        };
        let target = if let ValueExpr::ProcedureValue {
            procedure, ..
        } = callee
        {
            foreign::Callee::Direct(*self.functions.get(procedure).ok_or(Error::Invariant)?)
        } else {
            foreign::Callee::Indirect(self.value(callee)?.into_pointer_value())
        };
        let arguments = self.arguments(arguments)?;
        let result = if let Some(signature) = foreign_signature {
            foreign::call(
                &self.builder,
                self.types,
                self.lowerer,
                &self.target.data,
                &signature,
                target,
                &arguments,
            )?
        } else {
            self.internal_call(signature, target, &arguments)?
        };
        inline_hints::apply_call(self.context, result.site, hint);
        Ok(result)
    }
    fn internal_call(
        &mut self,
        signature: TypeId,
        callee: foreign::Callee<'ctx>,
        arguments: &[(TypeId, BasicValueEnum<'ctx>)],
    ) -> Result<foreign::CallResult<'ctx>, Error> {
        let definition = self.types.procedure_definition(signature)?;
        if arguments.len() != definition.parameters.len()
            || arguments
                .iter()
                .zip(&definition.parameters)
                .any(|((actual, _), expected)| actual != expected)
        {
            return Err(Error::Invariant);
        }
        let function_type = self.lowerer.function(signature)?;
        let args = self.context_arguments(definition.context, arguments)?;
        let name = if function_type.get_return_type().is_some() {
            "call"
        } else {
            ""
        };
        let previous_location = self.builder.get_current_debug_location();
        if previous_location.is_none()
            && let Some(debug) = &self.debug
        {
            debug
                .artificial_call_location(&self.builder)
                .map_err(Error::Debug)?;
        }
        let site = match callee {
            foreign::Callee::Direct(function) => self.builder.build_call(function, &args, name),
            foreign::Callee::Indirect(pointer) => {
                self.builder
                    .build_indirect_call(function_type, pointer, &args, name)
            }
        };
        self.debug_restore_location(previous_location);
        let site = site?;
        Ok(foreign::CallResult {
            site,
            value: site.try_as_basic_value().basic(),
        })
    }
    fn int(&mut self, e: &IntExpr) -> Result<Number<'ctx>, Error> {
        let ty = integer_type(self.context, e.ty());
        Ok(Number(match e.kind() {
            IntExprKind::FromPointer {
                value,
                mode,
            } => self.pointer_to_integer(value, e.ty(), *mode)?,
            IntExprKind::PointerDifference {
                left,
                right,
            } => self.pointer_difference(left, right)?,
            IntExprKind::Constant(n) => ty.const_int(n.bits(), false),
            IntExprKind::InvalidCheckedCast => {
                self.check_cast(Bit(self.bit.const_zero()))?;
                ty.const_zero()
            }
            IntExprKind::Value(value) => {
                if !matches!(self.types.kind(value.type_id(self.types))?, TypeKind::Integer(integer) if *integer == e.ty())
                {
                    return Err(Error::Invariant);
                }
                int_value(self.value(value)?)?
            }
            IntExprKind::EnumValue(value) => {
                if self
                    .types
                    .enum_definition(value.type_id(self.types))?
                    .representation
                    != e.ty()
                {
                    return Err(Error::Invariant);
                }
                int_value(self.value(value)?)?
            }
            IntExprKind::FromFloat(mode, source) => self.float_to_integer(source, e.ty(), *mode)?.0,
            IntExprKind::FromBool(e) => {
                let v = self.boolean(e)?;
                self.builder.build_int_z_extend(v.0, ty, "cast.int")?
            }
            IntExprKind::Load(id) => int_value(self.load(id.place())?)?,
            IntExprKind::Call(call) => call_int(self.call(call)?)?,
            IntExprKind::Cast(mode, source) => self.integer_cast(source, e.ty(), *mode)?.0,
            IntExprKind::Conditional(e) => {
                if let Some(selected) = self.native_condition(&e.condition) {
                    self.boolean(&e.condition)?;
                    return self.int(if selected {
                        &e.then_value
                    } else {
                        &e.else_value
                    });
                }
                let [(yes, yes_end), (no, no_end)] = self.conditional_values(e, Self::int)?;
                let phi = self.builder.build_phi(ty, "ifx.int")?;
                phi.add_incoming(&[(&yes.0, yes_end), (&no.0, no_end)]);
                int_value(phi.as_basic_value())?
            }
            IntExprKind::Negate(source) => {
                let value = self.int(source)?.0;
                self.negate_integer(value, e.ty(), e.overflow_check())?
            }
            IntExprKind::Complement(e) => {
                let v = self.int(e)?;
                self.builder.build_not(v.0, "complement")?
            }
            IntExprKind::Binary(op, lhs, rhs) => {
                let lhs = self.int(lhs)?.0;
                let rhs = self.int(rhs)?.0;
                self.integer_operation(*op, lhs, rhs, e.ty(), e.overflow_check())?
            }
        }))
    }
    fn boolean(&mut self, e: &BoolExpr) -> Result<Bit<'ctx>, Error> {
        if let Some(value) = self.native_boolean(e) {
            return Ok(Bit(self.bit.const_int(u64::from(value), false)));
        }
        Ok(Bit(match e {
            BoolExpr::CompileTime => self.bit.const_zero(),
            BoolExpr::FromPointer(value) => self.pointer_truth(value)?,
            BoolExpr::ComparePointers(operation, left, right) => {
                self.compare_pointers(*operation, left, right)?
            }
            BoolExpr::Value(value) => {
                if !matches!(self.types.kind(value.type_id(self.types))?, TypeKind::Bool) {
                    return Err(Error::Invariant);
                }
                int_value(self.value(value)?)?
            }
            BoolExpr::Constant(b) => self.bit.const_int(u64::from(*b), false),
            BoolExpr::FromInt(e) => {
                let v = self.int(e)?;
                self.builder.build_int_compare(
                    IntPredicate::NE,
                    v.0,
                    v.0.get_type().const_zero(),
                    "truthiness",
                )?
            }
            BoolExpr::Load(id) => int_value(self.load(id.place())?)?,
            BoolExpr::Call(call) => call_int(self.call(call)?)?,
            BoolExpr::Conditional(e) => {
                if let Some(selected) = self.native_condition(&e.condition) {
                    self.boolean(&e.condition)?;
                    return self.boolean(if selected {
                        &e.then_value
                    } else {
                        &e.else_value
                    });
                }
                let [(yes, yes_end), (no, no_end)] = self.conditional_values(e, Self::boolean)?;
                let phi = self.builder.build_phi(self.bit, "ifx.bool")?;
                phi.add_incoming(&[(&yes.0, yes_end), (&no.0, no_end)]);
                int_value(phi.as_basic_value())?
            }
            BoolExpr::Not(e) => {
                let v = self.boolean(e)?;
                self.builder.build_not(v.0, "not")?
            }
            BoolExpr::CompareStrings(op, lhs, rhs) => self.compare_strings(*op, lhs, rhs)?,
            BoolExpr::CompareFloats(op, lhs, rhs) => self.compare_floats(*op, lhs, rhs)?,
            BoolExpr::CompareInts(op, lhs, rhs) => {
                let pred = predicate(*op, lhs.ty());
                let lhs = self.int(lhs)?.0;
                let rhs = self.int(rhs)?.0;
                self.builder
                    .build_int_compare(pred, lhs, rhs, "compare.int")?
            }
            BoolExpr::CompareBools(op, lhs, rhs) => {
                let lhs = self.boolean(lhs)?.0;
                let rhs = self.boolean(rhs)?.0;
                let pred = match op {
                    Equality::Equal => IntPredicate::EQ,
                    Equality::NotEqual => IntPredicate::NE,
                };
                self.builder
                    .build_int_compare(pred, lhs, rhs, "compare.bool")?
            }
            BoolExpr::And(lhs, rhs) => self.short_circuit(lhs, rhs, Logical::And)?.0,
            BoolExpr::Or(lhs, rhs) => self.short_circuit(lhs, rhs, Logical::Or)?.0,
        }))
    }
    fn integer_cast(
        &mut self,
        source: &IntExpr,
        target: IntegerType,
        mode: CastMode,
    ) -> Result<Number<'ctx>, Error> {
        if matches!(mode, CastMode::Force(_)) {
            return Err(Error::UnsupportedIntegerCast(mode));
        }
        let value = self.int(source)?.0;
        let from = source.ty();
        if mode == CastMode::Checked && !target.contains(from) {
            let mut valid = self.bit.const_int(1, false);
            if from.min() < target.min() {
                let minimum = value.get_type().const_int(target.min() as u64, false);
                let lower = self.builder.build_int_compare(
                    predicate(Relation::GreaterEqual, from),
                    value,
                    minimum,
                    "cast.lower",
                )?;
                valid = self.builder.build_and(valid, lower, "cast.valid")?;
            }
            if from.max() > target.max() {
                let maximum = value.get_type().const_int(target.max() as u64, false);
                let upper = self.builder.build_int_compare(
                    predicate(Relation::LessEqual, from),
                    value,
                    maximum,
                    "cast.upper",
                )?;
                valid = self.builder.build_and(valid, upper, "cast.valid")?;
            }
            self.check_cast(Bit(valid))?;
        }
        let ty = integer_type(self.context, target);
        Ok(Number(if from.bits() == target.bits() {
            value
        } else if from.bits() > target.bits() {
            self.builder.build_int_truncate(value, ty, "cast.narrow")?
        } else if from.signed() {
            self.builder.build_int_s_extend(value, ty, "cast.signed")?
        } else {
            self.builder
                .build_int_z_extend(value, ty, "cast.unsigned")?
        }))
    }
    fn check_cast(&mut self, valid: Bit<'ctx>) -> Result<(), Error> {
        let pass = self.label("cast.pass");
        let fail = self.label("cast.fail");
        self.builder.build_conditional_branch(valid.0, pass, fail)?;
        self.builder.position_at_end(fail);
        self.builder.build_call(self.trap, &[], "")?;
        self.builder.build_unreachable()?;
        self.builder.position_at_end(pass);
        Ok(())
    }
    fn conditional_values<T, V>(
        &mut self,
        expression: &Conditional<T>,
        emit: fn(&mut Self, &T) -> Result<V, Error>,
    ) -> Result<[(V, BasicBlock<'ctx>); 2], Error> {
        let condition = self.boolean(&expression.condition)?;
        let yes = self.label("ifx.then");
        let no = self.label("ifx.else");
        let join = self.label("ifx.end");
        self.builder
            .build_conditional_branch(condition.0, yes, no)?;
        self.builder.position_at_end(yes);
        let then_value = emit(self, &expression.then_value)?;
        let then_end = self.builder.get_insert_block().ok_or(Error::Invariant)?;
        self.builder.build_unconditional_branch(join)?;
        self.builder.position_at_end(no);
        let else_value = emit(self, &expression.else_value)?;
        let else_end = self.builder.get_insert_block().ok_or(Error::Invariant)?;
        self.builder.build_unconditional_branch(join)?;
        self.builder.position_at_end(join);
        Ok([(then_value, then_end), (else_value, else_end)])
    }
    fn short_circuit(
        &mut self,
        lhs: &BoolExpr,
        rhs: &BoolExpr,
        op: Logical,
    ) -> Result<Bit<'ctx>, Error> {
        if let Some(left) = self.native_condition(lhs) {
            self.boolean(lhs)?;
            if matches!((op, left), (Logical::And, false) | (Logical::Or, true)) {
                return Ok(Bit(self.bit.const_int(u64::from(left), false)));
            }
            return self.boolean(rhs);
        }
        let lhs = self.boolean(lhs)?;
        let left_end = self.builder.get_insert_block().ok_or(Error::Invariant)?;
        let right = self.label("logical.rhs");
        let join = self.label("logical.end");
        let (yes, no, bypass) = match op {
            Logical::And => (right, join, false),
            Logical::Or => (join, right, true),
        };
        self.builder.build_conditional_branch(lhs.0, yes, no)?;
        self.builder.position_at_end(right);
        let rhs = self.boolean(rhs)?;
        let right_end = self.builder.get_insert_block().ok_or(Error::Invariant)?;
        self.builder.build_unconditional_branch(join)?;
        self.builder.position_at_end(join);
        let phi = self.builder.build_phi(self.bit, "logical.value")?;
        let bypass = self.bit.const_int(u64::from(bypass), false);
        phi.add_incoming(&[(&bypass, left_end), (&rhs.0, right_end)]);
        Ok(Bit(int_value(phi.as_basic_value())?))
    }
}
