// Independent wrappers for the Windows SDK UTF conversion APIs.
utf8_to_wide :: (utf8: string) -> (result_data: *u16, success: bool, result_length_in_bytes: s32) {
    if utf8.count == 0 return null, true, 0;
    if utf8.count < 0 || utf8.count > 0x7fffffff return null, false, 0;
    length := cast(s32) utf8.count;
    needed := MultiByteToWideChar(CP_UTF8, 0, utf8.data, length, null, 0);
    if needed <= 0 || needed > 0x3fffffff return null, false, 0;
    storage := cast(*u16) Basic.alloc((cast(s64) needed + 1) * 2);
    if storage == null return null, false, 0;
    written := MultiByteToWideChar(CP_UTF8, 0, utf8.data, length, storage, needed);
    if written <= 0 || written > needed {
        Basic.free(storage);
        return null, false, 0;
    }
    storage[written] = 0;
    return storage, true, written * 2;
}

wide_to_utf8 :: (data: *u16, length: s32 = -1) -> (result: string, success: bool) {
    if length == 0 return "", true;
    if data == null || length < -1 return "", false;
    needed := WideCharToMultiByte(CP_UTF8, 0, data, length, null, 0, null, null);
    if needed <= 0 return "", false;
    storage := cast(*u8) Basic.alloc(cast(s64) needed + 1);
    if storage == null return "", false;
    written := WideCharToMultiByte(CP_UTF8, 0, data, length, storage, needed, null, null);
    if written <= 0 || written > needed {
        Basic.free(storage);
        return "", false;
    }
    output: string;
    output.data = storage;
    output.count = written;
    if length == -1 output.count -= 1;
    storage[output.count] = 0;
    return output, true;
}

utf8_to_wide_new :: (utf8: string) -> (result_data: *u16, success: bool, result_length_in_bytes: s32) #deprecated "Use utf8_to_wide" {
    return utf8_to_wide(utf8);
}

wide_to_utf8_new :: (data: *u16, length: s32 = -1) -> (result: string, success: bool) #deprecated "Use wide_to_utf8" {
    return wide_to_utf8(data, length);
}

#scope_file
Basic :: #import "Basic";
kernel32 :: #system_library "kernel32";
CP_UTF8 :: 65001;
WideCharToMultiByte :: (CodePage: u32, dwFlags: u32, lpWideCharStr: *u16, cchWideChar: s32, lpMultiByteStr: *u8, cbMultiByte: s32, lpDefaultChar: *u8, lpUsedDefaultChar: *s32) -> s32 #foreign kernel32;
MultiByteToWideChar :: (CodePage: u32, dwFlags: u32, lpMultiByteStr: *u8, cbMultiByte: s32, lpWideCharStr: *u16, cchWideChar: s32) -> s32 #foreign kernel32;
