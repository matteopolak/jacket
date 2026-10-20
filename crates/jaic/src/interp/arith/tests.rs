//! The register-word arithmetic against native Rust integer and float operations.
use super::*;

const INTS: [Ty; 4] = [Ty::I8, Ty::I16, Ty::I32, Ty::I64];
const AMOUNTS: [u64; 16] = [0, 1, 2, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65, 200];

const WORDS: [u64; 19] = [
    0,
    1,
    2,
    3,
    5,
    0x7f,
    0x80,
    0xff,
    0x7fff,
    0x8000,
    0xffff,
    0x7fff_ffff,
    0x8000_0000,
    0xffff_ffff,
    0x7fff_ffff_ffff_ffff,
    0x8000_0000_0000_0000,
    u64::MAX,
    0x1234_5678_9abc_def0,
    0xdead_beef,
];

fn bits(ty: Ty) -> u32 {
    ty.size() as u32 * 8
}

/// `v` as a value of `ty`: the low bits, zero-extended.
fn norm(ty: Ty, v: u128) -> u64 {
    let mask = if bits(ty) == 64 {
        u64::MAX as u128
    } else {
        (1u128 << bits(ty)) - 1
    };
    (v & mask) as u64
}

/// `v` as a signed value of `ty`, sign-extended.
fn signed(ty: Ty, v: u64) -> i128 {
    let shift = 128 - bits(ty);
    (((norm(ty, v as u128) as u128) << shift) as i128) >> shift
}

fn samples(ty: Ty) -> Vec<u64> {
    WORDS.iter().map(|&w| norm(ty, w as u128)).collect()
}

fn each_pair(f: impl Fn(Ty, u64, u64)) {
    for ty in INTS {
        for x in samples(ty) {
            for y in samples(ty) {
                f(ty, x, y);
            }
        }
    }
}

#[test]
fn wrapping_integer_operations_match_native_arithmetic() {
    each_pair(|ty, x, y| {
        let (a, b) = (x as u128, y as u128);
        assert_eq!(
            bin_total(BinOp::Add, ty, x, y),
            norm(ty, a + b),
            "{ty:?} {x:#x} + {y:#x}"
        );
        assert_eq!(
            bin_total(BinOp::Sub, ty, x, y),
            norm(ty, a.wrapping_sub(b)),
            "{ty:?} {x:#x} - {y:#x}"
        );
        assert_eq!(
            bin_total(BinOp::Mul, ty, x, y),
            norm(ty, a * b),
            "{ty:?} {x:#x} * {y:#x}"
        );
        assert_eq!(bin_total(BinOp::And, ty, x, y), x & y);
        assert_eq!(bin_total(BinOp::Or, ty, x, y), x | y);
        assert_eq!(bin_total(BinOp::Xor, ty, x, y), x ^ y);
    });
}

#[test]
fn shifts_clear_or_fill_once_the_amount_reaches_the_width() {
    for ty in INTS {
        let width = bits(ty) as u64;
        for x in samples(ty) {
            for y in AMOUNTS {
                let shl = if y >= width {
                    0
                } else {
                    norm(ty, (x as u128) << y)
                };
                assert_eq!(bin_total(BinOp::Shl, ty, x, y), shl, "{ty:?} {x:#x} << {y}");
                let lshr = if y >= width {
                    0
                } else {
                    x >> y
                };
                assert_eq!(
                    bin_total(BinOp::LShr, ty, x, y),
                    lshr,
                    "{ty:?} {x:#x} >> {y}"
                );
                let ashr = norm(ty, (signed(ty, x) >> y.min(127)) as u128);
                assert_eq!(
                    bin_total(BinOp::AShr, ty, x, y),
                    ashr,
                    "{ty:?} {x:#x} >>> {y}"
                );
            }
        }
    }
}

