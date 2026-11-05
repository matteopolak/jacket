// Parse, traverse, look up and destroy one file, measuring requested allocation payload only.
#import "Basic";
#import "Hash_Table";
F :: #import "File";
T :: #import "Extensions/Toml";

Stats :: struct {
    sizes: Table(*void, int);
    live: int;
    peak: int;
    allocations: int;
    resizes: int;
}

track :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    stats := cast(*Stats) data;
    backing := context.default_allocator;

    push_allocator(backing);
    previous := 0;

    if old_memory {
        stored := table_find_pointer(*stats.sizes, old_memory);
        if stored previous = stored.*;
    }

    result := backing.proc(mode, size, old_size, old_memory, backing.data);

    if mode == .FREE && old_memory {
        assert(table_find_pointer(*stats.sizes, old_memory) != null, "untracked free");
        stats.live -= previous;
        table_remove(*stats.sizes, old_memory);
    } else if (mode == .ALLOCATE || mode == .RESIZE) && result {
        if old_memory {
            table_remove(*stats.sizes, old_memory);
            stats.resizes += 1;
        } else stats.allocations += 1;

        table_set(*stats.sizes, result, size);
        stats.live += size - previous;
        stats.peak = max(stats.peak, stats.live);
    }

    return result;
}

checksum :: (node: *T.Toml_Value) -> int {
    result := 1 + cast(int) node.kind;

    if node.kind == .TABLE {
        for node.members {
            for byte: it.key result += byte;
            result += checksum(it.value);
        }
    } else if node.kind == .ARRAY for node.items result += checksum(it);
    else if node.kind == .STRING for node.text result += it;
    else if node.kind == .INTEGER result += cast(int) ((cast,force(u64) node.integer) & 0xffff);
    else if node.kind == .FLOAT result += cast(int) ((cast,force(u64) node.real) & 0xffff);
    else if node.kind == .BOOL result += ifx node.boolean then 1 else 0;
    else {
        t := node.temporal;
        result += t.year + t.month + t.day + t.hour + t.minute + t.second + t.offset_minutes;
        for t.fraction result += it;
    }

    return result;
}

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 2);
    input, read_ok := F.read_entire_file(args[1]);

    assert(read_ok);
    defer free(input);
    stats: Stats;
    value := 0;
    retained := 0;
    nodes := 0;

    {
        push_allocator(track, *stats);
        options: T.Toml_Options;
        options.max_retained_bytes = 1 << 30;
        doc, error := T.parse_toml(input, options);

        assert(error.kind == .NONE, error.message);
        assert(doc.retained_bytes == stats.live, "retained budget equals actual document payload");
        retained = stats.live;
        nodes = doc.nodes.count;
        value = checksum(doc.root);
        // Equivalent selected lookup task, in addition to the complete traversal above.
        for doc.root.members assert(T.toml_get(doc.root, it.key) == it.value);

        T.free_toml(*doc);
    }

    assert(stats.live == 0 && stats.sizes.count == 0, "document allocation leak");
    print("{\"input_bytes\":%,\"nodes\":%,\"value_bytes\":%,\"allocations\":%,\"resizes\":%,\"retained_payload\":%,\"peak_payload\":%,\"checksum\":%}\n", input.count, nodes, size_of(T.Toml_Value), stats.allocations, stats.resizes, retained, stats.peak, value);
    deinit(*stats.sizes);
}
