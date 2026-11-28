// test: skip_on(.WASM, reason = "needs Extensions/Io fibers, which wasm cannot run (no stack switching)")
// A backend named in JAI_IO_BACKEND that this system cannot give ends the program with a message,
// before any fiber runs. The program asks for a backend its own OS does not have.
#import "Basic";
Io :: #import "Extensions/Io";

#if OS == .WINDOWS {
    SetEnvironmentVariableA :: (name: *u8, value: *u8) -> s32 #foreign refused_kernel32;
    refused_kernel32 :: #system_library "kernel32";

    set_env :: (name: string, value: string) {
        SetEnvironmentVariableA(temp_c_string(name), temp_c_string(value));
    }

    absent_backend :: "epoll";
} else {
    refused_setenv :: (name: *u8, value: *u8, overwrite: s32) -> s32 #foreign refused_libc "setenv";
    refused_libc :: #system_library "libc";

    set_env :: (name: string, value: string) {
        refused_setenv(temp_c_string(name), temp_c_string(value), 1);
    }

    absent_backend :: "iocp";
}

main :: () {
    set_env("JAI_IO_BACKEND", absent_backend);

    // Not reached: a scheduler that quietly used another backend would run this.
    Io.io_run(() {
        print("ran\n");
    });
}
