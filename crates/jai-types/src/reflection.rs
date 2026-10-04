//! Immutable descriptions of nominal types using the selected target's layout.
use crate::{
    CallingConvention, ContextMode, DistinctKind, FieldId, FloatType, Integer, IntegerType, Layout,
    LayoutEngine, LayoutError, LayoutPolicy, RecordKind, RecordMemberReflection,
    RecordReflectionFlag, TypeError, TypeId, TypeKind, TypeView, Variadic,
};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// A descriptor identity represents a type, rather than an address in the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DescriptorId(TypeId);
impl DescriptorId {
    pub fn represented_type(self) -> TypeId {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReflectionDependency {
    TargetLayout,
    Definition(TypeId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReflectionReadiness<T> {
    Ready(T),
    Pending(Box<[ReflectionDependency]>),
}

/// Optional source names are kept outside the canonical type registry. Absence
/// denotes an anonymous, programmatically constructed type, not an empty name.
#[derive(Clone, Debug, Default)]
pub struct ReflectionMetadata {
    names: HashMap<TypeId, Box<[u8]>>,
    fields: HashMap<FieldId, ReflectedFieldMetadata>,
    enumerations: HashMap<TypeId, ReflectedEnumMetadata>,
    field_notes: HashMap<FieldId, Box<[Box<[u8]>]>>,
    records: HashMap<TypeId, ReflectedRecordMetadata>,
    type_constants: HashMap<TypeId, Box<[ReflectedTypeConstant]>>,
}
impl ReflectionMetadata {
    pub fn name(&mut self, ty: TypeId, name: impl Into<Box<[u8]>>) {
        self.names.insert(ty, name.into());
    }
    pub fn field(&mut self, field: FieldId, metadata: ReflectedFieldMetadata) {
        self.fields.insert(field, metadata);
    }
    pub fn enumeration(&mut self, ty: TypeId, metadata: ReflectedEnumMetadata) {
        self.enumerations.insert(ty, metadata);
    }
    pub fn field_notes(&mut self, field: FieldId, notes: impl Into<Box<[Box<[u8]>]>>) {
        self.field_notes.insert(field, notes.into());
    }
    pub fn record(&mut self, ty: TypeId, metadata: ReflectedRecordMetadata) {
        self.records.insert(ty, metadata);
    }
    pub fn record_type_constants(
        &mut self,
        owner: TypeId,
        constants: impl Into<Box<[ReflectedTypeConstant]>>,
    ) {
        self.type_constants.insert(owner, constants.into());
        if let Some(metadata) = self.records.get_mut(&owner) {
            metadata.unsupported_members = false;
        }
    }
}

/// Checked nominal namespace entries whose stored value is a canonical Type.
/// Their value is a descriptor relocation, never a serialized host pointer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReflectedTypeConstant {
    pub name: Box<[u8]>,
    pub represented_type: TypeId,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReflectedRecordMetadata {
    pub notes: Box<[Box<[u8]>]>,
    pub textual_flags: u32,
    pub status_flags: u32,
    pub nontextual_flags: u32,
    pub unsupported_members: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReflectedFieldMetadata {
    pub name: Option<Box<[u8]>>,
    pub using: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReflectedEnumMember {
    pub name: Option<Box<[u8]>>,
    pub value: Integer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReflectedEnumMetadata {
    /// Names and values stay in declaration order, including equal-valued aliases.
    pub members: Box<[ReflectedEnumMember]>,
    pub flags: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReflectedField {
    pub id: FieldId,
    pub name: Option<Box<[u8]>>,
    pub ty: DescriptorId,
    pub offset_in_bytes: u64,
    pub using: bool,
    pub notes: Box<[Box<[u8]>]>,
}

impl ReflectedField {
    /// Read the reduction retained by this immutable descriptor, independently
    /// of policies added to the registry after its publication.
    pub fn procedure_as_void_pointer(
        &self,
        types: &dyn TypeView,
        owner: TypeId,
    ) -> Result<bool, TypeError> {
        let declared = types.validate_field(owner, self.id)?;
        if !matches!(types.kind(declared)?, TypeKind::Procedure(_)) {
            return Ok(false);
        }
        let TypeKind::Pointer(pointee) = types.kind(self.ty.represented_type())? else {
            return Ok(false);
        };
        Ok(matches!(types.kind(*pointee)?, TypeKind::Void))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DescriptorKind {
    Void,
    Type,
    Code,
    Any,
    Bool,
    Integer {
        representation: IntegerType,
    },
    Float {
        representation: FloatType,
    },
    String,
    Pointer {
        pointee: DescriptorId,
    },
    FixedArray {
        element: DescriptorId,
        count: u64,
        stride: u64,
    },
    Slice {
        element: DescriptorId,
    },
    DynamicArray {
        element: DescriptorId,
    },
    Procedure {
        parameters: Box<[DescriptorId]>,
        results: Box<[DescriptorId]>,
        convention: CallingConvention,
        return_abi: crate::ForeignReturnAbi,
        context: ContextMode,
        variadic: Variadic,
    },
    Distinct {
        kind: DistinctKind,
        representation: DescriptorId,
    },
    Record {
        kind: RecordKind,
        fields: Box<[ReflectedField]>,
        constants: Box<[ReflectedTypeConstant]>,
        metadata: ReflectedRecordMetadata,
    },
    Enum {
        representation: IntegerType,
        members: Box<[ReflectedEnumMember]>,
        flags: bool,
    },
}

/// The tag values are Jai's source-level `Type_Info_Tag` discriminants. The
/// graph itself has no native ABI or pointer representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum TypeInfoTag {
    Integer = 0,
    Float = 1,
    Bool = 2,
    String = 3,
    Pointer = 4,
    Procedure = 5,
    Void = 6,
    Struct = 7,
    Array = 8,
    Any = 10,
    Enum = 11,
    Type = 13,
    Code = 14,
    Variant = 18,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeDescriptor {
    pub id: DescriptorId,
    pub name: Option<Box<[u8]>>,
    /// Void and Code have no runtime storage. Type uses its checked descriptor
    /// pointer cell; unsized descriptions cannot invent a `size_of` result.
    pub layout: Option<Layout>,
    pub kind: DescriptorKind,
}
impl TypeDescriptor {
    pub fn tag(&self) -> TypeInfoTag {
        match self.kind {
            DescriptorKind::Void => TypeInfoTag::Void,
            DescriptorKind::Type => TypeInfoTag::Type,
            DescriptorKind::Code => TypeInfoTag::Code,
            DescriptorKind::Any => TypeInfoTag::Any,
            DescriptorKind::Bool => TypeInfoTag::Bool,
            DescriptorKind::Integer {
                ..
            } => TypeInfoTag::Integer,
            DescriptorKind::Float {
                ..
            } => TypeInfoTag::Float,
            DescriptorKind::String => TypeInfoTag::String,
            DescriptorKind::Pointer {
                ..
            } => TypeInfoTag::Pointer,
            DescriptorKind::FixedArray {
                ..
            }
            | DescriptorKind::Slice {
                ..
            }
            | DescriptorKind::DynamicArray {
                ..
            } => TypeInfoTag::Array,
            DescriptorKind::Procedure {
                ..
            } => TypeInfoTag::Procedure,
            DescriptorKind::Record {
                ..
            } => TypeInfoTag::Struct,
            DescriptorKind::Enum {
                ..
            } => TypeInfoTag::Enum,
            DescriptorKind::Distinct {
                ..
            } => TypeInfoTag::Variant,
        }
    }
    pub fn runtime_size(&self) -> Option<u64> {
        self.layout.as_ref().map(|layout| layout.size)
    }
}

#[derive(Debug)]
pub enum ReflectionError {
    Type(TypeError),
    Layout(LayoutError),
    EnumMetadata(TypeId),
    UnknownDescriptor(TypeId),
    UnsupportedType(TypeId),
    UnsupportedRecordMembers(TypeId),
}
impl From<TypeError> for ReflectionError {
    fn from(error: TypeError) -> Self {
        Self::Type(error)
    }
}
impl From<LayoutError> for ReflectionError {
    fn from(error: LayoutError) -> Self {
        Self::Layout(error)
    }
}
impl fmt::Display for ReflectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type(error) => write!(f, "invalid reflected type: {error}"),
            Self::Layout(error) => write!(f, "invalid reflected target layout: {error}"),
            Self::EnumMetadata(_) => {
                f.write_str("reflected enum metadata does not match its nominal definition")
            }
            Self::UnknownDescriptor(_) => {
                f.write_str("descriptor is outside this reflection graph")
            }
            Self::UnsupportedType(_) => {
                f.write_str("reflection is not implemented for this type kind")
            }
            Self::UnsupportedRecordMembers(_) => f.write_str("record reflection of constant, type, procedure, or inserted members is not implemented"),
        }
    }
}
impl std::error::Error for ReflectionError {
}

/// A closed, immutable descriptor graph. Cycles are represented by descriptor
/// IDs; recursively linked records do not recursively allocate descriptions.
#[derive(Clone, Debug)]
pub struct ReflectionGraph {
    root: DescriptorId,
    policy: LayoutPolicy,
    descriptors: Box<[TypeDescriptor]>,
    indices: HashMap<TypeId, usize>,
}
impl ReflectionGraph {
    pub fn root(&self) -> DescriptorId {
        self.root
    }
    pub fn policy(&self) -> LayoutPolicy {
        self.policy
    }
    pub fn descriptors(&self) -> &[TypeDescriptor] {
        &self.descriptors
    }
    pub fn get(&self, id: DescriptorId) -> Result<&TypeDescriptor, ReflectionError> {
        self.descriptor(id.0)
    }
    pub fn descriptor(&self, ty: TypeId) -> Result<&TypeDescriptor, ReflectionError> {
        self.indices
            .get(&ty)
            .map(|&index| &self.descriptors[index])
            .ok_or(ReflectionError::UnknownDescriptor(ty))
    }

    /// Nothing is published when a target or nominal definition is pending.
    /// A fresh immutable graph can be requested after the dependency is ready.
    pub fn build(
        types: &dyn TypeView,
        root: TypeId,
        policy: Option<LayoutPolicy>,
        metadata: &ReflectionMetadata,
    ) -> Result<ReflectionReadiness<Self>, ReflectionError> {
        Self::build_with_roots(types, root, &[], policy, metadata)
    }

    /// Close a certified set of roots in one immutable graph. The primary root
    /// retains its identity; additional roots stay in their supplied order,
    /// before dependencies, and duplicate type identities appear only once.
    /// Table membership remains the caller's source-provenance decision.
    pub fn build_with_roots(
        types: &dyn TypeView,
        root: TypeId,
        additional: &[TypeId],
        policy: Option<LayoutPolicy>,
        metadata: &ReflectionMetadata,
    ) -> Result<ReflectionReadiness<Self>, ReflectionError> {
        types.kind(root)?;
        for &ty in additional {
            types.kind(ty)?;
        }
        let Some(policy) = policy else {
            return Ok(ReflectionReadiness::Pending(Box::new([
                ReflectionDependency::TargetLayout,
            ])));
        };
        let mut ordered = vec![root];
        let mut seen = HashSet::from([root]);
        for &ty in additional {
            if seen.insert(ty) {
                ordered.push(ty);
            }
        }
        let mut pending = Vec::new();
        let mut cursor = 0;
        while cursor < ordered.len() {
            let ty = ordered[cursor];
            cursor += 1;
            let dependencies = match dependencies(types, ty, metadata) {
                Ok(dependencies) => dependencies,
                Err(ReflectionError::Type(TypeError::Incomplete(ty))) => {
                    pending.push(ReflectionDependency::Definition(ty));
                    continue;
                }
                Err(error) => return Err(error),
            };
            for dependency in dependencies {
                types.kind(dependency)?;
                if seen.insert(dependency) {
                    ordered.push(dependency);
                }
            }
        }
        if !pending.is_empty() {
            return Ok(ReflectionReadiness::Pending(pending.into()));
        }
        let mut layouts = LayoutEngine::new(types, policy);
        let mut descriptors = Vec::with_capacity(ordered.len());
        let mut indices = HashMap::with_capacity(ordered.len());
        for ty in ordered {
            let layout = match types.kind(ty)? {
                TypeKind::Void | TypeKind::Code => None,
                _ => match layouts.layout(ty) {
                    Ok(layout) => Some(layout.clone()),
                    Err(LayoutError::Type(TypeError::Incomplete(ty))) => {
                        return Ok(ReflectionReadiness::Pending(Box::new([
                            ReflectionDependency::Definition(ty),
                        ])));
                    }
                    Err(error) => return Err(error.into()),
                },
            };
            let kind = describe(types, ty, layout.as_ref(), metadata)?;
            indices.insert(ty, descriptors.len());
            descriptors.push(TypeDescriptor {
                id: DescriptorId(ty),
                name: metadata.names.get(&ty).cloned(),
                layout,
                kind,
            });
        }
        Ok(ReflectionReadiness::Ready(Self {
            root: DescriptorId(root),
            policy,
            descriptors: descriptors.into(),
            indices,
        }))
    }
}

/// `size_of` shares the exact layout engine used by descriptor construction.
pub fn reflected_size(
    types: &dyn TypeView,
    ty: TypeId,
    policy: Option<LayoutPolicy>,
) -> Result<ReflectionReadiness<u64>, ReflectionError> {
    types.kind(ty)?;
    let Some(policy) = policy else {
        return Ok(ReflectionReadiness::Pending(Box::new([
            ReflectionDependency::TargetLayout,
        ])));
    };
    match LayoutEngine::new(types, policy).layout(ty) {
        Ok(layout) => Ok(ReflectionReadiness::Ready(layout.size)),
        Err(LayoutError::Type(TypeError::Incomplete(ty))) => {
            Ok(ReflectionReadiness::Pending(Box::new([
                ReflectionDependency::Definition(ty),
            ])))
        }
        Err(error) => Err(error.into()),
    }
}

fn reflected_member_type(
    types: &dyn TypeView,
    record: TypeId,
    field: FieldId,
) -> Result<Option<TypeId>, ReflectionError> {
    Ok(
        match types
            .record_reflection_policy(record)?
            .member_type(types, record, field)?
        {
            RecordMemberReflection::Omitted => None,
            RecordMemberReflection::Declared(ty) => Some(ty),
            RecordMemberReflection::VoidPointer => {
                let void = types
                    .lookup(&TypeKind::Void)
                    .ok_or(ReflectionError::UnsupportedType(record))?;
                Some(
                    types
                        .lookup(&TypeKind::Pointer(void))
                        .ok_or(ReflectionError::UnsupportedType(record))?,
                )
            }
        },
    )
}

fn dependencies(
    types: &dyn TypeView,
    ty: TypeId,
    metadata: &ReflectionMetadata,
) -> Result<Vec<TypeId>, ReflectionError> {
    Ok(match types.kind(ty)? {
        TypeKind::Pointer(element)
        | TypeKind::Slice(element)
        | TypeKind::DynamicArray(element)
        | TypeKind::FixedArray {
            element, ..
        } => vec![*element],
        TypeKind::Record(id) => {
            let record = types.record(*id)?;
            let mut fields = Vec::with_capacity(record.fields.len());
            for index in 0..record.fields.len() {
                if let Some(ty) = reflected_member_type(types, ty, types.field(ty, index)?.id)? {
                    fields.push(ty);
                }
            }
            if !types
                .record_reflection_policy(ty)?
                .contains(RecordReflectionFlag::NoTypeInfo)
                && let Some(constants) = metadata.type_constants.get(&ty)
                && !constants.is_empty()
            {
                fields.push(types.meta_type());
                fields.extend(constants.iter().map(|constant| constant.represented_type));
            }
            fields
        }
        TypeKind::Any(id) => {
            types.record(*id)?;
            vec![]
        }
        TypeKind::Distinct(id) => vec![types.distinct(*id)?.representation],
        TypeKind::Enum(id) => {
            let representation = types.enumeration(*id)?.representation;
            vec![types.scalar(crate::ScalarType::Int(representation))]
        }
        TypeKind::Procedure(id) => {
            let signature = types.procedure_type(*id)?;
            signature
                .parameters
                .iter()
                .chain(signature.results.iter())
                .copied()
                .collect()
        }
        _ => vec![],
    })
}

fn describe(
    types: &dyn TypeView,
    ty: TypeId,
    layout: Option<&Layout>,
    metadata: &ReflectionMetadata,
) -> Result<DescriptorKind, ReflectionError> {
    Ok(match types.kind(ty)? {
        TypeKind::Void => DescriptorKind::Void,
        TypeKind::Type => DescriptorKind::Type,
        TypeKind::Code => DescriptorKind::Code,
        TypeKind::Any(_) => DescriptorKind::Any,
        TypeKind::Bool => DescriptorKind::Bool,
        TypeKind::Integer(representation) => DescriptorKind::Integer {
            representation: *representation,
        },
        TypeKind::Float(representation) => DescriptorKind::Float {
            representation: *representation,
        },
        TypeKind::String => DescriptorKind::String,
        TypeKind::Pointer(pointee) => DescriptorKind::Pointer {
            pointee: DescriptorId(*pointee),
        },
        TypeKind::FixedArray {
            element,
            count,
        } => DescriptorKind::FixedArray {
            element: DescriptorId(*element),
            count: *count,
            stride: layout
                .and_then(|layout| layout.array_stride)
                .ok_or(ReflectionError::UnsupportedType(ty))?,
        },
        TypeKind::Slice(element) => DescriptorKind::Slice {
            element: DescriptorId(*element),
        },
        TypeKind::DynamicArray(element) => DescriptorKind::DynamicArray {
            element: DescriptorId(*element),
        },
        TypeKind::Procedure(id) => {
            let procedure = types.procedure_type(*id)?;
            DescriptorKind::Procedure {
                parameters: procedure
                    .parameters
                    .iter()
                    .map(|&ty| DescriptorId(ty))
                    .collect(),
                results: procedure
                    .results
                    .iter()
                    .map(|&ty| DescriptorId(ty))
                    .collect(),
                convention: procedure.convention,
                return_abi: procedure.return_abi,
                context: procedure.context,
                variadic: procedure.variadic,
            }
        }
        TypeKind::Distinct(id) => {
            let definition = types.distinct(*id)?;
            DescriptorKind::Distinct {
                kind: definition.kind,
                representation: DescriptorId(definition.representation),
            }
        }
        TypeKind::Record(id) => {
            let policy = types.record_reflection_policy(ty)?;
            if !policy.contains(RecordReflectionFlag::NoTypeInfo)
                && metadata
                    .records
                    .get(&ty)
                    .is_some_and(|metadata| metadata.unsupported_members)
            {
                return Err(ReflectionError::UnsupportedRecordMembers(ty));
            }
            let record = types.record(*id)?;
            let offsets = &layout
                .ok_or(ReflectionError::UnsupportedType(ty))?
                .field_offsets;
            let mut fields = Vec::with_capacity(record.fields.len());
            for index in 0..record.fields.len() {
                let field = types.field(ty, index)?;
                let Some(reflected_type) = reflected_member_type(types, ty, field.id)? else {
                    continue;
                };
                let names = metadata.fields.get(&field.id);
                fields.push(ReflectedField {
                    id: field.id,
                    name: names.and_then(|metadata| metadata.name.clone()),
                    ty: DescriptorId(reflected_type),
                    offset_in_bytes: offsets[index],
                    using: names.is_some_and(|metadata| metadata.using),
                    notes: metadata
                        .field_notes
                        .get(&field.id)
                        .cloned()
                        .unwrap_or_default(),
                });
            }
            let mut record_metadata = metadata.records.get(&ty).cloned().unwrap_or_default();
            let reductions = if policy.contains(RecordReflectionFlag::NoTypeInfo) {
                8
            } else {
                0
            } | if policy.contains(RecordReflectionFlag::ProceduresAreVoidPointers)
            {
                16
            } else {
                0
            } | if policy.contains(RecordReflectionFlag::NoSizeComplaint) {
                32
            } else {
                0
            };
            // These three source bits describe the policy observed by this
            // immutable graph, including a retained demand's policy view.
            record_metadata.textual_flags =
                (record_metadata.textual_flags & !(8 | 16 | 32)) | reductions;
            DescriptorKind::Record {
                kind: record.kind,
                fields: fields.into(),
                constants: if policy.contains(RecordReflectionFlag::NoTypeInfo) {
                    Box::new([])
                } else {
                    metadata
                        .type_constants
                        .get(&ty)
                        .cloned()
                        .unwrap_or_default()
                },
                metadata: record_metadata,
            }
        }
        TypeKind::Enum(id) => {
            let enumeration = types.enumeration(*id)?;
            let (members, flags) = match metadata.enumerations.get(&ty) {
                Some(metadata) => {
                    if metadata.members.len() != enumeration.values.len()
                        || metadata
                            .members
                            .iter()
                            .zip(&enumeration.values)
                            .any(|(member, value)| member.value != *value)
                    {
                        return Err(ReflectionError::EnumMetadata(ty));
                    }
                    (metadata.members.clone(), metadata.flags)
                }
                None => (
                    enumeration
                        .values
                        .iter()
                        .map(|&value| ReflectedEnumMember {
                            name: None,
                            value,
                        })
                        .collect(),
                    false,
                ),
            };
            DescriptorKind::Enum {
                representation: enumeration.representation,
                members,
                flags,
            }
        }
        #[allow(unreachable_patterns)]
        _ => return Err(ReflectionError::UnsupportedType(ty)),
    })
}

#[cfg(test)]
mod tests;
