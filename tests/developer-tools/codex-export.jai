#import "Basic";
#import "File";
C :: #import "Codex";
R :: #import "Relative_Pointers";

main :: () {
    memory, read_ok := read_entire_file(tprint("%../../target/profiler-codex-export.codex", #filepath));

    assert(read_ok);
    defer free(memory);
    ok, help, error := C.make_help(.{ memory.count, memory.data });

    assert(ok, "%", error);
    defer C.deinit(*help);
    assert(help.header.magic == 0x05040003 && help.header.version == 2);
    assert(help.modules.count > 0 && help.files.count > 0 && help.procedures.count > 0);
    found_conditional := false;
    found_allocator := false;

    for record: help.procedures {
        name := R.decode(*record.name);
        if name == "conditional" {
            assert(record.metrics.if_points == 10 && record.metrics.maximum_if_depth == 2);
            assert(record.num_expressions > 0 && record.l0 > 0 && record.l1 >= record.l0);
            found_conditional = true;
        }

        if name == "allocation" {
            assert(record.metrics.allocations == 3 && record.metrics.frees == 3, "alloc %, free %", record.metrics.allocations, record.metrics.frees);
            found_allocator = true;
        }
    }

    assert(found_conditional && found_allocator);

    // Every truncation fails safely; no unchecked source boundary is reproduced.
    for count: 0..memory.count - 1 {
        truncated: [] u8 = .{ count, memory.data };
        parsed, _, failure := C.make_help(truncated);
        assert(!parsed && failure.count > 0);
    }

    print("ok\n");
}
