// Stored values of 8-bit relative pointers (sign bit flipped) and a block of pointers copied to a new address.
#import "Basic";
R :: #import "Relative_Pointers";

Frame8 :: struct {
    target: R.Relative_Pointer(s8, u8);
    data: [8] u8;
}

Back8 :: struct {
    data: [8] u8;
    target: R.Relative_Pointer(s8, u8);
}

Ref16 :: struct {
    target: R.Relative_Pointer(s16, s32);
}

Block :: struct {
    refs: [4] Ref16;
    values: [4] s32;
}

main :: () {
    // The target sits 4 bytes after the pointer: stored as 4 XOR 0x80.
    frame: Frame8;

    R.encode(*frame.target, *frame.data[3]);
    print("forward raw=% decoded_offset=%\n", cast(s64) frame.target.value, cast(s64) (cast(*u8) R.decode(*frame.target) - cast(*u8) *frame.target));
    assert(frame.target.value == -124);
    assert(R.decode(*frame.target) == *frame.data[3]);

    // A target at the pointer's own address: stored as 0 XOR 0x80.
    R.write_relative_pointer(s8, *frame.target.value, *frame.target.value);
    print("self raw=%\n", cast(s64) frame.target.value);
    assert(frame.target.value == -128);
    assert(R.decode_relative_pointer_static(s8, *frame.target.value) == *frame.target.value);

    // A null target: stored as 0.
    R.encode(*frame.target, null);
    print("null raw=%\n", cast(s64) frame.target.value);
    assert(frame.target.value == 0 && R.decode(*frame.target) == null);

    // A target 6 bytes before the pointer: -6 XOR 0x80 is 0x7A.
    back: Back8;

    R.encode(*back.target, *back.data[2]);
    print("backward raw=% decoded=%\n", cast(s64) back.target.value, R.decode(*back.target) == *back.data[2]);
    assert(back.target.value == 122);
    assert(R.decode(*back.target) == *back.data[2]);

    // Pointers into a block still resolve after the block is copied; the copy's pointers point into the copy.
    block: Block;
    for 0..3 block.values[it] = cast(s32) (10 * (it + 1));
    for 0..2 R.encode(*block.refs[it].target, *block.values[it]);

    R.encode(*block.refs[3].target, null);

    moved := block;
    moved.values[0] = 99;

    for 0..2 {
        original := R.decode(*block.refs[it].target);
        copied := R.decode(*moved.refs[it].target);

        assert(original != copied);
        assert(original == *block.values[it] && copied == *moved.values[it]);
        print("ref % original=% copied=%\n", it, original.*, copied.*);
    }

    assert(R.decode(*moved.refs[3].target) == null);
    assert(block.values[0] == 10 && moved.values[0] == 99);
    print("ok\n");
}
