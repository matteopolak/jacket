//! Borrow canonical portable type recipes without allocating a decoded copy.
use super::*;
use jai_syntax::{MAX_SOURCE_METADATA_DEPTH, SourceMetadataError};
use std::mem::size_of;

fn visit<E>(
    value: &ModuleType,
    depth: usize,
    admit: &mut impl FnMut(usize, usize) -> Result<(), E>,
) -> Result<(), SourceMetadataError<E>> {
    if depth > MAX_SOURCE_METADATA_DEPTH {
        return Err(SourceMetadataError::ExcessiveNesting);
    }
    admit(1, 0).map_err(SourceMetadataError::Admission)?;
    match value {
        ModuleType::Builtin(_) | ModuleType::Declaration(_) => Ok(()),
        ModuleType::Pointer(value)
        | ModuleType::Slice(value)
        | ModuleType::DynamicArray(value)
        | ModuleType::FixedArray {
            element: value, ..
        } => {
            admit(1, size_of::<ModuleType>()).map_err(SourceMetadataError::Admission)?;
            visit(value, depth + 1, admit)
        }
        ModuleType::Application {
            arguments, ..
        } => {
            admit(
                arguments.capacity().saturating_add(1),
                arguments
                    .capacity()
                    .saturating_mul(size_of::<ModuleBoundArgument>()),
            )
            .map_err(SourceMetadataError::Admission)?;
            for argument in arguments {
                visit_argument(argument, depth + 1, admit)?;
            }
            Ok(())
        }
        ModuleType::Procedure(value) => {
            for rows in [&value.parameters, &value.results] {
                admit(
                    rows.capacity().saturating_add(1),
                    rows.capacity().saturating_mul(size_of::<ModuleType>()),
                )
                .map_err(SourceMetadataError::Admission)?;
                for row in rows {
                    visit(row, depth + 1, admit)?;
                }
            }
            Ok(())
        }
    }
}
fn visit_argument<E>(
    value: &ModuleBoundArgument,
    depth: usize,
    admit: &mut impl FnMut(usize, usize) -> Result<(), E>,
) -> Result<(), SourceMetadataError<E>> {
    if depth > MAX_SOURCE_METADATA_DEPTH {
        return Err(SourceMetadataError::ExcessiveNesting);
    }
    admit(1, 0).map_err(SourceMetadataError::Admission)?;
    match value {
        ModuleBoundArgument::Type(value) => visit(value, depth + 1, admit),
        ModuleBoundArgument::String(value) => admit(value.len().saturating_add(1), value.len())
            .map_err(SourceMetadataError::Admission),
        ModuleBoundArgument::Integer(_)
        | ModuleBoundArgument::Boolean(_)
        | ModuleBoundArgument::Float(_)
        | ModuleBoundArgument::Enumeration(_) => Ok(()),
    }
}
impl ModuleType {
    pub fn visit_retained_metadata<E>(
        &self,
        admit: &mut impl FnMut(usize, usize) -> Result<(), E>,
    ) -> Result<(), SourceMetadataError<E>> {
        visit(self, 0, admit)
    }
}
impl ModuleBoundArgument {
    pub fn visit_retained_metadata<E>(
        &self,
        admit: &mut impl FnMut(usize, usize) -> Result<(), E>,
    ) -> Result<(), SourceMetadataError<E>> {
        visit_argument(self, 0, admit)
    }
}
