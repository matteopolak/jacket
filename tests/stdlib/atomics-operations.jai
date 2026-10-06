// Every Atomics procedure on one thread, over each scalar width and kind it accepts: the value it
// leaves behind and the value it returns. atomics-contention.jai runs the same operations from
// several threads at once.
#import "Basic";
#import "Atomics";

Flags :: enum_flags u16 {
    A;
    B;
    C;
    D;
}

check_integer :: ($T: Type) {
    value: T = 5;
    assert(atomic_read(*value) == 5, "%: read", T);
    atomic_write(*value, 9);
    assert(value == 9, "%: write left %", T, value);
    old := atomic_swap(*value, 3);
    assert(old == 9 && value == 3, "%: swap gave % and left %", T, old, value);
    old = atomic_add(*value, 4);
    assert(old == 3 && value == 7, "%: add gave % and left %", T, old, value);
    atomic_increment(*value);
    assert(value == 8, "%: increment left %", T, value);
    value = 2;
    assert(!atomic_decrement(*value), "%: decrement from 2 does not reach zero", T);
    assert(atomic_decrement(*value), "%: decrement from 1 reaches zero", T);
    assert(value == 0, "%: decrement left %", T, value);
    value = 0b1100;
    atomic_and(*value, 0b1010);
    assert(value == 0b1000, "%: and left %", T, value);
    atomic_or(*value, 0b0011);
    assert(value == 0b1011, "%: or left %", T, value);
    atomic_xor(*value, 0b1111);
    assert(value == 0b0100, "%: xor left %", T, value);
    atomic_xor(*value, 0b0100);
    assert(value == 0, "%: xor with itself left %", T, value);
    // The top bit of the type.
    TOP :: size_of(T) * 8 - 1;
    assert(!atomic_bit_test_and_set(*value, 0), "%: bit 0 was clear", T);
    assert(atomic_bit_test_and_set(*value, 0), "%: bit 0 now set", T);
    assert(!atomic_bit_test_and_set(*value, TOP), "%: top bit was clear", T);
    assert(atomic_bit_test_and_set(*value, TOP), "%: top bit now set", T);
    assert(value == cast,no_check(T) (1 | ((cast(u64) 1) << TOP)), "%: bits left %", T, value);
    // Wrap-around: adding past the largest value of an unsigned type.
    value = 0;
    atomic_add(*value, -1);
    low, high := get_integer_range(T);
    assert(value == cast,no_check(T) -1, "%: 0 - 1 left %", T, value);
    value = high;
    assert(atomic_read(*value) == high);
    value = low;
    assert(atomic_read(*value) == low);
    assert(IsAtomicScalar(T));
}

main :: () {
    check_integer(u8);
    check_integer(s8);
    check_integer(u16);
    check_integer(s16);
    check_integer(u32);
    check_integer(s32);
    check_integer(u64);
    check_integer(s64);

    // Values that are not plain integers.
    {
        flag := false;
        assert(!atomic_read(*flag));
        atomic_write(*flag, true);
        assert(flag);
        assert(atomic_swap(*flag, false) && !flag);

        flags := Flags.A;
        atomic_or(*flags, .C);
        assert(flags == .A | .C, "flags %", flags);
        atomic_and(*flags, .C | .D);
        assert(flags == .C, "flags %", flags);
        assert(atomic_swap(*flags, .D) == .C && flags == .D);

        // Floats are in atomics-float.jai.
        target: s64;
        other: s64;
        pointer: *s64 = *target;
        assert(atomic_swap(*pointer, *other) == *target && pointer == *other);
        assert(atomic_read(*pointer) == *other);

        // A carry across the 32-bit halves, so a 32-bit operation would be caught.
        big: u64 = 0x1_ffff_ffff;
        assert(atomic_add(*big, 1) == 0x1_ffff_ffff);
        assert(big == 0x2_0000_0000, "big %", formatInt(big, base = 16));
    }

    // Which types count as atomic scalars.
    {
        assert(IsAtomicScalar(bool));
        assert(IsAtomicScalar(Flags));
        assert(IsAtomicScalar(float32) && IsAtomicScalar(float64));
        assert(IsAtomicScalar(*u8));
        assert(IsAtomicScalar(type_of(main)));
        Twelve :: struct {
            a, b, c: s32;
        }
        assert(!IsAtomicScalar(Twelve));
        Small :: struct {
            a: s32;
        }
        assert(!IsAtomicScalar(Small), "a struct is not a scalar even when it is 4 bytes");
        assert(!IsAtomicScalar(string));
        assert(!IsAtomicScalar([4] u8));
        Distinct :: #type,distinct u32;
        assert(IsAtomicScalar(Distinct));
    }

    // compare_and_swap2 exchanges 16 bytes; only the Linux x64 build has a provider for it.
    #if OS == .LINUX && CPU == .X64 {
        Pair :: struct {
            first: u64;
            second: u64;
        } #align 16;
        pair := Pair.{ 1, 2 };
        assert(!compare_and_swap2(*pair, cast(u64) 1, cast(u64) 3, cast(u64) 10, cast(u64) 20), "a mismatch in the second half");
        assert(pair.first == 1 && pair.second == 2);
        assert(compare_and_swap2(*pair, cast(u64) 1, cast(u64) 2, cast(u64) 10, cast(u64) 20));
        assert(pair.first == 10 && pair.second == 20, "pair %", pair);
    }
    print("ok\n");
}
