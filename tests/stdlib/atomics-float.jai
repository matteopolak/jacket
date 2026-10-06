// Atomics on float32 and float64: read, write and swap go through compare_and_swap on the float.
#import "Basic";
#import "Atomics";

main :: () {
    f: float64 = 1.5;
    assert(atomic_read(*f) == 1.5);
    assert(atomic_swap(*f, -2.25) == 1.5 && f == -2.25);
    atomic_write(*f, 1e300);
    assert(f == 1e300);

    f32: float32 = 0.5;
    atomic_write(*f32, 4);
    assert(f32 == 4);
    assert(atomic_swap(*f32, 0.25) == 4 && f32 == 0.25);
    assert(atomic_read(*f32) == 0.25);

    // A negative zero differs from 0.0 bit for bit; atomic_read must still return it.
    nz: float64 = -0.0;
    read := atomic_read(*nz);
    assert(read == 0);
    bits := (cast(*u64) *read).*;
    assert(bits == 0x8000_0000_0000_0000, "read -0.0 as %", formatInt(bits, base = 16));

    ok, observed := compare_and_swap(*f32, 0.25, 8);
    assert(ok && observed == 0.25 && f32 == 8);
    ok, observed = compare_and_swap(*f32, 1, 2);
    assert(!ok && observed == 8 && f32 == 8);
    print("ok\n");
}
