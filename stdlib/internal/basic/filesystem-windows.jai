// The current working directory of the program, as an absolute path in a new string from `context.allocator` that the caller frees.
//
// Relative paths given to the file procedures are resolved against it. The path uses the system's own separators. The
// system cannot give the directory the result is the empty string, which no working directory is, and there is nothing to
// free. Not available on wasm.
//
// ```jai,skip_on(wasm)
// original := get_working_directory();
// defer free(original);
// assert(original.count > 0);
//
// assert(set_working_directory("."));
// same := get_working_directory();
// assert(same == original);
// free(same);
// ```
get_working_directory :: () -> result: string {
    capacity: u32 = 256;

    while true {
        buffer := cast(*u16) alloc(cast(s64) capacity * 2);

        assert(buffer != null);
        count := W.GetCurrentDirectoryW(capacity, buffer);
        if count == 0 {
            free(buffer);
            return "";
        }

        if count < capacity {
            result, ok := Text.from_utf16(buffer, cast(s32) count);
            free(buffer);
            if !ok return "";
            return result;
        }

        free(buffer);
        assert(count <= 0x7fffffff);
        capacity = count;
    }

    return "";
}

// Makes `s` the working directory of the whole program, for every thread, and returns whether it worked. A missing path, a
// file, a directory without permission to enter, an empty path and a path with a NUL byte give `false` and change nothing.
//
// ```jai,skip_on(wasm)
// assert(!set_working_directory("jai-doc-no-such-directory/inner"));
// ```
set_working_directory :: (s: string) -> bool {
    name := Text.to_utf16(s);
    if !name return false;
    defer free(name);
    return W.SetCurrentDirectoryW(name) != 0;
}

exit :: (status: s32) {
    W.ExitProcess(cast,no_check(u32) status);
} @Exits

sleep_milliseconds :: (milliseconds: s32) {
    assert(milliseconds >= 0);
    W.Sleep(milliseconds);
}

#scope_file
W :: #import "internal/bindings/windows-api";
Text :: #import "internal/files/windows-text";
