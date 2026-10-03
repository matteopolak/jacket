//! Owned debug metadata: arbitrary Inkwell metadata cannot enter the raw bridge.
use crate::{Error, raw};
use inkwell::{
    basic_block::BasicBlock,
    builder::{Builder, BuilderError},
    context::Context,
    debug_info::{
        AsDIScope, DICompileUnit, DIExpression, DIFlags, DIFlagsConstants, DILocation, DIScope,
        DWARFEmissionKind, DWARFSourceLanguage, DebugInfoBuilder, debug_metadata_version,
    },
    module::{FlagBehavior, Module},
    values::{FunctionValue, PointerValue},
};
use std::{
    num::{NonZeroU32, NonZeroU64},
    sync::atomic::{AtomicU64, Ordering},
};
pub(super) mod types;
pub use types::{DebugMember, DebugRecordKind, DebugType};
mod procedures;
pub use procedures::{DebugCallingConvention, DebugResultMember, DebugVariadic};
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug)]
pub enum DebugEmission {
    LineTables,
    Variables,
}
#[derive(Clone, Copy, Debug)]
pub enum DebugPrimitive {
    Bool,
    Signed8,
    Signed16,
    Signed32,
    Signed64,
    Unsigned8,
    Unsigned16,
    Unsigned32,
    Unsigned64,
    Float32,
    Float64,
}
#[derive(Clone, Copy, Debug)]
pub enum DebugVariableKind {
    Automatic,
    Parameter(NonZeroU32),
}
#[derive(Clone, Copy)]
pub struct DebugSource<'a> {
    pub file: &'a str,
    pub directory: &'a str,
    pub line: NonZeroU32,
    pub column: NonZeroU32,
}
#[derive(Clone, Copy)]
pub struct DebugScope<'ctx> {
    metadata: DIScope<'ctx>,
    owner: NonZeroU64,
    function: FunctionValue<'ctx>,
}
#[derive(Clone, Copy)]
pub struct DebugVariable<'ctx> {
    metadata: inkwell::llvm_sys::prelude::LLVMMetadataRef,
    location: DILocation<'ctx>,
    owner: NonZeroU64,
    function: FunctionValue<'ctx>,
}

