//! Explicit casts: the range check on narrowing casts, constant folding, and the bit-level
//! reinterpretations, each checked by `#assert` over compile-time execution.
mod common;
use common::compile_source;

/// Compiles `source`, which must hold.
fn holds(source: &str) {
    let (c, result) = compile_source(source);
    if let Err(error) = result {
        panic!("{}\n{source}", error.render(&c.src.sources));
    }
}

/// Compiles `source`, which must be rejected with a message containing `needle`.
fn rejected(source: &str, needle: &str) {
    let (c, result) = compile_source(source);
    let error = result.expect_err(source);
    let rendered = error.render(&c.src.sources);
    assert!(error.message.contains(needle), "{rendered}\n{source}");
}

/// `fn(x: from) -> to` casting `x`, called with `value` at compile time.
fn narrowing(from: &str, to: &str, value: &str) -> String {
    format!("narrow :: (x: {from}) -> {to} {{ return cast({to}) x; }}\n#run narrow({value});")
}

#[test]
fn narrowing_casts_accept_every_value_of_the_target_range() {
    for (from, to, values) in [
        ("s32", "u8", ["0", "255"]),
        ("u32", "s8", ["0", "127"]),
        ("s32", "s8", ["-128", "127"]),
        ("s64", "s32", ["-2147483648", "2147483647"]),
        ("u64", "u32", ["0", "4294967295"]),
        ("s16", "u8", ["0", "255"]),
        ("u16", "s16", ["0", "32767"]),
        ("s32", "u16", ["0", "65535"]),
        ("s64", "u8", ["0", "255"]),
        ("u64", "s32", ["0", "2147483647"]),
        ("s32", "s16", ["-32768", "32767"]),
    ] {
        for value in values {
            holds(&narrowing(from, to, value));
        }
    }
}

#[test]
fn narrowing_casts_reject_values_just_outside_the_target_range() {
    for (from, to, values) in [
        ("s32", "u8", ["256", "-1"]),
        ("u32", "s8", ["128", "4294967295"]),
        ("s32", "s8", ["-129", "128"]),
        ("s64", "s32", ["-2147483649", "2147483648"]),
        ("u64", "u32", ["4294967296", "18446744073709551615"]),
        ("s16", "u8", ["256", "-1"]),
        ("s32", "u16", ["65536", "-1"]),
        ("s64", "u8", ["256", "-1"]),
        ("u64", "s32", ["2147483648", "4294967296"]),
        ("s32", "s16", ["-32769", "32768"]),
    ] {
        for value in values {
            rejected(&narrowing(from, to, value), "overflows");
        }
    }
}

#[test]
fn casts_to_a_type_at_least_as_wide_are_never_checked() {
    for (from, to, value, expected) in [
        ("u8", "s8", "200", "-56"),
        ("s8", "u8", "-1", "255"),
        ("s8", "u64", "-1", "18446744073709551615"),
        ("u64", "s64", "18446744073709551615", "-1"),
        ("s64", "u64", "-1", "18446744073709551615"),
        ("s16", "u32", "-2", "4294967294"),
        ("u32", "s64", "4294967295", "4294967295"),
    ] {
        holds(&format!(
            "wide :: (x: {from}) -> {to} {{ return cast({to}) x; }}\n#assert(wide({value}) == {expected});"
        ));
    }
}

#[test]
fn casts_between_equal_widths_keep_the_bits() {
    holds("flip :: (x: s32) -> u32 { return cast(u32) x; }\n#assert(flip(-1) == 4294967295);");
    holds("flip :: (x: u16) -> s16 { return cast(s16) x; }\n#assert(flip(65535) == -1);");
}

#[test]
fn the_low_bits_survive_a_cast_that_skips_the_check() {
    holds("cut :: (x: s32) -> u8 { return cast,trunc(u8) x; }\n#assert(cut(300) == 44);");
    holds("cut :: (x: s32) -> u8 { return cast,no_check(u8) x; }\n#assert(cut(-1) == 255);");
}

