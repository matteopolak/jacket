Memory_Debugger_Interface :: struct {
    check_alloc :: inline (allocator: Allocator, memory: *void, size: s64) {}
    check_free :: inline (allocator: Allocator, memory: *void) {}
    check_create_heap :: inline (a: Allocator) {}
    check_destroy_heap :: inline (a: Allocator) {}
}
