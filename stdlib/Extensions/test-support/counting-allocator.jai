// Test-only allocator tracking, loaded by the ownership/leak scenarios (not a public module).
// Count successful allocations, including RESIZE from null; measure every successful RESIZE.

Tally :: struct {
    outstanding: s64;
    allocations: s64;
    largest: s64;
}

tally_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    tally := cast(*Tally) data;
    backing := context.default_allocator;
    result := backing.proc(mode, size, old_size, old_memory, backing.data);

    if (mode == .ALLOCATE || mode == .RESIZE) && result {
        if mode == .ALLOCATE || !old_memory {
            tally.outstanding += 1;
            tally.allocations += 1;
        }

        if size > tally.largest tally.largest = size;
    } else if mode == .FREE && old_memory {
        tally.outstanding -= 1;
    }

    return result;
}
