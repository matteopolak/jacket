use super::*;

fn expression(text: &str) -> Expression {
    let mut sources = jai_source::SourceMap::default();
    let source = sources.insert("safety.jai".into(), format!("value :: {text};"));
    let mut symbols = jai_source::Symbols::default();
    let file = jai_syntax::parse_file(sources.get(source).unwrap(), &mut symbols).unwrap();
    let jai_syntax::FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!("expected source declaration");
    };
    let jai_syntax::FileDeclarationKind::Constant(constant) = &declaration.kind else {
        panic!("expected source constant");
    };
    constant.initializer.clone()
}
fn evaluate_check(text: &str, check: CheckMode) -> Result<Value, Diagnostic> {
    evaluate_paths_with_overflow_check(&expression(text), check, |_, span| {
        Err(Diagnostic::new(span, "unknown test name"))
    })
}

#[test]
fn fixed_width_constants_select_checked_or_wrapping_arithmetic() {
    for (ty, maximum, wrapped) in [
        ("u8", "255", 0),
        ("u16", "65535", 0),
        ("u32", "4294967295", 0),
        ("u64", "18446744073709551615", 0),
        ("s8", "127", -128),
        ("s16", "32767", -32768),
        ("s32", "2147483647", -2147483648),
        ("s64", "9223372036854775807", -9223372036854775808),
    ] {
        let source = format!("cast({ty}){maximum}+cast({ty})1");
        assert!(
            evaluate_check(&source, CheckMode::Enabled).is_err(),
            "{source}"
        );
        assert!(
            matches!(
                evaluate_check(&source, CheckMode::Disabled),
                Ok(Value::Int(value)) if value.value() == wrapped
            ),
            "{source}"
        );
    }
    assert_eq!(
        evaluate_check("-cast(s8)(-128)", CheckMode::Disabled),
        Ok(Value::Int(Integer::wrapping(IntegerType::S8, -128)))
    );
    assert!(evaluate_check("-cast(s8)(-128)", CheckMode::Enabled).is_err());
}

#[test]
fn wide_multiplication_checks_the_exact_result_before_bit_truncation() {
    let source = "cast(u64)9223372036854775808*cast(u64)9223372036854775808";
    assert!(evaluate_check(source, CheckMode::Enabled).is_err());
    assert_eq!(
        evaluate_check(source, CheckMode::Disabled),
        Ok(Value::Int(Integer::wrapping(IntegerType::U64, 0)))
    );
}

#[test]
fn exact_float_constant_identity_retains_deferred_integer_check_policy() {
    let source = "ifx cast(u8)255+cast(u8)1 == 0 then 1.5 else 2.5";
    let Value::WeakFloat(checked) = evaluate_check(source, CheckMode::Enabled).unwrap() else {
        panic!("expected contextual decimal expression");
    };
    let Value::WeakFloat(wrapped) = evaluate_check(source, CheckMode::Disabled).unwrap() else {
        panic!("expected contextual decimal expression");
    };
    assert_ne!(checked.request_key(), wrapped.request_key());
    assert!(
        checked
            .round(jai_types::FloatType::F32, Span::default())
            .is_err()
    );
    assert_eq!(
        wrapped
            .round(jai_types::FloatType::F32, Span::default())
            .unwrap()
            .to_f64(),
        1.5
    );
}

#[test]
fn overflow_permission_keeps_cast_divisor_and_shift_validation() {
    for source in ["cast(u8)256", "cast(u8)1/cast(u8)0", "cast(u8)1<<cast(u8)8"] {
        assert!(
            evaluate_check(source, CheckMode::Disabled).is_err(),
            "{source}"
        );
    }
    assert!(
        matches!(evaluate_check("ifx false then cast(u8)255+cast(u8)1 else cast(u8)42", CheckMode::Enabled), Ok(Value::Int(value)) if value.value() == 42)
    );
    assert!(evaluate_check("cast(s8)(-128)/cast(s8)(-1)", CheckMode::Enabled).is_err());
    assert_eq!(
        evaluate_check("cast(s8)(-128)/cast(s8)(-1)", CheckMode::Disabled),
        Ok(Value::Int(Integer::wrapping(IntegerType::S8, -128)))
    );
    assert_eq!(
        evaluate_check("cast(s8)(-128)%cast(s8)(-1)", CheckMode::Disabled),
        Ok(Value::Int(Integer::wrapping(IntegerType::S8, 0)))
    );
}
