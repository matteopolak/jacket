//! The implicit Jai context is a shared typed pointer, scoped by `push_context`.
use super::*;
use jai_ir::{CleanupContext, ContextDefinition, ProcedureId, PushContextId};
use jai_types::ContextMode;

/// Entry wrappers own the initial context. Callees borrow its address so field
/// writes are visible in their callers without a host-global fallback.
pub(super) fn entry_context<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    lowerer: &mut types::TypeLowerer<'ctx, '_>,
    definition: &ContextDefinition,
    functions: &HashMap<ProcedureId, FunctionValue<'ctx>>,
    signatures: &HashMap<ProcedureId, TypeId>,
) -> Result<PointerValue<'ctx>, Error> {
    let pointer =
        builder.build_alloca(lowerer.basic(definition.record_type)?, "context.default")?;
    let value = aggregates::constant(
        lowerer,
        &definition.default,
        context,
        module,
        functions,
        signatures,
    )?;
    let record = lowerer.basic(definition.record_type)?;
    let alignment = lowerer
        .target_data()
        .ok_or(Error::Invariant)?
        .get_abi_alignment(&record);
    memory::store(builder, pointer, value, alignment)?;
    Ok(pointer)
}

pub(super) fn parameter<'ctx>(
    function: FunctionValue<'ctx>,
    mode: ContextMode,
) -> Result<Option<PointerValue<'ctx>>, Error> {
    match mode {
        ContextMode::None => Ok(None),
        ContextMode::Implicit => match function.get_first_param() {
            Some(BasicValueEnum::PointerValue(pointer)) => {
                pointer.set_name("context");
                Ok(Some(pointer))
            }
            _ => Err(Error::Invariant),
        },
    }
}

impl<'ctx> Generator<'ctx, '_, '_> {
    pub(super) fn context_slot(&mut self, ty: TypeId) -> Result<Slot<'ctx>, Error> {
        if self
            .context_definition
            .map(|definition| definition.record_type)
            != Some(ty)
        {
            return Err(Error::Invariant);
        }
        Ok(Slot {
            pointer: self.active_context.ok_or(Error::Invariant)?,
            ty,
            alignment: self.target.data.get_abi_alignment(&self.lowerer.basic(ty)?),
        })
    }

    pub(super) fn context_value(&mut self, ty: TypeId) -> Result<BasicValueEnum<'ctx>, Error> {
        let slot = self.context_slot(ty)?;
        memory::load(
            &self.builder,
            self.lowerer.basic(ty)?,
            slot.pointer,
            "context.snapshot",
            slot.alignment,
        )
    }

    pub(super) fn push_context(
        &mut self,
        id: PushContextId,
        value: &ValueExpr,
        body: &Block,
    ) -> Result<(), Error> {
        if id.procedure() != self.procedure {
            return Err(Error::Invariant);
        }
        let ty = value.type_id(self.types);
        if self
            .context_definition
            .map(|definition| definition.record_type)
            != Some(ty)
        {
            return Err(Error::Invariant);
        }
        // Evaluate under the previous context, then copy. A fresh entry alloca
        // avoids allocating repeatedly when a push appears inside a loop.
        let snapshot = self.value(value)?;
        let entry = self
            .function
            .get_first_basic_block()
            .ok_or(Error::Invariant)?;
        let allocations = self.context.create_builder();
        match entry.get_first_instruction() {
            Some(first) => allocations.position_before(&first),
            None => allocations.position_at_end(entry),
        }
        let pointer = allocations.build_alloca(self.lowerer.basic(ty)?, "context.pushed")?;
        let alignment = self.target.data.get_abi_alignment(&self.lowerer.basic(ty)?);
        memory::store(&self.builder, pointer, snapshot, alignment)?;
        if self.pushed_contexts.insert(id, pointer).is_some() {
            return Err(Error::Invariant);
        }
        let previous = self.active_context.replace(pointer);
        let result = self.debug_child_block(jai_ir::DebugBranch::PushContext, body);
        // This is compiler lexical state, so restoration must also happen when
        // the body's generated control flow returns, breaks, or continues.
        self.active_context = previous;
        self.pushed_contexts.remove(&id);
        result
    }

    pub(super) fn context_cleanup(&mut self, id: CleanupId) -> Result<(), Error> {
        let cleanup = self.cleanups.get(id.index()).ok_or(Error::Invariant)?;
        let pointer = match cleanup.context {
            CleanupContext::Procedure => self.procedure_context,
            CleanupContext::Push(id) => {
                Some(*self.pushed_contexts.get(&id).ok_or(Error::Invariant)?)
            }
        };
        let previous = std::mem::replace(&mut self.active_context, pointer);
        let result = self.debug_cleanup_block(id, &cleanup.body);
        self.active_context = previous;
        result
    }

    pub(super) fn context_arguments(
        &self,
        mode: ContextMode,
        arguments: &[(TypeId, BasicValueEnum<'ctx>)],
    ) -> Result<Vec<BasicMetadataValueEnum<'ctx>>, Error> {
        let mut values =
            Vec::with_capacity(arguments.len() + usize::from(mode == ContextMode::Implicit));
        if mode == ContextMode::Implicit {
            values.push(self.active_context.ok_or(Error::Invariant)?.into());
        }
        values.extend(
            arguments
                .iter()
                .map(|(_, value)| BasicMetadataValueEnum::from(*value)),
        );
        Ok(values)
    }
}
