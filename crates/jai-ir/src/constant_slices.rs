//! Private prototype: owned typed slice storage, identity and publication bounds.
//! This module is intentionally unregistered until every consumer is coherent.
use crate::{ConstantKind, ConstantValue, IrError};
use jai_types::{
    IntegerType, LayoutEngine, LayoutError, LayoutPolicy, ScalarType, TypeError, TypeId, TypeKind,
    TypeView,
};
use std::{
    fmt,
    hash::Hash,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConstantSliceAccess {
    ReadOnly,
    Mutable,
}

#[derive(Clone, Copy, Debug)]
pub struct ConstantSliceLimits {
    pub elements: usize,
    pub value_nodes: usize,
    pub value_depth: usize,
    pub owned_bytes: usize,
    pub receipt_bytes: usize,
}
impl Default for ConstantSliceLimits {
    fn default() -> Self {
        Self {
            elements: 1_048_576,
            value_nodes: 1_048_576,
            value_depth: 128,
            owned_bytes: 1_048_576,
            receipt_bytes: 1_048_576,
        }
    }
}

#[derive(Debug)]
pub enum ConstantSliceError {
    Type(TypeError),
    Constant(IrError),
    Layout(LayoutError),
    InvalidSlice(TypeId),
    InvalidLiteral(TypeId),
    ElementType {
        expected: TypeId,
        actual: TypeId,
    },
    Range {
        start: u64,
        count: u64,
        capacity: u64,
    },
    SignedCount(u64),
    TargetExtent,
    Limit(&'static str),
}
impl fmt::Display for ConstantSliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type(error) => write!(f, "{error}"),
            Self::Constant(error) => write!(f, "{error}"),
            Self::Layout(error) => write!(f, "{error}"),
            Self::InvalidSlice(ty) => write!(
                f,
                "constant slice requires a canonical slice type; found {ty:?}"
            ),
            Self::InvalidLiteral(ty) => write!(
                f,
                "constant slice literal storage {ty:?} has another type or count"
            ),
            Self::ElementType {
                expected,
                actual,
            } => write!(
                f,
                "constant slice element has type {actual:?}; expected {expected:?}"
            ),
            Self::Range {
                start,
                count,
                capacity,
            } => write!(
                f,
                "constant slice range start {start}, count {count} exceeds actual capacity {capacity}"
            ),
            Self::SignedCount(count) => write!(
                f,
                "constant slice count {count} exceeds signed descriptor range"
            ),
            Self::TargetExtent => {
                f.write_str("constant slice backing exceeds the selected target address domain")
            }
            Self::Limit(limit) => write!(f, "constant slice exceeds its {limit} budget"),
        }
    }
}
impl std::error::Error for ConstantSliceError {
}
impl From<TypeError> for ConstantSliceError {
    fn from(value: TypeError) -> Self {
        Self::Type(value)
    }
}
impl From<LayoutError> for ConstantSliceError {
    fn from(value: LayoutError) -> Self {
        Self::Layout(value)
    }
}

/// Canonical publication envelope, supplied from retained source-run origin facts.
/// The envelope must contain source pool references, never VM allocation counters.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConstantSliceReceipt {
    envelope: Arc<[u8]>,
    occurrence: u64,
    backing_ordinal: u64,
}
impl ConstantSliceReceipt {
    pub fn envelope(&self) -> &[u8] {
        &self.envelope
    }
    pub fn occurrence(&self) -> u64 {
        self.occurrence
    }
    pub fn backing_ordinal(&self) -> u64 {
        self.backing_ordinal
    }
}

/// Session cache identity is distinct from canonical replay meaning.
#[derive(Clone, Debug)]
pub struct CapturedSliceIdentity {
    session: u64,
    receipt: ConstantSliceReceipt,
}
impl CapturedSliceIdentity {
    pub fn session_identity(&self) -> u64 {
        self.session
    }
    pub fn receipt(&self) -> &ConstantSliceReceipt {
        &self.receipt
    }
}
impl PartialEq for CapturedSliceIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.session == other.session
            && self.receipt.backing_ordinal == other.receipt.backing_ordinal
    }
}
impl Eq for CapturedSliceIdentity {
}
impl std::hash::Hash for CapturedSliceIdentity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.session.hash(state);
        self.receipt.backing_ordinal.hash(state);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConstantSliceOrigin {
    Literal { storage: TypeId },
    Captured(CapturedSliceIdentity),
}

