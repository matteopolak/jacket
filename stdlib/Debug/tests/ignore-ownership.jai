#import "Basic";
D :: #import "Debug";

// Library-owned private unit: retain actual shared state without exporting it.
State :: #import "internal/debug-state";

T :: #import "Hash_Table";

Arena :: struct {
    bytes: [65536] u8;
    cursor: s64;
}

arena_allocator :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    arena := cast(*Arena) data;

    if mode == .FREE return null;
    if mode != .ALLOCATE && mode != .RESIZE return null;

    start := (cast(u64) arena.bytes.data + cast(u64) arena.cursor + 15) & ~15;
    next := cast(s64) (start - cast(u64) arena.bytes.data) + size;

    if size < 0 || next > arena.bytes.count return null;

    arena.cursor = next;
    result := cast(*void) start;
    if mode == .RESIZE && old_memory memcpy(result, old_memory, min(size, old_size));

    return result;
}

main :: () {
    arena: Arena;
    location := Source_Code_Location.{ "original-arena-ignore.jai", 8, 2 };

    push_context {
        context.allocator = .{ arena_allocator, *arena };
        T.table_set(*D.ignored_asserts, Source_Code_Location.{ "external-literal.jai", 1, 1 }, true);
        State.remember_ignored_assert(location);
    }

    assert(D.ignored_asserts.allocator.proc != arena_allocator);
    memset(arena.bytes.data, 0xcc, arena.bytes.count);
    arena.cursor = 0;
    assert(T.table_contains(*D.ignored_asserts, location));
    assert(T.table_contains(*D.ignored_asserts, Source_Code_Location.{ "external-literal.jai", 1, 1 }));
    D.set_report_mode(true);
    assert(!D.interactive_assert(location, "original arena-owned suppression"));
    D.set_report_mode(false);
    T.table_reset(*D.ignored_asserts);
    print("ok\n");
}
