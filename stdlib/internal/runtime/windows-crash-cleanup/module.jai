// Private normal-runtime shutdown; the public crash facade has no fini export.
#assert OS == .WINDOWS && CPU == .X64;

fini :: () {
    error := native_finish();
    assert(error == 0, "Windows exception-handler shutdown failed: %", error);
    Crash.symbols_initialized = false;
}

#scope_file
#import "Basic";
Crash :: #import "Runtime_Support_Crash_Handler";

native_runtime :: #library "../crash-native/platform_io_windows_crash";
native_finish :: () -> u32 #foreign native_runtime "platform_io_windows_crash_finish";
