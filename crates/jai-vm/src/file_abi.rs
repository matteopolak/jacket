//! Binding proofs for a closed stdio ABI, selected by trusted source identities.
//! Symbol spelling alone never selects a VM capability.
use jai_ir::{
    ForeignLibrary, ForeignLibraryId, ForeignLibraryKind, ProcedureId, ProcedurePrototype,
    PrototypeOrigin,
};
use jai_types::{
    CallingConvention, ContextMode, IntegerType, RecordKind, TypeError, TypeId, TypeKind, TypeView,
    Variadic,
};
use std::collections::HashMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileAbiOperation {
    Open,
    Read,
    Write,
    Seek,
    Tell,
    Eof,
    Close,
}
impl FileAbiOperation {
    fn symbol_matches(self, symbol: &str) -> bool {
        match self {
            Self::Open => symbol == "fopen",
            Self::Read => symbol == "fread",
            Self::Write => symbol == "fwrite",
            Self::Seek => symbol == "fseek",
            Self::Tell => matches!(symbol, "ftello" | "ftello64"),
            Self::Eof => symbol == "feof",
            Self::Close => symbol == "fclose",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileAbiError {
    Type(TypeError),
    Authority(&'static str),
    Signature(&'static str),
}
impl From<TypeError> for FileAbiError {
    fn from(error: TypeError) -> Self {
        Self::Type(error)
    }
}
impl std::fmt::Display for FileAbiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "stdio ABI binding: {self:?}")
    }
}
impl std::error::Error for FileAbiError {
}
/// The embedding source binder must supply only declarations from its verified
/// selected POSIX stdio source. This trusted Rust API is unavailable to Jai code.
/// Its exact library and procedure identities remain authoritative after aliases/imports.
#[derive(Clone, Debug)]
pub struct StdioAuthority {
    library: ForeignLibrary,
    file_type: TypeId,
    declarations: HashMap<ProcedureId, FileAbiOperation>,
}
#[derive(Clone, Copy, Debug)]
pub struct FileAbiProcedure {
    pub signature: TypeId,
    procedure: ProcedureId,
    library: ForeignLibraryId,
    operation: FileAbiOperation,
    file_type: TypeId,
}
impl FileAbiProcedure {
    pub fn procedure(self) -> ProcedureId {
        self.procedure
    }
    pub fn library(self) -> ForeignLibraryId {
        self.library
    }
    pub fn operation(self) -> FileAbiOperation {
        self.operation
    }
    pub fn file_type(self) -> TypeId {
        self.file_type
    }
    pub fn validate(self, types: &dyn TypeView) -> Result<(), FileAbiError> {
        validate_signature(self.operation, self.signature, self.file_type, types)
    }
}
impl StdioAuthority {
    pub fn from_verified_source(
        library: ForeignLibrary,
        file_type: TypeId,
        declarations: impl IntoIterator<Item = (ProcedureId, FileAbiOperation)>,
        types: &dyn TypeView,
    ) -> Result<Self, FileAbiError> {
        library
            .validate()
            .map_err(|_| FileAbiError::Authority("invalid canonical library metadata"))?;
        if !matches!(library.id, ForeignLibraryId::File(_))
            || !matches!(&library.kind, ForeignLibraryKind::System { name } if name == "libc")
        {
            return Err(FileAbiError::Authority(
                "stdio requires its canonical source libc declaration",
            ));
        }
        if !matches!(types.kind(file_type)?, TypeKind::Record(_)) {
            return Err(FileAbiError::Authority(
                "FILE must retain its actual nominal record identity",
            ));
        }
        if types.record_definition(file_type)?.kind != RecordKind::Struct {
            return Err(FileAbiError::Authority(
                "FILE requires its actual struct definition",
            ));
        }
        let mut bindings = HashMap::new();
        for (procedure, operation) in declarations {
            if bindings.insert(procedure, operation).is_some() {
                return Err(FileAbiError::Authority(
                    "duplicate stdio declaration identity",
                ));
            }
        }
        Ok(Self {
            library,
            file_type,
            declarations: bindings,
        })
    }
    pub fn bind(
        &self,
        prototype: &ProcedurePrototype,
        types: &dyn TypeView,
    ) -> Result<FileAbiProcedure, FileAbiError> {
        let operation =
            self.declarations
                .get(&prototype.id)
                .copied()
                .ok_or(FileAbiError::Authority(
                    "procedure is not an authorized stdio declaration",
                ))?;
        let PrototypeOrigin::Foreign {
            symbol,
            library: Some(library),
        } = &prototype.origin
        else {
            return Err(FileAbiError::Authority(
                "stdio declaration must retain its foreign library binding",
            ));
        };
        if library != &self.library || !operation.symbol_matches(symbol) {
            return Err(FileAbiError::Authority(
                "foreign prototype differs from its stdio source receipt",
            ));
        }
        validate_signature(operation, prototype.signature, self.file_type, types)?;
        Ok(FileAbiProcedure {
            signature: prototype.signature,
            procedure: prototype.id,
            library: library.id,
            operation,
            file_type: self.file_type,
        })
    }
}
fn validate_signature(
    operation: FileAbiOperation,
    signature: TypeId,
    file: TypeId,
    types: &dyn TypeView,
) -> Result<(), FileAbiError> {
    let signature = types.procedure_definition(signature)?;
    if signature.convention != CallingConvention::C
        || signature.context != ContextMode::None
        || signature.variadic != Variadic::None
    {
        return Err(FileAbiError::Signature(
            "stdio requires a fixed C ABI without context",
        ));
    }
    let integer = |ty: TypeId, expected| -> Result<bool, TypeError> {
        Ok(matches!(types.kind(ty)?, TypeKind::Integer(actual) if *actual == expected))
    };
    let pointer = |ty: TypeId, expected: TypeId| -> Result<bool, TypeError> {
        Ok(matches!(types.kind(ty)?, TypeKind::Pointer(actual) if *actual == expected))
    };
    let byte_pointer = |ty: TypeId| -> Result<bool, TypeError> {
        Ok(match types.kind(ty)? {
            TypeKind::Pointer(pointee) => integer(*pointee, IntegerType::U8)?,
            _ => false,
        })
    };
    let void_pointer = |ty: TypeId| -> Result<bool, TypeError> {
        Ok(
            matches!(types.kind(ty)?, TypeKind::Pointer(pointee) if matches!(types.kind(*pointee)?, TypeKind::Void)),
        )
    };
    let valid = match (
        operation,
        signature.parameters.as_ref(),
        signature.results.as_ref(),
    ) {
        (FileAbiOperation::Open, [path, mode], [result]) => {
            byte_pointer(*path)? && byte_pointer(*mode)? && pointer(*result, file)?
        }
        (
            FileAbiOperation::Read | FileAbiOperation::Write,
            [buffer, size, count, stream],
            [result],
        ) => {
            void_pointer(*buffer)?
                && integer(*size, IntegerType::U64)?
                && integer(*count, IntegerType::U64)?
                && pointer(*stream, file)?
                && integer(*result, IntegerType::U64)?
        }
        (FileAbiOperation::Seek, [stream, offset, origin], [result]) => {
            pointer(*stream, file)?
                && integer(*offset, IntegerType::S64)?
                && integer(*origin, IntegerType::S32)?
                && integer(*result, IntegerType::S32)?
        }
        (FileAbiOperation::Tell, [stream], [result]) => {
            pointer(*stream, file)? && integer(*result, IntegerType::S64)?
        }
        (FileAbiOperation::Eof | FileAbiOperation::Close, [stream], [result]) => {
            pointer(*stream, file)? && integer(*result, IntegerType::S32)?
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(FileAbiError::Signature(
            "stdio parameters/results or nominal FILE identity differ",
        ))
    }
}
#[cfg(test)]
#[path = "file_abi/tests.rs"]
mod tests;
