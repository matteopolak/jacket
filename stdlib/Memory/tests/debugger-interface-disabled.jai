#import "Basic"()(MEMORY_DEBUGGER = false, REPLACEMENT_INTERFACE = Hooks);

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
        assert(memory != null && allocator.proc != null);
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
    #assert !#exists(Memory_Debugger_Interface);
    plain := alloc(8);

    free(plain);
    assert(allocations == 0 && frees == 0 && creates == 0 && destroys == 0);
    print("ok\n");
}
