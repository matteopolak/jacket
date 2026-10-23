// Core types, literal typing, `Type` values, `ifx`, and compile-time directives.
#import "Basic";
#import "Math";

DEBUG :: true;
RELEASE :: false;

make_lookup_table :: () -> [4] int {
    return .[1, 4, 9, 16];
}

SQUARES :: #run make_lookup_table();

Video_File :: struct {
    size: int;
}

Audio_File :: struct {
    rate: int;
}

main :: () {
    // Ranges of the integer types.
    assert(size_of(u8) == 1 && size_of(u16) == 2 && size_of(u32) == 4 && size_of(u64) == 8);
    assert(size_of(s8) == 1 && size_of(s16) == 2 && size_of(s32) == 4 && size_of(s64) == 8);
    assert(size_of(float32) == 4 && size_of(float64) == 8);
    assert(int == s64);
    assert(float == float32);

    assert(U8_MAX == 255 && U16_MAX == 65535 && U32_MAX == 4294967295);
    assert(S8_MIN == -128 && S8_MAX == 127 && S16_MIN == -32768 && S16_MAX == 32767);
    assert(S32_MIN == -2147483648 && S32_MAX == 2147483647);
    assert(S64_MAX == 9223372036854775807 && U64_MAX == 18446744073709551615);

    // Two's complement.
    n: s8 = -1;

    assert(cast,no_check(u8) n == 255);
    big: u8 = 255;

    assert(cast,no_check(s8) big == -1);

    // Literal typing.
    assert(type_of(42) == s64);
    assert(type_of(3.5) == float);
    narrow: u8 = 10;

    assert(narrow == 10);
    c := 10;

    assert(type_of(c) == s64);
    d := cast(u8) c;

    assert(type_of(d) == u8);

    use_binary := true;
    base: u64 = ifx use_binary then cast(u64) 1024 else 1000;

    assert(base == 1024);

    // ifx as an expression.
    pick := ifx c > 5 then "big" else "small";

    assert(pick == "big");
    assert((ifx c < 5 then 1 else 2) == 2);
    short := ifx c > 5 then 10;

    assert(short == 10);
    short = ifx c < 5 then 10;
    assert(short == 0);
    nested := ifx c > 100 then 1 else ifx c > 5 then 2 else 3;

    assert(nested == 2);

    // Type values are comparable and unique.
    a: Type = int;
    b: Type = int;
    t: Type = Video_File;

    assert(a == b);
    assert(a != t);
    assert(t == Video_File && t != Audio_File);
    assert(type_of(c) == a);

    info := cast(*Type_Info) Video_File;

    assert(info.type == .STRUCT);
    struct_info := cast(*Type_Info_Struct) Video_File;

    assert(struct_info.name == "Video_File");
    int_info := cast(*Type_Info_Integer) int;

    assert(int_info.runtime_size == 8 && int_info.signed);
    u_info := cast(*Type_Info_Integer) u8;

    assert(u_info.runtime_size == 1 && !u_info.signed);

    // #if keeps one branch.
    #if DEBUG {
        picked := 1;
    } else {
        picked := 2;
    }

    assert(picked == 1);

    #if RELEASE {
        released := true;
    } else {
        released := false;
    }

    assert(!released);

    // #run evaluates at compile time.
    assert(SQUARES[2] == 9 && SQUARES.count == 4);
    assert(#run make_lookup_table()[3] == 16);

    print("ok\n");
}
