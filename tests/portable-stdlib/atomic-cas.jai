#import "Basic";
A :: #import "Atomics";

check :: ($T: Type) {
    value: T = 5;

    assert(A.atomic_add(*value, 7) == 5 && value == 12);
    A.atomic_increment(*value);
    assert(value == 13);
    value = 2;
    assert(!A.atomic_decrement(*value));
    assert(A.atomic_decrement(*value) && value == 0);
    A.atomic_add(*value, -1);
    assert(cast,no_check(u64) value == cast,no_check(u64) cast,no_check(T) -1);
    value = 0b1100;
    A.atomic_and(*value, 0b1010);
    assert(value == 0b1000);
    A.atomic_or(*value, 0b0011);
    A.atomic_xor(*value, 0b1111);
    assert(value == 4);

    #if size_of(T) > 1 {
        value = 0;
        index: s64 = -1;

        assert(!A.atomic_bit_test_and_set(*value, index));
        assert(A.atomic_bit_test_and_set(*value, -1));
        assert(!A.atomic_bit_test_and_set(*value, 0));
        assert(A.atomic_bit_test_and_set(*value, 0));
    }
}

main :: () {
    #assert !#exists(A.update_bits);
    check(u8);
    check(s8);
    check(u16);
    check(s16);
    check(u32);
    check(s32);
    check(u64);
    check(s64);

    Values :: enum_flags u16 {
        A :: 1;
        B :: 2;
    }

    flags := Values.A;

    A.atomic_or(*flags, .B);
    assert(flags == .A | .B);
    A.atomic_and(*flags, .B);
    assert(flags == .B);
    memory: [16] u32;
    pointer := memory.data;
    old := A.atomic_add(*pointer, 3);

    assert(old == memory.data && cast(*u8) pointer == cast(*u8) memory.data + 3);
    pointer = cast(*u32) 1;
    assert(A.atomic_decrement(*pointer) && pointer == null);
    assert(A.IsAtomicScalar(float32) && A.IsAtomicScalar(*u8));
    assert(!A.IsAtomicScalar(string));
    print("ok\n");
}
