// Explicit-base relative pointers in 64-bit storage, and Relative_String values that survive a copy.
#import "Basic";
R :: #import "Relative_Pointers";

Name :: struct {
    text: R.Relative_String(s32);
    tail: [5] u8;
}

main :: () {
    // A 64-bit slot points 12 bytes before its base address.
    region: [32] u8;
    base := region.data + 16;
    target := region.data + 4;
    slot: s64;

    R.write_relative_pointer(s64, *slot, base, target);
    decoded, offset := R.decode_relative_pointer(cast(s32) size_of(s64), *slot, base);

    print("explicit base offset=% matches=%\n", offset, decoded == target);
    assert(offset == -12 && decoded == target);

    // A slot whose target is its base has offset zero.
    R.write_relative_pointer(s64, *slot, base, base);
    decoded, offset = R.decode_relative_pointer(cast(s32) size_of(s64), *slot, base);
    assert(offset == 0 && decoded == base);

    // A string whose bytes live in the same struct.
    name: Name;
    for byte, index: "hello" name.tail[index] = byte;

    name.text.count = 5;
    R.write_relative_pointer(s32, *name.text.data, name.tail.data);
    view := R.decode(*name.text);

    assert(view == "hello" && view.data == name.tail.data);

    moved := name;

    view = R.decode(*moved.text);
    print("copied string=%  points into copy=%\n", view, view.data == moved.tail.data);
    assert(view == "hello" && view.data == moved.tail.data);
    assert(R.decode(*name.text) == "hello");

    // A zero-length string.
    empty: Name;
    empty.text.count = 0;
    R.write_relative_pointer(s32, *empty.text.data, empty.tail.data);
    view = R.decode(*empty.text);
    print("empty count=%\n", view.count);
    assert(view.count == 0);
    print("ok\n");
}
