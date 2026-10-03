#if OS == .WINDOWS {
    utf8_to_wide :: (utf8: string) -> (result_data: *u16, success: bool, result_length_in_bytes: s32) {
        if utf8.count == 0 return null, true, 0;
        if utf8.count < 0 || utf8.count > 2147483647 || !utf8.data return null, false, 0;

        input_count := cast(s32) utf8.count;
        count := Native.MultiByteToWideChar(Native.CP_UTF8, 0, utf8.data, input_count, null, 0);

        if count <= 0 || count > 1073741823 return null, false, 0;

        bytes := cast(s64) count * 2;
        output := cast(*u16) B.alloc(bytes + 2);

        if !output return null, false, 0;

        written := Native.MultiByteToWideChar(Native.CP_UTF8, 0, utf8.data, input_count, output, count);
        if written != count {
            B.free(output);
            return null, false, 0;
        }

        output[count] = 0;

        return output, true, cast(s32) bytes;
    }

    wide_to_utf8 :: (data: *u16, length: s32 = -1) -> (result: string, success: bool) {
        if !data || length == 0 || length < -1 return .{}, false;

        count := Native.WideCharToMultiByte(Native.CP_UTF8, 0, data, length, null, 0, null, null);
        if count <= 0 return .{}, false;

        capacity := cast(s64) count;
        if length != -1 capacity += 1;

        output := cast(*u8) B.alloc(capacity);
        if !output return .{}, false;

        written := Native.WideCharToMultiByte(Native.CP_UTF8, 0, data, length, output, count, null, null);
        if written != count {
            B.free(output);
            return .{}, false;
        }

        logical_count := cast(s64) written;
        if length == -1 logical_count -= 1;

        output[logical_count] = 0;

        return .{ logical_count, output }, true;
    }
}

#scope_file
#if OS == .WINDOWS {
    Native :: #import "Windows";
    B :: #import "Basic";
}
