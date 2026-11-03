// The current working directory of the program, as an absolute path in a new string from `context.allocator` that the caller frees.
//
// WASI has no working directory of its own: the host pre-opens directories and every path names one of them. The runtime keeps a
// directory name instead. It starts as `PWD` when a pre-opened directory covers it, else `/`, and `set_working_directory` changes
// it. The name is not a handle, so the directory can be deleted and the name stays; the result is never empty.
//
// ```jai
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

        if Wasi.getcwd(memory, cast(u64) capacity) {
            result := copy_string(to_string(memory));
            free(memory);
            return result;
        }

        error := Wasi.errno();

        free(memory);
        if error != Wasi.ERANGE return "";

        assert(capacity <= 0x3fffffffffffffff);
        capacity *= 2;
    }

    return "";
}

// Makes `s` the working directory of the program and returns whether it worked. A missing path, a file, an empty path and a path
// with a NUL byte give `false` and change nothing.
//
// ```jai
// assert(!set_working_directory("jai-doc-no-such-directory/inner"));
// ```
set_working_directory :: (s: string) -> bool {
    if !Native_Path.is_valid(s) return false;
    name := to_c_string(s);
    defer free(name);
    return Wasi.chdir(name) == 0;
}

#scope_file
Wasi :: #import "internal/files/wasi-libc";
Native_Path :: #import "internal/native-path";
