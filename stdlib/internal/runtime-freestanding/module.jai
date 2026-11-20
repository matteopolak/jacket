// Runtime_Support's implementation for a program with no operating system (`OS == .NONE`).
//
// There is no libc, no heap and no threads. Output and failure go through two hooks that the
// program or its board sets; `Extensions/Embedded` is how a program reaches them. With no console
// hook, output is dropped. With no panic hook, a failure writes its reason to the console and
// halts.

#load "../runtime/faults.jai";
#load "../runtime/checks.jai";

#assert OS == .NONE "internal/runtime-freestanding is for freestanding targets";

// Receives everything `write_string`, `print` and the failure reports write. The second argument
// is true for error output.
Console_Hook :: #type (text: string, to_standard_error: bool) #no_context;

// Called when the program cannot continue (a failed assert or check, or an allocation with no
// heap). It must not return; if it does, the program halts.
Panic_Hook :: #type (message: string) #no_context;

console_hook: Console_Hook;
panic_hook: Panic_Hook;

halt :: () #no_context {
    while true trap();
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

__element_duplicate :: (start: *u8, num_elements: s64, size: s64) #c_call #no_aoc {
    for i: 1..num_elements - 1 memcpy(start + i * size, start, size);
}

c_style_strlen :: (str: *u8) -> s64 #no_context {
    if !str return 0;
    count: s64 = 0;
    while str[count] != 0 count += 1;
    return count;
}

to_string :: (c_string: *u8) -> string #no_context {
    return .{ c_style_strlen(c_string), c_string };
}

write_string_unsynchronized :: (s: string, to_standard_error := false) #no_context {
    if console_hook console_hook(s, to_standard_error);
}

// There are no threads, so there is no lock to take.
write_string :: (s: string, to_standard_error := false) #no_context #compiler {
    write_string_unsynchronized(s, to_standard_error);
}

write_strings :: (strings: ..string, to_standard_error := false) #no_context #compiler {
    for s: strings write_string_unsynchronized(s, to_standard_error);
}

write_nonnegative_number :: (n: u64, base := 10, to_standard_error := false) #no_context #no_abc #no_aoc {
    radix := base;
    if radix < 2 radix = 2;
    if radix > 16 radix = 16;

    buffer: [64] u8;
    cursor: s64 = 64;
    remaining := n;

    while true {
        cursor -= 1;
        digit := remaining % cast(u64) radix;

        buffer[cursor] = "0123456789abcdef"[cast(s64) digit];
        remaining /= cast(u64) radix;
        if remaining == 0 break;
    }

    write_string(string.{ 64 - cursor, buffer.data + cursor }, to_standard_error);
}

write_number :: (n: s64, base := 10, to_standard_error := false) #no_context #no_abc #no_aoc {
    if n < 0 {
        write_string("-", to_standard_error);
        write_nonnegative_number(0 - cast,no_check(u64) n, base, to_standard_error);
    } else write_nonnegative_number(cast(u64) n, base, to_standard_error);
}

runtime_support_assertion_failed :: (loc: Source_Code_Location, message: string) -> bool {
    write_string(loc.fully_pathed_filename, true);
    write_string(":", true);
    write_number(loc.line_number, 10, true);
    write_strings(": assertion failed: ", message, "\n", to_standard_error = true);
    panic("assertion failed");

    return false;
} @Cold

runtime_support_default_logger :: (message: string, data: *void, info: Log_Info) {
    to_error := (info.common_flags & .ERROR) != 0;

    if message.count && message[message.count - 1] != #char "\n" {
        write_strings(message, "\n", to_standard_error = to_error);
    } else {
        write_string(message, to_error);
    }
}

debug_break :: () #no_context {
    trap();
}

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

#scope_module

trap :: () #intrinsic "llvm.trap";

abort_program :: () #no_context {
    panic("aborted");
}
