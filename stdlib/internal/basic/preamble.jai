Preamble :: () #expand {
    #if OS != BUILD_OS || CPU != BUILD_CPU `return;
    if context.inside_memory_debugger `return;
    if `allocator.proc == Preamble_Pool.pool_allocator_proc || `allocator.proc == temporary_allocator_proc `return;

    caps, allocator_name := get_capabilities(`allocator);
    if (caps & .HINT_I_AM_PER_FRAME_TEMPORARY_STORAGE) != 0 || allocator_name == "modules/Flat_Pool" `return;

    context.inside_memory_debugger = true;
    `defer context.inside_memory_debugger = false;
    previous_logger := context.logger;

    context.logger = runtime_support_default_logger;
    `defer context.logger = previous_logger;
    initialize_debugger();
}

#scope_file
Preamble_Pool :: #import "Pool";
