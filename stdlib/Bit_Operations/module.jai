// Byte swapping, bit scans and population count for integers of any size.

// Reverses the byte order of `input` and returns the result; `input` is untouched.
//
// Works on the bytes of any type, so `u16`, `s32`, `u64` and plain structs are swapped the same way (the
// whole value is reversed, not each field). Swapping twice gives the original value back.
//
// ```jai
// assert(byte_swap(cast(u32) 0x11223344) == 0x44332211);
// assert(byte_swap(cast(u16) 0x00ff) == 0xff00);
// assert(byte_swap(byte_swap(cast(u64) 0x0102030405060708)) == 0x0102030405060708);
// ```
byte_swap :: Impl.byte_swap;
// Reverses the byte order of `input` where it is stored, the in-place form of `byte_swap`.
//
// ```jai
// value: u32 = 0xAABBCCDD;
// byte_swap_in_place(*value);
// assert(value == 0xDDCCBBAA);
// ```
byte_swap_in_place :: Impl.byte_swap_in_place;
// The 1-based position of the lowest set bit of `input`: 1 for bit 0, 4 for bit 3. Returns 0 when no bit is set.
//
// Only the `size_of(T)` bytes of the value count, so a negative signed integer is scanned as its
// two's-complement bits and never as a sign-extended 64-bit number.
//
// ```jai
// assert(bit_scan_forward(cast(u32) 1) == 1);
// assert(bit_scan_forward(cast(u32) 0b1000) == 4);
// assert(bit_scan_forward(cast(u8) 0b1010_0000) == 6);
// assert(bit_scan_forward(cast(u32) 0) == 0);
// ```
bit_scan_forward :: Impl.bit_scan_forward;
// The 1-based position of the highest set bit of `input`, which is also the number of bits needed to hold it (so `floor(log2)` plus one). Returns 0 when no bit is set.
//
// A negative signed integer has its top bit set, so the result is the full width of the type.
//
// ```jai
// assert(bit_scan_reverse(cast(u32) 1) == 1);
// assert(bit_scan_reverse(cast(u32) 0b1000) == 4);
// assert(bit_scan_reverse(cast(u64) 0xFFFF_FFFF_FFFF_FFFF) == 64);
// assert(bit_scan_reverse(cast(s16) -1) == 16);
// assert(bit_scan_reverse(cast(u32) 0) == 0);
// ```
bit_scan_reverse :: Impl.bit_scan_reverse;
// The number of bits set in `value`, counting only its `size_of(T)` bytes.
//
// ```jai
// assert(popcount(cast(u32) 0) == 0);
// assert(popcount(cast(u32) 0b1011) == 3);
// assert(popcount(cast(s8) -1) == 8);
// assert(popcount(cast(u64) 0xFFFF_FFFF_FFFF_FFFF) == 64);
// ```
popcount :: Impl.popcount;

#scope_file
Impl :: #import "internal/bits";
