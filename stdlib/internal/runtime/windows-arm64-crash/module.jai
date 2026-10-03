// Private native ARM64 context handling; the public crash facade remains x64.
#assert OS == .WINDOWS && CPU == .ARM64;

init :: () {
    error := native_init();
    assert(error == 0, "Windows ARM64 exception-handler initialization failed: %", error);
}

fini :: () {
    error := native_fini();
    assert(error == 0, "Windows ARM64 exception-handler shutdown failed: %", error);
}

#scope_file
#import "Basic";

native_runtime :: #library "../crash-native/platform_io_windows_crash";
native_init :: () -> u32 #foreign native_runtime "platform_io_arm64_crash_init";
native_fini :: () -> u32 #foreign native_runtime "platform_io_arm64_crash_fini";
