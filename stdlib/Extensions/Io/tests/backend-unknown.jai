// test: skip_on(.WASM, reason = "needs Extensions/Io fibers, which wasm cannot run (no stack switching)")
// A JAI_IO_BACKEND that names no backend ends the program with a message, before any fiber runs.
#import "Basic";
Io :: #import "Extensions/Io";

#if OS == .WINDOWS {
    SetEnvironmentVariableA :: (name: *u8, value: *u8) -> s32 #foreign unknown_kernel32;
    unknown_kernel32 :: #system_library "kernel32";

    set_env :: (name: string, value: string) {
        SetEnvironmentVariableA(temp_c_string(name), temp_c_string(value));
    }
} else {
    unknown_setenv :: (name: *u8, value: *u8, overwrite: s32) -> s32 #foreign unknown_libc "setenv";
    unknown_libc :: #system_library "libc";

    set_env :: (name: string, value: string) {
        unknown_setenv(temp_c_string(name), temp_c_string(value), 1);
    }
}

main :: () {
    set_env("JAI_IO_BACKEND", "turbo");

    // Not reached: an unknown name must not be treated as unset.
    Io.io_run(() {
        print("ran\n");
    });
}
