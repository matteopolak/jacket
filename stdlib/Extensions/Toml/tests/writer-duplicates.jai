#import "Basic";
#import "Extensions/Toml";

// Hand-built tables have no trusted lookup index; the writer must still refuse repeated keys,
// at the repeat, for every size of table (inline set and heap set alike).
build :: (count: int, repeat_from: int, repeat_of: int) -> Toml_Value, [] Toml_Value {
    root: Toml_Value;
    root.kind = .TABLE;
    values := NewArray(count, Toml_Value);

    for *values {
        it.kind = .INTEGER;
        it.integer = it_index;
        key := tprint("k%", (it_index * 7919) % 100003);
        if repeat_from >= 0 && it_index == repeat_from key = tprint("k%", (repeat_of * 7919) % 100003);
        array_add(*root.members, Toml_Member.{ key = key, value = it });
    }

    return root, values;
}

main :: () {
    for count: int.[1, 2, 16, 17, 100, 127, 128, 129, 1000, 20000] {
        root, values := build(count, -1, -1);
        text, error := write_toml(*root, .{ max_nodes = 1 << 24 });

        assert(error.kind == .NONE, tprint("% unique keys: %", count, error.message));
        doc, read := parse_toml(text, .{ max_nodes = 1 << 24 });
        assert(read.kind == .NONE && doc.root.members.count == count);
        free_toml(*doc);
        free(text);
        array_free(root.members);
        free(values.data);

        if count < 2 continue;

        for repeat: int.[count - 1, count / 2, 1] {
            bad, bad_values := build(count, repeat, 0);
            _, duplicate := write_toml(*bad, .{ max_nodes = 1 << 24 });

            assert(duplicate.kind == .OUTPUT && duplicate.message == "duplicate constructed table key", tprint("% duplicate at %: %", count, repeat, duplicate.message));
            assert(toml_error_path(*duplicate) == tprint("$[\"k%\"]", 0), toml_error_path(*duplicate));
            array_free(bad.members);
            free(bad_values.data);
        }
    }

    // A node quota smaller than the table still reports the limit, not a duplicate.
    wide, wide_values := build(500, -1, -1);
    _, limited := write_toml(*wide, .{ max_nodes = 100 });

    assert(limited.kind == .LIMIT);
    print("ok\n");
}
