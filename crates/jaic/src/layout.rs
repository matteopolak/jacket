//! The data layout of a target: pointer width, byte order, the alignment of each scalar, and
//! the offsets and sizes of the aggregates the language builds in (`string`, array views,
//! resizable arrays, `Any`, `Allocator`).
//!
//! [`Target::data_layout`](crate::target::Target::data_layout) is the only place a layout is
//! chosen. The checker, the IR, the interpreter, data emission and the backends all take a
//! [`DataLayout`]; nothing assumes a pointer is eight bytes or that memory is little-endian.
//! Bytes of *program* data are read and written only through [`DataLayout::read_uint`],
//! [`DataLayout::write_uint`] and their pointer and float forms, which respect width and byte
//! order. A host-only read of the compiler's own memory uses `from_ne_bytes` directly, with an
//! `#[allow]` that says so (`clippy.toml` bans it otherwise).
//!
//! The builtin aggregates mirror the prelude: a resizable array ends in its `Allocator`, so
//! `Allocator` must be [`BuiltinLayouts::allocator`]`.size` bytes (`Compiler::preload_type`
//! refuses a prelude where it is not), and `Any` is a `*Type_Info` followed by a `*void`.

/// Byte order of a target's memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Endian {
    Little,
    Big,
}

/// The sizes of the C types a target's `#foreign` bindings use, in bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CModel {
    pub int: u8,
    pub long: u8,
    pub long_long: u8,
    /// `size_t`, `ptrdiff_t` and `intptr_t`.
    pub size_t: u8,
    /// The `long` members of `struct timespec` and `struct tm` (`tv_nsec`, `tm_gmtoff`) as the
    /// stdlib declares them: a `long`, except on wasm, whose C library keeps them 8 bytes.
    pub time_long: u8,
}

/// How a target lays out data in memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DataLayout {
    /// Bytes in a data pointer, a `Type` and a procedure value: 2, 4 or 8.
    pub pointer_size: u8,
    pub pointer_align: u8,
    pub endian: Endian,
    /// The alignment of `s8`, `s16`, `s32` and `s64`.
    pub int_align: [u8; 4],
    /// The alignment of `float32` and `float64`.
    pub float_align: [u8; 2],
    /// The size and alignment of a C `long double` wider than `float64` that is stored as x87
    /// extended precision (12 bytes on i686, 16 on x86-64).
    pub f80_size: u8,
    pub f80_align: u8,
    pub stack_align: u8,
    pub c: CModel,
}

/// A member of a compiler-built record whose Jai declaration the compiler mirrors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Member {
    /// An integer of this many bytes (1, 2, 4 or 8).
    Int(u8),
    Ptr,
    /// A `string` or `[] T`.
    View,
}

/// Where [`DataLayout::pack`] put each member of a record, and the record's size and alignment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Packed {
    pub offsets: Vec<u64>,
    pub size: u64,
    pub align: u64,
}

/// A group of scalars placed one after another, each at its alignment.
struct Packer {
    size: u64,
    align: u64,
}

impl Packer {
    fn new() -> Self {
        Packer {
            size: 0,
            align: 1,
        }
    }

    /// The offset of a field of `size` and `align`.
    fn field(&mut self, size: u64, align: u64) -> u64 {
        let at = self.size.next_multiple_of(align);
        self.size = at + size;
        self.align = self.align.max(align);
        at
    }

    fn finish(self) -> (u64, u64) {
        (self.size.next_multiple_of(self.align), self.align)
    }
}

/// `string` and `[] T`: a count, then the data pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct View {
    pub count: u64,
    pub data: u64,
    pub size: u64,
    pub align: u64,
}

/// The prelude's `Allocator`: a procedure and its data pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Allocator {
    pub size: u64,
    pub align: u64,
}

/// `[..] T`: a view, the allocated capacity, then the allocator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Resizable {
    pub count: u64,
    pub data: u64,
    pub allocated: u64,
    pub allocator: u64,
    pub size: u64,
    pub align: u64,
}

/// `Any`: the value's type, then a pointer to the value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Any {
    pub type_: u64,
    pub value_pointer: u64,
    pub size: u64,
    pub align: u64,
}

