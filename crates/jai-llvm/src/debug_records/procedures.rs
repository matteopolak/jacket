//! Source signatures are non-storage types; only their pointers have runtime storage.
use super::*;
use types::{valid_alignment, valid_layout};
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebugVariadic {
    None,
    C,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebugCallingConvention {
    Normal,
    X86Stdcall,
    CppThiscall,
}

/// A compiler-generated, unnamed result tuple member at a checked target offset.
#[derive(Clone, Copy)]
pub struct DebugResultMember {
    pub ty: DebugType,
    pub offset_bits: u64,
    pub alignment_bits: u32,
}

impl<'ctx> DebugSession<'_, 'ctx> {
    /// Attaches a same-session source signature to an owned function scope.
    /// Lexical scopes and storage types cannot stand in for a subprogram type.
    pub fn set_function_type(
        &self,
        scope: DebugScope<'ctx>,
        signature: DebugType,
    ) -> Result<(), Error> {
        self.scope(scope)?;
        let slot = self.type_slot(signature)?;
        let table = self.types.borrow();
        if table[slot].storage
            || !raw::debug_types::set_function_signature(scope.metadata, table[slot].metadata)
        {
            return Err(Error::InvalidDebugType);
        }
        Ok(())
    }
    /// Describes a source signature using LLVM's normal calling-convention encoding.
    /// The return entry is void when absent. Hidden ABI parameters belong outside
    /// this source signature; unsupported calling conventions must not use it.
    pub fn subroutine_type(
        &self,
        result: Option<DebugType>,
        parameters: &[DebugType],
        variadic: DebugVariadic,
    ) -> Result<DebugType, Error> {
        self.subroutine_type_with_convention(
            result,
            parameters,
            variadic,
            DebugCallingConvention::Normal,
        )
    }
    pub fn subroutine_type_with_convention(
        &self,
        result: Option<DebugType>,
        parameters: &[DebugType],
        variadic: DebugVariadic,
        convention: DebugCallingConvention,
    ) -> Result<DebugType, Error> {
        let count = parameters
            .len()
            .checked_add(2)
            .ok_or(Error::InvalidDebugType)?;
        u32::try_from(count).map_err(|_| Error::InvalidDebugType)?;
        let metadata = |ty| {
            let slot = self.type_slot(ty)?;
            let table = self.types.borrow();
            if !table[slot].storage {
                return Err(Error::InvalidDebugType);
            }
            Ok(table[slot].metadata)
        };
        let mut signature = Vec::with_capacity(count);
        signature.push(
            result
                .map(metadata)
                .transpose()?
                .unwrap_or(std::ptr::null_mut()),
        );
        for &parameter in parameters {
            signature.push(metadata(parameter)?);
        }
        if variadic == DebugVariadic::C {
            signature.push(std::ptr::null_mut());
        }
        let raw = raw::debug_types::subroutine(&self.debug, &mut signature, convention);
        let ty = self.add_type(raw, 0, 0)?;
        self.types.borrow_mut()[ty.slot].storage = false;
        Ok(ty)
    }

    pub fn result_tuple_type(
        &self,
        size_bits: u64,
        alignment_bits: u32,
        members: &[DebugResultMember],
    ) -> Result<DebugType, Error> {
        valid_layout(size_bits, alignment_bits)?;
        if members.len() < 2 {
            return Err(Error::InvalidDebugType);
        }
        u32::try_from(members.len()).map_err(|_| Error::InvalidDebugType)?;
        let mut children = Vec::with_capacity(members.len());
        let mut elements = Vec::with_capacity(members.len());
        let table = self.types.borrow();
        let mut end = 0_u64;
        let mut tuple_alignment = 8_u32;
        for member in members {
            let slot = self.type_slot(member.ty)?;
            let child = &table[slot];
            valid_alignment(member.alignment_bits)?;
            let alignment = u64::from(member.alignment_bits);
            let expected = end
                .checked_add(alignment - 1)
                .map(|offset| offset & !(alignment - 1))
                .ok_or(Error::InvalidDebugType)?;
            if !child.storage
                || member.offset_bits != expected
                || member.alignment_bits > alignment_bits
                || !member
                    .offset_bits
                    .is_multiple_of(u64::from(member.alignment_bits))
                || member
                    .offset_bits
                    .checked_add(child.size)
                    .is_none_or(|end| end > size_bits)
            {
                return Err(Error::InvalidDebugType);
            }
            end = member
                .offset_bits
                .checked_add(child.size)
                .ok_or(Error::InvalidDebugType)?;
            tuple_alignment = tuple_alignment.max(member.alignment_bits);
            children.push(slot);
            elements.push(raw::debug_types::result_member(
                &self.debug,
                &self.unit,
                child.metadata,
                child.size,
                member.alignment_bits,
                member.offset_bits,
            ));
        }
        let tail = u64::from(tuple_alignment);
        if tuple_alignment != alignment_bits
            || end.checked_add(tail - 1).map(|size| size & !(tail - 1)) != Some(size_bits)
        {
            return Err(Error::InvalidDebugType);
        }
        let metadata = raw::debug_types::result_tuple(
            &self.debug,
            &self.unit,
            size_bits,
            alignment_bits,
            &mut elements,
        );
        drop(table);
        let tuple = self.add_type(metadata, size_bits, alignment_bits)?;
        self.types.borrow_mut()[tuple.slot].by_value = children;
        Ok(tuple)
    }
}
