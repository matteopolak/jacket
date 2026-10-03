// Private strict encoding bridge. Returned storage uses the caller's allocator.
utf16_path :: (text: string) -> *u16 {
    if text.count <= 0 || text.count > 0x7ffffffe return null;
    for byte: text if byte == 0 return null;

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

utf8_text :: (text: *u16, count: s64) -> string, bool {
    if count < 0 || count > 0x7fffffff || (!text && count > 0) return "", false;
    if count == 0 return "", true;

    bytes := W.WideCharToMultiByte(65001, 128, text, cast(s32) count, null, 0, null, null);
    if bytes <= 0 return "", false;

    result := alloc_string(bytes);
    if W.WideCharToMultiByte(65001, 128, text, cast(s32) count, result.data, bytes, null, null) != bytes {
        free(result);
        return "", false;
    }

    return result, true;
}

#scope_file
#import "Basic";
W :: #import "Windows";