/// The offsets and sizes of the builtin aggregates on one target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BuiltinLayouts {
    pub view: View,
    pub allocator: Allocator,
    pub resizable: Resizable,
    pub any: Any,
}

impl DataLayout {
    /// The layout of a 64-bit little-endian target with the usual scalar alignments: pointers,
    /// `s64` and `float64` align to their size.
    pub const fn lp64_little() -> DataLayout {
        DataLayout {
            pointer_size: 8,
            pointer_align: 8,
            endian: Endian::Little,
            int_align: [1, 2, 4, 8],
            float_align: [4, 8],
            f80_size: 16,
            f80_align: 16,
            stack_align: 16,
            c: CModel {
                int: 4,
                long: 8,
                long_long: 8,
                size_t: 8,
                time_long: 8,
            },
        }
    }

    /// The layout of the platform the compiler runs on: what host memory looks like.
    pub fn host() -> DataLayout {
        let (os, cpu) = crate::sema::compiler_host();
        crate::target::Target::new(os, cpu).data_layout()
    }

    /// Whether pointers are narrower than the compiler's own, so the target's addresses are
    /// offsets into the interpreter's window and its compile-time code cannot call the
    /// compiler's C functions.
    pub const fn is_narrow(&self) -> bool {
        (self.pointer_size as usize) < std::mem::size_of::<usize>()
    }

    /// Whether the program's memory cannot be the compiler's own: its pointers are narrower, or
    /// its byte order differs from the host's. Its compile-time code then runs on the
    /// interpreter's emulated C library, because native functions would read the wrong bytes.
    pub const fn is_foreign_memory(&self) -> bool {
        self.is_narrow() || matches!(self.endian, Endian::Big) != cfg!(target_endian = "big")
    }

    /// The bytes a pointer can address: 64 KiB for a 2-byte pointer, 4 GiB for a 4-byte one, and
    /// `u64::MAX` from 8 bytes up.
    pub const fn address_space(&self) -> u64 {
        match self.pointer_size {
            1..=7 => 1 << (8 * self.pointer_size as u32),
            _ => u64::MAX,
        }
    }

    /// The size in bytes of a pointer as a `u64`.
    pub const fn ptr_bytes(&self) -> u64 {
        self.pointer_size as u64
    }

    pub const fn ptr_bits(&self) -> u32 {
        self.pointer_size as u32 * 8
    }

    /// The alignment in bytes of an integer of `bytes` bytes (1, 2, 4 or 8).
    pub const fn int_align_of(&self, bytes: u64) -> u64 {
        match bytes {
            1 => self.int_align[0] as u64,
            2 => self.int_align[1] as u64,
            4 => self.int_align[2] as u64,
            _ => self.int_align[3] as u64,
        }
    }

    /// The alignment in bytes of a `float32` (4 bytes) or `float64`.
    pub const fn float_align_of(&self, bytes: u64) -> u64 {
        if bytes == 4 {
            self.float_align[0] as u64
        } else {
            self.float_align[1] as u64
        }
    }

    /// The layout of a record with `members` in order, as a Jai struct of them is laid out.
    pub fn pack(&self, members: &[Member]) -> Packed {
        let views = self.builtins().view;
        let mut packer = Packer::new();
        let offsets = members
            .iter()
            .map(|member| match *member {
                Member::Int(bytes) => packer.field(bytes as u64, self.int_align_of(bytes as u64)),
                Member::Ptr => packer.field(self.ptr_bytes(), self.pointer_align as u64),
                Member::View => packer.field(views.size, views.align),
            })
            .collect();
        let (size, align) = packer.finish();
        Packed {
            offsets,
            size,
            align,
        }
    }

