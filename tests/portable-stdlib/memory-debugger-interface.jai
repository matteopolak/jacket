#import "Basic";
B :: #import "Basic"(MEMORY_DEBUGGER = true, REPLACEMENT_INTERFACE = Hooks, VISUALIZE_MEMORY_DEBUGGER = false);
Off :: #import "Basic"(MEMORY_DEBUGGER = false, REPLACEMENT_INTERFACE = Hooks);

allocations: int;
frees: int;
creates: int;
destroys: int;
last_size: s64;

Hooks :: struct {
    check_alloc :: inline (allocator: Allocator, memory: *void, size: s64) {
        assert(memory != null && allocator.proc != null);
        allocations += 1;
        last_size = size;
    }
    check_free :: inline (allocator: Allocator, memory: *void) {
        assert(memory != null);
        frees += 1;
    }
    check_create_heap :: inline (a: Allocator) {
        assert(a.proc != null);
        creates += 1;
    }
    check_destroy_heap :: inline (a: Allocator) {
        assert(a.proc != null);
        destroys += 1;
    }
}

main :: () {
    // The contextual constraint is private; the selected public alias retains
    // the caller's canonical type identity.
    #assert !#exists(B.Memory_Debugger_Interface);
    #assert !#exists(Off.Memory_Debugger_Interface);
    assert(type_info(B.Memory_Debugger) == type_info(Hooks));
    memory := B.alloc(16);

    assert(allocations == 1 && last_size == 16);
    memory = B.realloc(memory, 32, 16);
    assert(allocations == 2 && frees == 1 && last_size == 32);
    B.free(memory);
    assert(frees == 2);
    plain := Off.alloc(8);

    Off.free(plain);
    assert(allocations == 2 && frees == 2);
    numbers: [..] int;

    B.array_add(*numbers, 7);
    assert(allocations == 3 && numbers[0] == 7);
    B.array_free(numbers);
    assert(frees == 3);
    builder: B.String_Builder;
    bytes: [5000] u8;

    B.append(*builder, bytes.data, bytes.count);
    assert(allocations == 4);
    B.free_buffers(*builder);
    assert(frees == 4);
    underlying := context.allocator;

    push_context {
        context.allocator = .{ deny_resize, *underlying };
        original := cast(*u8) B.alloc(10);
        original[0] = 91;
        changed := B.realloc(original, 20, 10);

        assert(changed == null && original[0] == 91);
        assert(allocations == 5 && frees == 4);
        B.free(original);
        assert(frees == 5);
    }

    heap := B.create_heap(.{ heap_allocator, null });

    assert(creates == 1 && heap.data != null);

    push_context {
        context.allocator = heap;
        block := B.alloc(12);

        B.free(block);
        retained := B.alloc(8);
        (cast(*u8) retained).* = 42;
    }

    B.destroy_heap(heap);
    assert(destroys == 1 && heap_destroyed && heap_blocks_freed == 2);
    print("ok\n");
}

deny_resize :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    if mode == .RESIZE return null;
    allocator := (cast(*Allocator) allocator_data).*;
    return allocator.proc(mode, size, old_size, old_memory, allocator.data);
}

#load "support/test-heap.jai";
