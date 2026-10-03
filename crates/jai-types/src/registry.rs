use crate::{Integer, IntegerType, ScalarType};
use std::{
    collections::HashMap,
    fmt,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_ARENA: AtomicU64 = AtomicU64::new(1);

macro_rules! identity {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name {
            arena: u64,
            index: usize,
        }
        impl $name {
            pub fn index(self) -> usize {
                self.index
            }
        }
    };
}
identity!(TypeId);
identity!(RecordId);
identity!(EnumId);
identity!(DistinctId);
identity!(ProcedureTypeId);

/// A field ordinal bound to one nominal record in one compilation arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FieldId {
    record: RecordId,
    index: usize,
}
impl FieldId {
    pub fn record(self) -> RecordId {
        self.record
    }
    pub fn index(self) -> usize {
        self.index
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FieldDescriptor {
    pub id: FieldId,
    pub ty: TypeId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FloatType {
    F32,
    F64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RecordKind {
    Struct,
    Union,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CallingConvention {
    Jai,
    C,
    Stdcall,
    CppMethod,
}
impl CallingConvention {
    pub fn uses_c_abi(self) -> bool {
        matches!(self, Self::C | Self::CppMethod)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CppMethodIssue {
    Context,
    Receiver,
    Variadic,
    MultipleResults,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ContextMode {
    Implicit,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DistinctKind {
    Distinct,
    IsA,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Variadic {
    #[default]
    None,
    /// ABI parameters exclude the source pack; all stored parameters are fixed.
    C { fixed_parameters: usize },
    /// The stored parameter at this position is `Slice(element)`.
    Jai { parameter: usize, element: TypeId },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VariadicIssue {
    CallingConvention,
    FixedParameterCount,
    ParameterOutOfBounds,
    PackType,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Void,
    Type,
    /// Captured compiler syntax. It has no target runtime representation.
    Code,
    Bool,
    Integer(IntegerType),
    Float(FloatType),
    String,
    Pointer(TypeId),
    FixedArray {
        element: TypeId,
        count: u64,
    },
    Slice(TypeId),
    DynamicArray(TypeId),
    Procedure(ProcedureTypeId),
    Record(RecordId),
    /// The compilation's canonical universal value, with explicit record storage.
    Any(RecordId),
    Enum(EnumId),
    Distinct(DistinctId),
}
impl TypeKind {
    /// Storage ownership is shared; semantic record identity remains explicit.
    pub fn record_storage_id(&self) -> Option<RecordId> {
        match self {
            Self::Record(id) | Self::Any(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProcedureType {
    pub parameters: Box<[TypeId]>,
    pub results: Box<[TypeId]>,
    pub convention: CallingConvention,
    pub return_abi: crate::ForeignReturnAbi,
    pub context: ContextMode,
    pub variadic: Variadic,
}

#[derive(Debug)]
pub struct DistinctDefinition {
    pub kind: DistinctKind,
    pub representation: TypeId,
}
#[derive(Debug)]
pub struct RecordDefinition {
    pub kind: RecordKind,
    pub fields: Box<[TypeId]>,
    pub layout: RecordLayout,
}
/// Storage constraints declared by a record. A packed record suppresses natural
/// field alignment; an explicit alignment is a minimum for the whole record.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RecordLayout {
    pub packed: bool,
    pub minimum_alignment: Option<u32>,
    /// Explicit field alignments, including reductions below natural alignment.
    /// An empty list selects natural alignment for every field.
    pub field_alignments: Box<[Option<u32>]>,
    /// Rewind the cursor before this field to an earlier field of this record.
    /// Empty selects sequential storage. Identities are checked at definition.
    pub field_placements: Box<[Option<FieldId>]>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlacementIssue {
    FieldCount,
    AnchorNotEarlier { field: usize, anchor: usize },
    Union,
    DuplicateMetadata,
}
#[derive(Debug)]
pub struct EnumDefinition {
    pub representation: IntegerType,
    pub values: Box<[Integer]>,
}
enum Definition<T, K> {
    Reserved(K),
    Defined(T),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeError {
    InvalidPlacement {
        record: TypeId,
        issue: PlacementIssue,
    },
    InvalidCppMethod(CppMethodIssue),
    InvalidForeignReturn(crate::ForeignReturnIssue),
    ForeignType(TypeId),
    ForeignRecord(RecordId),
    ForeignEnum(EnumId),
    ForeignDistinct(DistinctId),
    ForeignProcedure(ProcedureTypeId),
    WrongKind(TypeId),
    NotAValue(TypeId),
    AlreadyDefined(TypeId),
    Incomplete(TypeId),
    RuntimeTypeHeaderConflict {
        expected: TypeId,
        actual: TypeId,
    },
    InvalidVariadic {
        variadic: Variadic,
        issue: VariadicIssue,
    },
    FieldOutOfBounds {
        record: TypeId,
        index: usize,
    },
    FieldOwner {
        record: TypeId,
        field: FieldId,
    },
    EnumRepresentation {
        expected: IntegerType,
        actual: IntegerType,
    },
    RecursiveValue {
        cycle: Vec<TypeId>,
    },
}
impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPlacement {
                issue, ..
            } => {
                write!(f, "invalid record placement: {issue:?}")
            }
            Self::InvalidCppMethod(issue) => write!(f, "invalid C++ method signature: {issue:?}"),
            Self::InvalidForeignReturn(issue) => {
                write!(f, "invalid foreign return policy: {issue:?}")
            }
            Self::ForeignType(_)
            | Self::ForeignRecord(_)
            | Self::ForeignEnum(_)
            | Self::ForeignDistinct(_)
            | Self::ForeignProcedure(_) => f.write_str("type identity belongs to another registry"),
            Self::WrongKind(_) => f.write_str("type definition has the wrong kind"),
            Self::NotAValue(_) => f.write_str("void and code values cannot occupy runtime storage"),
            Self::AlreadyDefined(_) => f.write_str("nominal type has already been defined"),
            Self::Incomplete(_) => f.write_str("nominal type definition is incomplete"),
            Self::RuntimeTypeHeaderConflict {
                ..
            } => f.write_str(
                "runtime Type descriptor header has already been bound to another nominal type",
            ),
            Self::InvalidVariadic {
                issue, ..
            } => {
                write!(f, "invalid variadic procedure signature: {issue:?}")
            }
            Self::FieldOutOfBounds {
                ..
            } => f.write_str("record field ordinal is out of bounds"),
            Self::FieldOwner {
                ..
            } => f.write_str("field belongs to a different nominal record"),
            Self::EnumRepresentation {
                ..
            } => f.write_str("enum value has the wrong integer representation"),
            Self::RecursiveValue {
                ..
            } => f.write_str("type contains itself by value"),
        }
    }
}
impl std::error::Error for TypeError {
}

/// Read-only access to completed descriptors, before or after the program freezes.
/// A ready descriptor can still refer to a nominal type whose definition is pending.
pub trait TypeView {
    fn kind(&self, id: TypeId) -> Result<&TypeKind, TypeError>;
    /// Finds an already interned structural form; nominal declarations are not canonical.
    fn lookup(&self, kind: &TypeKind) -> Option<TypeId>;
    /// Finds an already interned signature without reserving a new procedure identity.
    fn lookup_procedure(&self, _signature: &ProcedureType) -> Option<TypeId> {
        None
    }
    fn scalar(&self, ty: ScalarType) -> TypeId;
    fn float(&self, ty: FloatType) -> TypeId;
    /// A lazy universal reservation can exist before its storage definition is ready.
    fn any_type(&self) -> Option<TypeId>;
    /// The explicitly adopted Type_Info header, never inferred from its name or shape.
    fn runtime_type_header(&self) -> Option<TypeId> {
        None
    }
    fn allocator_schema(&self) -> Option<crate::AllocatorSchema> {
        None
    }
    fn record_reflection_policy(
        &self,
        record: TypeId,
    ) -> Result<crate::RecordReflectionPolicy, TypeError> {
        if !matches!(self.kind(record)?, TypeKind::Record(_)) {
            return Err(TypeError::WrongKind(record));
        }
        Ok(crate::RecordReflectionPolicy::default())
    }
    fn record(&self, id: RecordId) -> Result<&RecordDefinition, TypeError>;
    fn enumeration(&self, id: EnumId) -> Result<&EnumDefinition, TypeError>;
    fn distinct(&self, id: DistinctId) -> Result<&DistinctDefinition, TypeError>;
    fn procedure_type(&self, id: ProcedureTypeId) -> Result<&ProcedureType, TypeError>;
    fn record_type(&self, id: RecordId) -> Result<TypeId, TypeError>;

    fn code_type(&self) -> TypeId {
        self.lookup(&TypeKind::Code)
            .expect("registry includes the code metatype")
    }
    fn meta_type(&self) -> TypeId {
        self.lookup(&TypeKind::Type)
            .expect("registry includes the Type metatype")
    }

    fn record_definition(&self, ty: TypeId) -> Result<&RecordDefinition, TypeError> {
        let TypeKind::Record(id) = *self.kind(ty)? else {
            return Err(TypeError::WrongKind(ty));
        };
        self.record(id)
    }
    fn record_storage_definition(&self, ty: TypeId) -> Result<&RecordDefinition, TypeError> {
        let id = self
            .kind(ty)?
            .record_storage_id()
            .ok_or(TypeError::WrongKind(ty))?;
        self.record(id)
    }
    fn enum_definition(&self, ty: TypeId) -> Result<&EnumDefinition, TypeError> {
        let TypeKind::Enum(id) = *self.kind(ty)? else {
            return Err(TypeError::WrongKind(ty));
        };
        self.enumeration(id)
    }
    fn distinct_definition(&self, ty: TypeId) -> Result<&DistinctDefinition, TypeError> {
        let TypeKind::Distinct(id) = *self.kind(ty)? else {
            return Err(TypeError::WrongKind(ty));
        };
        self.distinct(id)
    }
    fn procedure_definition(&self, ty: TypeId) -> Result<&ProcedureType, TypeError> {
        let TypeKind::Procedure(id) = *self.kind(ty)? else {
            return Err(TypeError::WrongKind(ty));
        };
        self.procedure_type(id)
    }
    fn field(&self, record: TypeId, index: usize) -> Result<FieldDescriptor, TypeError> {
        let id = self
            .kind(record)?
            .record_storage_id()
            .ok_or(TypeError::WrongKind(record))?;
        let ty = *self
            .record(id)?
            .fields
            .get(index)
            .ok_or(TypeError::FieldOutOfBounds {
                record,
                index,
            })?;
        Ok(FieldDescriptor {
            id: FieldId {
                record: id,
                index,
            },
            ty,
        })
    }
    fn field_type(&self, field: FieldId) -> Result<TypeId, TypeError> {
        let record = self.record_type(field.record)?;
        Ok(self.field(record, field.index)?.ty)
    }
    /// Checks both the base record and the field's nominal owner before projection.
    fn validate_field(&self, record: TypeId, field: FieldId) -> Result<TypeId, TypeError> {
        let id = self
            .kind(record)?
            .record_storage_id()
            .ok_or(TypeError::WrongKind(record))?;
        let ty = self.field_type(field)?;
        if id != field.record {
            return Err(TypeError::FieldOwner {
                record,
                field,
            });
        }
        Ok(ty)
    }
}

pub struct TypeRegistry {
    arena: u64,
    kinds: Vec<TypeKind>,
    canonical: HashMap<TypeKind, TypeId>,
    any: Option<TypeId>,
    runtime_type_header: Option<TypeId>,
    allocator: Option<crate::AllocatorSchema>,
    record_reflection: HashMap<TypeId, crate::RecordReflectionPolicy>,
    records: Vec<Definition<RecordDefinition, RecordKind>>,
    record_types: Vec<TypeId>,
    enums: Vec<Definition<EnumDefinition, IntegerType>>,
    enum_types: Vec<TypeId>,
    distincts: Vec<Definition<DistinctDefinition, DistinctKind>>,
    distinct_types: Vec<TypeId>,
    procedures: Vec<ProcedureType>,
    procedure_ids: HashMap<ProcedureType, ProcedureTypeId>,
}

#[derive(Debug)]
pub struct Types {
    arena: u64,
    kinds: Vec<TypeKind>,
    canonical: HashMap<TypeKind, TypeId>,
    any: Option<TypeId>,
    runtime_type_header: Option<TypeId>,
    allocator: Option<crate::AllocatorSchema>,
    record_reflection: HashMap<TypeId, crate::RecordReflectionPolicy>,
    boolean: TypeId,
    integers: [TypeId; 8],
    floats: [TypeId; 2],
    records: Vec<RecordDefinition>,
    record_types: Vec<TypeId>,
    enums: Vec<EnumDefinition>,
    distincts: Vec<DistinctDefinition>,
    procedures: Vec<ProcedureType>,
    procedure_ids: HashMap<ProcedureType, ProcedureTypeId>,
}

const INTEGERS: [IntegerType; 8] = [
    IntegerType::S8,
    IntegerType::S16,
    IntegerType::S32,
    IntegerType::S64,
    IntegerType::U8,
    IntegerType::U16,
    IntegerType::U32,
    IntegerType::U64,
];

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
impl TypeView for TypeRegistry {
    fn kind(&self, id: TypeId) -> Result<&TypeKind, TypeError> {
        TypeRegistry::kind(self, id)
    }
    fn lookup(&self, kind: &TypeKind) -> Option<TypeId> {
        TypeRegistry::lookup(self, kind)
    }
    fn lookup_procedure(&self, signature: &ProcedureType) -> Option<TypeId> {
        TypeRegistry::lookup_procedure(self, signature)
    }
    fn scalar(&self, ty: ScalarType) -> TypeId {
        TypeRegistry::scalar(self, ty)
    }
    fn float(&self, ty: FloatType) -> TypeId {
        TypeRegistry::float(self, ty)
    }
    fn any_type(&self) -> Option<TypeId> {
        TypeRegistry::any_type(self)
    }
    fn runtime_type_header(&self) -> Option<TypeId> {
        TypeRegistry::runtime_type_header(self)
    }
    fn allocator_schema(&self) -> Option<crate::AllocatorSchema> {
        TypeRegistry::allocator_schema(self)
    }
    fn record_reflection_policy(
        &self,
        record: TypeId,
    ) -> Result<crate::RecordReflectionPolicy, TypeError> {
        TypeRegistry::record_reflection_policy(self, record)
    }
    fn record(&self, id: RecordId) -> Result<&RecordDefinition, TypeError> {
        let ty = self.record_type(id)?;
        match &self.records[id.index] {
            Definition::Reserved(_) => Err(TypeError::Incomplete(ty)),
            Definition::Defined(record) => Ok(record),
        }
    }
    fn enumeration(&self, id: EnumId) -> Result<&EnumDefinition, TypeError> {
        if id.arena != self.arena {
            return Err(TypeError::ForeignEnum(id));
        }
        let &ty = self
            .enum_types
            .get(id.index)
            .ok_or(TypeError::ForeignEnum(id))?;
        match &self.enums[id.index] {
            Definition::Reserved(_) => Err(TypeError::Incomplete(ty)),
            Definition::Defined(enumeration) => Ok(enumeration),
        }
    }
    fn distinct(&self, id: DistinctId) -> Result<&DistinctDefinition, TypeError> {
        if id.arena != self.arena {
            return Err(TypeError::ForeignDistinct(id));
        }
        let &ty = self
            .distinct_types
            .get(id.index)
            .ok_or(TypeError::ForeignDistinct(id))?;
        match &self.distincts[id.index] {
            Definition::Reserved(_) => Err(TypeError::Incomplete(ty)),
            Definition::Defined(distinct) => Ok(distinct),
        }
    }
    fn procedure_type(&self, id: ProcedureTypeId) -> Result<&ProcedureType, TypeError> {
        if id.arena != self.arena {
            return Err(TypeError::ForeignProcedure(id));
        }
        self.procedures
            .get(id.index)
            .ok_or(TypeError::ForeignProcedure(id))
    }
    fn record_type(&self, id: RecordId) -> Result<TypeId, TypeError> {
        if id.arena != self.arena {
            return Err(TypeError::ForeignRecord(id));
        }
        self.record_types
            .get(id.index)
            .copied()
            .ok_or(TypeError::ForeignRecord(id))
    }
}

impl TypeView for Types {
    fn kind(&self, id: TypeId) -> Result<&TypeKind, TypeError> {
        Types::kind(self, id)
    }
    fn lookup(&self, kind: &TypeKind) -> Option<TypeId> {
        Types::lookup(self, kind)
    }
    fn lookup_procedure(&self, signature: &ProcedureType) -> Option<TypeId> {
        Types::lookup_procedure(self, signature)
    }
    fn scalar(&self, ty: ScalarType) -> TypeId {
        Types::scalar(self, ty)
    }
    fn float(&self, ty: FloatType) -> TypeId {
        Types::float(self, ty)
    }
    fn any_type(&self) -> Option<TypeId> {
        Types::any_type(self)
    }
    fn runtime_type_header(&self) -> Option<TypeId> {
        Types::runtime_type_header(self)
    }
    fn allocator_schema(&self) -> Option<crate::AllocatorSchema> {
        Types::allocator_schema(self)
    }
    fn record_reflection_policy(
        &self,
        record: TypeId,
    ) -> Result<crate::RecordReflectionPolicy, TypeError> {
        Types::record_reflection_policy(self, record)
    }
    fn record(&self, id: RecordId) -> Result<&RecordDefinition, TypeError> {
        Types::record(self, id)
    }
    fn enumeration(&self, id: EnumId) -> Result<&EnumDefinition, TypeError> {
        Types::enumeration(self, id)
    }
    fn distinct(&self, id: DistinctId) -> Result<&DistinctDefinition, TypeError> {
        Types::distinct(self, id)
    }
    fn procedure_type(&self, id: ProcedureTypeId) -> Result<&ProcedureType, TypeError> {
        Types::procedure(self, id)
    }
    fn record_type(&self, id: RecordId) -> Result<TypeId, TypeError> {
        if id.arena != self.arena {
            return Err(TypeError::ForeignRecord(id));
        }
        self.record_types
            .get(id.index)
            .copied()
            .ok_or(TypeError::ForeignRecord(id))
    }
}

impl TypeRegistry {
    pub fn new() -> Self {
        let arena = NEXT_ARENA
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .expect("type registry identity space exhausted");
        let mut registry = Self {
            arena,
            kinds: vec![],
            canonical: HashMap::new(),
            any: None,
            runtime_type_header: None,
            allocator: None,
            record_reflection: HashMap::new(),
            records: vec![],
            record_types: vec![],
            enums: vec![],
            enum_types: vec![],
            distincts: vec![],
            distinct_types: vec![],
            procedures: vec![],
            procedure_ids: HashMap::new(),
        };
        for kind in [
            TypeKind::Void,
            TypeKind::Type,
            TypeKind::Code,
            TypeKind::Bool,
            TypeKind::String,
        ] {
            registry.intern(kind);
        }
        for ty in INTEGERS {
            registry.intern(TypeKind::Integer(ty));
        }
        for ty in [FloatType::F32, FloatType::F64] {
            registry.intern(TypeKind::Float(ty));
        }
        registry
    }
    fn push(&mut self, kind: TypeKind) -> TypeId {
        let id = TypeId {
            arena: self.arena,
            index: self.kinds.len(),
        };
        self.kinds.push(kind);
        id
    }
    fn intern(&mut self, kind: TypeKind) -> TypeId {
        if let Some(&id) = self.canonical.get(&kind) {
            return id;
        }
        let id = self.push(kind.clone());
        self.canonical.insert(kind, id);
        id
    }
    pub fn kind(&self, id: TypeId) -> Result<&TypeKind, TypeError> {
        if id.arena != self.arena {
            return Err(TypeError::ForeignType(id));
        }
        self.kinds.get(id.index).ok_or(TypeError::ForeignType(id))
    }
    pub fn lookup(&self, kind: &TypeKind) -> Option<TypeId> {
        self.canonical.get(kind).copied()
    }
    pub fn lookup_procedure(&self, signature: &ProcedureType) -> Option<TypeId> {
        let &id = self.procedure_ids.get(signature)?;
        self.lookup(&TypeKind::Procedure(id))
    }
    pub fn record_definition(&self, ty: TypeId) -> Result<&RecordDefinition, TypeError> {
        TypeView::record_definition(self, ty)
    }
    pub fn record_storage_definition(&self, ty: TypeId) -> Result<&RecordDefinition, TypeError> {
        TypeView::record_storage_definition(self, ty)
    }
    pub fn enum_definition(&self, ty: TypeId) -> Result<&EnumDefinition, TypeError> {
        TypeView::enum_definition(self, ty)
    }
    pub fn distinct_definition(&self, ty: TypeId) -> Result<&DistinctDefinition, TypeError> {
        TypeView::distinct_definition(self, ty)
    }
    pub fn procedure_definition(&self, ty: TypeId) -> Result<&ProcedureType, TypeError> {
        TypeView::procedure_definition(self, ty)
    }
    pub fn field(&self, record: TypeId, index: usize) -> Result<FieldDescriptor, TypeError> {
        TypeView::field(self, record, index)
    }
    pub fn field_type(&self, field: FieldId) -> Result<TypeId, TypeError> {
        TypeView::field_type(self, field)
    }
    pub fn validate_field(&self, record: TypeId, field: FieldId) -> Result<TypeId, TypeError> {
        TypeView::validate_field(self, record, field)
    }
    fn value(&self, id: TypeId) -> Result<(), TypeError> {
        match self.kind(id)? {
            TypeKind::Void | TypeKind::Code => Err(TypeError::NotAValue(id)),
            _ => Ok(()),
        }
    }
    pub fn scalar(&self, ty: ScalarType) -> TypeId {
        self.canonical[&match ty {
            ScalarType::Int(i) => TypeKind::Integer(i),
            ScalarType::Bool => TypeKind::Bool,
        }]
    }
    pub fn float(&self, ty: FloatType) -> TypeId {
        self.canonical[&TypeKind::Float(ty)]
    }
    pub fn void(&self) -> TypeId {
        self.canonical[&TypeKind::Void]
    }
    pub fn string(&self) -> TypeId {
        self.canonical[&TypeKind::String]
    }
    pub fn meta_type(&self) -> TypeId {
        self.canonical[&TypeKind::Type]
    }
    pub fn runtime_type_header(&self) -> Option<TypeId> {
        self.runtime_type_header
    }
    pub fn allocator_schema(&self) -> Option<crate::AllocatorSchema> {
        self.allocator
    }
    pub(crate) fn set_allocator_schema(&mut self, schema: crate::AllocatorSchema) {
        self.allocator = Some(schema);
    }
    pub fn record_reflection_policy(
        &self,
        record: TypeId,
    ) -> Result<crate::RecordReflectionPolicy, TypeError> {
        if !matches!(self.kind(record)?, TypeKind::Record(_)) {
            return Err(TypeError::WrongKind(record));
        }
        Ok(self
            .record_reflection
            .get(&record)
            .copied()
            .unwrap_or_default())
    }
    pub(crate) fn reserve_record_reflection_policy_capacity(&mut self, additional: usize) {
        self.record_reflection.reserve(additional);
    }
    pub(crate) fn set_record_reflection_policy(
        &mut self,
        record: TypeId,
        policy: crate::RecordReflectionPolicy,
    ) {
        if policy == crate::RecordReflectionPolicy::default() {
            self.record_reflection.remove(&record);
        } else {
            self.record_reflection.insert(record, policy);
        }
    }
    /// Bind the checked source or compiler-owned reflection catalog once.
    /// Failed attempts neither bind a header nor intern its pointer type.
    pub fn bind_runtime_type_header(&mut self, header: TypeId) -> Result<(), TypeError> {
        if self.record_definition(header)?.kind != RecordKind::Struct {
            return Err(TypeError::WrongKind(header));
        }
        if let Some(expected) = self.runtime_type_header {
            if expected != header {
                return Err(TypeError::RuntimeTypeHeaderConflict {
                    expected,
                    actual: header,
                });
            }
            return Ok(());
        }
        self.pointer(header)?;
        self.runtime_type_header = Some(header);
        Ok(())
    }
    pub fn code_type(&self) -> TypeId {
        self.canonical[&TypeKind::Code]
    }
    pub fn any_type(&self) -> Option<TypeId> {
        self.any
    }
    /// Reserves the one universal type without synthesizing a Type_Info schema.
    pub fn reserve_any(&mut self) -> TypeId {
        if let Some(ty) = self.any {
            return ty;
        }
        let id = RecordId {
            arena: self.arena,
            index: self.records.len(),
        };
        self.records.push(Definition::Reserved(RecordKind::Struct));
        let kind = TypeKind::Any(id);
        let ty = self.push(kind.clone());
        self.record_types.push(ty);
        self.canonical.insert(kind, ty);
        self.any = Some(ty);
        ty
    }
    /// Completes universal storage from the compilation's checked descriptor-header ID.
    pub fn define_any(&mut self, ty: TypeId, header_type: TypeId) -> Result<(), TypeError> {
        let TypeKind::Any(id) = *self.kind(ty)? else {
            return Err(TypeError::WrongKind(ty));
        };
        if matches!(self.records[id.index], Definition::Defined(_)) {
            return Err(TypeError::AlreadyDefined(ty));
        }
        if self.record_definition(header_type)?.kind != RecordKind::Struct {
            return Err(TypeError::WrongKind(header_type));
        }
        // No fallible semantic checks remain after creating these canonical pointers.
        let descriptor = self.pointer(header_type)?;
        let payload = self.pointer(self.void())?;
        self.records[id.index] = Definition::Defined(RecordDefinition {
            kind: RecordKind::Struct,
            fields: Box::new([descriptor, payload]),
            layout: RecordLayout::default(),
        });
        Ok(())
    }
    pub fn pointer(&mut self, pointee: TypeId) -> Result<TypeId, TypeError> {
        self.kind(pointee)?;
        Ok(self.intern(TypeKind::Pointer(pointee)))
    }
    pub fn fixed_array(&mut self, element: TypeId, count: u64) -> Result<TypeId, TypeError> {
        self.value(element)?;
        Ok(self.intern(TypeKind::FixedArray {
            element,
            count,
        }))
    }
    pub fn slice(&mut self, element: TypeId) -> Result<TypeId, TypeError> {
        self.value(element)?;
        Ok(self.intern(TypeKind::Slice(element)))
    }
    pub fn dynamic_array(&mut self, element: TypeId) -> Result<TypeId, TypeError> {
        self.value(element)?;
        Ok(self.intern(TypeKind::DynamicArray(element)))
    }
    pub fn procedure(&mut self, signature: ProcedureType) -> Result<TypeId, TypeError> {
        for &ty in signature.parameters.iter().chain(signature.results.iter()) {
            self.value(ty)?;
        }
        crate::foreign_return::validate(self, &signature)?;
        if signature.convention == CallingConvention::CppMethod {
            let issue = if signature.context != ContextMode::None {
                Some(CppMethodIssue::Context)
            } else if !matches!(
                signature.parameters.first().map(|ty| self.kind(*ty)),
                Some(Ok(TypeKind::Pointer(_)))
            ) {
                Some(CppMethodIssue::Receiver)
            } else if signature.variadic != Variadic::None {
                Some(CppMethodIssue::Variadic)
            } else if signature.results.len() > 1 {
                Some(CppMethodIssue::MultipleResults)
            } else {
                None
            };
            if let Some(issue) = issue {
                return Err(TypeError::InvalidCppMethod(issue));
            }
        }
        let issue = match signature.variadic {
            Variadic::None => None,
            Variadic::C {
                fixed_parameters,
            } => {
                if signature.convention != CallingConvention::C {
                    Some(VariadicIssue::CallingConvention)
                } else if fixed_parameters != signature.parameters.len() {
                    Some(VariadicIssue::FixedParameterCount)
                } else {
                    None
                }
            }
            Variadic::Jai {
                parameter,
                element,
            } => {
                self.value(element)?;
                if signature.convention != CallingConvention::Jai {
                    Some(VariadicIssue::CallingConvention)
                } else if let Some(&pack) = signature.parameters.get(parameter) {
                    if *self.kind(pack)? == TypeKind::Slice(element) {
                        None
                    } else {
                        Some(VariadicIssue::PackType)
                    }
                } else {
                    Some(VariadicIssue::ParameterOutOfBounds)
                }
            }
        };
        if let Some(issue) = issue {
            return Err(TypeError::InvalidVariadic {
                variadic: signature.variadic,
                issue,
            });
        }
        let id = if let Some(&id) = self.procedure_ids.get(&signature) {
            id
        } else {
            let id = ProcedureTypeId {
                arena: self.arena,
                index: self.procedures.len(),
            };
            self.procedures.push(signature.clone());
            self.procedure_ids.insert(signature, id);
            id
        };
        Ok(self.intern(TypeKind::Procedure(id)))
    }
    pub fn reserve_record(&mut self, kind: RecordKind) -> TypeId {
        let id = RecordId {
            arena: self.arena,
            index: self.records.len(),
        };
        self.records.push(Definition::Reserved(kind));
        let ty = self.push(TypeKind::Record(id));
        self.record_types.push(ty);
        ty
    }
    pub fn define_record(
        &mut self,
        ty: TypeId,
        fields: impl Into<Box<[TypeId]>>,
    ) -> Result<(), TypeError> {
        self.define_record_with_layout(ty, fields, RecordLayout::default())
    }
    pub fn define_record_with_layout(
        &mut self,
        ty: TypeId,
        fields: impl Into<Box<[TypeId]>>,
        layout: RecordLayout,
    ) -> Result<(), TypeError> {
        let TypeKind::Record(id) = *self.kind(ty)? else {
            return Err(TypeError::WrongKind(ty));
        };
        let fields = fields.into();
        for &field in &fields {
            self.value(field)?;
        }
        let Definition::Reserved(kind) = &self.records[id.index] else {
            return Err(TypeError::AlreadyDefined(ty));
        };
        Self::validate_placements(ty, id, *kind, fields.len(), &layout.field_placements)?;
        let definition = &mut self.records[id.index];
        let Definition::Reserved(kind) = definition else {
            return Err(TypeError::AlreadyDefined(ty));
        };
        *definition = Definition::Defined(RecordDefinition {
            kind: *kind,
            fields,
            layout,
        });
        Ok(())
    }
    /// Bind source-resolved anchor ordinals to the actual reserved record.
    /// Validation is transactional: a rejected definition remains reserved.
    pub fn define_record_with_placements(
        &mut self,
        ty: TypeId,
        fields: impl Into<Box<[TypeId]>>,
        mut layout: RecordLayout,
        anchors: impl Into<Box<[Option<usize>]>>,
    ) -> Result<(), TypeError> {
        let TypeKind::Record(id) = *self.kind(ty)? else {
            return Err(TypeError::WrongKind(ty));
        };
        if !layout.field_placements.is_empty() {
            return Err(TypeError::InvalidPlacement {
                record: ty,
                issue: PlacementIssue::DuplicateMetadata,
            });
        }
        layout.field_placements = anchors
            .into()
            .iter()
            .map(|anchor| {
                anchor.map(|index| FieldId {
                    record: id,
                    index,
                })
            })
            .collect();
        self.define_record_with_layout(ty, fields, layout)
    }
    fn validate_placements(
        ty: TypeId,
        owner: RecordId,
        kind: RecordKind,
        field_count: usize,
        placements: &[Option<FieldId>],
    ) -> Result<(), TypeError> {
        let invalid = |issue| TypeError::InvalidPlacement {
            record: ty,
            issue,
        };
        if placements.is_empty() {
            return Ok(());
        }
        if placements.len() != field_count {
            return Err(invalid(PlacementIssue::FieldCount));
        }
        for (index, anchor) in placements.iter().enumerate() {
            let Some(anchor) = anchor else {
                continue;
            };
            if anchor.record != owner {
                return Err(TypeError::FieldOwner {
                    record: ty,
                    field: *anchor,
                });
            }
            if kind == RecordKind::Union {
                return Err(invalid(PlacementIssue::Union));
            }
            if anchor.index >= index {
                return Err(invalid(PlacementIssue::AnchorNotEarlier {
                    field: index,
                    anchor: anchor.index,
                }));
            }
        }
        Ok(())
    }
    pub fn reserve_enum(&mut self, representation: IntegerType) -> TypeId {
        let id = EnumId {
            arena: self.arena,
            index: self.enums.len(),
        };
        self.enums.push(Definition::Reserved(representation));
        let ty = self.push(TypeKind::Enum(id));
        self.enum_types.push(ty);
        ty
    }
    pub fn reserve_distinct(&mut self, kind: DistinctKind) -> TypeId {
        let id = DistinctId {
            arena: self.arena,
            index: self.distincts.len(),
        };
        self.distincts.push(Definition::Reserved(kind));
        let ty = self.push(TypeKind::Distinct(id));
        self.distinct_types.push(ty);
        ty
    }
    pub fn define_distinct(&mut self, ty: TypeId, representation: TypeId) -> Result<(), TypeError> {
        let TypeKind::Distinct(id) = *self.kind(ty)? else {
            return Err(TypeError::WrongKind(ty));
        };
        self.value(representation)?;
        let definition = &mut self.distincts[id.index];
        let Definition::Reserved(kind) = definition else {
            return Err(TypeError::AlreadyDefined(ty));
        };
        *definition = Definition::Defined(DistinctDefinition {
            kind: *kind,
            representation,
        });
        Ok(())
    }
    pub fn define_enum(
        &mut self,
        ty: TypeId,
        values: impl Into<Box<[Integer]>>,
    ) -> Result<(), TypeError> {
        let TypeKind::Enum(id) = *self.kind(ty)? else {
            return Err(TypeError::WrongKind(ty));
        };
        let definition = &mut self.enums[id.index];
        let Definition::Reserved(representation) = definition else {
            return Err(TypeError::AlreadyDefined(ty));
        };
        let values = values.into();
        for value in &values {
            if value.ty() != *representation {
                return Err(TypeError::EnumRepresentation {
                    expected: *representation,
                    actual: value.ty(),
                });
            }
        }
        *definition = Definition::Defined(EnumDefinition {
            representation: *representation,
            values,
        });
        Ok(())
    }
    pub fn freeze(self) -> Result<Types, TypeError> {
        for (index, kind) in self.kinds.iter().enumerate() {
            let incomplete = match kind {
                TypeKind::Record(id) | TypeKind::Any(id) => {
                    matches!(self.records[id.index], Definition::Reserved(_))
                }
                TypeKind::Enum(id) => matches!(self.enums[id.index], Definition::Reserved(_)),
                TypeKind::Distinct(id) => {
                    matches!(self.distincts[id.index], Definition::Reserved(_))
                }
                _ => false,
            };
            if incomplete {
                return Err(TypeError::Incomplete(TypeId {
                    arena: self.arena,
                    index,
                }));
            }
        }
        let boolean = self.scalar(ScalarType::Bool);
        let integers = INTEGERS.map(|ty| self.scalar(ScalarType::Int(ty)));
        let floats = [self.float(FloatType::F32), self.float(FloatType::F64)];
        let types = Types {
            arena: self.arena,
            kinds: self.kinds,
            canonical: self.canonical,
            any: self.any,
            runtime_type_header: self.runtime_type_header,
            allocator: self.allocator,
            record_reflection: self.record_reflection,
            boolean,
            integers,
            floats,
            records: self
                .records
                .into_iter()
                .map(|r| match r {
                    Definition::Defined(r) => r,
                    Definition::Reserved(_) => unreachable!(),
                })
                .collect(),
            record_types: self.record_types,
            enums: self
                .enums
                .into_iter()
                .map(|r| match r {
                    Definition::Defined(r) => r,
                    Definition::Reserved(_) => unreachable!(),
                })
                .collect(),
            distincts: self
                .distincts
                .into_iter()
                .map(|definition| match definition {
                    Definition::Defined(distinct) => distinct,
                    Definition::Reserved(_) => unreachable!(),
                })
                .collect(),
            procedures: self.procedures,
            procedure_ids: self.procedure_ids,
        };
        types.check_value_cycles()?;
        Ok(types)
    }
}
impl Types {
    pub fn record_reflection_policy(
        &self,
        record: TypeId,
    ) -> Result<crate::RecordReflectionPolicy, TypeError> {
        if !matches!(self.kind(record)?, TypeKind::Record(_)) {
            return Err(TypeError::WrongKind(record));
        }
        Ok(self
            .record_reflection
            .get(&record)
            .copied()
            .unwrap_or_default())
    }
    pub fn allocator_schema(&self) -> Option<crate::AllocatorSchema> {
        self.allocator
    }
    pub fn meta_type(&self) -> TypeId {
        TypeView::meta_type(self)
    }
    pub fn runtime_type_header(&self) -> Option<TypeId> {
        self.runtime_type_header
    }
    pub fn any_type(&self) -> Option<TypeId> {
        self.any
    }
    /// Retrieves the same builtin identity reserved by the mutable registry.
    pub fn scalar(&self, ty: ScalarType) -> TypeId {
        match ty {
            ScalarType::Bool => self.boolean,
            ScalarType::Int(ty) => {
                self.integers[INTEGERS.iter().position(|&integer| integer == ty).unwrap()]
            }
        }
    }
    pub fn float(&self, ty: FloatType) -> TypeId {
        self.floats[match ty {
            FloatType::F32 => 0,
            FloatType::F64 => 1,
        }]
    }
    pub fn kind(&self, id: TypeId) -> Result<&TypeKind, TypeError> {
        if id.arena != self.arena {
            return Err(TypeError::ForeignType(id));
        }
        self.kinds.get(id.index).ok_or(TypeError::ForeignType(id))
    }
    pub fn lookup(&self, kind: &TypeKind) -> Option<TypeId> {
        self.canonical.get(kind).copied()
    }
    pub fn lookup_procedure(&self, signature: &ProcedureType) -> Option<TypeId> {
        let &id = self.procedure_ids.get(signature)?;
        self.lookup(&TypeKind::Procedure(id))
    }
    pub fn record_definition(&self, ty: TypeId) -> Result<&RecordDefinition, TypeError> {
        TypeView::record_definition(self, ty)
    }
    pub fn record_storage_definition(&self, ty: TypeId) -> Result<&RecordDefinition, TypeError> {
        TypeView::record_storage_definition(self, ty)
    }
    pub fn enum_definition(&self, ty: TypeId) -> Result<&EnumDefinition, TypeError> {
        TypeView::enum_definition(self, ty)
    }
    pub fn distinct_definition(&self, ty: TypeId) -> Result<&DistinctDefinition, TypeError> {
        TypeView::distinct_definition(self, ty)
    }
    pub fn procedure_definition(&self, ty: TypeId) -> Result<&ProcedureType, TypeError> {
        TypeView::procedure_definition(self, ty)
    }
    pub fn field(&self, record: TypeId, index: usize) -> Result<FieldDescriptor, TypeError> {
        TypeView::field(self, record, index)
    }
    pub fn field_type(&self, field: FieldId) -> Result<TypeId, TypeError> {
        TypeView::field_type(self, field)
    }
    pub fn validate_field(&self, record: TypeId, field: FieldId) -> Result<TypeId, TypeError> {
        TypeView::validate_field(self, record, field)
    }
    pub fn record(&self, id: RecordId) -> Result<&RecordDefinition, TypeError> {
        if id.arena != self.arena {
            return Err(TypeError::ForeignRecord(id));
        }
        self.records
            .get(id.index)
            .ok_or(TypeError::ForeignRecord(id))
    }
    pub fn enumeration(&self, id: EnumId) -> Result<&EnumDefinition, TypeError> {
        if id.arena != self.arena {
            return Err(TypeError::ForeignEnum(id));
        }
        self.enums.get(id.index).ok_or(TypeError::ForeignEnum(id))
    }
    pub fn distinct(&self, id: DistinctId) -> Result<&DistinctDefinition, TypeError> {
        if id.arena != self.arena {
            return Err(TypeError::ForeignDistinct(id));
        }
        self.distincts
            .get(id.index)
            .ok_or(TypeError::ForeignDistinct(id))
    }
    pub fn procedure(&self, id: ProcedureTypeId) -> Result<&ProcedureType, TypeError> {
        if id.arena != self.arena {
            return Err(TypeError::ForeignProcedure(id));
        }
        self.procedures
            .get(id.index)
            .ok_or(TypeError::ForeignProcedure(id))
    }
    pub fn iter(&self) -> impl Iterator<Item = (TypeId, &TypeKind)> {
        self.kinds.iter().enumerate().map(|(index, kind)| {
            (
                TypeId {
                    arena: self.arena,
                    index,
                },
                kind,
            )
        })
    }
    fn value_dependencies(&self, id: TypeId) -> &[TypeId] {
        match &self.kinds[id.index] {
            TypeKind::Record(record) | TypeKind::Any(record) => &self.records[record.index].fields,
            TypeKind::FixedArray {
                element, ..
            } => std::slice::from_ref(element),
            TypeKind::Distinct(distinct) => {
                std::slice::from_ref(&self.distincts[distinct.index].representation)
            }
            _ => &[],
        }
    }
    fn check_value_cycles(&self) -> Result<(), TypeError> {
        #[derive(Clone, Copy, PartialEq)]
        enum Visit {
            New,
            Active,
            Complete,
        }
        let mut visits = vec![Visit::New; self.kinds.len()];
        let mut stack: Vec<(TypeId, usize)> = vec![];
        for (root, _) in self.iter() {
            if visits[root.index] != Visit::New {
                continue;
            }
            visits[root.index] = Visit::Active;
            stack.push((root, 0));
            while let Some((ty, next)) = stack.last_mut() {
                let dependencies = self.value_dependencies(*ty);
                if let Some(&dependency) = dependencies.get(*next) {
                    *next += 1;
                    match visits[dependency.index] {
                        Visit::New => {
                            visits[dependency.index] = Visit::Active;
                            stack.push((dependency, 0));
                        }
                        Visit::Active => {
                            let start = stack.iter().position(|(id, _)| *id == dependency).unwrap();
                            let mut cycle =
                                stack[start..].iter().map(|(id, _)| *id).collect::<Vec<_>>();
                            cycle.push(dependency);
                            return Err(TypeError::RecursiveValue {
                                cycle,
                            });
                        }
                        Visit::Complete => {}
                    }
                } else {
                    visits[ty.index] = Visit::Complete;
                    stack.pop();
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn procedure_lookup_preserves_exact_signature_and_arena_through_freeze() {
        let mut registry = TypeRegistry::new();
        let boolean = registry.scalar(ScalarType::Bool);
        let signature = ProcedureType {
            parameters: Box::new([boolean]),
            results: Box::new([boolean]),
            return_abi: crate::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::Implicit,
            variadic: Variadic::None,
        };
        assert_eq!(registry.lookup_procedure(&signature), None);
        let procedure = registry.procedure(signature.clone()).unwrap();
        let foreign = TypeRegistry::new().scalar(ScalarType::Bool);
        let changed = ProcedureType {
            context: ContextMode::None,
            ..signature.clone()
        };
        let foreign_parameters = ProcedureType {
            parameters: Box::new([foreign]),
            ..signature.clone()
        };
        let foreign_results = ProcedureType {
            results: Box::new([foreign]),
            ..signature.clone()
        };
        let check = |view: &dyn TypeView| {
            assert_eq!(view.lookup_procedure(&signature), Some(procedure));
            assert_eq!(view.lookup_procedure(&changed), None);
            assert_eq!(view.lookup_procedure(&foreign_parameters), None);
            assert_eq!(view.lookup_procedure(&foreign_results), None);
            assert_eq!(view.procedure_definition(procedure).unwrap(), &signature);
        };
        check(&registry);
        check(&registry.freeze().unwrap());
    }

    #[test]
    fn structural_types_are_canonical_and_nominal_types_are_distinct() {
        let mut r = TypeRegistry::new();
        let byte = r.scalar(ScalarType::Int(IntegerType::U8));
        assert_eq!(r.pointer(byte).unwrap(), r.pointer(byte).unwrap());
        assert_eq!(
            r.fixed_array(byte, 3).unwrap(),
            r.fixed_array(byte, 3).unwrap()
        );
        assert_ne!(
            r.fixed_array(byte, 3).unwrap(),
            r.fixed_array(byte, 4).unwrap()
        );
        assert_ne!(r.slice(byte).unwrap(), r.dynamic_array(byte).unwrap());
        let a = r.reserve_record(RecordKind::Struct);
        let b = r.reserve_record(RecordKind::Struct);
        r.define_record(a, [byte]).unwrap();
        r.define_record(b, [byte]).unwrap();
        assert_ne!(a, b);
        r.freeze().unwrap();
    }
    #[test]
    fn builtin_identities_survive_freezing() {
        let r = TypeRegistry::new();
        let boolean = r.scalar(ScalarType::Bool);
        let integers = INTEGERS.map(|ty| (ty, r.scalar(ScalarType::Int(ty))));
        let floats = [FloatType::F32, FloatType::F64].map(|ty| (ty, r.float(ty)));
        let types = r.freeze().unwrap();
        assert_eq!(types.scalar(ScalarType::Bool), boolean);
        assert_eq!(types.kind(boolean).unwrap(), &TypeKind::Bool);
        for (ty, id) in integers {
            assert_eq!(types.scalar(ScalarType::Int(ty)), id);
            assert_eq!(types.kind(id).unwrap(), &TypeKind::Integer(ty));
        }
        let view: &dyn TypeView = &types;
        for (ty, id) in floats {
            assert_eq!(view.float(ty), id);
            assert_eq!(view.kind(id).unwrap(), &TypeKind::Float(ty));
        }
    }
    #[test]
    fn ready_views_report_pending_definitions_without_freezing_other_types() {
        let mut registry = TypeRegistry::new();
        let pending = registry.reserve_record(RecordKind::Struct);
        let ready = registry.reserve_record(RecordKind::Struct);
        registry.define_record(ready, [pending]).unwrap();
        let enumeration = registry.reserve_enum(IntegerType::S16);
        let procedure = registry
            .procedure(ProcedureType {
                parameters: Box::new([ready]),
                results: Box::new([pending]),
                return_abi: crate::ForeignReturnAbi::Natural,
                convention: CallingConvention::Jai,
                context: ContextMode::Implicit,
                variadic: Variadic::None,
            })
            .unwrap();
        let field = {
            let view: &dyn TypeView = &registry;
            assert!(
                matches!(view.record_definition(pending), Err(TypeError::Incomplete(id)) if id == pending)
            );
            assert!(
                matches!(view.enum_definition(enumeration), Err(TypeError::Incomplete(id)) if id == enumeration)
            );
            assert!(
                matches!(view.field(pending, 0), Err(TypeError::Incomplete(id)) if id == pending)
            );
            assert_eq!(
                view.record_definition(ready).unwrap().fields.as_ref(),
                &[pending]
            );
            assert_eq!(
                view.procedure_definition(procedure)
                    .unwrap()
                    .results
                    .as_ref(),
                &[pending]
            );
            assert_eq!(
                view.scalar(ScalarType::Bool),
                registry.scalar(ScalarType::Bool)
            );
            view.field(ready, 0).unwrap()
        };
        // Defining a referenced type does not change any previously ready descriptor or ID.
        registry.define_record(pending, []).unwrap();
        registry
            .define_enum(
                enumeration,
                [Integer::checked(IntegerType::S16, -2).unwrap()],
            )
            .unwrap();
        assert_eq!(registry.field(ready, 0).unwrap(), field);
        assert_eq!(
            registry.enum_definition(enumeration).unwrap().values[0].value(),
            -2
        );
        let frozen = registry.freeze().unwrap();
        let view: &dyn TypeView = &frozen;
        assert_eq!(view.field(ready, 0).unwrap(), field);
        assert_eq!(view.field_type(field.id).unwrap(), pending);
        assert_eq!(view.record_definition(pending).unwrap().fields.len(), 0);
        assert_eq!(
            view.procedure_definition(procedure)
                .unwrap()
                .parameters
                .as_ref(),
            &[ready]
        );
    }

    #[test]
    fn fields_check_nominal_owners_arena_identity_and_ordinals() {
        let mut registry = TypeRegistry::new();
        let byte = registry.scalar(ScalarType::Int(IntegerType::U8));
        let boolean = registry.scalar(ScalarType::Bool);
        let left = registry.reserve_record(RecordKind::Struct);
        let right = registry.reserve_record(RecordKind::Struct);
        registry.define_record(left, [byte, boolean]).unwrap();
        registry.define_record(right, [byte, boolean]).unwrap();
        let field = registry.field(left, 1).unwrap();
        assert_eq!(field.ty, boolean);
        assert_eq!(field.id.index(), 1);
        assert_eq!(registry.field_type(field.id), Ok(boolean));
        assert_eq!(registry.validate_field(left, field.id), Ok(boolean));
        assert_eq!(
            registry.validate_field(right, field.id),
            Err(TypeError::FieldOwner {
                record: right,
                field: field.id
            })
        );
        assert_eq!(registry.field(byte, 0), Err(TypeError::WrongKind(byte)));
        assert_eq!(
            registry.field(left, 2),
            Err(TypeError::FieldOutOfBounds {
                record: left,
                index: 2
            })
        );
        assert_eq!(
            registry.field_type(FieldId {
                record: field.id.record(),
                index: usize::MAX
            }),
            Err(TypeError::FieldOutOfBounds {
                record: left,
                index: usize::MAX
            })
        );

        let mut foreign = TypeRegistry::new();
        let foreign_record = foreign.reserve_record(RecordKind::Struct);
        let foreign_byte = foreign.scalar(ScalarType::Int(IntegerType::U8));
        foreign
            .define_record(foreign_record, [foreign_byte])
            .unwrap();
        let foreign_field = foreign.field(foreign_record, 0).unwrap().id;
        assert_eq!(
            registry.field_type(foreign_field),
            Err(TypeError::ForeignRecord(foreign_field.record()))
        );
        assert_eq!(
            registry.validate_field(left, foreign_field),
            Err(TypeError::ForeignRecord(foreign_field.record()))
        );
        assert_eq!(
            registry.field(foreign_record, 0),
            Err(TypeError::ForeignType(foreign_record))
        );
        let types = registry.freeze().unwrap();
        assert_eq!(types.field(left, 1).unwrap(), field);
        assert_eq!(
            types.validate_field(right, field.id),
            Err(TypeError::FieldOwner {
                record: right,
                field: field.id
            })
        );
        assert_eq!(
            types.field_type(foreign_field),
            Err(TypeError::ForeignRecord(foreign_field.record()))
        );
    }

    #[test]
    fn ready_views_reject_foreign_nominal_and_procedure_descriptors() {
        let mut owner = TypeRegistry::new();
        let record = owner.reserve_record(RecordKind::Struct);
        owner.define_record(record, []).unwrap();
        let enumeration = owner.reserve_enum(IntegerType::U8);
        owner.define_enum(enumeration, []).unwrap();
        let procedure = owner
            .procedure(ProcedureType {
                parameters: Box::new([]),
                results: Box::new([]),
                return_abi: crate::ForeignReturnAbi::Natural,
                convention: CallingConvention::C,
                context: ContextMode::None,
                variadic: Variadic::None,
            })
            .unwrap();
        let TypeKind::Record(record_id) = *owner.kind(record).unwrap() else {
            panic!()
        };
        let TypeKind::Enum(enum_id) = *owner.kind(enumeration).unwrap() else {
            panic!()
        };
        let TypeKind::Procedure(procedure_id) = *owner.kind(procedure).unwrap() else {
            panic!()
        };
        let foreign = TypeRegistry::new();
        let frozen = foreign.freeze().unwrap();
        let another = TypeRegistry::new();
        for view in [&another as &dyn TypeView, &frozen as &dyn TypeView] {
            assert!(
                matches!(view.record(record_id), Err(TypeError::ForeignRecord(id)) if id == record_id)
            );
            assert!(
                matches!(view.enumeration(enum_id), Err(TypeError::ForeignEnum(id)) if id == enum_id)
            );
            assert!(
                matches!(view.procedure_type(procedure_id), Err(TypeError::ForeignProcedure(id)) if id == procedure_id)
            );
        }
    }
    #[test]
    fn identities_cannot_escape_between_registries() {
        let mut a = TypeRegistry::new();
        let b = TypeRegistry::new();
        let foreign = b.scalar(ScalarType::Bool);
        assert_eq!(a.pointer(foreign), Err(TypeError::ForeignType(foreign)));
        assert_eq!(
            a.freeze().unwrap().kind(foreign),
            Err(TypeError::ForeignType(foreign))
        );
    }
    #[test]
    fn pointer_recursion_is_finite_and_value_recursion_is_rejected() {
        let mut r = TypeRegistry::new();
        let a = r.reserve_record(RecordKind::Struct);
        let b = r.reserve_record(RecordKind::Struct);
        let pa = r.pointer(a).unwrap();
        let pb = r.pointer(b).unwrap();
        r.define_record(a, [pb]).unwrap();
        r.define_record(b, [pa]).unwrap();
        r.freeze().unwrap();
        let mut r = TypeRegistry::new();
        let a = r.reserve_record(RecordKind::Struct);
        let array = r.fixed_array(a, 2).unwrap();
        r.define_record(a, [array]).unwrap();
        assert!(
            matches!(r.freeze(), Err(TypeError::RecursiveValue { cycle }) if cycle == vec![a, array, a])
        );
    }
    #[test]
    fn only_complete_definitions_become_immutable_types() {
        let mut r = TypeRegistry::new();
        let a = r.reserve_record(RecordKind::Struct);
        assert!(matches!(r.freeze(), Err(TypeError::Incomplete(id)) if id == a));
        let mut r = TypeRegistry::new();
        let a = r.reserve_record(RecordKind::Struct);
        let void = r.void();
        assert_eq!(r.define_record(a, [void]), Err(TypeError::NotAValue(void)));
        r.define_record(a, []).unwrap();
        assert_eq!(r.define_record(a, []), Err(TypeError::AlreadyDefined(a)));
        r.freeze().unwrap();
    }
    #[test]
    fn enum_aliases_preserve_nominal_identity_and_exact_representation() {
        let mut r = TypeRegistry::new();
        let a = r.reserve_enum(IntegerType::U8);
        let value = Integer::checked(IntegerType::U8, 16).unwrap();
        assert!(matches!(
            r.define_enum(a, [Integer::checked(IntegerType::S8, 16).unwrap()]),
            Err(TypeError::EnumRepresentation { .. })
        ));
        r.define_enum(a, [value, value]).unwrap();
        let types = r.freeze().unwrap();
        let TypeKind::Enum(id) = *types.kind(a).unwrap() else {
            panic!()
        };
        assert_eq!(types.enumeration(id).unwrap().values.len(), 2);
    }
    #[test]
    fn procedure_identity_includes_results_abi_and_context() {
        let mut r = TypeRegistry::new();
        let boolean = r.scalar(ScalarType::Bool);
        let signature = ProcedureType {
            parameters: Box::new([boolean]),
            results: Box::new([boolean]),
            return_abi: crate::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::Implicit,
            variadic: Variadic::None,
        };
        let a = r.procedure(signature.clone()).unwrap();
        assert_eq!(a, r.procedure(signature.clone()).unwrap());
        let b = r
            .procedure(ProcedureType {
                context: ContextMode::None,
                ..signature.clone()
            })
            .unwrap();
        let c = r
            .procedure(ProcedureType {
                results: Box::new([]),
                ..signature
            })
            .unwrap();
        assert_ne!(a, b);
        assert_ne!(a, c);
        r.freeze().unwrap();
    }
    #[test]
    fn deep_value_graph_uses_a_worklist() {
        let mut r = TypeRegistry::new();
        let leaf = r.scalar(ScalarType::Bool);
        let records = (0..20_000)
            .map(|_| r.reserve_record(RecordKind::Struct))
            .collect::<Vec<_>>();
        // Forward references force freeze's first record to traverse the entire chain.
        for pair in records.windows(2) {
            r.define_record(pair[0], [pair[1]]).unwrap();
        }
        r.define_record(*records.last().unwrap(), [leaf]).unwrap();
        r.freeze().unwrap();
    }

    #[test]
    fn rejected_definitions_can_be_retried_without_partial_state() {
        let mut r = TypeRegistry::new();
        let other = TypeRegistry::new();
        let byte = r.scalar(ScalarType::Int(IntegerType::U8));
        let foreign = other.scalar(ScalarType::Bool);
        let record = r.reserve_record(RecordKind::Union);
        assert_eq!(
            r.define_record(record, [byte, foreign]),
            Err(TypeError::ForeignType(foreign))
        );
        r.define_record(record, [byte, byte]).unwrap();

        let enumeration = r.reserve_enum(IntegerType::U8);
        let alias = Integer::checked(IntegerType::U8, 7).unwrap();
        let wrong = Integer::checked(IntegerType::U16, 8).unwrap();
        assert_eq!(
            r.define_enum(enumeration, [alias, wrong]),
            Err(TypeError::EnumRepresentation {
                expected: IntegerType::U8,
                actual: IntegerType::U16,
            })
        );
        r.define_enum(enumeration, [alias, alias]).unwrap();
        let types = r.freeze().unwrap();
        let TypeKind::Record(record_id) = *types.kind(record).unwrap() else {
            panic!()
        };
        let definition = types.record(record_id).unwrap();
        assert_eq!(definition.kind, RecordKind::Union);
        assert_eq!(definition.fields.as_ref(), &[byte, byte]);
        let TypeKind::Enum(enum_id) = *types.kind(enumeration).unwrap() else {
            panic!()
        };
        assert_eq!(
            types.enumeration(enum_id).unwrap().values.as_ref(),
            &[alias, alias]
        );
    }

    #[test]
    fn unions_still_require_finite_by_value_members() {
        let mut r = TypeRegistry::new();
        let union = r.reserve_record(RecordKind::Union);
        let record = r.reserve_record(RecordKind::Struct);
        r.define_record(union, [record]).unwrap();
        r.define_record(record, [union]).unwrap();
        assert!(matches!(
            r.freeze(),
            Err(TypeError::RecursiveValue { cycle }) if cycle == vec![union, record, union]
        ));

        let mut r = TypeRegistry::new();
        let union = r.reserve_record(RecordKind::Union);
        let view = r.slice(union).unwrap();
        let dynamic = r.dynamic_array(union).unwrap();
        let procedure = r
            .procedure(ProcedureType {
                parameters: Box::new([union]),
                results: Box::new([union]),
                return_abi: crate::ForeignReturnAbi::Natural,
                convention: CallingConvention::Jai,
                context: ContextMode::Implicit,
                variadic: Variadic::None,
            })
            .unwrap();
        r.define_record(union, [view, dynamic, procedure]).unwrap();
        r.freeze().unwrap();
    }
}

#[cfg(test)]
mod variant_tests {
    use super::*;

    #[test]
    fn universal_reservation_is_lazy_unique_and_pending_until_defined() {
        let mut registry = TypeRegistry::new();
        assert_eq!(registry.any_type(), None);
        let universal = registry.reserve_any();
        assert_eq!(registry.reserve_any(), universal);
        assert_eq!(TypeView::any_type(&registry), Some(universal));
        assert!(
            matches!(registry.record_storage_definition(universal), Err(TypeError::Incomplete(id)) if id == universal)
        );
        assert!(
            matches!(registry.field(universal, 0), Err(TypeError::Incomplete(id)) if id == universal)
        );
        assert!(
            matches!(registry.record_definition(universal), Err(TypeError::WrongKind(id)) if id == universal)
        );
        assert!(
            matches!(registry.procedure_definition(universal), Err(TypeError::WrongKind(id)) if id == universal)
        );
        assert_eq!(
            registry.define_record(universal, []),
            Err(TypeError::WrongKind(universal))
        );
        assert!(matches!(registry.freeze(), Err(TypeError::Incomplete(id)) if id == universal));
        assert_eq!(TypeRegistry::new().freeze().unwrap().any_type(), None);
    }

    #[test]
    fn universal_definition_retries_require_a_ready_local_struct_header() {
        let mut registry = TypeRegistry::new();
        let universal = registry.reserve_any();
        let header = registry.reserve_record(RecordKind::Struct);
        assert_eq!(
            registry.define_any(universal, header),
            Err(TypeError::Incomplete(header))
        );
        let boolean = registry.scalar(ScalarType::Bool);
        assert_eq!(
            registry.define_any(universal, boolean),
            Err(TypeError::WrongKind(boolean))
        );
        let union = registry.reserve_record(RecordKind::Union);
        registry.define_record(union, []).unwrap();
        assert_eq!(
            registry.define_any(universal, union),
            Err(TypeError::WrongKind(union))
        );
        let foreign = TypeRegistry::new().scalar(ScalarType::Bool);
        assert_eq!(
            registry.define_any(universal, foreign),
            Err(TypeError::ForeignType(foreign))
        );
        assert!(
            matches!(registry.record_storage_definition(universal), Err(TypeError::Incomplete(id)) if id == universal)
        );
        registry.define_record(header, []).unwrap();
        registry.define_any(universal, header).unwrap();
        let definition = registry.record_storage_definition(universal).unwrap();
        assert_eq!(definition.kind, RecordKind::Struct);
        assert_eq!(definition.layout, RecordLayout::default());
        assert_eq!(definition.fields.len(), 2);
        assert_eq!(
            registry.kind(definition.fields[0]).unwrap(),
            &TypeKind::Pointer(header)
        );
        assert_eq!(
            registry.kind(definition.fields[1]).unwrap(),
            &TypeKind::Pointer(registry.void())
        );
        assert_eq!(
            registry.define_any(universal, header),
            Err(TypeError::AlreadyDefined(universal))
        );
        let field = registry.field(universal, 0).unwrap();
        let kind = registry.kind(universal).unwrap().clone();
        assert_eq!(registry.lookup(&kind), Some(universal));
        let frozen = registry.freeze().unwrap();
        assert_eq!(frozen.any_type(), Some(universal));
        assert_eq!(frozen.lookup(&kind), Some(universal));
        assert_eq!(frozen.field(universal, 0).unwrap(), field);
        assert_eq!(frozen.field_type(field.id), Ok(field.ty));
    }

    #[test]
    fn universal_pointer_storage_breaks_header_recursion_and_rejects_foreign_fields() {
        let mut registry = TypeRegistry::new();
        let universal = registry.reserve_any();
        let header = registry.reserve_record(RecordKind::Struct);
        registry.define_record(header, [universal]).unwrap();
        registry.define_any(universal, header).unwrap();
        let descriptor = registry.field(universal, 0).unwrap();
        let mut foreign = TypeRegistry::new();
        let foreign_any = foreign.reserve_any();
        let foreign_header = foreign.reserve_record(RecordKind::Struct);
        foreign.define_record(foreign_header, []).unwrap();
        foreign.define_any(foreign_any, foreign_header).unwrap();
        let foreign_field = foreign.field(foreign_any, 0).unwrap();
        assert_eq!(
            registry.field_type(foreign_field.id),
            Err(TypeError::ForeignRecord(foreign_field.id.record()))
        );
        assert_eq!(registry.lookup(foreign.kind(foreign_any).unwrap()), None);
        let frozen = registry.freeze().unwrap();
        assert_eq!(
            frozen.validate_field(universal, descriptor.id),
            Ok(descriptor.ty)
        );
        assert_eq!(
            frozen.validate_field(header, descriptor.id),
            Err(TypeError::FieldOwner {
                record: header,
                field: descriptor.id
            })
        );
        assert_eq!(
            frozen.field_type(foreign_field.id),
            Err(TypeError::ForeignRecord(foreign_field.id.record()))
        );
    }

    #[test]
    fn structural_lookup_survives_freeze_without_accepting_foreign_constituents() {
        let mut registry = TypeRegistry::new();
        let boolean = registry.scalar(ScalarType::Bool);
        let pointer = registry.pointer(boolean).unwrap();
        let array = registry.fixed_array(boolean, 7).unwrap();
        let record = registry.reserve_record(RecordKind::Struct);
        registry.define_record(record, [boolean]).unwrap();
        let record_kind = registry.kind(record).unwrap().clone();
        let foreign = TypeRegistry::new().scalar(ScalarType::Bool);
        let inspect = |view: &dyn TypeView| {
            assert_eq!(view.lookup(&TypeKind::Pointer(boolean)), Some(pointer));
            assert_eq!(
                view.lookup(&TypeKind::FixedArray {
                    element: boolean,
                    count: 7
                }),
                Some(array)
            );
            assert_eq!(
                view.lookup(&TypeKind::FixedArray {
                    element: boolean,
                    count: 8
                }),
                None
            );
            assert_eq!(view.lookup(&TypeKind::Pointer(foreign)), None);
            assert_eq!(
                view.lookup(&TypeKind::FixedArray {
                    element: foreign,
                    count: 7
                }),
                None
            );
            assert_eq!(view.lookup(&record_kind), None);
            assert_eq!(view.lookup(&TypeKind::Bool), Some(boolean));
        };
        inspect(&registry);
        inspect(&registry.freeze().unwrap());
    }

    #[test]
    fn distinct_variants_preserve_nominal_identity_mode_and_representation() {
        let mut registry = TypeRegistry::new();
        let base = registry.scalar(ScalarType::Int(IntegerType::U32));
        let strict = registry.reserve_distinct(DistinctKind::Distinct);
        let same_shape = registry.reserve_distinct(DistinctKind::Distinct);
        let is_a = registry.reserve_distinct(DistinctKind::IsA);
        assert!(
            matches!(registry.distinct_definition(strict), Err(TypeError::Incomplete(id)) if id == strict)
        );
        let other = TypeRegistry::new();
        let foreign = other.scalar(ScalarType::Bool);
        assert_eq!(
            registry.define_distinct(strict, foreign),
            Err(TypeError::ForeignType(foreign))
        );
        let void = registry.void();
        assert_eq!(
            registry.define_distinct(strict, void),
            Err(TypeError::NotAValue(void))
        );
        registry.define_distinct(strict, base).unwrap();
        registry.define_distinct(same_shape, base).unwrap();
        registry.define_distinct(is_a, strict).unwrap();
        assert_eq!(
            registry.define_distinct(strict, base),
            Err(TypeError::AlreadyDefined(strict))
        );
        assert_ne!(strict, same_shape);
        assert_ne!(strict, base);
        assert_eq!(
            registry.distinct_definition(is_a).unwrap().kind,
            DistinctKind::IsA
        );
        assert_eq!(
            registry.distinct_definition(is_a).unwrap().representation,
            strict
        );
        let TypeKind::Distinct(id) = *registry.kind(strict).unwrap() else {
            panic!()
        };
        assert!(
            matches!(TypeView::distinct(&other, id), Err(TypeError::ForeignDistinct(foreign_id)) if foreign_id == id)
        );
        let types = registry.freeze().unwrap();
        assert_eq!(types.distinct(id).unwrap().kind, DistinctKind::Distinct);
        assert_eq!(types.distinct(id).unwrap().representation, base);
        assert_eq!(
            types.distinct_definition(is_a).unwrap().representation,
            strict
        );
    }

    #[test]
    fn distinct_representation_cycles_are_values_but_pointer_wrappers_are_finite() {
        let mut registry = TypeRegistry::new();
        let pending = registry.reserve_distinct(DistinctKind::Distinct);
        assert!(matches!(registry.freeze(), Err(TypeError::Incomplete(id)) if id == pending));
        let mut registry = TypeRegistry::new();
        let wrapper = registry.reserve_distinct(DistinctKind::Distinct);
        let record = registry.reserve_record(RecordKind::Struct);
        registry.define_distinct(wrapper, record).unwrap();
        registry.define_record(record, [wrapper]).unwrap();
        assert!(
            matches!(registry.freeze(), Err(TypeError::RecursiveValue {cycle}) if cycle == vec![wrapper,record,wrapper])
        );
        let mut registry = TypeRegistry::new();
        let wrapper = registry.reserve_distinct(DistinctKind::Distinct);
        let pointer = registry.pointer(wrapper).unwrap();
        registry.define_distinct(wrapper, pointer).unwrap();
        registry.freeze().unwrap();
    }

    #[test]
    fn deep_forward_distinct_representation_graph_is_iterative() {
        let mut registry = TypeRegistry::new();
        let base = registry.scalar(ScalarType::Bool);
        let wrappers = (0..10_000)
            .map(|_| registry.reserve_distinct(DistinctKind::IsA))
            .collect::<Vec<_>>();
        for pair in wrappers.windows(2) {
            registry.define_distinct(pair[0], pair[1]).unwrap();
        }
        registry
            .define_distinct(*wrappers.last().unwrap(), base)
            .unwrap();
        registry.freeze().unwrap();
    }

    #[test]
    fn variadic_signatures_have_separate_identity_and_validate_pack_slots() {
        let mut registry = TypeRegistry::new();
        let byte = registry.scalar(ScalarType::Int(IntegerType::U8));
        let boolean = registry.scalar(ScalarType::Bool);
        let pack = registry.slice(byte).unwrap();
        let jai = ProcedureType {
            parameters: Box::new([byte, pack, boolean]),
            results: Box::new([]),
            return_abi: crate::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::Implicit,
            variadic: Variadic::Jai {
                parameter: 1,
                element: byte,
            },
        };
        let variadic = registry.procedure(jai.clone()).unwrap();
        assert_eq!(registry.procedure(jai.clone()).unwrap(), variadic);
        assert_ne!(
            registry
                .procedure(ProcedureType {
                    variadic: Variadic::None,
                    ..jai.clone()
                })
                .unwrap(),
            variadic
        );
        for (metadata, issue) in [
            (
                Variadic::Jai {
                    parameter: 3,
                    element: byte,
                },
                VariadicIssue::ParameterOutOfBounds,
            ),
            (
                Variadic::Jai {
                    parameter: 0,
                    element: byte,
                },
                VariadicIssue::PackType,
            ),
            (
                Variadic::Jai {
                    parameter: 1,
                    element: boolean,
                },
                VariadicIssue::PackType,
            ),
            (
                Variadic::C {
                    fixed_parameters: 3,
                },
                VariadicIssue::CallingConvention,
            ),
        ] {
            let count = registry.kinds.len();
            assert_eq!(
                registry.procedure(ProcedureType {
                    variadic: metadata,
                    ..jai.clone()
                }),
                Err(TypeError::InvalidVariadic {
                    variadic: metadata,
                    issue
                })
            );
            assert_eq!(registry.kinds.len(), count);
        }
        let c = ProcedureType {
            parameters: Box::new([byte]),
            results: Box::new([]),
            return_abi: crate::ForeignReturnAbi::Natural,
            convention: CallingConvention::C,
            context: ContextMode::None,
            variadic: Variadic::C {
                fixed_parameters: 1,
            },
        };
        let c_variadic = registry.procedure(c.clone()).unwrap();
        let ordinary_c = registry
            .procedure(ProcedureType {
                variadic: Variadic::None,
                ..c.clone()
            })
            .unwrap();
        assert_ne!(c_variadic, ordinary_c);
        let wrong = Variadic::C {
            fixed_parameters: 0,
        };
        assert_eq!(
            registry.procedure(ProcedureType {
                variadic: wrong,
                ..c
            }),
            Err(TypeError::InvalidVariadic {
                variadic: wrong,
                issue: VariadicIssue::FixedParameterCount
            })
        );
        let foreign = TypeRegistry::new().scalar(ScalarType::Bool);
        assert_eq!(
            registry.procedure(ProcedureType {
                variadic: Variadic::Jai {
                    parameter: 1,
                    element: foreign
                },
                ..jai
            }),
            Err(TypeError::ForeignType(foreign))
        );
        let types = registry.freeze().unwrap();
        assert_eq!(
            types.procedure_definition(variadic).unwrap().variadic,
            Variadic::Jai {
                parameter: 1,
                element: byte
            }
        );
        assert_eq!(
            types
                .procedure_definition(c_variadic)
                .unwrap()
                .parameters
                .as_ref(),
            &[byte]
        );
    }
}
