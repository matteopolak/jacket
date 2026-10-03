use crate::{
    CheckMode, GlobalId, IntExpr, IrError, LocalId, ProcedureId, SequenceField, ValueExpr,
};
use jai_types::{
    FieldId, FloatValue, Integer, IntegerType, ScalarType, TypeId, TypeKind, TypeView,
};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlaceKind {
    Context(TypeId),
    Local(LocalId),
    Global(GlobalId),
    Field(ProjectionId),
    Dereference(DereferenceId),
    Index(IndexId),
    SequenceField(SequenceProjectionId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Place {
    kind: PlaceKind,
    ty: TypeId,
}
impl Place {
    pub fn context(record_type: TypeId, types: &dyn TypeView) -> Result<Self, IrError> {
        types.record_definition(record_type)?;
        Ok(Self {
            kind: PlaceKind::Context(record_type),
            ty: record_type,
        })
    }
    pub fn kind(self) -> PlaceKind {
        self.kind
    }
    pub fn ty(self) -> TypeId {
        self.ty
    }
}

static NEXT_PLACES: AtomicU64 = AtomicU64::new(1);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProjectionId {
    arena: u64,
    index: usize,
}
impl ProjectionId {
    pub fn index(self) -> usize {
        self.index
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlaceProjection {
    pub base: Place,
    pub field: FieldId,
}
macro_rules! place_id {
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
place_id!(DereferenceId);
place_id!(IndexId);
place_id!(SequenceProjectionId);
#[derive(Clone, Debug)]
pub struct DereferenceProjection {
    pub pointer: ValueExpr,
}
#[derive(Clone, Debug)]
pub struct IndexProjection {
    pub base: Place,
    pub index: IntExpr,
    pub check: CheckMode,
}
#[derive(Clone, Copy, Debug)]
pub struct SequenceProjection {
    pub base: Place,
    pub field: SequenceField,
}

/// Mutable field-place interning. Projections retain their nominal field owner.
#[derive(Debug)]
pub struct PlaceRegistry {
    arena: u64,
    projections: Vec<PlaceProjection>,
    canonical: HashMap<PlaceProjection, ProjectionId>,
    dereferences: Vec<DereferenceProjection>,
    indices: Vec<IndexProjection>,
    sequence_fields: Vec<SequenceProjection>,
}
impl Default for PlaceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
impl PlaceRegistry {
    pub fn new() -> Self {
        let arena = NEXT_PLACES
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .expect("place registry identity space exhausted");
        Self {
            arena,
            projections: vec![],
            canonical: HashMap::new(),
            dereferences: vec![],
            indices: vec![],
            sequence_fields: vec![],
        }
    }
    pub fn field(
        &mut self,
        base: Place,
        field: FieldId,
        types: &dyn TypeView,
    ) -> Result<Place, IrError> {
        self.owns(base)?;
        let ty = types.validate_field(base.ty, field)?;
        let projection = PlaceProjection {
            base,
            field,
        };
        let id = if let Some(&id) = self.canonical.get(&projection) {
            id
        } else {
            let id = ProjectionId {
                arena: self.arena,
                index: self.projections.len(),
            };
            self.projections.push(projection);
            self.canonical.insert(projection, id);
            id
        };
        Ok(Place {
            kind: PlaceKind::Field(id),
            ty,
        })
    }
    pub fn projection(&self, id: ProjectionId) -> Result<&PlaceProjection, IrError> {
        if id.arena != self.arena {
            return Err(IrError::ForeignProjection(id));
        }
        self.projections
            .get(id.index)
            .ok_or(IrError::ForeignProjection(id))
    }
    pub fn dereference(
        &mut self,
        pointer: ValueExpr,
        types: &dyn TypeView,
    ) -> Result<Place, IrError> {
        let pointer_ty = pointer.type_id(types);
        let TypeKind::Pointer(ty) = *types.kind(pointer_ty)? else {
            return Err(IrError::InvalidValue(pointer_ty));
        };
        runtime_type(types, ty)?;
        let id = DereferenceId {
            arena: self.arena,
            index: self.dereferences.len(),
        };
        self.dereferences.push(DereferenceProjection {
            pointer,
        });
        Ok(Place {
            kind: PlaceKind::Dereference(id),
            ty,
        })
    }
    pub fn index(
        &mut self,
        base: Place,
        index: IntExpr,
        types: &dyn TypeView,
    ) -> Result<Place, IrError> {
        self.index_with_check(base, index, CheckMode::Enabled, types)
    }
    pub fn index_with_check(
        &mut self,
        base: Place,
        index: IntExpr,
        check: CheckMode,
        types: &dyn TypeView,
    ) -> Result<Place, IrError> {
        self.owns(base)?;
        if crate::canonical_index_type(index.ty()) != index.ty() {
            return Err(IrError::IntegerMismatch {
                expected: IntegerType::S64,
                actual: index.ty(),
            });
        }
        let ty = crate::sequences::element(types, base.ty)?;
        runtime_type(types, ty)?;
        let id = IndexId {
            arena: self.arena,
            index: self.indices.len(),
        };
        self.indices.push(IndexProjection {
            base,
            index,
            check,
        });
        Ok(Place {
            kind: PlaceKind::Index(id),
            ty,
        })
    }
    pub fn sequence_field(
        &mut self,
        base: Place,
        field: SequenceField,
        types: &dyn TypeView,
    ) -> Result<Place, IrError> {
        self.owns(base)?;
        if !matches!(
            types.kind(base.ty)?,
            TypeKind::Slice(_) | TypeKind::DynamicArray(_) | TypeKind::String
        ) {
            return Err(IrError::InvalidValue(base.ty));
        }
        let ty = crate::sequences::field_type(types, base.ty, field)?;
        let id = SequenceProjectionId {
            arena: self.arena,
            index: self.sequence_fields.len(),
        };
        self.sequence_fields.push(SequenceProjection {
            base,
            field,
        });
        Ok(Place {
            kind: PlaceKind::SequenceField(id),
            ty,
        })
    }
    fn owns(&self, place: Place) -> Result<(), IrError> {
        match place.kind {
            PlaceKind::Context(_) | PlaceKind::Local(_) | PlaceKind::Global(_) => {}
            PlaceKind::Field(id) => {
                self.projection(id)?;
            }
            PlaceKind::Dereference(id) => {
                self.dereference_projection(id)?;
            }
            PlaceKind::Index(id) => {
                self.index_projection(id)?;
            }
            PlaceKind::SequenceField(id) => {
                self.sequence_projection(id)?;
            }
        }
        Ok(())
    }
    pub fn dereference_projection(
        &self,
        id: DereferenceId,
    ) -> Result<&DereferenceProjection, IrError> {
        if id.arena != self.arena {
            return Err(IrError::ForeignPlaceProjection {
                kind: "dereference",
                index: id.index,
            });
        }
        self.dereferences
            .get(id.index)
            .ok_or(IrError::ForeignPlaceProjection {
                kind: "dereference",
                index: id.index,
            })
    }
    pub fn index_projection(&self, id: IndexId) -> Result<&IndexProjection, IrError> {
        if id.arena != self.arena {
            return Err(IrError::ForeignPlaceProjection {
                kind: "index",
                index: id.index,
            });
        }
        self.indices
            .get(id.index)
            .ok_or(IrError::ForeignPlaceProjection {
                kind: "index",
                index: id.index,
            })
    }
    pub fn sequence_projection(
        &self,
        id: SequenceProjectionId,
    ) -> Result<&SequenceProjection, IrError> {
        if id.arena != self.arena {
            return Err(IrError::ForeignPlaceProjection {
                kind: "sequence field",
                index: id.index,
            });
        }
        self.sequence_fields
            .get(id.index)
            .ok_or(IrError::ForeignPlaceProjection {
                kind: "sequence field",
                index: id.index,
            })
    }
    pub fn snapshot(&self) -> Places {
        Places {
            arena: self.arena,
            projections: self.projections.clone(),
            dereferences: self.dereferences.clone(),
            indices: self.indices.clone(),
            sequence_fields: self.sequence_fields.clone(),
        }
    }
    pub fn freeze(self) -> Places {
        Places {
            arena: self.arena,
            projections: self.projections,
            dereferences: self.dereferences,
            indices: self.indices,
            sequence_fields: self.sequence_fields,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Places {
    arena: u64,
    projections: Vec<PlaceProjection>,
    dereferences: Vec<DereferenceProjection>,
    indices: Vec<IndexProjection>,
    sequence_fields: Vec<SequenceProjection>,
}
impl Default for Places {
    fn default() -> Self {
        PlaceRegistry::new().freeze()
    }
}
impl Places {
    pub(crate) fn drain_operands(&mut self, pending: &mut Vec<crate::disposal::Work>) {
        pending.extend(
            std::mem::take(&mut self.dereferences)
                .into_iter()
                .map(|projection| crate::disposal::Work::Value(projection.pointer)),
        );
        pending.extend(
            std::mem::take(&mut self.indices)
                .into_iter()
                .map(|projection| crate::disposal::Work::Int(projection.index)),
        );
    }
    pub fn projection(&self, id: ProjectionId) -> Result<&PlaceProjection, IrError> {
        if id.arena != self.arena {
            return Err(IrError::ForeignProjection(id));
        }
        self.projections
            .get(id.index)
            .ok_or(IrError::ForeignProjection(id))
    }
    pub fn dereference(&self, id: DereferenceId) -> Result<&DereferenceProjection, IrError> {
        if id.arena != self.arena {
            return Err(IrError::ForeignPlaceProjection {
                kind: "dereference",
                index: id.index,
            });
        }
        self.dereferences
            .get(id.index)
            .ok_or(IrError::ForeignPlaceProjection {
                kind: "dereference",
                index: id.index,
            })
    }
    pub fn index(&self, id: IndexId) -> Result<&IndexProjection, IrError> {
        if id.arena != self.arena {
            return Err(IrError::ForeignPlaceProjection {
                kind: "index",
                index: id.index,
            });
        }
        self.indices
            .get(id.index)
            .ok_or(IrError::ForeignPlaceProjection {
                kind: "index",
                index: id.index,
            })
    }
    pub fn sequence_field(&self, id: SequenceProjectionId) -> Result<&SequenceProjection, IrError> {
        if id.arena != self.arena {
            return Err(IrError::ForeignPlaceProjection {
                kind: "sequence field",
                index: id.index,
            });
        }
        self.sequence_fields
            .get(id.index)
            .ok_or(IrError::ForeignPlaceProjection {
                kind: "sequence field",
                index: id.index,
            })
    }
    pub fn len(&self) -> usize {
        self.projections.len()
            + self.dereferences.len()
            + self.indices.len()
            + self.sequence_fields.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
// Objects within an arena are append-only and cannot be mutated through a snapshot.
impl PartialEq for Places {
    fn eq(&self, other: &Self) -> bool {
        self.arena == other.arena
            && self.projections.len() == other.projections.len()
            && self.dereferences.len() == other.dereferences.len()
            && self.indices.len() == other.indices.len()
            && self.sequence_fields.len() == other.sequence_fields.len()
    }
}
impl Eq for Places {
}

#[derive(Clone, Copy, Debug)]
pub struct Local {
    id: LocalId,
    ty: TypeId,
}
impl Local {
    #[doc(hidden)]
    pub fn new(
        procedure: ProcedureId,
        index: usize,
        domain: ScalarType,
        types: &dyn TypeView,
    ) -> Self {
        Self {
            id: LocalId {
                procedure,
                index,
            },
            ty: types.scalar(domain),
        }
    }
    pub fn new_typed(
        procedure: ProcedureId,
        index: usize,
        ty: TypeId,
        types: &dyn TypeView,
    ) -> Result<Self, IrError> {
        runtime_type(types, ty)?;
        Ok(Self {
            id: LocalId {
                procedure,
                index,
            },
            ty,
        })
    }
    pub fn id(self) -> LocalId {
        self.id
    }
    pub fn ty(self) -> TypeId {
        self.ty
    }
    pub fn place(self) -> Place {
        Place {
            kind: PlaceKind::Local(self.id),
            ty: self.ty,
        }
    }
    pub fn integer(self, types: &dyn TypeView) -> Option<IntLocal> {
        match types.kind(self.ty).ok()? {
            TypeKind::Integer(ty) => Some(IntLocal {
                local: self,
                ty: *ty,
            }),
            _ => None,
        }
    }
    pub fn boolean(self, types: &dyn TypeView) -> Option<BoolLocal> {
        matches!(types.kind(self.ty), Ok(TypeKind::Bool)).then_some(BoolLocal(self))
    }
}
#[derive(Clone, Copy, Debug)]
pub struct IntLocal {
    local: Local,
    ty: IntegerType,
}
impl IntLocal {
    pub fn index(self) -> usize {
        self.local.id.index()
    }
    pub fn ty(self) -> IntegerType {
        self.ty
    }
    pub fn local(self) -> Local {
        self.local
    }
    pub fn place(self) -> IntPlace {
        IntPlace {
            place: self.local.place(),
            ty: self.ty,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct BoolLocal(Local);
impl BoolLocal {
    pub fn index(self) -> usize {
        self.0.id.index()
    }
    pub fn local(self) -> Local {
        self.0
    }
    pub fn place(self) -> BoolPlace {
        BoolPlace(self.0.place())
    }
}
#[derive(Clone, Copy, Debug)]
pub struct IntPlace {
    place: Place,
    ty: IntegerType,
}
impl IntPlace {
    pub fn try_from_place(place: Place, types: &dyn TypeView) -> Result<Self, IrError> {
        let TypeKind::Integer(ty) = types.kind(place.ty)? else {
            return Err(IrError::InvalidValue(place.ty));
        };
        Ok(Self {
            place,
            ty: *ty,
        })
    }
    pub fn ty(self) -> IntegerType {
        self.ty
    }
    pub fn place(self) -> Place {
        self.place
    }
}
#[derive(Clone, Copy, Debug)]
pub struct BoolPlace(Place);
impl BoolPlace {
    pub fn try_from_place(place: Place, types: &dyn TypeView) -> Result<Self, IrError> {
        if !matches!(types.kind(place.ty)?, TypeKind::Bool) {
            return Err(IrError::InvalidValue(place.ty));
        }
        Ok(Self(place))
    }
    pub fn place(self) -> Place {
        self.0
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Storage {
    Int(IntPlace),
    Bool(BoolPlace),
    Value(Place),
}
impl Storage {
    pub fn place(self) -> Place {
        match self {
            Self::Int(p) => p.place(),
            Self::Bool(p) => p.place(),
            Self::Value(p) => p,
        }
    }
    pub fn from_place(place: Place, types: &dyn TypeView) -> Result<Self, IrError> {
        Ok(match types.kind(place.ty)? {
            TypeKind::Integer(_) => Self::Int(IntPlace::try_from_place(place, types)?),
            TypeKind::Bool => Self::Bool(BoolPlace::try_from_place(place, types)?),
            TypeKind::Type => {
                jai_types::RuntimeTypeSchema::from_view(types)?;
                Self::Value(place)
            }
            TypeKind::Void | TypeKind::Code => {
                return Err(IrError::InvalidValue(place.ty));
            }
            _ => Self::Value(place),
        })
    }
    #[doc(hidden)]
    pub fn local(local: Local, types: &dyn TypeView) -> Self {
        Self::from_place(local.place(), types).expect("local type has runtime storage")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConstantValue {
    pub ty: TypeId,
    pub kind: ConstantKind,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConstantKind {
    /// Target-normalized address bits; does not grant VM allocation or code provenance.
    NativePointer(crate::NativePointerConstant),
    RuntimeType(crate::RuntimeTypeConstant),
    Int(Integer),
    Float(FloatValue),
    Bool(bool),
    Record(Vec<ConstantValue>),
    Union {
        field: FieldId,
        value: Box<ConstantValue>,
    },
    Array(Vec<ConstantValue>),
    StringBytes(Vec<u8>),
    Distinct(Box<ConstantValue>),
    Enum(Integer),
    /// A stable checked procedure identity. The enclosing type is its exact ABI signature.
    Procedure(ProcedureId),
    Zero,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GlobalInitializer {
    Int(Integer),
    Bool(bool),
    Value(ConstantValue),
    External(crate::ExternalData),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Global {
    id: GlobalId,
    ty: TypeId,
    initializer: GlobalInitializer,
}
impl Global {
    #[doc(hidden)]
    pub fn new(index: usize, initializer: GlobalInitializer, types: &dyn TypeView) -> Self {
        let ty = match &initializer {
            GlobalInitializer::Int(n) => types.scalar(ScalarType::Int(n.ty())),
            GlobalInitializer::Bool(_) => types.scalar(ScalarType::Bool),
            GlobalInitializer::Value(value) => value.ty,
            GlobalInitializer::External(data) => data.ty(),
        };
        Self {
            id: GlobalId::new(index),
            ty,
            initializer,
        }
    }
    pub fn new_typed(
        index: usize,
        initializer: ConstantValue,
        types: &dyn TypeView,
    ) -> Result<Self, IrError> {
        if let Err(error) = crate::verify::constant(types, &initializer) {
            crate::disposal::constant(initializer);
            return Err(error);
        }
        Ok(Self::new(
            index,
            GlobalInitializer::Value(initializer),
            types,
        ))
    }
    /// External storage is already checked and has no fabricated initial value.
    pub fn new_external(index: usize, data: crate::ExternalData) -> Self {
        Self {
            id: GlobalId::new(index),
            ty: data.ty(),
            initializer: GlobalInitializer::External(data),
        }
    }
    pub fn id(&self) -> GlobalId {
        self.id
    }
    pub fn ty(&self) -> TypeId {
        self.ty
    }
    pub fn initializer(&self) -> &GlobalInitializer {
        &self.initializer
    }
    pub(crate) fn into_initializer(self) -> GlobalInitializer {
        self.initializer
    }
    pub fn place(&self) -> Place {
        Place {
            kind: PlaceKind::Global(self.id),
            ty: self.ty,
        }
    }
    #[doc(hidden)]
    pub fn storage(&self) -> Storage {
        match &self.initializer {
            GlobalInitializer::Int(value) => Storage::Int(IntPlace {
                place: self.place(),
                ty: value.ty(),
            }),
            GlobalInitializer::Bool(_) => Storage::Bool(BoolPlace(self.place())),
            GlobalInitializer::Value(value) => match value.kind {
                ConstantKind::Int(integer) => Storage::Int(IntPlace {
                    place: self.place(),
                    ty: integer.ty(),
                }),
                ConstantKind::Bool(_) => Storage::Bool(BoolPlace(self.place())),
                _ => Storage::Value(self.place()),
            },
            GlobalInitializer::External(data) => match data.scalar() {
                Some(ScalarType::Int(ty)) => Storage::Int(IntPlace {
                    place: self.place(),
                    ty,
                }),
                Some(ScalarType::Bool) => Storage::Bool(BoolPlace(self.place())),
                None => Storage::Value(self.place()),
            },
        }
    }
}
pub(crate) fn runtime_type(types: &dyn TypeView, ty: TypeId) -> Result<(), IrError> {
    let mut heights: HashMap<TypeId, usize> = HashMap::new();
    let mut active = Vec::new();
    let mut pending = vec![(ty, false)];
    while let Some((ty, finish)) = pending.pop() {
        if finish {
            let child_height = match *types.kind(ty)? {
                TypeKind::Record(_) | TypeKind::Any(_) => types
                    .record_storage_definition(ty)?
                    .fields
                    .iter()
                    .map(|field| heights[field])
                    .max()
                    .unwrap_or(0),
                TypeKind::Distinct(id) => heights[&types.distinct(id)?.representation],
                TypeKind::FixedArray {
                    element, ..
                } => heights[&element],
                _ => 0,
            };
            let height = child_height + 1;
            active.pop();
            if active.len() + height > 256 {
                return Err(IrError::VerificationDepth);
            }
            heights.insert(ty, height);
            continue;
        }
        if let Some(&height) = heights.get(&ty) {
            if active.len() + height > 256 {
                return Err(IrError::VerificationDepth);
            }
            continue;
        }
        if active.len() >= 256 {
            return Err(IrError::VerificationDepth);
        }
        if let Some(index) = active.iter().position(|active| *active == ty) {
            let mut cycle = active[index..].to_vec();
            cycle.push(ty);
            return Err(jai_types::TypeError::RecursiveValue {
                cycle,
            }
            .into());
        }
        active.push(ty);
        pending.push((ty, true));
        match *types.kind(ty)? {
            TypeKind::Type => {
                jai_types::RuntimeTypeSchema::from_view(types)?;
            }
            TypeKind::Void | TypeKind::Code => {
                return Err(IrError::InvalidValue(ty));
            }
            TypeKind::Record(_) | TypeKind::Any(_) => {
                for &field in types.record_storage_definition(ty)?.fields.iter().rev() {
                    pending.push((field, false));
                }
            }
            TypeKind::Distinct(id) => pending.push((types.distinct(id)?.representation, false)),
            TypeKind::FixedArray {
                element, ..
            } => pending.push((element, false)),
            TypeKind::Enum(id) => {
                types.enumeration(id)?;
            }
            TypeKind::Procedure(id) => {
                types.procedure_type(id)?;
            }
            TypeKind::Pointer(element)
            | TypeKind::Slice(element)
            | TypeKind::DynamicArray(element) => {
                types.kind(element)?;
            }
            TypeKind::Bool | TypeKind::Integer(_) | TypeKind::Float(_) | TypeKind::String => {}
        }
    }
    Ok(())
}

pub(crate) fn nonzero_type(types: &dyn TypeView, ty: TypeId) -> Result<bool, IrError> {
    runtime_type(types, ty)?;
    let mut sizes = HashMap::new();
    let mut pending = vec![(ty, false)];
    while let Some((ty, finish)) = pending.pop() {
        if sizes.contains_key(&ty) {
            continue;
        }
        let kind = types.kind(ty)?;
        if finish {
            let nonzero = match *kind {
                TypeKind::Record(_) | TypeKind::Any(_) => types
                    .record_storage_definition(ty)?
                    .fields
                    .iter()
                    .any(|field| sizes[field]),
                TypeKind::FixedArray {
                    element,
                    count,
                } => count != 0 && sizes[&element],
                TypeKind::Distinct(id) => sizes[&types.distinct(id)?.representation],
                _ => true,
            };
            sizes.insert(ty, nonzero);
        } else {
            pending.push((ty, true));
            match *kind {
                TypeKind::Record(_) | TypeKind::Any(_) => {
                    for &field in &types.record_storage_definition(ty)?.fields {
                        pending.push((field, false));
                    }
                }
                TypeKind::FixedArray {
                    element, ..
                } => pending.push((element, false)),
                TypeKind::Distinct(id) => pending.push((types.distinct(id)?.representation, false)),
                _ => {}
            }
        }
    }
    Ok(sizes[&ty])
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::{RecordKind, TypeRegistry};
    #[test]
    fn scalar_views_use_the_owning_registry_and_preserve_common_roots() {
        let types = TypeRegistry::new();
        let foreign = TypeRegistry::new();
        let local = Local::new(
            ProcedureId::new(2),
            3,
            ScalarType::Int(IntegerType::U8),
            &types,
        );
        let integer = local.integer(&types).unwrap();
        assert_eq!(integer.place().place(), local.place());
        assert_eq!(integer.local().id().procedure(), ProcedureId::new(2));
        assert!(local.boolean(&types).is_none());
        assert!(local.integer(&foreign).is_none());
        let global = Global::new(0, GlobalInitializer::Bool(true), &types);
        assert_eq!(global.storage().place(), global.place());
        types.freeze().unwrap();
    }
    #[test]
    fn projections_keep_field_owner_and_place_arena() {
        let mut types = TypeRegistry::new();
        let a = types.reserve_record(RecordKind::Struct);
        let b = types.reserve_record(RecordKind::Struct);
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        types.define_record(a, [int]).unwrap();
        types.define_record(b, [int]).unwrap();
        let field = types.field(a, 0).unwrap().id;
        let local = Local::new_typed(ProcedureId::new(0), 0, a, &types).unwrap();
        let other = Local::new_typed(ProcedureId::new(0), 1, b, &types).unwrap();
        let mut places = PlaceRegistry::new();
        assert!(places.field(other.place(), field, &types).is_err());
        let place = places.field(local.place(), field, &types).unwrap();
        assert_eq!(place, places.field(local.place(), field, &types).unwrap());
        assert_eq!(
            IntPlace::try_from_place(place, &types).unwrap().ty(),
            IntegerType::S64
        );
        let foreign = PlaceRegistry::new().freeze();
        let PlaceKind::Field(id) = place.kind() else {
            panic!()
        };
        assert!(matches!(
            foreign.projection(id),
            Err(IrError::ForeignProjection(_))
        ));
    }
}