#[test]
fn constant_casts_check_their_range_at_compile_time() {
    holds("#assert(cast(u8) 255 == 255);");
    holds("#assert(cast(s8) -128 == -128);");
    holds("#assert(cast(u16) 65535 == 65535);");
    rejected("x := cast(u8) 256;", "out of range for u8");
    rejected("x := cast(u8) -1;", "out of range for u8");
    rejected("x := cast(s8) 128;", "out of range for s8");
    rejected("x := cast(u8) 256;", "cast,trunc(u8)");
}

#[test]
fn hex_binary_and_named_constants_may_stand_for_bit_patterns() {
    holds("#assert(cast(s8) 0xff == -1);");
    holds("#assert(cast(s16) 0xffff == -1);");
    holds("#assert(cast(s8) 0b10000000 == -128);");
    holds("K :: 255;\n#assert(cast(s8) K == -1);");
    rejected("x := cast(s8) 255;", "out of range for s8");
}

#[test]
fn constant_casts_to_an_enum_are_not_range_checked() {
    holds("E :: enum u8 { A :: 0; B :: 1; }\n#assert(cast(E) 1 == E.B);");
    holds("E :: enum u8 { A :: 0; B :: 1; }\n#assert(cast(u8) E.B == 1);");
}

#[test]
fn numeric_constants_fold_to_the_target_type() {
    holds("#assert(cast(float32) 3 == 3.0);");
    holds("#assert(cast(float64) 3 == 3.0);");
    holds("#assert(cast(s32) 3.9 == 3);");
    holds("#assert(cast(s32) -3.9 == -3);");
    holds("#assert(cast(u8) 3.9 == 3);");
    holds(
        "#assert(cast(float32) 0.1 == 0.1);"
            .replace("== 0.1", "== cast(float32) 0.1")
            .as_str(),
    );
    holds("a : float64 = 0.1; b := cast(float32) a; #assert(cast(float64) b != a);");
    holds("#assert(cast(float64) cast(float32) 16777217 == 16777216.0);");
    holds("#assert(cast(bool) 5 == true);");
    holds("#assert(cast(bool) 0 == false);");
    holds("#assert(cast(bool) -1 == true);");
    holds("#assert(cast(s32) true == 1);");
    holds("#assert(cast(s32) false == 0);");
    holds("#assert(cast(u8) \"A\" == 65);");
}

#[test]
fn forced_constant_casts_reinterpret_the_bits() {
    holds("a : u32 = 0x3f800000; #assert(cast,force(float32) a == 1.0);");
    holds("a : u64 = 0x3ff0000000000000; #assert(cast,force(float64) a == 1.0);");
    holds("a : float32 = 1.0; #assert(cast,force(u32) a == 0x3f800000);");
    holds("a : float64 = 1.0; #assert(cast,force(u64) a == 0x3ff0000000000000);");
    holds("a : float32 = -2.0; #assert(cast,force(s32) a == cast(s32) 0xc0000000);");
    holds("a : u32 = 0x3f800000; #assert(cast(float32) a == 1065353216.0);");
}

#[test]
fn pointer_constants_are_addresses() {
    holds("#assert(cast(*u8) 0 == null);");
    holds("#assert(cast(*u8) 16 != null);");
    holds("#assert(cast(u64) cast(*u8) 4096 == 4096);");
    holds("#assert(cast(*u8) null == null);");
}

#[test]
fn runtime_casts_to_bool_test_for_non_zero() {
    holds(
        "t :: (x: s32) -> bool { return cast(bool) x; }\n#assert(t(2) && t(-1) && t(1) && !t(0));",
    );
    holds("t :: (x: s32) -> bool { return !cast(bool) x; }\n#assert(t(0) && !t(2));");
    holds("t :: (x: u8) -> bool { return cast(bool) x; }\n#assert(t(128) && !t(0));");
    holds("t :: (x: float64) -> bool { return cast(bool) x; }\n#assert(t(0.5) && !t(0.0));");
    holds("t :: (x: *u8) -> bool { return cast(bool) x; }\n#assert(!t(null));");
    holds("t :: (x: string) -> bool { return cast(bool) x; }\n#assert(t(\"a\") && !t(\"\"));");
}

