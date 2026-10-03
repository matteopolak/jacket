//! Integer bits and virtual-address provenance travel together through VM dataflow.
use crate::{Pointer, Value};
use jai_types::{Integer, IntegerType};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AddressProvenance {
    Pointer(Pointer),
    Derived {
        memory: u64,
        allocations: Box<[u64]>,
    },
}

/// Plain literals never acquire provenance merely by matching an address bit pattern.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Number {
    integer: Integer,
    provenance: Option<AddressProvenance>,
}
impl Number {
    pub fn plain(integer: Integer) -> Self {
        Self {
            integer,
            provenance: None,
        }
    }
    pub(crate) fn address(integer: Integer, provenance: AddressProvenance) -> Self {
        let provenance = match provenance {
            AddressProvenance::Derived {
                memory,
                allocations,
            } => {
                let mut allocations = allocations.into_vec();
                allocations.sort_unstable();
                allocations.dedup();
                AddressProvenance::Derived {
                    memory,
                    allocations: allocations.into_boxed_slice(),
                }
            }
            provenance => provenance,
        };
        Self {
            integer,
            provenance: Some(provenance),
        }
    }
    pub fn portable_integer(&self) -> Result<Integer, crate::Error> {
        if self.provenance.is_some() {
            return Err(crate::Error::UnsupportedPointerOperation(
                "address-derived integer cannot be used as a target-independent scalar",
            ));
        }
        Ok(self.integer)
    }
    pub fn integer(&self) -> Integer {
        self.integer
    }
    pub fn ty(&self) -> IntegerType {
        self.integer.ty()
    }
    pub fn bits(&self) -> u64 {
        self.integer.bits()
    }
    pub fn value(&self) -> i128 {
        self.integer.value()
    }
    pub fn provenance(&self) -> Option<&AddressProvenance> {
        self.provenance.as_ref()
    }
    pub(crate) fn memory_identity(&self) -> Option<u64> {
        self.provenance.as_ref().map(|provenance| match provenance {
            AddressProvenance::Pointer(pointer) => pointer.memory_identity(),
            AddressProvenance::Derived { memory, .. } => *memory,
        })
    }
    pub(crate) fn origin_count(&self) -> usize {
        match &self.provenance {
            Some(AddressProvenance::Pointer(_)) => 1,
            Some(AddressProvenance::Derived { allocations, .. }) => allocations.len(),
            None => 0,
        }
    }
    /// Origin IDs and any dynamic projection path retained by an exact pointer.
    pub(crate) fn metadata_cells(&self) -> usize {
        self.origin_count().saturating_add(match &self.provenance {
            Some(AddressProvenance::Pointer(pointer)) => pointer.metadata_cells(),
            _ => 0,
        })
    }
    pub(crate) fn allocation_ids_iter(&self) -> impl Iterator<Item = u64> + '_ {
        let (single, many): (Option<u64>, &[u64]) = match &self.provenance {
            Some(AddressProvenance::Pointer(pointer)) => {
                (pointer.data_allocation_key().map(|key| key.1), &[])
            }
            Some(AddressProvenance::Derived { allocations, .. }) => (None, allocations),
            None => (None, &[]),
        };
        single.into_iter().chain(many.iter().copied())
    }
    pub(crate) fn allocation_ids(&self) -> Vec<u64> {
        self.allocation_ids_iter().collect()
    }
    pub fn into_value(self) -> Value {
        if self.provenance.is_some() {
            Value::AddressInteger(self)
        } else {
            Value::Int(self.integer)
        }
    }
}

#[cfg(test)]
#[path = "number_tests.rs"]
mod tests;
