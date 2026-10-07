// Behavior of small Basic, Bit_Array, Math, String and Toolchains helpers, checked against
// spelled-out expectations.
#import "Basic";
#import "Math";
String :: #import "String";
Bits :: #import "Bit_Array";
Toolchain :: #import "Toolchains/macOS";

Sparse :: enum s32 {
    A :: 5;
    B :: -3;
    C :: 12;
    D :: 0;
}

Lonely :: enum {
    ONLY :: 7;
}

Pair :: struct(N: int, T: Type) {
    values: [N] T;
}

character_classes :: () -> s64 {
    for 0..255 {
        c := cast(u8) it;
        upper := c >= 65 && c <= 90;
        lower := c >= 97 && c <= 122;
        digit := c >= 48 && c <= 57;
        if is_alpha(c) != (upper || lower) return 1;
        if is_digit(c) != digit return 2;
        if is_alnum(c) != (upper || lower || digit || c == 95) return 3;
    }
    return 0;
}

enums_and_arrays :: () -> s64 {
    low, high := enum_range(Sparse);
    if low != -3 || high != 12 return 10;
    low, high = enum_range(Lonely);
    if low != 7 || high != 7 return 11;

    a: [..] int;
    array_add(*a, 4, 8, 15, 16, 8, 42);
    if !array_ordered_remove_by_value(*a, 8) return 12;
    expected :: int.[4, 15, 16, 8, 42];
    if a.count != expected.count return 13;
    for expected if a[it_index] != it return 14;
    if array_ordered_remove_by_value(*a, 99) || a.count != 5 return 15;
    view: [] int = a;
    if !array_ordered_remove_by_value(*view, 42) || view.count != 4 || view[3] != 8 return 16;

    if type_to_string(Pair(3, float)) != "Pair(N=3, T=float32)" return 17;
    return 0;
}

bit_slots :: () -> s64 {
    for count: int.[0, 1, 63, 64, 65, 128, 129] {
        bits := Bits.make_bit_array(count);
        defer Bits.deinit(*bits);
        if bits.slots.count != (count + 63) / 64 || bits.count != count return 20;
        for bits.slots if it != 0 return 21;
    }
    return 0;
}

matrices :: () -> s64 {
    m3 := Matrix3.{ 1, 2, 3, 4, 5, 6, 7, 8, 9 };
    w := make_matrix4(m3);
    if w._11 != 1 || w._23 != 6 || w._33 != 9 || w._14 != 0 || w._34 != 0 || w._41 != 0 || w._44 != 1 return 30;
    m43: Matrix4x3;
    for i: 0..11 m43.floats[i] = cast(float) (i + 1);
    w = make_matrix4(m43);
    if w._11 != m43._11 || w._14 != m43._14 || w._24 != m43._24 || w._34 != m43._34 || w._33 != m43._33 return 31;
    if w._41 != 0 || w._42 != 0 || w._43 != 0 || w._44 != 1 return 32;
    m4: Matrix4;
    for i: 0..15 m4.floats[i] = cast(float) (i * 3 - 7);
    w = make_matrix4(m4);
    for i: 0..15 if w.floats[i] != m4.floats[i] return 33;

    s := scale(m3, Vector3.{ 10, 100, 1000 });
    if s._11 != 10 || s._21 != 40 || s._31 != 70 return 34;
    if s._12 != 200 || s._22 != 500 || s._32 != 800 return 35;
    if s._13 != 3000 || s._23 != 6000 || s._33 != 9000 return 36;
    t := m4;
    scale(*t, Vector3.{ 2, 3, 4 });
    if t._14 != m4._14 || t._41 != m4._41 || t._44 != m4._44 || t._12 != m4._12 * 3 || t._31 != m4._31 * 2 return 37;
    return 0;
}

strings :: () -> s64 {
    if String.compare("abc", "abd") != -1 || String.compare("abd", "abc") != 1 || String.compare("abc", "abc") != 0 return 40;
    if String.compare("ab", "abc") != -1 || String.compare("abc", "ab") != 1 || String.compare("", "") != 0 return 41;
    if String.compare("B", "a") != -1 || String.compare_nocase("B", "a") != 1 || String.compare_nocase("HeLLo", "hello") != 0 return 42;
    if String.compare_nocase("abc", "ABCD") != -1 return 43;
    if !String.equal_nocase("MiXeD", "mixed") || String.equal_nocase("abc", "abcd") || String.equal_nocase("abc", "abd") return 44;
    if !String.equal("x", "x") || String.equal("x", "y") return 45;

    if String.slice("hello", 1, 3) != "ell" || String.slice("hello", 3, 10) != "lo" || String.slice("hello", 5, 1) != "" || String.slice("hello", 9, 1) != "" return 46;

    line := "  Foo=bar ; TRUE false maybe";
    token, ok := String.match_token(*line, "foo");
    if ok || token != "Foo" return 47;
    token, ok = String.match_token(*line, "=");
    if !ok || token != "=" return 48;
    token, ok = String.match_token_nocase(*line, "BAR");
    if !ok || token != "bar" return 49;
    token, ok = String.match_token(*line, ";");
    if !ok return 50;
    value, parsed := String.parse_bool(*line);
    if !parsed || !value return 51;
    value, parsed = String.parse_bool(*line);
    if !parsed || value return 52;
    value, parsed = String.parse_bool(*line);
    if parsed || value return 53;
    value, parsed = String.parse_bool(*line);
    if parsed return 54;

    if String.basename("dir/sub/name.tar.gz") != "name.tar" || String.path_basename("plain") != "plain" return 55;
    if String.path_ensure_extension("a/b.txt", "txt") != "a/b.txt" || String.path_ensure_extension("a/b", "txt") != "a/b.txt" return 56;
    if String.path_ensure_extension("a/b.jai", "txt") != "a/b.jai.txt" return 57;
    return 0;
}

darwin :: () -> s64 {
    if Toolchain.get_darwin_version_from_macos_version(10, 13) != 17 || Toolchain.get_darwin_version_from_macos_version(10, 15) != 19 return 60;
    if Toolchain.get_darwin_version_from_macos_version(11, 0) != 20 || Toolchain.get_darwin_version_from_macos_version(14, 2) != 23 return 61;
    if Toolchain.get_darwin_version_from_ios_version(17, 0) != 23 return 62;
    if Toolchain.get_macos_target_triple(.ARM64, 14, 0) != "arm64-apple-darwin23" return 63;
    return 0;
}

main :: () -> s64 {
    for result: s64.[character_classes(), enums_and_arrays(), bit_slots(), matrices(), strings(), darwin()] {
        if result != 0 return result;
    }
    return 0;
}
