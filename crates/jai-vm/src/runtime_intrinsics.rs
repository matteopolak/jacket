//! Invocation adapter for checked runtime intrinsics, with no native call path.
use crate::memory::pools::PoolOperation;
use crate::{Error, LimitKind, Memory, Pointer, Value};
use jai_ir::{RuntimeIntrinsic, RuntimeIntrinsicError};
use jai_types::{Integer, IntegerType, TypeId, TypeView};

#[derive(Clone, Copy, Debug)]
pub struct RuntimeProcedure {
    pub signature: TypeId,
    pub intrinsic: RuntimeIntrinsic,
}

impl From<RuntimeIntrinsicError> for Error {
    fn from(error: RuntimeIntrinsicError) -> Self {
        match error {
            RuntimeIntrinsicError::Type(error) => Self::Type(error),
            error => Self::IrValidation(error.to_string()),
        }
    }
}

impl RuntimeProcedure {
    /// Visit borrowed address demands before an adapter can compute target layouts.
    /// The callback admits its work under the VM's remaining fuel. No pointer
    /// paths are cloned, and zero byte/get counts do not inspect their operands.
    pub(crate) fn visit_pointer_layouts<E: From<Error>>(
        self,
        arguments: &[Value],
        mut prepare: impl FnMut(&Pointer, bool) -> Result<(), E>,
    ) -> Result<(), E> {
        match (self.intrinsic, arguments) {
            (
                RuntimeIntrinsic::MemoryCopy
                | RuntimeIntrinsic::MemoryCopyReturningDestination
                | RuntimeIntrinsic::MemoryCompare,
                [left, right, count],
            ) => {
                if byte_count(count)? != 0 {
                    prepare(left.pointer()?, false)?;
                    prepare(right.pointer()?, false)?;
                }
            }
            (
                RuntimeIntrinsic::MemorySet | RuntimeIntrinsic::MemorySetReturningDestination,
                [destination, _, count],
            ) => {
                if byte_count(count)? != 0 {
                    prepare(destination.pointer()?, false)?;
                }
            }
            (RuntimeIntrinsic::Swap { .. }, [left, right]) => {
                prepare(left.pointer()?, true)?;
                prepare(right.pointer()?, true)?;
            }
            (RuntimeIntrinsic::CompareAndSwap { .. }, [pointer, _, _]) => {
                prepare(pointer.pointer()?, true)?
            }
            (
                RuntimeIntrinsic::PoolGet { .. } | RuntimeIntrinsic::FlatPoolGet { .. },
                [pool, size],
            ) => {
                if byte_count(size)? != 0 {
                    prepare(pool.pointer()?, true)?;
                }
            }
            (
                RuntimeIntrinsic::PoolReset { .. }
                | RuntimeIntrinsic::PoolRelease { .. }
                | RuntimeIntrinsic::FlatPoolFinish { .. },
                [pool],
            )
            | (RuntimeIntrinsic::FlatPoolReset { .. }, [pool, Value::Bool(_)]) => {
                prepare(pool.pointer()?, true)?
            }
            (RuntimeIntrinsic::DebugTrap, []) => {}
            _ => return Err(Error::InvalidIr("runtime intrinsic argument count differs").into()),
        }
        Ok(())
    }

    /// Header field accesses and new pool backing allocations have additional
    /// scalar layout demands beyond the descriptor's own cached offsets.
    pub(crate) fn visit_additional_layouts<E: From<Error>>(
        self,
        arguments: &[Value],
        types: &dyn TypeView,
        mut prepare: impl FnMut(TypeId) -> Result<(), E>,
    ) -> Result<(), E> {
        let (pool, flat, allocating) = match (self.intrinsic, arguments) {
            (RuntimeIntrinsic::PoolGet { pool }, [_, size])
            | (RuntimeIntrinsic::FlatPoolGet { pool }, [_, size]) => {
                if byte_count(size)? == 0 {
                    return Ok(());
                }
                (
                    pool,
                    matches!(self.intrinsic, RuntimeIntrinsic::FlatPoolGet { .. }),
                    true,
                )
            }
            (
                RuntimeIntrinsic::PoolReset { pool } | RuntimeIntrinsic::PoolRelease { pool },
                [_],
            ) => (pool, false, false),
            (RuntimeIntrinsic::FlatPoolReset { pool }, [_, Value::Bool(_)])
            | (RuntimeIntrinsic::FlatPoolFinish { pool }, [_]) => (pool, true, false),
            (
                RuntimeIntrinsic::PoolGet { .. }
                | RuntimeIntrinsic::FlatPoolGet { .. }
                | RuntimeIntrinsic::PoolReset { .. }
                | RuntimeIntrinsic::PoolRelease { .. }
                | RuntimeIntrinsic::FlatPoolReset { .. }
                | RuntimeIntrinsic::FlatPoolFinish { .. },
                _,
            ) => {
                return Err(Error::InvalidIr("runtime intrinsic argument count differs").into());
            }
            _ => return Ok(()),
        };
        let fields = &types
            .record_storage_definition(pool)
            .map_err(Error::from)?
            .fields;
        if fields.len() != if flat { 5 } else { 4 } {
            return Err(Error::InvalidIr("pool descriptor field count differs").into());
        }
        for &field in fields.iter() {
            prepare(field)?;
        }
        if allocating {
            prepare(
                types
                    .lookup(&jai_types::TypeKind::String)
                    .ok_or(Error::InvalidIr("string type missing"))?,
            )?;
        }
        Ok(())
    }

