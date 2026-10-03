//! Bytewise equality avoids C string termination and external runtime symbols.
use super::*;
use inkwell::module::Linkage;

impl<'ctx> Generator<'ctx, '_, '_> {
    pub(super) fn compare_strings(
        &mut self,
        relation: Equality,
        left: &ValueExpr,
        right: &ValueExpr,
    ) -> Result<IntValue<'ctx>, Error> {
        // Evaluate both descriptors once before inspecting their backing bytes.
        let left = self.value(left)?.into_struct_value();
        let right = self.value(right)?.into_struct_value();
        let left_count = self
            .builder
            .build_extract_value(left, 0, "string.left.count")?;
        let left_data = self
            .builder
            .build_extract_value(left, 1, "string.left.data")?;
        let right_count = self
            .builder
            .build_extract_value(right, 0, "string.right.count")?;
        let right_data = self
            .builder
            .build_extract_value(right, 1, "string.right.data")?;
        let helper = equality_function(self.context, self.module)?;
        let call = self.builder.build_call(
            helper,
            &[
                left_count.into(),
                left_data.into(),
                right_count.into(),
                right_data.into(),
            ],
            "string.equal",
        )?;
        let result = call
            .try_as_basic_value()
            .basic()
            .ok_or(Error::Invariant)?
            .into_int_value();
        Ok(match relation {
            Equality::Equal => result,
            Equality::NotEqual => self.builder.build_not(result, "string.not.equal")?,
        })
    }
}

fn equality_function<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<FunctionValue<'ctx>, Error> {
    const NAME: &str = "jai.internal.string.equal";
    let count = context.i64_type();
    let byte = context.i8_type();
    let bit = context.bool_type();
    let pointer = context.ptr_type(inkwell::AddressSpace::default());
    let signature = bit.fn_type(
        &[count.into(), pointer.into(), count.into(), pointer.into()],
        false,
    );
    if let Some(function) = module.get_function(NAME) {
        if function.get_linkage() != Linkage::Internal
            || function.get_type() != signature
            || function.count_basic_blocks() == 0
        {
            return Err(Error::Invariant);
        }
        return Ok(function);
    }
    let function = module.add_function(NAME, signature, Some(Linkage::Internal));
    let builder = context.create_builder();
    let entry = context.append_basic_block(function, "entry");
    let lengths = context.append_basic_block(function, "lengths.equal");
    let nonnegative = context.append_basic_block(function, "count.valid");
    let invalid = context.append_basic_block(function, "count.invalid");
    let compare = context.append_basic_block(function, "compare.byte");
    let advance = context.append_basic_block(function, "advance");
    let equal = context.append_basic_block(function, "equal");
    let unequal = context.append_basic_block(function, "unequal");
    let left_count = function
        .get_nth_param(0)
        .ok_or(Error::Invariant)?
        .into_int_value();
    let left_data = function
        .get_nth_param(1)
        .ok_or(Error::Invariant)?
        .into_pointer_value();
    let right_count = function
        .get_nth_param(2)
        .ok_or(Error::Invariant)?
        .into_int_value();
    let right_data = function
        .get_nth_param(3)
        .ok_or(Error::Invariant)?
        .into_pointer_value();
    builder.position_at_end(entry);
    let same_length =
        builder.build_int_compare(IntPredicate::EQ, left_count, right_count, "same.length")?;
    builder.build_conditional_branch(same_length, lengths, unequal)?;
    builder.position_at_end(lengths);
    let negative = builder.build_int_compare(
        IntPredicate::SLT,
        left_count,
        count.const_zero(),
        "negative.count",
    )?;
    builder.build_conditional_branch(negative, invalid, nonnegative)?;
    builder.position_at_end(nonnegative);
    let empty =
        builder.build_int_compare(IntPredicate::EQ, left_count, count.const_zero(), "empty")?;
    builder.build_conditional_branch(empty, equal, compare)?;
    builder.position_at_end(compare);
    let index = builder.build_phi(count, "index")?;
    index.add_incoming(&[(&count.const_zero(), nonnegative)]);
    let offset = index.as_basic_value().into_int_value();
    let left_address = jai_llvm::gep(
        &builder,
        byte.into(),
        left_data,
        &[offset],
        "left.byte.address",
    )?;
    let right_address = jai_llvm::gep(
        &builder,
        byte.into(),
        right_data,
        &[offset],
        "right.byte.address",
    )?;
    let left_byte = builder
        .build_load(byte, left_address, "left.byte")?
        .into_int_value();
    let right_byte = builder
        .build_load(byte, right_address, "right.byte")?
        .into_int_value();
    let same_byte =
        builder.build_int_compare(IntPredicate::EQ, left_byte, right_byte, "same.byte")?;
    builder.build_conditional_branch(same_byte, advance, unequal)?;
    builder.position_at_end(advance);
    let next = builder.build_int_add(offset, count.const_int(1, false), "next.index")?;
    index.add_incoming(&[(&next, advance)]);
    let complete = builder.build_int_compare(IntPredicate::EQ, next, left_count, "complete")?;
    builder.build_conditional_branch(complete, equal, compare)?;
    builder.position_at_end(equal);
    builder.build_return(Some(&bit.const_int(1, false)))?;
    builder.position_at_end(unequal);
    builder.build_return(Some(&bit.const_zero()))?;
    builder.position_at_end(invalid);
    let trap = Intrinsic::find("llvm.trap")
        .ok_or(Error::Invariant)?
        .get_declaration(module, &[])
        .ok_or(Error::Invariant)?;
    builder.build_call(trap, &[], "")?;
    builder.build_unreachable()?;
    Ok(function)
}
