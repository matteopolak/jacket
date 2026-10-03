//! Canonical runtime Type identities are certified by immutable static objects.
use super::*;

impl Memory {
    pub(crate) fn register_runtime_type(
        &mut self,
        types: &dyn TypeView,
        descriptor: &Pointer,
        identity: jai_ir::RuntimeTypeIdentity,
    ) -> Result<(), Error> {
        identity.validate(types)?;
        if identity.policy() != self.target.policy {
            return Err(Error::InvalidIr(
                "runtime Type descriptor uses a different target layout",
            ));
        }
        self.validate_pointer(types, descriptor)?;
        self.validate_access(types, descriptor)?;
        if descriptor.pointee != identity.schema().header_type() {
            return Err(Error::InvalidIr(
                "runtime Type binding has the wrong header type",
            ));
        }
        let key = (
            descriptor.allocation_id(),
            self.byte_offset(types, descriptor)?,
        );
        if let Some(old) = self.runtime_types.get(&key) {
            return if *old == identity {
                Ok(())
            } else {
                Err(Error::InvalidIr("runtime Type descriptor identity changed"))
            };
        }
        if self.runtime_types.len() >= self.limits.value_cells {
            return Err(Error::Limit(LimitKind::ValueCells));
        }
        self.runtime_types.insert(key, identity);
        Ok(())
    }

    pub(crate) fn runtime_type_identity(
        &self,
        types: &dyn TypeView,
        value: &Value,
    ) -> Result<jai_ir::RuntimeTypeIdentity, Error> {
        let Value::Type {
            descriptor,
        } = value
        else {
            return Err(Error::InvalidIr("expected runtime Type value"));
        };
        let descriptor = descriptor.as_ref().ok_or(Error::NullPointer)?;
        if types.runtime_type_header() != Some(descriptor.pointee) {
            return Err(Error::InvalidIr(
                "runtime Type value has the wrong header type",
            ));
        }
        self.validate_pointer(types, descriptor)?;
        self.validate_access(types, descriptor)?;
        let key = (
            descriptor.allocation_id(),
            self.byte_offset(types, descriptor)?,
        );
        let identity = self
            .runtime_types
            .get(&key)
            .copied()
            .ok_or(Error::InvalidIr(
                "runtime Type value does not name a canonical descriptor",
            ))?;
        identity.validate(types)?;
        if identity.policy() != self.target.policy {
            return Err(Error::InvalidIr(
                "runtime Type descriptor uses a different target layout",
            ));
        }
        Ok(identity)
    }

    pub(crate) fn validate_runtime_type_values(
        &self,
        types: &dyn TypeView,
        root: &Value,
    ) -> Result<(), Error> {
        let mut remaining = self.limits.value_cells;
        let mut pending = vec![root];
        while let Some(value) = pending.pop() {
            remaining = remaining
                .checked_sub(1)
                .ok_or(Error::Limit(LimitKind::ValueCells))?;
            // Descriptor data fields are numeric pointer values too. Check their
            // target domain before retaining any owner, while still traversing
            // a DynamicArray allocator's independent typed payload below.
            let pointer = match value {
                Value::Pointer(pointer)
                | Value::Slice {
                    pointer, ..
                }
                | Value::StringView {
                    pointer, ..
                }
                | Value::DynamicArray {
                    pointer, ..
                } => Some(pointer),
                _ => None,
            };
            if let Some(pointer) = pointer
                && pointer.is_opaque()
            {
                self.validate_opaque_address(types, pointer)?;
            }
            match value {
                Value::Pointer(pointer) if pointer.code_pointer().is_some() => {
                    self.validate_pointer(types, pointer)?;
                }
                Value::AddressInteger(number) => {
                    if let Some(crate::AddressProvenance::Pointer(pointer)) = number.provenance()
                        && pointer.code_pointer().is_some()
                    {
                        self.validate_pointer(types, pointer)?;
                    }
                }
                Value::StoredAggregate(snapshot) => {
                    self.validate_stored_aggregate(types, snapshot)?;
                    if let Some(semantic) = snapshot.decoded_semantic() {
                        pending.push(semantic);
                    }
                }
                Value::Type {
                    descriptor: Some(_),
                } => {
                    self.runtime_type_identity(types, value)?;
                }
                Value::Record {
                    fields, ..
                }
                | Value::Array {
                    elements: fields, ..
                } => {
                    if fields.len() > remaining {
                        return Err(Error::Limit(LimitKind::ValueCells));
                    }
                    pending.extend(fields);
                }
                Value::Union {
                    value, ..
                }
                | Value::Distinct {
                    value, ..
                } => pending.push(value),
                Value::DynamicArray {
                    allocator: Some(value),
                    ..
                } => pending.push(value),
                _ => {}
            }
        }
        Ok(())
    }
}
