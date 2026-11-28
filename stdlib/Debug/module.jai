// Crash and assert reporting, stack traces and debugger control.
//
// - [init] installs the crash handler and an assert handler that prints the failure and, with
//   `USE_GRAPHICS`, asks in a dialog whether to retry, ignore or abort.
// - [backtrace] returns the current stack, symbolized; free it with [free_backtrace].
// - [is_debugger_present], [attach_to_debugger], [breakpoint] and [break_once] work with a debugger.
// - [write_minidump] and [set_minidump_state] write crash dumps on Windows; elsewhere they do nothing.
//
// ```jai,no_run
// init();
// trace := backtrace();
// defer free_backtrace(trace);
// for trace print("%\n", it);
// ```

#module_parameters(
    // Reports a failed assert in an interactive window, not only on the console.
    USE_GRAPHICS := true
);

using,only(CRASH_HANDLER_USE_SEPARATE_THREAD, Crash_Handler, crash_handler, Assert_Response, Exit_Code, Minidump_State, ignored_asserts) #import "internal/debug-state";

#load "../internal/developer-tools/debug-report-mode.jai";
#if OS == .MACOS || OS == .LINUX || OS == .WINDOWS #load "../internal/developer-tools/debug-native.jai";
#if OS == .MACOS || OS == .LINUX || OS == .WINDOWS #load "../internal/developer-tools/debug-lifecycle.jai";
#if OS == .MACOS || OS == .LINUX || OS == .WINDOWS #load "../internal/developer-tools/debug-assertions.jai";
#load "../internal/developer-tools/debug-dumps.jai";

#if OS == .WINDOWS && CRASH_HANDLER_USE_SEPARATE_THREAD {
    // Does nothing: the crash handler runs on the crashing thread.
    init_crash_handler_thread :: () {
        // The alternate declaration promises no initialization effect.
    }
}

#if OS == .ANDROID || OS == .WASM {
    // Returns false: there is no debugger to detect on this target.
    os_is_debugger_present :: () -> bool {
        return false;
    }

    // Returns false: there is no debugger to detect on this target.
    is_debugger_present :: inline () -> bool {
        return os_is_debugger_present();
    }
}

#scope_file
State :: #import "internal/debug-state";
