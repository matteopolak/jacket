#import "Basic";
#import "Bit_Array";
B :: #import "Bit_Array";
Again :: #import "Bit_Array";

main :: () {
    bits := make_bit_array(65);

    defer deinit(*bits);
    bits[0] = true;
    bits[64] = true;
    assert(bits[0] && bits[64] && !bits[1] && !bits[63]);
    bits[0] = false;
    assert(!bits[0] && bits[64]);
    assert((bits.slots[1] & 1) != 0);
    first: (Bit_Array, int) -> bool = B.operator [];
    second: (Again.Bit_Array, int) -> bool = Again.operator [];
    store: (*Bit_Array, int, bool) = B.operator []=;
    store_again: (*Again.Bit_Array, int, bool) = Again.operator []=;

    assert(first == second && store == store_again);
    store(*bits, 1, true);
    assert(bits[1] && first(bits, 1));
    print("ok\n");
}
