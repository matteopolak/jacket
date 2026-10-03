// Canonical hyphenated GUID strings are the supported input contract here.
uid :: (str: string) -> GUID {
    if str.count != 36 guid_invalid();

    digits: [32] u8;
    at: s64 = 0;

    for i: 0..35 {
        c := str[i];

        if i == 8 || i == 13 || i == 18 || i == 23 {
            if c != #char "-" guid_invalid();
        } else {
            digits[at] = guid_nibble(c);
            at += 1;
        }
    }

    result: GUID;
    for i: 0..7 result.Data1 = (result.Data1 << 4) | cast(u32) digits[i];
    for i: 8..11 result.Data2 = (result.Data2 << 4) | cast(u16) digits[i];
    for i: 12..15 result.Data3 = (result.Data3 << 4) | cast(u16) digits[i];
    for i: 0..7 result.Data4[i] = (digits[16 + 2 * i] << 4) | digits[17 + 2 * i];

    return result;
}

#scope_file
libc :: #library,system "c";
guid_invalid :: () #foreign libc "abort";

guid_nibble :: (c: u8) -> u8 {
    if c >= #char "0" && c <= #char "9" return c - #char "0";
    if c >= #char "a" && c <= #char "f" return c - #char "a" + 10;
    if c >= #char "A" && c <= #char "F" return c - #char "A" + 10;

    guid_invalid();

    return 0;
}
