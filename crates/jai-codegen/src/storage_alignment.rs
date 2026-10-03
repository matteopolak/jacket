//! Bound source storage alignment to the selected target's address space.
use inkwell::{targets::TargetData, types::BasicTypeEnum};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    UnsupportedPointerWidth(u32),
    ExceedsTargetAddressSpace {
        size: u64,
        alignment: u32,
        maximum: u64,
    },
}
impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPointerWidth(bits) => {
                write!(
                    formatter,
                    "storage allocation requires a supported target pointer width, got {bits}"
                )
            }
            Self::ExceedsTargetAddressSpace {
                size,
                alignment,
                maximum,
            } => write!(
                formatter,
                "storage size {size} or alignment {alignment} exceeds target signed address limit {maximum}",
            ),
        }
    }
}
impl std::error::Error for Error {}

/// Declaration alignment raises the allocation guarantee without changing its type.
pub(super) fn allocation(
    target: &TargetData,
    ty: BasicTypeEnum<'_>,
    requested: Option<u32>,
) -> Result<u32, Error> {
    let alignment = target.get_abi_alignment(&ty).max(requested.unwrap_or(1));
    let bits = target.get_pointer_byte_size(None) * 8;
    let maximum = match bits {
        1..=64 => (1u64 << (bits - 1)) - 1,
        _ => return Err(Error::UnsupportedPointerWidth(bits)),
    };
    let size = target.get_abi_size(&ty);
    if size > maximum || u64::from(alignment) > maximum {
        return Err(Error::ExceedsTargetAddressSpace {
            size,
            alignment,
            maximum,
        });
    }
    Ok(alignment)
}

#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::context::Context;

    #[test]
    fn storage_alignment_is_independent_of_type_layout() {
        let context = Context::create();
        let target = TargetData::create("e-p:64:64-i64:64");
        let ty = context.i8_type().array_type(7).into();
        assert_eq!(allocation(&target, ty, Some(64)), Ok(64));
        assert_eq!(target.get_abi_size(&ty), 7);
        assert_eq!(target.get_abi_alignment(&ty), 1);
        assert_eq!(
            allocation(&target, context.i64_type().into(), Some(1)),
            Ok(8)
        );
    }

    #[test]
    fn narrow_target_rejects_unrepresentable_storage_requirements() {
        let context = Context::create();
        let target = TargetData::create("e-p:32:32-i64:64");
        assert!(matches!(
            allocation(&target, context.i8_type().into(), Some(1 << 31)),
            Err(Error::ExceedsTargetAddressSpace { .. }),
        ));
        assert!(matches!(
            allocation(&target, context.i8_type().array_type(u32::MAX).into(), None),
            Err(Error::ExceedsTargetAddressSpace { .. }),
        ));
    }
}
