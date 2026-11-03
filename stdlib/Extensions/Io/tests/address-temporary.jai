// test: skip_on(.WASM, reason = "Extensions/Io needs fibers, unavailable on wasm")
// Formatting an address keeps its returned bytes in temporary storage, including IPv6.
#import "Basic";
Io :: #import "Extensions/Io";

Counter :: struct {
    backing: Allocator;
    allocations: s64;
}

count_allocations :: (mode: Allocator_Mode, size: s64, old_size: s64, memory: *void, data: *void) -> *void {
    counter := cast(*Counter) data;
    if mode == .ALLOCATE || mode == .RESIZE counter.allocations += 1;
    return counter.backing.proc(mode, size, old_size, memory, counter.backing.data);
}

main :: () {
    counter := Counter.{ backing = context.allocator };
    tracked := Allocator.{ proc = count_allocations, data = *counter };

    push_allocator(tracked);

    address, valid := Io.parse_address("[2001:db8::1]:443");

    assert(valid);
    before := counter.allocations;
    text := Io.address_to_string(address);

    assert(text == "[2001:db8::1]:443");
    assert(counter.allocations == before, "IPv6 output leaked a caller-owned allocation");
}
