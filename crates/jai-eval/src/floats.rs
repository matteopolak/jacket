//! Pure contextual float constants, sharing bit-level operations with the VM.
use super::*;
use jai_syntax::{BuiltinType, DecimalLiteral, FloatLiteral, TypeSyntax};
use jai_types::{FloatOp, FloatToIntMode, FloatType, FloatValue};
mod keys;
pub use keys::{WeakFloatEncodingError, WeakFloatKey};

/// A bound weak numeric expression retaining validated decimal literals.
/// Names and all conditional arms are resolved before this value is created.
#[derive(Clone)]
pub struct WeakFloatValue {
    expression: FloatExpr,
    key: WeakFloatKey,
    default_type: FloatType,
    rounded_f32: std::sync::OnceLock<FloatValue>,
    rounded_f64: std::sync::OnceLock<FloatValue>,
    definition_source: Option<jai_source::SourceId>,
}
impl WeakFloatValue {
    fn new(expression: FloatExpr) -> Self {
        let key = keys::build(&expression);
        let default_type = expression.default_type();
        Self {
            expression,
            key,
            default_type,
            rounded_f32: std::sync::OnceLock::new(),
            rounded_f64: std::sync::OnceLock::new(),
            definition_source: None,
        }
    }
    pub fn request_key(&self) -> &WeakFloatKey {
        &self.key
    }
    pub fn default_type(&self) -> FloatType {
        self.default_type
    }
    pub(crate) fn with_fallback_source(&mut self, source: jai_source::SourceId) {
        self.definition_source.get_or_insert(source);
    }
    pub(crate) fn has_definition_source(&self) -> bool {
        self.definition_source.is_some()
    }
    pub fn round(&self, target: FloatType, span: Span) -> Result<FloatValue, Diagnostic> {
        let cached = match target {
            FloatType::F32 => &self.rounded_f32,
            FloatType::F64 => &self.rounded_f64,
        };
        if let Some(value) = cached.get() {
            return Ok(*value);
        }
        // The bound tree freezes integer check policy and contains no effects.
        // Only successful results are cached; a failed use keeps its own span.
        let value =
            self.expression
                .evaluate_in_source(Some(target), span, self.definition_source)?;
        let _ = cached.set(value);
        Ok(value)
    }
}
impl std::fmt::Debug for WeakFloatValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeakFloatValue")
            .field("default_type", &self.default_type)
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}
impl PartialEq for WeakFloatValue {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl Eq for WeakFloatValue {}
impl std::hash::Hash for WeakFloatValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}
/// Bind an exact decimal expression without materializing an inferred float width.
/// This is the immutable cache representation for contextual named constants.
pub fn bind_weak_float_paths(
    expression: &Expression,
    mut lookup: impl FnMut(&NamePath, Span) -> Result<Value, Diagnostic>,
) -> Result<WeakFloatValue, Diagnostic> {
    let bound = bind(expression, crate::CheckMode::Enabled, &mut lookup)?.float(expression.span)?;
    if bound.ty.is_some() {
        return Err(Diagnostic::new(
            expression.span,
            "expected a weak floating-point expression",
        ));
    }
    Ok(WeakFloatValue::new(bound))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct FloatExpr {
    pub ty: Option<FloatType>,
    kind: FloatKind,
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum FloatKind {
    Bound(std::sync::Arc<WeakFloatValue>),
    Decimal(DecimalLiteral),
    Constant(FloatValue),
    Number(NumberExpr),
    Cast(FloatType, Box<FloatExpr>),
    Negate(Box<FloatExpr>),
    Binary(FloatOp, Box<FloatExpr>, Box<FloatExpr>),
    Conditional(Box<Conditional<FloatExpr>>),
}
impl FloatExpr {
    pub(super) fn bound(value: std::sync::Arc<WeakFloatValue>) -> Self {
        Self {
            ty: None,
            kind: FloatKind::Bound(value),
        }
    }
    pub(super) fn into_value(self, span: Span) -> Result<Value, Diagnostic> {
        if self.ty.is_none() {
            Ok(Value::WeakFloat(std::sync::Arc::new(WeakFloatValue::new(
                self,
            ))))
        } else {
            self.evaluate(None, span).map(Value::Float)
        }
    }
    pub(super) fn constant(value: FloatValue) -> Self {
        Self {
            ty: Some(value.ty()),
            kind: FloatKind::Constant(value),
        }
    }
    pub(super) fn literal(value: &FloatLiteral) -> Self {
        match value {
            FloatLiteral::Decimal(value) => Self {
                ty: None,
                kind: FloatKind::Decimal(value.clone()),
            },
            FloatLiteral::Bits32(value) => Self::constant(FloatValue::F32(*value)),
            FloatLiteral::Bits64(value) => Self::constant(FloatValue::F64(*value)),
        }
    }
    pub(super) fn default_type(&self) -> FloatType {
        if let Some(ty) = self.ty {
            return ty;
        }
        match &self.kind {
            FloatKind::Bound(value) => value.default_type(),
            FloatKind::Decimal(value) => default_decimal_type(value),
            FloatKind::Negate(value) => value.default_type(),
            FloatKind::Binary(_, a, b) => common_default(a, b),
            FloatKind::Conditional(value) => common_default(&value.then_value, &value.else_value),
            _ => FloatType::F32,
        }
    }
    pub(super) fn evaluate(
        &self,
        expected: Option<FloatType>,
        span: Span,
    ) -> Result<FloatValue, Diagnostic> {
        self.evaluate_in_source(expected, span, None)
    }
    fn evaluate_in_source(
        &self,
        expected: Option<FloatType>,
        span: Span,
        definition_source: Option<jai_source::SourceId>,
    ) -> Result<FloatValue, Diagnostic> {
        let ty = self.ty.or(expected).unwrap_or_else(|| self.default_type());
        let error = |error: jai_types::FloatError| Diagnostic::new(span, error.to_string());
        let deferred_error = |error: Diagnostic| match definition_source {
            Some(source) => error.with_fallback_source(source),
            None => error,
        };
        let result = match &self.kind {
            FloatKind::Bound(value) => value.round(ty, span)?,
            FloatKind::Decimal(value) => {
                FloatValue::parse_decimal(ty, value.spelling()).map_err(error)?
            }
            FloatKind::Constant(value) => value.cast(ty),
            FloatKind::Number(value) => {
                let value = value.evaluate().map_err(deferred_error)?.number(span)?;
                let integer = Integer::checked(IntegerType::S64, value)
                    .or_else(|| Integer::checked(IntegerType::U64, value))
                    .ok_or_else(|| {
                        Diagnostic::new(span, "integer literal exceeds float conversion range")
                    })?;
                FloatValue::from_integer(ty, integer)
            }
            FloatKind::Cast(target, value) => value
                .evaluate_in_source(Some(*target), span, definition_source)?
                .cast(ty),
            FloatKind::Negate(value) => value
                .evaluate_in_source(Some(ty), span, definition_source)?
                .negate(),
            FloatKind::Binary(op, a, b) => a
                .evaluate_in_source(Some(ty), span, definition_source)?
                .binary(
                    *op,
                    b.evaluate_in_source(Some(ty), span, definition_source)?,
                )
                .map_err(error)?,
            FloatKind::Conditional(value) => {
                if value.condition.evaluate().map_err(deferred_error)? {
                    value
                        .then_value
                        .evaluate_in_source(Some(ty), span, definition_source)?
                } else {
                    value
                        .else_value
                        .evaluate_in_source(Some(ty), span, definition_source)?
                }
            }
        };
        Ok(result.cast(expected.unwrap_or(ty)))
    }
}
/// Preserve widening after an inactive arm contributed the common domain.
pub(super) fn widen(value: FloatExpr, ty: FloatType) -> FloatExpr {
    FloatExpr {
        ty: Some(ty),
        kind: FloatKind::Cast(ty, Box::new(value)),
    }
}
/// Selected exact expressions retain the domain inferred from every arm.
pub(super) fn retain_default(value: FloatExpr, default: FloatType) -> FloatExpr {
    if value.default_type() == default {
        return value;
    }
    let mut bound = WeakFloatValue::new(value);
    bound.key = keys::with_default(bound.key, default);
    bound.default_type = default;
    FloatExpr::bound(std::sync::Arc::new(bound))
}
pub fn default_decimal_type(decimal: &DecimalLiteral) -> FloatType {
    let mantissa = decimal.spelling().split(['e', 'E']).next().unwrap();
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    if digits.trim_start_matches('0').trim_end_matches('0').len() <= 7
        && decimal.round_f32().is_ok()
        && decimal.round_f64().is_ok_and(|value| {
            // Jai_Lexer::has_a_big_exponent compares the finite f64 exponent
            // with f32's *normal* range, including otherwise representable subnormals.
            // This determines the default width; contextual f32 rounding stays direct.
            let exponent = ((value.to_bits() >> 52) & 0x7ff) as i32;
            exponent == 0 || (-126..=127).contains(&(exponent - 1023))
        })
    {
        FloatType::F32
    } else {
        FloatType::F64
    }
}
pub(super) fn common_default(a: &FloatExpr, b: &FloatExpr) -> FloatType {
    if a.default_type() == FloatType::F64 || b.default_type() == FloatType::F64 {
        FloatType::F64
    } else {
        FloatType::F32
    }
}
impl Expr {
    pub(super) fn float(self, span: Span) -> Result<FloatExpr, Diagnostic> {
        match self {
            Self::Float(value) => Ok(value),
            Self::Number(value) if value.ty == NumberType::Literal => Ok(FloatExpr {
                ty: None,
                kind: FloatKind::Number(value),
            }),
            _ => Err(Diagnostic::new(
                span,
                "expected numeric floating-point constant",
            )),
        }
    }
}
pub(super) fn cast(
    ty: &TypeSyntax,
    value: Expr,
    mode: CastMode,
    span: Span,
) -> Result<Expr, Diagnostic> {
    let TypeSyntax::Builtin(BuiltinType::Float(ty)) = ty else {
        return Err(Diagnostic::new(
            span,
            "constant cast requires a builtin numeric type",
        ));
    };
    if mode == CastMode::Truncate {
        return Err(Diagnostic::new(
            span,
            "cast,trunc targeting floating-point types has no established source policy",
        ));
    }
    if matches!(mode, CastMode::Force(_)) {
        return Err(Diagnostic::new(
            span,
            "force storage casts require target-bound typed constant evaluation",
        ));
    }
    Ok(Expr::Float(FloatExpr {
        ty: Some(*ty),
        kind: FloatKind::Cast(
            *ty,
            Box::new(match value {
                Expr::Number(value) => FloatExpr {
                    ty: None,
                    kind: FloatKind::Number(value),
                },
                value => value.float(span)?,
            }),
        ),
    }))
}
pub(super) fn unary(op: &UnaryOp, value: Expr, span: Span) -> Result<Expr, Diagnostic> {
    let value = value.float(span)?;
    Ok(match op {
        UnaryOp::Positive => Expr::Float(value),
        UnaryOp::Negate => Expr::Float(FloatExpr {
            ty: value.ty,
            kind: FloatKind::Negate(Box::new(value)),
        }),
        _ => {
            return Err(Diagnostic::new(
                span,
                "operator is not defined for floating-point constants",
            ));
        }
    })
}
pub(super) fn binary(
    op: jai_syntax::BinaryOp,
    lhs: Expr,
    rhs: Expr,
    span: Span,
) -> Result<Expr, Diagnostic> {
    let (lhs, rhs) = (lhs.float(span)?, rhs.float(span)?);
    let ty = match (lhs.ty, rhs.ty) {
        (Some(FloatType::F64), _) | (_, Some(FloatType::F64)) => Some(FloatType::F64),
        (Some(ty), _) | (_, Some(ty)) => Some(ty),
        _ => None,
    };
    let relation = match Operator::from(op) {
        Operator::Relation(op) => Some(op),
        Operator::Equality(Equality::Equal) => Some(Relation::Equal),
        Operator::Equality(Equality::NotEqual) => Some(Relation::NotEqual),
        _ => None,
    };
    if let Some(relation) = relation {
        return Ok(Expr::Bool(BoolExpr::CompareFloats(
            relation,
            Box::new(lhs),
            Box::new(rhs),
            span,
        )));
    }
    let op = match Operator::from(op) {
        Operator::Integer(IntOp::Add) => FloatOp::Add,
        Operator::Integer(IntOp::Subtract) => FloatOp::Subtract,
        Operator::Integer(IntOp::Multiply) => FloatOp::Multiply,
        Operator::Integer(IntOp::Divide) => FloatOp::Divide,
        Operator::Integer(IntOp::Remainder) => FloatOp::Remainder,
        _ => {
            return Err(Diagnostic::new(
                span,
                "operator is not defined for floating-point constants",
            ));
        }
    };
    Ok(Expr::Float(FloatExpr {
        ty,
        kind: FloatKind::Binary(op, Box::new(lhs), Box::new(rhs)),
    }))
}
pub(super) fn conditional(
    condition: BoolExpr,
    yes: Expr,
    no: Option<Expr>,
    span: Span,
) -> Result<Expr, Diagnostic> {
    let yes = yes.float(span)?;
    let no = no
        .unwrap_or_else(|| literal(Value::Literal(0)))
        .float(span)?;
    let ty = match (yes.ty, no.ty) {
        (Some(FloatType::F64), _) | (_, Some(FloatType::F64)) => Some(FloatType::F64),
        (Some(ty), _) | (_, Some(ty)) => Some(ty),
        _ => None,
    };
    Ok(Expr::Float(FloatExpr {
        ty,
        kind: FloatKind::Conditional(Box::new(Conditional {
            condition,
            then_value: yes,
            else_value: no,
        })),
    }))
}
pub(super) fn to_integer(
    value: &FloatExpr,
    target: IntegerType,
    mode: CastMode,
    span: Span,
) -> Result<Integer, Diagnostic> {
    let unsupported = match mode {
        CastMode::Checked => None,
        CastMode::Unchecked => {
            Some("unchecked float-to-integer conversion has no established source policy")
        }
        CastMode::Truncate => {
            Some("cast,trunc float-to-integer conversion has no established source policy")
        }
        CastMode::Force(_) => {
            Some("force storage casts require target-bound typed constant evaluation")
        }
    };
    if let Some(message) = unsupported {
        return Err(Diagnostic::new(span, message));
    }
    value
        .evaluate(None, span)?
        .to_integer(target, FloatToIntMode::Truncate)
        .map_err(|error| Diagnostic::new(span, error.to_string()))
}
/// Bind the entire expression before evaluating; contextual decimal rounding occurs only here.
pub fn evaluate_float_paths(
    expression: &Expression,
    target: FloatType,
    mut lookup: impl FnMut(&NamePath, Span) -> Result<Value, Diagnostic>,
) -> Result<FloatValue, Diagnostic> {
    evaluate_float_paths_with_overflow_check(
        expression,
        target,
        crate::CheckMode::Enabled,
        &mut lookup,
    )
}
pub fn evaluate_float_paths_with_overflow_check(
    expression: &Expression,
    target: FloatType,
    overflow_check: crate::CheckMode,
    mut lookup: impl FnMut(&NamePath, Span) -> Result<Value, Diagnostic>,
) -> Result<FloatValue, Diagnostic> {
    let value = bind(expression, overflow_check, &mut lookup)?.float(expression.span)?;
    if value.ty == Some(FloatType::F64) && target == FloatType::F32 {
        return Err(Diagnostic::new(
            expression.span,
            "implicit floating-point conversion does not preserve the source width",
        ));
    }
    value.evaluate(Some(target), expression.span)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn expression(text: &str) -> Expression {
        let mut sources = jai_source::SourceMap::default();
        let source = sources.insert("float.jai".into(), format!("VALUE :: {text};"));
        let mut symbols = jai_source::Symbols::default();
        let file = jai_syntax::parse_file(sources.get(source).unwrap(), &mut symbols).unwrap();
        let jai_syntax::FileItem::Declaration(declaration) = &file.items()[0] else {
            panic!()
        };
        let jai_syntax::FileDeclarationKind::Constant(constant) = &declaration.kind else {
            panic!()
        };
        constant.initializer.clone()
    }
    fn lookup(_: &NamePath, span: Span) -> Result<Value, Diagnostic> {
        Err(Diagnostic::new(span, "unknown constant"))
    }
    #[test]
    fn storage_force_casts_never_become_numeric_float_conversions() {
        for text in [
            "cast,force(float32) cast(u32) 1065353216",
            "cast,FORCE(float32) cast(u64) 1065353216",
            "cast,force(u32) 1.0",
        ] {
            let error = evaluate_paths(&expression(text), lookup).unwrap_err();
            assert!(error.message.contains("storage casts"), "{error}");
        }
    }
    #[test]
    fn deferred_errors_keep_definition_origins_without_relocating_context_errors() {
        let mut sources = jai_source::SourceMap::default();
        let definition = sources.insert("definition.jai".into(), String::new());
        let alias_source = sources.insert("alias.jai".into(), String::new());
        let input = expression("ifx 1 / 0 == 0 then 1.0 else 2.0");
        let unlocated = evaluate_paths(&input, lookup).unwrap();
        let located = unlocated.clone().with_fallback_source(definition);
        assert_eq!(unlocated, located); // Origins never change exact request identity.
        let Value::WeakFloat(weak) = &located else {
            panic!()
        };
        let use_span = Span::new(100, 110);
        let error = weak.round(FloatType::F64, use_span).unwrap_err();
        assert_eq!(error.source, Some(definition));
        assert_ne!(error.span, use_span);
        assert!(weak.rounded_f64.get().is_none());
        let alias = evaluate_paths(&expression("ORIGINAL + 0.0"), |_, _| Ok(located.clone()))
            .unwrap()
            .with_fallback_source(alias_source);
        let Value::WeakFloat(alias) = alias else {
            panic!()
        };
        assert_eq!(
            alias.round(FloatType::F32, use_span).unwrap_err().source,
            Some(definition)
        );

        let contextual = evaluate_paths(&expression("1e39"), lookup)
            .unwrap()
            .with_fallback_source(definition);
        let Value::WeakFloat(contextual) = contextual else {
            panic!()
        };
        let error = contextual.round(FloatType::F32, use_span).unwrap_err();
        assert_eq!(error.source, None);
        assert_eq!(error.span, use_span);
    }
    #[test]
    fn inferred_width_uses_the_source_lexer_normal_exponent_boundary() {
        // Only normal finite f64 exponents are compared; exponent field zero
        // (zero or a subnormal f64) is explicitly exempt in that source helper.
        for (text, expected) in [
            ("0.0", FloatType::F32),
            ("1e-37", FloatType::F32),
            ("1e-38", FloatType::F64),
            ("1e-40", FloatType::F64),
            ("-1e-40", FloatType::F64),
            ("1e-310", FloatType::F32),
            ("5e-324", FloatType::F32),
            ("1e38", FloatType::F32),
            ("1e39", FloatType::F64),
        ] {
            let weak = bind_weak_float_paths(&expression(text), lookup).unwrap();
            assert_eq!(weak.default_type(), expected, "{text}");
        }
        let value = evaluate_float_paths(&expression("1e-40"), FloatType::F32, lookup).unwrap();
        assert_eq!(value, FloatValue::from_f32(1e-40_f32));
        assert_ne!(value.bits(), 0);
        assert_eq!(
            evaluate_float_paths(&expression("1e-310"), FloatType::F32, lookup).unwrap(),
            FloatValue::F32(0)
        );
    }
    #[test]
    fn repeated_named_aliases_share_exact_keys_and_successful_width_results() {
        let repeated = expression("PREVIOUS + PREVIOUS");
        let build = || {
            let mut value = evaluate_paths(&expression("0.125"), lookup).unwrap();
            for _ in 0..128 {
                value = evaluate_paths(&repeated, |_, _| Ok(value.clone())).unwrap();
            }
            let Value::WeakFloat(value) = value else {
                panic!()
            };
            value
        };
        let a = build();
        let b = build();
        // This chain denotes 2^128 repeated leaves; the retained key is a DAG.
        assert_eq!(a.request_key().node_count(), 2 + 2 * 128);
        assert_eq!(a.request_key(), b.request_key());
        let wide = a.round(FloatType::F64, Span::default()).unwrap();
        assert_eq!(wide, FloatValue::from_f64(0.125_f64 * 2.0_f64.powi(128)));
        assert_eq!(a.rounded_f64.get(), Some(&wide));
        assert!(a.rounded_f32.get().is_none());
        let narrow = a.round(FloatType::F32, Span::default()).unwrap();
        assert_eq!(narrow, wide.cast(FloatType::F32));
        assert_eq!(a.rounded_f32.get(), Some(&narrow));

        let named = evaluate_paths(&expression("NAME + NAME"), |_, _| {
            evaluate_paths(&expression("0.125"), lookup)
        })
        .unwrap();
        let direct = evaluate_paths(&expression("0.125 + 0.125"), lookup).unwrap();
        assert_eq!(named, direct);
    }
    #[test]
    fn unsuccessful_rounding_is_not_cached_or_reused_at_another_source_use() {
        let weak = bind_weak_float_paths(&expression("1e39"), lookup).unwrap();
        for span in [Span::new(10, 20), Span::new(30, 40)] {
            assert_eq!(weak.round(FloatType::F32, span).unwrap_err().span, span);
            assert!(weak.rounded_f32.get().is_none());
        }
        assert!(weak.round(FloatType::F64, Span::default()).is_ok());
        assert!(weak.rounded_f64.get().is_some());
        assert!(weak.rounded_f32.get().is_none());
    }
    #[test]
    fn contextual_constants_do_not_double_round() {
        let value = evaluate_float_paths(&expression("1.0000000596046448"), FloatType::F32, lookup)
            .unwrap();
        assert_eq!(value, FloatValue::F32(1.0_f32.to_bits() + 1));
        let value = evaluate_float_paths(
            &expression("-1.0000000596046448 + 0.0"),
            FloatType::F32,
            lookup,
        )
        .unwrap();
        assert_eq!(value, FloatValue::F32((-1.0_f32).to_bits() + 1));
    }
    #[test]
    fn exact_bound_values_round_independently_in_multiple_contexts() {
        let weak = bind_weak_float_paths(&expression("1.0000000596046448 + 0.0"), lookup).unwrap();
        assert_eq!(
            weak.round(FloatType::F32, Span::default()).unwrap(),
            FloatValue::F32(1.0_f32.to_bits() + 1)
        );
        let wide = weak.round(FloatType::F64, Span::default()).unwrap();
        assert_eq!(wide.cast(FloatType::F32), FloatValue::from_f32(1.0));
        assert!(
            bind_weak_float_paths(&expression("ifx true then 1.0 else missing"), lookup).is_err()
        );
    }
    #[test]
    fn named_weak_values_retain_exact_decimal_context_through_aliases() {
        let precise = evaluate_paths(&expression("1.0000000596046448"), lookup).unwrap();
        assert!(matches!(precise, Value::WeakFloat(_)));
        let alias =
            evaluate_paths(&expression("PRECISE + 0.0"), |_, _| Ok(precise.clone())).unwrap();
        assert!(matches!(alias, Value::WeakFloat(_)));
        let narrow = evaluate_float_paths(&expression("ALIAS"), FloatType::F32, |_, _| {
            Ok(alias.clone())
        })
        .unwrap();
        let wide = evaluate_float_paths(&expression("ALIAS"), FloatType::F64, |_, _| {
            Ok(alias.clone())
        })
        .unwrap();
        assert_eq!(narrow, FloatValue::F32(1.0_f32.to_bits() + 1));
        assert_eq!(wide.cast(FloatType::F32), FloatValue::from_f32(1.0));
    }
    #[test]
    fn exact_request_identity_ignores_diagnostic_spans_but_preserves_decimal_digits() {
        use std::hash::{Hash, Hasher};
        let a = evaluate_paths(&expression("ifx (1 + 2) == 3 then 1.0 else 2.0"), lookup).unwrap();
        let b = evaluate_paths(
            &expression("    ifx (1 + 2) == 3 then 1.0 else 2.0"),
            lookup,
        )
        .unwrap();
        assert_eq!(a, b);
        let (Value::WeakFloat(a), Value::WeakFloat(b)) = (a, b) else {
            panic!()
        };
        let hash = |value: &WeakFloatValue| {
            let mut state = std::collections::hash_map::DefaultHasher::new();
            value.hash(&mut state);
            state.finish()
        };
        assert_eq!(hash(&a), hash(&b));
        let a = bind_weak_float_paths(&expression("1.00000000000000000000000001"), lookup).unwrap();
        let b = bind_weak_float_paths(&expression("1.00000000000000000000000002"), lookup).unwrap();
        assert_eq!(
            a.round(FloatType::F64, Span::default()).unwrap(),
            b.round(FloatType::F64, Span::default()).unwrap()
        );
        assert_ne!(a.request_key(), b.request_key());
    }
    #[test]
    fn typed_operations_keep_their_width_before_a_cast() {
        let source = expression("cast(float64) (cast(float32) 16777216.0 + cast(float32) 1.0)");
        assert_eq!(
            evaluate_paths(&source, lookup).unwrap(),
            Value::Float(FloatValue::from_f64(16777216.0))
        );
        let source = expression("cast(float32) (cast(float64) 16777216.0 + cast(float64) 1.0)");
        assert_eq!(
            evaluate_paths(&source, lookup).unwrap(),
            Value::Float(FloatValue::from_f32(16777216.0))
        );
    }
    #[test]
    fn implicit_narrowing_is_rejected_but_explicit_casts_round() {
        assert!(
            evaluate_float_paths(&expression("cast(float64) 1.0"), FloatType::F32, lookup).is_err()
        );
        assert_eq!(
            evaluate_float_paths(
                &expression("cast(float32) cast(float64) 1.0"),
                FloatType::F32,
                lookup
            )
            .unwrap(),
            FloatValue::from_f32(1.0)
        );
        assert_eq!(
            evaluate_paths(
                &expression("cast(float32) 1.0 == cast(float64) 1.0"),
                lookup
            )
            .unwrap(),
            Value::Bool(true)
        );
    }
    #[test]
    fn typed_integer_conversion_requires_an_explicit_float_cast() {
        assert!(evaluate_float_paths(&expression("cast(s32) 1"), FloatType::F32, lookup).is_err());
        assert!(evaluate_paths(&expression("cast(s32) 1 + 1.0"), lookup).is_err());
        assert_eq!(
            evaluate_paths(&expression("cast(float32) cast(s32) 1"), lookup).unwrap(),
            Value::Float(FloatValue::from_f32(1.0))
        );
    }
    #[test]
    fn checked_integer_casts_truncate_before_testing_the_range() {
        for (source, ty, expected) in [
            ("cast(s8) -128.9", IntegerType::S8, -128),
            ("cast(s8) 127.9", IntegerType::S8, 127),
            ("cast(u8) -0.9", IntegerType::U8, 0),
            ("cast(u8) 255.9", IntegerType::U8, 255),
            (
                "cast(s64) 0hc3e0000000000000",
                IntegerType::S64,
                i64::MIN as i128,
            ),
            (
                "cast(u64) 0h43efffffffffffff",
                IntegerType::U64,
                18446744073709549568,
            ),
        ] {
            assert_eq!(
                evaluate_paths(&expression(source), lookup).unwrap(),
                Value::Int(Integer::checked(ty, expected).unwrap())
            );
        }
        for source in [
            "cast(s8) -129.0",
            "cast(u8) -1.0",
            "cast(u8) 256.0",
            "cast(s64) 0h7ff8000000000001",
            "cast(u64) 0h43f0000000000000",
            "cast(s64) 0h43e0000000000000",
            "cast(s64) 0hfff0000000000000",
        ] {
            assert!(
                evaluate_paths(&expression(source), lookup).is_err(),
                "{source}"
            );
        }
    }
    #[test]
    fn float_conditions_and_ieee_comparisons_are_pure() {
        for (source, expected) in [
            ("0h7fbfffff != 0h7fbfffff", true),
            ("0h8000000000000000 == 0.0", true),
            ("0h7fbfffff < 0.0", false),
            ("1.0 == 1.00000001", false),
            ("1.00000001 == 1.0", false),
            ("cast(float64) 0.0 || true", true),
            ("cast(float32) 2.0 && true", true),
            ("!0h7fbfffff", false),
        ] {
            assert_eq!(
                evaluate_paths(&expression(source), lookup).unwrap(),
                Value::Bool(expected)
            );
        }
        assert!(
            evaluate_float_paths(
                &expression("ifx true then 1.0 else missing"),
                FloatType::F32,
                lookup
            )
            .is_err()
        );
        assert_eq!(
            evaluate_float_paths(
                &expression("ifx true then 1.0 else 1e1000"),
                FloatType::F32,
                lookup
            )
            .unwrap(),
            FloatValue::from_f32(1.0)
        );
    }
}
