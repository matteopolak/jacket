// How far a prefix `cast(T)` / `xx` reaches: bitwise and shift operators after its value
// belong to the value, arithmetic, comparison and logical operators apply to the result.
// Postfix operators (`.`, `[]`, calls) always bind first. Every case below would give a
// different value or type with the other grouping.
#import "Basic";

Turn :: enum s32 { WHITE :: 0; BLACK :: 1; }
Flags :: enum_flags { REVERSE; POINTER; }
Pair :: struct { x: s64; y: s64; }

shifted :: () -> u32 { return 0x1234; }

main :: () {
    // `&`: ui_builder's `(cast(float) (hex >> 16) & 0xFF) / 255.0` masks, then converts.
    hex: u32 = 0x427b58;
    red := cast(float) (hex >> 16) & 0xFF;
    assert(type_of(red) == float && red == 66);
    // ... and a pointer can be masked (`cast(u64) p & MASK` is `cast(u64) (p & MASK)`).
    words: [4] u64;
    p := *words[1];
    assert(cast(u64) p & 7 == 0);
    assert(type_of(p & 7) == *u64);
    flags := Flags.POINTER;
    assert(cast(bool) flags & .POINTER);
    assert(!(cast(bool) flags & .REVERSE));

    // `|` and `^`: the value is combined before it is converted.
    small: u16 = 0x10;
    wide: u32 = 0x1;
    assert(type_of(cast(u16) small | wide) == u16);
    turn := Turn.BLACK;
    assert(cast(int) turn ^ 1 == 0);

    // `<<` and `>>`: a byte shifted by 16 is 0 before it is widened, and a value is
    // narrowed after it is shifted down (narrowing 0x1234 first would fail its check).
    byte: u8 = 0x66;
    assert(cast(u32) byte << 16 == 0);
    assert((cast(u32) byte) << 16 == 0x660000);
    word: u32 = 0x1234;
    assert(cast(u8) word >> 8 == 0x12);
    // Among themselves the usual precedence holds: `a << 4 | b` is `(a << 4) | b`.
    low: u8 = 0x3;
    assert(cast(u32) low << 4 | 1 == 0x31);

    // `+` and `-`: pointer offsets and differences use the cast's result.
    a: [4] u32;
    q := *a[0];
    assert(cast(*u8) q + 1 == cast(*u8) (cast(u64) q + 1));
    r := q + 2;
    assert(cast(s64) r - cast(s64) q == 8);

    // `*`, `/`, `%`: arithmetic happens in the target type.
    seven := 7;
    assert(cast(float) seven / 2 == 3.5);
    n8: u8 = 200;
    n32: u32 = 3;
    assert(type_of(cast(u32) n8 * n32) == u32 && cast(u32) n8 * n32 == 600);
    assert(type_of(cast(u32) n8 % n32) == u32);

    // Comparisons and logical operators take the cast's result as an operand.
    five: u32 = 5;
    less := cast(u8) five < 300;
    assert(type_of(less) == bool && less);
    count := 1;
    yes := true;
    assert(type_of(cast(bool) count && yes) == bool);
    assert(cast(bool) 0 || yes);

    // Postfix operators bind before the cast, and keep the bitwise rule after them.
    pair := Pair.{3, 9};
    assert(cast(float) pair.x / 2 == 1.5);
    values := s64.[0x1ff, 0x2ff];
    assert(cast(s32) values[1] & 0xff == 0xff);
    assert(cast(u16) shifted() >> 4 == 0x123);

    // `xx` follows the same rule.
    wide_value: u32 = 0x1ff;
    masked: u8 = xx wide_value & 0xff;
    assert(masked == 0xff);
    total: s64 = xx five + 1;
    assert(total == 6);

    print("ok\n");
}
