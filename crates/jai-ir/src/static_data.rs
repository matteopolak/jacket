//! Validated immutable storage graphs for compiler-owned runtime constants.
use crate::{ConstantKind, ConstantValue, IrError};
use jai_types::{FieldId, RecordKind, TypeError, TypeId, TypeKind, TypeView};
use std::{
    fmt,
    sync::Arc,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_STATIC_ARENA: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StaticObjectId {
    arena: u64,
    index: usize,
    reservation: u64,
}
impl StaticObjectId {
    /// Process-local storage graph identity; never a host address or serialized ID.
    pub fn arena_identity(self) -> u64 {
        self.arena
    }
    pub fn index(self) -> usize {
        self.index
    }
    /// Distinguishes reservations that reused an unpublished physical slot.
    pub fn reservation_identity(self) -> u64 {
        self.reservation
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StaticProjection {
    Field(FieldId),
    Index(u64),
    ByteView(Arc<crate::StaticByteView>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StaticAddress {
    object: StaticObjectId,
    path: Box<[StaticProjection]>,
}
impl StaticAddress {
    pub fn new(object: StaticObjectId) -> Self {
        Self {
            object,
            path: Box::new([]),
        }
    }
    pub fn project(&self, projection: StaticProjection) -> Self {
        let mut path = self.path.to_vec();
        path.push(projection);
        Self {
            object: self.object,
            path: path.into(),
        }
    }
    pub fn object(&self) -> StaticObjectId {
        self.object
    }
    pub fn path(&self) -> &[StaticProjection] {
        &self.path
    }
}

#[derive(Clone, Debug)]
pub struct StaticValue {
    pub ty: TypeId,
    pub kind: StaticValueKind,
}
#[derive(Clone, Debug)]
pub enum StaticValueKind {
    Constant(ConstantValue),
    Record(Vec<StaticValue>),
    Array(Vec<StaticValue>),
    Address(StaticAddress),
    Slice {
        data: Option<StaticAddress>,
        count: u64,
    },
}
impl StaticValue {
    pub fn constant(value: ConstantValue) -> Self {
        Self {
            ty: value.ty,
            kind: StaticValueKind::Constant(value),
        }
    }
}

#[derive(Debug)]
pub struct StaticObject {
    id: StaticObjectId,
    value: StaticValue,
    descriptor: Option<crate::runtime_types::RuntimeTypeBinding>,
    retained_bytes: usize,
}
impl Drop for StaticObject {
    fn drop(&mut self) {
        let ty = self.value.ty;
        let value = std::mem::replace(
            &mut self.value,
            StaticValue {
                ty,
                kind: StaticValueKind::Constant(ConstantValue {
                    ty,
                    kind: ConstantKind::Zero,
                }),
            },
        );
        dispose_value(value);
    }
}
impl StaticObject {
    /// Sealed upper bound for this exact shared immutable object allocation.
    pub fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }
    pub fn id(&self) -> StaticObjectId {
        self.id
    }
    pub fn ty(&self) -> TypeId {
        self.value.ty
    }
    pub fn value(&self) -> &StaticValue {
        &self.value
    }
    pub fn runtime_type_identity(&self) -> Option<crate::RuntimeTypeIdentity> {
        self.descriptor.as_ref().map(|binding| binding.identity())
    }
    pub fn descriptor_header(&self) -> Option<&StaticAddress> {
        self.descriptor.as_ref().map(|binding| binding.header())
    }
    pub(crate) fn descriptor_binding(&self) -> Option<&crate::runtime_types::RuntimeTypeBinding> {
        self.descriptor.as_ref()
    }
}

/// Limits cover objects, aggregate value nodes, and address projection depth.
#[derive(Clone, Copy, Debug)]
pub struct StaticDataLimits {
    pub objects: usize,
    pub value_nodes: usize,
    pub value_depth: usize,
    pub projection_depth: usize,
    /// Complete retained allocation footprint, including spare Vec capacity.
    pub retained_bytes: usize,
}
impl Default for StaticDataLimits {
    fn default() -> Self {
        Self {
            objects: 65_536,
            value_nodes: 1_048_576,
            value_depth: 256,
            projection_depth: 256,
            retained_bytes: 64 * 1024 * 1024,
        }
    }
}

#[derive(Debug)]
pub enum StaticDataError {
    ByteView(crate::StaticByteViewError),
    Type(TypeError),
    Ir(IrError),
    ForeignObject(StaticObjectId),
    IncompleteObject(StaticObjectId),
    AlreadyDefined(StaticObjectId),
    TypeMismatch { expected: TypeId, actual: TypeId },
    InvalidValue(TypeId),
    OutOfBounds { index: u64, count: u64 },
    Limit(&'static str),
    ReservationExhausted,
}
impl From<crate::StaticByteViewError> for StaticDataError {
    fn from(error: crate::StaticByteViewError) -> Self {
        Self::ByteView(error)
    }
}
impl From<TypeError> for StaticDataError {
    fn from(error: TypeError) -> Self {
        Self::Type(error)
    }
}
impl From<IrError> for StaticDataError {
    fn from(error: IrError) -> Self {
        Self::Ir(error)
    }
}
impl fmt::Display for StaticDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ByteView(error) => error.fmt(f),
            Self::Type(error) => write!(f, "invalid static type: {error}"),
            Self::Ir(error) => write!(f, "invalid static constant: {error}"),
            Self::ForeignObject(_) => {
                f.write_str("static object belongs to a different storage graph")
            }
            Self::IncompleteObject(_) => f.write_str("static object definition is pending"),
            Self::AlreadyDefined(_) => f.write_str("static object is already defined"),
            Self::TypeMismatch {
                ..
            } => f.write_str("static value has a different nominal type"),
            Self::InvalidValue(_) => f.write_str("static value does not match its type"),
            Self::OutOfBounds {
                ..
            } => f.write_str("static address or view exceeds its array bounds"),
            Self::Limit(limit) => write!(f, "static storage exceeds its {limit} limit"),
            Self::ReservationExhausted => {
                f.write_str("static object reservation identity space exhausted")
            }
        }
    }
}
impl std::error::Error for StaticDataError {
}

enum BorrowedValue<'a> {
    Static(&'a StaticValue),
    Constant(&'a ConstantValue),
}

/// Prove resource bounds before retaining or cloning any recursive public tree.
fn validate_shapes<'a>(
    values: impl Iterator<Item = &'a StaticValue>,
    limits: StaticDataLimits,
) -> Result<usize, StaticDataError> {
    let mut pending = Vec::new();
    let mut nodes = 0usize;
    for value in values {
        if pending.len() >= limits.value_nodes {
            return Err(StaticDataError::Limit("value node count"));
        }
        pending.push((BorrowedValue::Static(value), 0usize));
    }
    while let Some((value, depth)) = pending.pop() {
        nodes = nodes
            .checked_add(1)
            .filter(|nodes| *nodes <= limits.value_nodes)
            .ok_or(StaticDataError::Limit("value node count"))?;
        if depth > limits.value_depth.min(256) {
            return Err(StaticDataError::Limit("value depth"));
        }
        let mut push = |value, depth| {
            if pending.len() >= limits.value_nodes - nodes {
                return Err(StaticDataError::Limit("value node count"));
            }
            pending.push((value, depth));
            Ok(())
        };
        match value {
            BorrowedValue::Static(value) => match &value.kind {
                StaticValueKind::Constant(value) => push(BorrowedValue::Constant(value), depth)?,
                StaticValueKind::Record(values) | StaticValueKind::Array(values) => {
                    for value in values {
                        push(BorrowedValue::Static(value), depth + 1)?;
                    }
                }
                StaticValueKind::Address(address)
                | StaticValueKind::Slice {
                    data: Some(address),
                    ..
                } => {
                    if address.path.len() > limits.projection_depth.min(256) {
                        return Err(StaticDataError::Limit("projection depth"));
                    }
                }
                StaticValueKind::Slice {
                    data: None, ..
                } => {}
            },
            BorrowedValue::Constant(value) => match &value.kind {
                ConstantKind::RuntimeType(_) => {
                    return Err(StaticDataError::InvalidValue(value.ty));
                }
                ConstantKind::Record(values) | ConstantKind::Array(values) => {
                    for value in values {
                        push(BorrowedValue::Constant(value), depth + 1)?;
                    }
                }
                ConstantKind::Union {
                    value, ..
                }
                | ConstantKind::Distinct(value) => {
                    push(BorrowedValue::Constant(value), depth + 1)?;
                }
                _ => {}
            },
        }
    }
    Ok(nodes)
}

enum OwnedValue {
    Static(StaticValue),
    Constant(ConstantValue),
}
fn dispose_value(value: StaticValue) {
    let mut pending = vec![OwnedValue::Static(value)];
    while let Some(value) = pending.pop() {
        match value {
            OwnedValue::Static(value) => match value.kind {
                StaticValueKind::Constant(value) => pending.push(OwnedValue::Constant(value)),
                StaticValueKind::Record(values) | StaticValueKind::Array(values) => {
                    pending.extend(values.into_iter().map(OwnedValue::Static));
                }
                _ => {}
            },
            OwnedValue::Constant(value) => match value.kind {
                ConstantKind::Record(values) | ConstantKind::Array(values) => {
                    pending.extend(values.into_iter().map(OwnedValue::Constant));
                }
                ConstantKind::Union {
                    value, ..
                }
                | ConstantKind::Distinct(value) => {
                    pending.push(OwnedValue::Constant(*value));
                }
                _ => {}
            },
        }
    }
}

struct PendingObject {
    id: StaticObjectId,
    ty: TypeId,
    value: Option<Arc<StaticObject>>,
    nodes: usize,
}
pub struct StaticDataBuilder {
    arena: u64,
    next_reservation: u64,
    objects: Vec<PendingObject>,
    published: usize,
    published_references: usize,
    retained_nodes: usize,
    retained_bytes: usize,
    published_bytes: usize,
}
impl Default for StaticDataBuilder {
    fn default() -> Self {
        Self::new()
    }
}
impl StaticDataBuilder {
    pub fn new() -> Self {
        let arena = NEXT_STATIC_ARENA
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |arena| {
                arena.checked_add(1)
            })
            .expect("static storage identity space exhausted");
        Self {
            arena,
            next_reservation: 0,
            objects: vec![],
            published: 0,
            published_references: 0,
            retained_nodes: 0,
            retained_bytes: 0,
            published_bytes: 0,
        }
    }
    pub fn reserve(
        &mut self,
        ty: TypeId,
        types: &dyn TypeView,
    ) -> Result<StaticObjectId, StaticDataError> {
        if self.objects.len() >= StaticDataLimits::default().objects {
            return Err(StaticDataError::Limit("object count"));
        }
        crate::storage::runtime_type(types, ty)?;
        let next_reservation = self
            .next_reservation
            .checked_add(1)
            .ok_or(StaticDataError::ReservationExhausted)?;
        let id = StaticObjectId {
            arena: self.arena,
            index: self.objects.len(),
            reservation: self.next_reservation,
        };
        self.objects.push(PendingObject {
            id,
            ty,
            value: None,
            nodes: 0,
        });
        self.next_reservation = next_reservation;
        Ok(id)
    }
    /// Prove a bounded root byte view of a reserved typed object.
    pub fn byte_view(
        &self,
        object: StaticObjectId,
        policy: jai_types::LayoutPolicy,
        offset: u64,
        length: u64,
        types: &dyn TypeView,
    ) -> Result<crate::StaticByteView, StaticDataError> {
        if object.arena != self.arena {
            return Err(StaticDataError::ForeignObject(object));
        }
        let backing = self
            .objects
            .get(object.index)
            .filter(|reserved| reserved.id == object)
            .ok_or(StaticDataError::ForeignObject(object))?
            .ty;
        Ok(crate::StaticByteView::new(
            object, backing, policy, offset, length, types,
        )?)
    }
    pub fn define(
        &mut self,
        id: StaticObjectId,
        value: StaticValue,
    ) -> Result<(), StaticDataError> {
        self.define_bound(id, value, None)
    }
    /// Bind a descriptor identity while defining its immutable storage object.
    /// A published object cannot gain or change this compiler-owned provenance.
    pub fn define_type_descriptor(
        &mut self,
        id: StaticObjectId,
        value: StaticValue,
        graph: &jai_types::ReflectionGraph,
        descriptor: jai_types::DescriptorId,
        types: &dyn TypeView,
    ) -> Result<(), StaticDataError> {
        let preflight = (|| {
            let remaining = StaticDataLimits::default()
                .retained_bytes
                .checked_sub(self.retained_bytes)
                .and_then(|bytes| bytes.checked_sub(self.published_bytes))
                .ok_or(StaticDataError::Limit("retained bytes"))?;
            let source = graph
                .get(descriptor)
                .map_err(|_| StaticDataError::InvalidValue(value.ty))?;
            retained_bytes::descriptor_preflight(&value, source, remaining)
        })();
        if let Err(error) = preflight {
            dispose_value(value);
            return Err(error);
        }
        let binding = match crate::runtime_types::RuntimeTypeBinding::new(
            id, &value, graph, descriptor, types,
        ) {
            Ok(binding) => binding,
            Err(error) => {
                dispose_value(value);
                return Err(error);
            }
        };
        self.define_bound(id, value, Some(binding))
    }
    fn define_bound(
        &mut self,
        id: StaticObjectId,
        value: StaticValue,
        descriptor: Option<crate::runtime_types::RuntimeTypeBinding>,
    ) -> Result<(), StaticDataError> {
        let admission = (|| {
            let mut limits = StaticDataLimits::default();
            limits.value_nodes -= self.retained_nodes;
            let nodes = validate_shapes(std::iter::once(&value), limits)?;
            let nodes = nodes
                .checked_add(descriptor.as_ref().map_or(0, |binding| binding.nodes()))
                .filter(|nodes| *nodes <= limits.value_nodes)
                .ok_or(StaticDataError::Limit("descriptor metadata count"))?;
            if id.arena != self.arena {
                return Err(StaticDataError::ForeignObject(id));
            }
            let object = self
                .objects
                .get(id.index)
                .filter(|object| object.id == id)
                .ok_or(StaticDataError::ForeignObject(id))?;
            if object.value.is_some() {
                return Err(StaticDataError::AlreadyDefined(id));
            }
            if object.ty != value.ty {
                return Err(StaticDataError::TypeMismatch {
                    expected: object.ty,
                    actual: value.ty,
                });
            }
            let remaining = limits
                .retained_bytes
                .checked_sub(self.retained_bytes)
                .and_then(|bytes| bytes.checked_sub(self.published_bytes))
                .ok_or(StaticDataError::Limit("retained bytes"))?;
            let bytes = retained_bytes::object(
                &value,
                descriptor
                    .as_ref()
                    .map(|binding| (binding.header(), binding.descriptor())),
                remaining,
            )?;
            Ok((nodes, bytes))
        })();
        let (nodes, bytes) = match admission {
            Ok(proof) => proof,
            Err(error) => {
                dispose_value(value);
                return Err(error);
            }
        };
        self.objects[id.index].value = Some(Arc::new(StaticObject {
            id,
            value,
            descriptor,
            retained_bytes: bytes,
        }));
        self.objects[id.index].nodes = nodes;
        self.retained_nodes += nodes;
        self.retained_bytes += bytes;
        Ok(())
    }
    /// Publish an immutable closure while retaining the sole append authority.
    /// Later publications preserve every previously published object's identity
    /// and value. The builder is deliberately not clonable.
    pub fn publish(
        &mut self,
        types: &dyn TypeView,
        limits: StaticDataLimits,
    ) -> Result<StaticData, StaticDataError> {
        if self.objects.len() > limits.objects {
            return Err(StaticDataError::Limit("object count"));
        }
        let published_references = self
            .published_references
            .checked_add(self.objects.len())
            .filter(|count| *count <= limits.value_nodes)
            .ok_or(StaticDataError::Limit("publication reference count"))?;
        validate_shapes(
            self.objects
                .iter()
                .filter_map(|object| object.value.as_ref().map(|value| value.value())),
            limits,
        )?;
        let table_bytes = retained_bytes::table(self.objects.len())?;
        self.retained_bytes
            .checked_add(table_bytes)
            .filter(|bytes| *bytes <= limits.retained_bytes)
            .ok_or(StaticDataError::Limit("retained bytes"))?;
        let published_bytes = self
            .published_bytes
            .checked_add(table_bytes)
            .filter(|bytes| {
                self.retained_bytes
                    .checked_add(*bytes)
                    .is_some_and(|total| total <= StaticDataLimits::default().retained_bytes)
            })
            .ok_or(StaticDataError::Limit("publication retained bytes"))?;
        let mut objects = Vec::with_capacity(self.objects.len());
        for object in &self.objects {
            let value = object
                .value
                .as_ref()
                .ok_or(StaticDataError::IncompleteObject(object.id))?;
            objects.push(Arc::clone(value));
        }
        let mut data = StaticData {
            arena: self.arena,
            objects: objects.into(),
            limits,
            validation_work: 0,
            table_bytes,
            retained_bytes: self.retained_bytes + table_bytes,
        };
        data.validation_work = validation_work::validate_and_measure(&data, types)?;
        self.published = self.objects.len();
        self.published_references = published_references;
        self.published_bytes = published_bytes;
        Ok(data)
    }

    /// Roll back failed construction without invalidating an exported address.
    pub fn discard_unpublished(&mut self) {
        self.retained_bytes -= self.objects[self.published..]
            .iter()
            .filter_map(|object| object.value.as_ref())
            .map(|object| object.retained_bytes())
            .sum::<usize>();
        self.retained_nodes -= self.objects[self.published..]
            .iter()
            .map(|object| object.nodes)
            .sum::<usize>();
        self.objects.truncate(self.published);
    }

    pub fn finish(
        mut self,
        types: &dyn TypeView,
        limits: StaticDataLimits,
    ) -> Result<StaticData, StaticDataError> {
        self.publish(types, limits)
    }
}

/// Validated closure of immutable objects. Addresses never contain host bytes.
#[derive(Clone, Debug)]
pub struct StaticData {
    arena: u64,
    objects: Box<[Arc<StaticObject>]>,
    limits: StaticDataLimits,
    validation_work: usize,
    table_bytes: usize,
    retained_bytes: usize,
}
impl StaticData {
    /// Compiler-owned identity for consumer caches; this is not a host address.
    pub fn identity(&self) -> u64 {
        self.arena
    }
    /// A sealed conservative bound for revalidating this exact immutable catalog.
    /// It is measured at publication, including descriptor metadata, paths,
    /// registry storage traversal and selected-layout dependency work.
    pub fn validation_work(&self) -> usize {
        self.validation_work
    }
    /// Sealed footprint of this catalog table, separately from shared objects.
    pub fn table_retained_bytes(&self) -> usize {
        self.table_bytes
    }
    /// Complete checked closure footprint. Consumers deduplicate shared objects.
    pub fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }
    pub fn objects(&self) -> &[Arc<StaticObject>] {
        &self.objects
    }
    pub fn object(&self, id: StaticObjectId) -> Result<&StaticObject, StaticDataError> {
        if id.arena != self.arena {
            return Err(StaticDataError::ForeignObject(id));
        }
        self.objects
            .get(id.index)
            .filter(|object| object.id == id)
            .map(Arc::as_ref)
            .ok_or(StaticDataError::ForeignObject(id))
    }
    pub fn address_type(
        &self,
        address: &StaticAddress,
        types: &dyn TypeView,
    ) -> Result<TypeId, StaticDataError> {
        self.address_extent(address, types).map(|(ty, _)| ty)
    }
    fn address_extent(
        &self,
        address: &StaticAddress,
        types: &dyn TypeView,
    ) -> Result<(TypeId, u64), StaticDataError> {
        if address.path.len() > self.limits.projection_depth {
            return Err(StaticDataError::Limit("projection depth"));
        }
        let mut ty = self.object(address.object)?.ty();
        let mut extent = 1;
        types.kind(ty)?;
        for projection in &address.path {
            match projection {
                StaticProjection::Field(field) => {
                    ty = types.validate_field(ty, *field)?;
                    extent = 1;
                }
                StaticProjection::ByteView(view) => {
                    if address.path.len() != 1 {
                        return Err(StaticDataError::InvalidValue(ty));
                    }
                    view.validate(address.object, ty, types)?;
                    ty = types.scalar(jai_types::ScalarType::Int(jai_types::IntegerType::U8));
                    extent = view.length();
                }
                StaticProjection::Index(index) => {
                    let TypeKind::FixedArray {
                        element,
                        count,
                    } = *types.kind(ty)?
                    else {
                        return Err(StaticDataError::InvalidValue(ty));
                    };
                    if *index >= count {
                        return Err(StaticDataError::OutOfBounds {
                            index: *index,
                            count,
                        });
                    }
                    ty = element;
                    extent = count - index;
                }
            }
        }
        Ok((ty, extent))
    }
    /// Revalidation at a consumer boundary checks registry ownership as well as
    /// all aggregate members. The traversal is bounded and iterative.
    pub fn validate(&self, types: &dyn TypeView) -> Result<(), StaticDataError> {
        self.validate_inner(types, None)
    }
    fn validate_inner(
        &self,
        types: &dyn TypeView,
        work: Option<&validation_work::ValidationWork>,
    ) -> Result<(), StaticDataError> {
        if let Some(work) = work {
            work.add(
                self.objects
                    .len()
                    .checked_mul(8)
                    .ok_or(StaticDataError::Limit("validation work"))?,
            )?;
        }
        // Multiple immutable revisions may certify the same nominal type.
        // Validate every exact object receipt and charge all retained payloads.
        let mut descriptor_work = self.limits.value_nodes;
        for object in &self.objects {
            if let Some(binding) = object.descriptor_binding() {
                if let Some(work) = work {
                    work.add(
                        binding
                            .nodes()
                            .checked_mul(32)
                            .ok_or(StaticDataError::Limit("validation work"))?,
                    )?;
                }
                binding.validate(self, types, &mut descriptor_work)?;
            }
        }
        let mut pending: Vec<_> = self
            .objects
            .iter()
            .map(|object| (&object.value, object.ty(), 0usize))
            .collect();
        let mut nodes = 0usize;
        while let Some((value, expected, depth)) = pending.pop() {
            if let Some(work) = work {
                work.add(8)?;
                match &value.kind {
                    StaticValueKind::Address(address)
                    | StaticValueKind::Slice {
                        data: Some(address),
                        ..
                    } => work.add(
                        address
                            .path()
                            .len()
                            .checked_mul(8)
                            .ok_or(StaticDataError::Limit("validation work"))?,
                    )?,
                    StaticValueKind::Constant(crate::ConstantValue {
                        kind: ConstantKind::StringBytes(bytes),
                        ..
                    }) => work.add(
                        bytes
                            .len()
                            .checked_mul(2)
                            .ok_or(StaticDataError::Limit("validation work"))?,
                    )?,
                    _ => {}
                }
            }
            nodes = nodes
                .checked_add(1)
                .filter(|nodes| *nodes <= self.limits.value_nodes)
                .ok_or(StaticDataError::Limit("value node count"))?;
            if depth > self.limits.value_depth.min(256) {
                return Err(StaticDataError::Limit("value depth"));
            }
            if value.ty != expected {
                return Err(StaticDataError::TypeMismatch {
                    expected,
                    actual: value.ty,
                });
            }
            let kind = types.kind(expected)?;
            match &value.kind {
                StaticValueKind::Constant(constant) => {
                    // Type cells use symbolic addresses in this graph. Keeping
                    // cross-graph Arc constants here could form unbounded
                    // recursive validation and disposal chains.
                    if matches!(constant.kind, ConstantKind::RuntimeType(_)) {
                        return Err(StaticDataError::InvalidValue(expected));
                    }
                    if constant.ty != expected {
                        return Err(StaticDataError::TypeMismatch {
                            expected,
                            actual: constant.ty,
                        });
                    }
                    // Aggregates use StaticValue's bounded traversal. A compact
                    // typed zero initializer does not recurse into Rust values.
                    if matches!(
                        kind,
                        TypeKind::Record(_) | TypeKind::Any(_) | TypeKind::FixedArray { .. }
                    ) && !matches!(constant.kind, ConstantKind::Zero)
                    {
                        return Err(StaticDataError::InvalidValue(expected));
                    }
                    crate::verify::constant(types, constant)?;
                }
                StaticValueKind::Record(fields) => {
                    let record = types.record_storage_definition(expected)?;
                    if record.kind != RecordKind::Struct || fields.len() != record.fields.len() {
                        return Err(StaticDataError::InvalidValue(expected));
                    }
                    pending.extend(
                        fields
                            .iter()
                            .zip(record.fields.iter())
                            .map(|(field, &ty)| (field, ty, depth + 1)),
                    );
                }
                StaticValueKind::Array(elements) => {
                    let TypeKind::FixedArray {
                        element,
                        count,
                    } = *kind
                    else {
                        return Err(StaticDataError::InvalidValue(expected));
                    };
                    if u64::try_from(elements.len()).ok() != Some(count) {
                        return Err(StaticDataError::InvalidValue(expected));
                    }
                    pending.extend(
                        elements
                            .iter()
                            .map(|element_value| (element_value, element, depth + 1)),
                    );
                }
                StaticValueKind::Address(address) => {
                    let pointee = match *kind {
                        TypeKind::Pointer(pointee) => pointee,
                        TypeKind::Type => {
                            let schema = jai_types::RuntimeTypeSchema::from_view(types)?;
                            let object = self.object(address.object())?;
                            if object.descriptor_header() != Some(address) {
                                return Err(StaticDataError::InvalidValue(expected));
                            }
                            schema.header_type()
                        }
                        _ => return Err(StaticDataError::InvalidValue(expected)),
                    };
                    let actual = self.address_type(address, types)?;
                    if actual != pointee {
                        return Err(StaticDataError::TypeMismatch {
                            expected: pointee,
                            actual,
                        });
                    }
                }
                StaticValueKind::Slice {
                    data,
                    count,
                } => {
                    let TypeKind::Slice(element) = *kind else {
                        return Err(StaticDataError::InvalidValue(expected));
                    };
                    match data {
                        None if *count == 0 => {}
                        Some(address) => {
                            let (actual, extent) = self.address_extent(address, types)?;
                            if actual != element {
                                return Err(StaticDataError::TypeMismatch {
                                    expected: element,
                                    actual,
                                });
                            }
                            if *count > extent {
                                return Err(StaticDataError::OutOfBounds {
                                    index: *count,
                                    count: extent,
                                });
                            }
                        }
                        None => return Err(StaticDataError::InvalidValue(expected)),
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

mod validation_work;

mod retained_bytes;
