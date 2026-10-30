// Environment variables and terminal detection, the two OS facts Args needs.

#scope_module

#if OS == .WASM {
    wasi :: #system_library "wasi_snapshot_preview1";
    wasi_proc_exit :: (status: u32) #foreign wasi "proc_exit";
}

// Basic exposes exit on native targets. WASI commands terminate through the
// preview1 process ABI, rather than an unresolved host libc import.
terminate_process :: (status: s32) {
    #if OS == .WASM wasi_proc_exit(cast(u32) status);
    else exit(status);
}

Env :: #import "Extensions/Environment";

#if OS == .WINDOWS {
    Windows :: #import "Windows";
} else {
    libc :: #system_library "libc";

    #if OS != .WASM {
        c_isatty :: (fd: s32) -> s32 #foreign libc "isatty";
    }
}

forced :: (name: string) -> bool {
    value := Env.environment_variable(name);
    return value.count > 0 && value != "0";
}

is_terminal :: (stream: Stream) -> bool {
    #if OS == .WINDOWS {
        ansi := Env.has_environment_variable("WT_SESSION") || Env.has_environment_variable("ANSICON") || Env.environment_variable("ConEmuANSI") == "ON" || Env.has_environment_variable("TERM");
        mode: u32;
        handle := Windows.GetStdHandle(cast(s32) ifx stream == .ERROR then Windows.STD_ERROR_HANDLE else Windows.STD_OUTPUT_HANDLE);
        return ansi && Windows.GetConsoleMode(handle, *mode) != 0;
    } else #if OS == .WASM {
        return false;
    } else {
        return c_isatty(cast(s32) ifx stream == .ERROR then 2 else 1) != 0;
    }
}
