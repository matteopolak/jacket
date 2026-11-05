#import "Basic";
#import "Hash_Table";
#import "Extensions/Toml";
#load "../key-storage.jai";

Key_Ledger :: struct {
    sizes: Table(*void, int);
    live: int;
    zero_allocations: int;
    zero_frees: int;
}

// Track requested payloads independently, including a real nonnull zero-byte allocation.
key_track :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    stats := cast(*Key_Ledger) data;
    backing := context.default_allocator;

    push_allocator(backing);
    previous := 0;

    if old_memory {
        stored := table_find_pointer(*stats.sizes, old_memory);
        assert(stored != null, "unknown allocation");
        previous = stored.*;
    }

    actual := ifx mode == .ALLOCATE && size == 0 then 1 else size;
    result := backing.proc(mode, actual, old_size, old_memory, backing.data);

    if mode == .FREE && old_memory {
        if previous == 0 stats.zero_frees += 1;
        stats.live -= previous;
        table_remove(*stats.sizes, old_memory);
    } else if (mode == .ALLOCATE || mode == .RESIZE) && result {
        if old_memory table_remove(*stats.sizes, old_memory);
        if mode == .ALLOCATE && size == 0 stats.zero_allocations += 1;
        table_set(*stats.sizes, result, size);
        stats.live += size - previous;
    }

    return result;
}

check_ledger :: (text: string, budget: int, expect_success: bool = false) {
    ledger: Key_Ledger;
    doc: Toml_Document;

    {
        push_allocator(key_track, *ledger);
        error: Toml_Error;
        doc, error = parse_toml(text, .{ max_retained_bytes = budget, record_statements = true });
        if expect_success assert(error.kind == .NONE, error.message);

        if error.kind == .NONE {
            assert(doc.retained_bytes == ledger.live && ledger.live <= budget);
            for doc.root.members assert(toml_get(doc.root, it.key) == it.value);
        }
    }

    free_toml(*doc);
    free_toml(*doc);
    assert(ledger.live == 0 && ledger.sizes.count == 0);
    assert(ledger.zero_allocations == ledger.zero_frees);
    deinit(*ledger.sizes);
}

main :: () {
    bytes: [8] u8;
    external: [8] u8;
    source := string.{ 8, bytes.data };

    assert(toml_key_uses_source(source, string.{ 1, bytes.data }));
    assert(toml_key_uses_source(source, string.{ 6, bytes.data + 1 }));
    assert(toml_key_uses_source(source, string.{ 1, bytes.data + 7 }));
    assert(toml_key_uses_source(source, source));
    assert(!toml_key_uses_source(source, string.{ 2, bytes.data + 7 }));
    assert(!toml_key_uses_source(source, string.{ 1, bytes.data + 8 }));
    assert(!toml_key_uses_source(source, string.{ 0, bytes.data }));
    assert(!toml_key_uses_source(source, string.{ 0, bytes.data + 8 }));
    assert(!toml_key_uses_source(source, string.{ 1, external.data }));
    assert(!toml_key_uses_source(string.{ 7, bytes.data + 1 }, string.{ 1, bytes.data }));
    assert(!toml_key_uses_source(source, string.{}));
    assert(!toml_key_uses_source(string.{}, source));

    // Decoded escapes own payloads; plain dotted/header/AOT segments use the retained source.
    mixed := "alpha.beta = { plain = 1, \"escaped\\u0000key\" = 2, \"\" = 3 }\n[alpha.child]\nx=1\n[[alpha.records]]\nx=2\n[[alpha.records]]\nx=3\n";

    check_ledger(mixed, 1 << 20, true);
    for cut: 0..mixed.count check_ledger(string.{ cut, mixed.data }, 1 << 20);
    for budget: 0..8192 check_ledger(mixed, budget);

    ledger: Key_Ledger;

    {
        push_allocator(key_track, *ledger);
        doc, error := parse_toml("\"\" = 1\n");

        assert(error.kind == .NONE && ledger.zero_allocations == 1);
        assert(doc.retained_bytes == ledger.live);
        free_toml(*doc);
    }

    assert(ledger.live == 0 && ledger.sizes.count == 0 && ledger.zero_frees == 1);
    deinit(*ledger.sizes);
}
