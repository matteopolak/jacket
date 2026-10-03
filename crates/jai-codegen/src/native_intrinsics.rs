//! Real runtime bodies for the checked intrinsic catalog, with the ordinary Jai ABI.
use crate::*;
use inkwell::AtomicOrdering;
use jai_ir::RuntimeIntrinsic;
#[path = "native_pools.rs"]
mod pools;

pub(super) fn define<'ctx>(
    module: &Module<'ctx>,
    lowerer: &mut types::TypeLowerer<'ctx, '_>,
    target: &target::NativeTarget,
    signature: TypeId,
    operation: RuntimeIntrinsic,
    name: &str,
) -> Result<FunctionValue<'ctx>, Error> {
    let context = lowerer.context();
    operation
        .validate_signature(
            signature,
            lowerer.registry(),
            types::layout_policy(context, &target.data)?,
        )
        .map_err(Error::RuntimeIntrinsic)?;
    if matches!(
        operation,
        RuntimeIntrinsic::PoolGet { .. }
            | RuntimeIntrinsic::PoolReset { .. }
            | RuntimeIntrinsic::PoolRelease { .. }
            | RuntimeIntrinsic::FlatPoolGet { .. }
            | RuntimeIntrinsic::FlatPoolReset { .. }
            | RuntimeIntrinsic::FlatPoolFinish { .. }
    ) {
        pools::prepare(module, context, target)?;
    }
    let function = module.add_function(name, lowerer.function(signature)?, None);
    let builder = context.create_builder();
    builder.position_at_end(context.append_basic_block(function, "entry"));
    let trap = Intrinsic::find("llvm.trap")
        .and_then(|intrinsic| intrinsic.get_declaration(module, &[]))
        .ok_or(Error::Invariant)?;
    let arguments: Vec<_> = function.get_param_iter().collect();
    match operation {
        RuntimeIntrinsic::PoolGet { .. }
        | RuntimeIntrinsic::PoolReset { .. }
        | RuntimeIntrinsic::PoolRelease { .. }
        | RuntimeIntrinsic::FlatPoolGet { .. }
        | RuntimeIntrinsic::FlatPoolReset { .. }
        | RuntimeIntrinsic::FlatPoolFinish { .. } => {
            pools::emit(module, lowerer, target, &builder, operation, &arguments)?;
        }
        RuntimeIntrinsic::DebugTrap => {
            let intrinsic = Intrinsic::find("llvm.debugtrap")
                .and_then(|intrinsic| intrinsic.get_declaration(module, &[]))
                .ok_or(Error::Invariant)?;
            builder.build_call(intrinsic, &[], "")?;
            builder.build_return(None)?;
        }
        RuntimeIntrinsic::Swap { value } => {
            let left = arguments[0].into_pointer_value();
            let right = arguments[1].into_pointer_value();
            let storage = lowerer.basic(value)?;
            let size_type = context.ptr_sized_int_type(&target.data, None);
            let bytes = target.data.get_abi_size(&storage);
            if size_type.get_bit_width() < 64 && bytes > (1u64 << size_type.get_bit_width()) - 1 {
                return Err(Error::RuntimeIntrinsic(
                    jai_ir::RuntimeIntrinsicError::Signature(
                        "swap storage must fit the target pointer width",
                    ),
                ));
            }
            let size = size_type.const_int(bytes, false);
            let left_start = checked_range(context, &builder, function, trap, left, size)?;
            let right_start = checked_range(context, &builder, function, trap, right, size)?;
            let left_end = builder.build_int_add(left_start, size, "swap.left.end")?;
            let right_end = builder.build_int_add(right_start, size, "swap.right.end")?;
            let same = builder.build_int_compare(
                IntPredicate::EQ,
                left_start,
                right_start,
                "swap.same",
            )?;
            let before = builder.build_int_compare(
                IntPredicate::ULE,
                left_end,
                right_start,
                "swap.before",
            )?;
            let after = builder.build_int_compare(
                IntPredicate::ULE,
                right_end,
                left_start,
                "swap.after",
            )?;
            let disjoint = builder.build_or(before, after, "swap.disjoint")?;
            require(
                context,
                &builder,
                function,
                trap,
                builder.build_or(same, disjoint, "swap.valid")?,
            )?;
            let identical = context.append_basic_block(function, "swap.identical");
            let exchange = context.append_basic_block(function, "swap.exchange");
            builder.build_conditional_branch(same, identical, exchange)?;
            builder.position_at_end(identical);
            builder.build_return(None)?;
            builder.position_at_end(exchange);
            let first = builder.build_load(storage, left, "swap.first")?;
            let second = builder.build_load(storage, right, "swap.second")?;
            first
                .as_instruction_value()
                .ok_or(Error::Invariant)?
                .set_alignment(1)
                .map_err(|_| Error::Invariant)?;
            second
                .as_instruction_value()
                .ok_or(Error::Invariant)?
                .set_alignment(1)
                .map_err(|_| Error::Invariant)?;
            builder
                .build_store(left, second)?
                .set_alignment(1)
                .map_err(|_| Error::Invariant)?;
            builder
                .build_store(right, first)?
                .set_alignment(1)
                .map_err(|_| Error::Invariant)?;
            builder.build_return(None)?;
        }
        RuntimeIntrinsic::CompareAndSwap { value } => {
            let pointer = arguments[0].into_pointer_value();
            let expected = arguments[1];
            let replacement = arguments[2];
            require(
                context,
                &builder,
                function,
                trap,
                builder.build_is_not_null(pointer, "atomic.nonnull")?,
            )?;
            let storage = lowerer.basic(value)?;
            let alignment =
                u32::try_from(target.data.get_abi_size(&storage)).map_err(|_| Error::Invariant)?;
            let address_type = context.ptr_sized_int_type(&target.data, None);
            let address = builder.build_ptr_to_int(pointer, address_type, "atomic.address")?;
            let misalignment = builder.build_and(
                address,
                address_type.const_int(u64::from(alignment - 1), false),
                "atomic.misalignment",
            )?;
            require(
                context,
                &builder,
                function,
                trap,
                builder.build_int_compare(
                    IntPredicate::EQ,
                    misalignment,
                    address_type.const_zero(),
                    "atomic.aligned",
                )?,
            )?;
            let (success, observed) = if expected.is_int_value()
                && expected.into_int_value().get_type().get_bit_width() == 1
            {
                boolean_cas(
                    context,
                    &builder,
                    function,
                    pointer,
                    expected.into_int_value(),
                    replacement.into_int_value(),
                )?
            } else {
                let exchanged = builder.build_cmpxchg(
                    pointer,
                    expected,
                    replacement,
                    AtomicOrdering::SequentiallyConsistent,
                    AtomicOrdering::SequentiallyConsistent,
                )?;
                (
                    builder
                        .build_extract_value(exchanged, 1, "atomic.success")?
                        .into_int_value(),
                    builder.build_extract_value(exchanged, 0, "atomic.observed")?,
                )
            };
            let carrier = function
                .get_type()
                .get_return_type()
                .ok_or(Error::Invariant)?
                .into_struct_type();
            let result = builder.build_insert_value(
                carrier.get_undef(),
                success,
                0,
                "atomic.result.success",
            )?;
            let result =
                builder.build_insert_value(result, observed, 1, "atomic.result.observed")?;
            builder.build_return(Some(&result))?;
        }
        RuntimeIntrinsic::MemoryCopy
        | RuntimeIntrinsic::MemoryCopyReturningDestination
        | RuntimeIntrinsic::MemoryCompare
        | RuntimeIntrinsic::MemorySet
        | RuntimeIntrinsic::MemorySetReturningDestination => {
            let count = arguments[2].into_int_value();
            require(
                context,
                &builder,
                function,
                trap,
                builder.build_int_compare(
                    IntPredicate::SGE,
                    count,
                    count.get_type().const_zero(),
                    "memory.nonnegative",
                )?,
            )?;
            let size_type = context.ptr_sized_int_type(&target.data, None);
            if size_type.get_bit_width() < count.get_type().get_bit_width() {
                let maximum = (1u64 << size_type.get_bit_width()) - 1;
                require(
                    context,
                    &builder,
                    function,
                    trap,
                    builder.build_int_compare(
                        IntPredicate::ULE,
                        count,
                        count.get_type().const_int(maximum, false),
                        "memory.count.fits",
                    )?,
                )?;
            }
            let size = builder.build_int_cast(count, size_type, "memory.size")?;
            let empty = context.append_basic_block(function, "memory.empty");
            let nonempty = context.append_basic_block(function, "memory.nonempty");
            builder.build_conditional_branch(
                builder.build_int_compare(
                    IntPredicate::EQ,
                    size,
                    size_type.const_zero(),
                    "memory.zero",
                )?,
                empty,
                nonempty,
            )?;
            builder.position_at_end(empty);
            if operation == RuntimeIntrinsic::MemoryCompare {
                builder.build_return(Some(&context.i16_type().const_zero()))?;
            } else if matches!(
                operation,
                RuntimeIntrinsic::MemoryCopyReturningDestination
                    | RuntimeIntrinsic::MemorySetReturningDestination
            ) {
                builder.build_return(Some(&arguments[0]))?;
            } else {
                builder.build_return(None)?;
            }
            builder.position_at_end(nonempty);
            let first = arguments[0].into_pointer_value();
            let first_start = checked_range(context, &builder, function, trap, first, size)?;
            match operation {
                RuntimeIntrinsic::MemorySet | RuntimeIntrinsic::MemorySetReturningDestination => {
                    let byte = arguments[1].into_int_value();
                    let byte = if operation == RuntimeIntrinsic::MemorySetReturningDestination {
                        builder.build_int_truncate(byte, context.i8_type(), "memory.byte")?
                    } else {
                        byte
                    };
                    builder.build_memset(first, 1, byte, size)?;
                    if operation == RuntimeIntrinsic::MemorySetReturningDestination {
                        builder.build_return(Some(&first))?;
                    } else {
                        builder.build_return(None)?;
                    }
                }
                RuntimeIntrinsic::MemoryCopy
                | RuntimeIntrinsic::MemoryCopyReturningDestination
                | RuntimeIntrinsic::MemoryCompare => {
                    let second = arguments[1].into_pointer_value();
                    let second_start =
                        checked_range(context, &builder, function, trap, second, size)?;
                    if matches!(
                        operation,
                        RuntimeIntrinsic::MemoryCopy
                            | RuntimeIntrinsic::MemoryCopyReturningDestination
                    ) {
                        let first_end =
                            builder.build_int_add(first_start, size, "copy.first.end")?;
                        let second_end =
                            builder.build_int_add(second_start, size, "copy.second.end")?;
                        let before = builder.build_int_compare(
                            IntPredicate::ULE,
                            first_end,
                            second_start,
                            "copy.first.before",
                        )?;
                        let after = builder.build_int_compare(
                            IntPredicate::ULE,
                            second_end,
                            first_start,
                            "copy.second.before",
                        )?;
                        require(
                            context,
                            &builder,
                            function,
                            trap,
                            builder.build_or(before, after, "copy.disjoint")?,
                        )?;
                        builder.build_memcpy(first, 1, second, 1, size)?;
                        if operation == RuntimeIntrinsic::MemoryCopyReturningDestination {
                            builder.build_return(Some(&first))?;
                        } else {
                            builder.build_return(None)?;
                        }
                    } else {
                        // These supported native C targets all use a 32-bit C int.
                        target.c_platform()?;
                        let ty = context.i32_type().fn_type(
                            &[
                                first.get_type().into(),
                                second.get_type().into(),
                                size_type.into(),
                            ],
                            false,
                        );
                        let compare = if let Some(existing) = module.get_function("memcmp") {
                            if existing.get_type() != ty {
                                return Err(Error::Invariant);
                            }
                            existing
                        } else {
                            module.add_function("memcmp", ty, None)
                        };
                        let result = builder
                            .build_call(
                                compare,
                                &[first.into(), second.into(), size.into()],
                                "memory.compare",
                            )?
                            .try_as_basic_value()
                            .basic()
                            .ok_or(Error::Invariant)?
                            .into_int_value();
                        let less = builder.build_int_compare(
                            IntPredicate::SLT,
                            result,
                            result.get_type().const_zero(),
                            "memory.less",
                        )?;
                        let greater = builder.build_int_compare(
                            IntPredicate::SGT,
                            result,
                            result.get_type().const_zero(),
                            "memory.greater",
                        )?;
                        let positive = builder.build_select(
                            greater,
                            context.i16_type().const_int(1, false),
                            context.i16_type().const_zero(),
                            "memory.positive",
                        )?;
                        let sign = builder.build_select(
                            less,
                            context.i16_type().const_int(u64::MAX, true),
                            positive.into_int_value(),
                            "memory.sign",
                        )?;
                        builder.build_return(Some(&sign))?;
                    }
                }
                RuntimeIntrinsic::PoolGet { .. }
                | RuntimeIntrinsic::PoolReset { .. }
                | RuntimeIntrinsic::PoolRelease { .. }
                | RuntimeIntrinsic::FlatPoolGet { .. }
                | RuntimeIntrinsic::FlatPoolReset { .. }
                | RuntimeIntrinsic::FlatPoolFinish { .. }
                | RuntimeIntrinsic::CompareAndSwap { .. }
                | RuntimeIntrinsic::Swap { .. }
                | RuntimeIntrinsic::DebugTrap => {
                    return Err(Error::Invariant);
                }
            }
        }
    }
    Ok(function)
}

