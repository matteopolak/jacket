// Checks the claims of docs/language/casts-and-conversions.md, one group per rule.
// rules: cast.1 cast.2 cast.3 cast.4 cast.5 cast.6 cast.7 cast.8 cast.9 cast.11 cast.13 cast.15 cast.21 cast.22 cast.23
// rules: cast.25 cast.26 cast.27 cast.28
#import "Basic";

Mode :: enum u8 {
    IDLE;
    BUSY;
}

takes_mode :: (m: Mode) -> int {
    return cast(int) m;
}

Name :: #type,distinct string;
Label :: #type,isa string;
Triple :: #type,distinct[3] int;

name_length :: (n: Name) -> int {
    return (cast(string) n).count;
}

Vec3 :: struct {
    x, y, z: float;
}
operator + :: (a: Vec3, b: Vec3) -> Vec3 {
    return .{ a.x + b.x, a.y + b.y, a.z + b.z };
}
Place :: #type,isa Vec3;

doubled :: (v: Vec3) -> Vec3 {
    return .{ v.x * 2, v.y * 2, v.z * 2 };
}

main :: () {
    // cast.1: the example's values.
    wide: u16 = 298;
    small := cast,no_check(u8) wide;
    assert(small == 42, "no_check wraps 298 to 42");
    t := cast,trunc(u8) 256;
    assert(t == 0, "trunc of 256 is 0");
    x: u8 = xx,trunc 298;
    assert(x == 42, "xx,trunc takes the declared type");
    p := (298).(u8);
    assert(type_of(p) == u8 && p == 42, "postfix form");
    f: float32 = 3.7;
    bits := cast,force(u32) f;
    assert(bits == 1080872141, "cast,force reinterprets float bits");

    // cast.2: integer casts wrap, no range trap.
    w := 300;
    assert(cast(u8) w == 44, "300 to u8 is 44");
    neg := -1;
    assert(cast(u16) neg == 65535, "-1 to u16");

    // cast.3: float to integer truncates toward zero.
    a := -3.99;
    assert(cast(s64) a == -3, "-3.99 to -3");
    b := 3.99;
    assert(cast(s64) b == 3, "3.99 to 3");

    // cast.4: s8 -1 to u64.
    n: s8 = -1;
    assert(cast(u64) n == 18446744073709551615, "sign extends then reinterprets");

    // cast.5: cast,force between same-size integer and float.
    one_bits: u64 = 0x3ff0_0000_0000_0000;
    assert(cast,force(float64) one_bits == 1.0, "u64 bits to float64");
    one: float64 = 1.0;
    assert(cast,force(u64) one == 0x3ff0_0000_0000_0000, "float64 to u64 bits");
    half: float32 = 0.5;
    assert(cast,force(u32) half == 0x3f00_0000, "float32 to u32 bits");

    // cast.6: cast(bool) of 5 and "".
    five := 5;
    empty := "";
    assert(cast(bool) five, "5 is true");
    assert(!cast(bool) empty, "\"\" is false");
    assert(cast(bool) "x", "non-empty string is true");

    // cast.7, cast.22: number, enum and pointer casts to bool are exactly true or false.
    two := 2;
    assert(cast(bool) two == true, "2 is exactly true");
    assert(!cast(bool) 2 == false, "!cast(bool) 2 is false");
    busy := Mode.BUSY;
    assert(cast(bool) busy == true && cast(bool) Mode.IDLE == false, "enum to bool");
    np: *int = null;
    assert(cast(bool) np == false && cast(bool) *two == true, "pointer to bool");

    // cast.8: *void accepts any pointer, and converts back implicitly.
    value := 77;
    vp: *void = *value;
    ip: *int = vp;
    assert(ip.* == 77, "*void back to *int");
    raw := alloc(size_of(int));
    typed: *int = raw;
    typed.* = 5;
    assert(typed.* == 5, "alloc result as *int");
    free(raw);

    // cast.9: postfix casts take modifiers after the type.
    big: u64 = 0x1_0000_0005;
    assert(big.(u32, trunc) == 5, "postfix trunc");
    assert(big.(u16, no_check) == 5, "postfix no_check");

    // cast.11: in `xx a == b`, a takes b's type.
    err: s32 = 0;
    assert(xx err == false, "xx s32 compared with a bool");
    wide_code: s32 = 0x101;
    assert(xx wide_code == cast(u8) 1, "xx s32 narrowed to the u8 on the right");
    level: u8 = 1;
    assert(xx level == Mode.BUSY, "xx u8 compared with an enum");

    // cast.13: ifx takes the else-type when only the then-value converts to it.
    some_float: float = 2.5;
    yes := true;
    mixed := ifx yes then 0 else some_float;
    #assert type_of(mixed) == float;
    assert(mixed == 0, "then-value converted to float");
    other: s16 = 4;
    then_wins := ifx yes then some_float else other;
    #assert type_of(then_wins) == float;

    // cast.15: E.loose and E convert implicitly both ways.
    loose: Mode.loose = Mode.BUSY;
    assert(takes_mode(loose) == 1, "loose passed as Mode");
    strict: Mode = loose;
    assert(strict == .BUSY, "loose to Mode");

    // cast.21: pointer & | ^ integer keep the pointer type.
    words: [4] u64;
    wp := *words[2];
    assert(type_of(wp & 7) == *u64 && type_of(wp | 1) == *u64 && type_of(wp ^ 1) == *u64, "pointer type kept");
    assert(cast(u64) ((wp | 1) ^ 1) == cast(u64) wp, "| then ^ round-trips");
    assert(cast(u64) (wp & 7) == 0, "aligned pointer masks to 0");

    // cast.23: pointers and integers convert both ways.
    addr := cast(s64) wp;
    back := cast(*u64) addr;
    assert(back == wp, "integer back to pointer");
    assert(cast(*u8) 0 == null, "cast(*u8) 0 is null");

    // cast.25: a string literal converts to *u8.
    lit: *u8 = "abc";
    assert(lit[0] == #char "a" && lit[2] == #char "c", "literal as *u8");

    // cast.26: literals convert to distinct / isa string variants, and .[...] to a distinct array.
    nm: Name = "hello";
    assert(name_length("hello") == 5, "literal as a distinct string argument");
    assert((cast(string) nm).count == 5, "distinct string variable");
    lb: Label = "tag";
    s: string = lb;
    assert(s == "tag", "isa variant converts to its base");
    tr: Triple = .[1, 2, 3];
    assert((cast([3] int) tr)[2] == 3, "array literal as a distinct array");

    // cast.27, cast.28: a call on an isa variant whose result is the base type returns the variant.
    pa: Place = .{ 1, 2, 3 };
    pb: Place = .{ 1, 1, 1 };
    sum := pa + pb;
    assert(type_of(sum) == Place, "operator + on Place stays Place");
    assert(sum.z == 4, "sum value");
    twice := doubled(pa);
    assert(type_of(twice) == Place && twice.y == 4, "procedure result stays Place");

    print("ok\n");
}