#[test]
fn casting_a_value_to_its_own_type_changes_nothing() {
    holds("same :: (x: s32) -> s32 { return cast(s32) x; }\n#assert(same(-7) == -7);");
    holds(
        "same :: (x: float32) -> float32 { return cast(float32) x; }\n#assert(same(2.5) == 2.5);",
    );
}

#[test]
fn runtime_numeric_casts_convert_the_value() {
    holds("f :: (x: s32) -> float64 { return cast(float64) x; }\n#assert(f(-3) == -3.0);");
    holds("f :: (x: float64) -> s32 { return cast(s32) x; }\n#assert(f(-3.9) == -3);");
    holds("f :: (x: u8) -> s64 { return cast(s64) x; }\n#assert(f(255) == 255);");
    holds("f :: (x: s8) -> s64 { return cast(s64) x; }\n#assert(f(-1) == -1);");
}

#[test]
fn forced_runtime_casts_reinterpret_floats_and_integers_of_one_size() {
    holds("f :: (x: float32) -> u32 { return cast,force(u32) x; }\n#assert(f(1.0) == 0x3f800000);");
    holds(
        "f :: (x: u32) -> float32 { return cast,force(float32) x; }\n#assert(f(0x40000000) == 2.0);",
    );
    holds(
        "f :: (x: float64) -> u64 { return cast,force(u64) x; }\n#assert(f(1.0) == 0x3ff0000000000000);",
    );
    holds(
        "f :: (x: u64) -> float64 { return cast,force(float64) x; }\n#assert(f(0x4000000000000000) == 2.0);",
    );
    // Without `force`, or with different sizes, the value converts instead.
    holds("f :: (x: u32) -> float32 { return cast(float32) x; }\n#assert(f(3) == 3.0);");
    holds("f :: (x: u64) -> float32 { return cast,force(float32) x; }\n#assert(f(3) == 3.0);");
}

#[test]
fn strings_and_byte_arrays_convert_by_layout() {
    holds("a := cast([4] u8) \"abcd\"; #assert(a[0] == 97 && a[3] == 100);");
    holds("a := cast([4] s8) \"abcd\"; #assert(a[1] == 98);");
    rejected("a := cast([3] u8) \"abcd\";", "cannot cast");
    holds("v :: (s: string) -> [] u8 { return xx s; }\n#assert(v(\"hi\").count == 2);");
    holds("v :: (b: [] u8) -> string { return xx b; }\n#assert(v(.[104, 105]) == \"hi\");");
    holds("v :: (b: [2] u8) -> string { return xx b; }\n#assert(v(.[104, 105]) == \"hi\");");
    holds(
        "p :: (s: string) -> *u8 { return cast(*u8) s; }"
            .replace(
                "(s: string) -> *u8 { return cast(*u8) s; }",
                "() -> *u8 { return cast(*u8) \"hi\"; }\n#assert(p() != null);",
            )
            .as_str(),
    );
}

#[test]
fn forced_casts_reinterpret_aggregates_of_the_same_size() {
    holds(
        "A :: struct { x: s32; y: s32; }\nB :: struct { lo: u32; hi: u32; }\nf :: (a: A) -> B { return cast,force(B) a; }\n#assert(f(.{1, 2}).hi == 2);",
    );
    rejected(
        "A :: struct { x: s32; }\nB :: struct { lo: u64; }\nf :: (a: A) -> B { return cast,force(B) a; }",
        "cannot cast",
    );
    rejected(
        "A :: struct { x: s32; }\nB :: struct { lo: s32; }\nf :: (a: A) -> B { return cast(B) a; }",
        "cannot cast",
    );
}

