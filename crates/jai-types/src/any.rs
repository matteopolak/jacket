//! Checked storage identities for the universal value descriptor.
use crate::{
    FieldDescriptor, Layout, LayoutEngine, LayoutError, LayoutPolicy, RecordKind, TypeError,
    TypeId, TypeKind, TypeView,
};
use std::fmt;

/// Source member names are resolved once to these typed field choices.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnyField {
    Type,
    ValuePointer,
}

/// Evidence that one universal type has the canonical two-pointer storage.
/// The descriptor header identity comes from the compilation's Type_Info schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnySchema {
    ty: TypeId,
    header: TypeId,
    descriptor: FieldDescriptor,
    payload: FieldDescriptor,
}

/// A source nominal record can expose the same bytes without becoming Any.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnyStorageBridge {
    universal: TypeId,
    record: TypeId,
    descriptor: FieldDescriptor,
    payload: FieldDescriptor,
}
impl AnyStorageBridge {
    pub fn universal_type(self) -> TypeId {
        self.universal
    }
    pub fn record_type(self) -> TypeId {
        self.record
    }
    pub fn field(self, field: AnyField) -> FieldDescriptor {
        match field {
            AnyField::Type => self.descriptor,
            AnyField::ValuePointer => self.payload,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnyConversion {
    /// An Any value already contains the original descriptor and payload pointers.
    Identity,
    /// Borrow a place, or materialize the value in its evaluating frame.
    Borrow { represented: TypeId },
}

#[derive(Debug)]
pub enum AnyError {
    Type(TypeError),
    Layout(LayoutError),
    NotUniversal(TypeId),
    InvalidStorage(TypeId),
    InvalidDescriptorHeader(TypeId),
    IncompatibleStorageMirror(TypeId),
    InvalidField { field: AnyField, actual: TypeId },
}
impl From<TypeError> for AnyError {
    fn from(error: TypeError) -> Self {
        Self::Type(error)
    }
}
impl From<LayoutError> for AnyError {
    fn from(error: LayoutError) -> Self {
        Self::Layout(error)
    }
}
impl fmt::Display for AnyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type(error) => error.fmt(f),
            Self::Layout(error) => error.fmt(f),
            Self::NotUniversal(_) => f.write_str("type is not the universal Any type"),
            Self::InvalidStorage(_) => {
                f.write_str("Any storage must be a struct with two pointer fields")
            }
            Self::InvalidDescriptorHeader(_) => {
                f.write_str("Any descriptor header must have a completed struct type")
            }
            Self::IncompatibleStorageMirror(_) => f.write_str(
                "source record does not share Any's exact pointer fields and target layout",
            ),
            Self::InvalidField { field, .. } => {
                write!(f, "Any {field:?} field has the wrong pointer type")
            }
        }
    }
}
impl std::error::Error for AnyError {}

impl AnySchema {
    pub fn validate(
        types: &dyn TypeView,
        ty: TypeId,
        type_info_header: TypeId,
    ) -> Result<Self, AnyError> {
        if !matches!(types.kind(ty)?, TypeKind::Any(_)) {
            return Err(AnyError::NotUniversal(ty));
        }
        let record = types.record_storage_definition(ty)?;
        if record.kind != RecordKind::Struct || record.fields.len() != 2 {
            return Err(AnyError::InvalidStorage(ty));
        }
        if types.record_definition(type_info_header)?.kind != RecordKind::Struct {
            return Err(AnyError::InvalidDescriptorHeader(type_info_header));
        }
        let descriptor = types.field(ty, 0)?;
        let payload = types.field(ty, 1)?;
        if *types.kind(descriptor.ty)? != TypeKind::Pointer(type_info_header) {
            return Err(AnyError::InvalidField {
                field: AnyField::Type,
                actual: descriptor.ty,
            });
        }
        let TypeKind::Pointer(pointee) = *types.kind(payload.ty)? else {
            return Err(AnyError::InvalidField {
                field: AnyField::ValuePointer,
                actual: payload.ty,
            });
        };
        if *types.kind(pointee)? != TypeKind::Void {
            return Err(AnyError::InvalidField {
                field: AnyField::ValuePointer,
                actual: payload.ty,
            });
        }
        Ok(Self {
            ty,
            header: type_info_header,
            descriptor,
            payload,
        })
    }