    /// Legacy scalar-count estimate. VM execution uses `work_cost_for_target` to
    /// include complete root images and metadata before invoking this adapter.
    pub fn work_cost(self, arguments: &[Value]) -> Result<u64, Error> {
        Ok(match self.intrinsic {
            RuntimeIntrinsic::MemoryCopy
            | RuntimeIntrinsic::MemoryCopyReturningDestination
            | RuntimeIntrinsic::MemoryCompare
            | RuntimeIntrinsic::MemorySet
            | RuntimeIntrinsic::MemorySetReturningDestination => {
                let [_, _, count] = arguments else {
                    return Err(Error::InvalidIr("runtime intrinsic argument count differs"));
                };
                u64::try_from(byte_count(count)?).map_err(|_| Error::Limit(LimitKind::Fuel))?
            }
            RuntimeIntrinsic::Swap { .. }
            | RuntimeIntrinsic::PoolGet { .. }
            | RuntimeIntrinsic::PoolReset { .. }
            | RuntimeIntrinsic::PoolRelease { .. }
            | RuntimeIntrinsic::FlatPoolGet { .. }
            | RuntimeIntrinsic::FlatPoolReset { .. }
            | RuntimeIntrinsic::FlatPoolFinish { .. } => {
                return Err(Error::InvalidIr(
                    "runtime operation requires target-aware work accounting",
                ));
            }
            RuntimeIntrinsic::CompareAndSwap { .. } | RuntimeIntrinsic::DebugTrap => 1,
        })
    }

    /// Estimate actual target storage work without loading or mutating its values.
    pub fn work_cost_for_target(
        self,
        arguments: &[Value],
        types: &dyn TypeView,
        memory: &Memory,
    ) -> Result<u64, Error> {
        self.intrinsic
            .validate_signature_shape(self.signature, types)
            .map_err(Error::from)?;
        match (self.intrinsic, arguments) {
            (RuntimeIntrinsic::Swap { .. }, [left, right]) => {
                memory.swap_work_cost(types, left.pointer()?, right.pointer()?)
            }
            (
                RuntimeIntrinsic::MemoryCopy
                | RuntimeIntrinsic::MemoryCopyReturningDestination
                | RuntimeIntrinsic::MemoryCompare,
                [first, second, count],
            ) => {
                let count = byte_count(count)?;
                if count == 0 {
                    return Ok(0);
                }
                memory.intrinsic_work_cost(
                    types,
                    &[
                        (
                            first.pointer()?,
                            self.intrinsic != RuntimeIntrinsic::MemoryCompare,
                        ),
                        (second.pointer()?, false),
                    ],
                    count,
                )
            }
            (
                RuntimeIntrinsic::MemorySet | RuntimeIntrinsic::MemorySetReturningDestination,
                [destination, _, count],
            ) => {
                let count = byte_count(count)?;
                if count == 0 {
                    return Ok(0);
                }
                memory.intrinsic_work_cost(types, &[(destination.pointer()?, true)], count)
            }
            (RuntimeIntrinsic::CompareAndSwap { value }, [pointer, _, _]) => {
                memory.atomic_work_cost(types, pointer.pointer()?, value)
            }
            (RuntimeIntrinsic::DebugTrap, []) => Ok(1),
            (
                RuntimeIntrinsic::PoolGet { .. } | RuntimeIntrinsic::FlatPoolGet { .. },
                [pool, size],
            ) => memory.pool_work_cost(
                types,
                pool.pointer()?,
                matches!(self.intrinsic, RuntimeIntrinsic::FlatPoolGet { .. }),
                PoolOperation::Get(byte_count(size)?),
            ),
            (RuntimeIntrinsic::PoolReset { .. }, [pool]) => {
                memory.pool_work_cost(types, pool.pointer()?, false, PoolOperation::Reset(false))
            }
            (RuntimeIntrinsic::FlatPoolReset { .. }, [pool, Value::Bool(overwrite)]) => memory
                .pool_work_cost(
                    types,
                    pool.pointer()?,
                    true,
                    PoolOperation::Reset(*overwrite),
                ),
            (
                RuntimeIntrinsic::PoolRelease { .. } | RuntimeIntrinsic::FlatPoolFinish { .. },
                [pool],
            ) => memory.pool_work_cost(
                types,
                pool.pointer()?,
                matches!(self.intrinsic, RuntimeIntrinsic::FlatPoolFinish { .. }),
                PoolOperation::Release,
            ),
            _ => Err(Error::InvalidIr("runtime intrinsic argument count differs")),
        }
    }