#[test]
fn procedures_cast_by_address() {
    holds("g :: () {}\nf :: () -> *void { return cast(*void) g; }\n#assert(f() != null);");
    holds("g :: () {}\nf :: () -> u64 { return cast(u64) cast(*void) g; }\n#assert(f() != 0);");
}

#[test]
fn checked_casts_report_the_value_and_the_target() {
    let (c, result) = compile_source(&narrowing("s64", "u8", "-1"));
    let rendered = result.expect_err("overflow").render(&c.src.sources);
    assert!(rendered.contains("-1"), "{rendered}");
    let (c, result) = compile_source(&narrowing("s32", "u8", "300"));
    let rendered = result.expect_err("overflow").render(&c.src.sources);
    assert!(rendered.contains("300"), "{rendered}");
}

#[test]
fn enums_are_not_range_checked_at_run_time() {
    holds(
        "E :: enum u32 { A :: 0; B :: 1000; }\nf :: (e: E) -> u8 { return cast(u8) e; }\n#assert(f(.B) == 232);",
    );
    holds(
        "E :: enum u8 { A :: 0; B :: 200; }\nf :: (x: s32) -> E { return cast(E) x; }\n#assert(f(200) == E.B);",
    );
    holds(
        "E :: enum u8 { A :: 0; B :: 200; }\nf :: (x: s32) -> E { return cast(E) x; }\n#assert(f(456) == E.B);",
    );
}

#[test]
fn a_typed_constant_cast_to_its_own_type_stays_typed() {
    rejected("x : u8 = cast(s64) 5;", "");
    holds("x : s64 = cast(s64) 5; #assert(x == 5);");
}

#[test]
fn types_cast_to_type_unchanged() {
    holds("T :: cast(Type) s32;\n#assert(T == s32);");
    holds("x : cast(Type) s32 = 5;\n#assert(type_of(x) == s32);");
}

#[test]
fn string_constants_become_byte_arrays_only_of_matching_element_and_length() {
    rejected("a := cast([4] s32) \"abcd\";", "cannot cast");
    rejected("a := cast([5] u8) \"abcd\";", "cannot cast");
}

#[test]
fn overloaded_procedure_names_do_not_cast_to_a_pointer() {
    rejected(
        "g :: (a: s32) {}\ng :: (a: float32) {}\nf :: () -> *void { return cast(*void) g; }",
        "",
    );
}

#[test]
fn constants_declared_from_casts_stay_constant() {
    holds("U : u32 : 0x3f800000;\nF :: cast,force(float32) U;\n#assert(F == 1.0);");
    holds("U : u64 : 0x3ff0000000000000;\nF :: cast,force(float64) U;\n#assert(F == 1.0);");
    holds("F : float32 : 1.0;\nU :: cast,force(u32) F;\n#assert(U == 0x3f800000);");
    holds("F : float64 : 1.0;\nU :: cast,force(u64) F;\n#assert(U == 0x3ff0000000000000);");
    holds("F : float32 : -1.0;\nU :: cast,force(u32) F;\n#assert(U == 0xbf800000);");
    holds("N :: cast(float32) 3;\n#assert(N == 3.0);");
    holds("N :: cast(float32) 0.1;\n#assert(cast(float64) N != 0.1);");
    holds("N :: cast(float64) 0.1;\n#assert(N == 0.1);");
    holds("N :: cast(s16) 3.9;\n#assert(N == 3);");
    holds("N :: cast(s32) true;\n#assert(N == 1);");
    holds("N :: cast(bool) 3;\n#assert(N);");
    holds("N :: cast(bool) 0;\n#assert(!N);");
    holds("P :: cast(*u8) 4096;\n#assert(cast(u64) P == 4096);");
    holds("P : *u8 : null;\nQ :: cast(*s32) P;\n#assert(Q == null);");
    holds("A : s64 : -1;\nP :: cast(*u8) A;\n#assert(cast(u64) P == 0xffff_ffff_ffff_ffff);");
}
