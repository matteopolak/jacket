// The start of an allocator call when Basic's `MEMORY_DEBUGGER` parameter is on: it starts the
// memory debugger unless the call is from the debugger itself, for temporary storage, or in
// compile-time code for another target. Expects a variable `allocator` in the calling scope, and
// returns from the caller in those cases.
Preamble :: () #expand {
    #if OS != BUILD_OS || CPU != BUILD_CPU `return;
    if context.inside_memory_debugger `return;
    if `allocator.proc == temporary_allocator_proc `return;

    caps := get_capabilities(`allocator);
    if (caps & .HINT_I_AM_PER_FRAME_TEMPORARY_STORAGE) != 0 `return;

    context.inside_memory_debugger = true;
    `defer context.inside_memory_debugger = false;
    previous_logger := context.logger;

    context.logger = runtime_support_default_logger;
    `defer context.logger = previous_logger;
    initialize_debugger();
}
