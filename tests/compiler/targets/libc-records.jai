#import "Basic";
#import "File";
#import "File_Utilities";
#import "POSIX";

// Records the C library fills, read back at the program's own layout: the sizes, offsets and
// byte order of `struct stat`, `struct tm`, `struct timespec` and of the `char **` an `strto*`
// call writes depend on the target.
main :: () {
    write_entire_file("records-probe.txt", "hello world");
    defer file_delete("records-probe.txt");

    file: stat_t;

    print("file % % %\n", stat("records-probe.txt", *file), file.st_size, S_ISREG(file.st_mode));
    directory: stat_t;

    print("directory % %\n", stat(".", *directory), S_ISDIR(directory.st_mode));
    print("dated %\n", file.st_mtime.tv_sec > 1_000_000_000);

    listed := false;
    visit :: (info: *File_Visit_Info, listed: *bool) {
        if info.short_name == "records-probe.txt" listed.* = true;
    }

    visit_files(".", false, *listed, visit);
    print("listed %\n", listed);

    // The end pointer is as wide as a pointer: the word after it stays as it was.
    text := "12.5x";

    guard: struct {
        end: *u8;
        after: u32;
    } = .{ null, 0xAABBCCDD };

    value := c_strtod(text.data, *guard.end);

    print("strtod % % %\n", value, guard.end - text.data, formatInt(guard.after, base = 16));
    guard.end = null;
    number := c_strtol("77z", *guard.end, 10);

    print("strtol % %\n", number, formatInt(guard.after, base = 16));

    now: timespec;

    clock_gettime(0, *now);
    print("clock % %\n", now.tv_sec > 1_000_000_000, now.tv_nsec >= 0 && now.tv_nsec < 1_000_000_000);

    seconds: time_t = 86_400 * 365;
    parts: tm;

    gmtime_r(*seconds, *parts);
    print("tm % % %\n", parts.tm_year, parts.tm_mon, parts.tm_mday);
}

c_strtod :: (text: *u8, end: **u8) -> float64 #foreign c_library "strtod";
c_strtol :: (text: *u8, end: **u8, base: s32) -> C_Long #foreign c_library "strtol";
c_library :: #system_library "libc";

#if size_of(*void) == 4 {
    C_Long :: s32;
} else {
    C_Long :: s64;
}
