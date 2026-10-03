//! Target-aligned opaque union payloads and LLVM-only member reinterpretation.
use inkwell::{
    builder::{Builder, BuilderError},
    context::Context,
    targets::TargetData,
    types::{BasicType, BasicTypeEnum, StructType},
    values::{BasicValueEnum, PointerValue, StructValue},
};
use jai_types::Layout;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    PayloadTooLarge(u64),
    Layout { expected: Layout, actual: Layout },
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PayloadTooLarge(size) => write!(
                f,
                "union payload size {size} exceeds the LLVM array API limit"
            ),
            Self::Layout { expected, actual } => write!(
                f,
                "union storage differs from target policy: expected {expected:?}, actual {actual:?}"
            ),
        }
    }
}
impl std::error::Error for Error {}

/// The zero-length carrier imposes the strongest member alignment without
/// representing an active member. The byte payload begins at offset zero.
pub fn body<'ctx>(
    context: &'ctx Context,
    target: &TargetData,
    members: &[BasicTypeEnum<'ctx>],
    expected: &Layout,
) -> Result<Vec<BasicTypeEnum<'ctx>>, Error> {
    let carrier = members
        .iter()
        .copied()
        .max_by_key(|member| target.get_abi_alignment(member))
        .unwrap_or(context.i8_type().into());
    let count = u32::try_from(expected.size).map_err(|_| Error::PayloadTooLarge(expected.size))?;
    let body = vec![
        carrier.array_type(0).into(),
        context.i8_type().array_type(count).into(),
    ];
    let storage = context.struct_type(&body, false);
    let actual = Layout {
        size: target.get_abi_size(&storage),
        alignment: target.get_abi_alignment(&storage),
        field_offsets: vec![0; members.len()].into_boxed_slice(),
        array_stride: None,
    };
    if actual != *expected {
        return Err(Error::Layout {
            expected: expected.clone(),
            actual,
        });
    }
    Ok(body)
}

/// Compiler-owned temporary allocation in function entry, never per iteration.
pub fn entry_alloca<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    ty: BasicTypeEnum<'ctx>,
    name: &str,
) -> Result<PointerValue<'ctx>, BuilderError> {
    let function = builder
        .get_insert_block()
        .and_then(|block| block.get_parent())
        .ok_or(BuilderError::UnsetPosition)?;
    let entry = function
        .get_first_basic_block()
        .ok_or(BuilderError::UnsetPosition)?;
    let allocation = context.create_builder();
    if let Some(first) = entry.get_first_instruction() {
        allocation.position_before(&first);
    } else {
        allocation.position_at_end(entry);
    }
    allocation.build_alloca(ty, name)
}

/// Checked semantic ownership and member size must be validated by the caller.
pub fn construct<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    storage: StructType<'ctx>,
    member: BasicValueEnum<'ctx>,
) -> Result<StructValue<'ctx>, crate::Error> {
    let temporary = entry_alloca(context, builder, storage.into(), "union.snapshot")?;
    builder.build_store(temporary, storage.const_zero())?;
    crate::memory::store(builder, temporary, member, 1)?;
    Ok(builder
        .build_load(storage, temporary, "union.value")?
        .into_struct_value())
}

/// The full union snapshot is copied before loading a member at byte offset zero.
pub fn extract<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    snapshot: StructValue<'ctx>,
    member: BasicTypeEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, crate::Error> {
    let temporary = entry_alloca(
        context,
        builder,
        snapshot.get_type().into(),
        "union.snapshot",
    )?;
    builder.build_store(temporary, snapshot)?;
    crate::memory::load(builder, member, temporary, "union.member", 1)
}