/// The module borrow keeps the owning LLVM module alive until builder finalization.
/// Opaque scopes and variables can only be constructed by this session.
pub struct DebugSession<'module, 'ctx> {
    module: &'module Module<'ctx>,
    context: &'ctx Context,
    debug: DebugInfoBuilder<'ctx>,
    unit: DICompileUnit<'ctx>,
    owner: NonZeroU64,
    optimized: bool,
    types: types::TypeTable<'ctx>,
}
impl Drop for DebugSession<'_, '_> {
    fn drop(&mut self) {
        // Inkwell finalizes its builder during Drop. Resolve every remaining
        // temporary first, including when an error exits native lowering early.
        self.finish_types();
    }
}
impl<'module, 'ctx> DebugSession<'module, 'ctx> {
    pub fn new(
        module: &'module Module<'ctx>,
        context: &'ctx Context,
        file: &str,
        directory: &str,
        producer: &str,
        optimized: bool,
        emission: DebugEmission,
    ) -> Result<Self, Error> {
        if module.get_context() != *context {
            return Err(Error::ContextMismatch);
        }
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let owner = loop {
            let value = NEXT.load(Ordering::Relaxed);
            let next = value.checked_add(1).ok_or(Error::NullResult)?;
            if NEXT
                .compare_exchange_weak(value, next, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break NonZeroU64::new(value).ok_or(Error::NullResult)?;
            }
        };
        for (key, value) in [
            ("Debug Info Version", u64::from(debug_metadata_version())),
            ("Dwarf Version", 4),
        ] {
            if module.get_flag(key).is_none() {
                module.add_basic_value_flag(
                    key,
                    FlagBehavior::Warning,
                    context.i32_type().const_int(value, false),
                );
            }
        }
        let (debug, unit) = module.create_debug_info_builder(
            true,
            DWARFSourceLanguage::C,
            file,
            directory,
            producer,
            optimized,
            "",
            0,
            "",
            match emission {
                DebugEmission::LineTables => DWARFEmissionKind::LineTablesOnly,
                DebugEmission::Variables => DWARFEmissionKind::Full,
            },
            0,
            false,
            false,
            "",
            "",
        );
        Ok(Self {
            module,
            context,
            debug,
            unit,
            owner,
            optimized,
            types: std::cell::RefCell::new(Vec::new()),
        })
    }
    pub fn context_matches(&self, context: &Context) -> bool {
        self.module.get_context() == *context
    }
    fn block(
        &self,
        builder: &Builder<'ctx>,
        function: FunctionValue<'ctx>,
    ) -> Result<BasicBlock<'ctx>, Error> {
        let block = builder
            .get_insert_block()
            .ok_or(BuilderError::UnsetPosition)?;
        if block.get_context() != self.module.get_context() || block.get_parent() != Some(function)
        {
            return Err(Error::ContextMismatch);
        }
        if !raw::function_in_module(function, self.module) {
            return Err(Error::ContextMismatch);
        }
        Ok(block)
    }
    fn scope(&self, scope: DebugScope<'ctx>) -> Result<(), Error> {
        if scope.owner != self.owner {
            return Err(Error::DebugOwnership);
        }
        Ok(())
    }
    pub fn function_scope(
        &self,
        builder: &Builder<'ctx>,
        function: FunctionValue<'ctx>,
        source: DebugSource<'_>,
        name: &str,
    ) -> Result<DebugScope<'ctx>, Error> {
        self.source(source)?;
        self.block(builder, function)?;
        let file = self.debug.create_file(source.file, source.directory);
        let signature = self
            .debug
            .create_subroutine_type(file, None, &[], DIFlags::ZERO);
        let linkage = function
            .get_name()
            .to_str()
            .map_err(|_| Error::InvalidName)?;
        let metadata = self.debug.create_function(
            self.unit.as_debug_info_scope(),
            name,
            Some(linkage),
            file,
            source.line.get(),
            signature,
            false,
            true,
            source.line.get(),
            DIFlags::ZERO,
            self.optimized,
        );
        function.set_subprogram(metadata);
        let scope = DebugScope {
            metadata: metadata.as_debug_info_scope(),
            owner: self.owner,
            function,
        };
        self.set_location(builder, scope, source)?;
        Ok(scope)
    }
    pub fn lexical_scope(
        &self,
        parent: DebugScope<'ctx>,
        source: DebugSource<'_>,
    ) -> Result<DebugScope<'ctx>, Error> {
        self.source(source)?;
        self.scope(parent)?;
        let file = self.debug.create_file(source.file, source.directory);
        Ok(DebugScope {
            metadata: self
                .debug
                .create_lexical_block(
                    parent.metadata,
                    file,
                    source.line.get(),
                    source.column.get(),
                )
                .as_debug_info_scope(),
            owner: self.owner,
            function: parent.function,
        })
    }
    fn location(&self, scope: DebugScope<'ctx>, source: DebugSource<'_>) -> DILocation<'ctx> {
        self.debug.create_debug_location(
            self.context,
            source.line.get(),
            source.column.get(),
            scope.metadata,
            None,
        )
    }
    fn source(&self, source: DebugSource<'_>) -> Result<(), Error> {
        // LLVM's location/lexical-block columns occupy a 16-bit field.
        u16::try_from(source.column.get()).map_err(|_| Error::InvalidDebugCoordinates)?;
        Ok(())
    }
    pub fn set_location(
        &self,
        builder: &Builder<'ctx>,
        scope: DebugScope<'ctx>,
        source: DebugSource<'_>,
    ) -> Result<(), Error> {
        self.source(source)?;
        self.scope(scope)?;
        self.block(builder, scope.function)?;
        builder.set_current_debug_location(self.location(scope, source));
        Ok(())
    }
    /// A compiler-generated call site has no source line. Its genuine owning
    /// subprogram still anchors metadata required by LLVM's debug verifier.
    pub fn set_artificial_location(
        &self,
        builder: &Builder<'ctx>,
        scope: DebugScope<'ctx>,
    ) -> Result<(), Error> {
        self.scope(scope)?;
        self.block(builder, scope.function)?;
        let location = self
            .debug
            .create_debug_location(self.context, 0, 0, scope.metadata, None);
        raw::suppressed_debug::retain(self.module, location);
        builder.set_current_debug_location(location);
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    pub fn variable(
        &self,
        scope: DebugScope<'ctx>,
        source: DebugSource<'_>,
        name: &str,
        kind: DebugVariableKind,
        primitive: DebugPrimitive,
        alignment_bytes: u32,
    ) -> Result<DebugVariable<'ctx>, Error> {
        self.scope(scope)?;
        let ty = self.primitive_type(primitive)?;
        self.typed_variable(scope, source, name, kind, ty, alignment_bytes)
    }
    pub fn declare(
        &self,
        builder: &Builder<'ctx>,
        storage: PointerValue<'ctx>,
        variable: DebugVariable<'ctx>,
    ) -> Result<(), Error> {
        if variable.owner != self.owner {
            return Err(Error::DebugOwnership);
        }
        let block = self.block(builder, variable.function)?;
        if storage.get_type().get_context() != self.module.get_context() {
            return Err(Error::ContextMismatch);
        }
        if !raw::debug_storage_in_owner(storage, variable.function, self.module) {
            return Err(Error::DebugStorageOwnership);
        }
        raw::debug_declare(CheckedDebugDeclare {
            debug: &self.debug,
            storage,
            variable: variable.metadata,
            expression: self.debug.create_expression(vec![]),
            location: variable.location,
            block,
        })
    }
    pub fn finish(self) {
        self.finish_types();
        self.debug.finalize();
    }
}
pub(super) struct CheckedDebugDeclare<'a, 'ctx> {
    pub(super) debug: &'a DebugInfoBuilder<'ctx>,
    pub(super) storage: PointerValue<'ctx>,
    pub(super) variable: inkwell::llvm_sys::prelude::LLVMMetadataRef,
    pub(super) expression: DIExpression<'ctx>,
    pub(super) location: DILocation<'ctx>,
    pub(super) block: BasicBlock<'ctx>,
}
