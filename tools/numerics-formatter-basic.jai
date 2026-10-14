// Real Basic tool bootstrap for the local macOS formatter build. These APIs
// remain foundation-owned; this file is copied only into local staging.
numerics_tool_tprint :: (format_string: string, args: ..Any) -> string {
    previous := context.allocator;
    context.allocator = temporary_allocator;
    defer context.allocator = previous;
    return sprint(format_string, ..args);
}

numerics_tool_get_working_directory :: () -> string {
    // POSIX getcwd with caller storage; double only for ERANGE.
    capacity: int = 256;

    while true {
        buffer := alloc_string(capacity);
        result := tool_getcwd(buffer.data, cast(u64) capacity);

        if result != null {
            buffer.count = to_string(result).count;
            return buffer;
        }

        free(buffer);
        assert(tool_errno().* == 34, "getcwd failed");
        assert(capacity <= 0x3fffffffffffffff, "getcwd path is too large");
        capacity *= 2;
    }

    return ""; // Unreachable, not a fabricated successful working directory.
}

numerics_tool_exit :: (status: s32) #foreign numerics_tool_libc "exit";

#scope_file
#import "Basic";

#assert(OS == .MACOS, "Formatter bootstrap is for this macOS worker only");
numerics_tool_libc :: #system_library "libc";
tool_getcwd :: (buffer: *u8, size: u64) -> *u8 #foreign numerics_tool_libc "getcwd";
tool_errno :: () -> *s32 #foreign numerics_tool_libc "__error";