pub struct CapturedSliceBatch {
    session: u64,
    envelope: Arc<[u8]>,
    occurrence: u64,
    next: u64,
}
impl CapturedSliceBatch {
    pub fn new(
        envelope: Vec<u8>,
        occurrence: u64,
        limits: ConstantSliceLimits,
    ) -> Result<Self, ConstantSliceError> {
        if envelope.is_empty() || envelope.len() > limits.receipt_bytes {
            return Err(ConstantSliceError::Limit("publication receipt bytes"));
        }
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let session = NEXT
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| ConstantSliceError::Limit("publication identity"))?;
        Ok(Self {
            session,
            envelope: envelope.into(),
            occurrence,
            next: 0,
        })
    }
    fn next_identity(&mut self) -> Result<CapturedSliceIdentity, ConstantSliceError> {
        let next = self
            .next
            .checked_add(1)
            .ok_or(ConstantSliceError::Limit("backing identity"))?;
        let identity = CapturedSliceIdentity {
            session: self.session,
            receipt: ConstantSliceReceipt {
                envelope: Arc::clone(&self.envelope),
                occurrence: self.occurrence,
                backing_ordinal: self.next,
            },
        };
        self.next = next;
        Ok(identity)
    }
}

#[derive(Debug)]
pub struct ConstantSliceBacking {
    element: TypeId,
    slots: Vec<Option<ConstantValue>>,
    origin: ConstantSliceOrigin,
    access: ConstantSliceAccess,
    cells: usize,
    depth: usize,
    owned_bytes: usize,
}
impl Drop for ConstantSliceBacking {
    fn drop(&mut self) {
        dispose_slots(std::mem::take(&mut self.slots));
    }
}
impl ConstantSliceBacking {
    pub fn element(&self) -> TypeId {
        self.element
    }
    pub fn capacity(&self) -> u64 {
        self.slots.len() as u64
    }
    pub fn slots(&self) -> &[Option<ConstantValue>] {
        &self.slots
    }
    pub fn origin(&self) -> &ConstantSliceOrigin {
        &self.origin
    }
    pub fn access(&self) -> ConstantSliceAccess {
        self.access
    }
    pub fn cells(&self) -> usize {
        self.cells
    }
    pub fn depth(&self) -> usize {
        self.depth
    }
    pub fn owned_bytes(&self) -> usize {
        self.owned_bytes
    }
    pub fn work(&self) -> usize {
        self.cells + self.owned_bytes // Checked together before retention.
    }

    pub fn literal(
        types: &dyn TypeView,
        storage: TypeId,
        elements: Vec<ConstantValue>,
        limits: ConstantSliceLimits,
    ) -> Result<Arc<Self>, ConstantSliceError> {
        let admission = (|| {
            if elements.len() > limits.elements || elements.len() > limits.value_nodes {
                return Err(ConstantSliceError::Limit("element slots"));
            }
            match *types.kind(storage)? {
                TypeKind::FixedArray {
                    element,
                    count,
                } if count == elements.len() as u64 => Ok(element),
                TypeKind::String => Ok(types.scalar(ScalarType::Int(IntegerType::U8))),
                _ => Err(ConstantSliceError::InvalidLiteral(storage)),
            }
        })();
        let element = match admission {
            Ok(element) => element,
            Err(error) => {
                for value in elements {
                    crate::disposal::constant(value);
                }
                return Err(error);
            }
        };
        Self::build(
            types,
            element,
            elements.into_iter().map(Some).collect(),
            ConstantSliceOrigin::Literal {
                storage,
            },
            ConstantSliceAccess::ReadOnly,
            limits,
        )
    }

    /// Caller must certify live source storage and its full actual capacity first.
    /// No virtual address or allocation/version integer enters the retained recipe.
    pub fn captured(
        types: &dyn TypeView,
        element: TypeId,
        slots: Vec<Option<ConstantValue>>,
        access: ConstantSliceAccess,
        batch: &mut CapturedSliceBatch,
        limits: ConstantSliceLimits,
    ) -> Result<Arc<Self>, ConstantSliceError> {
        let admission = validate_slots(types, element, &slots, limits);
        let (cells, depth, owned_bytes) = match admission {
            Ok(shape) => shape,
            Err(error) => {
                dispose_slots(slots);
                return Err(error);
            }
        };
        let identity = match batch.next_identity() {
            Ok(identity) => identity,
            Err(error) => {
                dispose_slots(slots);
                return Err(error);
            }
        };
        Ok(Arc::new(Self {
            element,
            slots,
            origin: ConstantSliceOrigin::Captured(identity),
            access,
            cells,
            depth,
            owned_bytes,
        }))
    }

