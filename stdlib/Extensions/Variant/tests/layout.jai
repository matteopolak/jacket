// The size and alignment of a Variant: an `index: s32` and payload bytes as large as the largest
// case and aligned for the strictest one.
//
//     jaic stdlib/Extensions/Variant/tests/layout.jai -interp

#import "Basic";
#import "Extensions/Variant";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

Empty :: struct {}

Small :: struct {
    flag: u8;
}

Wide :: struct {
    first: u8;
    second: float64;
}

Triple :: struct {
    values: [3] u16;
}

// Where a Variant's payload sits and what it is aligned to.
payload_offset :: ($V: Type) -> s64 {
    info := cast(*Type_Info_Struct) type_info(V);

    for info.members if it.name == "payload" return it.offset_in_bytes;

    return -1;
}

aligned_at :: (address: *void, alignment: s64) -> bool {
    return cast(s64) address % alignment == 0;
}

sizes :: () {
    check(size_of(Variant(s64, string)) == 24, "Variant(s64, string) is 24 bytes");
    check(align_of(Variant(s64, string)) == 8, "Variant(s64, string) is 8-aligned");
    check(payload_offset(Variant(s64, string)) == 8, "the payload follows the index and padding");

    check(size_of(Variant(u8, float64)) == 16, "Variant(u8, float64) is 16 bytes");
    check(align_of(Variant(u8, float64)) == 8, "Variant(u8, float64) is 8-aligned");
    check(payload_offset(Variant(u8, float64)) == 8, "float64 aligns the payload to 8");

    check(size_of(Variant(u8)) == 8, "Variant(u8): index, a byte and padding");
    check(align_of(Variant(u8)) == 4, "Variant(u8) is as aligned as its index");
    check(payload_offset(Variant(u8)) == 4, "Variant(u8): no padding before the payload");

    check(size_of(Variant(u8, u16)) == 8 && align_of(Variant(u8, u16)) == 4, "Variant(u8, u16)");
    check(payload_offset(Variant(u8, u16)) == 4, "a 2-aligned payload sits right after the index");

    check(size_of(Variant(u8, u32)) == 8 && payload_offset(Variant(u8, u32)) == 4, "Variant(u8, u32)");
    check(size_of(Variant(Empty)) == 8, "a case without a payload still has one byte of storage");
    check(size_of(Variant(Empty, Small)) == 8, "Variant(Empty, Small)");

    check(size_of(Variant(Wide, u8)) == 24, "a struct with padding of its own");
    check(payload_offset(Variant(Wide, u8)) == 8, "a struct case aligns the payload to its strictest member");

    check(size_of(Variant(Triple, u8)) == 12, "a fixed array of u16 is 2-aligned and 6 bytes");
    check(size_of(Variant([4] u8, bool)) == 8, "a fixed array of bytes");

    check(size_of(Variant([..] int, string)) == 8 + size_of([..] int), "a resizable array is the largest case");
    check(size_of(Variant(*void, bool)) == 16, "a pointer");

    // A variant inside a variant is aligned for its own payload.
    check(size_of(Variant(Variant(u8, float64), u8)) == 24, "a nested variant");
    check(payload_offset(Variant(Variant(u8, float64), u8)) == 8, "a nested variant keeps its alignment");
}

Holder :: struct {
    tag: u8;
    inner: Variant(u8, float64);
    after: u8;
    list: [3] Variant(s64, string);
}

placement :: () {
    // Payload bytes are aligned for the strictest case wherever the variant lives.
    value: Variant(u8, float64);

    check(aligned_at(value.payload.data, 8), "on the stack");

    holder: Holder;

    check(aligned_at(holder.inner.payload.data, 8), "inside a struct");

    for *holder.list check(aligned_at(it.payload.data, 8), "in an array");

    heap := New(Variant(u8, float64));

    defer free(heap);

    check(aligned_at(heap.payload.data, 8), "on the heap");

    // The strictest case really is stored whole.
    set(*value, cast(float64) 2.5);

    check(unwrap(value, float64) == 2.5, "a float64 round trips");

    set(*value, cast(u8) 200);

    check(unwrap(value, u8) == 200, "a u8 round trips");

    wide: Variant(Wide, u8);

    set(*wide, Wide.{ 7, 8.5 });

    check(unwrap(wide, Wide).first == 7 && unwrap(wide, Wide).second == 8.5, "a padded struct round trips");

    triple: Variant(Triple, u8);

    set(*triple, Triple.{ .[1, 2, 3] });

    check(unwrap(triple, Triple).values[2] == 3, "a fixed array of u16 round trips");
}

main :: () {
    sizes();
    placement();

    assert(failures == 0);
    print("ok\n");
}
