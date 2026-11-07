#import "Basic";
#import "String";
#import "Extensions/Toml";
#load "../../test-support/counting-allocator.jai";

Key_Stats :: struct {
    allocations: int;
    key_copies: int;
}

count_keys :: (mode: Allocator_Mode, size: s64, old_size: s64, memory: *void, data: *void) -> *void {
    stats := cast(*Key_Stats) data;

    if mode == .ALLOCATE {
        stats.allocations += 1;
        if size == 59 stats.key_copies += 1;
    }

    backing := context.default_allocator;

    return backing.proc(mode, size, old_size, memory, backing.data);
}

count_parse :: (text: string) -> Key_Stats {
    stats: Key_Stats;

    {
        push_allocator(count_keys, *stats);
        doc, error := parse_toml(text);
        assert(error.kind == .NONE, error.message);
        free_toml(*doc);
    }

    return stats;
}

main :: () {
    key := "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZabcdefg";

    assert(key.count == 59);

    for text: string.[tprint("% = 1", key), tprint("\"%\" = 1", key)] {
        stats := count_parse(text);
        // Plain parsed keys refer to the document-owned source, without another payload copy.
        assert(stats.key_copies == 0, tprint("key copies: %", stats.key_copies));
    }

    transformed := tprint("\"\\u0061%\" = 1", slice(key, 1, key.count - 1));
    decoded := count_parse(transformed);

    assert(decoded.key_copies == 1, "escaped keys retain one decoded payload copy");

    small := "a=1\nb=2\nc=3\nd=4\ne=5\nf=6\ng=7\nh=8\n";
    large := tprint("%i=9\nj=10\nk=11\nl=12\n", small);
    a := count_parse(small);
    b := count_parse(large);

    // Four additional scalar nodes need four allocations. Source-backed keys and temporary
    // key vectors should reuse capacity rather than adding one allocation per pair.
    assert(b.allocations - a.allocations == 4, tprint("allocation delta: %", b.allocations - a.allocations));

    nested := "a.b = { c = { d = { e = { f = { g = { h = { i = { j = { k = { l = { m = { n = { o = { p = { q = { r = 1 } } } } } } } } } } } } } } } }\n[a.c]\nx=2\n[[a.records]]\ny=3\n[[a.records]]\ny=4\n";
    complete, complete_error := parse_toml(nested);

    assert(complete_error.kind == .NONE, complete_error.message);
    free_toml(*complete);

    for cut: 0..nested.count {
        tally: Tally;

        {
            push_allocator(tally_proc, *tally);
            doc := parse_toml(slice(nested, 0, cut), .{ record_statements = true });
            free_toml(*doc);
        }

        assert(tally.outstanding == 0, tprint("truncation leak at %", cut));
    }

    for budget: 0..8192 {
        tally: Tally;

        {
            push_allocator(tally_proc, *tally);
            doc, error := parse_toml(nested, .{ max_retained_bytes = budget });
            if error.kind == .NONE assert(doc.retained_bytes <= budget);
            free_toml(*doc);
        }

        assert(tally.outstanding == 0, tprint("budget leak at %", budget));
    }

    print("ok\n");
}