#[test]
fn rotations_match_the_native_rotate_of_the_width() {
    for ty in INTS {
        for x in samples(ty) {
            for y in AMOUNTS {
                let n = y as u32;
                let (left, right) = match ty {
                    Ty::I8 => (
                        (x as u8).rotate_left(n) as u64,
                        (x as u8).rotate_right(n) as u64,
                    ),
                    Ty::I16 => (
                        (x as u16).rotate_left(n) as u64,
                        (x as u16).rotate_right(n) as u64,
                    ),
                    Ty::I32 => (
                        (x as u32).rotate_left(n) as u64,
                        (x as u32).rotate_right(n) as u64,
                    ),
                    _ => (x.rotate_left(n), x.rotate_right(n)),
                };
                assert_eq!(
                    bin_total(BinOp::Rotl, ty, x, y),
                    left,
                    "{ty:?} rotl {x:#x} by {y}"
                );
                assert_eq!(
                    bin_total(BinOp::Rotr, ty, x, y),
                    right,
                    "{ty:?} rotr {x:#x} by {y}"
                );
            }
        }
    }
}

#[test]
fn float_operations_match_native_arithmetic() {
    let values = [
        0.0f64,
        -0.0,
        1.0,
        -1.5,
        2.25,
        1e30,
        f64::INFINITY,
        f64::NAN,
        3.0e-310,
    ];
    for &a in &values {
        for &b in &values {
            for (op, want) in [
                (BinOp::FAdd, a + b),
                (BinOp::FSub, a - b),
                (BinOp::FMul, a * b),
                (BinOp::FDiv, a / b),
            ] {
                let got = bin_total(op, Ty::F64, a.to_bits(), b.to_bits());
                assert_eq!(got, want.to_bits(), "{op:?} {a} {b}");
            }
            let (a, b) = (a as f32, b as f32);
            for (op, want) in [
                (BinOp::FAdd, a + b),
                (BinOp::FSub, a - b),
                (BinOp::FMul, a * b),
                (BinOp::FDiv, a / b),
            ] {
                let got = bin_total(op, Ty::F32, a.to_bits() as u64, b.to_bits() as u64);
                assert_eq!(got, want.to_bits() as u64, "{op:?} {a} {b} as f32");
            }
        }
    }
}

#[test]
#[should_panic(expected = "divisions go through")]
fn division_is_not_a_total_operation() {
    bin_total(BinOp::SDiv, Ty::I32, 1, 1);
}

#[test]
fn only_division_and_remainder_can_fail() {
    let failing = [BinOp::SDiv, BinOp::SRem, BinOp::UDiv, BinOp::URem];
    let total = [
        BinOp::Add,
        BinOp::Sub,
        BinOp::Mul,
        BinOp::And,
        BinOp::Or,
        BinOp::Xor,
        BinOp::Shl,
        BinOp::LShr,
        BinOp::AShr,
        BinOp::Rotl,
        BinOp::Rotr,
        BinOp::FAdd,
        BinOp::FSub,
        BinOp::FMul,
        BinOp::FDiv,
    ];
    assert!(failing.iter().all(|&op| divides(op)));
    assert!(total.iter().all(|&op| !divides(op)));
}

#[test]
fn addresses_below_the_first_page_and_tagged_procedures_are_not_plain() {
    assert!(!plain_address(0));
    assert!(!plain_address(4095));
    assert!(plain_address(4096));
    assert!(plain_address(0x7fff_0000_0000));
    assert!(plain_address(FUNC_TAG - 1));
    assert!(!plain_address(FUNC_TAG));
    assert!(!plain_address(u64::MAX));
}

#[test]
fn masks_sign_extension_and_comparison_shifts_follow_the_width() {
    for (ty, keep, shift) in [
        (Ty::I8, 0xffu64, 56u8),
        (Ty::I16, 0xffff, 48),
        (Ty::I32, 0xffff_ffff, 32),
        (Ty::F32, 0xffff_ffff, 32),
        (Ty::I64, u64::MAX, 0),
        (Ty::F64, u64::MAX, 0),
        (Ty::Ptr, u64::MAX, 0),
    ] {
        assert_eq!(mask(ty, u64::MAX), keep, "{ty:?}");
        assert_eq!(
            mask(ty, 0x1234_5678_9abc_def0),
            0x1234_5678_9abc_def0 & keep
        );
        assert_eq!(shift_of(ty), shift, "{ty:?}");
    }
    assert_eq!(sext(Ty::I8, 0x80), -128);
    assert_eq!(sext(Ty::I8, 0x7f), 127);
    assert_eq!(sext(Ty::I8, 0x1_80), -128);
    assert_eq!(sext(Ty::I16, 0x8000), -32768);
    assert_eq!(sext(Ty::I16, 0x7fff), 32767);
    assert_eq!(sext(Ty::I32, 0x8000_0000), -(1 << 31));
    assert_eq!(sext(Ty::I32, 0x7fff_ffff), (1 << 31) - 1);
    assert_eq!(sext(Ty::I64, u64::MAX), -1);
    assert_eq!(sext(Ty::Ptr, 0x8000_0000), 0x8000_0000);
}

