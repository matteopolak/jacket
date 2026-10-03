//! Owned block ledgers for checked Pool and Flat_Pool declarations.
use super::*;

const DEFAULT_CAPACITY: usize = 65_536;
const STATE_CELLS: usize = 12;
const BLOCK_CELLS: usize = 6;

#[derive(Clone, Copy, Debug)]
pub(crate) enum PoolOperation {
    Get(usize),
    Reset(bool),
    Release,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PoolKey {
    memory: u64,
    allocation: u64,
    offset: u64,
    ty: TypeId,
}

#[derive(Clone)]
struct Block {
    root: Pointer,
    capacity: usize,
    alignment: u32,
}

#[derive(Clone)]
struct State {
    flat: bool,
    blocks: Vec<Block>,
    current: usize,
    position: usize,
}

/// Metadata is charged cumulatively alongside allocation cells. Snapshot cloning
/// retains this ledger but never recycles allocation or virtual address identities.
#[derive(Clone, Default)]
pub(super) struct PoolLedger {
    states: HashMap<PoolKey, State>,
    owners: HashMap<u64, usize>,
    identities: HashMap<(u64, u64), TypeId>,
    cells: usize,
}

impl PoolLedger {
    pub(super) fn table_capacity(&self) -> usize {
        self.states
            .capacity()
            .saturating_add(self.owners.capacity())
            .saturating_add(self.identities.capacity())
    }
    pub(super) fn owns_descriptor(&self, allocation: u64) -> bool {
        self.owners.contains_key(&allocation)
    }
}

struct Config {
    capacity: usize,
    alignment: u32,
    reserve: usize,
}

impl Memory {
    /// No root image is created while estimating work. Config reads inspect only
    /// the fixed eight-byte scalar fields, refusing address-derived integers.
    pub(crate) fn pool_work_cost(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        flat: bool,
        operation: PoolOperation,
    ) -> Result<u64, Error> {
        if matches!(operation, PoolOperation::Get(0)) {
            return Ok(0);
        }
        let key = self.pool_key(types, pointer, flat)?;
        let config = self.pool_config_for(types, pointer, flat, operation)?;
        let state = self.pool_ledger.states.get(&key);
        let mut work = self.snapshot_pool_work()?;
        let allocation = self.allocation(pointer)?;
        let root = allocation
            .virtual_extent
            .checked_add(u64::try_from(allocation.cells.get()).map_err(|_| fuel())?)
            .and_then(|cost| cost.checked_mul(8))
            .ok_or_else(fuel)?;
        work = work.checked_add(root).ok_or_else(fuel)?;
        match operation {
            PoolOperation::Get(size) => {
                let reused = state.is_some_and(|state| can_reuse(state, &config, size));
                if !reused {
                    let capacity = required_capacity(&config, size)?;
                    self.check_pool_capacity(capacity)?;
                    work = work
                        .checked_add(
                            u64::try_from(capacity)
                                .map_err(|_| fuel())?
                                .checked_mul(2)
                                .ok_or_else(fuel)?,
                        )
                        .ok_or_else(fuel)?;
                }
            }
            PoolOperation::Reset(overwrite) => {
                if overwrite && let Some(state) = state {
                    for block in &state.blocks {
                        let allocation = self.allocation(&block.root)?;
                        work = work
                            .checked_add(
                                allocation
                                    .virtual_extent
                                    .checked_add(
                                        u64::try_from(allocation.cells.get())
                                            .map_err(|_| fuel())?,
                                    )
                                    .and_then(|cost| cost.checked_mul(2))
                                    .ok_or_else(fuel)?,
                            )
                            .ok_or_else(fuel)?;
                    }
                }
            }
            PoolOperation::Release => {}
        }
        Ok(work)
    }

    fn snapshot_pool_work(&self) -> Result<u64, Error> {
        let mut work = u64::try_from(self.cells.get())
            .map_err(|_| fuel())?
            .checked_mul(2)
            .ok_or_else(fuel)?;
        for count in [
            self.allocations.capacity(),
            self.handle_tokens.borrow().values.capacity(),
            self.virtual_regions.len(),
            self.runtime_types.capacity(),
            self.pool_ledger.table_capacity(),
        ] {
            work = work
                .checked_add(u64::try_from(count).map_err(|_| fuel())?)
                .ok_or_else(fuel)?;
        }
        Ok(work)
    }

