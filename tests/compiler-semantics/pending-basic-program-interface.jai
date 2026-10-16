B :: #import "Basic"()(MEMORY_DEBUGGER = true, REPLACEMENT_INTERFACE = Hooks, VISUALIZE_MEMORY_DEBUGGER = false);

allocations: s64;
frees: s64;
created: s64;
destroyed: s64;
last_allocator: Allocator;
last_memory: *void;
last_size: s64;

Hooks :: struct {
    check_alloc :: inline (allocator: Allocator, memory: *void, size: s64) {
        allocations += 1;
        last_allocator = allocator;
        last_memory = memory;
        last_size = size;
    }
    check_free :: inline (allocator: Allocator, memory: *void) {
        frees += 1;
        last_allocator = allocator;
        last_memory = memory;
    }
    check_create_heap :: inline (a: Allocator) {
        created += 1;
        last_allocator = a;
    }
    check_destroy_heap :: inline (a: Allocator) {
        destroyed += 1;
        last_allocator = a;
    }
}

main :: () {
    values: [..] int;

    B.array_reserve(*values, 4);
    B.assert(allocations == 1 && frees == 0 && last_memory == values.data && last_size == 4 * size_of(int));
    B.assert(last_allocator.proc == values.allocator.proc && last_allocator.data == values.allocator.data);
    B.array_add(*values, 1);
    B.array_reserve(*values, 9);
    B.assert(allocations == 2 && frees == 1 && last_size == 9 * size_of(int) && values[0] == 1);
    B.array_reset(*values);
    B.assert(allocations == 2 && frees == 2 && values.data == null);
    raw := B.alloc(13);

    B.assert(raw != null && allocations == 3 && frees == 2 && last_memory == raw && last_size == 13);
    B.free(raw);
    B.assert(allocations == 3 && frees == 3 && created == 0 && destroyed == 0);
    B.assert(!#exists(B.allocator_call) && !#exists(B.copy_array_elements));
}