fn require<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    function: FunctionValue<'ctx>,
    trap: FunctionValue<'ctx>,
    condition: IntValue<'ctx>,
) -> Result<(), Error> {
    let valid = context.append_basic_block(function, "intrinsic.valid");
    let invalid = context.append_basic_block(function, "intrinsic.invalid");
    builder.build_conditional_branch(condition, valid, invalid)?;
    builder.position_at_end(invalid);
    builder.build_call(trap, &[], "")?;
    builder.build_unreachable()?;
    builder.position_at_end(valid);
    Ok(())
}
fn checked_range<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    function: FunctionValue<'ctx>,
    trap: FunctionValue<'ctx>,
    pointer: PointerValue<'ctx>,
    size: IntValue<'ctx>,
) -> Result<IntValue<'ctx>, Error> {
    require(
        context,
        builder,
        function,
        trap,
        builder.build_is_not_null(pointer, "memory.nonnull")?,
    )?;
    let start = builder.build_ptr_to_int(pointer, size.get_type(), "memory.start")?;
    let end = builder.build_int_add(start, size, "memory.end")?;
    require(
        context,
        builder,
        function,
        trap,
        builder.build_int_compare(IntPredicate::UGE, end, start, "memory.no.wrap")?,
    )?;
    Ok(start)
}

