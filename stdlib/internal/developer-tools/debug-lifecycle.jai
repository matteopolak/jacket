// Installs the crash handler and, unless the program set its own, an assert handler:
// [interactive_assert] with `USE_GRAPHICS`, else [break_assert]. On Windows it also sends log
// output to the debugger's output window when the default logger is in use.
init :: () {
    #if OS == .WINDOWS {
        if lifecycle_initialize() == 0 log_error("Debug: DbgHelp initialization failed\n");
        if context.logger == Runtime.runtime_support_default_logger context.logger = os_logger;
    }

    enable_signal_handler();

    if context.assertion_failed == Runtime.runtime_support_assertion_failed {
        #if USE_GRAPHICS context.assertion_failed = interactive_assert;
        else context.assertion_failed = break_assert;
    }
}

#if OS == .WINDOWS {
    // Independent policy: preserve message bytes on the native diagnostic sink.
    // Log_Info metadata and caller data do not change the message text.
    os_logger :: (message: string, data: *void, info: Log_Info) {
        if lifecycle_output(message.data, message.count) == 0 {
            // Bypass the installed logger on sink failure to avoid recursion.
            Runtime.runtime_support_default_logger("Debug: native logger output failed\n", null, Log_Info.{ common_flags = .ERROR });
        }
    }
}

// Installs the handler that prints a stack trace, or calls `crash_handler`, when the program
// crashes. `interactive` is ignored; use [set_report_mode].
enable_signal_handler :: (interactive := false) {
    // This argument deliberately does not change the separate report mode.
    os_enable_signal_handler();
}

// Puts back the crash handling that was there before [enable_signal_handler].
disable_signal_handler :: () {
    os_disable_signal_handler();
}

// [enable_signal_handler]; logs an error when the system refuses.
os_enable_signal_handler :: () {
    if lifecycle_enable(deliver_crash, *context, size_of(#Context)) == 0 {
        log_error("Debug: signal/exception handler installation failed\n");
        return;
    }

}

// [disable_signal_handler]; logs an error when the system refuses.
os_disable_signal_handler :: () {
    if lifecycle_disable() == 0 {
        log_error("Debug: signal/exception handler restoration failed\n");
        return;
    }

}

// An assert handler that does nothing, so failed asserts are ignored.
no_assert :: (loc: Source_Code_Location, message: string) {
    // Explicitly selected assertion suppression callback.
}

// An assert handler that prints `file:line:character: assertion failed: message` and returns.
print_assert :: (loc: Source_Code_Location, message: string) {
    output(tprint("%:%:%: assertion failed: %\n", loc.fully_pathed_filename, loc.line_number, loc.character_number, message));
}

// An assert handler that prints the failure and a stack trace, then returns.
trace_assert :: (loc: Source_Code_Location, message: string) {
    print_assert(loc, message);
    trace := backtrace();

    defer free_backtrace(trace);

    for frame: trace {
        output(frame);
        output("\n");
    }
}

#scope_file
#import "Basic";
Runtime :: #import "Runtime_Support";

lifecycle_native :: #library "debug-native/developer_tools_debug"; @native(developer)
Lifecycle_Callback :: #type (trace: *string, count: s64, data: *void) #c_call;
lifecycle_enable :: (callback: Lifecycle_Callback, data: *void, count: s64) -> s32 #foreign lifecycle_native "developer_debug_enable_handlers";
lifecycle_disable :: () -> s32 #foreign lifecycle_native "developer_debug_disable_handlers";
#if OS == .WINDOWS lifecycle_initialize :: () -> s32 #foreign lifecycle_native "developer_debug_initialize";
#if OS == .WINDOWS lifecycle_output :: (text: *u8, count: s64) -> s32 #foreign lifecycle_native "developer_debug_output";

deliver_crash :: (trace: *string, count: s64, data: *void) #c_call {
    // Native storage is a byte snapshot; restore into target-aligned Jai storage.
    snapshot: #Context = ---;

    memcpy(*snapshot, data, size_of(#Context));

    push_context snapshot {

        if crash_handler crash_handler(.{ count, trace });
        else for frame: ([] string).{ count, trace } {
            output(frame);
            output("\n");
        }
    }
}
