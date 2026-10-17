A :: #import "Atomics";

main :: () {
    value: u8;
    A.atomic_bit_test_and_set(*value, 0);
}
