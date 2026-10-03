get_working_directory :: () -> result: string {
    capacity: u32 = 256;

    while true {
        buffer := cast(*u16) alloc(cast(s64) capacity * 2);

        assert(buffer != null);
        count := W.GetCurrentDirectoryW(capacity, buffer);
        if count == 0 {
            free(buffer);
            assert(false, "GetCurrentDirectoryW failed");
            return "";
        }

        if count < capacity {
            result, ok := Text.from_utf16(buffer, cast(s32) count);
            free(buffer);
            assert(ok, "working directory cannot be converted to UTF-8");
            return result;
        }

        free(buffer);
        assert(count <= 0x7fffffff);
        capacity = count;
    }

    return "";
}

set_working_directory :: (s: string) -> bool {
    name := Text.to_utf16(s);
    if !name return false;
    defer free(name);
    return W.SetCurrentDirectoryW(name) != 0;
}

exit :: (status: s32) {
    W.ExitProcess(cast,no_check(u32) status);
}

sleep_milliseconds :: (milliseconds: s32) {
    assert(milliseconds >= 0);
    W.Sleep(milliseconds);
}

#scope_file
W :: #import "Windows";
Text :: #import "internal/files/windows-text";
