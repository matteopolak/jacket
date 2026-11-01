// A compiled regex gives everything it allocated back to `free_regex`, and a pattern that does not compile
// leaves nothing behind.
#import "Basic";
Regex :: #import "Extensions/Regex";

Tracking :: struct {
    base: Allocator;
    outstanding: s64;
}

tracking_allocator :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    state := cast(*Tracking) allocator_data;
    result := state.base.proc(mode, requested_size, old_size, old_memory, state.base.data);

    if mode == .ALLOCATE && result state.outstanding += 1;
    if mode == .RESIZE && !old_memory && result state.outstanding += 1;
    if mode == .FREE && old_memory state.outstanding -= 1;

    return result;
}

PATTERNS :: string.["a+b", "(?<year>[0-9]{4})-(?<month>[0-9]{2})", "\\p{Greek}+|[^x]*y", "(?i)hello\\b", "(a|ab)(c|bcd)"];

main :: () {
    tracking := Tracking.{ base = context.allocator };
    scoped := context;

    scoped.allocator = .{ tracking_allocator, *tracking };

    push_context scoped {
        for PATTERNS {
            re, ok := Regex.compile(it);

            assert(ok);
            assert(Regex.is_match(*re, "xaab 2026-10 hello ac y"));

            matches := Regex.find_all(*re, "2026-10 hello aab");

            array_free(matches);
            Regex.free_regex(*re);
        }

        _, ok := Regex.compile("(unclosed");

        assert(!ok);
    }

    assert(tracking.outstanding == 0);
    print("ok\n");
}