    pub(crate) fn pool_operation(
        &mut self,
        types: &dyn TypeView,
        pointer: &Pointer,
        flat: bool,
        operation: PoolOperation,
    ) -> Result<Option<Pointer>, Error> {
        if matches!(operation, PoolOperation::Get(0)) {
            let void = types
                .lookup(&TypeKind::Void)
                .ok_or(Error::InvalidIr("void type missing"))?;
            return Ok(Some(Pointer::null(void)));
        }
        let key = self.pool_key(types, pointer, flat)?;
        let config = self.pool_config_for(types, pointer, flat, operation)?;
        self.validate_pool_descriptor(types, pointer, key, flat)?;
        if !matches!(operation, PoolOperation::Get(_))
            && self.pool_ledger.states.get(&key).is_some_and(|state| {
                state
                    .blocks
                    .iter()
                    .any(|block| self.pool_ledger.owns_descriptor(block.root.allocation_id()))
            })
        {
            return Err(Error::InvalidIr(
                "finish nested pools before resetting or releasing their backing pool",
            ));
        }
        if let PoolOperation::Get(size) = operation
            && !self
                .pool_ledger
                .states
                .get(&key)
                .is_some_and(|state| can_reuse(state, &config, size))
        {
            self.preflight_pool_block(key, &config, size)?;
        }
        let snapshot = self.snapshot();
        let result = self.pool_operation_inner(types, pointer, key, flat, config, operation);
        if result.is_err() {
            self.restore(snapshot);
        }
        result
    }

    fn pool_operation_inner(
        &mut self,
        types: &dyn TypeView,
        pointer: &Pointer,
        key: PoolKey,
        flat: bool,
        config: Config,
        operation: PoolOperation,
    ) -> Result<Option<Pointer>, Error> {
        match operation {
            PoolOperation::Release => {
                // Remove the descriptor's relocation while every block is still
                // alive, so retokenizing its enclosing image cannot encounter a
                // just-freed handle during the cursor stores.
                self.write_pool_cursor(types, pointer, None, 0)?;
                if let Some(state) = self.pool_ledger.states.remove(&key) {
                    let owners = self
                        .pool_ledger
                        .owners
                        .get_mut(&key.allocation)
                        .ok_or(Error::InvalidIr("pool descriptor ownership missing"))?;
                    *owners -= 1;
                    if *owners == 0 {
                        self.pool_ledger.owners.remove(&key.allocation);
                    }
                    let cells = state_cells(state.blocks.len())?;
                    self.pool_ledger.cells -= cells;
                    self.pool_ledger
                        .identities
                        .remove(&(key.allocation, key.offset));
                    self.cells.set(self.cells.get() - cells);
                    for block in state.blocks {
                        self.release(&block.root)?;
                    }
                }
                Ok(None)
            }
            PoolOperation::Reset(overwrite) => {
                let Some(mut state) = self.pool_ledger.states.get(&key).cloned() else {
                    self.write_pool_cursor(types, pointer, None, 0)?;
                    return Ok(None);
                };
                if overwrite {
                    for block in &state.blocks {
                        self.byte_set(types, &block.root, 0xcc, block.capacity)?;
                    }
                }
                state.current = 0;
                state.position = aligned(config.reserve, config.alignment)?;
                if state.position > state.blocks[0].capacity {
                    return Err(Error::CheckedCast);
                }
                self.write_pool_cursor(types, pointer, Some(&state.blocks[0]), state.position)?;
                self.pool_ledger.states.insert(key, state);
                Ok(None)
            }
            PoolOperation::Get(size) => {
                let mut state = self.pool_ledger.states.get(&key).cloned().unwrap_or(State {
                    flat,
                    blocks: vec![],
                    current: 0,
                    position: 0,
                });
                let mut chosen = None;
                for index in state.current..state.blocks.len() {
                    let position = if index == state.current {
                        state.position
                    } else {
                        config.reserve
                    };
                    let start = aligned(position, config.alignment)?;
                    let end = start.checked_add(size).ok_or(Error::CheckedCast)?;
                    let block = &state.blocks[index];
                    if block.alignment >= config.alignment && end <= block.capacity {
                        chosen = Some((index, start, end));
                        break;
                    }
                }
                let (index, start, end) = if let Some(chosen) = chosen {
                    chosen
                } else {
                    let (capacity, extra) = self.preflight_pool_block(key, &config, size)?;
                    let string = types
                        .lookup(&TypeKind::String)
                        .ok_or(Error::InvalidIr("string type missing"))?;
                    let root = self.allocate_with_alignment(
                        types,
                        string,
                        Some(Value::String(vec![0; capacity])),
                        config.alignment,
                    )?;
                    self.pool_ledger.cells += extra;
                    self.cells.set(self.cells.get() + extra);
                    state.blocks.push(Block {
                        root,
                        capacity,
                        alignment: config.alignment,
                    });
                    let start = aligned(config.reserve, config.alignment)?;
                    (
                        state.blocks.len() - 1,
                        start,
                        start.checked_add(size).ok_or(Error::CheckedCast)?,
                    )
                };
                state.current = index;
                state.position = end;
                let block = &state.blocks[index];
                let void = types
                    .lookup(&TypeKind::Void)
                    .ok_or(Error::InvalidIr("void type missing"))?;
                let mut result = self.cast_pointer(types, &block.root, void, CastMode::Checked)?;
                result.data_mut()?.path = vec![Projection::Bytes {
                    offset: u64::try_from(start).map_err(|_| Error::CheckedCast)?,
                    ty: void,
                }];
                result.data_mut()?.region = Some((
                    u64::try_from(start).map_err(|_| Error::CheckedCast)?,
                    u64::try_from(end).map_err(|_| Error::CheckedCast)?,
                ));
                self.write_pool_cursor(types, pointer, Some(block), end)?;
                if self.peek_pool_integer(types, pointer, 0)? == 0 {
                    self.write_pool_integer(types, pointer, 0, config.capacity)?;
                }
                if flat && self.peek_pool_integer(types, pointer, 4)? == 0 {
                    self.write_pool_integer(types, pointer, 4, config.alignment as usize)?;
                }
                if !self.pool_ledger.states.contains_key(&key) {
                    *self.pool_ledger.owners.entry(key.allocation).or_default() += 1;
                    self.pool_ledger
                        .identities
                        .insert((key.allocation, key.offset), key.ty);
                }
                self.pool_ledger.states.insert(key, state);
                Ok(Some(result))
            }
        }
    }