#[test]
fn integer_comparisons_agree_with_native_orderings() {
    each_pair(|ty, x, y| {
        let (sx, sy) = (signed(ty, x), signed(ty, y));
        for (op, want) in [
            (CmpOp::Eq, x == y),
            (CmpOp::Ne, x != y),
            (CmpOp::SLt, sx < sy),
            (CmpOp::SLe, sx <= sy),
            (CmpOp::SGt, sx > sy),
            (CmpOp::SGe, sx >= sy),
            (CmpOp::ULt, x < y),
            (CmpOp::ULe, x <= y),
            (CmpOp::UGt, x > y),
            (CmpOp::UGe, x >= y),
        ] {
            assert_eq!(cmp(op, ty, x, y), want, "{ty:?} {x:#x} {op:?} {y:#x}");
        }
    });
}

#[test]
fn comparisons_ignore_bits_above_the_width() {
    assert!(cmp(CmpOp::Eq, Ty::I8, 0x1_05, 0x2_05));
    assert!(!cmp(CmpOp::Ne, Ty::I16, 0xf_0001, 0x1));
    assert!(cmp(CmpOp::SLt, Ty::I32, 0xffff_ffff, 0));
    assert!(!cmp(CmpOp::ULt, Ty::I32, 0xffff_ffff, 0));
}

#[test]
fn float_comparisons_follow_ieee_including_nan() {
    let values = [0.0f64, -0.0, 1.0, -2.5, f64::INFINITY, f64::NAN];
    for &a in &values {
        for &b in &values {
            for (op, want) in [
                (CmpOp::FEq, a == b),
                (CmpOp::FNe, a != b),
                (CmpOp::FLt, a < b),
                (CmpOp::FLe, a <= b),
                (CmpOp::FGt, a > b),
                (CmpOp::FGe, a >= b),
            ] {
                assert_eq!(
                    cmp(op, Ty::F64, a.to_bits(), b.to_bits()),
                    want,
                    "{a} {op:?} {b}"
                );
                let (fa, fb) = (a as f32, b as f32);
                assert_eq!(
                    cmp(op, Ty::F32, fa.to_bits() as u64, fb.to_bits() as u64),
                    want,
                    "{fa} {op:?} {fb} as f32"
                );
            }
        }
    }
}

#[test]
fn integer_conversions_change_width_by_truncation_and_extension() {
    let word = 0xfedc_ba98_7654_3280u64;
    assert_eq!(conv(ConvOp::Trunc, Ty::I64, Ty::I8, word), 0x80);
    assert_eq!(conv(ConvOp::Trunc, Ty::I64, Ty::I16, word), 0x3280);
    assert_eq!(conv(ConvOp::Trunc, Ty::I64, Ty::I32, word), 0x7654_3280);
    assert_eq!(conv(ConvOp::Trunc, Ty::I64, Ty::I64, word), word);
    assert_eq!(conv(ConvOp::ZExt, Ty::I8, Ty::I64, word), 0x80);
    assert_eq!(conv(ConvOp::ZExt, Ty::I16, Ty::I64, word), 0x3280);
    assert_eq!(conv(ConvOp::ZExt, Ty::I32, Ty::I64, word), 0x7654_3280);
    assert_eq!(conv(ConvOp::SExt, Ty::I8, Ty::I16, 0x80), 0xff80);
    assert_eq!(conv(ConvOp::SExt, Ty::I8, Ty::I32, 0x80), 0xffff_ff80);
    assert_eq!(
        conv(ConvOp::SExt, Ty::I8, Ty::I64, 0x80),
        0xffff_ffff_ffff_ff80
    );
    assert_eq!(conv(ConvOp::SExt, Ty::I8, Ty::I64, 0x7f), 0x7f);
    assert_eq!(conv(ConvOp::SExt, Ty::I16, Ty::I32, 0x8000), 0xffff_8000);
    assert_eq!(conv(ConvOp::SExt, Ty::I16, Ty::I64, 0x7fff), 0x7fff);
    assert_eq!(
        conv(ConvOp::SExt, Ty::I32, Ty::I64, 0x8000_0000),
        0xffff_ffff_8000_0000
    );
    assert_eq!(
        conv(ConvOp::Bitcast, Ty::I32, Ty::F32, 0x1_3f80_0000),
        0x3f80_0000
    );
    assert_eq!(
        conv(ConvOp::Bitcast, Ty::F64, Ty::I64, 1.0f64.to_bits()),
        1.0f64.to_bits()
    );
}

