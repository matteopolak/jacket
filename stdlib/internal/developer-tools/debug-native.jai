backtrace :: (trace: [] *void, skip := 0) -> int {
    return os_backtrace(trace, trace.count, skip);
}

backtrace :: (skip := 0) -> [] string {
    addresses: [256] *void;
    count := backtrace(addresses, skip);
    return os_backtrace_symbols(addresses, count);
}

free_backtrace :: (trace_symbols: [] string) {
    native_free_symbols(trace_symbols.data);
} @Releases

backtrace_symbols :: os_backtrace_symbols;

translate_symbols :: backtrace_symbols;

os_backtrace :: (trace: [] *void, maxcount: int, skip := 0) -> int {
    assert(skip >= 0 && maxcount >= 0, "Debug backtrace counts must be non-negative");
    return native_backtrace(trace.data, min(trace.count, maxcount), skip);
}

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
    os_is_debugger_present :: inline () -> bool {
        return native_debugger_present() != 0;
    }
} else {
    os_is_debugger_present :: () -> bool {
        return native_debugger_present() != 0;
    }
}

is_debugger_present :: inline () -> bool {
    return os_is_debugger_present();
}

is_valid_pointer :: (ptr: *void) -> bool {
    return native_readable(ptr) != 0;
}

output :: (text: string) {
    if native_output(text.data, text.count) == 0 log_error("Debug: native diagnostic output failed\n");
}

abort :: (exit_code := Exit_Code.Abort) {
    os_abort(exit_code);
}

os_abort :: (exit_code: Exit_Code) {
    native_abort(cast(s32) exit_code);
}

breakpoint :: () -> bool #expand #no_debug {
    return native_breakpoint() != 0;
}

break_once :: inline (loc := #caller_location) -> bool {
    result := native_break_once(loc.fully_pathed_filename.data, loc.fully_pathed_filename.count, loc.line_number, loc.character_number);
    if result < 0 log_error("Debug: could not retain breakpoint location\n");
    return result > 0;
}

#scope_file
#import "Basic";

native :: #foreign_library "debug-native/developer_tools_debug";

native_backtrace :: (trace: **void, maxcount: s64, skip: s64) -> s64 #foreign native "developer_debug_backtrace";
native_symbols :: (trace: **void, count: s64) -> *string #foreign native "developer_debug_symbols";
native_free_symbols :: (symbols: *string) #foreign native "developer_debug_free_symbols";
native_debugger_present :: () -> s32 #foreign native "developer_debug_debugger_present";
native_readable :: (pointer: *void) -> s32 #foreign native "developer_debug_readable";
native_output :: (text: *u8, count: s64) -> s32 #foreign native "developer_debug_output";
native_abort :: (code: s32) #foreign native "developer_debug_abort";
native_breakpoint :: () -> s32 #foreign native "developer_debug_breakpoint";
native_break_once :: (filename: *u8, count: s64, line: s64, character: s64) -> s32 #foreign native "developer_debug_break_once";
