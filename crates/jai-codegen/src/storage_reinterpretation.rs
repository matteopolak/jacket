//! Selected-target scratch storage for sealed explicit byte reinterpretation.
use crate::*;
use inkwell::types::{AsTypeRef, BasicTypeEnum};
use jai_ir::StorageBitcastSource;
use jai_types::StorageBitcast;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageValueRole {
    SourceValue,
    DestinationValue,
}
impl std::fmt::Display for StorageValueRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::SourceValue => "source value",
            Self::DestinationValue => "destination value",
        })
    }
}

/// Inspect the actual LLVM representation, including nested implicit gaps.
/// Explicit byte padding remains a value; implicit LLVM struct padding does not.
fn complete_value_storage(data: &inkwell::targets::TargetData, root: BasicTypeEnum<'_>) -> bool {
    let mut known = HashMap::new();
    let mut pending = vec![(root, false)];
    while let Some((ty, expanded)) = pending.pop() {
        let key = ty.as_type_ref();
        if known.contains_key(&key) {
            continue;
        }
        if data.get_abi_size(&ty) == 0 {
            known.insert(key, true);
            continue;
        }
        let children = match ty {
            BasicTypeEnum::StructType(record) => record.get_field_types(),
            BasicTypeEnum::ArrayType(array) => vec![array.get_element_type()],
            _ => Vec::new(),
        };
        if !expanded {
            pending.push((ty, true));
            pending.extend(children.into_iter().map(|child| (child, false)));
            continue;
        }
        let mut complete = children
            .iter()
            .all(|child| known.get(&child.as_type_ref()) == Some(&true));
        if let BasicTypeEnum::StructType(record) = ty {
            let mut end = 0u64;
            for (index, field) in children.into_iter().enumerate() {
                let Ok(index) = u32::try_from(index) else {
                    complete = false;
                    break;
                };
                if data.offset_of_element(&record, index) != Some(end) {
                    complete = false;
                }
                let Some(next) = end.checked_add(data.get_abi_size(&field)) else {
                    complete = false;
                    break;
                };
                end = next;
            }
            complete &= end == data.get_abi_size(&ty);
        }
        known.insert(key, complete);
    }
    known.get(&root.as_type_ref()) == Some(&true)
}

impl<'ctx> Generator<'ctx, '_, '_> {
    pub(super) fn reinterpret_storage(
        &mut self,
        expression: &StorageBitcastSource,
        cast: StorageBitcast,
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        // The immutable IR proves type ownership. Recheck the selected target
        // before evaluating effects or issuing a storage access.
        cast.revalidate(
            self.types,
            types::layout_policy(self.context, &self.target.data)?,
        )
        .map_err(Error::StorageBitcast)?;
        let semantic_source = match expression {
            StorageBitcastSource::Place(place) => place.ty(),
            StorageBitcastSource::Value(value) => value.type_id(self.types),
        };
        if semantic_source != cast.source_type() {
            return Err(Error::Invariant);
        }
        let source = self.lowerer.basic(cast.source_type())?;
        let destination = self.lowerer.basic(cast.target_type())?;
        if self.target.data.get_abi_size(&source) != cast.source_size()
            || self.target.data.get_abi_size(&destination) != cast.target_size()
        {
            return Err(Error::Invariant);
        }
        if !complete_value_storage(&self.target.data, destination) {
            return Err(Error::UnsupportedStorageBitcast {
                procedure: self.procedure,
                ty: cast.target_type(),
                role: StorageValueRole::DestinationValue,
            });
        }
        if matches!(expression, StorageBitcastSource::Value(_))
            && !complete_value_storage(&self.target.data, source)
        {
            return Err(Error::UnsupportedStorageBitcast {
                procedure: self.procedure,
                ty: cast.source_type(),
                role: StorageValueRole::SourceValue,
            });
        }
        // The source type owns the complete allocation even for a smaller
        // destination prefix. Opaque LLVM pointers retain pointer-valued stores.
        let scratch =
            unions::entry_alloca(self.context, &self.builder, source, "storage.cast.snapshot")?;
        scratch
            .as_instruction_value()
            .ok_or(Error::Invariant)?
            .set_alignment(cast.scratch_alignment())
            .map_err(|_| Error::Invariant)?;
        match expression {
            StorageBitcastSource::Place(place) => {
                let slot = self.slot(*place)?;
                let size_type = self.context.ptr_sized_int_type(&self.target.data, None);
                let bits = size_type.get_bit_width();
                if bits < 64 && cast.source_size() >= (1u64 << bits) {
                    return Err(Error::Invariant);
                }
                // Resolve the source address once. Byte copying preserves a
                // prefix without first loading an uninitialized typed tail.
                self.builder.build_memcpy(
                    scratch,
                    cast.scratch_alignment(),
                    slot.pointer,
                    slot.alignment,
                    size_type.const_int(cast.source_size(), false),
                )?;
            }
            StorageBitcastSource::Value(expression) => {
                let value = self.value(expression)?;
                if value.get_type() != source {
                    return Err(Error::Invariant);
                }
                memory::store(&self.builder, scratch, value, cast.scratch_alignment())?;
            }
        }
        if cast.target_size() == 0 {
            return Ok(destination.const_zero());
        }
        self.validate_storage_bools(scratch, cast.target_type())?;
        memory::load(
            &self.builder,
            destination,
            scratch,
            "storage.cast.value",
            cast.scratch_alignment(),
        )
    }
}

