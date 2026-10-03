//! Whole-value snapshots, checked record projections, and internal result carriers.
use super::*;
use inkwell::types::BasicType;
use jai_ir::{ConstantKind, ConstantValue, ProcedureId};

pub(super) fn constant<'ctx>(
    lowerer: &mut types::TypeLowerer<'ctx, '_>,
    value: &ConstantValue,
    context: &'ctx Context,
    module: &Module<'ctx>,
    functions: &HashMap<ProcedureId, FunctionValue<'ctx>>,
    signatures: &HashMap<ProcedureId, TypeId>,
) -> Result<BasicValueEnum<'ctx>, Error> {
    let ty = lowerer.basic(value.ty)?;
    let result: BasicValueEnum<'ctx> = match &value.kind {
        ConstantKind::NativePointer(value) => native_pointer_constants::constant(lowerer, value)?,
        ConstantKind::RuntimeType(value) => static_data::runtime_type_constant(
            lowerer, value, context, module, functions, signatures,
        )?,
        ConstantKind::Int(value) | ConstantKind::Enum(value) => {
            ty.into_int_type().const_int(value.bits(), false).into()
        }
        ConstantKind::Float(value) => floats::constant(lowerer.context(), *value)?.into(),
        ConstantKind::Bool(value) => ty
            .into_int_type()
            .const_int(u64::from(*value), false)
            .into(),
        ConstantKind::Distinct(value) => {
            constant(lowerer, value, context, module, functions, signatures)?
        }
        ConstantKind::Procedure(procedure) => {
            if signatures.get(procedure) != Some(&value.ty) {
                return Err(Error::Invariant);
            }
            functions
                .get(procedure)
                .ok_or(Error::Invariant)?
                .as_global_value()
                .as_pointer_value()
                .into()
        }
        ConstantKind::Zero => ty.const_zero(),
        ConstantKind::Array(elements) => {
            let values = elements
                .iter()
                .map(|element| constant(lowerer, element, context, module, functions, signatures))
                .collect::<Result<Vec<_>, _>>()?;
            let array = ty.into_array_type();
            if values
                .iter()
                .all(|value| value.get_type() == array.get_element_type())
            {
                sequences::array_constant(array.get_element_type(), &values)?
            } else {
                context
                    .struct_type(
                        &values
                            .iter()
                            .map(|value| value.get_type())
                            .collect::<Vec<_>>(),
                        false,
                    )
                    .const_named_struct(&values)
                    .into()
            }
        }
        ConstantKind::StringBytes(_) => {
            sequences::constant_string(lowerer, value, context, module)?
        }
        ConstantKind::Union {
            field,
            value: member,
        } => {
            if lowerer.registry().validate_field(value.ty, *field)? != member.ty {
                return Err(Error::Invariant);
            }
            let member = constant(lowerer, member, context, module, functions, signatures)?;
            let target = lowerer.target_data().ok_or(Error::Invariant)?;
            let size = target.get_abi_size(&ty);
            let used = target.get_abi_size(&member.get_type());
            let tail = context.i8_type().array_type(
                u32::try_from(size.checked_sub(used).ok_or(Error::Invariant)?)
                    .map_err(|_| Error::Invariant)?,
            );
            let carrier = ty.array_type(0);
            let payload = context.struct_type(&[member.get_type(), tail.into()], true);
            let structure = context.struct_type(&[carrier.into(), payload.into()], false);
            structure
                .const_named_struct(&[
                    carrier.const_zero().into(),
                    payload
                        .const_named_struct(&[member, tail.const_zero().into()])
                        .into(),
                ])
                .into()
        }
        ConstantKind::Record(fields) => {
            let fields = fields
                .iter()
                .map(|field| constant(lowerer, field, context, module, functions, signatures))
                .collect::<Result<Vec<_>, _>>()?;
            let named = ty.into_struct_type();
            let layout = lowerer.semantic_layout(value.ty)?;
            records::constant(
                context,
                lowerer.target_data().ok_or(Error::Invariant)?,
                named,
                &fields,
                &layout,
            )?
        }
    };
    if let Some(target) = lowerer.target_data() {
        let actual = result.get_type();
        if target.get_abi_size(&ty) != target.get_abi_size(&actual)
            || target.get_abi_alignment(&ty) != target.get_abi_alignment(&actual)
        {
            return Err(Error::Invariant);
        }
        match (&value.kind, ty, actual) {
            (
                ConstantKind::Record(_),
                inkwell::types::BasicTypeEnum::StructType(expected),
                inkwell::types::BasicTypeEnum::StructType(actual),
            ) => {
                for index in 0..expected.count_fields() {
                    if target.offset_of_element(&expected, index)
                        != target.offset_of_element(&actual, index)
                    {
                        return Err(Error::Invariant);
                    }
                }
            }
            (
                ConstantKind::Array(_),
                inkwell::types::BasicTypeEnum::ArrayType(expected),
                inkwell::types::BasicTypeEnum::StructType(actual),
            ) => {
                let stride = target.get_abi_size(&expected.get_element_type());
                for index in 0..actual.count_fields() {
                    if target.offset_of_element(&actual, index) != Some(u64::from(index) * stride) {
                        return Err(Error::Invariant);
                    }
                }
            }
            _ => {}
        }
    }
    Ok(result)
}