#[test]
fn float_to_integer_conversions_saturate_and_send_nan_to_zero() {
    let cases: [(f64, i64, u64); 8] = [
        (0.0, 0, 0),
        (-1.9, -1, 0),
        (1.9, 1, 1),
        (300.0, 300, 300),
        (-300.0, -300, 0),
        (1e30, i64::MAX, u64::MAX),
        (-1e30, i64::MIN, 0),
        (f64::NAN, 0, 0),
    ];
    for (x, _, _) in cases {
        let from64 = x.to_bits();
        let from32 = (x as f32).to_bits() as u64;
        let narrow = x as f32 as f64;
        for (from, source, f) in [(Ty::F64, from64, x), (Ty::F32, from32, narrow)] {
            assert_eq!(
                conv(ConvOp::FToS, from, Ty::I8, source),
                f as i8 as u8 as u64
            );
            assert_eq!(
                conv(ConvOp::FToS, from, Ty::I16, source),
                f as i16 as u16 as u64
            );
            assert_eq!(
                conv(ConvOp::FToS, from, Ty::I32, source),
                f as i32 as u32 as u64
            );
            assert_eq!(conv(ConvOp::FToS, from, Ty::I64, source), f as i64 as u64);
            assert_eq!(conv(ConvOp::FToU, from, Ty::I8, source), f as u8 as u64);
            assert_eq!(conv(ConvOp::FToU, from, Ty::I16, source), f as u16 as u64);
            assert_eq!(conv(ConvOp::FToU, from, Ty::I32, source), f as u32 as u64);
            assert_eq!(conv(ConvOp::FToU, from, Ty::I64, source), f as u64);
        }
    }
    assert_eq!(
        conv(ConvOp::FToS, Ty::F64, Ty::I8, (-3.0f64).to_bits()),
        0xfd
    );
}

#[test]
fn integer_to_float_conversions_round_once_to_the_target() {
    for ty in INTS {
        for x in samples(ty) {
            let (s, u) = (signed(ty, x), norm(ty, x as u128));
            assert_eq!(
                conv(ConvOp::SToF, ty, Ty::F64, x),
                (s as i64 as f64).to_bits()
            );
            assert_eq!(conv(ConvOp::UToF, ty, Ty::F64, x), (u as f64).to_bits());
            assert_eq!(
                conv(ConvOp::SToF, ty, Ty::F32, x),
                ((s as i64) as f32).to_bits() as u64
            );
            assert_eq!(
                conv(ConvOp::UToF, ty, Ty::F32, x),
                (u as f32).to_bits() as u64
            );
        }
    }
    // 2^63 + 2^39 + 1 is above the halfway point between two f32s, but rounding through f64 first
    // drops the final bit and lands on the tie.
    let tie = (1u64 << 63) + (1 << 39) + 1;
    assert_eq!(
        conv(ConvOp::UToF, Ty::I64, Ty::F32, tie),
        (tie as f32).to_bits() as u64
    );
    assert_ne!((tie as f64) as f32, tie as f32);
}

#[test]
fn float_width_changes_round_to_the_target() {
    for x in [
        0.0f64,
        -0.0,
        1.5,
        3.141592653589793,
        1e300,
        f64::NAN,
        f64::INFINITY,
    ] {
        assert_eq!(
            conv(ConvOp::FTrunc, Ty::F64, Ty::F32, x.to_bits()),
            (x as f32).to_bits() as u64
        );
        let narrow = x as f32;
        assert_eq!(
            conv(ConvOp::FExt, Ty::F32, Ty::F64, narrow.to_bits() as u64),
            (narrow as f64).to_bits()
        );
    }
}
