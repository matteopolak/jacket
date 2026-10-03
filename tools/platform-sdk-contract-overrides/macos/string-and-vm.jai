// Independently authored CoreFoundation ownership and conversion wrappers.
cfstring_from_string :: (str: string) -> CFStringRef {
    return CFStringCreateWithBytes(null, str.data, str.count, .UTF8, false);
}
to_string :: (cfstring: CFStringRef) -> string {
    if cfstring == null return "";
    character_count := CFStringGetLength(cfstring);
    if character_count == 0 return "";
    capacity := CFStringGetMaximumSizeForEncoding(character_count, .UTF8);
    if capacity < 0 || capacity == 0x7fffffffffffffff return "";
    bytes := cast(*u8) Basic.alloc(capacity + 1);
    if bytes == null return "";
    if !CFStringGetCString(cfstring, bytes, capacity + 1, .UTF8) {
        Basic.free(bytes);
        return "";
    }
    length: s64 = 0;
    while length < capacity && bytes[length] != 0 length += 1;
    result: string;
    result.data = bytes;
    result.count = length;
    return result;
}
VM_MAKE_TAG :: (val: s32) -> s32 {
    return cast,trunc(s32) (cast(u32) val << 24);
}

#scope_file
Basic :: #import "Basic";
