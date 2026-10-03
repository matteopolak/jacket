//! Checked boundaries for LLVM operations missing safe Inkwell equivalents.
mod debug_records;
#[allow(unsafe_code)]
mod raw;
pub use debug_records::{
    DebugCallingConvention, DebugEmission, DebugMember, DebugPrimitive, DebugRecordKind,
    DebugResultMember, DebugScope, DebugSession, DebugSource, DebugType, DebugVariable,
    DebugVariableKind, DebugVariadic,
};
use inkwell::{
    IntPredicate,
    builder::{Builder, BuilderError},
    context::ContextRef,
    types::BasicTypeEnum,
    values::{IntValue, PointerValue},
};
use std::fmt;

/// Reanchors only callee locations inlined through this bridge's artificial-call
/// ledger. Normal source calls and their inlined variables remain unchanged.
pub fn normalize_suppressed_debug(module: &inkwell::module::Module<'_>) {
    raw::suppressed_debug::normalize(module);
}

#[derive(Debug)]
pub enum Error {
    Build(BuilderError),
    ContextMismatch,
    Unsized,
    InvalidIndex,
    InvalidName,
    ExpectedConstant,
    NullResult,
    InvalidDebugFormat,
    DebugOwnership,
    DebugStorageOwnership,
    InvalidDebugType,
    InvalidDebugCoordinates,
    InvalidDebugParameter,
}
impl From<BuilderError> for Error {
    fn from(error: BuilderError) -> Self {
        Self::Build(error)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Build(error) => error.fmt(f),
            Self::ContextMismatch => f.write_str("LLVM operands belong to different contexts"),
            Self::Unsized => f.write_str("LLVM pointer element has no storage size"),
            Self::InvalidIndex => f.write_str("LLVM pointer index does not select a valid element"),
            Self::InvalidName => f.write_str("LLVM instruction name contains a null byte"),
            Self::ExpectedConstant => {
                f.write_str("LLVM constant address requires constant operands")
            }
            Self::NullResult => f.write_str("LLVM returned no instruction value"),
            Self::InvalidDebugFormat => {
                f.write_str("LLVM debug records require a module using the new debug format")
            }
            Self::DebugOwnership => f.write_str("LLVM debug metadata belongs to another session"),
            Self::DebugStorageOwnership => {
                f.write_str("LLVM debug storage has unsupported or foreign ownership")
            }
            Self::InvalidDebugType => {
                f.write_str("LLVM debug type layout or construction is invalid")
            }
            Self::InvalidDebugCoordinates => {
                f.write_str("LLVM debug source column exceeds its 16-bit field")
            }
            Self::InvalidDebugParameter => {
                f.write_str("LLVM debug parameter ordinal exceeds its 16-bit field")
            }
        }
    }
}
impl std::error::Error for Error {}

fn context(ty: BasicTypeEnum<'_>) -> ContextRef<'_> {
    match ty {
        BasicTypeEnum::ArrayType(ty) => ty.get_context(),
        BasicTypeEnum::FloatType(ty) => ty.get_context(),
        BasicTypeEnum::IntType(ty) => ty.get_context(),
        BasicTypeEnum::PointerType(ty) => ty.get_context(),
        BasicTypeEnum::StructType(ty) => ty.get_context(),
        BasicTypeEnum::VectorType(ty) => ty.get_context(),
        BasicTypeEnum::ScalableVectorType(ty) => ty.get_context(),
    }
}
fn builder_context<'ctx>(builder: &Builder<'ctx>, name: &str) -> Result<ContextRef<'ctx>, Error> {
    if name.as_bytes().contains(&0) {
        return Err(Error::InvalidName);
    }
    Ok(builder
        .get_insert_block()
        .ok_or(BuilderError::UnsetPosition)?
        .get_context())
}

struct CheckedGep<'a, 'ctx> {
    builder: &'a Builder<'ctx>,
    element: BasicTypeEnum<'ctx>,
    pointer: PointerValue<'ctx>,
    indices: &'a [IntValue<'ctx>],
    name: &'a str,
}

/// Emit a non-inbounds GEP after checking its complete structural index path.
pub fn gep<'ctx>(
    builder: &Builder<'ctx>,
    element: BasicTypeEnum<'ctx>,
    pointer: PointerValue<'ctx>,
    indices: &[IntValue<'ctx>],
    name: &str,
) -> Result<PointerValue<'ctx>, Error> {
    let owner = builder_context(builder, name)?;
    if owner != pointer.get_type().get_context() {
        return Err(Error::ContextMismatch);
    }
    validate_path(element, pointer, indices)?;
    raw::gep(CheckedGep {
        builder,
        element,
        pointer,
        indices,
        name,
    })
}

