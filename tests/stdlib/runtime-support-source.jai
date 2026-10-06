Runtime :: #import "Runtime_Support"(DEFINE_SYSTEM_ENTRY_POINT = false, DEFINE_INITIALIZATION = false, ENABLE_BACKTRACE_ON_CRASH = false);
probe :: () -> bool {
    if Runtime.c_style_strlen("abc\0ignored".data) != 3 return false;
    if Runtime.c_style_strlen(null) != 0 return false;
    bytes: [6] u8 = .[7, 9, 0, 0, 0, 0];
    Runtime.__element_duplicate(bytes.data, 3, 2);
    if bytes[2] != 7 || bytes[3] != 9 || bytes[4] != 7 || bytes[5] != 9 return false;
    return true;
}
VERIFIED :: #run probe();
#assert VERIFIED;
