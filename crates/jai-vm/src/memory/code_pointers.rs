//! Memory-owned provenance for opaque procedure addresses. A code token never
//! identifies a data allocation or readable bytes.
use super::*;
use jai_ir::ProcedureId;

/// An existing canonical procedure token, bound to its issuing Memory.
/// This is address provenance, not permission to execute a procedure body.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct CodePointer {
    memory: u64,
    signature: TypeId,
    procedure: ProcedureId,
    token: u64,
}

impl CodePointer {
    pub(crate) fn memory_identity(self) -> u64 {
        self.memory
    }

    pub(crate) fn signature(self) -> TypeId {
        self.signature
    }

    pub(crate) fn procedure(self) -> ProcedureId {
        self.procedure
    }

    pub(crate) fn token(self) -> u64 {
        self.token
    }
}

impl Memory {
    /// Issue a receipt only for a nonnull procedure already admitted to the ledger.
    /// An unowned ByteImage token or a matching plain integer cannot issue a receipt.
    pub(crate) fn code_pointer(
        &self,
        types: &dyn TypeView,
        value: &Value,
    ) -> Result<CodePointer, Error> {
        let Value::Procedure {
            signature,
            procedure,
        } = value
        else {
            return Err(Error::InvalidIr("code address requires a procedure handle"));
        };
        types.procedure_definition(*signature)?;
        let procedure = procedure.ok_or(Error::NullProcedure)?;
        let token = self
            .handle_tokens
            .borrow()
            .values
            .get(&HandleKey::Procedure {
                signature: *signature,
                procedure,
            })
            .copied()
            .ok_or(Error::UnsupportedPointerOperation(
                "procedure has no canonical virtual address token",
            ))?;
        Ok(CodePointer {
            memory: self.identity,
            signature: *signature,
            procedure,
            token,
        })
    }

    pub(crate) fn validate_code_pointer(
        &self,
        types: &dyn TypeView,
        pointer: CodePointer,
    ) -> Result<(), Error> {
        if pointer.memory != self.identity {
            return Err(Error::ForeignPointer);
        }
        types.procedure_definition(pointer.signature)?;
        if self
            .handle_tokens
            .borrow()
            .values
            .get(&HandleKey::Procedure {
                signature: pointer.signature,
                procedure: pointer.procedure,
            })
            .copied()
            != Some(pointer.token)
        {
            return Err(Error::DanglingPointer);
        }
        Ok(())
    }

    /// Recover the original callable handle; the VM still checks its provider.
    pub(crate) fn code_pointer_value(
        &self,
        types: &dyn TypeView,
        pointer: CodePointer,
        signature: TypeId,
    ) -> Result<Value, Error> {
        self.validate_code_pointer(types, pointer)?;
        types.procedure_definition(signature)?;
        if signature != pointer.signature {
            return Err(Error::TypeMismatch {
                expected: signature,
            });
        }
        Ok(Value::Procedure {
            signature,
            procedure: Some(pointer.procedure),
        })
    }

    /// The caller must retain `pointer` as provenance alongside these integer bits.
    /// This helper does not construct a portable scalar or publish an address.
    pub(crate) fn code_pointer_integer(
        &self,
        types: &dyn TypeView,
        pointer: CodePointer,
        target: IntegerType,
    ) -> Result<Integer, Error> {
        self.validate_code_pointer(types, pointer)?;
        if target.bits() < self.code_pointer_bits()? {
            return Err(Error::UnsupportedPointerOperation(
                "nonnull code address cannot convert to a narrower integer",
            ));
        }
        Ok(Integer::wrapping(target, i128::from(pointer.token)))
    }

    /// Only an opaque original receipt can recover code-address provenance.
    pub(crate) fn code_pointer_from_integer(
        &self,
        types: &dyn TypeView,
        origin: CodePointer,
        integer: Integer,
        mode: CastMode,
    ) -> Result<CodePointer, Error> {
        self.validate_code_pointer(types, origin)?;
        let bits = self.code_pointer_bits()?;
        let maximum = u64::MAX >> (64 - bits);
        let address = if integer.ty().bits() == bits {
            integer.bits()
        } else if integer.ty().bits() < bits {
            integer.value() as u64 & maximum
        } else {
            if mode == CastMode::Checked
                && (integer.value() < 0 || integer.value() > i128::from(maximum))
            {
                return Err(Error::CheckedCast);
            }
            integer.bits() & maximum
        };
        if address != origin.token {
            return Err(Error::UnsupportedPointerOperation(
                "integer bits no longer match their proven code address",
            ));
        }
        Ok(origin)
    }

    fn code_pointer_bits(&self) -> Result<u32, Error> {
        match self.target.policy.pointer().size {
            size @ (1 | 2 | 4 | 8) => Ok(size as u32 * 8),
            _ => Err(Error::UnsupportedPointerOperation(
                "target pointer width exceeds the virtual address representation",
            )),
        }
    }
}

#[cfg(test)]
mod tests;
