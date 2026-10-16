// Valid C strings and end pointers; no native fault or malformed pointer probes.
libc :: #system_library "libc";
strtod_c :: (text: *u8, end: **u8) -> float64 #foreign libc "strtod";
strtof_c :: (text: *u8, end: **u8) -> float32 #foreign libc "strtof";
errno_c :: () -> *s32 #foreign libc "__errno_location";
locale_c :: (category: s32, name: *u8) -> *u8 #foreign libc "setlocale";
write_c :: (fd: s32, text: *u8, length: s64) -> s64 #foreign libc "write";
exit_c :: (code: s32) #foreign libc "exit";

check64 :: (text: string, bits: u64, used: s64, error: s32 = 77) {
    errno_c().* = 77;
    end: *u8;
    value := strtod_c(text.data, *end);
    if (cast(*u64) *value).* != bits || end != text.data + used || errno_c().* != error exit_c(1);
}

check32 :: (text: string, bits: u32, used: s64, error: s32 = 77) {
    errno_c().* = 77;
    end: *u8;
    value := strtof_c(text.data, *end);
    if (cast(*u32) *value).* != bits || end != text.data + used || errno_c().* != error exit_c(2);
}

main :: () {
    check64("1.5!\0", 0x3ff8000000000000, 3);
    check32("1.5!\0", 0x3fc00000, 3);
    check32("1.000000059604644775390625000001!\0", 0x3f800001, 32);
    check32("1.000000059604644775390625!\0", 0x3f800000, 26);
    check64("0x1.8p+2!\0", 0x4018000000000000, 8);
    check64("-0x1p-1074!\0", 0x8000000000000001, 10);
    check32("0x1p-149!\0", 1, 8);
    check32("0x1.000001p0!\0", 0x3f800000, 12);
    check32("0x1.000003p0!\0", 0x3f800002, 12);
    check64("1e+tail\0", 0x3ff0000000000000, 1);
    check64("  nope\0", 0, 0);
    check64("-0!\0", 0x8000000000000000, 2);
    check64("\v1!\0", 0x3ff0000000000000, 2);
    check64("INF!\0", 0x7ff0000000000000, 3);
    check64("-infinity!\0", 0xfff0000000000000, 9);
    check64("nan(payload_7)!\0", 0x7ff8000000000000, 14);
    check64("nan(unclosed!\0", 0x7ff8000000000000, 3);
    check64("1e999!\0", 0x7ff0000000000000, 5, 34);
    check64("-1e-999!\0", 0x8000000000000000, 7, 34);
    check32("1e999!\0", 0x7f800000, 5, 34);
    check32("-1e-999!\0", 0x80000000, 7, 34);
    c := locale_c(1, null);
    if !c || c[0] != 67 || c[1] != 0 exit_c(3);
    if !locale_c(1, "C\0".data) || !locale_c(1, "POSIX\0".data) exit_c(4);
    if locale_c(1, "de_DE.UTF-8\0".data) || locale_c(-1, null) exit_c(5);

    check64("1,5\0", 0x3ff0000000000000, 1);
    value := strtod_c("7\0".data, null);
    if value != 7 exit_c(6);

    okay: [3] u8 = .[111, 107, 10];
    if write_c(1, okay.data, 3) != 3 exit_c(7);
}
