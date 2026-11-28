// Fills `trace` with the return addresses on the current stack, innermost first, skipping the
// first `skip` frames. Returns how many it wrote.
backtrace :: (trace: [] *void, skip := 0) -> int {
    return os_backtrace(trace, trace.count, skip);
}

// Returns up to 256 frames of the current stack as symbolized lines, innermost first, skipping
// the first `skip`. Free the result with [free_backtrace]. Empty when symbolizing fails.
backtrace :: (skip := 0) -> [] string {
    addresses: [256] *void;
    count := backtrace(addresses, skip);
    return os_backtrace_symbols(addresses, count);
}

// Frees what [backtrace] or [backtrace_symbols] returned.
free_backtrace :: (trace_symbols: [] string) {
    native_free_symbols(trace_symbols.data);
}

// [os_backtrace_symbols].
backtrace_symbols :: os_backtrace_symbols;

// [os_backtrace_symbols].
translate_symbols :: backtrace_symbols;

// Fills at most `maxcount` entries of `trace` with return addresses, as [backtrace] does.
os_backtrace :: (trace: [] *void, maxcount: int, skip := 0) -> int {
    assert(skip >= 0 && maxcount >= 0, "Debug backtrace counts must be non-negative");
    return native_backtrace(trace.data, min(trace.count, maxcount), skip);
}

// Returns one symbolized line for each of the first `n` addresses in `trace`. Free the result
// with [free_backtrace]. Logs an error and returns an empty array when symbolizing fails.
os_backtrace_symbols :: (trace: [] *void, n: int) -> [] string {
    assert(n >= 0 && n <= trace.count, "Debug symbol count exceeds the supplied trace");
    if n == 0 return .{};

    result := native_symbols(trace.data, n);
    if !result {
        log_error("Debug: could not symbolize native stack\n");
        return .{};
    }

    return .{ n, result };
}

#if OS == .MACOS || OS == .WINDOWS {
    // Returns whether a debugger is attached to this process.
    os_is_debugger_present :: inline () -> bool {
        return native_debugger_present() != 0;
    }
} else {
    // Returns whether a debugger is attached to this process.
    os_is_debugger_present :: () -> bool {
        return native_debugger_present() != 0;
    }
}

// Returns whether a debugger is attached to this process.
is_debugger_present :: inline () -> bool {
    return os_is_debugger_present();
}

// Returns whether the memory `ptr` points at can be read, without crashing when it can't.
is_valid_pointer :: (ptr: *void) -> bool {
    return native_readable(ptr) != 0;
}

// Writes `text` to standard error, unbuffered.
output :: (text: string) {
    if native_output(text.data, text.count) == 0 log_error("Debug: native diagnostic output failed\n");
}

// Ends the program at once with `exit_code`.
abort :: (exit_code := Exit_Code.Abort) {
    os_abort(exit_code);
}

// [abort].
os_abort :: (exit_code: Exit_Code) {
    native_abort(cast(s32) exit_code);
}

// Stops in the debugger at the caller, when one is attached. Returns whether it stopped.
breakpoint :: () -> bool #expand #no_debug {
    return native_breakpoint() != 0;
}

// [breakpoint], but only the first time this call site is reached. Returns whether it stopped.
break_once :: inline (loc := #caller_location) -> bool {
    result := native_break_once(loc.fully_pathed_filename.data, loc.fully_pathed_filename.count, loc.line_number, loc.character_number);
    if result < 0 log_error("Debug: could not retain breakpoint location\n");
    return result > 0;
}

#scope_file
#import "Basic";

native :: #library "debug-native/developer_tools_debug"; @native(developer)

native_backtrace :: (trace: **void, maxcount: s64, skip: s64) -> s64 #foreign native "developer_debug_backtrace";
native_symbols :: (trace: **void, count: s64) -> *string #foreign native "developer_debug_symbols";
native_free_symbols :: (symbols: *string) #foreign native "developer_debug_free_symbols"; @FreesArgument
native_debugger_present :: () -> s32 #foreign native "developer_debug_debugger_present";
native_readable :: (pointer: *void) -> s32 #foreign native "developer_debug_readable";
native_output :: (text: *u8, count: s64) -> s32 #foreign native "developer_debug_output";
native_abort :: (code: s32) #foreign native "developer_debug_abort";
native_breakpoint :: () -> s32 #foreign native "developer_debug_breakpoint";
native_break_once :: (filename: *u8, count: s64, line: s64, character: s64) -> s32 #foreign native "developer_debug_break_once";