    fn build(
        types: &dyn TypeView,
        element: TypeId,
        slots: Vec<Option<ConstantValue>>,
        origin: ConstantSliceOrigin,
        access: ConstantSliceAccess,
        limits: ConstantSliceLimits,
    ) -> Result<Arc<Self>, ConstantSliceError> {
        match validate_slots(types, element, &slots, limits) {
            Ok((cells, depth, owned_bytes)) => Ok(Arc::new(Self {
                element,
                slots,
                origin,
                access,
                cells,
                depth,
                owned_bytes,
            })),
            Err(error) => {
                dispose_slots(slots);
                Err(error)
            }
        }
    }

    pub fn selected_extent(
        &self,
        types: &dyn TypeView,
        policy: &LayoutPolicy,
    ) -> Result<ConstantSliceExtent, ConstantSliceError> {
        let mut engine = LayoutEngine::new(types, *policy);
        let layout = engine.layout(self.element)?;
        let alignment = layout.alignment;
        let mask = u64::from(alignment - 1);
        let stride = layout
            .size
            .checked_add(mask)
            .map(|bytes| bytes & !mask)
            .ok_or(ConstantSliceError::TargetExtent)?;
        let bytes = stride
            .checked_mul(self.slots.len() as u64)
            .ok_or(ConstantSliceError::TargetExtent)?;
        let bits = policy
            .pointer()
            .size
            .checked_mul(8)
            .ok_or(ConstantSliceError::TargetExtent)?;
        let maximum = if bits == 64 {
            u64::MAX
        } else if bits < 64 {
            (1u64 << bits) - 1
        } else {
            return Err(ConstantSliceError::TargetExtent);
        };
        if bytes > maximum {
            return Err(ConstantSliceError::TargetExtent);
        }
        Ok(ConstantSliceExtent {
            stride,
            bytes,
            alignment,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstantSliceExtent {
    pub stride: u64,
    pub bytes: u64,
    pub alignment: u32,
}

#[derive(Clone, Debug)]
pub struct ConstantSlice {
    ty: TypeId,
    backing: Option<Arc<ConstantSliceBacking>>,
    start: u64,
    count: u64,
}
impl ConstantSlice {
    pub fn ty(&self) -> TypeId {
        self.ty
    }
    pub fn backing(&self) -> Option<&Arc<ConstantSliceBacking>> {
        self.backing.as_ref()
    }
    pub fn start(&self) -> u64 {
        self.start
    }
    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn new(
        types: &dyn TypeView,
        policy: &LayoutPolicy,
        ty: TypeId,
        backing: Option<Arc<ConstantSliceBacking>>,
        start: u64,
        count: u64,
        limits: ConstantSliceLimits,
    ) -> Result<Self, ConstantSliceError> {
        let value = Self {
            ty,
            backing,
            start,
            count,
        };
        value.validate(types, policy, limits)?;
        Ok(value)
    }
    pub fn checked_clone(
        &self,
        types: &dyn TypeView,
        policy: &LayoutPolicy,
        limits: ConstantSliceLimits,
    ) -> Result<Self, ConstantSliceError> {
        self.validate(types, policy, limits)?;
        Ok(self.clone()) // Arc-only; no recursive ConstantValue clone.
    }
    pub fn validate(
        &self,
        types: &dyn TypeView,
        policy: &LayoutPolicy,
        limits: ConstantSliceLimits,
    ) -> Result<(), ConstantSliceError> {
        let TypeKind::Slice(element) = *types.kind(self.ty)? else {
            return Err(ConstantSliceError::InvalidSlice(self.ty));
        };
        if self.count > i64::MAX as u64 {
            return Err(ConstantSliceError::SignedCount(self.count));
        }
        let Some(backing) = &self.backing else {
            return if self.start == 0 && self.count == 0 {
                Ok(())
            } else {
                Err(ConstantSliceError::Range {
                    start: self.start,
                    count: self.count,
                    capacity: 0,
                })
            };
        };
        if backing.element != element {
            return Err(ConstantSliceError::ElementType {
                expected: element,
                actual: backing.element,
            });
        }
        let (cells, depth, owned_bytes) = validate_slots(types, element, &backing.slots, limits)?;
        if cells != backing.cells || depth != backing.depth || owned_bytes != backing.owned_bytes {
            return Err(ConstantSliceError::Limit("stale backing proof"));
        }
        if let ConstantSliceOrigin::Literal {
            storage,
        } = &backing.origin
        {
            match *types.kind(*storage)? {
                TypeKind::FixedArray {
                    element: stored,
                    count,
                } if stored == element && count == backing.slots.len() as u64 => {}
                TypeKind::String if element == types.scalar(ScalarType::Int(IntegerType::U8)) => {}
                _ => return Err(ConstantSliceError::InvalidLiteral(*storage)),
            }
            if backing.slots.is_empty() || backing.access != ConstantSliceAccess::ReadOnly {
                return Err(ConstantSliceError::InvalidLiteral(*storage));
            }
        }
        let capacity = backing.slots.len() as u64;
        self.start
            .checked_add(self.count)
            .filter(|end| *end <= capacity)
            .ok_or(ConstantSliceError::Range {
                start: self.start,
                count: self.count,
                capacity,
            })?;
        backing.selected_extent(types, policy)?;
        Ok(())
    }
}

fn validate_slots(
    types: &dyn TypeView,
    element: TypeId,
    slots: &[Option<ConstantValue>],
    limits: ConstantSliceLimits,
) -> Result<(usize, usize, usize), ConstantSliceError> {
    if limits.value_depth == 0 {
        return Err(ConstantSliceError::Limit("value depth"));
    }
    if slots.len() > limits.elements {
        return Err(ConstantSliceError::Limit("element slots"));
    }
    crate::storage::runtime_type(types, element).map_err(ConstantSliceError::Constant)?;
    let mut cells = slots
        .len()
        .checked_add(1)
        .filter(|cells| *cells <= limits.value_nodes)
        .ok_or(ConstantSliceError::Limit("value nodes"))?;
    let mut pending = Vec::new();
    for value in slots.iter().flatten() {
        if pending.len() >= limits.value_nodes.saturating_sub(cells) {
            return Err(ConstantSliceError::Limit("value nodes"));
        }
        pending.push((value, 1usize));
    }
    let mut maximum_depth = 1;
    let mut owned_bytes = 0usize;
    while let Some((value, depth)) = pending.pop() {
        if depth >= limits.value_depth.min(256) {
            return Err(ConstantSliceError::Limit("value depth"));
        }
        cells = cells
            .checked_add(1)
            .filter(|cells| *cells <= limits.value_nodes)
            .ok_or(ConstantSliceError::Limit("value nodes"))?;
        maximum_depth = maximum_depth.max(depth + 1);
        if let ConstantKind::StringBytes(bytes) = &value.kind {
            owned_bytes = owned_bytes
                .checked_add(bytes.len())
                .filter(|bytes| *bytes <= limits.owned_bytes)
                .ok_or(ConstantSliceError::Limit("owned bytes"))?;
        }
        let children: &[ConstantValue] = match &value.kind {
            ConstantKind::Record(values) | ConstantKind::Array(values) => values,
            ConstantKind::Union {
                value, ..
            }
            | ConstantKind::Distinct(value) => std::slice::from_ref(value),
            _ => &[],
        };
        if children.len()
            > limits
                .value_nodes
                .saturating_sub(cells)
                .saturating_sub(pending.len())
        {
            return Err(ConstantSliceError::Limit("value nodes"));
        }
        pending.extend(children.iter().map(|value| (value, depth + 1)));
    }
    for value in slots.iter().flatten() {
        if value.ty != element {
            return Err(ConstantSliceError::ElementType {
                expected: element,
                actual: value.ty,
            });
        }
        crate::verify::constant(types, value).map_err(ConstantSliceError::Constant)?;
    }
    cells
        .checked_add(owned_bytes)
        .ok_or(ConstantSliceError::Limit("retained work"))?;
    Ok((cells, maximum_depth, owned_bytes))
}
fn dispose_slots(slots: Vec<Option<ConstantValue>>) {
    for value in slots.into_iter().flatten() {
        crate::disposal::constant(value);
    }
}

#[cfg(test)]
#[path = "constant_slices/tests.rs"]
mod tests;
