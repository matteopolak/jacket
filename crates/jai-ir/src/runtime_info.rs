//! Certified compile-time Runtime_Info storage, separate from source eligibility.
use crate::{
    ConstantKind, RuntimeTypeIdentity, StaticAddress, StaticData, StaticDataError,
    StaticDataLimits, StaticObjectId, StaticProjection, StaticValue, StaticValueKind,
};
use jai_types::{
    LayoutPolicy, RuntimeInfoError, RuntimeInfoField, RuntimeInfoSchema, TypeId, TypeView,
};
use std::{collections::HashSet, fmt, sync::Arc};

/// An immutable table whose ordered rows name canonical descriptor objects.
/// The source catalog chooses the eligible rows before building backing storage;
/// this proof checks membership and storage, not source registration provenance.
#[derive(Clone, Debug)]
pub struct RuntimeInfoSnapshot {
    data: Arc<StaticData>,
    address: StaticAddress,
    schema: RuntimeInfoSchema,
    policy: LayoutPolicy,
    rows: Arc<[RuntimeTypeIdentity]>,
}

#[derive(Debug)]
pub enum RuntimeInfoSnapshotError {
    Schema(RuntimeInfoError),
    Data(StaticDataError),
    TargetMismatch,
    InvalidRecord,
    InvalidTypeTable,
    GlobalDataNotNull,
    DescriptorMismatch,
    DuplicateType(TypeId),
}
impl From<RuntimeInfoError> for RuntimeInfoSnapshotError {
    fn from(value: RuntimeInfoError) -> Self {
        Self::Schema(value)
    }
}
impl From<StaticDataError> for RuntimeInfoSnapshotError {
    fn from(value: StaticDataError) -> Self {
        Self::Data(value)
    }
}
impl fmt::Display for RuntimeInfoSnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Schema(error) => error.fmt(f),
            Self::Data(error) => error.fmt(f),
            Self::TargetMismatch => f.write_str("Runtime_Info snapshot was built for a different target layout"),
            Self::InvalidRecord => f.write_str("Runtime_Info object does not match its source schema"),
            Self::InvalidTypeTable => f.write_str("Runtime_Info type table does not match the sealed ordered type set"),
            Self::GlobalDataNotNull => f.write_str("compile-time Runtime_Info.global_data_info must be null"),
            Self::DescriptorMismatch => f.write_str("Runtime_Info table row is not the exact canonical descriptor header for its type and target"),
            Self::DuplicateType(_) => f.write_str("Runtime_Info checkpoint contains a duplicate type"),
        }
    }
}
impl std::error::Error for RuntimeInfoSnapshotError {}

impl RuntimeInfoSnapshot {
    pub fn new_compile_time(
        data: Arc<StaticData>,
        object: StaticObjectId,
        schema: RuntimeInfoSchema,
        types: &dyn TypeView,
        policy: LayoutPolicy,
        expected_ordered_types: &[TypeId],
    ) -> Result<Self, RuntimeInfoSnapshotError> {
        // Reject unbounded external checkpoints before allocating proof metadata.
        if expected_ordered_types.len() > StaticDataLimits::default().value_nodes {
            return Err(StaticDataError::Limit("Runtime_Info row count").into());
        }
        schema.layout(types, policy)?;
        data.validate(types)?;
        // Consumers retain this entire immutable publication, including rows'
        // descriptor dependencies and earlier objects in the same arena.
        for object in data.objects() {
            if let Some(identity) = object.runtime_type_identity() {
                if identity.policy() != policy {
                    return Err(RuntimeInfoSnapshotError::TargetMismatch);
                }
                if identity.schema() != schema.runtime_type_schema() {
                    return Err(RuntimeInfoSnapshotError::DescriptorMismatch);
                }
            }
        }
        let value = data.object(object)?.value();
        if value.ty != schema.ty() {
            return Err(RuntimeInfoSnapshotError::InvalidRecord);
        }
        let StaticValueKind::Record(fields) = &value.kind else {
            return Err(RuntimeInfoSnapshotError::InvalidRecord);
        };
        let [table, global] = fields.as_slice() else {
            return Err(RuntimeInfoSnapshotError::InvalidRecord);
        };
        if table.ty != schema.field(RuntimeInfoField::TypeTable).ty
            || global.ty != schema.field(RuntimeInfoField::GlobalDataInfo).ty
        {
            return Err(RuntimeInfoSnapshotError::InvalidRecord);
        }
        if !matches!(&global.kind, StaticValueKind::Constant(value) if value.ty == global.ty && matches!(value.kind, ConstantKind::Zero))
        {
            return Err(RuntimeInfoSnapshotError::GlobalDataNotNull);
        }
        let table = table_rows(&data, table, expected_ordered_types.len())?;
        let mut seen = HashSet::with_capacity(expected_ordered_types.len());
        let mut rows = Vec::with_capacity(expected_ordered_types.len());
        for (row, expected) in table.iter().zip(expected_ordered_types.iter().copied()) {
            types.kind(expected).map_err(StaticDataError::from)?;
            if !seen.insert(expected) {
                return Err(RuntimeInfoSnapshotError::DuplicateType(expected));
            }
            if row.ty != schema.runtime_type_schema().descriptor_type() {
                return Err(RuntimeInfoSnapshotError::DescriptorMismatch);
            }
            let StaticValueKind::Address(address) = &row.kind else {
                return Err(RuntimeInfoSnapshotError::DescriptorMismatch);
            };
            let descriptor = data.object(address.object())?;
            let identity = descriptor
                .runtime_type_identity()
                .ok_or(RuntimeInfoSnapshotError::DescriptorMismatch)?;
            if descriptor.descriptor_header() != Some(address)
                || identity.ty() != expected
                || identity.schema() != schema.runtime_type_schema()
                || identity.policy() != policy
            {
                return Err(RuntimeInfoSnapshotError::DescriptorMismatch);
            }
            rows.push(identity);
        }
        Ok(Self {
            data,
            address: StaticAddress::new(object),
            schema,
            policy,
            rows: rows.into(),
        })
    }

