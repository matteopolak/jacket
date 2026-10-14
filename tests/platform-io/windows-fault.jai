main :: () {
    // Preserve the actual fault and exit status without an interactive WER box.
    set_error_mode(get_error_mode() | 2);
    address := cast(*u8) 1;
    address.* = 23;
}

#scope_file
kernel32 :: #system_library "kernel32";
set_error_mode :: (mode: u32) -> u32 #foreign kernel32 "SetErrorMode";
get_error_mode :: () -> u32 #foreign kernel32 "GetErrorMode";