    pub fn ty(self) -> TypeId {
        self.ty
    }
    pub fn header(self) -> TypeId {
        self.header
    }
    pub fn field(self, field: AnyField) -> FieldDescriptor {
        match field {
            AnyField::Type => self.descriptor,
            AnyField::ValuePointer => self.payload,
        }
    }

    /// Confirm storage against the selected target, never Rust's host pointers.
    pub fn layout(self, types: &dyn TypeView, policy: LayoutPolicy) -> Result<Layout, AnyError> {
        let layout = LayoutEngine::new(types, policy).layout(self.ty)?.clone();
        let pointer = policy.pointer();
        if layout.size
            != pointer
                .size
                .checked_mul(2)
                .ok_or(LayoutError::Overflow(self.ty))?
            || layout.alignment != pointer.alignment
            || layout.field_offsets.as_ref() != [0, pointer.size]
        {
            return Err(AnyError::InvalidStorage(self.ty));
        }
        Ok(layout)
    }

    /// Ordinary casts out of Any use its value_pointer; this classifies only boxing.
    pub fn conversion(
        self,
        types: &dyn TypeView,
        source: TypeId,
        policy: LayoutPolicy,
    ) -> Result<AnyConversion, AnyError> {
        if Self::validate(types, self.ty, self.header)? != self {
            return Err(AnyError::InvalidStorage(self.ty));
        }
        types.kind(source)?;
        if source == self.ty {
            return Ok(AnyConversion::Identity);
        }
        if matches!(types.kind(source)?, TypeKind::Type) {
            let runtime = crate::RuntimeTypeSchema::from_view(types)?;
            if runtime.header_type() != self.header {
                return Err(AnyError::InvalidDescriptorHeader(runtime.header_type()));
            }
        }
        LayoutEngine::new(types, policy).layout(source)?;
        Ok(AnyConversion::Borrow {
            represented: source,
        })
    }

