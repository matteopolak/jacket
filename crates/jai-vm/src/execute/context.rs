//! Dynamic implicit context uses checked virtual record storage, never a host pointer.
use super::*;
use jai_types::ContextMode;

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub(super) fn initialize_context(&mut self) -> Result<()> {
        // Root expressions have logical context availability; storage is needed
        // only when they read it or enter an implicit-context procedure.
        self.current_context = self.default_context.clone();
        Ok(())
    }

    pub(super) fn context_pointer(&mut self, record_type: TypeId) -> Result<Pointer> {
        let definition = self
            .provider
            .context()
            .ok_or(Error::InvalidIr("implicit context schema is unavailable"))?;
        if definition.record_type != record_type {
            return Err(Error::TypeMismatch {
                expected: definition.record_type,
            }
            .into());
        }
        if self.current_context.is_none() && self.frames.is_empty() {
            self.current_context = Some(self.default_context_pointer()?);
        }
        self.current_context
            .clone()
            .ok_or_else(|| Error::InvalidIr("implicit context is unavailable in this call").into())
    }

    pub(super) fn context_value(&mut self, record_type: TypeId) -> Result<Value> {
        let pointer = self.context_pointer(record_type)?;
        self.prepare_pointer_layouts(&pointer, true)?;
        self.charge_work(
            self.memory
                .load_work_cost(self.provider.types(), &pointer)?,
        )?;
        Ok(self.memory.load(self.provider.types(), &pointer)?)
    }

    /// Enter after argument evaluation. No-context callees hide even an inherited
    /// context; an explicit push inside that callee can establish a fresh copy.
    pub(super) fn enter_call_context(&mut self, mode: ContextMode) -> Result<Option<Pointer>> {
        let previous = self.current_context.clone();
        if mode == ContextMode::Implicit
            && previous.is_none()
            && let Some(frame) = self.frames.last()
            && self.signature(frame.procedure.signature)?.context == ContextMode::None
        {
            return Err(
                Error::InvalidIr("implicit-context callee requires an active context").into(),
            );
        }
        self.current_context = match mode {
            ContextMode::None => None,
            ContextMode::Implicit => {
                if let Some(pointer) = &previous {
                    Some(pointer.clone())
                } else if self.provider.context().is_none() {
                    // Legacy scalar-only checked IR may have implicit signatures
                    // without needing any concrete context storage.
                    None
                } else if self.frames.is_empty() {
                    Some(self.default_context_pointer()?)
                } else {
                    return Err(Error::InvalidIr(
                        "implicit-context callee requires an active context",
                    )
                    .into());
                }
            }
        };
        Ok(previous)
    }

    pub(super) fn restore_call_context(&mut self, previous: Option<Pointer>) {
        self.current_context = previous;
    }

    fn default_context_pointer(&mut self) -> Result<Pointer> {
        if let Some(pointer) = &self.default_context {
            return Ok(pointer.clone());
        }
        let definition = self
            .provider
            .context()
            .ok_or(Error::InvalidIr("implicit context schema is unavailable"))?;
        let value = self.constant_value(&definition.default, 0)?;
        self.prepare_layout(definition.record_type)?;
        let pointer =
            self.memory
                .allocate(self.provider.types(), definition.record_type, Some(value))?;
        self.default_context = Some(pointer.clone());
        Ok(pointer)
    }

    pub(super) fn push_context(
        &mut self,
        id: PushContextId,
        expression: &ValueExpr,
        body: &Block,
        depth: usize,
    ) -> Result<Control> {
        let frame = self.frame()?;
        if id.procedure() != frame.procedure.id || frame.push_contexts.contains_key(&id) {
            return Err(
                Error::InvalidIr("push context identity is not available in this frame").into(),
            );
        }
        // Capture the record before replacing the old context. The value evaluator
        // and Memory both enforce the configured structural/value limits.
        let value = self.value(expression, depth + 1)?;
        let record_type = self
            .provider
            .context()
            .ok_or(Error::InvalidIr("implicit context schema is unavailable"))?
            .record_type;
        value.validate(
            self.provider.types(),
            record_type,
            self.limits.evaluation_depth.min(256),
        )?;
        self.prepare_layout(record_type)?;
        let pointer = self
            .memory
            .allocate(self.provider.types(), record_type, Some(value))?;
        self.frames
            .last_mut()
            .ok_or(Error::InvalidIr("push context requires an execution frame"))?
            .push_contexts
            .insert(id, pointer.clone());
        let previous = self.current_context.replace(pointer.clone());
        let result = self.block(body, depth + 1);
        // block executes exit cleanups before returning its captured transfer.
        // Restore even when a dependency, arithmetic failure or fuel limit halts it.
        self.current_context = previous;
        self.frames
            .last_mut()
            .ok_or(Error::InvalidIr("push context lost its execution frame"))?
            .push_contexts
            .remove(&id);
        let released = self.memory.release(&pointer);
        match result {
            Ok(control) => {
                released?;
                Ok(control)
            }
            Err(error) => Err(error),
        }
    }

    pub(super) fn cleanup_in_context(
        &mut self,
        context: CleanupContext,
        body: &Block,
        depth: usize,
    ) -> Result<()> {
        let frame = self.frame()?;
        let captured = match context {
            CleanupContext::Procedure => frame.procedure_context.clone(),
            CleanupContext::Push(id) => Some(frame.push_contexts.get(&id).cloned().ok_or(
                Error::InvalidIr("cleanup captured an inactive push context"),
            )?),
        };
        let previous = std::mem::replace(&mut self.current_context, captured);
        let result = self.block(body, depth + 1);
        self.current_context = previous;
        if !matches!(result?, Control::Next) {
            return Err(Error::InvalidIr("cleanup transferred control outside its body").into());
        }
        Ok(())
    }
}
