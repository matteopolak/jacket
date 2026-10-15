B :: #import "Basic";
Off :: #import "Basic"(ENABLE_ASSERT = false);

backing: Allocator;
attempts: int;
fail_at: int;
live: int;

fail_copy :: (mode: Allocator_Mode, size: s64, old_size: s64, memory: *void, data: *void) -> *void {
    if mode == .ALLOCATE {
        attempts += 1;
        if attempts == fail_at return null;
    }

    result := backing.proc(mode, size, old_size, memory, backing.data);
    if mode == .ALLOCATE && result live += 1;
    if mode == .FREE && memory live -= 1;

    return result;
}

Record :: struct {
    value: int;
}

large_output :: (builder: *B.String_Builder, _item: Any, _data: *void) -> bool {
    bytes: [5000] u8;
    for *bytes it.* = #char "x";
    B.append(builder, bytes.data, bytes.count);
    return true;
}

main :: () {
    backing = context.allocator;
    B.push_allocator(fail_copy, null);
    context.print_style.struct_printer = large_output;

    for failure: 1..2 {
        attempts = 0;
        fail_at = failure;
        builder: Off.String_Builder;

        Off.print_item_to_builder(*builder, Record.{ 7 });
        B.assert(attempts == fail_at);
        B.assert(builder.failed, "callback or output-copy allocation failure must survive the ABI bridge");
        Off.free_buffers(*builder);
        B.assert(live == 0);
    }
}
