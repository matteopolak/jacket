// Actual SDK-backed access violation, independent of Jai pointer guards.
Env_Output :: #import "Extensions/Environment";
Compiler_Output :: #import "Compiler";

main :: () {
    set_error_mode(get_error_mode() | 2);
    error := native_fault();
    print("native fault unexpectedly returned; SDK error %\n", error);
}

#scope_file
#import "Basic";

#run Compiler_Output.compiler_add_library_search_directory(Env_Output.output_path("platform-io-tests"));

sdk :: #library "platform_io_fault_sdk";
native_fault :: () -> u32 #foreign sdk "platform_io_native_fault";
kernel32 :: #system_library "kernel32";
set_error_mode :: (mode: u32) -> u32 #foreign kernel32 "SetErrorMode";
get_error_mode :: () -> u32 #foreign kernel32 "GetErrorMode";
