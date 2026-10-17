//! Numeric literal contracts from the language and Jai_Lexer specifications.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

fn check(source: &str) {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    if let Err(error) = c.compile_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", error.render(&c.sources));
    }
}

#[test]
fn decimal_precision_counts_fractional_digits_and_ignores_trailing_zeros() {
    check(
        "#assert(type_of(16777216.0) == float32); #assert(type_of(12345678.5) == float32); #assert(type_of(0.1234567) == float32); #assert(type_of(0.12345678) == float64); #assert(type_of(0.123456789) == float64); #assert(type_of(0.300000000000) == float32); #assert(type_of(1.1234_5678) == float64);",
    );
}

#[test]
fn contextual_string_record_literal_is_valid_as_a_call_argument() {
    check(
        "measure :: (s: string) -> int #no_context { return s.count; } text :: \"orchid\"; #assert(measure(.{text.count, text.data}) == 6); #assert(measure(.{count=3, data=text.data}) == 3);",
    );
}

#[test]
fn typed_array_literals_keep_the_written_element_type() {
    check(
        "bytes :: u8.[1, 2, 3]; rows :: [2]u64.[.[3, 4], .[5, 6], .[7, 8]]; #assert(type_of(bytes) == [3]u8); #assert(type_of(rows) == [3][2]u64); #assert(rows[1][0] == 5);",
    );
}

#[test]
fn type_only_indexing_describes_an_array_field_element_without_reading_storage() {
    check(
        "Packet :: struct { values: [4]u16; rows: [2][3]s32; } #assert(type_of(Packet.values[0]) == u16); #assert(type_of(Packet.rows[0][1]) == s32);",
    );
}

#[test]
fn scalar_float_intrinsics_decode_their_actual_register_width() {
    check(
        r#"
        sqrt :: (x: float32) -> float32 #intrinsic;
        sqrt :: (x: float64) -> float64 #intrinsic;
        sin :: (x: float32) -> float32 #intrinsic;
        cos :: (x: float32) -> float32 #intrinsic;
        floor :: (x: float32) -> float32 #intrinsic;
        ceil :: (x: float32) -> float32 #intrinsic;
        round :: (x: float32) -> float32 #intrinsic;
        trunc :: (x: float32) -> float32 #intrinsic;
        fabs :: (x: float32) -> float32 #intrinsic;
        #assert(sqrt(cast(float32) 9) == 3);
        #assert(sqrt(cast(float64) 16) == 4);
        #assert(sin(cast(float32) 0) == 0 && cos(cast(float32) 0) == 1);
        #assert(floor(cast(float32) 1.75) == 1 && ceil(cast(float32) -1.75) == -1);
        #assert(round(cast(float32) -1.5) == -2 && trunc(cast(float32) -1.75) == -1);
        #assert(fabs(cast(float32) -3) == 3);
        #assert(cast,force(u32) sqrt(0h80000000) == 0x80000000);
    "#,
    );
}

#[test]
fn scalar_float_intrinsics_reject_inconsistent_signatures() {
    for source in [
        "sqrt :: (x: s32) -> s32 #intrinsic; #run sqrt(9);",
        "sqrt :: (x: float32) -> float64 #intrinsic; #run sqrt(cast(float32) 9);",
        "sqrt :: (x: float32, y: float32) -> float32 #intrinsic; #run sqrt(cast(float32) 9, cast(float32) 1);",
    ] {
        let mut options = Options::host();
        options.runtime_support = false;
        let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
        let error = c
            .compile_sources(&[ProgramSource::String(source.into())])
            .expect_err("invalid floating intrinsic signature accepted");
        assert!(
            error.message.contains("floating intrinsic signature"),
            "{}",
            error.render(&c.sources)
        );
    }
}

#[test]
fn hex_float_spellings_require_four_eight_or_sixteen_digits() {
    for literal in ["0h0", "0h123", "0h12345", "0h123456789", "0h123456789abcde"] {
        let mut options = Options::host();
        options.runtime_support = false;
        let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
        let result = c.compile_sources(&[ProgramSource::String(format!("value :: {literal};"))]);
        assert!(
            result.is_err(),
            "invalid bit pattern width accepted: {literal}"
        );
    }
    check(
        "#assert(type_of(0h3c00) == float32); #assert(type_of(0h3f800000) == float32); #assert(type_of(0h3ff0000000000000) == float64);",
    );
}

#[test]
fn required_float_precision_survives_aliases_and_constant_arithmetic() {
    for source in [
        "value: float32 = 0.123456789;",
        "precise :: 0.123456789; alias :: precise; value: float32 = alias;",
        "precise :: 0.123456789; value: float32 = -precise;",
        "precise :: 0.123456789; value: float32 = precise + 0.0;",
        "consume :: (x: float32) {} main :: () { consume(0.123456789); }",
    ] {
        let mut options = Options::host();
        options.runtime_support = false;
        let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
        assert!(
            c.compile_sources(&[ProgramSource::String(source.into())])
                .is_err(),
            "required precision was silently narrowed: {source}"
        );
    }
    check(
        "value: float32 = 0.12345678; precise :: 0.123456789; narrowed: float32 = cast(float32) precise; #assert(type_of(narrowed) == float32);",
    );
    check(
        "select :: (x: float32) -> int { return 32; } select :: (x: float64) -> int { return 64; } #assert(select(0.123456789) == 64); #assert(type_of(0.0 + 0.12345678) == float64); #assert(type_of(0.0 + 0.123456789) == float64);",
    );
}

fn literal_record_flags(literal: &str) -> i64 {
    use jaic::records::{FieldRef, Item, Records};
    let mut options = Options::host();
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    c.compile_sources(&[ProgramSource::String(format!("value :: {literal};"))])
        .unwrap();
    let mut records = Records::default();
    let message = c.export_typechecked(&mut records).unwrap();
    let Some(FieldRef::List(list)) = records.field(message, "declarations") else {
        panic!("declarations missing");
    };
    let Some(Item::Ref(typed)) = list.get(0) else {
        panic!("typechecked declaration missing");
    };
    let Some(FieldRef::Ref(decl)) = records.field(typed, "expression") else {
        panic!("declaration missing");
    };
    let Some(FieldRef::Ref(value)) = records.field(decl, "expression") else {
        panic!("literal missing");
    };
    assert_eq!(records.get(value).unwrap().tag, "Code_Literal");
    match records.field(value, "value_flags") {
        Some(FieldRef::Int(v)) => v,
        None => 0,
        _ => panic!("flags have wrong type"),
    }
}

#[test]
fn literal_records_preserve_precision_radix_and_sign_flags() {
    assert_eq!(literal_record_flags("0.12345678"), 0x1 | 0x4 | 0x100);
    assert_eq!(literal_record_flags("0.123456789"), 0x1 | 0x4 | 0x200);
    assert_eq!(literal_record_flags("1.0e39"), 0x1 | 0x4 | 0x200);
    assert_eq!(
        literal_record_flags("-0.12345678"),
        0x1 | 0x4 | 0x40 | 0x100
    );
    assert_eq!(literal_record_flags("0xff"), 0x1 | 0x2);
    assert_eq!(literal_record_flags("0b101"), 0x1 | 0x20);
    assert_eq!(literal_record_flags("-7"), 0x1 | 0x40);
}
