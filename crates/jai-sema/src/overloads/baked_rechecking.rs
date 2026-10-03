//! Recheck one accepted baked value against its actual resolved formal type.
use super::*;

pub(crate) fn recheck_baked_value(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    value: BakedValue,
    target: TypeId,
    substitution: &Substitution,
    span: Span,
) -> Result<BakedValue, Diagnostic> {
    let source = match &value {
        BakedValue::Value(value) => value.ty,
        BakedValue::Type(_) => types.meta_type(),
        BakedValue::Float(value) => types.float(value.ty()),
        BakedValue::String(_) => types
            .lookup(&TypeKind::String)
            .ok_or_else(|| Diagnostic::new(span, "string type is unavailable"))?,
        BakedValue::Code(_) => types.code_type(),
    };
    let pattern = TypePattern::Concrete(target);
    let info = ArgumentInfo::constant(value, source);
    compatible(
        types,
        nominals,
        &pattern,
        &info.ty,
        substitution,
        true,
        span,
    )?;
    let value = bake_with_nominals(types, nominals, &pattern, &info, substitution, span)?;
    // Accepted scalar conversion still must produce the exact checked formal.
    // Equal layouts or a Type descriptor never substitute for nominal identity.
    let value = value
        .into_runtime(target, types)
        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
    BakedValue::runtime(value, types).map_err(|error| Diagnostic::new(span, error.to_string()))
}
