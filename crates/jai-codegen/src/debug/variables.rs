//! Lexical scopes and precisely represented primitive runtime variables.
use super::{DebugInformation, Error, LineTables, bridge_error, source_descriptor};
use inkwell::{builder::Builder, context::Context, values::PointerValue};
use jai_ir::DebugSourceLocation;
pub use jai_llvm::DebugVariableKind as VariableKind;
use jai_llvm::{DebugPrimitive, DebugScope, DebugVariable};
use jai_types::{FloatType, IntegerType, TypeId, TypeKind, TypeView};
#[cfg(test)]
pub(super) mod tests;
#[derive(Clone, Copy)]
pub struct Scope<'ctx> {
    pub(super) metadata: DebugScope<'ctx>,
    pub(super) source: jai_source::SourceId,
}
#[derive(Clone, Copy)]
pub struct VariableRecord<'ctx> {
    metadata: DebugVariable<'ctx>,
}
impl<'ctx> LineTables<'ctx, '_> {
    pub fn lexical_scope(
        &self,
        parent: Scope<'ctx>,
        source: &DebugSourceLocation,
    ) -> Result<Scope<'ctx>, Error> {
        let metadata = self
            .session
            .lexical_scope(parent.metadata, source_descriptor(source)?)
            .map_err(bridge_error)?;
        Ok(Scope {
            metadata,
            source: source.span().source,
        })
    }
    pub fn statement_location(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        scope: Scope<'ctx>,
        source: &DebugSourceLocation,
    ) -> Result<(), Error> {
        if self.context != *context {
            return Err(Error::Ownership);
        }
        let scope = if scope.source == source.span().source {
            scope
        } else {
            self.lexical_scope(scope, source)?
        };
        self.session
            .set_location(builder, scope.metadata, source_descriptor(source)?)
            .map_err(bridge_error)
    }
    /// Returns false for a runtime type that does not yet have a faithful debug descriptor.
    /// Such variables are omitted; their storage is never described as an unrelated scalar.
    #[allow(clippy::too_many_arguments)]
    pub fn declare_variable(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        scope: Scope<'ctx>,
        source: &DebugSourceLocation,
        name: &str,
        kind: VariableKind,
        ty: TypeId,
        types: &dyn TypeView,
        storage: PointerValue<'ctx>,
        alignment_bytes: u32,
    ) -> Result<bool, Error> {
        let Some(variable) = self.create_variable(
            context,
            scope,
            source,
            name,
            kind,
            ty,
            types,
            alignment_bytes,
        )?
        else {
            return Ok(false);
        };
        self.emit_variable(builder, storage, variable)?;
        Ok(true)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_variable(
        &self,
        context: &'ctx Context,
        scope: Scope<'ctx>,
        source: &DebugSourceLocation,
        name: &str,
        kind: VariableKind,
        ty: TypeId,
        types: &dyn TypeView,
        alignment_bytes: u32,
    ) -> Result<Option<VariableRecord<'ctx>>, Error> {
        self.create_variable_with_sources(
            context,
            scope,
            source,
            name,
            kind,
            ty,
            types,
            alignment_bytes,
            None,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub fn create_variable_with_sources(
        &self,
        context: &'ctx Context,
        scope: Scope<'ctx>,
        source: &DebugSourceLocation,
        name: &str,
        kind: VariableKind,
        ty: TypeId,
        types: &dyn TypeView,
        alignment_bytes: u32,
        sources: Option<&jai_ir::DebugSources>,
    ) -> Result<Option<VariableRecord<'ctx>>, Error> {
        if self.context != *context {
            return Err(Error::Ownership);
        }
        if self.information != DebugInformation::Variables {
            return Ok(None);
        }
        let Some(runtime_type) = self.runtime_type(ty, types, sources)? else {
            return Ok(None);
        };
        let metadata = self
            .session
            .typed_variable(
                scope.metadata,
                source_descriptor(source)?,
                name,
                kind,
                runtime_type,
                alignment_bytes,
            )
            .map_err(bridge_error)?;
        Ok(Some(VariableRecord { metadata }))
    }
    pub fn emit_variable(
        &self,
        builder: &Builder<'ctx>,
        storage: PointerValue<'ctx>,
        variable: VariableRecord<'ctx>,
    ) -> Result<(), Error> {
        self.session
            .declare(builder, storage, variable.metadata)
            .map_err(bridge_error)
    }
    pub(super) fn primitive_type(
        &self,
        ty: TypeId,
        types: &dyn TypeView,
    ) -> Result<Option<DebugPrimitive>, Error> {
        Ok(Some(match types.kind(ty).map_err(Error::Type)? {
            TypeKind::Bool => DebugPrimitive::Bool,
            TypeKind::Integer(integer) => match integer {
                IntegerType::S8 => DebugPrimitive::Signed8,
                IntegerType::S16 => DebugPrimitive::Signed16,
                IntegerType::S32 => DebugPrimitive::Signed32,
                IntegerType::S64 => DebugPrimitive::Signed64,
                IntegerType::U8 => DebugPrimitive::Unsigned8,
                IntegerType::U16 => DebugPrimitive::Unsigned16,
                IntegerType::U32 => DebugPrimitive::Unsigned32,
                IntegerType::U64 => DebugPrimitive::Unsigned64,
            },
            TypeKind::Float(float) => match float {
                FloatType::F32 => DebugPrimitive::Float32,
                FloatType::F64 => DebugPrimitive::Float64,
            },
            _ => return Ok(None),
        }))
    }
}
