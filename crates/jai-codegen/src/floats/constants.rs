//! Raw IEEE constants are constructed entirely through LLVM's safe builder API.
use inkwell::{
    builder::BuilderError, context::Context, types::FloatType as LlvmFloatType,
    values::FloatValue as LlvmFloatValue,
};
use jai_types::{FloatType, FloatValue};

pub(super) fn llvm_type(context: &Context, ty: FloatType) -> LlvmFloatType<'_> {
    match ty {
        FloatType::F32 => context.f32_type(),
        FloatType::F64 => context.f64_type(),
    }
}

/// LLVM folds this constant bitcast. A temporary insertion point satisfies the
/// safe builder API, while the resulting uniqued constant belongs to the context.
pub(super) fn constant(
    context: &Context,
    value: FloatValue,
) -> Result<LlvmFloatValue<'_>, BuilderError> {
    let module = context.create_module("float.constant");
    let function = module.add_function("constant", context.void_type().fn_type(&[], false), None);
    let block = context.append_basic_block(function, "entry");
    let builder = context.create_builder();
    builder.position_at_end(block);
    let integer = match value.ty() {
        FloatType::F32 => context.i32_type(),
        FloatType::F64 => context.i64_type(),
    }
    .const_int(value.bits(), false);
    let result = builder.build_bit_cast(integer, llvm_type(context, value.ty()), "float.bits")?;
    builder.build_return(None)?;
    Ok(result.into_float_value())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_constants_preserve_ieee_payloads_with_safe_llvm_bitcasts() {
        let context = Context::create();
        let module = context.create_module("float.bits.test");
        let function = module.add_function("test", context.void_type().fn_type(&[], false), None);
        let block = context.append_basic_block(function, "entry");
        let builder = context.create_builder();
        builder.position_at_end(block);
        for value in [
            FloatValue::F32(0x7fbf_ffff),
            FloatValue::F32(0x8000_0000),
            FloatValue::F32(1),
            FloatValue::F64(0x7ff0_0000_0000_0001),
            FloatValue::F64(0x8000_0000_0000_0000),
        ] {
            let float = constant(&context, value).unwrap();
            let integer = match value.ty() {
                FloatType::F32 => context.i32_type(),
                FloatType::F64 => context.i64_type(),
            };
            let bits = builder
                .build_bit_cast(float, integer, "bits")
                .unwrap()
                .into_int_value();
            assert_eq!(bits.get_zero_extended_constant(), Some(value.bits()));
        }
        builder.build_return(None).unwrap();
        module.verify().unwrap();
    }
}