    /// The layouts of `string`, views, resizable arrays, `Any` and `Allocator`.
    pub fn builtins(&self) -> BuiltinLayouts {
        let p = self.ptr_bytes();
        let pa = self.pointer_align as u64;
        let (s64, s64_align) = (8, self.int_align_of(8));

        let mut view = Packer::new();
        let count = view.field(s64, s64_align);
        let data = view.field(p, pa);
        let (view_size, view_align) = view.finish();

        let mut allocator = Packer::new();
        allocator.field(p, pa);
        allocator.field(p, pa);
        let (allocator_size, allocator_align) = allocator.finish();

        let mut resizable = Packer::new();
        resizable.field(s64, s64_align);
        resizable.field(p, pa);
        let allocated = resizable.field(s64, s64_align);
        let allocator_at = resizable.field(allocator_size, allocator_align);
        let (resizable_size, resizable_align) = resizable.finish();

        let mut any = Packer::new();
        let type_ = any.field(p, pa);
        let value_pointer = any.field(p, pa);
        let (any_size, any_align) = any.finish();

        BuiltinLayouts {
            view: View {
                count,
                data,
                size: view_size,
                align: view_align,
            },
            allocator: Allocator {
                size: allocator_size,
                align: allocator_align,
            },
            resizable: Resizable {
                count,
                data,
                allocated,
                allocator: allocator_at,
                size: resizable_size,
                align: resizable_align,
            },
            any: Any {
                type_,
                value_pointer,
                size: any_size,
                align: any_align,
            },
        }
    }

    /// The unsigned integer stored in `bytes` (1 to 8 of them) in the target's byte order.
    pub fn read_uint(&self, bytes: &[u8]) -> u64 {
        debug_assert!((1..=8).contains(&bytes.len()));
        let fold = |acc: u64, b: &u8| acc << 8 | u64::from(*b);
        match self.endian {
            Endian::Little => bytes.iter().rev().fold(0, fold),
            Endian::Big => bytes.iter().fold(0, fold),
        }
    }

    /// Store the low `bytes.len()` bytes of `value` in the target's byte order.
    pub fn write_uint(&self, bytes: &mut [u8], value: u64) {
        debug_assert!((1..=8).contains(&bytes.len()));
        let n = bytes.len();
        for (i, byte) in bytes.iter_mut().enumerate() {
            let place = match self.endian {
                Endian::Little => i,
                Endian::Big => n - 1 - i,
            };
            *byte = (value >> (8 * place)) as u8;
        }
    }

    /// The pointer stored at the start of `bytes`.
    pub fn read_ptr(&self, bytes: &[u8]) -> u64 {
        self.read_uint(&bytes[..self.pointer_size as usize])
    }

    /// Store a pointer at the start of `bytes`.
    pub fn write_ptr(&self, bytes: &mut [u8], value: u64) {
        self.write_uint(&mut bytes[..self.pointer_size as usize], value);
    }

    /// The `s64`/`u64` stored at the start of `bytes`.
    pub fn read_u64(&self, bytes: &[u8]) -> u64 {
        self.read_uint(&bytes[..8])
    }

    pub fn write_u64(&self, bytes: &mut [u8], value: u64) {
        self.write_uint(&mut bytes[..8], value);
    }

    /// Store a `string` or view image at the start of `bytes`: the count, then the data pointer.
    pub fn write_view(&self, bytes: &mut [u8], count: u64, data: u64) {
        let view = self.builtins().view;
        self.write_u64(&mut bytes[view.count as usize..], count);
        self.write_ptr(&mut bytes[view.data as usize..], data);
    }

    /// The count and data pointer of the `string` or view image at the start of `bytes`.
    pub fn read_view(&self, bytes: &[u8]) -> (u64, u64) {
        let view = self.builtins().view;
        (
            self.read_u64(&bytes[view.count as usize..]),
            self.read_ptr(&bytes[view.data as usize..]),
        )
    }

    pub fn read_u32(&self, bytes: &[u8]) -> u32 {
        self.read_uint(&bytes[..4]) as u32
    }

    /// The 8 bytes of an `s64` or `u64` in the target's byte order.
    pub fn s64_bytes(&self, value: impl Into<i128>) -> Vec<u8> {
        let mut bytes = vec![0; 8];
        self.write_u64(&mut bytes, value.into() as u64);
        bytes
    }

    pub fn write_u32(&self, bytes: &mut [u8], value: u32) {
        self.write_uint(&mut bytes[..4], u64::from(value));
    }

    pub fn read_u16(&self, bytes: &[u8]) -> u16 {
        self.read_uint(&bytes[..2]) as u16
    }

    pub fn write_u16(&self, bytes: &mut [u8], value: u16) {
        self.write_uint(&mut bytes[..2], u64::from(value));
    }

    pub fn read_f32(&self, bytes: &[u8]) -> f32 {
        f32::from_bits(self.read_u32(bytes))
    }

