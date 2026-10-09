// Environment variables and terminal detection, the two OS facts Args needs.

#scope_module

#if OS == .WINDOWS {
    Windows :: #import "Windows";
} else {
    libc :: #system_library "libc";
    c_getenv :: (name: *u8) -> *u8 #foreign libc "getenv";

    #if OS != .WASM {
        c_isatty :: (fd: s32) -> s32 #foreign libc "isatty";
    }
}

environment_variable :: (name: string) -> *u8 {
    #if OS == .WINDOWS return Windows.getenv(temp_c_string(name));
    else return c_getenv(temp_c_string(name));
}

environment_set :: (name: string) -> bool {
    return environment_variable(name) != null;
}

// The variable's value, empty when it is not set.
environment_value :: (name: string) -> string {
    value := environment_variable(name);
    return ifx value then to_string(value) else "";
}

forced :: (name: string) -> bool {
    value := environment_value(name);
    return value.count > 0 && value != "0";
}

is_terminal :: (stream: Stream) -> bool {
    #if OS == .WINDOWS {
        ansi := environment_set("WT_SESSION") || environment_set("ANSICON") || environment_value("ConEmuANSI") == "ON" || environment_set("TERM");
        mode: u32;
        handle := Windows.GetStdHandle(ifx stream == .ERROR then Windows.STD_ERROR_HANDLE else Windows.STD_OUTPUT_HANDLE);
        return ansi && Windows.GetConsoleMode(handle, *mode) != 0;
    } else #if OS == .WASM {
        return false;
    } else {
        return c_isatty(ifx stream == .ERROR then 2 else 1) != 0;
    }
}
