// Process handler ownership is implemented by the source-built OS helper.
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

enable_signal_handler :: (interactive := false) {
    // This argument deliberately does not change the separate report mode.
    os_enable_signal_handler();
}

disable_signal_handler :: () {
    os_disable_signal_handler();
}

os_enable_signal_handler :: () {
    if lifecycle_enable(deliver_crash, *context, size_of(#Context)) == 0 {
        log_error("Debug: signal/exception handler installation failed\n");
        return;
    }

}

os_disable_signal_handler :: () {
    if lifecycle_disable() == 0 {
        log_error("Debug: signal/exception handler restoration failed\n");
        return;
    }

}

no_assert :: (loc: Source_Code_Location, message: string) {
    // Explicitly selected assertion suppression callback.
}

print_assert :: (loc: Source_Code_Location, message: string) {
    output(tprint("%:%:%: assertion failed: %\n", loc.fully_pathed_filename, loc.line_number, loc.character_number, message));
}

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

State :: #import "internal/debug-state";

lifecycle_native :: #foreign_library "debug-native/developer_tools_debug";
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
