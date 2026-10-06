// Unicode exports the classic UTF-8 tables: decode "é" (C3 A9) by hand.
#import "Basic";
#import "Unicode";
main :: () {
    s := "é";
    extra := trailingBytesForUTF8[s[0]];
    assert(extra == 1 && trailingBytesForUTF8[#char "a"] == 0 && trailingBytesForUTF8[0xF0] == 3);
    code: u32;
    for 0..extra {
        code <<= 6;
        code += s[it];
    }
    code -= offsetsFromUTF8[extra];
    assert(code == 0xE9);
    assert(firstByteMark[2] == 0xC0 && utf8_inital_byte_mask[1] == 0x1F);
}