impl<'ctx> Generator<'ctx, '_, '_> {
    pub(super) fn value(&mut self, value: &ValueExpr) -> Result<BasicValueEnum<'ctx>, Error> {
        let result: BasicValueEnum<'ctx> = match value {
            ValueExpr::StorageBitcast {
                source,
                cast,
            } => self.reinterpret_storage(source, *cast)?,
            ValueExpr::Bind {
                bindings,
                body,
                ty,
            } => self.bind_values(bindings, body, *ty)?,
            ValueExpr::Bound {
                binding,
                ty,
            } => self.bound_value(*binding, *ty)?,
            ValueExpr::NativePointer(value) => {
                native_pointer_constants::constant(self.lowerer, value)?
            }
            ValueExpr::RuntimeType(value) => static_data::runtime_type_constant(
                self.lowerer,
                value,
                self.context,
                self.module,
                self.functions,
                self.signatures,
            )?,
            ValueExpr::TypeDescriptor {
                value,
                ty,
            } => {
                let schema = jai_types::RuntimeTypeSchema::from_view(self.types)?;
                if value.type_id(self.types) != schema.ty() || *ty != schema.descriptor_type() {
                    return Err(Error::Invariant);
                }
                self.value(value)?
            }
            ValueExpr::Context {
                ty,
            } => self.context_value(*ty)?,
            ValueExpr::StaticAddress {
                data,
                address,
                ty,
            } => static_data::address(self, data, address, *ty)?,
            ValueExpr::SequenceView {
                sequence,
                ty,
            } => self.sequence_view(sequence, *ty)?,
            ValueExpr::SequenceConcat {
                ty,
                parts,
            } => self.sequence_concat(*ty, parts)?,
            ValueExpr::Union {
                ty,
                field,
                value,
            } => {
                let expected = self.types.validate_field(*ty, *field)?;
                if value.type_id(self.types) != expected {
                    return Err(Error::Invariant);
                }
                let storage = self.lowerer.basic(*ty)?.into_struct_type();
                let member = self.value(value)?;
                unions::construct(self.context, &self.builder, storage, member)?.into()
            }
            ValueExpr::Distinct {
                value, ..
            }
            | ValueExpr::UnwrapDistinct {
                value, ..
            } => self.value(value)?,
            ValueExpr::ProcedureValue {
                procedure,
                ty,
            } => {
                if self.signatures.get(procedure) != Some(ty) {
                    return Err(Error::Invariant);
                }
                self.functions
                    .get(procedure)
                    .ok_or(Error::Invariant)?
                    .as_global_value()
                    .as_pointer_value()
                    .into()
            }
            ValueExpr::IndirectCall {
                callee,
                arguments,
                inline_hint,
                ..
            } => self
                .indirect_call(callee, arguments, *inline_hint)?
                .value
                .ok_or(Error::Invariant)?,
            ValueExpr::Conditional {
                ty,
                expression,
            } => self.conditional_value(*ty, expression)?,
            ValueExpr::AddressOfValue {
                value,
                ty,
            } => self.address_of_value(value, *ty)?,
            ValueExpr::AddressOf {
                place,
                ty,
            } => self.address_value(*place, *ty)?,
            ValueExpr::PointerCast {
                value,
                ty,
                mode,
            } => self.cast_pointer(value, *ty, *mode)?,
            ValueExpr::PointerFromInteger {
                value,
                ty,
                mode,
            } => self.integer_to_pointer(value, *ty, *mode)?,
            ValueExpr::PointerOffsetLeft {
                offset,
                pointer,
                ty,
            } => self.offset_pointer_left(offset, pointer, *ty)?,
            ValueExpr::PointerOffset {
                pointer,
                offset,
                subtract,
                ty,
            } => self.offset_pointer(pointer, offset, *subtract, *ty)?,
            ValueExpr::Index {
                base,
                index,
                ty,
                check,
            } => self.index_value(base, index, *ty, *check)?,
            ValueExpr::Int(value) => self.int(value)?.0.into(),
            ValueExpr::Float(value) => self.float(value)?.into(),
            ValueExpr::Bool(value) => self.boolean(value)?.0.into(),
            ValueExpr::Load(place) => self.load(*place)?,
            ValueExpr::Zero(ty) => self.lowerer.basic(*ty)?.const_zero(),
            ValueExpr::Array {
                ty,
                elements,
            } => self.sequence_array(*ty, elements)?,
            ValueExpr::StringBytes {
                ty,
                bytes,
            } => self.sequence_string(*ty, bytes)?,
            ValueExpr::SequenceField {
                base,
                field,
                ty,
            } => self.sequence_field(base, *field, *ty)?,
            ValueExpr::ArrayToSlice {
                array,
                ty,
            } => self.array_to_slice(*array, *ty)?,
            ValueExpr::ArrayView {
                array,
                ty,
            } => self.array_view(array, *ty)?,
            ValueExpr::SequenceBuild {
                ty,
                initializers,
            } => self.sequence_build(*ty, initializers)?,
            ValueExpr::Enum {
                ty,
                value,
            } => {
                if self.types.enum_definition(*ty)?.representation != value.ty() {
                    return Err(Error::Invariant);
                }
                integer_type(self.context, value.ty())
                    .const_int(value.bits(), false)
                    .into()
            }
            ValueExpr::EnumFromInt {
                ty,
                value,
            } => {
                if self.types.enum_definition(*ty)?.representation != value.ty() {
                    return Err(Error::Invariant);
                }
                self.int(value)?.0.into()
            }
            ValueExpr::Record {
                ty,
                fields,
            } => {
                if fields.len() != self.types.record_storage_definition(*ty)?.fields.len() {
                    return Err(Error::Invariant);
                }
                let initializers = fields
                    .iter()
                    .enumerate()
                    .map(|(index, value)| {
                        self.types.field(*ty, index).map(|field| (field.id, value))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                self.custom_record_build(*ty, initializers)?
            }
            ValueExpr::RecordBuild {
                ty,
                initializers,
            } => self.custom_record_build(
                *ty,
                initializers.iter().map(|(field, value)| (*field, value)),
            )?,
            ValueExpr::OrderedRecord {
                ty,
                backing,
                initializers,
            } => self.ordered_record_build(*ty, *backing, initializers)?,
            ValueExpr::Field {
                base,
                field,
                ty,
            } => {
                let base_ty = base.type_id(self.types);
                if self.types.validate_field(base_ty, *field)? != *ty {
                    return Err(Error::Invariant);
                }
                self.lowerer.basic(base_ty)?;
                let snapshot = self.value(base)?.into_struct_value();
                if self.types.record_storage_definition(base_ty)?.kind
                    == jai_types::RecordKind::Union
                {
                    let member = self.lowerer.basic(*ty)?;
                    unions::extract(self.context, &self.builder, snapshot, member)?
                } else {
                    self.custom_record_extract(base_ty, snapshot.into(), *field, *ty)?
                }
            }
            ValueExpr::Call {
                call, ..
            } => self.call(call)?.value.ok_or(Error::Invariant)?,
        };
        if result.get_type() != self.lowerer.basic(value.type_id(self.types))? {
            return Err(Error::Invariant);
        }
        Ok(result)
    }

