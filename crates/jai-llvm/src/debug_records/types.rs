//! Session-branded type slots remain valid when recursive placeholders are replaced.
use super::*;
use inkwell::debug_info::DIFile;
use inkwell::llvm_sys::prelude::LLVMMetadataRef;
use std::cell::RefCell;

#[derive(Clone, Copy, Debug)]
pub struct DebugType {
    owner: NonZeroU64,
    pub(super) slot: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebugRecordKind {
    Struct,
    Union,
}
#[derive(Clone, Copy)]
pub struct DebugMember<'a> {
    pub name: &'a str,
    pub source: DebugSource<'a>,
    pub ty: DebugType,
    pub offset_bits: u64,
    pub alignment_bits: u32,
}
pub(super) struct TypeSlot<'ctx> {
    pub(super) metadata: LLVMMetadataRef,
    pub(super) size: u64,
    pub(super) alignment: u32,
    pub(super) pending: Option<RecordSeed<'ctx>>,
    pub(super) storage: bool,
    // Pointers terminate this graph: only storage contained by value is followed.
    pub(super) by_value: Vec<usize>,
}
pub(crate) struct RecordSeed<'ctx> {
    pub(crate) name: String,
    pub(crate) file: DIFile<'ctx>,
    pub(crate) line: u32,
    pub(crate) kind: DebugRecordKind,
    pub(crate) identifier: String,
}
pub(super) type TypeTable<'ctx> = RefCell<Vec<TypeSlot<'ctx>>>;

