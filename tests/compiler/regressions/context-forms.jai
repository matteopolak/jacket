// The context: reading and writing fields, push_context, allocators, logging, extra fields
// and print style.
#import "Basic";

#add_context depth: int = 33;

Counting :: struct {
    allocations: int;
    last_size: s64;
}

counting := Counting.{};

counting_allocator :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory_pointer: *void, proc_data: *void) -> *void {
    if mode == .ALLOCATE || mode == .RESIZE {
        counting.allocations += 1;
        counting.last_size = size;
    }

    return context.default_allocator.proc(mode, size, old_size, old_memory_pointer, proc_data);
}

bump :: () {
    context.depth += 1;
}

bump_isolated :: () {
    ctx := context;
    ctx.depth += 100;
    push_context ctx {
        assert(context.depth >= 100);
        bump();
    }
}

read_depth :: () -> int {
    return context.depth;
}

logged: [..] string;

collect_logger :: (message: string, data: *void, info: Log_Info) {
    array_add(*logged, copy_string(message));
}

main :: () {
    // A field added with #add_context starts at its initial value.
    assert(context.depth == 33);
    context.depth = 99;
    assert(read_depth() == 99);
    context.depth = 33;

    // Changes made by a callee are visible to the caller afterwards.
    bump();
    assert(context.depth == 34);

    // push_context restores the previous context at the end of the block.
    bump_isolated();
    assert(context.depth == 34);

    ctx := context;
    ctx.depth = 7;
    push_context ctx {
        assert(read_depth() == 7);
        bump();
        assert(read_depth() == 8);
    }

    assert(context.depth == 34);

    // A custom allocator sees allocations made while it is installed.
    counting = .{};
    replaced := context;
    replaced.allocator.proc = counting_allocator;
    replaced.allocator.data = null;
    push_context replaced {
        memory := alloc(42);
        assert(counting.allocations == 1 && counting.last_size == 42);
        free(memory);
    }

    outside := alloc(42);

    free(outside);
    assert(counting.allocations == 1);

    // push_allocator changes only the allocator for the scope.
    counting = .{};

    {
        push_allocator(.{ counting_allocator, null });
        memory := alloc(8);
        assert(counting.allocations == 1);
        free(memory);
    }

    outside = alloc(8);
    free(outside);
    assert(counting.allocations == 1);

    // Containers remember the allocator they started with.
    counting = .{};
    kept: [..] int;

    push_context replaced {
        array_add(*kept, 1);
    }

    allocations_after_first := counting.allocations;

    assert(allocations_after_first >= 1);
    for 1..100 array_add(*kept, it);

    assert(counting.allocations > allocations_after_first);
    array_free(kept);

    // Logging goes through context.logger.
    logging := context;
    logging.logger = collect_logger;
    logging.logger_data = null;
    push_context logging {
        log("routed message");
    }

    assert(logged.count >= 1);
    found := false;
    for logged if contains_text(it, "routed message") found = true;

    assert(found);

    // Print style: integer base.
    numbers := u32.[1, 69105, 1491625];
    styled := context;

    push_context styled {
        format_int := *context.print_style.default_format_int;
        format_int.base = 16;
        assert(tprint("%", numbers[1]) == "10df1");
        format_int.base = 2;
        assert(tprint("%", numbers[0]) == "1");
        assert(tprint("%", numbers[2]) == "101101100001010101001");
    }

    assert(tprint("%", numbers[1]) == "69105");

    // Temporary storage is per context and can be reset.
    reset_temporary_storage();
    text := tprint("temporary %", 5);

    assert(text == "temporary 5");
    reset_temporary_storage();

    // The thread index of the main thread is stable.
    assert(context.thread_index == context.thread_index);

    print("ok\n");
}

contains_text :: (haystack: string, needle: string) -> bool {
    if needle.count > haystack.count return false;

    for 0..haystack.count - needle.count {
        if slice(haystack, it, needle.count) == needle return true;
    }

    return false;
}

slice :: (s: string, start: s64, count: s64) -> string {
    result: string;
    result.data = s.data + start;
    result.count = count;
    return result;
}
