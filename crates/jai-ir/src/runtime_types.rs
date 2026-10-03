//! Runtime Type constants derive their identity from immutable descriptor storage.
use crate::{
    ConstantKind, StaticAddress, StaticData, StaticDataError, StaticObjectId, StaticProjection,
    StaticValue, StaticValueKind,
};
use jai_types::{
    DescriptorId, DescriptorKind, LayoutPolicy, ReflectionGraph, RuntimeTypeSchema, TypeDescriptor,
    TypeId, TypeView,
};
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

/// Semantic identity certified by one immutable descriptor object.
/// A VM must also check the actual pointer names this object's header before
/// using the identity of a dynamically loaded Type value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RuntimeTypeIdentity {
    ty: TypeId,
    object: StaticObjectId,
    schema: RuntimeTypeSchema,
    policy: LayoutPolicy,
}
impl RuntimeTypeIdentity {
    pub fn ty(self) -> TypeId {
        self.ty
    }
    pub fn object(self) -> StaticObjectId {
        self.object
    }
    pub fn schema(self) -> RuntimeTypeSchema {
        self.schema
    }
    pub fn policy(self) -> LayoutPolicy {
        self.policy
    }
    pub fn validate(self, types: &dyn TypeView) -> Result<(), jai_types::TypeError> {
        types.kind(self.ty)?;
        self.schema.validate(types)
    }
}

#[derive(Debug)]
pub(crate) struct RuntimeTypeBinding {
    identity: RuntimeTypeIdentity,
    header: StaticAddress,
    descriptor: TypeDescriptor,
    nodes: usize,
}
impl RuntimeTypeBinding {
    pub(crate) fn new(
        object: StaticObjectId,
        value: &StaticValue,
        graph: &ReflectionGraph,
        descriptor: DescriptorId,
        types: &dyn TypeView,
    ) -> Result<Self, StaticDataError> {
        let represented = descriptor.represented_type();
        types.kind(represented)?;
        let descriptor = graph
            .get(descriptor)
            .map_err(|_| StaticDataError::InvalidValue(represented))?;
        let schema = RuntimeTypeSchema::from_view(types)?;
        let nodes = payload::metadata_nodes(descriptor)?;
        let (header, header_value) = if value.ty == schema.header_type() {
            (StaticAddress::new(object), value)
        } else {
            let field = types.field(value.ty, 0)?;
            if field.ty != schema.header_type() {
                return Err(StaticDataError::InvalidValue(value.ty));
            }
            let StaticValueKind::Record(fields) = &value.kind else {
                return Err(StaticDataError::InvalidValue(value.ty));
            };
            let header_value = fields
                .first()
                .ok_or(StaticDataError::InvalidValue(value.ty))?;
            (
                StaticAddress::new(object).project(StaticProjection::Field(field.id)),
                header_value,
            )
        };
        Self::validate_header(header_value, descriptor.tag(), descriptor.runtime_size())?;
        payload::validate_shape(value, &descriptor.kind, schema)?;
        if let DescriptorKind::Integer {
            representation,
        } = &descriptor.kind
        {
            let StaticValueKind::Record(fields) = &value.kind else {
                return Err(StaticDataError::InvalidValue(value.ty));
            };
            if !matches!(fields.get(1).map(|value| &value.kind), Some(StaticValueKind::Constant(crate::ConstantValue { kind: ConstantKind::Bool(signed), .. })) if *signed == representation.signed())
            {
                return Err(StaticDataError::InvalidValue(value.ty));
            }
        }
        Ok(Self {
            identity: RuntimeTypeIdentity {
                ty: represented,
                object,
                schema,
                policy: graph.policy(),
            },
            header,
            descriptor: descriptor.clone(),
            nodes,
        })
    }
    fn validate_header(
        value: &StaticValue,
        tag: jai_types::TypeInfoTag,
        size: Option<u64>,
    ) -> Result<(), StaticDataError> {
        let StaticValueKind::Record(fields) = &value.kind else {
            return Err(StaticDataError::InvalidValue(value.ty));
        };
        let [tag_value, size_value] = fields.as_slice() else {
            return Err(StaticDataError::InvalidValue(value.ty));
        };
        if !matches!(&tag_value.kind, StaticValueKind::Constant(crate::ConstantValue { kind: ConstantKind::Enum(value), .. }) if value.value() == i128::from(tag as u32))
            || !matches!(&size_value.kind, StaticValueKind::Constant(crate::ConstantValue { kind: ConstantKind::Int(value), .. }) if value.value() == size.map_or(-1, i128::from))
        {
            return Err(StaticDataError::InvalidValue(value.ty));
        }
        Ok(())
    }
    pub(crate) fn identity(&self) -> RuntimeTypeIdentity {
        self.identity
    }
    pub(crate) fn header(&self) -> &StaticAddress {
        &self.header
    }
    pub(crate) fn descriptor(&self) -> &TypeDescriptor {
        &self.descriptor
    }
    pub(crate) fn nodes(&self) -> usize {
        self.nodes
    }
    pub(crate) fn validate(
        &self,
        data: &StaticData,
        types: &dyn TypeView,
        remaining: &mut usize,
    ) -> Result<(), StaticDataError> {
        self.identity.validate(types)?;
        if data.address_type(&self.header, types)? != self.identity.schema.header_type() {
            return Err(StaticDataError::InvalidValue(self.identity.ty));
        }
        let object = data.object(self.identity.object)?;
        let header = match self.header.path() {
            [] => object.value(),
            [StaticProjection::Field(field)] => {
                let StaticValueKind::Record(fields) = &object.value().kind else {
                    return Err(StaticDataError::InvalidValue(object.ty()));
                };
                fields
                    .get(field.index())
                    .ok_or(StaticDataError::InvalidValue(object.ty()))?
            }
            _ => return Err(StaticDataError::InvalidValue(object.ty())),
        };
        Self::validate_header(
            header,
            self.descriptor.tag(),
            self.descriptor.runtime_size(),
        )?;
        payload::charge(remaining, self.nodes)?;
        payload::validate(
            data,
            object.value(),
            &self.descriptor,
            self.identity,
            types,
            remaining,
        )
    }
}

