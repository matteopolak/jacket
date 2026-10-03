to_utf16 :: (text: string) -> *u16 {
    if text.count > 0x7fffffff return null;
    if !Native_Path.is_valid(text) return null;

    if text.count == 0 {
        result := cast(*u16) alloc(2);
        if result result.* = 0;
        return result;
    }

    count := W.MultiByteToWideChar(65001, 8, text.data, cast(s32) text.count, null, 0);
    if count <= 0 return null;

    result := cast(*u16) alloc((cast(s64) count + 1) * 2);
    if !result return null;

    if W.MultiByteToWideChar(65001, 8, text.data, cast(s32) text.count, result, count) != count {
        free(result);
        return null;
    }

    result[count] = 0;

    return result;
}

from_utf16 :: (text: *u16, count: s32 = -1) -> string, bool {
    bytes := W.WideCharToMultiByte(65001, 0x80, text, count, null, 0, null, null);
    if bytes <= 0 return "", false;

    result := alloc_string(bytes);
    if W.WideCharToMultiByte(65001, 0x80, text, count, result.data, bytes, null, null) != bytes {
        free(result);
        return "", false;
    }

    if count == -1 result.count -= 1;

    return result, true;
}

#scope_file
#import "Basic";
W :: #import "Windows";
Native_Path :: #import "internal/native-path";