/// Bool uses an i1 value but byte storage. Retry on upper-bit-only changes so
/// aliased bytes still compare according to the same low-bit semantics as loads.
fn boolean_cas<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    function: FunctionValue<'ctx>,
    pointer: PointerValue<'ctx>,
    expected: IntValue<'ctx>,
    replacement: IntValue<'ctx>,
) -> Result<(IntValue<'ctx>, BasicValueEnum<'ctx>), Error> {
    let initial =
        builder.build_int_z_extend(expected, context.i8_type(), "atomic.bool.expected")?;
    let replacement =
        builder.build_int_z_extend(replacement, context.i8_type(), "atomic.bool.replacement")?;
    let before = builder.get_insert_block().ok_or(Error::Invariant)?;
    let exchange = context.append_basic_block(function, "atomic.bool.exchange");
    let retry = context.append_basic_block(function, "atomic.bool.retry");
    let done = context.append_basic_block(function, "atomic.bool.done");
    builder.build_unconditional_branch(exchange)?;
    builder.position_at_end(exchange);
    let candidate = builder.build_phi(context.i8_type(), "atomic.bool.candidate")?;
    let result = builder.build_cmpxchg(
        pointer,
        candidate.as_basic_value().into_int_value(),
        replacement,
        AtomicOrdering::SequentiallyConsistent,
        AtomicOrdering::SequentiallyConsistent,
    )?;
    let observed_byte = builder
        .build_extract_value(result, 0, "atomic.bool.byte")?
        .into_int_value();
    let success = builder
        .build_extract_value(result, 1, "atomic.bool.success")?
        .into_int_value();
    let observed =
        builder.build_int_truncate(observed_byte, context.bool_type(), "atomic.bool.observed")?;
    let different = builder.build_int_compare(
        IntPredicate::NE,
        observed,
        expected,
        "atomic.bool.different",
    )?;
    builder.build_conditional_branch(
        builder.build_or(success, different, "atomic.bool.complete")?,
        done,
        retry,
    )?;
    builder.position_at_end(retry);
    builder.build_unconditional_branch(exchange)?;
    candidate.add_incoming(&[(&initial, before), (&observed_byte, retry)]);
    builder.position_at_end(done);
    Ok((success, observed.into()))
}
