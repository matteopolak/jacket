//! Resolve concrete names while retaining introducing and dependent type patterns.
use crate::overloads::{
    ArgumentInfo, ArgumentType, Candidate, CandidateVariadic, CountPattern, Parameter, TypePattern,
};
use jai_source::{DeclarationId, Diagnostic, Span, Symbol};
use jai_syntax::{
    self as syntax, BuiltinType, Expression, ExpressionKind, ParameterBinding, ResultBinding,
    TypeSyntax,
};
use jai_types::{IntegerType, ScalarType, TypeId};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct ResultPattern {
    pub usage: syntax::ResultUsage,
    pub name: Option<Symbol>,
    pub ty: TypePattern,
    pub default: Option<ArgumentInfo>,
}
#[derive(Clone, Debug)]
pub struct ProcedureTemplate<Origin = DeclarationId> {
    pub candidate: Candidate<Origin>,
    pub results: Vec<ResultPattern>,
    pub generic: bool,
}

pub fn is_polymorphic(procedure: &syntax::Procedure) -> bool {
    procedure.modify.is_some() || polymorphic_parameters(&procedure.parameters)
}
pub fn is_polymorphic_prototype(prototype: &syntax::ProcedurePrototype) -> bool {
    polymorphic_parameters(&prototype.parameters)
}
fn polymorphic_parameters(parameters: &[syntax::Parameter]) -> bool {
    let mut names = HashSet::new();
    let mut generic = parameters
        .iter()
        .any(|parameter| parameter.baking != syntax::ParameterBaking::None);
    for parameter in parameters {
        match &parameter.binding {
            ParameterBinding::RequiredType(ty)
            | ParameterBinding::DefaultedType { ty: Some(ty), .. } => {
                collect_variables(ty, &mut names, &mut generic);
            }
            _ => {}
        }
    }
    generic
}

/// Retain the same structural patterns while resolving a specialized body's
/// annotation against its definition's immutable substitution overlay.
pub fn substituted_pattern(
    ty: &TypeSyntax,
    substitution: &super::Substitution,
    span: Span,
    mut concrete: impl FnMut(&TypeSyntax, Span) -> Result<TypeId, Diagnostic>,
    mut count: impl FnMut(&Expression) -> Result<u64, Diagnostic>,
) -> Result<TypePattern, Diagnostic> {
    let names = substitution
        .types
        .iter()
        .map(|binding| binding.name)
        .chain(substitution.constants.iter().map(|binding| binding.name))
        .collect();
    pattern(ty, &names, span, &mut concrete, &mut count, &HashMap::new())
}

/// Callbacks resolve names/defaults in the defining file. They never inspect a
/// caller's locals or perform substitution in source text.
pub fn from_procedure<Origin: Copy>(
    declaration: Origin,
    procedure: &syntax::Procedure,
    concrete: impl FnMut(&TypeSyntax, Span) -> Result<TypeId, Diagnostic>,
    default: impl FnMut(&Expression) -> Result<ArgumentInfo, Diagnostic>,
    count: impl FnMut(&Expression) -> Result<u64, Diagnostic>,
) -> Result<ProcedureTemplate<Origin>, Diagnostic> {
    from_procedure_with_patterns(
        declaration,
        procedure,
        concrete,
        default,
        count,
        &HashMap::new(),
    )
}

/// Nominal applications are resolved to typed origin/argument patterns before
/// this pass. The span indexes the original AST; it is never source rewriting.
pub fn from_procedure_with_patterns<Origin: Copy>(
    declaration: Origin,
    procedure: &syntax::Procedure,
    concrete: impl FnMut(&TypeSyntax, Span) -> Result<TypeId, Diagnostic>,
    default: impl FnMut(&Expression) -> Result<ArgumentInfo, Diagnostic>,
    count: impl FnMut(&Expression) -> Result<u64, Diagnostic>,
    applications: &HashMap<(usize, usize), TypePattern>,
) -> Result<ProcedureTemplate<Origin>, Diagnostic> {
    from_header(
        declaration,
        &procedure.parameters,
        &procedure.results,
        procedure.convention,
        procedure.span,
        concrete,
        default,
        count,
        applications,
        procedure.modify.is_some(),
    )
}

