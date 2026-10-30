// The current working directory of the program, as an absolute path in a new string from `context.allocator` that the caller frees.
//
// Relative paths given to the file procedures are resolved against it. The path uses the system's own separators. The
// procedure has no way to report failure: if the system cannot give the directory (it was deleted while the program was in
// it), the program stops with a failed assertion. Not available on wasm.
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
    capacity: s64 = 256;

    while true {
        memory := cast(*u8) alloc(capacity);

        assert(memory != null);

        if P.getcwd(memory, cast(u64) capacity) {
            result := copy_string(to_string(memory));
            free(memory);
            return result;
        }

        error := P.errno();

        free(memory);
        assert(error == cast(P.OS_Error_Code) P.ERANGE, "getcwd failed");
        assert(capacity <= 0x3fffffffffffffff);
        capacity *= 2;
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
    if !Native_Path.is_valid(s) return false;
    name := to_c_string(s);
    defer free(name);
    return P.chdir(name) == 0;
}

exit :: (status: s32) {
    c_exit(status);
} @Exits

sleep_milliseconds :: (milliseconds: s32) {
    assert(milliseconds >= 0);
    remaining: P.timespec;
    remaining.tv_sec = milliseconds / 1000;
    remaining.tv_nsec = cast(s64) (milliseconds % 1000) * 1000000;
    while P.nanosleep(*remaining, *remaining) != 0 {
        assert(P.errno() == cast(P.OS_Error_Code) P.EINTR, "nanosleep failed");
    }
}

#scope_file
P :: #import "internal/bindings/posix-api";
Native_Path :: #import "internal/native-path";

libc :: #system_library "libc"; @native(external)
c_exit :: (status: s32) #foreign libc "exit";
