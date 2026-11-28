#load "faults.jai";
#load "checks.jai";
#if OS == .WINDOWS #load "windows-io.jai";

// Allocates and sets up the context the program runs on, with temporary storage and the command
// line in `__command_line_arguments`. Returns null when memory runs out. Free it with
// [__jai_runtime_fini].
__jai_runtime_init :: (argc: s32, argv: **u8) -> *#Context #c_call {
    alignment := align_of(#Context);
    if alignment < 16 alignment = 16;

    allocation := c_malloc(cast(u64) (size_of(#Context) + size_of(Runtime_Context_Header) + alignment - 1));
    if !allocation return null;

    address := (cast(u64) allocation + size_of(Runtime_Context_Header) + cast(u64) alignment - 1) & ~(cast(u64) alignment - 1);
    ctx := cast(*#Context) address;
    header := cast(*Runtime_Context_Header) (cast(*u8) ctx - size_of(Runtime_Context_Header));
    header.allocation = allocation;
    ctx.* = .{};
    ctx.context_info = type_info(#Context);
    storage := cast(*B.Temporary_Storage_Paged_Alloc) c_malloc(size_of(B.Temporary_Storage_Paged_Alloc));
    if !storage {
        c_free(allocation);
        return null;
    }

    temporary_size := Runtime_Config.temporary_storage_size();

    storage.* = .{};
    storage.size = temporary_size;

    if temporary_size > 0 {
        storage.data = cast(*u8) c_malloc(cast(u64) temporary_size);
        if !storage.data {
            c_free(storage);
            c_free(allocation);
            return null;
        }
    }

    storage.original_data = storage.data;
    storage.original_size = storage.size;
    storage.overflow_allocator = ctx.allocator;
    header.storage = storage;
    ctx.temporary_storage = *storage.base;
    __command_line_arguments = .{ argc, argv };

    return ctx;
}

// Frees a context made by [__jai_runtime_init], with its temporary storage. Null is ignored.
__jai_runtime_fini :: (_context: *void) #c_call {
    ctx := cast(*#Context) _context;
    if !ctx return;

    header := cast(*Runtime_Context_Header) (cast(*u8) ctx - size_of(Runtime_Context_Header));
    storage := header.storage;

    if storage {
        push_context ctx.* B.reset_temporary_storage(*storage.base);
        c_free(storage.original_data);
        c_free(storage);
    }

    c_free(header.allocation);
}

// Fills an array of `num_elements` elements of `size` bytes by copying the first one into the rest.
__element_duplicate :: (start: *u8, num_elements: s64, size: s64) #c_call #no_aoc {
    for i: 1..num_elements - 1 memcpy(start + i * size, start, size);
}

// Returns the number of bytes before the first zero byte at `str`, or 0 for null.
c_style_strlen :: (str: *u8) -> s64 #no_context {
    if !str return 0;
    count: s64 = 0;
    while str[count] != 0 count += 1;
    return count;
}

// Returns a string viewing the zero-terminated bytes at `c_string`, without copying. Null gives an
// empty string.
to_string :: (c_string: *u8) -> string #no_context {
    return .{ c_style_strlen(c_string), c_string };
}

// [write_string] without taking the output lock, so text from other threads may interleave with
// it. Text a stream won't take is dropped.
write_string_unsynchronized :: (s: string, to_standard_error := false) #no_context {
    #if OS == .ANDROID {
        Android_Output.append_unsynchronized(s, to_standard_error);
        return;
    }

    fd: s32 = 1;
    if to_standard_error fd = 2;

    // A stream that can't be written drops the text: a GUI program has no console, and a closed
    // pipe is no reason to stop the program.
    cursor: s64 = 0;
    while cursor < s.count {
        written := c_write(fd, s.data + cursor, cast(u64) (s.count - cursor));
        if written <= 0 return;

        cursor += written;
    }
}

// Writes `s` to standard output, or standard error with `to_standard_error`, holding a lock so
// lines from different threads don't mix. Unbuffered; text a stream won't take is dropped.
write_string :: (s: string, to_standard_error := false) #no_context #compiler {
    acquire_output_lock();
    defer release_output_lock();
    write_string_unsynchronized(s, to_standard_error);
}

// Writes each string as [write_string] does, all under one lock.
write_strings :: (strings: ..string, to_standard_error := false) #no_context #compiler {
    acquire_output_lock();
    defer release_output_lock();
    for s: strings write_string_unsynchronized(s, to_standard_error);
}

#if OS == .ANDROID {
    // Writes the buffered standard output and error. Without `force`, only complete lines go.
    android_flush_write_string_buffer :: (force := false) #no_context {
        Android_Output.flush(force);
    }
}

// Writes `n` in `base`, clamped to 2..16, with lowercase digits and no prefix.
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

// Writes `n` in `base`, clamped to 2..16, with a leading `-` when negative.
write_number :: (n: s64, base := 10, to_standard_error := false) #no_context #no_abc #no_aoc {
    if n < 0 {
        write_string("-", to_standard_error);
        write_nonnegative_number(0 - cast,no_check(u64) n, base, to_standard_error);
    } else write_nonnegative_number(cast(u64) n, base, to_standard_error);
}

// The default `context.assertion_failed`: prints `file:line: assertion failed: message` and the
// stack trace, if there is one, to standard error, then aborts.
runtime_support_assertion_failed :: (loc: Source_Code_Location, message: string) -> bool {
    write_string(loc.fully_pathed_filename, true);
    write_string(":", true);
    write_number(loc.line_number, 10, true);
    write_strings(": assertion failed: ", message, "\n", to_standard_error = true);
    node := context.stack_trace;
    if node write_string("stack trace:\n", true);

    while node {
        if node.info {
            write_strings("    ", node.info.name, " at ", node.info.location.fully_pathed_filename, ":", to_standard_error = true);
            write_number(node.line_number, 10, true);
            write_string("\n", true);
        }

        node = node.next;
    }

    abort_program();

    return false;
} @Cold

// The default `context.logger`: writes the message to standard output, or standard error when it
// has `.ERROR`, adding a newline if it lacks one.
runtime_support_default_logger :: (message: string, data: *void, info: Log_Info) {
    to_error := (info.common_flags & .ERROR) != 0;

    if message.count && message[message.count - 1] != #char "\n" {
        write_strings(message, "\n", to_standard_error = to_error);
    } else {
        write_string(message, to_error);
    }
}

// Stops in the debugger on Windows; elsewhere it aborts the program.
debug_break :: () #no_context {
    #if OS == .WINDOWS windows_debug_break();
    else abort_program();
}

// [debug_break], callable at compile time.
compile_time_debug_break :: () #compiler #no_context {
    debug_break();
}

#scope_file
B :: #import "Basic";
Runtime_Config :: #import "internal/compiler-api/runtime-config";

Runtime_Context_Header :: struct {
    storage: *B.Temporary_Storage_Paged_Alloc;
    allocation: *void;
}

#import "internal/libc";

Write_Lock :: #import "internal/runtime/output-lock";
#if OS == .ANDROID Android_Output :: #import "internal/runtime/android-output";

// What every failed check ends in, and so visible to the files loaded beside this one. The
// freestanding runtime halts through a panic hook instead.
#scope_module

abort_program :: () #no_context {
    c_abort();
}

#scope_file

acquire_output_lock :: () #no_context {
    Write_Lock.acquire();
}

release_output_lock :: () #no_context {
    released := Write_Lock.release();
    if !released abort_program();
}