fn validate_path<'ctx>(
    element: BasicTypeEnum<'ctx>,
    pointer: PointerValue<'ctx>,
    indices: &[IntValue<'ctx>],
) -> Result<(), Error> {
    let owner = pointer.get_type().get_context();
    if indices.len() > u32::MAX as usize {
        return Err(Error::InvalidIndex);
    }
    if owner != context(element)
        || owner != pointer.get_type().get_context()
        || indices
            .iter()
            .any(|index| owner != index.get_type().get_context())
    {
        return Err(Error::ContextMismatch);
    }
    if !raw::sized(element) {
        return Err(Error::Unsized);
    }
    let mut selected = element;
    for index in indices.iter().skip(1) {
        selected = match selected {
            BasicTypeEnum::ArrayType(array) => array.get_element_type(),
            BasicTypeEnum::VectorType(vector) => vector.get_element_type(),
            BasicTypeEnum::StructType(record) => {
                if index.get_type().get_bit_width() != 32 {
                    return Err(Error::InvalidIndex);
                }
                let field = index
                    .get_zero_extended_constant()
                    .and_then(|field| u32::try_from(field).ok())
                    .ok_or(Error::InvalidIndex)?;
                record
                    .get_field_type_at_index(field)
                    .ok_or(Error::InvalidIndex)?
            }
            _ => return Err(Error::InvalidIndex),
        };
    }
    Ok(())
}

struct CheckedConstGep<'a, 'ctx> {
    element: BasicTypeEnum<'ctx>,
    pointer: PointerValue<'ctx>,
    indices: &'a [IntValue<'ctx>],
}

/// Construct a relocatable constant address with a checked aggregate index path.
pub fn const_gep<'ctx>(
    element: BasicTypeEnum<'ctx>,
    pointer: PointerValue<'ctx>,
    indices: &[IntValue<'ctx>],
) -> Result<PointerValue<'ctx>, Error> {
    validate_path(element, pointer, indices)?;
    if !pointer.is_const() || indices.iter().any(|index| !index.is_const()) {
        return Err(Error::ExpectedConstant);
    }
    Ok(raw::const_gep(CheckedConstGep {
        element,
        pointer,
        indices,
    }))
}

struct CheckedComparison<'a, 'ctx> {
    builder: &'a Builder<'ctx>,
    left: PointerValue<'ctx>,
    right: PointerValue<'ctx>,
    predicate: IntPredicate,
    name: &'a str,
}

