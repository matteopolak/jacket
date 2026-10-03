//! Object-owned target-layout proofs for immutable heterogeneous byte views.
use crate::{StaticAddress, StaticObjectId, StaticProjection};
use jai_types::{LayoutEngine, LayoutError, LayoutPolicy, TypeId, TypeView};
use std::{fmt, sync::Arc};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StaticByteView {
    object: StaticObjectId,
    backing: TypeId,
    policy: LayoutPolicy,
    offset: u64,
    length: u64,
}

#[derive(Debug)]
pub enum StaticByteViewError {
    Layout(LayoutError),
    ForeignObject,
    UnaddressableBacking {
        size: u64,
        pointer_bits: u64,
    },
    SliceCount(u64),
    BackingType {
        expected: TypeId,
        actual: TypeId,
    },
    Range {
        offset: u64,
        length: u64,
        size: u64,
    },
    Target {
        expected: Box<LayoutPolicy>,
        actual: Box<LayoutPolicy>,
    },
}
impl From<LayoutError> for StaticByteViewError {
    fn from(error: LayoutError) -> Self {
        Self::Layout(error)
    }
}
impl fmt::Display for StaticByteViewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnaddressableBacking {
                ..
            } => f.write_str("immutable byte backing exceeds its selected pointer address space"),
            Self::SliceCount(_) => {
                f.write_str("immutable byte view length does not fit the signed slice count")
            }
            Self::Layout(error) => write!(f, "invalid immutable byte backing layout: {error}"),
            Self::ForeignObject => {
                f.write_str("immutable byte view belongs to another static object")
            }
            Self::BackingType {
                ..
            } => f.write_str("immutable byte view has a different nominal backing type"),
            Self::Range {
                ..
            } => f.write_str("immutable byte view exceeds its target backing layout"),
            Self::Target {
                ..
            } => f.write_str("immutable byte view uses a different selected target layout"),
        }
    }
}
impl std::error::Error for StaticByteViewError {
}

impl StaticByteView {
    // Only StaticDataBuilder calls this with its checked reserved-object type.
    pub(crate) fn new(
        object: StaticObjectId,
        backing: TypeId,
        policy: LayoutPolicy,
        offset: u64,
        length: u64,
        types: &dyn TypeView,
    ) -> Result<Self, StaticByteViewError> {
        let result = Self {
            object,
            backing,
            policy,
            offset,
            length,
        };
        result.validate(object, backing, types)?;
        Ok(result)
    }
    pub fn object(&self) -> StaticObjectId {
        self.object
    }
    pub fn backing_type(&self) -> TypeId {
        self.backing
    }
    pub fn policy(&self) -> LayoutPolicy {
        self.policy
    }
    pub fn offset(&self) -> u64 {
        self.offset
    }
    pub fn length(&self) -> u64 {
        self.length
    }
    pub fn address(&self) -> StaticAddress {
        StaticAddress::new(self.object).project(StaticProjection::ByteView(Arc::new(self.clone())))
    }
    /// Check the actual consumer policy before constructing a target pointer.
    pub fn validate_target(&self, actual: LayoutPolicy) -> Result<(), StaticByteViewError> {
        if self.policy != actual {
            return Err(StaticByteViewError::Target {
                expected: Box::new(self.policy),
                actual: Box::new(actual),
            });
        }
        Ok(())
    }
    pub(crate) fn validate(
        &self,
        object: StaticObjectId,
        backing: TypeId,
        types: &dyn TypeView,
    ) -> Result<(), StaticByteViewError> {
        if self.object != object {
            return Err(StaticByteViewError::ForeignObject);
        }
        if self.backing != backing {
            return Err(StaticByteViewError::BackingType {
                expected: self.backing,
                actual: backing,
            });
        }
        let size = LayoutEngine::new(types, self.policy).layout(backing)?.size;
        let pointer_bits = self.policy.pointer().size.checked_mul(8).ok_or(
            StaticByteViewError::UnaddressableBacking {
                size,
                pointer_bits: u64::MAX,
            },
        )?;
        if pointer_bits < 64 && size > ((1u64 << pointer_bits) - 1) {
            return Err(StaticByteViewError::UnaddressableBacking {
                size,
                pointer_bits,
            });
        }
        if self.length > i64::MAX as u64 {
            return Err(StaticByteViewError::SliceCount(self.length));
        }
        // Zero-length views may point exactly one past a real object. No load is
        // authorized there; offset+length must never wrap or exceed its extent.
        if self.offset > size || self.length > size - self.offset {
            return Err(StaticByteViewError::Range {
                offset: self.offset,
                length: self.length,
                size,
            });
        }
        Ok(())
    }
}
