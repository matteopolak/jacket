use super::*;
use jai_types::RecordKind;

pub(crate) fn constant(types: &dyn TypeView, root: &ConstantValue) -> Result<(), IrError> {
    constant_with_closures(types, root, &mut static_closures::StaticClosures::default())
}

pub(super) fn constant_with_closures(
    types: &dyn TypeView,
    root: &ConstantValue,
    closures: &mut static_closures::StaticClosures,
) -> Result<(), IrError> {
    let mut pending = vec![(root, 0usize)];
    let mut nodes = 0usize;
    while let Some((value, depth)) = pending.pop() {
        nodes += 1;
        if depth >= MAX_CONSTANT_DEPTH || nodes > 1_048_576 {
            return Err(IrError::VerificationDepth);
        }
        storage::runtime_type(types, value.ty)?;
        let kind = types.kind(value.ty)?;
        match (&value.kind, kind) {
            (ConstantKind::NativePointer(pointer), TypeKind::Pointer(_)) => {
                same_type(value.ty, pointer.type_id())?;
                pointer
                    .validate(types)
                    .map_err(|_| IrError::InvalidConstant(value.ty))?;
            }
            (ConstantKind::RuntimeType(value), TypeKind::Type) => {
                closures.validate(value.data(), types)?;
                value.validate_identity(types)?;
            }
            (ConstantKind::Int(integer), TypeKind::Integer(ty)) if integer.ty() == *ty => {}
            (ConstantKind::Float(value), TypeKind::Float(ty)) if value.ty() == *ty => {}
            (ConstantKind::Bool(_), TypeKind::Bool) => {}
            (ConstantKind::Procedure(_), TypeKind::Procedure(_)) => {}
            (ConstantKind::StringBytes(_), TypeKind::String) => {}
            (ConstantKind::Enum(integer), TypeKind::Enum(id))
                if types.enumeration(*id)?.representation == integer.ty() => {}
            (ConstantKind::Record(fields), TypeKind::Record(_) | TypeKind::Any(_)) => {
                let record = types.record_storage_definition(value.ty)?;
                if record.kind != RecordKind::Struct {
                    return Err(IrError::InvalidConstant(value.ty));
                }
                arity("constant fields", record.fields.len(), fields.len())?;
                for (field, &ty) in fields.iter().zip(&record.fields) {
                    same_type(ty, field.ty)?;
                    pending.push((field, depth + 1));
                }
            }
            (
                ConstantKind::Union {
                    field,
                    value: payload,
                },
                TypeKind::Record(id),
            ) => {
                if types.record(*id)?.kind != RecordKind::Union {
                    return Err(IrError::InvalidConstant(value.ty));
                }
                same_type(types.validate_field(value.ty, *field)?, payload.ty)?;
                pending.push((payload, depth + 1));
            }
            (
                ConstantKind::Array(elements),
                TypeKind::FixedArray {
                    element,
                    count,
                },
            ) => {
                let count =
                    usize::try_from(*count).map_err(|_| IrError::InvalidConstant(value.ty))?;
                arity("constant array elements", count, elements.len())?;
                for value in elements {
                    same_type(*element, value.ty)?;
                    pending.push((value, depth + 1));
                }
            }
            (ConstantKind::Distinct(value), TypeKind::Distinct(id)) => {
                same_type(types.distinct(*id)?.representation, value.ty)?;
                pending.push((value, depth + 1));
            }
            (ConstantKind::Zero, _) => {}
            _ => return Err(IrError::InvalidConstant(value.ty)),
        }
    }
    Ok(())
}

/// Type-only constant construction cannot prove an executable procedure identity.
/// This bounded pass closes those leaves against the selected signature environment.
pub(super) fn constant_procedures(
    types: &dyn TypeView,
    root: &ConstantValue,
    signatures: &HashMap<ProcedureId, TypeId>,
) -> Result<(), IrError> {
    constant_procedures_with_closures(
        types,
        root,
        signatures,
        &mut static_closures::StaticClosures::default(),
    )
}

