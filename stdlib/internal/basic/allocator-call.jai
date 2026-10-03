#scope_module
allocator_call :: inline (allocator: Allocator, mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void) -> *void {
    #if MEMORY_DEBUGGER {
        if mode == .FREE && old_memory Memory_Debugger.check_free(allocator, old_memory);
    }

    result := allocator.proc(mode, size, old_size, old_memory, allocator.data);

    #if MEMORY_DEBUGGER {
        if (mode == .ALLOCATE || mode == .RESIZE) && result {
            if mode == .RESIZE && old_memory Memory_Debugger.check_free(allocator, old_memory);
            Memory_Debugger.check_alloc(allocator, result, size);
        }
    }

    return result;
}
