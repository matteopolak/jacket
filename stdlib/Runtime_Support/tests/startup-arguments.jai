#import "Basic";

main :: () {
    #if OS == .WINDOWS {
        arguments := get_command_line_arguments();

        assert(arguments.count >= 1 && arguments[0].count > 0);
        assert(context.context_info == type_info(#Context));
        assert(context.allocator.proc != null && context.temporary_storage != null);

        pair: [3] u16 = .[0xd83d, 0xde00, 0];
        bytes := wide_to_utf8(65001, 128, pair.data, -1, null, 0, null, null);

        assert(bytes == 5);
        output: [5] u8;

        assert(wide_to_utf8(65001, 128, pair.data, -1, output.data, 5, null, null) == 5);
        assert(output[0] == 0xf0 && output[1] == 0x9f && output[2] == 0x98 && output[3] == 0x80 && output[4] == 0);

        for invalid: u16.[0xd800, 0xdc00] {
            malformed: [2] u16 = .[invalid, 0];
            count := wide_to_utf8(65001, 128, malformed.data, -1, null, 0, null, null);
            error := native_error();
            assert(count == 0 && error == 1113);
        }

        short := wide_to_utf8(65001, 128, pair.data, -1, output.data, 4, null, null);
        error := native_error();

        assert(short == 0 && error == 122);
    }

    print("ok\n");
}

#scope_file
#if OS == .WINDOWS {
    kernel32 :: #system_library "kernel32";
    wide_to_utf8 :: (page: u32, flags: u32, source: *u16, source_count: s32, destination: *u8, capacity: s32, default_character: *u8, used_default: *s32) -> s32 #foreign kernel32 "WideCharToMultiByte";
    native_error :: () -> u32 #foreign kernel32 "GetLastError";
}