/// An actual descriptor pointer constant, with checked static closure and identity.
#[derive(Clone, Debug)]
pub struct RuntimeTypeConstant {
    data: Arc<StaticData>,
    identity: Arc<RuntimeTypeIdentity>,
}
impl RuntimeTypeConstant {
    pub fn new(
        data: Arc<StaticData>,
        object: StaticObjectId,
        types: &dyn TypeView,
    ) -> Result<Self, StaticDataError> {
        data.validate(types)?;
        let identity = data
            .object(object)?
            .runtime_type_identity()
            .ok_or(StaticDataError::InvalidValue(data.object(object)?.ty()))?;
        Ok(Self {
            data,
            identity: Arc::new(identity),
        })
    }
    /// Reify an opaque identity recovered from a canonical runtime descriptor.
    /// The immutable data closure can only originate in checked publication.
    pub fn from_identity(
        data: Arc<StaticData>,
        identity: RuntimeTypeIdentity,
        types: &dyn TypeView,
    ) -> Result<Self, StaticDataError> {
        let value = Self {
            data,
            identity: Arc::new(identity),
        };
        value.validate_identity(types)?;
        Ok(value)
    }
    pub fn ty(&self) -> TypeId {
        self.identity.schema.ty()
    }
    pub fn identity(&self) -> RuntimeTypeIdentity {
        *self.identity
    }
    pub fn descriptor_type(&self) -> TypeId {
        self.identity.schema.descriptor_type()
    }
    pub fn data(&self) -> &Arc<StaticData> {
        &self.data
    }
    pub fn address(&self) -> &StaticAddress {
        self.data
            .object(self.identity.object)
            .expect("checked immutable descriptor")
            .descriptor_header()
            .expect("checked binding")
    }
    pub fn validate(&self, types: &dyn TypeView) -> Result<(), StaticDataError> {
        self.data.validate(types)?;
        self.validate_identity(types)
    }
    /// The caller must have validated this immutable data closure already.
    pub(crate) fn validate_identity(&self, types: &dyn TypeView) -> Result<(), StaticDataError> {
        self.identity.validate(types)?;
        let object = self.data.object(self.identity.object)?;
        if object.runtime_type_identity() != Some(*self.identity) {
            return Err(StaticDataError::InvalidValue(self.identity.ty));
        }
        Ok(())
    }
}

mod payload;
impl PartialEq for RuntimeTypeConstant {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}
impl Eq for RuntimeTypeConstant {
}
impl Hash for RuntimeTypeConstant {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identity.hash(state);
    }
}

#[cfg(test)]
mod tests;