    fn pool_key(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        flat: bool,
    ) -> Result<PoolKey, Error> {
        jai_ir::RuntimeIntrinsic::validate_pool_storage(
            types,
            pointer.pointee,
            flat,
            self.target.policy,
        )?;
        let size = self.layout(types, pointer.pointee)?.size;
        self.intrinsic_range(
            types,
            pointer,
            usize::try_from(size).map_err(|_| Error::CheckedCast)?,
            true,
        )?;
        let key = PoolKey {
            memory: self.identity,
            allocation: pointer.allocation_id(),
            offset: self.byte_offset(types, pointer)?,
            ty: pointer.pointee,
        };
        if self
            .pool_ledger
            .identities
            .get(&(key.allocation, key.offset))
            .is_some_and(|ty| *ty != key.ty)
        {
            return Err(Error::InvalidIr(
                "pool descriptor is already owned by another nominal pool type",
            ));
        }
        Ok(key)
    }

    fn pool_config_for(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        flat: bool,
        operation: PoolOperation,
    ) -> Result<Config, Error> {
        if matches!(operation, PoolOperation::Release) {
            return Ok(Config {
                capacity: 0,
                alignment: 1,
                reserve: 0,
            });
        }
        self.pool_config(
            types,
            pointer,
            flat,
            matches!(operation, PoolOperation::Get(_)),
        )
    }

