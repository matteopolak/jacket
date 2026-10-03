#module_parameters(USE_GRAPHICS := true);

using,only(CRASH_HANDLER_USE_SEPARATE_THREAD, Crash_Handler, crash_handler, Assert_Response, Exit_Code, Minidump_State, ignored_asserts) #import "internal/debug-state";

#load "../internal/developer-tools/debug-report-mode.jai";
#if OS == .MACOS || OS == .LINUX || OS == .WINDOWS #load "../internal/developer-tools/debug-native.jai";
#if OS == .MACOS || OS == .LINUX || OS == .WINDOWS #load "../internal/developer-tools/debug-lifecycle.jai";
#if OS == .MACOS || OS == .LINUX || OS == .WINDOWS #load "../internal/developer-tools/debug-assertions.jai";
#load "../internal/developer-tools/debug-dumps.jai";

#if OS == .WINDOWS && CRASH_HANDLER_USE_SEPARATE_THREAD {
    init_crash_handler_thread :: () {
        // The reviewed alternate declaration promises no initialization effect.
    }
}

#if OS == .ANDROID || OS == .WASM {
    os_is_debugger_present :: () -> bool {
        return false;
    }

    is_debugger_present :: inline () -> bool {
        return os_is_debugger_present();
    }
}

#scope_file
State :: #import "internal/debug-state";
