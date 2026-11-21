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

// Ends the run with a status: a debugger or an emulator reads it (semihosting's exit call).
Exit_Hook :: Impl.Exit_Hook;

set_exit_hook :: (hook: Exit_Hook) #no_context {
    Impl.exit_hook = hook;
}

// Ends the run with `status` through the exit hook. With no hook set, or if it returns, the
// program halts.
exit :: (status: s32) #no_context {
    Impl.exit_program(status);
}

// The command line the board started the program with, the program's name first, as a hosted
// `get_command_line_arguments` gives it. A board that can pass one (QEMU's semihosting command
// line) sets it before `main`; otherwise it is empty.
set_arguments :: (arguments: [] string) #no_context {
    Impl.program_arguments = arguments;
}

arguments :: () -> [] string {
    return Impl.program_arguments;
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

// An allocator over the RAM between `.bss` and the stack (`__heap_start` to `__heap_end`, which
// `link_for` defines). It hands out memory in order and never reuses it, except that freeing or
// resizing the newest block gives the space back. Set it as the context's allocator:
//
// ```jai
// heap := make_heap();
// context_copy := static_context().*;
// context_copy.allocator = heap_allocator(*heap);
// push_context context_copy {
//     main();
// }
// ```
//
// Running out of memory calls the panic hook.
Heap :: struct {
    next: *u8;
    end: *u8;
    newest: *u8;
}

// A heap over the whole region the linker script reserves.
make_heap :: () -> Heap #no_context {
    return .{ cast(*u8) *__heap_start, cast(*u8) *__heap_end, null };
}

heap_allocator :: (heap: *Heap) -> Allocator #no_context {
    return .{ heap_allocator_proc, heap };
}

#scope_file

HEAP_ALIGNMENT :: 16;

heap_allocator_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    heap := cast(*Heap) allocator_data;

    if mode == {
        case .ALLOCATE;
            return heap_take(heap, size);
        case .RESIZE;
            if !old_memory return heap_take(heap, size);

            if old_memory == heap.newest {
                // The newest block grows or shrinks where it is.
                end := cast(*u8) old_memory + size;

                if end > heap.end Impl.panic("heap exhausted");

                heap.next = cast(*u8) ((cast(u64) end + HEAP_ALIGNMENT - 1) & ~cast(u64) (HEAP_ALIGNMENT - 1));

                return old_memory;
            }

            fresh := heap_take(heap, size);
            copy := ifx size < old_size then size else old_size;

            for index: 0..copy - 1 {
                (cast(*u8) fresh)[index] = (cast(*u8) old_memory)[index];
            }

            return fresh;
        case .FREE;
            if old_memory == heap.newest {
                heap.next = heap.newest;
                heap.newest = null;
            }

            return null;
        case .CAPS;
            if old_memory {
                (cast(*string) old_memory).* = "bump heap";
            }

            return null;
    }

    return null;
}

heap_take :: (heap: *Heap, size: s64) -> *void {
    block := heap.next;
    end := cast(*u8) ((cast(u64) block + cast(u64) size + HEAP_ALIGNMENT - 1) & ~cast(u64) (HEAP_ALIGNMENT - 1));

    if end > heap.end || end < block Impl.panic("heap exhausted");

    heap.newest = block;
    heap.next = end;

    return block;
}

__heap_start: u8 #elsewhere;
__heap_end: u8 #elsewhere;

__data_load: u8 #elsewhere;
__data_start: u8 #elsewhere;
__data_end: u8 #elsewhere;
__bss_start: u8 #elsewhere;
__bss_end: u8 #elsewhere;

context_storage: #Context;
temporary_storage: Basic.Temporary_Storage_Paged_Alloc;
temporary_bytes: [TEMPORARY_STORAGE_SIZE] u8;
