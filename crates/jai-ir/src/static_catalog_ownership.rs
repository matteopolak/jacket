//! Borrowed admission of the complete union of strongly retained static catalogs.
use crate::{StaticData, StaticDataError, StaticObjectId};
use std::sync::{Arc, Weak};

#[derive(Debug)]
pub enum StaticCatalogAdmissionError<E> {
    Work(E),
    Storage(StaticDataError),
}
impl<E> From<StaticDataError> for StaticCatalogAdmissionError<E> {
    fn from(error: StaticDataError) -> Self {
        Self::Storage(error)
    }
}

/// No catalog payload is strongly retained by this cache. Weak table identity
/// prevents address reuse while a taken controller owns the genuine roots.
pub struct StaticCatalogOwnership {
    tables: Vec<Weak<StaticData>>,
    objects: Vec<(StaticObjectId, usize)>,
    bytes: usize,
}
impl StaticCatalogOwnership {
    pub fn payload_bytes(&self) -> usize {
        self.bytes
    }
    pub fn receipt_bytes(&self) -> usize {
        self.tables.capacity() * std::mem::size_of::<Weak<StaticData>>()
            + self.objects.capacity() * std::mem::size_of::<(StaticObjectId, usize)>()
    }
    pub fn accounted_bytes(&self) -> usize {
        self.bytes + self.receipt_bytes()
    }
    /// Credit an existing owner only after scanning the exact current roots.
    /// The comparison is independent of visitation order and retains no Arc.
    pub fn matches_prefix<E>(
        &self,
        prefix: &StaticCatalogBudget<'_>,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<bool, E> {
        charge(1)?;
        if self.bytes != prefix.bytes
            || self.tables.len() != prefix.tables.len()
            || self.objects.len() != prefix.objects.len()
        {
            return Ok(false);
        }
        for table in &self.tables {
            let mut found = false;
            for candidate in &prefix.tables {
                charge(1)?;
                if table.as_ptr() == candidate.as_ptr() {
                    found = true;
                    break;
                }
            }
            if !found {
                return Ok(false);
            }
        }
        for object in &self.objects {
            let mut found = false;
            for candidate in &prefix.objects {
                charge(1)?;
                if object == candidate {
                    found = true;
                    break;
                }
            }
            if !found {
                return Ok(false);
            }
        }
        Ok(true)
    }
    /// A local receipt can share payload only when every actual table and
    /// immutable object fact belongs to this already admitted common owner.
    pub fn covers_prefix<E>(
        &self,
        prefix: &StaticCatalogBudget<'_>,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<bool, E> {
        self.covers_facts(&prefix.tables, &prefix.objects, prefix.bytes, charge)
    }
    fn covers_facts<E>(
        &self,
        tables: &[Weak<StaticData>],
        objects: &[(StaticObjectId, usize)],
        bytes: usize,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<bool, E> {
        charge(1)?;
        if bytes > self.bytes {
            return Ok(false);
        }
        for table in tables {
            let mut found = false;
            for candidate in &self.tables {
                charge(1)?;
                if table.as_ptr() == candidate.as_ptr() {
                    found = true;
                    break;
                }
            }
            if !found {
                return Ok(false);
            }
        }
        for object in objects {
            let mut found = false;
            for candidate in &self.objects {
                charge(1)?;
                if object == candidate {
                    found = true;
                    break;
                }
            }
            if !found {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn covers_table<E>(
        &self,
        data: &Arc<StaticData>,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<bool, E> {
        // Each sealed table was admitted together with its complete immutable
        // object closure. Exact Arc identity certifies those unchanged facts.
        for table in &self.tables {
            charge(1)?;
            if table.as_ptr() == Arc::as_ptr(data) {
                return Ok(true);
            }
        }
        Ok(false)
    }
    /// Old-only weak tokens retain an allocation header until the receipt is
    /// swapped. Shared current tables already contribute their complete header.
    pub fn retired_header_overlap<E>(
        &self,
        current: &StaticCatalogBudget<'_>,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<usize, StaticCatalogAdmissionError<E>> {
        let header = std::mem::size_of::<StaticData>()
            .checked_add(2 * std::mem::size_of::<usize>())
            .ok_or(StaticDataError::Limit("catalog retained bytes"))?;
        let mut bytes = 0usize;
        for table in &self.tables {
            charge(1).map_err(StaticCatalogAdmissionError::Work)?;
            let mut found = false;
            for candidate in &current.tables {
                charge(1).map_err(StaticCatalogAdmissionError::Work)?;
                if table.as_ptr() == candidate.as_ptr() {
                    found = true;
                    break;
                }
            }
            if !found {
                bytes = bytes
                    .checked_add(header)
                    .ok_or(StaticDataError::Limit("catalog retained bytes"))?;
            }
        }
        Ok(bytes)
    }
}

/// Scratch retains weak table identities and sealed object facts. Only optional
/// coverage proofs are borrowed; source visitors need not lend their Arc. Every
/// candidate is admitted before the caller retains another strong Arc.
pub struct StaticCatalogBudget<'a> {
    tables: Vec<Weak<StaticData>>,
    objects: Vec<(StaticObjectId, usize)>,
    external_receipt_bytes: usize,
    external_payload: Option<&'a StaticCatalogOwnership>,
    external_payload_bytes: usize,
    coverage: Option<&'a StaticCatalogOwnership>,
    bytes: usize,
}
impl<'a> StaticCatalogBudget<'a> {
    pub fn new() -> Self {
        Self {
            tables: Vec::new(),
            objects: Vec::new(),
            external_receipt_bytes: 0,
            external_payload: None,
            external_payload_bytes: 0,
            coverage: None,
            bytes: 0,
        }
    }
    pub fn into_ownership(self) -> StaticCatalogOwnership {
        StaticCatalogOwnership {
            tables: self.tables,
            objects: self.objects,
            bytes: self.bytes,
        }
    }
    /// Seed the exact previously admitted root union while its owner is taken
    /// out of the VM. Rebuild from genuine live roots when that owner returns.
    pub fn from_ownership<E>(
        owned: &'a StaticCatalogOwnership,
        available: usize,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Self, StaticCatalogAdmissionError<E>> {
        owned
            .accounted_bytes()
            .checked_add(owned.receipt_bytes())
            .filter(|bytes| *bytes <= available)
            .ok_or(StaticDataError::Limit("catalog retained bytes"))?;
        charge(owned.tables.len() + owned.objects.len())
            .map_err(StaticCatalogAdmissionError::Work)?;
        let budget = Self {
            tables: owned.tables.clone(),
            objects: owned.objects.clone(),
            bytes: owned.bytes,
            external_receipt_bytes: owned.receipt_bytes(),
            external_payload: None,
            external_payload_bytes: 0,
            coverage: None,
        };
        budget.check_retained(available)?;
        Ok(budget)
    }
    /// Receipt-only admission under an exact genuine common-owner proof.
    /// Candidates absent from that proof fail before another Weak is retained.
    pub fn covered_new<E>(
        coverage: &'a StaticCatalogOwnership,
        available: usize,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Self, StaticCatalogAdmissionError<E>> {
        charge(1).map_err(StaticCatalogAdmissionError::Work)?;
        let budget = Self {
            coverage: Some(coverage),
            ..Self::new()
        };
        budget.check_retained(available)?;
        Ok(budget)
    }
    pub fn covered_from_ownership<E>(
        old: &'a StaticCatalogOwnership,
        coverage: &'a StaticCatalogOwnership,
        available: usize,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Self, StaticCatalogAdmissionError<E>> {
        if !coverage
            .covers_facts(&old.tables, &old.objects, old.bytes, charge)
            .map_err(StaticCatalogAdmissionError::Work)?
        {
            return Err(StaticDataError::Limit("catalog coverage changed").into());
        }
        old.receipt_bytes()
            .checked_mul(2)
            .filter(|bytes| *bytes <= available)
            .ok_or(StaticDataError::Limit("catalog retained bytes"))?;
        charge(old.tables.len() + old.objects.len()).map_err(StaticCatalogAdmissionError::Work)?;
        let budget = Self {
            tables: old.tables.clone(),
            objects: old.objects.clone(),
            bytes: old.bytes,
            external_receipt_bytes: old.receipt_bytes(),
            coverage: Some(coverage),
            ..Self::new()
        };
        budget.check_retained(available)?;
        Ok(budget)
    }
    /// Prepare an exact survivor union while the original owner is still live.
    /// Shared payload transfers its existing charge into the fresh receipt;
    /// departing old payload stays reserved until publication swaps owners.
    pub fn fresh_from_ownership<E>(
        owned: &'a StaticCatalogOwnership,
        available: usize,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Self, StaticCatalogAdmissionError<E>> {
        charge(1).map_err(StaticCatalogAdmissionError::Work)?;
        let budget = Self {
            external_receipt_bytes: owned.receipt_bytes(),
            external_payload: Some(owned),
            external_payload_bytes: owned.payload_bytes(),
            ..Self::new()
        };
        budget.check_retained(available)?;
        Ok(budget)
    }
    pub fn bytes(&self) -> usize {
        self.bytes
    }
    pub fn receipt_bytes(&self) -> Result<usize, StaticDataError> {
        self.tables
            .capacity()
            .checked_mul(std::mem::size_of::<Weak<StaticData>>())
            .and_then(|bytes| {
                self.objects
                    .capacity()
                    .checked_mul(std::mem::size_of::<(StaticObjectId, usize)>())
                    .and_then(|objects| bytes.checked_add(objects))
            })
            .ok_or(StaticDataError::Limit("catalog retained bytes"))
    }
    pub fn accounted_bytes(&self) -> Result<usize, StaticDataError> {
        self.charged_payload_bytes()?
            .checked_add(self.external_receipt_bytes)
            .and_then(|bytes| bytes.checked_add(self.external_payload_bytes))
            .and_then(|bytes| bytes.checked_add(self.receipt_bytes().ok()?))
            .ok_or(StaticDataError::Limit("catalog retained bytes"))
    }

    /// Merge a receipt scanned from genuine independently retained roots.
    /// The caller keeps those owners alive throughout this operation. Weak
    /// upgrades inspect sealed tables; no catalog payload is copied or stored.
    /// A rejected forest restores every prior accepted fact, but not work or
    /// admitted replacement-buffer spare capacity.
    pub fn admit_ownership<E>(
        &mut self,
        owned: &StaticCatalogOwnership,
        available: usize,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<usize, StaticCatalogAdmissionError<E>> {
        self.check_retained(available)?;
        let before = (
            self.bytes,
            self.tables.len(),
            self.objects.len(),
            self.external_payload_bytes,
        );
        let result = (|| {
            let visits = owned
                .tables
                .capacity()
                .checked_add(owned.objects.capacity())
                .ok_or(StaticDataError::Limit("catalog ownership work"))?;
            charge(visits).map_err(StaticCatalogAdmissionError::Work)?;
            for table in &owned.tables {
                charge(1).map_err(StaticCatalogAdmissionError::Work)?;
                let data = table
                    .upgrade()
                    .ok_or(StaticDataError::Limit("catalog owner retired"))?;
                self.admit(&data, available, charge)?;
            }
            // Bind the merged union to every sealed source object fact as well
            // as table identity; a scalar payload credit alone is insufficient.
            for object in &owned.objects {
                let mut found = false;
                for candidate in &self.objects {
                    charge(1).map_err(StaticCatalogAdmissionError::Work)?;
                    if object == candidate {
                        found = true;
                        break;
                    }
                }
                if !found {
                    return Err(StaticDataError::Limit("catalog owner facts changed").into());
                }
            }
            self.check_retained(available)?;
            Ok(self.bytes - before.0)
        })();
        if result.is_err() {
            self.bytes = before.0;
            self.tables.truncate(before.1);
            self.objects.truncate(before.2);
            self.external_payload_bytes = before.3;
        }
        result
    }
    pub fn admit<E>(
        &mut self,
        data: &Arc<StaticData>,
        available: usize,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<usize, StaticCatalogAdmissionError<E>> {
        self.check_retained(available)?;
        let before = (
            self.bytes,
            self.tables.len(),
            self.objects.len(),
            self.external_payload_bytes,
        );
        let result = self.admit_inner(data, available, charge);
        if result.is_err() {
            self.bytes = before.0;
            self.tables.truncate(before.1);
            self.objects.truncate(before.2);
            self.external_payload_bytes = before.3;
        }
        result
    }
    fn admit_inner<E>(
        &mut self,
        data: &Arc<StaticData>,
        available: usize,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<usize, StaticCatalogAdmissionError<E>> {
        let before = self.bytes;
        for old in &self.tables {
            charge(1).map_err(StaticCatalogAdmissionError::Work)?;
            if old.as_ptr() == Arc::as_ptr(data) {
                return Ok(0);
            }
        }
        if let Some(coverage) = self.coverage
            && !coverage
                .covers_table(data, charge)
                .map_err(StaticCatalogAdmissionError::Work)?
        {
            return Err(StaticDataError::Limit("catalog coverage changed").into());
        }
        if let Some(external) = self.external_payload {
            for old in &external.tables {
                charge(1).map_err(StaticCatalogAdmissionError::Work)?;
                if old.as_ptr() == Arc::as_ptr(data) {
                    self.transfer_payload(data.table_retained_bytes())?;
                    break;
                }
            }
        }
        self.retain_bytes(data.table_retained_bytes(), available)?;
        self.prepare_scratch(1, 0, available, charge)?;
        self.tables.push(Arc::downgrade(data));
        // This is a sealed receipt traversal, not a recursive payload inspection.
        for object in data.objects() {
            charge(1).map_err(StaticCatalogAdmissionError::Work)?;
            let mut shared = false;
            for old in &self.objects {
                charge(1).map_err(StaticCatalogAdmissionError::Work)?;
                if old.0 == object.id() {
                    shared = true;
                    break;
                }
            }
            if shared {
                continue;
            }
            if let Some(external) = self.external_payload {
                for old in &external.objects {
                    charge(1).map_err(StaticCatalogAdmissionError::Work)?;
                    if old.0 == object.id() {
                        if old.1 != object.retained_bytes() {
                            return Err(
                                StaticDataError::Limit("catalog object receipt changed").into()
                            );
                        }
                        self.transfer_payload(old.1)?;
                        break;
                    }
                }
            }
            self.retain_bytes(object.retained_bytes(), available)?;
            self.prepare_scratch(0, 1, available, charge)?;
            self.objects.push((object.id(), object.retained_bytes()));
        }
        Ok(self.bytes - before)
    }
    fn check_retained(&self, available: usize) -> Result<(), StaticDataError> {
        let scratch = self
            .tables
            .capacity()
            .checked_mul(std::mem::size_of::<Weak<StaticData>>())
            .and_then(|bytes| {
                self.objects
                    .capacity()
                    .checked_mul(std::mem::size_of::<(StaticObjectId, usize)>())
                    .and_then(|objects| bytes.checked_add(objects))
            })
            .ok_or(StaticDataError::Limit("catalog retained bytes"))?;
        self.charged_payload_bytes()?
            .checked_add(self.external_receipt_bytes)
            .and_then(|bytes| bytes.checked_add(self.external_payload_bytes))
            .and_then(|bytes| bytes.checked_add(scratch))
            .filter(|bytes| *bytes <= available)
            .ok_or(StaticDataError::Limit("catalog retained bytes"))?;
        Ok(())
    }
    fn transfer_payload(&mut self, bytes: usize) -> Result<(), StaticDataError> {
        self.external_payload_bytes = self
            .external_payload_bytes
            .checked_sub(bytes)
            .ok_or(StaticDataError::Limit("catalog object receipt changed"))?;
        Ok(())
    }
    fn charged_payload_bytes(&self) -> Result<usize, StaticDataError> {
        if self.coverage.is_some() {
            Ok(0)
        } else {
            Ok(self.bytes)
        }
    }
    fn retain_bytes(&mut self, bytes: usize, available: usize) -> Result<(), StaticDataError> {
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .filter(|bytes| self.coverage.is_some() || *bytes <= available)
            .ok_or(StaticDataError::Limit("catalog retained bytes"))?;
        Ok(())
    }
    fn prepare_scratch<E>(
        &mut self,
        tables: usize,
        objects: usize,
        available: usize,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<(), StaticCatalogAdmissionError<E>> {
        let table_capacity = self
            .tables
            .len()
            .checked_add(tables)
            .ok_or(StaticDataError::Limit("catalog retained bytes"))?;
        let object_capacity = self
            .objects
            .len()
            .checked_add(objects)
            .ok_or(StaticDataError::Limit("catalog retained bytes"))?;
        let table_capacity = table_capacity.max(self.tables.capacity());
        let object_capacity = object_capacity.max(self.objects.capacity());
        let table_growth = table_capacity > self.tables.capacity();
        let object_growth = object_capacity > self.objects.capacity();
        // Replacement buffers coexist with the old backing until all capacity
        // proofs pass. Failed preparation leaves the old receipt untouched.
        let predicted = table_capacity
            .checked_mul(std::mem::size_of::<Weak<StaticData>>())
            .and_then(|bytes| {
                object_capacity
                    .checked_mul(std::mem::size_of::<(StaticObjectId, usize)>())
                    .and_then(|objects| bytes.checked_add(objects))
            })
            .ok_or(StaticDataError::Limit("catalog retained bytes"))?;
        let old_backing = if table_growth {
            self.tables.capacity() * std::mem::size_of::<Weak<StaticData>>()
        } else {
            0
        }
        .checked_add(if object_growth {
            self.objects.capacity() * std::mem::size_of::<(StaticObjectId, usize)>()
        } else {
            0
        })
        .ok_or(StaticDataError::Limit("catalog retained bytes"))?;
        self.charged_payload_bytes()?
            .checked_add(self.external_receipt_bytes)
            .and_then(|bytes| bytes.checked_add(self.external_payload_bytes))
            .and_then(|bytes| bytes.checked_add(predicted))
            .and_then(|bytes| bytes.checked_add(old_backing))
            .filter(|bytes| *bytes <= available)
            .ok_or(StaticDataError::Limit("catalog retained bytes"))?;
        let relocation = if table_growth {
            self.tables.len()
        } else {
            0
        } + if object_growth {
            self.objects.len()
        } else {
            0
        };
        charge(tables + objects + relocation).map_err(StaticCatalogAdmissionError::Work)?;
        let mut new_tables = Vec::new();
        let mut new_objects = Vec::new();
        if table_growth {
            new_tables
                .try_reserve_exact(table_capacity)
                .map_err(|_| StaticDataError::Limit("catalog retained bytes"))?;
        }
        if object_growth {
            new_objects
                .try_reserve_exact(object_capacity)
                .map_err(|_| StaticDataError::Limit("catalog retained bytes"))?;
        }
        let actual = if table_growth {
            new_tables.capacity()
        } else {
            self.tables.capacity()
        }
        .checked_mul(std::mem::size_of::<Weak<StaticData>>())
        .and_then(|bytes| {
            (if object_growth {
                new_objects.capacity()
            } else {
                self.objects.capacity()
            })
            .checked_mul(std::mem::size_of::<(StaticObjectId, usize)>())
            .and_then(|objects| bytes.checked_add(objects))
        })
        .ok_or(StaticDataError::Limit("catalog retained bytes"))?;
        self.charged_payload_bytes()?
            .checked_add(self.external_receipt_bytes)
            .and_then(|bytes| bytes.checked_add(self.external_payload_bytes))
            .and_then(|bytes| bytes.checked_add(actual))
            .and_then(|bytes| bytes.checked_add(old_backing))
            .filter(|bytes| *bytes <= available)
            .ok_or(StaticDataError::Limit("catalog retained bytes"))?;
        if table_growth {
            new_tables.append(&mut self.tables);
            self.tables = new_tables;
        }
        if object_growth {
            new_objects.append(&mut self.objects);
            self.objects = new_objects;
        }
        Ok(())
    }
}

/// Admit the complete shared catalog union against a retained byte allowance.
pub fn measure_static_catalogs<'a, E>(
    catalogs: impl IntoIterator<Item = &'a Arc<StaticData>>,
    available: usize,
    charge: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<usize, StaticCatalogAdmissionError<E>> {
    let mut budget = StaticCatalogBudget::new();
    for data in catalogs {
        budget.admit(data, available, charge)?;
    }
    Ok(budget.bytes())
}

impl Default for StaticCatalogBudget<'_> {
    fn default() -> Self {
        Self::new()
    }
}
