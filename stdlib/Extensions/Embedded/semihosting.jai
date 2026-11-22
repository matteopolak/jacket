// Semihosting: the program asks the debugger or emulator that runs it for a console, its command
// line and an exit status, with a trap instruction (ARM: `hlt #0xf000` on AArch64, `bkpt 0xab` on
// Cortex-M). A board that has no other console provides the trap as the symbol `semihost` (see
// `Qemu_Virt_Aarch64`, `Mps2_An386`) and starts its program through `start_semihosted`.

Program :: #type ();

// Runs `program` as the machine starts it: sets up the runtime, sends output and panics to the
// semihosting console, runs the program, and exits with status 0. Call it from the exported reset
// function once the stack, and floating point if the core needs it, are ready.
start_semihosted :: (program: Program) #c_call #no_context {
    init_runtime();
    set_console_hook(console_write);
    set_panic_hook(panic);
    set_exit_hook(exit_semihosted);
    set_arguments(command_line());

    heap := make_heap();
    context_copy := static_context().*;
    context_copy.allocator = heap_allocator(*heap);

    push_context context_copy {
        program();
    }

    exit_semihosted(0);
}

// Ends the run with `status`.
exit_semihosted :: (status: s32) #no_context {
    // SYS_EXIT_EXTENDED: reason ADP_Stopped_ApplicationExit and the status.
    block: [2] Word;
    block[0] = 0x20026;
    block[1] = cast,no_check(Word) status;
    semihost(0x20, *block);
}

#scope_file

// The size of a pointer: a semihosting parameter block is made of words.
#if size_of(*void) == 4 {
    Word :: u32;
} else {
    Word :: u64;
}

semihost :: (operation: Word, block: *void) -> Word #elsewhere #c_call;

// What the machine was told to start the program with (`-append`, or `-semihosting-config arg=`):
// the semihosting command-line call, split at spaces. QEMU puts the kernel's file name first, as
// a hosted program's name is. A line longer than the buffer is cut.
command_line :: () -> [] string #no_context {
    words_used = 0;

    block: [2] Word;
    block[0] = cast,no_check(Word) line.data;
    block[1] = cast,no_check(Word) line.count;

    if semihost(0x15, *block) != 0 return .{ words_used, words_storage.data }; // SYS_GET_CMDLINE

    length := cast(s64) block[1];
    start := 0;

    for 0..length {
        at_end := it == length || line[it] == 0 || line[it] == #char " ";

        if !at_end continue;

        if it > start && words_used < words_storage.count {
            words_storage[words_used] = .{ it - start, line.data + start };
            words_used += 1;
        }

        start = it + 1;
        if it == length || line[it] == 0 break;
    }

    return .{ words_used, words_storage.data };
}

line: [1024] u8;
words_storage: [64] string;
words_used: s64;

// Standard output and standard error are two files of the host, opened as ARM semihosting
// defines the console (`:tt`: opened for writing it is standard output, for appending standard
// error), so what a program prints keeps its stream.
console_write :: (text: string, to_standard_error: bool) #no_context {
    if !console_opened open_console();

    handle := ifx to_standard_error then error_handle else output_handle;
    block: [3] Word;
    block[0] = cast,no_check(Word) handle;
    block[1] = cast,no_check(Word) text.data;
    block[2] = cast,no_check(Word) text.count;

    semihost(0x05, *block); // SYS_WRITE
}

open_console :: () #no_context {
    console_opened = true;
    output_handle = open_console_file(4);
    error_handle = open_console_file(8);
}

open_console_file :: (mode: Word) -> s64 #no_context {
    name := ":tt";
    block: [3] Word;
    block[0] = cast,no_check(Word) name.data;
    block[1] = mode;
    block[2] = cast,no_check(Word) name.count;

    return cast,no_check(s64) semihost(0x01, *block); // SYS_OPEN
}

console_opened: bool;
output_handle: s64;
error_handle: s64;

panic :: (message: string) #no_context {
    console_write("panic: ", true);
    console_write(message, true);
    console_write("\n", true);
    exit_semihosted(1);
}