pub(super) fn constant_procedures_with_closures(
    types: &dyn TypeView,
    root: &ConstantValue,
    signatures: &HashMap<ProcedureId, TypeId>,
    closures: &mut static_closures::StaticClosures,
) -> Result<(), IrError> {
    let mut pending = vec![(root, 0usize)];
    let mut nodes = 1usize;
    while let Some((value, depth)) = pending.pop() {
        if depth >= MAX_CONSTANT_DEPTH {
            return Err(IrError::VerificationDepth);
        }
        let children: &[ConstantValue] = match &value.kind {
            ConstantKind::RuntimeType(value) => {
                closures.procedures(value.data(), types, signatures)?;
                value.validate_identity(types)?;
                &[]
            }
            ConstantKind::Procedure(procedure) => {
                let &signature = signatures
                    .get(procedure)
                    .ok_or_else(|| unknown("constant procedure", procedure.index()))?;
                same_type(signature, value.ty)?;
                types.procedure_definition(signature)?;
                &[]
            }
            ConstantKind::Record(children) | ConstantKind::Array(children) => children,
            ConstantKind::Union {
                value, ..
            }
            | ConstantKind::Distinct(value) => std::slice::from_ref(value.as_ref()),
            ConstantKind::Int(_)
            | ConstantKind::NativePointer(_)
            | ConstantKind::Float(_)
            | ConstantKind::Bool(_)
            | ConstantKind::StringBytes(_)
            | ConstantKind::Enum(_)
            | ConstantKind::Zero => &[],
        };
        nodes = nodes
            .checked_add(children.len())
            .filter(|count| *count <= 1_048_576)
            .ok_or(IrError::VerificationDepth)?;
        pending.extend(children.iter().map(|child| (child, depth + 1)));
    }
    Ok(())
}

