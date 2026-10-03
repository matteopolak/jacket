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

set_working_directory :: (s: string) -> bool {
    if !Native_Path.is_valid(s) return false;
    name := to_c_string(s);
    defer free(name);
    return P.chdir(name) == 0;
}

exit :: (status: s32) {
    c_exit(status);
}

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
P :: #import "POSIX";
Native_Path :: #import "internal/native-path";

libc :: #system_library "libc";
c_exit :: (status: s32) #foreign libc "exit";