    /// Recheck nominal signature shape; memory operations consume the selected
    /// target's admitted layouts when checking their actual storage.
    pub fn invoke(
        self,
        arguments: &[Value],
        memory: &mut Memory,
        types: &dyn TypeView,
    ) -> Result<Vec<Value>, Error> {
        self.intrinsic
            .validate_signature_shape(self.signature, types)
            .map_err(Error::from)?;
        let signature = types.procedure_definition(self.signature)?;
        if arguments.len() != signature.parameters.len() {
            return Err(Error::InvalidIr("runtime intrinsic argument count differs"));
        }
        for (argument, &expected) in arguments.iter().zip(signature.parameters.iter()) {
            argument.validate(types, expected, 256)?;
        }
        let mut results = vec![];
        match (self.intrinsic, arguments) {
            (
                RuntimeIntrinsic::MemoryCopy | RuntimeIntrinsic::MemoryCopyReturningDestination,
                [destination, source, count],
            ) => {
                memory.byte_copy(
                    types,
                    destination.pointer()?,
                    source.pointer()?,
                    byte_count(count)?,
                )?;
                if self.intrinsic == RuntimeIntrinsic::MemoryCopyReturningDestination {
                    results.push(destination.clone());
                }
            }
            (RuntimeIntrinsic::MemoryCompare, [left, right, count]) => {
                let compared = memory.byte_compare(
                    types,
                    left.pointer()?,
                    right.pointer()?,
                    byte_count(count)?,
                )?;
                results.push(Value::Int(Integer::wrapping(
                    IntegerType::S16,
                    i128::from(compared),
                )));
            }
            (
                RuntimeIntrinsic::MemorySet | RuntimeIntrinsic::MemorySetReturningDestination,
                [destination, byte, count],
            ) => {
                let mut byte = byte.number()?;
                if self.intrinsic == RuntimeIntrinsic::MemorySetReturningDestination {
                    byte = crate::scalar::cast_number(
                        IntegerType::U8,
                        byte,
                        jai_types::CastMode::Unchecked,
                    )?;
                }
                memory.byte_set_number(types, destination.pointer()?, &byte, byte_count(count)?)?;
                if self.intrinsic == RuntimeIntrinsic::MemorySetReturningDestination {
                    results.push(destination.clone());
                }
            }
            (RuntimeIntrinsic::Swap { .. }, [left, right]) => {
                memory.swap_values(types, left.pointer()?, right.pointer()?)?;
            }
            (RuntimeIntrinsic::CompareAndSwap { .. }, [pointer, expected, replacement]) => {
                let (success, observed) =
                    memory.compare_and_swap(types, pointer.pointer()?, expected, replacement)?;
                results.extend([Value::Bool(success), observed]);
            }
            (RuntimeIntrinsic::DebugTrap, []) => return Err(Error::RuntimeTrap),
            (
                RuntimeIntrinsic::PoolGet { .. } | RuntimeIntrinsic::FlatPoolGet { .. },
                [pool, size],
            ) => {
                let pointer = memory
                    .pool_operation(
                        types,
                        pool.pointer()?,
                        matches!(self.intrinsic, RuntimeIntrinsic::FlatPoolGet { .. }),
                        PoolOperation::Get(byte_count(size)?),
                    )?
                    .ok_or(Error::InvalidIr("pool get returned no pointer"))?;
                results.push(Value::Pointer(pointer));
            }
            (RuntimeIntrinsic::PoolReset { .. }, [pool]) => {
                memory.pool_operation(
                    types,
                    pool.pointer()?,
                    false,
                    PoolOperation::Reset(false),
                )?;
            }
            (RuntimeIntrinsic::FlatPoolReset { .. }, [pool, Value::Bool(overwrite)]) => {
                memory.pool_operation(
                    types,
                    pool.pointer()?,
                    true,
                    PoolOperation::Reset(*overwrite),
                )?;
            }
            (
                RuntimeIntrinsic::PoolRelease { .. } | RuntimeIntrinsic::FlatPoolFinish { .. },
                [pool],
            ) => {
                memory.pool_operation(
                    types,
                    pool.pointer()?,
                    matches!(self.intrinsic, RuntimeIntrinsic::FlatPoolFinish { .. }),
                    PoolOperation::Release,
                )?;
            }
            _ => return Err(Error::InvalidIr("runtime intrinsic argument count differs")),
        }
        Ok(results)
    }
}

fn byte_count(value: &Value) -> Result<usize, Error> {
    let count = value.number()?.portable_integer()?;
    if count.ty() != IntegerType::S64 {
        return Err(Error::InvalidIr(
            "memory intrinsic requires an s64 byte count",
        ));
    }
    usize::try_from(count.value()).map_err(|_| Error::CheckedCast)
}

#[cfg(test)]
#[path = "runtime_intrinsics/tests.rs"]
mod tests;