    pub fn data(&self) -> &Arc<StaticData> {
        &self.data
    }
    pub fn address(&self) -> &StaticAddress {
        &self.address
    }
    pub fn schema(&self) -> RuntimeInfoSchema {
        self.schema
    }
    pub fn policy(&self) -> LayoutPolicy {
        self.policy
    }
    pub fn rows(&self) -> &[RuntimeTypeIdentity] {
        &self.rows
    }
    pub fn represented_types(&self) -> impl ExactSizeIterator<Item = TypeId> + '_ {
        self.rows.iter().map(|row| row.ty())
    }
    /// Check the issuing type arena and fixed source schema without traversing
    /// rows or static storage. Consumers still admit storage through their
    /// metered static-graph boundary; this is not an execution-work receipt.
    pub fn validate_owner(
        &self,
        types: &dyn TypeView,
        policy: LayoutPolicy,
    ) -> Result<(), RuntimeInfoSnapshotError> {
        if policy != self.policy {
            return Err(RuntimeInfoSnapshotError::TargetMismatch);
        }
        self.schema.revalidate(types)?;
        Ok(())
    }
    /// Recheck owner, layout, and immutable descriptor closure at a consumer boundary.
    pub fn revalidate(
        &self,
        types: &dyn TypeView,
        policy: LayoutPolicy,
    ) -> Result<(), RuntimeInfoSnapshotError> {
        self.validate_owner(types, policy)?;
        schema_revalidate(self, types, policy)?;
        Ok(())
    }
}

fn schema_revalidate(
    snapshot: &RuntimeInfoSnapshot,
    types: &dyn TypeView,
    policy: LayoutPolicy,
) -> Result<(), RuntimeInfoSnapshotError> {
    snapshot.schema.layout(types, policy)?;
    snapshot.data.validate(types)?;
    for row in snapshot.rows.iter().copied() {
        row.validate(types).map_err(StaticDataError::from)?;
        let object = snapshot.data.object(row.object())?;
        if object.runtime_type_identity() != Some(row) {
            return Err(RuntimeInfoSnapshotError::DescriptorMismatch);
        }
    }
    Ok(())
}

fn resolve<'a>(
    data: &'a StaticData,
    address: &StaticAddress,
) -> Result<&'a StaticValue, RuntimeInfoSnapshotError> {
    let mut value = data.object(address.object())?.value();
    for step in address.path() {
        value = match (&value.kind, step) {
            (StaticValueKind::Record(fields), StaticProjection::Field(field)) => {
                fields.get(field.index())
            }
            (StaticValueKind::Array(values), StaticProjection::Index(index)) => {
                usize::try_from(*index)
                    .ok()
                    .and_then(|index| values.get(index))
            }
            _ => None,
        }
        .ok_or(RuntimeInfoSnapshotError::InvalidTypeTable)?;
    }
    Ok(value)
}

fn table_rows<'a>(
    data: &'a StaticData,
    table: &StaticValue,
    expected: usize,
) -> Result<&'a [StaticValue], RuntimeInfoSnapshotError> {
    let StaticValueKind::Slice {
        data: address,
        count,
    } = &table.kind
    else {
        return Err(RuntimeInfoSnapshotError::InvalidTypeTable);
    };
    if usize::try_from(*count).ok() != Some(expected) {
        return Err(RuntimeInfoSnapshotError::InvalidTypeTable);
    }
    let Some(address) = address else {
        return if expected == 0 {
            Ok(&[])
        } else {
            Err(RuntimeInfoSnapshotError::InvalidTypeTable)
        };
    };
    let Some((StaticProjection::Index(start), path)) = address.path().split_last() else {
        return Err(RuntimeInfoSnapshotError::InvalidTypeTable);
    };
    let parent = path
        .iter()
        .fold(StaticAddress::new(address.object()), |address, step| {
            address.project(step.clone())
        });
    let StaticValueKind::Array(values) = &resolve(data, &parent)?.kind else {
        return Err(RuntimeInfoSnapshotError::InvalidTypeTable);
    };
    let start = usize::try_from(*start).map_err(|_| RuntimeInfoSnapshotError::InvalidTypeTable)?;
    let end = start
        .checked_add(expected)
        .ok_or(RuntimeInfoSnapshotError::InvalidTypeTable)?;
    values
        .get(start..end)
        .ok_or(RuntimeInfoSnapshotError::InvalidTypeTable)
}

#[cfg(test)]
mod tests;
