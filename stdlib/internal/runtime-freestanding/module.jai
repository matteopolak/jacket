// Runtime_Support's implementation for a program with no operating system (`OS == .NONE`).
//
// There is no libc, no heap and no threads. Output and failure go through two hooks that the
// program or its board sets; `Extensions/Embedded` is how a program reaches them. With no console
// hook, output is dropped. With no panic hook, a failure writes its reason to the console and
// halts.

#load "../runtime/faults.jai";
#load "../runtime/shared.jai";
#load "../runtime/checks.jai";

#assert OS == .NONE "internal/runtime-freestanding is for freestanding targets";

// Receives everything `write_string`, `print` and the failure reports write. The second argument
// is true for error output.
Console_Hook :: #type (text: string, to_standard_error: bool) #no_context;

// Called when the program cannot continue (a failed assert or check, or an allocation with no
// heap). It must not return; if it does, the program halts.
Panic_Hook :: #type (message: string) #no_context;

// Ends the run with the status. It must not return; if it does, the program halts.
Exit_Hook :: #type (status: s32) #no_context;

console_hook: Console_Hook;
panic_hook: Panic_Hook;
exit_hook: Exit_Hook;

// What the board says the program was started with (`Extensions/Embedded.set_arguments`).
program_arguments: [] string;

halt :: () #no_context {
    while true trap();
}

// Ends the run with `status` through the exit hook, then halts.
exit_program :: (status: s32) #no_context {
    if exit_hook exit_hook(status);
    halt();
}

// Stops the program: the panic hook, then a breakpoint loop.
panic :: (message: string) #no_context {
    if panic_hook {
        panic_hook(message);
    } else {
        write_string("panic: ", true);
        write_string(message, true);
        write_string("\n", true);
    }

    halt();
}

// Passes `s` to the console hook, or drops it when there is none.
write_string_unsynchronized :: (s: string, to_standard_error := false) #no_context {
    if console_hook console_hook(s, to_standard_error);
}

// Passes `s` to the console hook, or drops it when there is none. There are no threads, so
// there is no lock to take.
write_string :: (s: string, to_standard_error := false) #no_context #compiler {
    write_string_unsynchronized(s, to_standard_error);
}

// Writes each string as [write_string] does.
write_strings :: (strings: ..string, to_standard_error := false) #no_context #compiler {
    for s: strings write_string_unsynchronized(s, to_standard_error);
}

// The default `context.assertion_failed`: writes `file:line: assertion failed: message` as error
// output, then calls [panic].
runtime_support_assertion_failed :: (loc: Source_Code_Location, message: string) -> bool {
    write_string(loc.fully_pathed_filename, true);
    write_string(":", true);
    write_number(loc.line_number, 10, true);
    write_strings(": assertion failed: ", message, "\n", to_standard_error = true);
    panic("assertion failed");

    return false;
} @Cold

// Executes a trap instruction.
debug_break :: () #no_context {
    trap();
}

// [debug_break], callable at compile time.
compile_time_debug_break :: () #compiler #no_context {
    debug_break();
}

// The default allocator of a target with no heap. A board that has memory sets its own allocator
// in the context.
no_heap_allocator_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    if mode == .ALLOCATE || mode == .RESIZE {
        panic("allocation with no heap: the board must set context.allocator");
    }

    if mode == .FREE return null;

    if mode == .CAPS {
        if old_memory {
            (cast(*string) old_memory).* = "no heap";
        }

        return null;
    }

    return null;
}

// C's `size_t` and `int`: the compiler's calls to these functions pass them, so a 32-bit target
// passes 4-byte sizes and a 16-bit one (AVR) passes 2-byte sizes and 2-byte ints.
#if size_of(*void) == 2 {
    Size :: u16;
    Int :: s16;
} else #if size_of(*void) == 4 {
    Size :: u32;
    Int :: s32;
} else {
    Size :: u64;
    Int :: s32;
}

// The C memory functions the compiler's code calls for block copies and fills. There is no libc
// to supply them. Plain byte loops: a board that wants faster ones defines its own `memcpy`.
#program_export "memcpy"
memcpy_symbol :: (to: *void, from: *void, count: Size) -> *void #c_call {
    destination := cast(*u8) to;
    source := cast(*u8) from;
    for index: 0..cast(s64) count - 1 destination[index] = source[index];
    return to;
}

#program_export "memmove"
memmove_symbol :: (to: *void, from: *void, count: Size) -> *void #c_call {
    destination := cast(*u8) to;
    source := cast(*u8) from;

    if cast(u64) destination <= cast(u64) source {
        for index: 0..cast(s64) count - 1 destination[index] = source[index];
    } else {
        index := cast(s64) count;
        while index > 0 {
            index -= 1;
            destination[index] = source[index];
        }
    }

    return to;
}

#program_export "memset"
memset_symbol :: (to: *void, value: Int, count: Size) -> *void #c_call {
    destination := cast(*u8) to;
    byte := cast,no_check(u8) value;
    for index: 0..cast(s64) count - 1 destination[index] = byte;
    return to;
}

#program_export "memcmp"
memcmp_symbol :: (left: *void, right: *void, count: Size) -> Int #c_call {
    a := cast(*u8) left;
    b := cast(*u8) right;

    for index: 0..cast(s64) count - 1 {
        if a[index] != b[index] return cast(Int) a[index] - cast(Int) b[index];
    }

    return 0;
}

// What LLVM calls for a loop that counts bytes up to a zero.
#program_export "strlen"
strlen_symbol :: (text: *u8) -> Size #c_call {
    count: Size = 0;
    while text[count] != 0 count += 1;
    return count;
}

// What LLVM calls for an equality-only `memcmp`.
#program_export "bcmp"
bcmp_symbol :: (left: *void, right: *void, count: Size) -> Int #c_call {
    return memcmp_symbol(left, right, count);
}

#scope_module

trap :: () #intrinsic "llvm.trap";

abort_program :: () #no_context {
    panic("aborted");
}
