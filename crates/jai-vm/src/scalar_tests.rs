use crate::{ArithmeticError, Error, scalar};
use jai_ir::CheckMode;
use jai_types::{IntOp, Integer, IntegerType};

const WIDTHS: [IntegerType; 8] = [
    IntegerType::S8,
    IntegerType::S16,
    IntegerType::S32,
    IntegerType::S64,
    IntegerType::U8,
    IntegerType::U16,
    IntegerType::U32,
    IntegerType::U64,
];
fn number(ty: IntegerType, value: i128) -> Integer {
    Integer::checked(ty, value).unwrap()
}
fn checked(ty: IntegerType, op: IntOp, a: i128, b: i128) -> Result<Integer, Error> {
    scalar::binary_with_check(ty, op, number(ty, a), number(ty, b), CheckMode::Enabled)
}
fn overflow() -> Error {
    Error::Arithmetic(ArithmeticError::IntegerOverflow)
}

#[test]
fn checked_arithmetic_enforces_every_signed_and_unsigned_target_width() {
    for ty in WIDTHS {
        for (op, a, b, result) in [
            (IntOp::Add, ty.max(), 0, ty.max()),
            (IntOp::Subtract, ty.min(), 0, ty.min()),
            (IntOp::Multiply, ty.max(), 1, ty.max()),
            (IntOp::Multiply, ty.min(), 1, ty.min()),
        ] {
            assert_eq!(
                checked(ty, op, a, b),
                Ok(number(ty, result)),
                "{ty:?} {op:?}"
            );
        }
        for (op, a, b) in [
            (IntOp::Add, ty.max(), 1),
            (IntOp::Subtract, ty.min(), 1),
            (IntOp::Multiply, ty.max(), 2),
            (IntOp::Multiply, ty.max(), ty.max()),
        ] {
            assert_eq!(checked(ty, op, a, b), Err(overflow()), "{ty:?} {op:?}");
        }
        if ty.signed() {
            for (op, a, b) in [
                (IntOp::Add, ty.min(), -1),
                (IntOp::Subtract, ty.max(), -1),
                (IntOp::Multiply, ty.min(), -1),
            ] {
                assert_eq!(checked(ty, op, a, b), Err(overflow()), "{ty:?} {op:?}");
            }
        }
    }
}

#[test]
fn eight_bit_checked_add_subtract_and_multiply_match_full_mathematical_ranges() {
    for ty in [IntegerType::S8, IntegerType::U8] {
        for a in ty.min()..=ty.max() {
            for b in ty.min()..=ty.max() {
                for (op, result) in [
                    (IntOp::Add, a + b),
                    (IntOp::Subtract, a - b),
                    (IntOp::Multiply, a * b),
                ] {
                    let expected = if (ty.min()..=ty.max()).contains(&result) {
                        Ok(number(ty, result))
                    } else {
                        Err(overflow())
                    };
                    assert_eq!(checked(ty, op, a, b), expected, "{ty:?} {op:?}({a}, {b})");
                }
            }
        }
    }
}

#[test]
fn disabled_arithmetic_preserves_fixed_width_wraparound() {
    for ty in WIDTHS {
        for (op, a, b, expected) in [
            (IntOp::Add, ty.max(), 1, ty.min()),
            (IntOp::Subtract, ty.min(), 1, ty.max()),
            (IntOp::Multiply, ty.max(), ty.max(), 1),
        ] {
            let a = number(ty, a);
            let b = number(ty, b);
            assert_eq!(
                scalar::binary_with_check(ty, op, a, b, CheckMode::Disabled),
                Ok(number(ty, expected))
            );
            assert_eq!(scalar::binary(ty, op, a, b), Ok(number(ty, expected)));
        }
    }
}

#[test]
fn checked_and_disabled_negation_respect_signedness_and_minimum_values() {
    for ty in WIDTHS {
        assert_eq!(
            scalar::negate_with_check(ty, number(ty, 0), CheckMode::Enabled),
            Ok(number(ty, 0))
        );
        if ty.signed() {
            assert_eq!(
                scalar::negate_with_check(ty, number(ty, ty.min()), CheckMode::Enabled),
                Err(overflow())
            );
            assert_eq!(
                scalar::negate_with_check(ty, number(ty, ty.min() + 1), CheckMode::Enabled),
                Ok(number(ty, ty.max()))
            );
            assert_eq!(
                scalar::negate_with_check(ty, number(ty, ty.min()), CheckMode::Disabled),
                Ok(number(ty, ty.min()))
            );
        } else {
            assert_eq!(
                scalar::negate_with_check(ty, number(ty, 1), CheckMode::Enabled),
                Err(overflow())
            );
            assert_eq!(
                scalar::negate_with_check(ty, number(ty, ty.max()), CheckMode::Disabled),
                Ok(number(ty, 1))
            );
        }
    }
}

#[test]
fn division_overflow_respects_mode_while_zero_and_shift_guards_remain_active() {
    for ty in WIDTHS {
        for check in [CheckMode::Enabled, CheckMode::Disabled] {
            for op in [IntOp::Divide, IntOp::Remainder] {
                assert_eq!(
                    scalar::binary_with_check(ty, op, number(ty, 1), number(ty, 0), check),
                    Err(Error::Arithmetic(ArithmeticError::ZeroDivisor))
                );
                if ty.signed() {
                    let expected = if check.enabled() {
                        Err(Error::Arithmetic(ArithmeticError::SignedDivisionOverflow))
                    } else {
                        Ok(number(ty, if op == IntOp::Divide { ty.min() } else { 0 }))
                    };
                    assert_eq!(
                        scalar::binary_with_check(
                            ty,
                            op,
                            number(ty, ty.min()),
                            number(ty, -1),
                            check
                        ),
                        expected,
                    );
                }
            }
            for op in [IntOp::ShiftLeft, IntOp::ShiftRight] {
                assert_eq!(
                    scalar::binary_with_check(
                        ty,
                        op,
                        number(ty, 1),
                        number(ty, i128::from(ty.bits())),
                        check
                    ),
                    Err(Error::Arithmetic(ArithmeticError::ShiftCount))
                );
                if ty.signed() {
                    assert_eq!(
                        scalar::binary_with_check(ty, op, number(ty, 1), number(ty, -1), check),
                        Err(Error::Arithmetic(ArithmeticError::ShiftCount))
                    );
                }
            }
            assert_eq!(
                scalar::binary_with_check(
                    ty,
                    IntOp::ShiftLeft,
                    number(ty, ty.max()),
                    number(ty, 1),
                    check
                ),
                Ok(Integer::wrapping(ty, ty.max() << 1))
            );
        }
    }
}

#[test]
fn arithmetic_rejects_operand_type_mismatches_before_computing() {
    for check in [CheckMode::Enabled, CheckMode::Disabled] {
        assert_eq!(
            scalar::binary_with_check(
                IntegerType::S8,
                IntOp::Add,
                number(IntegerType::U8, 1),
                number(IntegerType::S8, 1),
                check
            ),
            Err(Error::InvalidIr("integer operand types differ"))
        );
        assert_eq!(
            scalar::negate_with_check(IntegerType::S8, number(IntegerType::U8, 1), check),
            Err(Error::InvalidIr("integer operand types differ"))
        );
    }
}