    pub fn write_f32(&self, bytes: &mut [u8], value: f32) {
        self.write_u32(bytes, value.to_bits());
    }

    pub fn read_f64(&self, bytes: &[u8]) -> f64 {
        f64::from_bits(self.read_u64(bytes))
    }

    pub fn write_f64(&self, bytes: &mut [u8], value: f64) {
        self.write_u64(bytes, value.to_bits());
    }
}

#[cfg(test)]
#[allow(
    clippy::disallowed_methods,
    reason = "the tests build expected target bytes by hand to check the reader"
)]
mod tests {
    use super::*;

    fn big_32() -> DataLayout {
        DataLayout {
            pointer_size: 4,
            pointer_align: 4,
            endian: Endian::Big,
            int_align: [1, 2, 4, 4],
            float_align: [4, 4],
            f80_size: 12,
            f80_align: 4,
            stack_align: 16,
            c: CModel {
                int: 4,
                long: 4,
                long_long: 8,
                size_t: 4,
                time_long: 4,
            },
        }
    }

    #[test]
    fn the_64_bit_tables_match_the_documented_layouts() {
        let b = DataLayout::lp64_little().builtins();
        assert_eq!((b.view.size, b.view.count, b.view.data), (16, 0, 8));
        assert_eq!(b.allocator.size, 16);
        assert_eq!(b.resizable.allocated, 16);
        assert_eq!(b.resizable.allocator, 24);
        assert_eq!(b.resizable.size, 40);
        assert_eq!((b.any.size, b.any.type_, b.any.value_pointer), (16, 0, 8));
    }

    #[test]
    fn a_narrow_pointer_shrinks_the_builtins_around_the_s64_count() {
        let b = big_32().builtins();
        assert_eq!((b.view.size, b.view.data, b.view.align), (12, 8, 4));
        assert_eq!(b.allocator.size, 8);
        assert_eq!((b.resizable.allocated, b.resizable.allocator), (12, 20));
        assert_eq!(b.resizable.size, 28);
        assert_eq!(b.any.size, 8);
    }

    #[test]
    fn an_eight_aligned_s64_pads_a_narrow_view() {
        let mut l = big_32();
        l.int_align = [1, 2, 4, 8];
        let b = l.builtins();
        assert_eq!((b.view.size, b.view.data, b.view.align), (16, 8, 8));
        assert_eq!(b.resizable.allocator, 24);
        assert_eq!(b.resizable.size, 32);
    }

    #[test]
    fn a_packed_record_follows_the_targets_alignments() {
        let members = [Member::Int(2), Member::View, Member::Ptr];
        let wide = DataLayout::lp64_little().pack(&members);
        assert_eq!((wide.offsets, wide.size), (vec![0, 8, 24], 32));
        let narrow = big_32().pack(&members);
        assert_eq!((narrow.offsets, narrow.size), (vec![0, 4, 16], 20));
    }

    #[test]
    fn integers_round_trip_in_either_byte_order() {
        for layout in [DataLayout::lp64_little(), big_32()] {
            let mut bytes = [0u8; 8];
            for size in 1..=8 {
                let value = 0x0102_0304_0506_0708_u64 >> (8 * (8 - size));
                layout.write_uint(&mut bytes[..size], value);
                assert_eq!(layout.read_uint(&bytes[..size]), value, "{size} bytes");
            }
        }
    }

    #[test]
    fn byte_order_decides_where_the_low_byte_goes() {
        let mut bytes = [0u8; 4];
        DataLayout::lp64_little().write_u32(&mut bytes, 0x0102_0304);
        assert_eq!(bytes, [4, 3, 2, 1]);
        big_32().write_u32(&mut bytes, 0x0102_0304);
        assert_eq!(bytes, [1, 2, 3, 4]);
        assert_eq!(big_32().read_f32(&1.5f32.to_bits().to_be_bytes()), 1.5);
    }

    #[test]
    fn a_pointer_uses_only_the_target_width() {
        let layout = big_32();
        let mut bytes = [0xff; 8];
        layout.write_ptr(&mut bytes, 0x1122_3344);
        assert_eq!(bytes, [0x11, 0x22, 0x33, 0x44, 0xff, 0xff, 0xff, 0xff]);
        assert_eq!(layout.read_ptr(&bytes), 0x1122_3344);
    }
}