    /// Certify a designated source mirror such as Preload's Any_Struct. This is
    /// storage evidence only: it does not register a universal conversion rule.
    pub fn storage_bridge(
        self,
        types: &dyn TypeView,
        record_type: TypeId,
        policy: LayoutPolicy,
    ) -> Result<AnyStorageBridge, AnyError> {
        let record = types.record_definition(record_type)?;
        if record.kind != RecordKind::Struct
            || record.fields.as_ref() != [self.descriptor.ty, self.payload.ty]
        {
            return Err(AnyError::IncompatibleStorageMirror(record_type));
        }
        let universal_layout = self.layout(types, policy)?;
        if *LayoutEngine::new(types, policy).layout(record_type)? != universal_layout {
            return Err(AnyError::IncompatibleStorageMirror(record_type));
        }
        Ok(AnyStorageBridge {
            universal: self.ty,
            record: record_type,
            descriptor: types.field(record_type, 0)?,
            payload: types.field(record_type, 1)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IntegerType, ScalarLayout, ScalarType, TypeRegistry};

    fn fixture() -> (TypeRegistry, AnySchema) {
        let mut types = TypeRegistry::new();
        let header = types.reserve_record(RecordKind::Struct);
        types
            .define_record(
                header,
                [
                    types.scalar(ScalarType::Int(IntegerType::U32)),
                    types.scalar(ScalarType::Int(IntegerType::S64)),
                ],
            )
            .unwrap();
        let any = types.reserve_any();
        types.define_any(any, header).unwrap();
        let schema = AnySchema::validate(&types, any, header).unwrap();
        (types, schema)
    }

    #[test]
    fn source_storage_bridge_keeps_nominal_identity_and_exact_field_owners() {
        let (mut types, schema) = fixture();
        let mirror = types.reserve_record(RecordKind::Struct);
        types
            .define_record(
                mirror,
                [
                    schema.field(AnyField::Type).ty,
                    schema.field(AnyField::ValuePointer).ty,
                ],
            )
            .unwrap();
        let bridge = schema
            .storage_bridge(&types, mirror, LayoutPolicy::lp64())
            .unwrap();
        assert_eq!(bridge.universal_type(), schema.ty());
        assert_eq!(bridge.record_type(), mirror);
        assert_ne!(bridge.record_type(), bridge.universal_type());
        assert_eq!(
            types
                .validate_field(mirror, bridge.field(AnyField::Type).id)
                .unwrap(),
            schema.field(AnyField::Type).ty
        );
        assert!(
            types
                .validate_field(schema.ty(), bridge.field(AnyField::Type).id)
                .is_err()
        );
        assert!(
            matches!(schema.conversion(&types,mirror,LayoutPolicy::lp64()).unwrap(),AnyConversion::Borrow { represented } if represented==mirror)
        );
        let types = types.freeze().unwrap();
        assert_eq!(
            schema
                .storage_bridge(&types, mirror, LayoutPolicy::lp64())
                .unwrap(),
            bridge
        );
    }

    #[test]
    fn storage_bridge_rejects_other_pointer_types_union_and_different_offsets() {
        let (mut types, schema) = fixture();
        let fields = [
            schema.field(AnyField::Type).ty,
            schema.field(AnyField::ValuePointer).ty,
        ];
        let wrong = types.reserve_record(RecordKind::Struct);
        types.define_record(wrong, [fields[1], fields[0]]).unwrap();
        assert!(
            schema
                .storage_bridge(&types, wrong, LayoutPolicy::lp64())
                .is_err()
        );
        let union = types.reserve_record(RecordKind::Union);
        types.define_record(union, fields).unwrap();
        assert!(
            schema
                .storage_bridge(&types, union, LayoutPolicy::lp64())
                .is_err()
        );
        let aligned = types.reserve_record(RecordKind::Struct);
        types
            .define_record_with_layout(
                aligned,
                fields,
                crate::RecordLayout {
                    field_alignments: vec![None, Some(16)].into_boxed_slice(),
                    ..crate::RecordLayout::default()
                },
            )
            .unwrap();
        assert!(
            schema
                .storage_bridge(&types, aligned, LayoutPolicy::lp64())
                .is_err()
        );
    }

    #[test]
    fn identity_and_field_owners_survive_freezing() {
        let (mut types, schema) = fixture();
        assert_eq!(types.reserve_any(), schema.ty());
        assert!(matches!(types.kind(schema.ty()).unwrap(), TypeKind::Any(_)));
        assert!(types.record_definition(schema.ty()).is_err());
        let field = schema.field(AnyField::Type);
        assert_eq!(
            types.validate_field(schema.ty(), field.id).unwrap(),
            field.ty
        );
        let types = types.freeze().unwrap();
        assert_eq!(
            AnySchema::validate(&types, schema.ty(), schema.header()).unwrap(),
            schema
        );
        assert_eq!(types.field_type(field.id).unwrap(), field.ty);
    }

    #[test]
    fn storage_uses_selected_pointer_width_and_alignment() {
        let (types, schema) = fixture();
        let wide = schema.layout(&types, LayoutPolicy::lp64()).unwrap();
        assert_eq!((wide.size, wide.alignment), (16, 8));
        assert_eq!(wide.field_offsets.as_ref(), [0, 8]);
        let narrow = LayoutPolicy::new(
            ScalarLayout::new(4, 4),
            [
                ScalarLayout::new(1, 1),
                ScalarLayout::new(2, 2),
                ScalarLayout::new(4, 4),
                ScalarLayout::new(8, 4),
            ],
            [ScalarLayout::new(4, 4), ScalarLayout::new(8, 4)],
            ScalarLayout::new(1, 1),
        )
        .unwrap();
        let layout = schema.layout(&types, narrow).unwrap();
        assert_eq!((layout.size, layout.alignment), (8, 4));
        assert_eq!(layout.field_offsets.as_ref(), [0, 4]);
    }

    #[test]
    fn structurally_equal_records_do_not_acquire_universal_rules() {
        let (mut types, schema) = fixture();
        let imitation = types.reserve_record(RecordKind::Struct);
        types
            .define_record(
                imitation,
                [
                    schema.field(AnyField::Type).ty,
                    schema.field(AnyField::ValuePointer).ty,
                ],
            )
            .unwrap();
        assert!(matches!(
            AnySchema::validate(&types, imitation, schema.header()),
            Err(AnyError::NotUniversal(actual)) if actual == imitation
        ));
        assert!(
            types
                .validate_field(imitation, schema.field(AnyField::Type).id)
                .is_err()
        );
    }

    #[test]
    fn header_proof_requires_exact_nominal_identity() {
        let (mut types, schema) = fixture();
        let other_header = types.reserve_record(RecordKind::Struct);
        let fields = types
            .record_definition(schema.header())
            .unwrap()
            .fields
            .clone();
        types.define_record(other_header, fields).unwrap();
        assert!(matches!(
            AnySchema::validate(&types, schema.ty(), other_header),
            Err(AnyError::InvalidField {
                field: AnyField::Type,
                ..
            })
        ));
        let foreign = TypeRegistry::new();
        assert!(matches!(
            AnySchema::validate(&foreign, schema.ty(), schema.header()),
            Err(AnyError::Type(TypeError::ForeignType(_)))
        ));
    }

    #[test]
    fn conversion_rejects_a_schema_from_another_type_registry() {
        let (types, schema) = fixture();
        let (foreign, foreign_schema) = fixture();
        let check = |view: &dyn TypeView| {
            for source in [view.scalar(ScalarType::Bool), foreign_schema.ty()] {
                assert!(matches!(
                    schema.conversion(view, source, LayoutPolicy::lp64()),
                    Err(AnyError::Type(TypeError::ForeignType(id))) if id == schema.ty()
                ));
            }
        };
        check(&foreign);
        check(&foreign.freeze().unwrap());
        assert_eq!(
            schema
                .conversion(&types, schema.ty(), LayoutPolicy::lp64())
                .unwrap(),
            AnyConversion::Identity
        );
        let boolean = types.scalar(ScalarType::Bool);
        assert_eq!(
            schema
                .conversion(&types.freeze().unwrap(), boolean, LayoutPolicy::lp64())
                .unwrap(),
            AnyConversion::Borrow {
                represented: boolean
            }
        );
    }

    #[test]
    fn conversion_copies_any_and_borrows_only_sized_payloads() {
        let (mut types, schema) = fixture();
        assert_eq!(
            schema
                .conversion(&types, schema.ty(), LayoutPolicy::lp64())
                .unwrap(),
            AnyConversion::Identity
        );
        let scalar = types.scalar(ScalarType::Int(IntegerType::S32));
        assert_eq!(
            schema
                .conversion(&types, scalar, LayoutPolicy::lp64())
                .unwrap(),
            AnyConversion::Borrow {
                represented: scalar
            }
        );
        assert!(matches!(
            schema.conversion(&types, types.meta_type(), LayoutPolicy::lp64()),
            Err(AnyError::Type(TypeError::Incomplete(_)))
        ));
        types.bind_runtime_type_header(schema.header()).unwrap();
        assert_eq!(
            schema
                .conversion(&types, types.meta_type(), LayoutPolicy::lp64())
                .unwrap(),
            AnyConversion::Borrow {
                represented: types.meta_type(),
            }
        );
        assert!(matches!(
            schema.conversion(&types, types.code_type(), LayoutPolicy::lp64()),
            Err(AnyError::Layout(LayoutError::Unsized(_)))
        ));
        let pending = types.reserve_record(RecordKind::Struct);
        assert!(matches!(
            schema.conversion(&types, pending, LayoutPolicy::lp64()),
            Err(AnyError::Layout(LayoutError::Type(TypeError::Incomplete(
                _
            ))))
        ));
    }

    #[test]
    fn runtime_type_payload_uses_the_same_nominal_header_as_any() {
        let (mut types, schema) = fixture();
        let unrelated = types.reserve_record(RecordKind::Struct);
        types
            .define_record(
                unrelated,
                types
                    .record_definition(schema.header())
                    .unwrap()
                    .fields
                    .to_vec(),
            )
            .unwrap();
        types.bind_runtime_type_header(unrelated).unwrap();
        assert!(matches!(
            schema.conversion(&types, types.meta_type(), LayoutPolicy::lp64()),
            Err(AnyError::InvalidDescriptorHeader(header)) if header == unrelated
        ));
    }
}
