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
identity!(ProcedureTypeId);

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
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ContextMode {
    Implicit,
    None,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Void,
    Type,
    Bool,
    Integer(IntegerType),
    Float(FloatType),
    String,
    Pointer(TypeId),
    FixedArray { element: TypeId, count: u64 },
    Slice(TypeId),
    DynamicArray(TypeId),
    Procedure(ProcedureTypeId),
    Record(RecordId),
    Enum(EnumId),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProcedureType {
    pub parameters: Box<[TypeId]>,
    pub results: Box<[TypeId]>,
    pub convention: CallingConvention,
    pub context: ContextMode,
}
#[derive(Debug)]
pub struct RecordDefinition {
    pub kind: RecordKind,
    pub fields: Box<[TypeId]>,
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

#[derive(Debug, PartialEq, Eq)]
pub enum TypeError {
    ForeignType(TypeId),
    ForeignRecord(RecordId),
    ForeignEnum(EnumId),
    ForeignProcedure(ProcedureTypeId),
    WrongKind(TypeId),
    NotAValue(TypeId),
    AlreadyDefined(TypeId),
    Incomplete(TypeId),
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
            Self::ForeignType(_)
            | Self::ForeignRecord(_)
            | Self::ForeignEnum(_)
            | Self::ForeignProcedure(_) => f.write_str("type identity belongs to another registry"),
            Self::WrongKind(_) => f.write_str("type definition has the wrong kind"),
            Self::NotAValue(_) => f.write_str("void and type values cannot occupy runtime storage"),
            Self::AlreadyDefined(_) => f.write_str("nominal type has already been defined"),
            Self::Incomplete(_) => f.write_str("nominal type definition is incomplete"),
            Self::EnumRepresentation { .. } => {
                f.write_str("enum value has the wrong integer representation")
            }
            Self::RecursiveValue { .. } => f.write_str("type contains itself by value"),
        }
    }
}
impl std::error::Error for TypeError {}

pub struct TypeRegistry {
    arena: u64,
    kinds: Vec<TypeKind>,
    canonical: HashMap<TypeKind, TypeId>,
    records: Vec<Definition<RecordDefinition, RecordKind>>,
    enums: Vec<Definition<EnumDefinition, IntegerType>>,
    procedures: Vec<ProcedureType>,
    procedure_ids: HashMap<ProcedureType, ProcedureTypeId>,
}

#[derive(Debug)]
pub struct Types {
    arena: u64,
    kinds: Vec<TypeKind>,
    boolean: TypeId,
    integers: [TypeId; 8],
    records: Vec<RecordDefinition>,
    enums: Vec<EnumDefinition>,
    procedures: Vec<ProcedureType>,
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
impl TypeRegistry {
    pub fn new() -> Self {
        let arena = NEXT_ARENA
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .expect("type registry identity space exhausted");
        let mut registry = Self {
            arena,
            kinds: vec![],
            canonical: HashMap::new(),
            records: vec![],
            enums: vec![],
            procedures: vec![],
            procedure_ids: HashMap::new(),
        };
        for kind in [
            TypeKind::Void,
            TypeKind::Type,
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
    fn value(&self, id: TypeId) -> Result<(), TypeError> {
        match self.kind(id)? {
            TypeKind::Void | TypeKind::Type => Err(TypeError::NotAValue(id)),
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
    pub fn pointer(&mut self, pointee: TypeId) -> Result<TypeId, TypeError> {
        self.kind(pointee)?;
        Ok(self.intern(TypeKind::Pointer(pointee)))
    }
    pub fn fixed_array(&mut self, element: TypeId, count: u64) -> Result<TypeId, TypeError> {
        self.value(element)?;
        Ok(self.intern(TypeKind::FixedArray { element, count }))
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
        self.push(TypeKind::Record(id))
    }
    pub fn define_record(
        &mut self,
        ty: TypeId,
        fields: impl Into<Box<[TypeId]>>,
    ) -> Result<(), TypeError> {
        let TypeKind::Record(id) = *self.kind(ty)? else {
            return Err(TypeError::WrongKind(ty));
        };
        let fields = fields.into();
        for &field in &fields {
            self.value(field)?;
        }
        let definition = &mut self.records[id.index];
        let Definition::Reserved(kind) = definition else {
            return Err(TypeError::AlreadyDefined(ty));
        };
        *definition = Definition::Defined(RecordDefinition {
            kind: *kind,
            fields,
        });
        Ok(())
    }
    pub fn reserve_enum(&mut self, representation: IntegerType) -> TypeId {
        let id = EnumId {
            arena: self.arena,
            index: self.enums.len(),
        };
        self.enums.push(Definition::Reserved(representation));
        self.push(TypeKind::Enum(id))
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
                TypeKind::Record(id) => matches!(self.records[id.index], Definition::Reserved(_)),
                TypeKind::Enum(id) => matches!(self.enums[id.index], Definition::Reserved(_)),
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
        let types = Types {
            arena: self.arena,
            kinds: self.kinds,
            boolean,
            integers,
            records: self
                .records
                .into_iter()
                .map(|r| match r {
                    Definition::Defined(r) => r,
                    Definition::Reserved(_) => unreachable!(),
                })
                .collect(),
            enums: self
                .enums
                .into_iter()
                .map(|r| match r {
                    Definition::Defined(r) => r,
                    Definition::Reserved(_) => unreachable!(),
                })
                .collect(),
            procedures: self.procedures,
        };
        types.check_value_cycles()?;
        Ok(types)
    }
}
impl Types {
    /// Retrieves the same builtin identity reserved by the mutable registry.
    pub fn scalar(&self, ty: ScalarType) -> TypeId {
        match ty {
            ScalarType::Bool => self.boolean,
            ScalarType::Int(ty) => {
                self.integers[INTEGERS.iter().position(|&integer| integer == ty).unwrap()]
            }
        }
    }
    pub fn kind(&self, id: TypeId) -> Result<&TypeKind, TypeError> {
        if id.arena != self.arena {
            return Err(TypeError::ForeignType(id));
        }
        self.kinds.get(id.index).ok_or(TypeError::ForeignType(id))
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
            TypeKind::Record(record) => &self.records[record.index].fields,
            TypeKind::FixedArray { element, .. } => std::slice::from_ref(element),
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
                            return Err(TypeError::RecursiveValue { cycle });
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
        let types = r.freeze().unwrap();
        assert_eq!(types.scalar(ScalarType::Bool), boolean);
        assert_eq!(types.kind(boolean).unwrap(), &TypeKind::Bool);
        for (ty, id) in integers {
            assert_eq!(types.scalar(ScalarType::Int(ty)), id);
            assert_eq!(types.kind(id).unwrap(), &TypeKind::Integer(ty));
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
            convention: CallingConvention::Jai,
            context: ContextMode::Implicit,
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
                convention: CallingConvention::Jai,
                context: ContextMode::Implicit,
            })
            .unwrap();
        r.define_record(union, [view, dynamic, procedure]).unwrap();
        r.freeze().unwrap();
    }
}