impl<'ctx> DebugSession<'_, 'ctx> {
    pub(super) fn type_slot(&self, ty: DebugType) -> Result<usize, Error> {
        if ty.owner != self.owner {
            return Err(Error::DebugOwnership);
        }
        if ty.slot >= self.types.borrow().len() {
            return Err(Error::InvalidDebugType);
        }
        Ok(ty.slot)
    }
    pub(super) fn add_type(
        &self,
        metadata: LLVMMetadataRef,
        size: u64,
        alignment: u32,
    ) -> Result<DebugType, Error> {
        if metadata.is_null() {
            return Err(Error::NullResult);
        }
        let mut table = self.types.borrow_mut();
        let slot = table.len();
        table.push(TypeSlot {
            metadata,
            size,
            alignment,
            pending: None,
            storage: true,
            by_value: Vec::new(),
        });
        Ok(DebugType {
            owner: self.owner,
            slot,
        })
    }
    pub fn primitive_type(&self, primitive: DebugPrimitive) -> Result<DebugType, Error> {
        let (name, size, encoding) = primitive.descriptor();
        let ty = self
            .debug
            .create_basic_type(name, size, encoding, DIFlags::ZERO)
            .map_err(|_| Error::NullResult)?;
        self.add_type(
            ty.as_mut_ptr(),
            size,
            u32::try_from(size).map_err(|_| Error::InvalidDebugType)?,
        )
    }
    pub fn pointer_type(
        &self,
        pointee: Option<DebugType>,
        size_bits: u64,
        alignment_bits: u32,
    ) -> Result<DebugType, Error> {
        valid_layout(size_bits, alignment_bits)?;
        if size_bits == 0 {
            return Err(Error::InvalidDebugType);
        }
        let metadata = match pointee {
            Some(ty) => {
                let slot = self.type_slot(ty)?;
                self.types.borrow()[slot].metadata
            }
            None => std::ptr::null_mut(),
        };
        let pointer = raw::debug_types::pointer(&self.debug, metadata, size_bits, alignment_bits);
        self.add_type(pointer, size_bits, alignment_bits)
    }
    pub fn array_type(
        &self,
        element: DebugType,
        count: u64,
        size_bits: u64,
        alignment_bits: u32,
    ) -> Result<DebugType, Error> {
        valid_layout(size_bits, alignment_bits)?;
        let count = i64::try_from(count).map_err(|_| Error::InvalidDebugType)?;
        let slot = self.type_slot(element)?;
        let table = self.types.borrow();
        let child = &table[slot];
        if !child.storage {
            return Err(Error::InvalidDebugType);
        }
        if child
            .size
            .checked_mul(count as u64)
            .is_none_or(|size| size > size_bits)
        {
            return Err(Error::InvalidDebugType);
        }
        let metadata = raw::debug_types::array(
            &self.debug,
            child.metadata,
            count,
            size_bits,
            alignment_bits,
        );
        drop(table);
        let array = self.add_type(metadata, size_bits, alignment_bits)?;
        self.types.borrow_mut()[array.slot].by_value.push(slot);
        Ok(array)
    }
    pub fn begin_record(
        &self,
        source: DebugSource<'_>,
        name: Option<&str>,
        kind: DebugRecordKind,
        size_bits: u64,
        alignment_bits: u32,
    ) -> Result<DebugType, Error> {
        valid_layout(size_bits, alignment_bits)?;
        let file = self.debug.create_file(source.file, source.directory);
        let mut table = self.types.borrow_mut();
        let slot = table.len();
        let seed = RecordSeed {
            name: name.unwrap_or("").into(),
            file,
            line: source.line.get(),
            kind,
            identifier: format!("jai.debug.{}.{}", self.owner, slot),
        };
        let metadata = raw::debug_types::record(
            &self.debug,
            &self.unit,
            &seed,
            size_bits,
            alignment_bits,
            None,
        );
        if metadata.is_null() {
            return Err(Error::NullResult);
        }
        table.push(TypeSlot {
            metadata,
            size: size_bits,
            alignment: alignment_bits,
            pending: Some(seed),
            storage: true,
            by_value: Vec::new(),
        });
        Ok(DebugType {
            owner: self.owner,
            slot,
        })
    }
    pub fn finish_record(
        &self,
        record: DebugType,
        members: &[DebugMember<'_>],
    ) -> Result<(), Error> {
        let slot = self.type_slot(record)?;
        u32::try_from(members.len()).map_err(|_| Error::InvalidDebugType)?;
        let table = self.types.borrow();
        let target = &table[slot];
        let seed = target.pending.as_ref().ok_or(Error::InvalidDebugType)?;
        let children = members
            .iter()
            .map(|member| self.type_slot(member.ty))
            .collect::<Result<Vec<_>, _>>()?;
        let mut stack = children.clone();
        let mut seen = std::collections::HashSet::new();
        while let Some(child) = stack.pop() {
            if child == slot {
                return Err(Error::InvalidDebugType);
            }
            if seen.insert(child) {
                stack.extend(table[child].by_value.iter().copied());
            }
        }
        let mut elements = Vec::with_capacity(members.len());
        for member in members {
            let child_slot = self.type_slot(member.ty)?;
            let child = &table[child_slot];
            if !child.storage {
                return Err(Error::InvalidDebugType);
            }
            valid_alignment(member.alignment_bits)?;
            if member.alignment_bits > target.alignment
                || !member
                    .offset_bits
                    .is_multiple_of(u64::from(member.alignment_bits))
                || member
                    .offset_bits
                    .checked_add(child.size)
                    .is_none_or(|end| end > target.size)
                || (seed.kind == DebugRecordKind::Union && member.offset_bits != 0)
            {
                return Err(Error::InvalidDebugType);
            }
            let file = self
                .debug
                .create_file(member.source.file, member.source.directory);
            let metadata = raw::debug_types::member(
                &self.debug,
                target.metadata,
                member.name,
                file,
                member.source.line.get(),
                child.metadata,
                child.size,
                member.alignment_bits,
                member.offset_bits,
            );
            if metadata.is_null() {
                return Err(Error::NullResult);
            }
            elements.push(metadata);
        }
        let metadata = raw::debug_types::record(
            &self.debug,
            &self.unit,
            seed,
            target.size,
            target.alignment,
            Some(&mut elements),
        );
        if metadata.is_null() {
            return Err(Error::NullResult);
        }
        raw::debug_types::replace(target.metadata, metadata);
        drop(table);
        let mut table = self.types.borrow_mut();
        table[slot].metadata = metadata;
        table[slot].pending = None;
        table[slot].by_value = children;
        Ok(())
    }
    pub(super) fn finish_types(&self) {
        let mut table = self.types.borrow_mut();
        for slot in table.iter_mut() {
            if let Some(seed) = slot.pending.take() {
                let metadata = raw::debug_types::forward(
                    &self.debug,
                    &self.unit,
                    &seed,
                    slot.size,
                    slot.alignment,
                );
                raw::debug_types::replace(slot.metadata, metadata);
                slot.metadata = metadata;
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn typed_variable(
        &self,
        scope: DebugScope<'ctx>,
        source: DebugSource<'_>,
        name: &str,
        kind: DebugVariableKind,
        ty: DebugType,
        alignment_bytes: u32,
    ) -> Result<DebugVariable<'ctx>, Error> {
        self.source(source)?;
        if let DebugVariableKind::Parameter(ordinal) = kind {
            // DILocalVariable stores Arg in 16 bits and asserts before truncation.
            u16::try_from(ordinal.get()).map_err(|_| Error::InvalidDebugParameter)?;
        }
        self.scope(scope)?;
        let slot = self.type_slot(ty)?;
        if !self.types.borrow()[slot].storage {
            return Err(Error::InvalidDebugType);
        }
        let alignment = alignment_bytes
            .checked_mul(8)
            .ok_or(Error::InvalidDebugType)?;
        if alignment != 0 && !alignment.is_power_of_two() {
            return Err(Error::InvalidDebugType);
        }
        let file = self.debug.create_file(source.file, source.directory);
        let variable = raw::debug_types::variable(
            &self.debug,
            scope.metadata,
            file,
            source.line.get(),
            name,
            kind,
            self.types.borrow()[slot].metadata,
            alignment,
        );
        if variable.is_null() {
            return Err(Error::NullResult);
        }
        Ok(DebugVariable {
            metadata: variable,
            location: self.location(scope, source),
            owner: self.owner,
            function: scope.function,
        })
    }
}
pub(super) fn valid_layout(size: u64, alignment: u32) -> Result<(), Error> {
    valid_alignment(alignment)?;
    if !size.is_multiple_of(8) || !size.is_multiple_of(u64::from(alignment)) {
        return Err(Error::InvalidDebugType);
    }
    Ok(())
}
pub(super) fn valid_alignment(alignment: u32) -> Result<(), Error> {
    if alignment < 8 || !alignment.is_power_of_two() {
        return Err(Error::InvalidDebugType);
    }
    Ok(())
}
impl DebugPrimitive {
    pub(super) fn descriptor(self) -> (&'static str, u64, u32) {
        match self {
            Self::Bool => ("bool", 8, 0x02),
            Self::Signed8 => ("s8", 8, 0x05),
            Self::Signed16 => ("s16", 16, 0x05),
            Self::Signed32 => ("s32", 32, 0x05),
            Self::Signed64 => ("s64", 64, 0x05),
            Self::Unsigned8 => ("u8", 8, 0x07),
            Self::Unsigned16 => ("u16", 16, 0x07),
            Self::Unsigned32 => ("u32", 32, 0x07),
            Self::Unsigned64 => ("u64", 64, 0x07),
            Self::Float32 => ("float32", 32, 0x04),
            Self::Float64 => ("float64", 64, 0x04),
        }
    }
}