/// Unions retain opaque bytes: selecting a member is a separate typed read.
fn bool_children(types: &Types, ty: TypeId) -> Result<Vec<TypeId>, Error> {
    Ok(match types.kind(ty)? {
        jai_types::TypeKind::FixedArray { element, count } if *count != 0 => vec![*element],
        jai_types::TypeKind::Distinct(_) => vec![types.distinct_definition(ty)?.representation],
        jai_types::TypeKind::Record(_) => {
            let record = types.record_definition(ty)?;
            if record.kind == jai_types::RecordKind::Struct
                && record.layout.field_placements.iter().all(Option::is_none)
            {
                record.fields.to_vec()
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    })
}

fn bool_storage(types: &Types, root: TypeId) -> Result<HashMap<TypeId, bool>, Error> {
    let mut known = HashMap::new();
    let mut pending = vec![(root, false)];
    while let Some((ty, expanded)) = pending.pop() {
        if known.contains_key(&ty) {
            continue;
        }
        let children = bool_children(types, ty)?;
        if expanded {
            let contains = matches!(types.kind(ty)?, jai_types::TypeKind::Bool)
                || children.iter().any(|child| known.get(child) == Some(&true));
            known.insert(ty, contains);
        } else {
            pending.push((ty, true));
            pending.extend(children.into_iter().map(|child| (child, false)));
        }
    }
    Ok(known)
}

enum BoolCheck<'ctx> {
    Value(TypeId, PointerValue<'ctx>),
    EndArray {
        index: IntValue<'ctx>,
        phi: inkwell::values::PhiValue<'ctx>,
        header: BasicBlock<'ctx>,
        end: BasicBlock<'ctx>,
    },
}
impl<'ctx> Generator<'ctx, '_, '_> {
    fn validate_storage_bools(
        &mut self,
        pointer: PointerValue<'ctx>,
        ty: TypeId,
    ) -> Result<(), Error> {
        let known = bool_storage(self.types, ty)?;
        let mut pending = vec![BoolCheck::Value(ty, pointer)];
        while let Some(check) = pending.pop() {
            match check {
                BoolCheck::EndArray {
                    index,
                    phi,
                    header,
                    end,
                } => {
                    let next = self.builder.build_int_add(
                        index,
                        index.get_type().const_int(1, false),
                        "storage.bool.next",
                    )?;
                    let latch = self.builder.get_insert_block().ok_or(Error::Invariant)?;
                    self.builder.build_unconditional_branch(header)?;
                    phi.add_incoming(&[(&next, latch)]);
                    self.builder.position_at_end(end);
                }
                BoolCheck::Value(ty, pointer) => {
                    if known.get(&ty) != Some(&true) {
                        continue;
                    }
                    match self.types.kind(ty)?.clone() {
                        jai_types::TypeKind::Bool => {
                            let byte = memory::load(
                                &self.builder,
                                self.context.i8_type().into(),
                                pointer,
                                "storage.bool.byte",
                                1,
                            )?
                            .into_int_value();
                            let valid = self.builder.build_int_compare(
                                IntPredicate::ULE,
                                byte,
                                byte.get_type().const_int(1, false),
                                "storage.bool.valid",
                            )?;
                            self.check_cast(Bit(valid))?;
                        }
                        jai_types::TypeKind::Distinct(_) => pending.push(BoolCheck::Value(
                            self.types.distinct_definition(ty)?.representation,
                            pointer,
                        )),
                        jai_types::TypeKind::Record(_) => {
                            let fields = self.types.record_definition(ty)?.fields.to_vec();
                            let layout = self.lowerer.semantic_layout(ty)?;
                            for (field, offset) in
                                fields.into_iter().zip(layout.field_offsets).rev()
                            {
                                if known.get(&field) == Some(&true) {
                                    let address = jai_llvm::gep(
                                        &self.builder,
                                        self.context.i8_type().into(),
                                        pointer,
                                        &[self.context.i64_type().const_int(offset, false)],
                                        "storage.bool.field",
                                    )?;
                                    pending.push(BoolCheck::Value(field, address));
                                }
                            }
                        }
                        jai_types::TypeKind::FixedArray { element, count } => {
                            let preheader =
                                self.builder.get_insert_block().ok_or(Error::Invariant)?;
                            let header = self.label("storage.bool.header");
                            let body = self.label("storage.bool.body");
                            let end = self.label("storage.bool.end");
                            self.builder.build_unconditional_branch(header)?;
                            self.builder.position_at_end(header);
                            let index_type =
                                self.context.ptr_sized_int_type(&self.target.data, None);
                            let phi = self.builder.build_phi(index_type, "storage.bool.index")?;
                            let zero = index_type.const_zero();
                            phi.add_incoming(&[(&zero, preheader)]);
                            let index = phi.as_basic_value().into_int_value();
                            let within = self.builder.build_int_compare(
                                IntPredicate::ULT,
                                index,
                                index_type.const_int(count, false),
                                "storage.bool.within",
                            )?;
                            self.builder.build_conditional_branch(within, body, end)?;
                            self.builder.position_at_end(body);
                            let address = jai_llvm::gep(
                                &self.builder,
                                self.lowerer.basic(element)?,
                                pointer,
                                &[index],
                                "storage.bool.element",
                            )?;
                            pending.push(BoolCheck::EndArray {
                                index,
                                phi,
                                header,
                                end,
                            });
                            pending.push(BoolCheck::Value(element, address));
                        }
                        _ => return Err(Error::Invariant),
                    }
                }
            }
        }
        Ok(())
    }
}