/// Compare pointer values directly, retaining address-space and pointer semantics.
pub fn compare_pointers<'ctx>(
    builder: &Builder<'ctx>,
    left: PointerValue<'ctx>,
    right: PointerValue<'ctx>,
    predicate: IntPredicate,
    name: &str,
) -> Result<IntValue<'ctx>, Error> {
    let owner = builder_context(builder, name)?;
    if owner != left.get_type().get_context() || owner != right.get_type().get_context() {
        return Err(Error::ContextMismatch);
    }
    if left.get_type() != right.get_type() {
        return Err(BuilderError::NotSameType.into());
    }
    raw::compare(CheckedComparison {
        builder,
        left,
        right,
        predicate,
        name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::{AddressSpace, context::Context};
    #[test]
    fn array_and_nested_struct_indices_generate_verified_ir() {
        let context = Context::create();
        let module = context.create_module("gep");
        let builder = context.create_builder();
        let int = context.i64_type();
        let function = module.add_function("main", int.fn_type(&[], false), None);
        builder.position_at_end(context.append_basic_block(function, "entry"));
        let array = int.array_type(4);
        let record = context.struct_type(&[array.into()], false);
        let pointer = builder.build_alloca(record, "record").unwrap();
        let indices = [
            int.const_zero(),
            context.i32_type().const_zero(),
            int.const_int(3, false),
        ];
        let element = gep(&builder, record.into(), pointer, &indices, "element").unwrap();
        builder
            .build_store(element, int.const_int(42, false))
            .unwrap();
        let value = builder
            .build_load(int, element, "value")
            .unwrap()
            .into_int_value();
        let same = compare_pointers(&builder, element, element, IntPredicate::EQ, "same").unwrap();
        let same = builder.build_int_z_extend(same, int, "wide").unwrap();
        builder
            .build_return(Some(&builder.build_int_add(value, same, "sum").unwrap()))
            .unwrap();
        module.verify().unwrap();
    }
    #[test]
    fn invalid_struct_indices_and_unsized_elements_are_rejected_before_ffi() {
        let context = Context::create();
        let module = context.create_module("reject");
        let builder = context.create_builder();
        let int = context.i64_type();
        let function = module.add_function("f", int.fn_type(&[int.into()], false), None);
        builder.position_at_end(context.append_basic_block(function, "entry"));
        let record = context.struct_type(&[int.into()], false);
        let pointer = builder.build_alloca(record, "record").unwrap();
        for selector in [
            int.const_zero(),
            function.get_first_param().unwrap().into_int_value(),
            context.i32_type().const_int(9, false),
        ] {
            assert!(matches!(
                gep(
                    &builder,
                    record.into(),
                    pointer,
                    &[int.const_zero(), selector],
                    "invalid"
                ),
                Err(Error::InvalidIndex)
            ));
        }
        let opaque = context.opaque_struct_type("pending");
        assert!(matches!(
            gep(
                &builder,
                opaque.into(),
                pointer,
                &[int.const_zero()],
                "opaque"
            ),
            Err(Error::Unsized)
        ));
        builder.build_return(Some(&int.const_zero())).unwrap();
        module.verify().unwrap();
    }
    #[test]
    fn contexts_address_spaces_names_and_position_are_checked() {
        let context = Context::create();
        let other = Context::create();
        let builder = context.create_builder();
        let pointer = context.ptr_type(AddressSpace::default()).const_null();
        assert!(matches!(
            compare_pointers(&builder, pointer, pointer, IntPredicate::EQ, "same"),
            Err(Error::Build(BuilderError::UnsetPosition))
        ));
        let module = context.create_module("checks");
        let function = module.add_function("f", context.void_type().fn_type(&[], false), None);
        builder.position_at_end(context.append_basic_block(function, "entry"));
        let foreign = other.ptr_type(AddressSpace::default()).const_null();
        assert!(matches!(
            compare_pointers(&builder, pointer, foreign, IntPredicate::EQ, "foreign"),
            Err(Error::ContextMismatch)
        ));
        let space = context.ptr_type(AddressSpace::from(1u16)).const_null();
        assert!(matches!(
            compare_pointers(&builder, pointer, space, IntPredicate::EQ, "space"),
            Err(Error::Build(BuilderError::NotSameType))
        ));
        assert!(matches!(
            compare_pointers(&builder, pointer, pointer, IntPredicate::EQ, "bad\0name"),
            Err(Error::InvalidName)
        ));
        let foreign_index = other.i64_type().const_zero();
        assert!(matches!(
            gep(
                &builder,
                context.i64_type().into(),
                pointer,
                &[foreign_index],
                "index"
            ),
            Err(Error::ContextMismatch)
        ));
        builder.build_return(None).unwrap();
        module.verify().unwrap();
    }
    #[test]
    fn constant_offsets_preserve_global_relocations_and_reject_runtime_operands() {
        let context = Context::create();
        let module = context.create_module("relocation");
        let int = context.i64_type();
        let array = int.array_type(3);
        let storage = module.add_global(array, None, "values");
        storage.set_initializer(&int.const_array(&[
            int.const_int(1, false),
            int.const_int(2, false),
            int.const_int(3, false),
        ]));
        let pointer = const_gep(
            array.into(),
            storage.as_pointer_value(),
            &[int.const_zero(), int.const_int(2, false)],
        )
        .unwrap();
        let address = module.add_global(pointer.get_type(), None, "last");
        address.set_initializer(&pointer);
        let builder = context.create_builder();
        let function = module.add_function("f", int.fn_type(&[int.into()], false), None);
        builder.position_at_end(context.append_basic_block(function, "entry"));
        let index = function.get_first_param().unwrap().into_int_value();
        assert!(matches!(
            const_gep(
                array.into(),
                storage.as_pointer_value(),
                &[int.const_zero(), index]
            ),
            Err(Error::ExpectedConstant)
        ));
        let local = builder.build_alloca(array, "local").unwrap();
        assert!(matches!(
            const_gep(array.into(), local, &[int.const_zero()]),
            Err(Error::ExpectedConstant)
        ));
        builder
            .build_return(Some(
                &builder
                    .build_load(int, pointer, "value")
                    .unwrap()
                    .into_int_value(),
            ))
            .unwrap();
        module.verify().unwrap();
    }
}
