#import "Basic";
A :: #import "Atomics";

main :: () {
    value: u32;

    atomic_store(*value, 9);
    assert(atomic_load(*value) == 9);
    assert(atomic_exchange(*value, 4) == 9 && value == 4);
    A.atomic_write(*value, 11);
    assert(A.atomic_read(*value) == 11);
    assert(A.atomic_swap(*value, 7) == 11 && value == 7);
    print("ok\n");
}
