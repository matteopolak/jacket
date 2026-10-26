//! The shared integer arithmetic against Rust's native operations, type by type.
use super::*;

const AMOUNTS: [i128; 14] = [-200, -1, 0, 1, 7, 8, 9, 15, 16, 31, 32, 33, 64, 200];

/// A test of every operation on the type `$t` (`$signed` says if it is) against native
/// arithmetic.
macro_rules! width_test {
    ($name:ident, $t:ty, $signed:expr) => {
        #[test]
        fn $name() {
            let bits = <$t>::BITS as u8;
            let samples: Vec<$t> = vec![
                0,
                1,
                2,
                3,
                <$t>::MAX,
                <$t>::MAX - 1,
                <$t>::MAX / 2,
                <$t>::MAX / 2 + 1,
                <$t>::MIN,
                <$t>::MIN.wrapping_add(1),
                <$t>::MAX.wrapping_mul(3),
                0x55,
            ];
            let op = |op, x: $t, y: i128| int_binop(op, x as i128, y, bits, $signed);
            let value = |r: Result<i128, DivideByZero>| r.map(|v| v as $t);
            for &x in &samples {
                for &y in &samples {
                    let name = format!("{} {x} {y}", stringify!($t));
                    let y128 = y as i128;
                    assert_eq!(
                        value(op(IntOp::Add, x, y128)),
                        Ok(x.wrapping_add(y)),
                        "{name}"
                    );
                    assert_eq!(
                        value(op(IntOp::Sub, x, y128)),
                        Ok(x.wrapping_sub(y)),
                        "{name}"
                    );
                    assert_eq!(
                        value(op(IntOp::Mul, x, y128)),
                        Ok(x.wrapping_mul(y)),
                        "{name}"
                    );
                    assert_eq!(value(op(IntOp::And, x, y128)), Ok(x & y), "{name}");
                    assert_eq!(value(op(IntOp::Or, x, y128)), Ok(x | y), "{name}");
                    assert_eq!(value(op(IntOp::Xor, x, y128)), Ok(x ^ y), "{name}");
                    if y == 0 {
                        assert_eq!(op(IntOp::Div, x, y128), Err(DivideByZero), "{name}");
                        assert_eq!(op(IntOp::Rem, x, y128), Err(DivideByZero), "{name}");
                    } else {
                        assert_eq!(
                            value(op(IntOp::Div, x, y128)),
                            Ok(x.wrapping_div(y)),
                            "{name}"
                        );
                        assert_eq!(
                            value(op(IntOp::Rem, x, y128)),
                            Ok(x.wrapping_rem(y)),
                            "{name}"
                        );
                    }
                }
                for amount in AMOUNTS {
                    let name = format!("{} {x} by {amount}", stringify!($t));
                    let (shl, shr) = if (0..i128::from(bits)).contains(&amount) {
                        (x << amount, x >> amount)
                    } else {
                        // Out of range: cleared, or filled with the sign bit by a signed right shift.
                        (
                            0,
                            if $signed && (x as i128) < 0 {
                                !0
                            } else {
                                0
                            },
                        )
                    };
                    assert_eq!(value(op(IntOp::Shl, x, amount)), Ok(shl), "{name}");
                    assert_eq!(value(op(IntOp::Shr, x, amount)), Ok(shr), "{name}");
                    let by = amount.rem_euclid(i128::from(bits)) as u32;
                    assert_eq!(
                        value(op(IntOp::Rotl, x, amount)),
                        Ok(x.rotate_left(by)),
                        "{name}"
                    );
                    assert_eq!(
                        value(op(IntOp::Rotr, x, amount)),
                        Ok(x.rotate_right(by)),
                        "{name}"
                    );
                }
            }
        }
    };
}

width_test!(i8_matches_native_arithmetic, i8, true);
width_test!(u8_matches_native_arithmetic, u8, false);
width_test!(i16_matches_native_arithmetic, i16, true);
width_test!(u16_matches_native_arithmetic, u16, false);
width_test!(i32_matches_native_arithmetic, i32, true);
width_test!(u32_matches_native_arithmetic, u32, false);
width_test!(i64_matches_native_arithmetic, i64, true);
width_test!(u64_matches_native_arithmetic, u64, false);

#[test]
fn untyped_constants_use_the_full_width() {
    let op = |op, x, y| int_binop(op, x, y, 128, true);
    assert_eq!(op(IntOp::Shl, 1, 100), Ok(1 << 100));
    assert_eq!(op(IntOp::Shl, 1, 128), Ok(0));
    assert_eq!(op(IntOp::Shr, -5, 128), Ok(-1));
    assert_eq!(op(IntOp::Shr, -5, 200), Ok(-1));
    assert_eq!(op(IntOp::Shr, 5, 200), Ok(0));
    assert_eq!(op(IntOp::Shr, -5, -1), Ok(-1));
    assert_eq!(op(IntOp::Div, i128::MIN, -1), Ok(i128::MIN));
    assert_eq!(op(IntOp::Rotl, 1, 127), Ok(i128::MIN));
}

#[test]
fn wrap_keeps_the_low_bits() {
    assert_eq!(wrap(300, 8, false), 44);
    assert_eq!(wrap(300, 8, true), 44);
    assert_eq!(wrap(200, 8, true), -56);
    assert_eq!(wrap(-1, 8, false), 255);
    assert_eq!(wrap(-1, 64, false), i128::from(u64::MAX));
    assert_eq!(wrap(i128::MIN, 128, true), i128::MIN);
}

#[test]
fn floats_truncate_and_saturate() {
    assert_eq!(float_to_int(300.7, 8, true), 127);
    assert_eq!(float_to_int(-300.7, 8, true), -128);
    assert_eq!(float_to_int(-3.9, 8, true), -3);
    assert_eq!(float_to_int(-1.0, 8, false), 0);
    assert_eq!(float_to_int(255.9, 8, false), 255);
    assert_eq!(float_to_int(1e30, 64, false), i128::from(u64::MAX));
    assert_eq!(float_to_int(-1e30, 64, true), i128::from(i64::MIN));
    assert_eq!(float_to_int(f64::NAN, 32, true), 0);
    assert_eq!(float_to_int(f64::INFINITY, 32, true), i128::from(i32::MAX));
}
