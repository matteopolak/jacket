Impl :: #import "internal/runtime-freestanding";
Basic :: #import "Basic";

// Receives everything `write_string` and `print` write, and the detail of failed checks. The
// second argument is true for error output. With no hook set, output is dropped.
Console_Hook :: Impl.Console_Hook;

// Called when the program cannot continue: a failed assert or check, or an allocation through
// the default allocator, which has no heap. It must not return; if it does, the program halts in
// a breakpoint loop. With no hook set, the reason is written to the console, then the program
// halts.
Panic_Hook :: Impl.Panic_Hook;

// Sends output to `hook` (a UART, semihosting, or nothing).
set_console_hook :: (hook: Console_Hook) #no_context {
    Impl.console_hook = hook;
}

set_panic_hook :: (hook: Panic_Hook) #no_context {
    Impl.panic_hook = hook;
}

// Starts the runtime: copies `.data` from its load address, zeroes `.bss`, and sets up the static
// `Context` (see `static_context`) with `TEMPORARY_STORAGE_SIZE` bytes of temporary storage. Call
// it first from the reset handler, before anything that uses a global.
//
// The linker script defines `__data_load`, `__data_start`, `__data_end`, `__bss_start` and
// `__bss_end`; `link_for` writes them. A program that runs from RAM puts `__data_load` equal to
// `__data_start`.
init_runtime :: () #c_call #no_context {
    data_start := cast(*u8) *__data_start;
    data_load := cast(*u8) *__data_load;
    data_bytes := cast(s64) (cast(u64) *__data_end - cast(u64) data_start);

    if data_load != data_start {
        for index: 0..data_bytes - 1 {
            data_start[index] = data_load[index];
        }
    }

    bss_start := cast(*u8) *__bss_start;
    bss_bytes := cast(s64) (cast(u64) *__bss_end - cast(u64) bss_start);

    for index: 0..bss_bytes - 1 {
        bss_start[index] = 0;
    }

    context_info := type_info(#Context);

    context_storage.context_info = context_info;

    storage := *temporary_storage;
    storage.* = .{};
    storage.size = TEMPORARY_STORAGE_SIZE;
    storage.data = temporary_bytes.data;
    storage.original_data = storage.data;
    storage.original_size = storage.size;
    storage.overflow_allocator = context_storage.allocator;
    context_storage.temporary_storage = *storage.base;
}

// The context `init_runtime` set up: `push_context static_context().* { ... }`. Copy it
// before `push_context` to give a board its own allocator or logger.
static_context :: () -> *#Context #no_context {
    return *context_storage;
}

#scope_file

__data_load: u8 #elsewhere;
__data_start: u8 #elsewhere;
__data_end: u8 #elsewhere;
__bss_start: u8 #elsewhere;
__bss_end: u8 #elsewhere;

context_storage: #Context;
temporary_storage: Basic.Temporary_Storage_Paged_Alloc;
temporary_bytes: [TEMPORARY_STORAGE_SIZE] u8;