    pub(super) fn store(&mut self, place: Place, value: &ValueExpr) -> Result<(), Error> {
        if place.ty() != value.type_id(self.types) {
            return Err(Error::Invariant);
        }
        let destination = self.slot(place)?;
        let snapshot = self.value(value)?;
        memory::store(
            &self.builder,
            destination.pointer,
            snapshot,
            destination.alignment,
        )
    }

    pub(super) fn return_values(
        &mut self,
        values: &[ValueExpr],
    ) -> Result<Option<BasicValueEnum<'ctx>>, Error> {
        // Evaluate every result before any deferred block can modify its source.
        let snapshots = values
            .iter()
            .map(|value| self.value(value))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(match snapshots.as_slice() {
            [] => None,
            [value] => Some(*value),
            fields => {
                let structure = self
                    .function
                    .get_type()
                    .get_return_type()
                    .ok_or(Error::Invariant)?
                    .into_struct_type();
                let mut carrier = structure.const_zero();
                for (index, value) in fields.iter().enumerate() {
                    carrier = self
                        .builder
                        .build_insert_value(
                            carrier,
                            *value,
                            u32::try_from(index).map_err(|_| Error::Invariant)?,
                            "result.field",
                        )?
                        .into_struct_value();
                }
                Some(carrier.into())
            }
        })
    }

    pub(super) fn call_results(
        &mut self,
        call: &Call,
        destinations: &[Option<Place>],
    ) -> Result<(), Error> {
        let destinations = self.capture_call_destinations(destinations)?;
        let result = self.call(call)?.value;
        self.store_call_results(result, &destinations)
    }

    pub(super) fn capture_call_destinations(
        &mut self,
        destinations: &[Option<Place>],
    ) -> Result<Vec<Option<Slot<'ctx>>>, Error> {
        destinations
            .iter()
            .map(|place| place.map(|place| self.slot(place)).transpose())
            .collect()
    }

    pub(super) fn store_call_results(
        &mut self,
        result: Option<BasicValueEnum<'ctx>>,
        destinations: &[Option<Slot<'ctx>>],
    ) -> Result<(), Error> {
        let snapshots = match destinations {
            [] => {
                if result.is_some() {
                    return Err(Error::Invariant);
                }
                vec![]
            }
            [_] => vec![result.ok_or(Error::Invariant)?],
            destinations => {
                let carrier = result.ok_or(Error::Invariant)?.into_struct_value();
                (0..destinations.len())
                    .map(|index| {
                        self.builder
                            .build_extract_value(
                                carrier,
                                u32::try_from(index).map_err(|_| Error::Invariant)?,
                                "result.field",
                            )
                            .map_err(Error::from)
                    })
                    .collect::<Result<Vec<_>, _>>()?
            }
        };
        for (slot, snapshot) in destinations.iter().zip(snapshots) {
            if let Some(slot) = slot {
                if snapshot.get_type() != self.lowerer.basic(slot.ty)? {
                    return Err(Error::Invariant);
                }
                memory::store(&self.builder, slot.pointer, snapshot, slot.alignment)?;
            }
        }
        Ok(())
    }
}
