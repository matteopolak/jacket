// test: should_fail("atomic bit index out of range")
A :: #import "Atomics";

main :: () {
    value: u16;
    A.atomic_bit_test_and_set(*value, -17);
}
