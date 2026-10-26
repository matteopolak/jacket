//! Byte layouts of the aggregates the language builds in: `string`, array views, resizable
//! arrays and `Any`.
//!
//! Jacket targets 64-bit machines only: pointers, `s64` counts and `Type` values are eight bytes
//! in the checker, the IR, the interpreter and both backends, so [`POINTER_SIZE`] is a constant
//! and not a setting of a target. A 32-bit target would have to change every table here, the
//! prelude's `Allocator` and the code generators together.
//!
//! These tables are the one place offsets and sizes are written down. They mirror the prelude:
//! a resizable array ends in its `Allocator`, so `Allocator` must be [`ALLOCATOR_SIZE`] bytes
//! (`Compiler::preload_type` refuses a prelude where it is not), and `Any` is a `*Type_Info`
//! followed by a `*void`.

/// The size of a pointer, a `Type` and a procedure value.
pub const POINTER_SIZE: u64 = 8;

/// `string` and `[] T`: a count, then the data pointer.
pub mod view {
    use super::POINTER_SIZE;

    pub const COUNT: u64 = 0;
    pub const DATA: u64 = POINTER_SIZE;
    pub const SIZE: u64 = 2 * POINTER_SIZE;
}

/// The prelude's `Allocator`: a procedure and its data pointer.
pub const ALLOCATOR_SIZE: u64 = 2 * POINTER_SIZE;

/// `[..] T`: a view, the allocated capacity, then the allocator.
pub mod resizable {
    use super::{ALLOCATOR_SIZE, POINTER_SIZE, view};

    pub const COUNT: u64 = view::COUNT;
    pub const DATA: u64 = view::DATA;
    pub const ALLOCATED: u64 = view::SIZE;
    pub const ALLOCATOR: u64 = ALLOCATED + POINTER_SIZE;
    pub const SIZE: u64 = ALLOCATOR + ALLOCATOR_SIZE;
}

/// `Any`: the value's type, then a pointer to the value.
pub mod any {
    use super::POINTER_SIZE;

    pub const TYPE: u64 = 0;
    pub const VALUE_POINTER: u64 = POINTER_SIZE;
    pub const SIZE: u64 = 2 * POINTER_SIZE;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tables_match_the_documented_layouts() {
        assert_eq!(view::SIZE, 16);
        assert_eq!(resizable::ALLOCATED, 16);
        assert_eq!(resizable::ALLOCATOR, 24);
        assert_eq!(resizable::SIZE, 40);
        assert_eq!(any::SIZE, 16);
    }
}
