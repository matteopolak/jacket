#scope_module
// Use the byte API, avoiding CRT text-mode translation and 32-bit count truncation.
c_write :: (fd: s32, data: *u8, count: u64) -> s64 #no_context {
    if count > 0x7fffffffffffffff return -1;

    identifier: s32 = -11;
    if fd == 2 identifier = -12;
    else if fd != 1 return -1;

    handle := GetStdHandle(identifier);
    if !handle || handle == cast(*void) -1 return -1;

    chunk := count;
    if chunk > 0x7fffffff chunk = 0x7fffffff;

    written: u32;
    if WriteFile(handle, data, cast(u32) chunk, *written, null) == 0 return -1;

    return cast(s64) written;
}

windows_debug_break :: () #no_context {
    DebugBreak();
}

#scope_file
kernel32 :: #system_library "kernel32";
GetStdHandle :: (identifier: s32) -> *void #foreign kernel32;
WriteFile :: (handle: *void, buffer: *void, count: u32, written: *u32, overlapped: *void) -> s32 #foreign kernel32;
DebugBreak :: () #foreign kernel32;
