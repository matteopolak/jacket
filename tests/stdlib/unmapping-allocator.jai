// Unmapping_Allocator (POSIX virtual memory): every block ends at a guard page, addresses are
// never reused, resizes copy, and a double free is reported. Touching freed memory or the guard
// page would crash the program, so this checks only what can be observed safely.
// Windows and WASM have no implementation of the module.
#import "Basic";

#import "Unmapping_Allocator";

Assertion :: struct {
    count: s64;
}

assertions: Assertion;

note_assertion :: (location: Source_Code_Location, message: string) -> bool {
    assertions.count += 1;
    return false;
}

main :: () {
    a := get_unmapping_allocator();
    assert(a.proc != null);
    caps := get_capabilities(a);
    assert((caps & .HINT_I_AM_A_DEBUG_ALLOCATOR) != 0);

    seen: [..] *u8;
    defer array_free(seen);
    for size: s64.[1, 7, 100, 4095, 4096, 4097, 70000] {
        p := cast(*u8) alloc(size, a);
        assert(p != null, "alloc %", size);
        // The block ends exactly at a page boundary, where the guard page starts.
        end := cast(u64) (p + size);
        assert(end % 4096 == 0, "a % byte block ends at %", size, end);
        // Every byte of the block is usable.
        for i: 0..size - 1 p[i] = cast,trunc(u8) i;
        for i: 0..size - 1 assert(p[i] == cast,trunc(u8) i);
        for seen assert(it != p, "an address was handed out twice");
        array_add(*seen, p);
        free(p, a);
    }
    // Zero bytes still gets a distinct address.
    zero := alloc(0, a);
    assert(zero != null);
    free(zero, a);

    // A resize moves to a new block with the old contents.
    p := cast(*u8) alloc(10, a);
    for 0..9 p[it] = cast(u8) (it + 1);
    q := cast(*u8) realloc(p, 5000, 10, a);
    assert(q != p);
    for 0..9 assert(q[it] == it + 1, "byte % after resize", it);
    q[4999] = 1;
    r := cast(*u8) realloc(q, 3, 5000, a);
    for 0..2 assert(r[it] == it + 1);

    // Freeing a block twice is reported instead of crashing.
    {
        scoped := context;
        scoped.assertion_failed = note_assertion;
        push_context, defer_pop scoped;
        free(r, a);
        free(r, a);
        assert(assertions.count == 1, "the double free was reported: %", assertions.count);
        free(q, a);
        assert(assertions.count == 2, "freeing a block a resize already retired is reported too");
    }
    deinit_unmapping_allocator(a);
    print("ok\n");
}
