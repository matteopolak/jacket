Arguments :: struct {
    count: s32;
    values: **u8;
}

acquire :: () -> Arguments, bool #no_context {
    count: s32;
    wide := CommandLineToArgvW(GetCommandLineW(), *count);

    if !wide || count < 0 return .{}, false;

    defer {
        released := LocalFree(wide);
        if released c_abort();
    }

    result := Arguments.{ count, cast(**u8) c_malloc((cast(u64) count + 1) * size_of(*u8)) };
    if !result.values return .{}, false;
    for i: 0..count result.values[i] = null;

    for i: 0..count - 1 {
        bytes := WideCharToMultiByte(65001, 128, wide[i], -1, null, 0, null, null);
        if bytes <= 0 {
            release(*result);
            return .{}, false;
        }

        result.values[i] = cast(*u8) c_malloc(cast(u64) bytes);
        if !result.values[i] || WideCharToMultiByte(65001, 128, wide[i], -1, result.values[i], bytes, null, null) != bytes {
            release(*result);
            return .{}, false;
        }
    }

    return result, true;
}

release :: (arguments: *Arguments) #no_context {
    if arguments.values {
        for i: 0..arguments.count - 1 c_free(arguments.values[i]);
        c_free(arguments.values);
    }

    arguments.* = .{};
}

#scope_file
#import "internal/libc";

kernel32 :: #system_library "kernel32";
shell32 :: #system_library "shell32";
GetCommandLineW :: () -> *u16 #foreign kernel32;
CommandLineToArgvW :: (command: *u16, count: *s32) -> **u16 #foreign shell32;
WideCharToMultiByte :: (code_page: u32, flags: u32, source: *u16, source_count: s32, destination: *u8, capacity: s32, default_character: *u8, used_default: *s32) -> s32 #foreign kernel32;
LocalFree :: (memory: *void) -> *void #foreign kernel32;