impl Context<'_> {
    pub(super) fn value(&self, value: &ValueExpr) -> Result<TypeId, IrError> {
        let _depth = self.enter()?;
        let types = self.types;
        let ty = value.type_id(types);
        storage::runtime_type(types, ty)?;
        match value {
            ValueExpr::NativePointer(value) => value
                .validate(types)
                .map_err(|_| IrError::InvalidValue(value.type_id()))?,
            ValueExpr::StorageBitcast {
                source,
                cast,
            } => {
                cast.revalidate(types, cast.layout_policy())
                    .map_err(IrError::StorageBitcast)?;
                storage::runtime_type(types, cast.source_type())?;
                let source_type = match source {
                    StorageBitcastSource::Place(place) => self.place(*place)?,
                    StorageBitcastSource::Value(value) => self.value(value)?,
                };
                same_type(cast.source_type(), source_type)?;
            }
            ValueExpr::Bind {
                bindings,
                body,
                ..
            } => self.bound_scope(bindings, body, ty)?,
            ValueExpr::Bound {
                binding, ..
            } => self.bound_value(*binding, ty)?,
            ValueExpr::RuntimeType(value) => {
                self.static_closures.borrow_mut().procedures(
                    value.data(),
                    types,
                    self.signatures,
                )?;
                value.validate_identity(types)?;
            }
            ValueExpr::TypeDescriptor {
                value, ..
            } => {
                let schema = jai_types::RuntimeTypeSchema::from_view(types)?;
                same_type(schema.ty(), self.value(value)?)?;
                same_type(schema.descriptor_type(), ty)?;
            }
            ValueExpr::Context {
                ..
            } => same_type(self.context_type()?, ty)?,
            ValueExpr::Conditional {
                expression, ..
            } => {
                self.boolean(&expression.condition)?;
                same_type(ty, self.value(&expression.then_value)?)?;
                same_type(ty, self.value(&expression.else_value)?)?;
            }
            ValueExpr::Int(value) => self.integer(value)?,
            ValueExpr::Float(value) => self.float(value)?,
            ValueExpr::Bool(value) => self.boolean(value)?,
            ValueExpr::Array {
                ..
            }
            | ValueExpr::StringBytes {
                ..
            }
            | ValueExpr::SequenceField {
                ..
            }
            | ValueExpr::ArrayToSlice {
                ..
            }
            | ValueExpr::ArrayView {
                ..
            }
            | ValueExpr::Index {
                ..
            }
            | ValueExpr::SequenceView {
                ..
            }
            | ValueExpr::SequenceBuild {
                ..
            } => self.sequence_value(value, ty)?,
            ValueExpr::AddressOf {
                ..
            }
            | ValueExpr::AddressOfValue {
                ..
            }
            | ValueExpr::PointerCast {
                ..
            }
            | ValueExpr::PointerOffset {
                ..
            }
            | ValueExpr::PointerOffsetLeft {
                ..
            }
            | ValueExpr::PointerFromInteger {
                ..
            } => self.pointer_value(value, ty)?,
            // Concatenated backing belongs to the caller frame and is admitted
            // only as the actual Jai variadic parameter by call_arguments.
            ValueExpr::SequenceConcat {
                ..
            } => return Err(IrError::InvalidValue(ty)),
            ValueExpr::Distinct {
                value, ..
            } => {
                same_type(
                    types.distinct_definition(ty)?.representation,
                    self.value(value)?,
                )?;
            }
            ValueExpr::UnwrapDistinct {
                value, ..
            } => {
                let wrapped = self.value(value)?;
                same_type(types.distinct_definition(wrapped)?.representation, ty)?;
            }
            ValueExpr::ProcedureValue {
                procedure, ..
            } => {
                let &signature = self
                    .signatures
                    .get(procedure)
                    .ok_or_else(|| unknown("procedure value", procedure.index()))?;
                same_type(signature, ty)?;
                types.procedure_definition(ty)?;
            }
            ValueExpr::IndirectCall {
                callee,
                arguments,
                inline_hint,
                ..
            } => {
                let results = self.indirect_call(callee, arguments, *inline_hint)?;
                arity("indirect call results", 1, results.len())?;
                same_type(ty, results[0])?;
            }
            ValueExpr::StaticAddress {
                data,
                address,
                ..
            } => {
                let TypeKind::Pointer(pointee) = *types.kind(ty)? else {
                    return Err(IrError::InvalidValue(ty));
                };
                self.static_closures
                    .borrow_mut()
                    .procedures(data, types, self.signatures)?;
                same_type(pointee, data.address_type(address, types)?)?;
            }
            ValueExpr::Load(place) => {
                self.place(*place)?;
            }
            ValueExpr::Zero(_) => {}
            ValueExpr::Record {
                fields, ..
            } => {
                let record = types.record_storage_definition(ty)?;
                if record.kind != RecordKind::Struct {
                    return Err(IrError::InvalidValue(ty));
                }
                arity("record fields", record.fields.len(), fields.len())?;
                for (field, &expected) in fields.iter().zip(&record.fields) {
                    same_type(expected, self.value(field)?)?;
                }
            }
            ValueExpr::Union {
                field,
                value,
                ..
            } => {
                if types.record_definition(ty)?.kind != RecordKind::Union {
                    return Err(IrError::InvalidValue(ty));
                }
                same_type(types.validate_field(ty, *field)?, self.value(value)?)?;
            }
            ValueExpr::RecordBuild {
                initializers, ..
            } => {
                let record = types.record_storage_definition(ty)?;
                if record.kind != RecordKind::Struct {
                    return Err(IrError::InvalidValue(ty));
                }
                arity(
                    "record initializers",
                    record.fields.len(),
                    initializers.len(),
                )?;
                let mut fields = vec![false; record.fields.len()];
                for (field, value) in initializers {
                    let expected = types.validate_field(ty, *field)?;
                    if std::mem::replace(&mut fields[field.index()], true) {
                        return Err(IrError::DuplicateIdentity {
                            kind: "record field",
                            index: field.index(),
                        });
                    }
                    same_type(expected, self.value(value)?)?;
                }
            }
            ValueExpr::OrderedRecord {
                initializers, ..
            } => {
                if types.record_storage_definition(ty)?.kind != RecordKind::Struct {
                    return Err(IrError::InvalidValue(ty));
                }
                // Preflight the complete journal before checking any RHS. This
                // also checks each nominal owner before execution can allocate.
                let expected = initializers
                    .iter()
                    .map(|(path, _)| crate::ordered_record_path_type(types, ty, path))
                    .collect::<Result<Vec<_>, _>>()?;
                for ((_, value), expected) in initializers.iter().zip(expected) {
                    same_type(expected, self.value(value)?)?;
                }
            }
            ValueExpr::Field {
                base,
                field,
                ..
            } => {
                let base = self.value(base)?;
                same_type(types.validate_field(base, *field)?, ty)?;
            }
            ValueExpr::Call {
                call, ..
            } => self.single_call(call, ty)?,
            ValueExpr::Enum {
                value, ..
            } => {
                same_integer(types.enum_definition(ty)?.representation, value.ty())?;
            }
            ValueExpr::EnumFromInt {
                value, ..
            } => {
                self.integer(value)?;
                same_integer(types.enum_definition(ty)?.representation, value.ty())?;
            }
        }
        Ok(ty)
    }
    pub(super) fn integer(&self, expression: &IntExpr) -> Result<(), IrError> {
        let _depth = self.enter()?;
        let ty = expression.ty();
        let expected = self.types.scalar(ScalarType::Int(ty));
        match expression.kind() {
            IntExprKind::Value(value) => {
                same_type(expected, self.value(value)?)?;
            }
            IntExprKind::EnumValue(value) => {
                let value_ty = self.value(value)?;
                same_integer(ty, self.types.enum_definition(value_ty)?.representation)?;
            }
            IntExprKind::PointerDifference {
                left,
                right,
            } => {
                same_integer(IntegerType::S64, ty)?;
                self.pointer_compare(left, right)?;
                let pointer = left.type_id(self.types);
                let TypeKind::Pointer(element) = *self.types.kind(pointer)? else {
                    return Err(IrError::InvalidValue(pointer));
                };
                if !storage::nonzero_type(self.types, element)? {
                    return Err(IrError::InvalidValue(element));
                }
            }
            IntExprKind::Constant(value) => same_integer(ty, value.ty())?,
            IntExprKind::InvalidCheckedCast => {}
            IntExprKind::FromBool(value) => self.boolean(value)?,
            IntExprKind::FromFloat(mode, value) => {
                if *mode != CastMode::Checked {
                    return Err(IrError::InvalidValue(expected));
                }
                self.float(value)?;
            }
            IntExprKind::FromPointer {
                value,
                mode,
            } => {
                if matches!(mode, CastMode::Force(_)) {
                    return Err(IrError::InvalidValue(expected));
                }
                let value = self.value(value)?;
                if !matches!(self.types.kind(value)?, TypeKind::Pointer(_)) {
                    return Err(IrError::InvalidValue(value));
                }
            }
            IntExprKind::Cast(mode, value) => {
                if matches!(mode, CastMode::Force(_)) {
                    return Err(IrError::InvalidValue(expected));
                }
                self.integer(value)?;
            }
            IntExprKind::Load(place) => {
                self.integer_place(*place)?;
                same_integer(ty, place.ty())?;
            }
            IntExprKind::Call(call) => self.single_call(call, expected)?,
            IntExprKind::Negate(value) | IntExprKind::Complement(value) => {
                self.integer(value)?;
                same_integer(ty, value.ty())?;
            }
            IntExprKind::Binary(_, left, right) => {
                self.integer(left)?;
                self.integer(right)?;
                same_integer(ty, left.ty())?;
                same_integer(ty, right.ty())?;
            }
            IntExprKind::Conditional(conditional) => {
                self.boolean(&conditional.condition)?;
                self.integer(&conditional.then_value)?;
                self.integer(&conditional.else_value)?;
                same_integer(ty, conditional.then_value.ty())?;
                same_integer(ty, conditional.else_value.ty())?;
            }
        }
        Ok(())
    }
    pub(super) fn boolean(&self, expression: &BoolExpr) -> Result<(), IrError> {
        let _depth = self.enter()?;
        match expression {
            BoolExpr::Value(value) => {
                same_type(self.types.scalar(ScalarType::Bool), self.value(value)?)?;
            }
            BoolExpr::CompileTime | BoolExpr::Constant(_) => {}
            BoolExpr::FromInt(value) => self.integer(value)?,
            BoolExpr::FromPointer(value) => self.pointer_truth(value)?,
            BoolExpr::Load(place) => self.boolean_place(*place)?,
            BoolExpr::Call(call) => self.single_call(call, self.types.scalar(ScalarType::Bool))?,
            BoolExpr::Not(value) => self.boolean(value)?,
            BoolExpr::CompareInts(_, left, right) => {
                self.integer(left)?;
                self.integer(right)?;
                same_integer(left.ty(), right.ty())?;
            }
            BoolExpr::CompareFloats(_, left, right) => {
                self.float(left)?;
                self.float(right)?;
                same_type(left.type_id(self.types), right.type_id(self.types))?;
            }
            BoolExpr::ComparePointers(_, left, right) => self.pointer_compare(left, right)?,
            BoolExpr::CompareStrings(_, left, right) => {
                let left = self.value(left)?;
                let right = self.value(right)?;
                if !matches!(self.types.kind(left)?, TypeKind::String) {
                    return Err(IrError::InvalidValue(left));
                }
                same_type(left, right)?;
            }
            BoolExpr::CompareBools(_, left, right)
            | BoolExpr::And(left, right)
            | BoolExpr::Or(left, right) => {
                self.boolean(left)?;
                self.boolean(right)?;
            }
            BoolExpr::Conditional(conditional) => {
                self.boolean(&conditional.condition)?;
                self.boolean(&conditional.then_value)?;
                self.boolean(&conditional.else_value)?;
            }
        }
        Ok(())
    }
}