pub fn from_prototype_with_patterns(
    declaration: DeclarationId,
    prototype: &syntax::ProcedurePrototype,
    concrete: impl FnMut(&TypeSyntax, Span) -> Result<TypeId, Diagnostic>,
    default: impl FnMut(&Expression) -> Result<ArgumentInfo, Diagnostic>,
    count: impl FnMut(&Expression) -> Result<u64, Diagnostic>,
    applications: &HashMap<(usize, usize), TypePattern>,
) -> Result<ProcedureTemplate, Diagnostic> {
    from_header(
        declaration,
        &prototype.parameters,
        &prototype.results,
        prototype.convention,
        prototype.span,
        concrete,
        default,
        count,
        applications,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
fn from_header<Origin: Copy>(
    declaration: Origin,
    source_parameters: &[syntax::Parameter],
    source_results: &[syntax::ProcedureResult],
    convention: jai_types::CallingConvention,
    span: Span,
    mut concrete: impl FnMut(&TypeSyntax, Span) -> Result<TypeId, Diagnostic>,
    mut default: impl FnMut(&Expression) -> Result<ArgumentInfo, Diagnostic>,
    mut count: impl FnMut(&Expression) -> Result<u64, Diagnostic>,
    applications: &HashMap<(usize, usize), TypePattern>,
    modifier: bool,
) -> Result<ProcedureTemplate<Origin>, Diagnostic> {
    let mut variables = HashSet::new();
    let mut generic = modifier;
    for parameter in source_parameters {
        if parameter.baking != syntax::ParameterBaking::None {
            generic = true;
            variables.insert(parameter.name);
        }
        let ty = match &parameter.binding {
            ParameterBinding::RequiredType(ty)
            | ParameterBinding::DefaultedType { ty: Some(ty), .. } => Some(ty),
            _ => None,
        };
        if let Some(ty) = ty {
            collect_variables(ty, &mut variables, &mut generic);
        }
    }
    if modifier {
        for result in source_results {
            if let ResultBinding::Typed { ty, .. } = &result.binding {
                collect_variables(ty, &mut variables, &mut generic);
            }
        }
    }
    let mut parameters = Vec::with_capacity(source_parameters.len());
    let mut variadic = CandidateVariadic::None;
    for parameter in source_parameters {
        if parameter.variadic {
            if variadic != CandidateVariadic::None {
                return Err(Diagnostic::new(
                    parameter.span,
                    "multiple variadic parameters",
                ));
            }
            variadic = if convention == jai_types::CallingConvention::C {
                CandidateVariadic::C {
                    fixed_parameters: parameters.len(),
                }
            } else {
                CandidateVariadic::Jai {
                    parameter: parameters.len(),
                }
            };
            if convention == jai_types::CallingConvention::C {
                continue;
            }
        }
        let (ty, value) = match &parameter.binding {
            ParameterBinding::Required(ty) => {
                (Some(TypeSyntax::Builtin(BuiltinType::Scalar(*ty))), None)
            }
            ParameterBinding::RequiredType(ty) => (Some(ty.clone()), None),
            ParameterBinding::Defaulted { ty, expression } => (
                ty.map(|ty| TypeSyntax::Builtin(BuiltinType::Scalar(ty))),
                Some(default(expression)?),
            ),
            ParameterBinding::DefaultedType { ty, expression } => {
                (ty.clone(), Some(default(expression)?))
            }
        };
        let ty = match ty {
            Some(ty) => pattern(
                &ty,
                &variables,
                parameter.span,
                &mut concrete,
                &mut count,
                applications,
            )?,
            None => inferred_default_type(
                value.as_ref().expect("untyped parameter has a default"),
                parameter.span,
                &mut concrete,
            )?,
        };
        parameters.push(Parameter {
            evaluation: parameter.evaluation,
            name: parameter.name,
            ty,
            default: value,
            baking: parameter.baking,
        });
    }
    let mut results = Vec::with_capacity(source_results.len());
    for result in source_results {
        let (ty, value) = match &result.binding {
            ResultBinding::Typed {
                ty,
                default: expression,
            } => (
                Some(ty.clone()),
                expression.as_ref().map(&mut default).transpose()?,
            ),
            ResultBinding::InferredDefault(expression) => (None, Some(default(expression)?)),
        };
        let ty = match ty {
            Some(ty) => {
                let mut result_variables = HashSet::new();
                let mut introduces = false;
                collect_variables(&ty, &mut result_variables, &mut introduces);
                if introduces && !modifier {
                    return Err(Diagnostic::new(
                        result.span,
                        "return type variables cannot be inferred from a call",
                    ));
                }
                pattern(
                    &ty,
                    &variables,
                    result.span,
                    &mut concrete,
                    &mut count,
                    applications,
                )?
            }
            None => inferred_default_type(
                value.as_ref().expect("inferred result has a default"),
                result.span,
                &mut concrete,
            )?,
        };
        results.push(ResultPattern {
            usage: result.usage,
            name: result.name,
            ty,
            default: value,
        });
    }
    if results.len() == 1
        && let TypePattern::Concrete(result_type) = results[0].ty
        && result_type
            == concrete(
                &TypeSyntax::Builtin(BuiltinType::Void),
                source_results[0].span,
            )?
    {
        results.clear();
    }
    let candidate = Candidate {
        declaration,
        parameters,
        variadic,
    };
    crate::overloads::validate_candidate(&candidate, span)?;
    Ok(ProcedureTemplate {
        candidate,
        results,
        generic,
    })
}

fn inferred_default_type(
    value: &ArgumentInfo,
    span: Span,
    concrete: &mut impl FnMut(&TypeSyntax, Span) -> Result<TypeId, Diagnostic>,
) -> Result<TypePattern, Diagnostic> {
    Ok(TypePattern::Concrete(match &value.ty {
        ArgumentType::ContextualCast { .. } | ArgumentType::ContextualProcedure { .. } => {
            return Err(Diagnostic::new(
                span,
                "contextual cast default requires an explicit parameter type",
            ));
        }
        ArgumentType::Null => {
            return Err(Diagnostic::new(
                span,
                "null default requires an explicit pointer type",
            ));
        }
        ArgumentType::EnumMember(_) => {
            return Err(Diagnostic::new(
                span,
                "leading-dot default requires an explicit enum type",
            ));
        }
        ArgumentType::Known(ty)
        | ArgumentType::StringLiteral(ty)
        | ArgumentType::RecordLiteral { ty: Some(ty), .. }
        | ArgumentType::ArrayLiteral {
            default: Some(ty), ..
        } => *ty,
        ArgumentType::RecordLiteral { ty: None, .. }
        | ArgumentType::ArrayLiteral { default: None, .. } => {
            return Err(Diagnostic::new(
                span,
                "aggregate default requires a concrete type",
            ));
        }
        ArgumentType::WeakInteger { .. } => concrete(
            &TypeSyntax::Builtin(BuiltinType::Scalar(ScalarType::Int(IntegerType::S64))),
            span,
        )?,
        ArgumentType::WeakFloat { default, .. }
        | ArgumentType::WeakFloatExpression { default, .. } => {
            concrete(&TypeSyntax::Builtin(BuiltinType::Float(*default)), span)?
        }
    }))
}
fn collect_variables(ty: &TypeSyntax, variables: &mut HashSet<Symbol>, generic: &mut bool) {
    match ty {
        TypeSyntax::Restricted { variable, .. } => {
            variables.insert(*variable);
            *generic = true;
        }
        TypeSyntax::Variable(name) => {
            variables.insert(*name);
            *generic = true;
        }
        TypeSyntax::Pointer(ty) | TypeSyntax::Slice(ty) | TypeSyntax::DynamicArray(ty) => {
            collect_variables(ty, variables, generic)
        }
        TypeSyntax::FixedArray { element, count } => {
            collect_variables(element, variables, generic);
            if let ExpressionKind::CompileVariable(name) = count.kind {
                variables.insert(name);
                *generic = true;
            }
        }
        TypeSyntax::Procedure(procedure) => {
            for parameter in procedure.parameters.iter().chain(&procedure.results) {
                collect_variables(&parameter.ty, variables, generic);
            }
        }
        TypeSyntax::Application(application) => {
            for argument in &application.arguments {
                collect_expression_variables(&argument.value, variables, generic);
            }
        }
        TypeSyntax::TypeOf(expression) => {
            collect_expression_variables(expression, variables, generic)
        }
        _ => {}
    }
}
fn collect_expression_variables(
    expression: &Expression,
    variables: &mut HashSet<Symbol>,
    generic: &mut bool,
) {
    match &expression.kind {
        ExpressionKind::CompileVariable(name) => {
            variables.insert(*name);
            *generic = true;
        }
        ExpressionKind::Type(ty) => collect_variables(ty, variables, generic),
        ExpressionKind::Call(_, arguments) | ExpressionKind::QualifiedCall(_, arguments) => {
            for argument in arguments {
                collect_expression_variables(&argument.value, variables, generic);
            }
        }
        ExpressionKind::AddressOf(inner)
        | ExpressionKind::Dereference(inner)
        | ExpressionKind::Unary(_, inner)
        | ExpressionKind::Cast(_, _, inner)
        | ExpressionKind::Member { base: inner, .. }
        | ExpressionKind::TypeQuery { value: inner, .. }
        | ExpressionKind::CallHint { call: inner, .. }
        | ExpressionKind::InferredCast { value: inner, .. } => {
            collect_expression_variables(inner, variables, generic)
        }
        ExpressionKind::TypeCast { ty, value, .. } => {
            collect_variables(ty, variables, generic);
            collect_expression_variables(value, variables, generic);
        }
        ExpressionKind::Binary(_, left, right)
        | ExpressionKind::Index {
            base: left,
            index: right,
        } => {
            collect_expression_variables(left, variables, generic);
            collect_expression_variables(right, variables, generic);
        }
        ExpressionKind::Conditional(conditional) => {
            collect_expression_variables(&conditional.condition, variables, generic);
            collect_expression_variables(&conditional.then_value, variables, generic);
            if let Some(value) = &conditional.else_value {
                collect_expression_variables(value, variables, generic);
            }
        }
        ExpressionKind::ArrayLiteral(array) => {
            if let Some(ty) = &array.element_type {
                collect_variables(ty, variables, generic);
            }
            for value in &array.elements {
                collect_expression_variables(value, variables, generic);
            }
        }
        _ => {}
    }
}

fn pattern(
    ty: &TypeSyntax,
    variables: &HashSet<Symbol>,
    span: Span,
    concrete: &mut impl FnMut(&TypeSyntax, Span) -> Result<TypeId, Diagnostic>,
    count: &mut impl FnMut(&Expression) -> Result<u64, Diagnostic>,
    applications: &HashMap<(usize, usize), TypePattern>,
) -> Result<TypePattern, Diagnostic> {
    if let Some(pattern) = applications.get(&(span.start, span.end)) {
        return Ok(pattern.clone());
    }
    Ok(match ty {
        TypeSyntax::This => {
            return Err(Diagnostic::new(
                span,
                "#this is not allowed in procedure headers",
            ));
        }
        TypeSyntax::Restricted {
            variable,
            restriction,
            span: restriction_span,
        } => TypePattern::Restricted {
            ty: Box::new(TypePattern::Infer(*variable)),
            restriction: match restriction {
                syntax::TypeRestrictionSyntax::Nominal(ty) => {
                    crate::overloads::TypeRestrictionPattern::Nominal(Box::new(pattern(
                        ty,
                        variables,
                        *restriction_span,
                        concrete,
                        count,
                        applications,
                    )?))
                }
                syntax::TypeRestrictionSyntax::Interface(ty) => {
                    crate::overloads::TypeRestrictionPattern::Interface(concrete(
                        ty,
                        *restriction_span,
                    )?)
                }
            },
        },
        TypeSyntax::Variable(name) => TypePattern::Infer(*name),
        TypeSyntax::Named(path) if path.members.is_empty() && variables.contains(&path.root) => {
            TypePattern::Variable(path.root)
        }
        TypeSyntax::Pointer(ty) => TypePattern::Pointer(Box::new(pattern(
            ty,
            variables,
            span,
            concrete,
            count,
            applications,
        )?)),
        TypeSyntax::Slice(ty) => TypePattern::Slice(Box::new(pattern(
            ty,
            variables,
            span,
            concrete,
            count,
            applications,
        )?)),
        TypeSyntax::DynamicArray(ty) => TypePattern::DynamicArray(Box::new(pattern(
            ty,
            variables,
            span,
            concrete,
            count,
            applications,
        )?)),
        TypeSyntax::FixedArray {
            element,
            count: expression,
        } => {
            let resolved_count = match &expression.kind {
                ExpressionKind::CompileVariable(name) => CountPattern::Infer(*name),
                ExpressionKind::Name(name) if variables.contains(name) => {
                    CountPattern::Variable(*name)
                }
                _ => CountPattern::Exact(count(expression)?),
            };
            TypePattern::FixedArray {
                element: Box::new(pattern(
                    element,
                    variables,
                    span,
                    concrete,
                    count,
                    applications,
                )?),
                count: resolved_count,
            }
        }
        TypeSyntax::Application(application)
            if applications.contains_key(&(application.span.start, application.span.end)) =>
        {
            applications[&(application.span.start, application.span.end)].clone()
        }
        TypeSyntax::Procedure(procedure)
            if procedure
                .parameters
                .iter()
                .chain(&procedure.results)
                .any(|parameter| contains_variable(&parameter.ty, variables)) =>
        {
            crate::overloads::procedure_pattern(procedure, span, |ty, span| {
                pattern(ty, variables, span, concrete, count, applications)
            })?
        }
        _ => TypePattern::Concrete(concrete(ty, span)?),
    })
}
fn contains_variable(ty: &TypeSyntax, variables: &HashSet<Symbol>) -> bool {
    match ty {
        TypeSyntax::Restricted { .. } => true,
        TypeSyntax::Variable(_) => true,
        TypeSyntax::Named(path) => path.members.is_empty() && variables.contains(&path.root),
        TypeSyntax::Pointer(ty) | TypeSyntax::Slice(ty) | TypeSyntax::DynamicArray(ty) => {
            contains_variable(ty, variables)
        }
        TypeSyntax::FixedArray { element, count } => {
            contains_variable(element, variables)
                || matches!(count.kind, ExpressionKind::CompileVariable(_))
                || matches!(count.kind, ExpressionKind::Name(name) if variables.contains(&name))
        }
        TypeSyntax::Procedure(procedure) => procedure
            .parameters
            .iter()
            .chain(&procedure.results)
            .any(|parameter| contains_variable(&parameter.ty, variables)),
        TypeSyntax::Application(application) => application
            .arguments
            .iter()
            .any(|argument| expression_mentions_variable(&argument.value, variables)),
        TypeSyntax::TypeOf(expression) => expression_mentions_variable(expression, variables),
        _ => false,
    }
}

fn expression_mentions_variable(expression: &Expression, variables: &HashSet<Symbol>) -> bool {
    match &expression.kind {
        ExpressionKind::CompileVariable(_) => true,
        ExpressionKind::Name(name) => variables.contains(name),
        ExpressionKind::QualifiedName(path) => variables.contains(&path.root),
        ExpressionKind::Type(ty) => contains_variable(ty, variables),
        ExpressionKind::Call(_, arguments) | ExpressionKind::QualifiedCall(_, arguments) => {
            arguments
                .iter()
                .any(|argument| expression_mentions_variable(&argument.value, variables))
        }
        ExpressionKind::AddressOf(value)
        | ExpressionKind::Dereference(value)
        | ExpressionKind::Unary(_, value)
        | ExpressionKind::Cast(_, _, value)
        | ExpressionKind::Member { base: value, .. }
        | ExpressionKind::TypeQuery { value, .. }
        | ExpressionKind::CallHint { call: value, .. }
        | ExpressionKind::InferredCast { value, .. } => {
            expression_mentions_variable(value, variables)
        }
        ExpressionKind::TypeCast { ty, value, .. } => {
            contains_variable(ty, variables) || expression_mentions_variable(value, variables)
        }
        ExpressionKind::Binary(_, left, right)
        | ExpressionKind::Index {
            base: left,
            index: right,
        } => {
            expression_mentions_variable(left, variables)
                || expression_mentions_variable(right, variables)
        }
        ExpressionKind::Conditional(conditional) => {
            expression_mentions_variable(&conditional.condition, variables)
                || expression_mentions_variable(&conditional.then_value, variables)
                || conditional
                    .else_value
                    .as_ref()
                    .is_some_and(|value| expression_mentions_variable(value, variables))
        }
        ExpressionKind::ArrayLiteral(array) => {
            array
                .element_type
                .as_ref()
                .is_some_and(|ty| contains_variable(ty, variables))
                || array
                    .elements
                    .iter()
                    .any(|value| expression_mentions_variable(value, variables))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::{Identities, SourceMap, Symbols};
    use jai_syntax::{FileDeclarationKind, FileItem};
    use jai_types::TypeRegistry;

    fn template(source: &str) -> Result<ProcedureTemplate, Diagnostic> {
        let mut sources = SourceMap::default();
        let id = sources.insert("generic.jai".into(), source.into());
        let mut symbols = Symbols::default();
        let parsed = syntax::parse_file(sources.get(id).unwrap(), &mut symbols)
            .map_err(|error| Diagnostic::at_source(error.location, error.message))?;
        let procedure = parsed
            .items()
            .iter()
            .find_map(|item| match item {
                FileItem::Declaration(declaration) => match &declaration.kind {
                    FileDeclarationKind::Procedure(procedure) => Some(procedure),
                    _ => None,
                },
                _ => None,
            })
            .unwrap();
        let types = TypeRegistry::new();
        from_procedure(
            Identities::default().declaration(),
            procedure,
            |ty, span| match ty {
                TypeSyntax::Builtin(BuiltinType::Scalar(ty)) => Ok(types.scalar(*ty)),
                TypeSyntax::Builtin(BuiltinType::Type) => Ok(types.meta_type()),
                _ => Err(Diagnostic::new(span, "unknown concrete test type")),
            },
            |expression| {
                jai_eval::evaluate(expression, |_, span| {
                    Err(Diagnostic::new(span, "unknown test constant"))
                })
                .map(|value| ArgumentInfo::scalar_constant(value, &types))
            },
            |expression| match jai_eval::evaluate(expression, |_, span| {
                Err(Diagnostic::new(span, "unknown test count"))
            })? {
                jai_eval::Value::Literal(value) => u64::try_from(value)
                    .map_err(|_| Diagnostic::new(expression.span, "invalid test count")),
                _ => Err(Diagnostic::new(expression.span, "invalid test count")),
            },
        )
    }
    #[test]
    fn basic_min_signature_retains_introduction_and_uses_instead_of_text_rewriting() {
        let template =
            template("min :: (a: $T, b: T) -> T { if a < b return a; return b; }").unwrap();
        assert!(template.generic);
        let TypePattern::Infer(t) = template.candidate.parameters[0].ty else {
            panic!("expected introduction");
        };
        assert_eq!(
            template.candidate.parameters[1].ty,
            TypePattern::Variable(t)
        );
        assert_eq!(template.results[0].ty, TypePattern::Variable(t));
    }
    #[test]
    fn array_count_and_element_introductions_stay_structural() {
        let template =
            template("first :: (values: [$N] $T, fallback: T = 0) -> T { return fallback; }")
                .unwrap();
        let TypePattern::FixedArray { element, count } = &template.candidate.parameters[0].ty
        else {
            panic!("expected array pattern");
        };
        assert!(matches!(**element, TypePattern::Infer(_)));
        assert!(matches!(count, CountPattern::Infer(_)));
        assert!(matches!(
            template.candidate.parameters[1]
                .default
                .as_ref()
                .unwrap()
                .ty,
            ArgumentType::WeakInteger {
                minimum: 0,
                maximum: 0
            }
        ));
    }
    #[test]
    fn duplicate_introductions_and_result_only_inference_are_diagnosed() {
        assert!(
            template("bad :: (a:$T, b:$T) {}")
                .unwrap_err()
                .message
                .contains("introduced more than once")
        );
        assert!(
            template("bad :: () -> $T { return 3; }")
                .unwrap_err()
                .message
                .contains("return type variables")
        );
    }
}