    fn preflight_pool_block(
        &self,
        key: PoolKey,
        config: &Config,
        size: usize,
    ) -> Result<(usize, usize), Error> {
        let capacity = required_capacity(config, size)?;
        self.check_pool_capacity(capacity)?;
        let previous = self.pool_ledger.states.get(&key);
        let old_cells = previous.map_or(Ok(0), |state| state_cells(state.blocks.len()))?;
        let count = previous.map_or(0, |state| state.blocks.len());
        let extra = state_cells(count.checked_add(1).ok_or(Error::CheckedCast)?)? - old_cells;
        self.cells
            .get()
            .checked_add(extra)
            .and_then(|cells| cells.checked_add(capacity.checked_add(1)?))
            .filter(|cells| *cells <= self.limits.value_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        if self.allocations.len() >= self.limits.allocations {
            return Err(Error::Limit(LimitKind::Allocations));
        }
        Ok((capacity, extra))
    }

    fn pool_config(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        flat: bool,
        capacity_needed: bool,
    ) -> Result<Config, Error> {
        let requested = if capacity_needed {
            self.peek_pool_integer(types, pointer, 0)?
        } else {
            0
        };
        let capacity = if requested == 0 {
            DEFAULT_CAPACITY
        } else {
            usize::try_from(requested).map_err(|_| Error::CheckedCast)?
        };
        let alignment = if flat {
            match self.peek_pool_integer(types, pointer, 4)? {
                0 => 8,
                value => u32::try_from(value).map_err(|_| Error::CheckedCast)?,
            }
        } else {
            let scalar = self
                .layout(types, types.scalar(ScalarType::Int(IntegerType::S64)))?
                .alignment;
            self.target.policy.pointer().alignment.max(scalar)
        };
        if !alignment.is_power_of_two() {
            return Err(Error::InvalidIr(
                "pool alignment must be a positive power of two",
            ));
        }
        Ok(Config {
            capacity,
            alignment,
            reserve: if flat {
                0
            } else {
                usize::try_from(self.target.policy.pointer().size)
                    .map_err(|_| Error::CheckedCast)?
            },
        })
    }

    fn check_pool_capacity(&self, capacity: usize) -> Result<(), Error> {
        let bytes = u64::try_from(capacity).map_err(|_| Error::CheckedCast)?;
        let bits = self
            .target
            .policy
            .pointer()
            .size
            .checked_mul(8)
            .ok_or(Error::CheckedCast)?;
        let maximum = if bits == 64 {
            u64::MAX
        } else {
            (1u64 << bits) - 1
        };
        if bytes > maximum || bytes > i64::MAX as u64 {
            return Err(Error::CheckedCast);
        }
        if capacity
            .checked_add(1)
            .is_none_or(|cells| cells > self.limits.value_cells)
        {
            return Err(Error::Limit(LimitKind::ValueCells));
        }
        Ok(())
    }

    fn validate_pool_descriptor(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        key: PoolKey,
        flat: bool,
    ) -> Result<(), Error> {
        let position = self.peek_pool_integer(types, pointer, 3)?;
        let left = self.peek_pool_integer(types, pointer, 1)?;
        let current = self.load(types, &self.field(types, pointer, 2)?)?;
        let current = current.pointer()?;
        match self.pool_ledger.states.get(&key) {
            Some(state) if state.flat == flat => {
                let block = &state.blocks[state.current];
                if position != state.position as i128
                    || left != (block.capacity - state.position) as i128
                    || !self.same_address(types, current, &block.root)?
                {
                    return Err(Error::InvalidIr(
                        "pool descriptor no longer matches its owned ledger",
                    ));
                }
            }
            Some(_) => {
                return Err(Error::InvalidIr(
                    "pool dialect does not match its owned ledger",
                ));
            }
            None if position == 0 && left == 0 && current.is_null() => {}
            None => {
                return Err(Error::InvalidIr(
                    "pool descriptor has no owned block ledger",
                ));
            }
        }
        Ok(())
    }

    fn write_pool_cursor(
        &mut self,
        types: &dyn TypeView,
        pointer: &Pointer,
        block: Option<&Block>,
        position: usize,
    ) -> Result<(), Error> {
        let void = types
            .lookup(&TypeKind::Void)
            .ok_or(Error::InvalidIr("void type missing"))?;
        let current = match block {
            Some(block) => self.cast_pointer(types, &block.root, void, CastMode::Checked)?,
            None => Pointer::null(void),
        };
        self.write_pool_integer(
            types,
            pointer,
            1,
            block.map_or(0, |block| block.capacity - position),
        )?;
        self.store(
            types,
            &self.field(types, pointer, 2)?,
            Value::Pointer(current),
        )?;
        self.write_pool_integer(types, pointer, 3, position)
    }

    fn write_pool_integer(
        &mut self,
        types: &dyn TypeView,
        pointer: &Pointer,
        field: usize,
        value: usize,
    ) -> Result<(), Error> {
        let value = i64::try_from(value).map_err(|_| Error::CheckedCast)?;
        self.store(
            types,
            &self.field(types, pointer, field)?,
            Value::Int(
                Integer::checked(IntegerType::S64, i128::from(value)).ok_or(Error::CheckedCast)?,
            ),
        )
    }

    fn peek_pool_integer(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        field: usize,
    ) -> Result<i128, Error> {
        let scalar = self.field(types, pointer, field)?;
        let allocation = self.allocation(&scalar)?;
        let offset =
            usize::try_from(self.byte_offset(types, &scalar)?).map_err(|_| Error::CheckedCast)?;
        let image = allocation.image.borrow();
        if let Some(image) = image.as_ref() {
            return peek_image_integer(image, offset, self.target.endian);
        }
        let mut value = allocation.value.as_ref().ok_or(Error::Uninitialized)?;
        if let Value::String(bytes) = value {
            let end = offset.checked_add(8).ok_or(Error::CheckedCast)?;
            let bytes = bytes.get(offset..end).ok_or(Error::Uninitialized)?;
            return decode_s64(bytes, self.target.endian);
        }
        let mut prefix_offset = 0usize;
        for projection in &scalar.data()?.path {
            if let Value::StoredAggregate(snapshot) = value {
                return peek_image_integer(
                    snapshot.image(),
                    offset
                        .checked_sub(prefix_offset)
                        .ok_or(Error::CheckedCast)?,
                    self.target.endian,
                );
            }
            value = match (projection, value) {
                (
                    Projection::Field(index),
                    Value::Record {
                        ty,
                        fields,
                    },
                ) => {
                    let layout = self.layout(types, *ty)?;
                    let field_offset = *layout
                        .field_offsets
                        .get(*index)
                        .ok_or(Error::Uninitialized)?;
                    prefix_offset = prefix_offset
                        .checked_add(usize::try_from(field_offset).map_err(|_| Error::CheckedCast)?)
                        .ok_or(Error::CheckedCast)?;
                    fields.get(*index).ok_or(Error::Uninitialized)?
                }
                (
                    Projection::Index(index),
                    Value::Array {
                        ty,
                        elements,
                    },
                ) => {
                    let TypeKind::FixedArray {
                        element, ..
                    } = *types.kind(*ty)?
                    else {
                        return Err(Error::InvalidIr("pool configuration array type mismatch"));
                    };
                    let stride = self.layout(types, element)?.size;
                    prefix_offset = prefix_offset
                        .checked_add(
                            usize::try_from(stride)
                                .map_err(|_| Error::CheckedCast)?
                                .checked_mul(*index)
                                .ok_or(Error::CheckedCast)?,
                        )
                        .ok_or(Error::CheckedCast)?;
                    elements.get(*index).ok_or(Error::Uninitialized)?
                }
                _ => {
                    return Err(Error::InvalidIr(
                        "pool configuration cannot be read without materializing storage",
                    ));
                }
            };
        }
        match value {
            Value::Int(integer) if integer.ty() == IntegerType::S64 => Ok(integer.value()),
            Value::AddressInteger(_) => Err(Error::UnsupportedPointerOperation(
                "pool configuration must use portable integers",
            )),
            _ => Err(Error::InvalidIr("pool configuration requires s64 storage")),
        }
    }
}

fn peek_image_integer(image: &ByteImage, offset: usize, endian: Endian) -> Result<i128, Error> {
    if image.range_has_provenance(offset, 8)? {
        return Err(Error::UnsupportedPointerOperation(
            "pool configuration must use portable integers",
        ));
    }
    decode_s64(image.read_range(offset, 8)?, endian)
}

fn decode_s64(bytes: &[u8], endian: Endian) -> Result<i128, Error> {
    let bytes: [u8; 8] = bytes.try_into().map_err(|_| Error::Uninitialized)?;
    Ok(i128::from(match endian {
        Endian::Little => i64::from_le_bytes(bytes),
        Endian::Big => i64::from_be_bytes(bytes),
    }))
}
fn can_reuse(state: &State, config: &Config, size: usize) -> bool {
    state
        .blocks
        .iter()
        .enumerate()
        .skip(state.current)
        .any(|(index, block)| {
            let position = if index == state.current {
                state.position
            } else {
                config.reserve
            };
            aligned(position, config.alignment)
                .ok()
                .and_then(|start| start.checked_add(size))
                .is_some_and(|end| block.alignment >= config.alignment && end <= block.capacity)
        })
}

fn aligned(position: usize, alignment: u32) -> Result<usize, Error> {
    let mask = usize::try_from(alignment - 1).map_err(|_| Error::CheckedCast)?;
    position
        .checked_add(mask)
        .map(|position| position & !mask)
        .ok_or(Error::CheckedCast)
}
fn required_capacity(config: &Config, size: usize) -> Result<usize, Error> {
    Ok(config.capacity.max(
        aligned(config.reserve, config.alignment)?
            .checked_add(size)
            .ok_or(Error::CheckedCast)?,
    ))
}
fn state_cells(blocks: usize) -> Result<usize, Error> {
    blocks
        .checked_mul(BLOCK_CELLS)
        .and_then(|cells| cells.checked_add(STATE_CELLS))
        .ok_or(Error::Limit(LimitKind::ValueCells))
}
fn fuel() -> Error {
    Error::Limit(LimitKind::Fuel)
}

#[cfg(test)]
#[path = "pools/tests.rs"]
mod tests;
