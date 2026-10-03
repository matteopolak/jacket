//! Exact source-bound C allocator capabilities, independent of symbol lookup.
use crate::{Error, LimitKind, Memory, Value, virtual_heap::VirtualHeap};
use jai_ir::{
    ForeignLibrary, ForeignLibraryId, ForeignLibraryKind, ProcedureId, ProcedurePrototype,
    PrototypeOrigin,
};
use jai_types::{
    CallingConvention, ContextMode, IntegerType, TypeError, TypeId, TypeKind, TypeView, Variadic,
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeapAbiOperation {
    Malloc,
    Realloc,
    Free,
}
impl HeapAbiOperation {
    fn symbol(self) -> &'static str {
        match self {
            Self::Malloc => "malloc",
            Self::Realloc => "realloc",
            Self::Free => "free",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HeapAbiError {
    Type(TypeError),
    Authority(&'static str),
    Signature(&'static str),
}
impl From<TypeError> for HeapAbiError {
    fn from(error: TypeError) -> Self {
        Self::Type(error)
    }
}
impl std::fmt::Display for HeapAbiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "heap ABI binding: {self:?}")
    }
}
impl std::error::Error for HeapAbiError {}
#[derive(Clone, Debug)]
pub struct HeapAuthority {
    library: ForeignLibrary,
    declarations: HashMap<ProcedureId, HeapAbiOperation>,
}
#[derive(Clone, Copy, Debug)]
pub struct HeapAbiProcedure {
    pub signature: TypeId,
    procedure: ProcedureId,
    library: ForeignLibraryId,
    operation: HeapAbiOperation,
}
impl HeapAuthority {
    /// Trusted Rust source binder only: the caller must retain and validate its
    /// graph/source/target receipt before supplying actual allocator declarations.
    pub fn from_verified_source(
        library: ForeignLibrary,
        declarations: impl IntoIterator<Item = (ProcedureId, HeapAbiOperation)>,
    ) -> Result<Self, HeapAbiError> {
        library
            .validate()
            .map_err(|_| HeapAbiError::Authority("invalid canonical allocator library"))?;
        if !matches!(library.id, ForeignLibraryId::File(_))
            || !matches!(&library.kind, ForeignLibraryKind::System { name } if name == "libc")
        {
            return Err(HeapAbiError::Authority(
                "allocator requires its actual source libc library",
            ));
        }
        let mut bindings = HashMap::new();
        for (id, operation) in declarations {
            if bindings.insert(id, operation).is_some() {
                return Err(HeapAbiError::Authority(
                    "duplicate allocator declaration identity",
                ));
            }
        }
        Ok(Self {
            library,
            declarations: bindings,
        })
    }
    pub fn bind(
        &self,
        prototype: &ProcedurePrototype,
        types: &dyn TypeView,
    ) -> Result<HeapAbiProcedure, HeapAbiError> {
        let operation = *self
            .declarations
            .get(&prototype.id)
            .ok_or(HeapAbiError::Authority(
                "procedure is not an authorized allocator declaration",
            ))?;
        let PrototypeOrigin::Foreign {
            symbol,
            library: Some(library),
        } = &prototype.origin
        else {
            return Err(HeapAbiError::Authority(
                "allocator must retain its foreign library binding",
            ));
        };
        if library != &self.library || symbol != operation.symbol() {
            return Err(HeapAbiError::Authority(
                "allocator differs from its source receipt",
            ));
        }
        validate_signature(operation, prototype.signature, types)?;
        Ok(HeapAbiProcedure {
            signature: prototype.signature,
            procedure: prototype.id,
            library: library.id,
            operation,
        })
    }
}
impl HeapAbiProcedure {
    pub fn procedure(self) -> ProcedureId {
        self.procedure
    }
    pub fn library(self) -> ForeignLibraryId {
        self.library
    }
    pub fn operation(self) -> HeapAbiOperation {
        self.operation
    }
    pub fn validate(self, types: &dyn TypeView) -> Result<(), HeapAbiError> {
        validate_signature(self.operation, self.signature, types)
    }
    fn arguments<'a>(
        self,
        args: &'a [Value],
        types: &dyn TypeView,
    ) -> Result<(Option<&'a crate::Pointer>, usize), Error> {
        // Preserve structured TypeError (including pending types) in VM execution.
        validate_signature(self.operation, self.signature, types).map_err(|error| match error {
            HeapAbiError::Type(error) => Error::Type(error),
            _ => Error::InvalidIr("invalid allocator ABI signature"),
        })?;
        let signature = types.procedure_definition(self.signature)?;
        if args.len() != signature.parameters.len() {
            return Err(Error::InvalidIr(
                "allocator argument count differs from signature",
            ));
        }
        for (value, ty) in args.iter().zip(signature.parameters.iter()) {
            value.validate(types, *ty, 128)?;
        }
        let size = |value: &Value| -> Result<usize, Error> {
            if matches!(value, Value::AddressInteger(number) if number.provenance().is_some()) {
                return Err(Error::InvalidIr(
                    "heap byte count carries address provenance",
                ));
            }
            usize::try_from(value.integer()?.value())
                .map_err(|_| Error::Limit(LimitKind::ValueCells))
        };
        match (self.operation, args) {
            (HeapAbiOperation::Malloc, [bytes]) => Ok((None, size(bytes)?)),
            (HeapAbiOperation::Realloc, [pointer, bytes]) => {
                Ok((Some(pointer.pointer()?), size(bytes)?))
            }
            (HeapAbiOperation::Free, [pointer]) => Ok((Some(pointer.pointer()?), 0)),
            _ => Err(Error::InvalidIr("invalid allocator arguments")),
        }
    }
    pub fn work_cost(
        self,
        args: &[Value],
        memory: &Memory,
        types: &dyn TypeView,
        heap: &VirtualHeap,
    ) -> Result<u64, Error> {
        let (pointer, size) = self.arguments(args, types)?;
        match (self.operation, pointer) {
            (HeapAbiOperation::Malloc, None) => {
                memory.preflight_host_heap_allocation(size)?;
                heap.malloc_work_cost(size)
            }
            (HeapAbiOperation::Realloc, Some(pointer)) => {
                heap.realloc_work_cost(memory, types, pointer, size)
            }
            (HeapAbiOperation::Free, Some(pointer)) => heap.free_work_cost(memory, types, pointer),
            _ => Err(Error::InvalidIr("invalid allocator work arguments")),
        }
    }
    pub fn invoke(
        self,
        args: &[Value],
        memory: &mut Memory,
        types: &dyn TypeView,
        heap: &mut VirtualHeap,
    ) -> Result<Vec<Value>, Error> {
        let (pointer, size) = self.arguments(args, types)?;
        match (self.operation, pointer) {
            (HeapAbiOperation::Malloc, None) => {
                Ok(vec![Value::Pointer(heap.malloc(memory, types, size)?)])
            }
            (HeapAbiOperation::Realloc, Some(pointer)) => Ok(vec![Value::Pointer(
                heap.realloc(memory, types, pointer, size)?,
            )]),
            (HeapAbiOperation::Free, Some(pointer)) => {
                heap.free(memory, types, pointer)?;
                Ok(vec![])
            }
            _ => Err(Error::InvalidIr("invalid allocator invoke arguments")),
        }
    }
}
fn validate_signature(
    operation: HeapAbiOperation,
    signature: TypeId,
    types: &dyn TypeView,
) -> Result<(), HeapAbiError> {
    let signature = types.procedure_definition(signature)?;
    if signature.convention != CallingConvention::C
        || signature.context != ContextMode::None
        || signature.variadic != Variadic::None
    {
        return Err(HeapAbiError::Signature(
            "allocator requires fixed C ABI without context",
        ));
    }
    let size = |ty| -> Result<bool, TypeError> {
        Ok(matches!(
            types.kind(ty)?,
            TypeKind::Integer(IntegerType::U64)
        ))
    };
    let void_pointer = |ty| -> Result<bool, TypeError> {
        Ok(
            matches!(types.kind(ty)?, TypeKind::Pointer(pointee) if matches!(types.kind(*pointee)?, TypeKind::Void)),
        )
    };
    let valid = match (
        operation,
        signature.parameters.as_ref(),
        signature.results.as_ref(),
    ) {
        (HeapAbiOperation::Malloc, [bytes], [result]) => size(*bytes)? && void_pointer(*result)?,
        (HeapAbiOperation::Realloc, [pointer, bytes], [result]) => {
            void_pointer(*pointer)? && size(*bytes)? && void_pointer(*result)?
        }
        (HeapAbiOperation::Free, [pointer], []) => void_pointer(*pointer)?,
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(HeapAbiError::Signature(
            "allocator parameter or result shape differs",
        ))
    }
}
#[cfg(test)]
#[path = "heap_abi/tests.rs"]
mod tests;
#[cfg(test)]
#[path = "heap_abi/vm_tests.rs"]
mod vm_tests;
