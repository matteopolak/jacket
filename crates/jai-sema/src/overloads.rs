//! Candidate matching does not evaluate source expressions or mutate type arenas.
use crate::polymorphism::{BakedValue, Substitution};
use jai_source::{DeclarationId, Diagnostic, Span, Symbol};
use jai_types::{
    CastMode, FloatType, FloatValue, Integer, IntegerType, ScalarType, TypeId, TypeKind, TypeView,
};
use std::collections::HashSet;
mod baked_rechecking;
mod result_types;
pub(crate) use baked_rechecking::recheck_baked_value;
pub(crate) use result_types::collect as collect_result_type_parameters;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypePattern {
    Concrete(TypeId),
    /// The sole introducing occurrence of `$T`.
    Infer(Symbol),
    /// A use of the type variable `T`, which never determines its binding.
    Variable(Symbol),
    Restricted {
        ty: Box<TypePattern>,
        restriction: TypeRestrictionPattern,
    },
    Pointer(Box<TypePattern>),
    FixedArray {
        element: Box<TypePattern>,
        count: CountPattern,
    },
    Slice(Box<TypePattern>),
    DynamicArray(Box<TypePattern>),
    Procedure(Box<ProcedurePattern>),
    NominalApplication {
        declaration: DeclarationId,
        arguments: Vec<NominalArgumentPattern>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeRestrictionPattern {
    Nominal(Box<TypePattern>),
    Interface(TypeId),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcedurePattern {
    pub parameters: Vec<TypePattern>,
    pub results: Vec<TypePattern>,
    pub convention: jai_types::CallingConvention,
    pub return_abi: jai_types::ForeignReturnAbi,
    pub context: jai_types::ContextMode,
    pub variadic: CandidateVariadic,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NominalArgumentPattern {
    pub name: Symbol,
    pub kind: NominalArgumentKind,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NominalArgumentKind {
    Type(Box<TypePattern>),
    InferValue(Symbol),
    ValueVariable(Symbol),
    Value(BakedValue),
    /// The defining template's default, normalized at this instance's preceding arguments.
    Default,
}
/// Pure semantic origin lookup; a record's field shape does not establish identity.
pub trait NominalView {
    fn specialization(&self, ty: TypeId) -> Option<(DeclarationId, &Substitution)>;
    fn layout_policy(&self) -> Option<jai_types::LayoutPolicy> {
        None
    }
    fn default_argument(&self, _: TypeId, _: Symbol) -> Option<&BakedValue> {
        None
    }
    fn enum_member(&self, _: TypeId, _: Symbol) -> Option<Integer> {
        None
    }
    fn enum_flags(&self, _: TypeId) -> bool {
        false
    }
    fn implicit_conversion(&self, _: TypeId, _: TypeId, _: Span) -> Result<bool, Diagnostic> {
        Ok(false)
    }
    fn nominal_ancestor(
        &self,
        actual: TypeId,
        required: TypeId,
        _: Span,
    ) -> Result<bool, Diagnostic> {
        Ok(actual == required)
    }
    fn interface_member(
        &self,
        _: TypeId,
        _: Symbol,
        _: Span,
    ) -> Result<Option<TypeId>, Diagnostic> {
        Ok(None)
    }
    fn symbol_name(&self, _: Symbol) -> Option<&str> {
        None
    }
    fn record_fields(&self, _: TypeId, _: Span) -> Result<Vec<LiteralField>, Diagnostic> {
        Err(Diagnostic::new(
            Span::default(),
            "record literal metadata is unavailable",
        ))
    }
    fn record_field_path(
        &self,
        ty: TypeId,
        name: Symbol,
        span: Span,
    ) -> Result<Vec<jai_types::FieldId>, Diagnostic> {
        let fields = self.record_fields(ty, span)?;
        let mut matches = fields.iter().filter(|field| field.name == Some(name));
        let field = matches
            .next()
            .ok_or_else(|| Diagnostic::new(span, "unknown record literal field"))?;
        if matches.next().is_some() {
            return Err(Diagnostic::new(span, "ambiguous record literal field"));
        }
        Ok(vec![field.id])
    }
    /// Only an actual source initializer or override supplies an inherited overlay.
    fn field_construction_overlay(
        &self,
        _: jai_types::FieldId,
        _: Span,
    ) -> Result<Option<jai_ir::ConstantValue>, Diagnostic> {
        Ok(None)
    }
    fn literal_element_default(
        &self,
        _: jai_types::FieldId,
        _: TypeId,
        span: Span,
    ) -> Result<jai_ir::ConstantValue, Diagnostic> {
        Err(Diagnostic::new(
            span,
            "literal element default requires its original field environment",
        ))
    }
    fn field_default(
        &self,
        _: jai_types::FieldId,
        span: Span,
    ) -> Result<jai_ir::ConstantValue, Diagnostic> {
        Err(Diagnostic::new(
            span,
            "record literal default is unavailable",
        ))
    }
}
struct NoNominals;
impl NominalView for NoNominals {
    fn specialization(&self, _: TypeId) -> Option<(DeclarationId, &Substitution)> {
        None
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CountPattern {
    Exact(u64),
    Infer(Symbol),
    Variable(Symbol),
}
#[derive(Clone, Debug)]
pub struct Parameter {
    pub evaluation: jai_syntax::ParameterEvaluation,
    pub name: Symbol,
    pub ty: TypePattern,
    /// A pure default description, resolved in the declaration's scope.
    pub default: Option<ArgumentInfo>,
    pub baking: jai_syntax::ParameterBaking,
}
impl Parameter {
    pub(crate) fn is_baked(&self, substitution: &Substitution) -> bool {
        self.baking != jai_syntax::ParameterBaking::None
            && substitution.constant(self.name).is_some()
    }
}
#[derive(Clone, Debug)]
pub struct Candidate<Origin = DeclarationId> {
    pub declaration: Origin,
    pub parameters: Vec<Parameter>,
    /// Source result introductions never occupy runtime formal positions.
    pub result_type_parameters: Vec<ResultTypeParameter>,
    pub variadic: CandidateVariadic,
}
#[derive(Clone, Debug)]
pub struct ResultTypeParameter {
    pub name: Symbol,
    pub result: usize,
    pub span: Span,
    pub pattern: TypePattern,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResultTypeArgument {
    pub parameter: usize,
    pub argument: usize,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CandidateVariadic {
    #[default]
    None,
    Jai {
        parameter: usize,
    },
    C {
        fixed_parameters: usize,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArgumentType {
    /// A source lambda has been checked independently in each concrete callback context.
    ContextualProcedure {
        compatible_signatures: Box<[TypeId]>,
    },
    ContextualCast {
        mode: CastMode,
        value: Box<ArgumentInfo>,
    },
    /// Null acquires a pointer or procedure context and never introduces a type variable.
    Null,
    EnumMember(Symbol),
    RecordLiteral {
        ty: Option<TypeId>,
        fields: Vec<RecordArgumentField>,
    },
    ArrayLiteral {
        explicit: Option<TypeId>,
        default: Option<TypeId>,
        elements: Vec<ArgumentInfo>,
    },
    /// Only an authored quoted string can provide implicit NUL-terminated byte backing.
    StringLiteral(TypeId),
    Known(TypeId),
    /// Includes weak conditional leaves; it does not imply a constant expression.
    WeakInteger {
        minimum: i128,
        maximum: i128,
    },
    WeakFloat {
        spelling: Box<str>,
        negative: bool,
        default: FloatType,
    },
    WeakFloatExpression {
        default: FloatType,
        permits_f32: bool,
        permits_f64: bool,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordArgumentField {
    pub target: RecordArgumentTarget,
    pub value: ArgumentInfo,
    pub span: Span,
}
#[derive(Clone, Copy, Debug)]
pub struct LiteralField {
    pub name: Option<Symbol>,
    pub id: jai_types::FieldId,
    pub ty: TypeId,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConstantArgument {
    Null,
    EnumMember(Symbol),
    /// A source default evaluated at the eventual call site, never baked.
    CallerLocation,
    /// Compiler-only absence of quoted source, never a runtime null value.
    CodeNull,
    /// A checked definition-site path read only when the argument is omitted.
    RuntimeRead(crate::runtime_defaults::RuntimeDefaultRead),
    IntegerLiteral(i128),
    FloatLiteral {
        spelling: Box<str>,
        negative: bool,
    },
    FloatExpression {
        f32: Option<FloatValue>,
        f64: Option<FloatValue>,
    },
    Value(BakedValue),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArgumentInfo {
    pub ty: ArgumentType,
    pub constant: Option<ConstantArgument>,
}
impl ArgumentInfo {
    pub(crate) fn is_compile_time_constant(&self) -> bool {
        match self.constant {
            Some(
                ConstantArgument::CallerLocation
                | ConstantArgument::CodeNull
                | ConstantArgument::RuntimeRead(_),
            ) => false,
            Some(_) => true,
            None => match &self.ty {
                ArgumentType::ContextualCast {
                    value, ..
                } => value.is_compile_time_constant(),
                ArgumentType::RecordLiteral {
                    fields, ..
                } => fields
                    .iter()
                    .all(|field| field.value.is_compile_time_constant()),
                ArgumentType::ArrayLiteral {
                    elements, ..
                } => elements.iter().all(ArgumentInfo::is_compile_time_constant),
                _ => false,
            },
        }
    }
    pub fn contextual_cast(mode: CastMode, value: ArgumentInfo) -> Self {
        Self {
            ty: ArgumentType::ContextualCast {
                mode,
                value: Box::new(value),
            },
            constant: None,
        }
    }
    pub fn null() -> Self {
        Self {
            ty: ArgumentType::Null,
            constant: Some(ConstantArgument::Null),
        }
    }
    pub fn enum_member(name: Symbol) -> Self {
        Self {
            ty: ArgumentType::EnumMember(name),
            constant: Some(ConstantArgument::EnumMember(name)),
        }
    }
    pub fn caller_location(ty: TypeId) -> Self {
        Self {
            ty: ArgumentType::Known(ty),
            constant: Some(ConstantArgument::CallerLocation),
        }
    }
    pub fn code_null(ty: TypeId) -> Self {
        Self {
            ty: ArgumentType::Known(ty),
            constant: Some(ConstantArgument::CodeNull),
        }
    }
    pub(crate) fn runtime_read(read: crate::runtime_defaults::RuntimeDefaultRead) -> Self {
        Self {
            ty: ArgumentType::Known(read.ty()),
            constant: Some(ConstantArgument::RuntimeRead(read)),
        }
    }
    pub fn typed(ty: TypeId) -> Self {
        Self {
            ty: ArgumentType::Known(ty),
            constant: None,
        }
    }
    pub fn integer_literal(value: i128) -> Self {
        Self {
            ty: ArgumentType::WeakInteger {
                minimum: value,
                maximum: value,
            },
            constant: Some(ConstantArgument::IntegerLiteral(value)),
        }
    }
    pub fn decimal_literal(spelling: Box<str>, negative: bool, default: FloatType) -> Self {
        Self {
            ty: ArgumentType::WeakFloat {
                spelling: spelling.clone(),
                negative,
                default,
            },
            constant: Some(ConstantArgument::FloatLiteral {
                spelling,
                negative,
            }),
        }
    }
    pub fn constant(value: BakedValue, ty: TypeId) -> Self {
        Self {
            ty: ArgumentType::Known(ty),
            constant: Some(ConstantArgument::Value(value)),
        }
    }
    pub(crate) fn string_literal(bytes: Box<[u8]>, ty: TypeId) -> Self {
        Self {
            ty: ArgumentType::StringLiteral(ty),
            constant: Some(ConstantArgument::Value(BakedValue::String(bytes))),
        }
    }
    pub fn scalar_constant(value: jai_eval::Value, types: &dyn TypeView) -> Self {
        match value {
            jai_eval::Value::Literal(value) => Self::integer_literal(value),
            jai_eval::Value::Int(value) => Self::constant(
                BakedValue::integer(value, types),
                types.scalar(ScalarType::Int(value.ty())),
            ),
            jai_eval::Value::Bool(value) => {
                let ty = types.scalar(ScalarType::Bool);
                Self::constant(
                    BakedValue::Value(jai_ir::ConstantValue {
                        ty,
                        kind: jai_ir::ConstantKind::Bool(value),
                    }),
                    ty,
                )
            }
            jai_eval::Value::Float(value) => {
                Self::constant(BakedValue::Float(value), types.float(value.ty()))
            }
            jai_eval::Value::WeakFloat(value) => {
                let f32 = value.round(FloatType::F32, Span::default()).ok();
                let f64 = value.round(FloatType::F64, Span::default()).ok();
                Self {
                    ty: ArgumentType::WeakFloatExpression {
                        default: value.default_type(),
                        permits_f32: f32.is_some(),
                        permits_f64: f64.is_some(),
                    },
                    constant: Some(ConstantArgument::FloatExpression {
                        f32,
                        f64,
                    }),
                }
            }
        }
    }

    /// Extract a resolved scalar fact without choosing a width for weak floats.
    pub(crate) fn scalar_value(&self) -> Option<jai_eval::Value> {
        match self.constant.as_ref()? {
            ConstantArgument::IntegerLiteral(value) => Some(jai_eval::Value::Literal(*value)),
            ConstantArgument::Value(BakedValue::Float(value)) => {
                Some(jai_eval::Value::Float(*value))
            }
            ConstantArgument::Value(BakedValue::Value(value)) => match &value.kind {
                jai_ir::ConstantKind::Int(value) => Some(jai_eval::Value::Int(*value)),
                jai_ir::ConstantKind::Bool(value) => Some(jai_eval::Value::Bool(*value)),
                jai_ir::ConstantKind::Float(value) => Some(jai_eval::Value::Float(*value)),
                _ => None,
            },
            _ => None,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Argument {
    pub name: Option<Symbol>,
    pub spread: bool,
    pub info: ArgumentInfo,
    pub span: Span,
}

/// ABI destinations retain the caller's original evaluation order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArgumentBinding {
    pub argument: usize,
    pub parameter: usize,
    /// Baked parameters do not occupy runtime argument slots.
    pub runtime_parameter: Option<usize>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConversionRank {
    Exact,
    Literal,
    Widening,
    ArrayView,
    Boxing,
}
#[derive(Clone, Debug)]
pub struct Match<Origin = DeclarationId> {
    pub declaration: Origin,
    pub substitution: Substitution,
    pub bindings: Vec<ArgumentBinding>,
    pub defaults: Vec<usize>,
    pub result_type_arguments: Vec<ResultTypeArgument>,
    /// One rank per explicitly supplied argument, in source order.
    pub conversions: Vec<ConversionRank>,
    pub variadic: bool,
}
#[derive(Clone, Debug)]
pub struct Rejection<Origin = DeclarationId> {
    pub declaration: Origin,
    pub diagnostic: Diagnostic,
}
#[derive(Clone, Debug)]
pub enum SelectionError<Origin = DeclarationId> {
    NoMatch(Vec<Rejection<Origin>>),
    Ambiguous(Vec<Origin>),
}
impl<Origin> SelectionError<Origin> {
    pub fn diagnostic(&self, span: Span) -> Diagnostic {
        match self {
            Self::NoMatch(rejections) if rejections.len() == 1 => rejections[0].diagnostic.clone(),
            Self::NoMatch(rejections) => Diagnostic::new(
                span,
                format!(
                    "no overload matches the arguments ({} candidates rejected)",
                    rejections.len()
                ),
            ),
            Self::Ambiguous(declarations) => Diagnostic::new(
                span,
                format!(
                    "ambiguous procedure call ({} matching overloads)",
                    declarations.len()
                ),
            ),
        }
    }
}

pub fn select<Origin: Copy>(
    types: &dyn TypeView,
    candidates: &[Candidate<Origin>],
    arguments: &[Argument],
    span: Span,
) -> Result<Match<Origin>, SelectionError<Origin>> {
    select_with_nominals(types, &NoNominals, candidates, arguments, span)
}
pub fn select_with_nominals<Origin: Copy>(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    candidates: &[Candidate<Origin>],
    arguments: &[Argument],
    span: Span,
) -> Result<Match<Origin>, SelectionError<Origin>> {
    let mut matches = Vec::new();
    let mut rejections = Vec::new();
    for candidate in candidates {
        match match_candidate_with_nominals(types, nominals, candidate, arguments, span) {
            Ok(found) => matches.push(found),
            Err(diagnostic) => rejections.push(Rejection {
                declaration: candidate.declaration,
                diagnostic,
            }),
        }
    }
    if matches.is_empty() {
        return Err(SelectionError::NoMatch(rejections));
    }
    select_matches(matches)
}

/// Rank already checked candidates, including accepted modifier refinements.
pub fn select_matches<Origin: Copy>(
    mut matches: Vec<Match<Origin>>,
) -> Result<Match<Origin>, SelectionError<Origin>> {
    if matches.is_empty() {
        return Err(SelectionError::NoMatch(Vec::new()));
    }
    // Pareto dominance avoids depending on declaration or parameter ordering.
    // Generic inference is an exact match, not a penalty that would silently
    // hide the ambiguity described in the polymorphism reference.
    let survivors: Vec<_> = matches
        .iter()
        .enumerate()
        .filter_map(|(index, candidate)| {
            (!matches
                .iter()
                .enumerate()
                .any(|(other, challenger)| other != index && dominates(challenger, candidate)))
            .then_some(index)
        })
        .collect();
    if survivors.len() != 1 {
        return Err(SelectionError::Ambiguous(
            survivors
                .iter()
                .map(|&index| matches[index].declaration)
                .collect(),
        ));
    }
    Ok(matches.swap_remove(survivors[0]))
}
fn dominates<Origin>(left: &Match<Origin>, right: &Match<Origin>) -> bool {
    left.conversions
        .iter()
        .zip(&right.conversions)
        .all(|(left, right)| left <= right)
        && (left
            .conversions
            .iter()
            .zip(&right.conversions)
            .any(|(left, right)| left < right)
            || (!left.variadic && right.variadic))
}

pub fn match_candidate<Origin: Copy>(
    types: &dyn TypeView,
    candidate: &Candidate<Origin>,
    arguments: &[Argument],
    span: Span,
) -> Result<Match<Origin>, Diagnostic> {
    match_candidate_with_nominals(types, &NoNominals, candidate, arguments, span)
}
pub fn match_candidate_with_nominals<Origin: Copy>(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    candidate: &Candidate<Origin>,
    arguments: &[Argument],
    span: Span,
) -> Result<Match<Origin>, Diagnostic> {
    let PreparedArguments {
        bindings,
        defaults,
        substitution,
        result_type_arguments,
    } = prepare_candidate_arguments(types, nominals, candidate, arguments, span)?;
    let conversions = check_arguments(
        types,
        nominals,
        candidate,
        arguments,
        &bindings,
        &substitution,
        span,
    )?;
    Ok(Match {
        declaration: candidate.declaration,
        substitution,
        bindings,
        defaults,
        result_type_arguments,
        conversions,
        variadic: candidate.variadic != CandidateVariadic::None,
    })
}

/// Type facts for contextual callback preview; this is not an applicability proof.
pub(crate) struct PreparedArguments {
    pub bindings: Vec<ArgumentBinding>,
    pub defaults: Vec<usize>,
    pub substitution: Substitution,
    pub result_type_arguments: Vec<ResultTypeArgument>,
}
pub(crate) fn prepare_candidate_arguments<Origin>(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    candidate: &Candidate<Origin>,
    arguments: &[Argument],
    span: Span,
) -> Result<PreparedArguments, Diagnostic> {
    validate_candidate(candidate, span)?;
    let (mut bindings, defaults, result_type_arguments) =
        bind_arguments(candidate, arguments, span)?;
    let mut substitution = Substitution::default();
    result_types::bind(
        types,
        nominals,
        candidate,
        arguments,
        &result_type_arguments,
        &mut substitution,
    )?;
    // Visit parameters in declaration order. Only introducing occurrences bind.
    // This pass is separate from compatibility so `T` may precede `$T`.
    for (parameter_index, parameter) in candidate.parameters.iter().enumerate() {
        if matches!(candidate.variadic, CandidateVariadic::Jai { parameter } if parameter == parameter_index)
            && !bindings
                .iter()
                .any(|binding| binding.parameter == parameter_index)
        {
            continue;
        }
        let info = info_for(parameter_index, parameter, &bindings, arguments);
        if parameter.baking != jai_syntax::ParameterBaking::None
            && let Some(ConstantArgument::Value(BakedValue::Type(ty))) = &info.constant
        {
            // `$T: Type` introduces T as a type through its baked value.
            if !substitution.bind_type(parameter.name, *ty) {
                return Err(Diagnostic::new(span, "conflicting baked type argument"));
            }
        }
        let pattern = argument_pattern(
            candidate,
            parameter_index,
            &parameter.ty,
            bindings
                .iter()
                .find(|binding| binding.parameter == parameter_index),
            arguments,
        );
        infer(
            types,
            nominals,
            &pattern,
            &info.ty,
            &mut substitution,
            true,
            span,
        )?;
    }
    // Baked values are ready before checking dependent array counts, even if
    // `$N` occurs after a parameter whose type is `[N]T`.
    for (parameter_index, parameter) in candidate.parameters.iter().enumerate() {
        if parameter.baking == jai_syntax::ParameterBaking::Required
            || (parameter.baking == jai_syntax::ParameterBaking::Optional
                && info_for(parameter_index, parameter, &bindings, arguments)
                    .is_compile_time_constant())
        {
            let info = info_for(parameter_index, parameter, &bindings, arguments);
            let argument_span = bindings
                .iter()
                .find(|binding| binding.parameter == parameter_index)
                .map_or(span, |binding| arguments[binding.argument].span);
            let value = bake_with_nominals(
                types,
                nominals,
                &parameter.ty,
                info,
                &substitution,
                argument_span,
            )?;
            if !substitution.bind_constant(parameter.name, value) {
                return Err(Diagnostic::new(argument_span, "conflicting baked argument"));
            }
        }
    }
    // Contextual callback parameters may precede the arguments introducing
    // their types. Retry only those descriptions after ordinary inference.
    for (parameter_index, parameter) in candidate.parameters.iter().enumerate() {
        if let Some(binding) = bindings
            .iter()
            .find(|binding| binding.parameter == parameter_index)
            && matches!(
                arguments[binding.argument].info.ty,
                ArgumentType::ContextualProcedure { .. }
            )
        {
            infer(
                types,
                nominals,
                &parameter.ty,
                &arguments[binding.argument].info.ty,
                &mut substitution,
                true,
                arguments[binding.argument].span,
            )?;
        }
    }
    result_types::check(
        types,
        nominals,
        candidate,
        arguments,
        &result_type_arguments,
        &substitution,
    )?;
    assign_runtime_parameters(candidate, &substitution, &mut bindings);
    Ok(PreparedArguments {
        substitution,
        bindings,
        defaults,
        result_type_arguments,
    })
}
/// Validate final bindings without inferring again or rebaking original input values.
pub fn recheck_match_with_nominals<Origin: Copy>(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    candidate: &Candidate<Origin>,
    arguments: &[Argument],
    mut matched: Match<Origin>,
    span: Span,
) -> Result<Match<Origin>, Diagnostic> {
    let (mut bindings, defaults, result_type_arguments) =
        bind_arguments(candidate, arguments, span)?;
    result_types::check(
        types,
        nominals,
        candidate,
        arguments,
        &result_type_arguments,
        &matched.substitution,
    )?;
    assign_runtime_parameters(candidate, &matched.substitution, &mut bindings);
    if bindings != matched.bindings
        || defaults != matched.defaults
        || result_type_arguments != matched.result_type_arguments
    {
        return Err(Diagnostic::new(
            span,
            "refined candidate has inconsistent argument bindings",
        ));
    }
    let baked_parameters = candidate
        .parameters
        .iter()
        .filter(|parameter| {
            parameter.baking == jai_syntax::ParameterBaking::Required
                || parameter.is_baked(&matched.substitution)
        })
        .collect::<Vec<_>>();
    for parameter in baked_parameters {
        let value = matched
            .substitution
            .constant(parameter.name)
            .cloned()
            .ok_or_else(|| Diagnostic::new(span, "modifier removed a required baked binding"))?;
        let ty = match &value {
            BakedValue::Value(value) => value.ty,
            BakedValue::Type(_) => types.meta_type(),
            BakedValue::Float(value) => types.float(value.ty()),
            BakedValue::String(_) => types
                .lookup(&TypeKind::String)
                .ok_or_else(|| Diagnostic::new(span, "string type is unavailable"))?,
            BakedValue::Code(_) => types.code_type(),
        };
        compatible(
            types,
            nominals,
            &parameter.ty,
            &ArgumentType::Known(ty),
            &matched.substitution,
            true,
            span,
        )?;
        let normalized = bake_with_nominals(
            types,
            nominals,
            &parameter.ty,
            &ArgumentInfo::constant(value, ty),
            &matched.substitution,
            span,
        )?;
        matched
            .substitution
            .constants
            .iter_mut()
            .find(|binding| binding.name == parameter.name)
            .expect("required baked binding exists")
            .value = normalized;
    }
    matched.conversions = check_arguments(
        types,
        nominals,
        candidate,
        arguments,
        &matched.bindings,
        &matched.substitution,
        span,
    )?;
    Ok(matched)
}
fn check_arguments<Origin>(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    candidate: &Candidate<Origin>,
    arguments: &[Argument],
    bindings: &[ArgumentBinding],
    substitution: &Substitution,
    span: Span,
) -> Result<Vec<ConversionRank>, Diagnostic> {
    let mut conversions = vec![ConversionRank::Exact; arguments.len()];
    for (parameter_index, parameter) in candidate.parameters.iter().enumerate() {
        let explicit: Vec<_> = bindings
            .iter()
            .filter(|binding| binding.parameter == parameter_index)
            .collect();
        if explicit.is_empty() {
            if matches!(candidate.variadic, CandidateVariadic::Jai { parameter } if parameter == parameter_index)
            {
                ensure_bound(&parameter.ty, substitution, span)?;
            } else {
                compatible(
                    types,
                    nominals,
                    &parameter.ty,
                    &parameter
                        .default
                        .as_ref()
                        .expect("bound defaults checked")
                        .ty,
                    substitution,
                    true,
                    span,
                )?;
            }
        } else {
            for argument in explicit {
                let pattern = argument_pattern(
                    candidate,
                    parameter_index,
                    &parameter.ty,
                    Some(argument),
                    arguments,
                );
                conversions[argument.argument] = compatible(
                    types,
                    nominals,
                    &pattern,
                    &arguments[argument.argument].info.ty,
                    substitution,
                    true,
                    arguments[argument.argument].span,
                )?;
            }
        }
    }
    if let CandidateVariadic::C {
        fixed_parameters,
    } = candidate.variadic
    {
        for argument in bindings
            .iter()
            .filter(|binding| binding.parameter >= fixed_parameters)
        {
            let info = &arguments[argument.argument].info;
            let ty = match &info.ty {
                ArgumentType::Known(ty) => Some(*ty),
                ArgumentType::WeakInteger {
                    minimum,
                    maximum,
                } if fits(IntegerType::S64, *minimum, *maximum) => {
                    Some(types.scalar(ScalarType::Int(IntegerType::S64)))
                }
                ArgumentType::WeakFloat {
                    ..
                }
                | ArgumentType::WeakFloatExpression {
                    ..
                } => Some(types.float(FloatType::F64)),
                _ => None,
            }
            .ok_or_else(|| {
                Diagnostic::new(
                    arguments[argument.argument].span,
                    "C variadic argument requires a default scalar or pointer type",
                )
            })?;
            if !matches!(
                types.kind(ty),
                Ok(TypeKind::Integer(_)
                    | TypeKind::Bool
                    | TypeKind::Float(_)
                    | TypeKind::Pointer(_))
            ) {
                return Err(Diagnostic::new(
                    arguments[argument.argument].span,
                    "C variadic argument must be scalar or pointer",
                ));
            }
        }
    }
    Ok(conversions)
}

fn argument_pattern<Origin>(
    candidate: &Candidate<Origin>,
    index: usize,
    pattern: &TypePattern,
    binding: Option<&ArgumentBinding>,
    arguments: &[Argument],
) -> TypePattern {
    if matches!(candidate.variadic, CandidateVariadic::Jai { parameter } if parameter == index)
        && binding.is_some_and(|binding| {
            arguments[binding.argument].name.is_some() || arguments[binding.argument].spread
        })
    {
        TypePattern::Slice(Box::new(pattern.clone()))
    } else {
        pattern.clone()
    }
}

pub fn validate_candidate<Origin>(
    candidate: &Candidate<Origin>,
    span: Span,
) -> Result<(), Diagnostic> {
    match candidate.variadic {
        CandidateVariadic::Jai {
            parameter,
        } if candidate.parameters.get(parameter).is_none_or(|parameter| {
            parameter.baking != jai_syntax::ParameterBaking::None || parameter.default.is_some()
        }) =>
        {
            return Err(Diagnostic::new(
                span,
                "variadic parameter must be a runtime pack without a default",
            ));
        }
        CandidateVariadic::C {
            fixed_parameters,
        } if fixed_parameters != candidate.parameters.len() => {
            return Err(Diagnostic::new(
                span,
                "invalid fixed C variadic parameter count",
            ));
        }
        _ => {}
    }
    let mut parameters = HashSet::new();
    let mut introductions = HashSet::new();
    for parameter in &candidate.parameters {
        if !parameters.insert(parameter.name) {
            return Err(Diagnostic::new(span, "duplicate parameter name"));
        }
        if parameter.baking != jai_syntax::ParameterBaking::None
            && !introductions.insert(parameter.name)
        {
            return Err(Diagnostic::new(
                span,
                "compile-time variable is introduced more than once",
            ));
        }
        let mut pending = vec![&parameter.ty];
        while let Some(pattern) = pending.pop() {
            match pattern {
                TypePattern::Restricted {
                    ty, ..
                } => pending.push(ty),
                TypePattern::Infer(name) if !introductions.insert(*name) => {
                    return Err(Diagnostic::new(
                        span,
                        "type variable is introduced more than once",
                    ));
                }
                TypePattern::Pointer(inner)
                | TypePattern::Slice(inner)
                | TypePattern::DynamicArray(inner) => pending.push(inner),
                TypePattern::Procedure(procedure) => {
                    pending.extend(procedure.results.iter().rev());
                    pending.extend(procedure.parameters.iter().rev());
                }
                TypePattern::FixedArray {
                    element,
                    count,
                } => {
                    pending.push(element);
                    if let CountPattern::Infer(name) = count
                        && !introductions.insert(*name)
                    {
                        return Err(Diagnostic::new(
                            span,
                            "array count variable is introduced more than once",
                        ));
                    }
                }
                TypePattern::NominalApplication {
                    arguments, ..
                } => {
                    for argument in arguments.iter().rev() {
                        match &argument.kind {
                            NominalArgumentKind::Type(pattern) => pending.push(pattern),
                            NominalArgumentKind::InferValue(name)
                                if !introductions.insert(*name) =>
                            {
                                return Err(Diagnostic::new(
                                    span,
                                    "record value variable is introduced more than once",
                                ));
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
    for result in &candidate.result_type_parameters {
        if parameters.contains(&result.name) {
            return Err(Diagnostic::new(
                result.span,
                "result Type input conflicts with a source parameter",
            ));
        }
        if !introductions.insert(result.name) {
            return Err(Diagnostic::new(
                result.span,
                "type variable is introduced more than once",
            ));
        }
        if !matches!(&result.pattern, TypePattern::Infer(name) if *name == result.name)
            && !matches!(&result.pattern, TypePattern::Restricted { ty, .. } if matches!(ty.as_ref(),TypePattern::Infer(name) if *name == result.name))
        {
            return Err(Diagnostic::new(
                result.span,
                "result Type input has no original introducing pattern",
            ));
        }
    }
    Ok(())
}
fn assign_runtime_parameters<Origin>(
    candidate: &Candidate<Origin>,
    substitution: &Substitution,
    bindings: &mut [ArgumentBinding],
) {
    let mut count = 0;
    let slots = candidate
        .parameters
        .iter()
        .map(|parameter| {
            if !parameter.is_baked(substitution)
                && parameter.evaluation == jai_syntax::ParameterEvaluation::Evaluate
            {
                let slot = count;
                count += 1;
                Some(slot)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    for binding in bindings {
        binding.runtime_parameter = slots
            .get(binding.parameter)
            .copied()
            .unwrap_or_else(|| Some(count + binding.parameter - candidate.parameters.len()));
    }
}
fn bind_arguments<Origin>(
    candidate: &Candidate<Origin>,
    arguments: &[Argument],
    span: Span,
) -> Result<(Vec<ArgumentBinding>, Vec<usize>, Vec<ResultTypeArgument>), Diagnostic> {
    let mut supplied = vec![false; candidate.parameters.len()];
    let mut supplied_results = vec![false; candidate.result_type_parameters.len()];
    let mut result_type_arguments = Vec::new();
    let mut bindings = Vec::with_capacity(arguments.len());
    let mut runtime_parameters = Vec::with_capacity(candidate.parameters.len());
    let mut runtime_count = 0;
    for parameter in &candidate.parameters {
        let runtime = parameter.baking != jai_syntax::ParameterBaking::Required
            && parameter.evaluation == jai_syntax::ParameterEvaluation::Evaluate;
        runtime_parameters.push(runtime.then_some(runtime_count));
        if runtime {
            runtime_count += 1;
        }
    }
    let mut positional = 0;
    let mut named = false;
    let mut forwarded_pack = false;
    for (argument_index, argument) in arguments.iter().enumerate() {
        if let Some(name) = argument.name
            && let Some(parameter) = candidate
                .result_type_parameters
                .iter()
                .position(|parameter| parameter.name == name)
        {
            named = true;
            if argument.spread {
                return Err(Diagnostic::new(
                    argument.span,
                    "result Type input cannot be spread",
                ));
            }
            if std::mem::replace(&mut supplied_results[parameter], true) {
                return Err(Diagnostic::new(
                    argument.span,
                    "duplicate result Type argument",
                ));
            }
            result_type_arguments.push(ResultTypeArgument {
                parameter,
                argument: argument_index,
            });
            continue;
        }
        let parameter = match argument.name {
            Some(name) => {
                named = true;
                candidate
                    .parameters
                    .iter()
                    .position(|parameter| parameter.name == name)
                    .ok_or_else(|| Diagnostic::new(argument.span, "unknown named argument"))?
            }
            None => {
                if named {
                    return Err(Diagnostic::new(
                        argument.span,
                        "positional argument cannot follow a named argument",
                    ));
                }
                match candidate.variadic {
                    CandidateVariadic::Jai {
                        parameter,
                    } if positional >= parameter && !forwarded_pack => parameter,
                    _ => {
                        let index = positional;
                        positional += 1;
                        index
                    }
                }
            }
        };
        let append_spread = argument.spread
            && argument.name.is_none()
            && !forwarded_pack
            && bindings
                .iter()
                .filter(|binding: &&ArgumentBinding| binding.parameter == parameter)
                .all(|binding| {
                    arguments[binding.argument].name.is_none()
                        && !arguments[binding.argument].spread
                });
        if argument.spread {
            if !matches!(candidate.variadic, CandidateVariadic::Jai { parameter: pack } if pack == parameter)
            {
                return Err(Diagnostic::new(
                    argument.span,
                    "spread argument requires a Jai variadic pack",
                ));
            }
            if forwarded_pack || (supplied[parameter] && !append_spread) {
                return Err(Diagnostic::new(
                    argument.span,
                    "spread pack cannot be combined with an already forwarded or named pack",
                ));
            }
            forwarded_pack = true;
            positional = parameter + 1;
        }
        if parameter >= supplied.len() {
            if let CandidateVariadic::C {
                fixed_parameters,
            } = candidate.variadic
                && parameter >= fixed_parameters
                && argument.name.is_none()
            {
                bindings.push(ArgumentBinding {
                    argument: argument_index,
                    parameter,
                    runtime_parameter: Some(runtime_count + parameter - fixed_parameters),
                });
                continue;
            }
            return Err(Diagnostic::new(argument.span, "too many arguments"));
        }
        let pack_element = matches!(candidate.variadic, CandidateVariadic::Jai { parameter: pack } if pack == parameter)
            && argument.name.is_none()
            && !argument.spread
            && !forwarded_pack;
        if supplied[parameter] && !pack_element && !append_spread {
            return Err(Diagnostic::new(
                argument.span,
                "duplicate argument for parameter",
            ));
        }
        supplied[parameter] = true;
        bindings.push(ArgumentBinding {
            argument: argument_index,
            parameter,
            runtime_parameter: runtime_parameters[parameter],
        });
    }
    let mut defaults = Vec::new();
    for (index, parameter) in candidate.parameters.iter().enumerate() {
        if !supplied[index] {
            if matches!(candidate.variadic, CandidateVariadic::Jai { parameter } if parameter == index)
            {
                continue;
            }
            if parameter.default.is_none() {
                return Err(Diagnostic::new(span, "missing required argument"));
            }
            defaults.push(index);
        }
    }
    if supplied_results.iter().any(|supplied| !supplied) {
        return Err(Diagnostic::new(
            span,
            "missing required named result Type argument",
        ));
    }
    Ok((bindings, defaults, result_type_arguments))
}
fn info_for<'a>(
    index: usize,
    parameter: &'a Parameter,
    bindings: &[ArgumentBinding],
    arguments: &'a [Argument],
) -> &'a ArgumentInfo {
    match bindings.iter().find(|binding| binding.parameter == index) {
        Some(binding) => &arguments[binding.argument].info,
        None => parameter
            .default
            .as_ref()
            .expect("argument binding checked defaults"),
    }
}

fn infer(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    pattern: &TypePattern,
    source: &ArgumentType,
    substitution: &mut Substitution,
    allow_conversion: bool,
    span: Span,
) -> Result<(), Diagnostic> {
    if let TypePattern::Restricted {
        ty, ..
    } = pattern
    {
        return infer(
            types,
            nominals,
            ty,
            source,
            substitution,
            allow_conversion,
            span,
        );
    }
    if let ArgumentType::ContextualProcedure {
        compatible_signatures,
    } = source
    {
        if let TypePattern::Procedure(procedure) = pattern {
            return procedures::infer_contextual(
                types,
                nominals,
                procedure,
                compatible_signatures,
                substitution,
                span,
            );
        }
        return Ok(());
    }
    if matches!(source, ArgumentType::ContextualCast { .. }) {
        return Ok(());
    }
    if let TypePattern::Infer(name) = pattern {
        let ty = match source {
            ArgumentType::ContextualCast {
                ..
            }
            | ArgumentType::ContextualProcedure {
                ..
            } => {
                unreachable!("contextual arguments do not infer")
            }
            ArgumentType::Null => {
                return Err(Diagnostic::new(
                    span,
                    "null cannot infer a polymorphic type",
                ));
            }
            ArgumentType::EnumMember(_) => {
                return Err(Diagnostic::new(
                    span,
                    "leading-dot member cannot infer a polymorphic enum type",
                ));
            }
            ArgumentType::RecordLiteral {
                ty: Some(ty), ..
            }
            | ArgumentType::ArrayLiteral {
                default: Some(ty), ..
            } => *ty,
            ArgumentType::RecordLiteral {
                ty: None, ..
            }
            | ArgumentType::ArrayLiteral {
                default: None, ..
            } => {
                return Err(Diagnostic::new(
                    span,
                    "aggregate literal needs a concrete type before polymorphic inference",
                ));
            }
            ArgumentType::Known(ty) | ArgumentType::StringLiteral(ty) => {
                types
                    .kind(*ty)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                *ty
            }
            ArgumentType::WeakInteger {
                minimum,
                maximum,
            } if fits(IntegerType::S64, *minimum, *maximum) => {
                types.scalar(ScalarType::Int(IntegerType::S64))
            }
            ArgumentType::WeakInteger {
                ..
            } => {
                return Err(Diagnostic::new(
                    span,
                    "integer literal cannot determine its default s64 type",
                ));
            }
            ArgumentType::WeakFloat {
                spelling,
                default,
                ..
            } => {
                FloatValue::parse_decimal(*default, spelling)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                types.float(*default)
            }
            ArgumentType::WeakFloatExpression {
                default,
                permits_f32,
                permits_f64,
            } => {
                if !match default {
                    FloatType::F32 => *permits_f32,
                    FloatType::F64 => *permits_f64,
                } {
                    return Err(Diagnostic::new(
                        span,
                        "weak floating-point expression has no valid default type",
                    ));
                }
                types.float(*default)
            }
        };
        if !substitution.bind_type(*name, ty) {
            return Err(Diagnostic::new(span, "conflicting type inference"));
        }
        return Ok(());
    }
    if allow_conversion
        && matches!(source, ArgumentType::StringLiteral(_))
        && let TypePattern::Pointer(element) = pattern
    {
        return infer(
            types,
            nominals,
            element,
            &ArgumentType::Known(types.scalar(ScalarType::Int(IntegerType::U8))),
            substitution,
            false,
            span,
        );
    }
    if let ArgumentType::ArrayLiteral {
        explicit,
        elements,
        ..
    } = source
    {
        let element = match pattern {
            TypePattern::FixedArray {
                element,
                count,
            } => {
                if let CountPattern::Infer(name) = count {
                    let count = i128::try_from(elements.len())
                        .ok()
                        .and_then(|count| Integer::checked(IntegerType::S64, count))
                        .ok_or_else(|| Diagnostic::new(span, "array literal count exceeds s64"))?;
                    if !substitution.bind_constant(*name, BakedValue::integer(count, types)) {
                        return Err(Diagnostic::new(
                            span,
                            "conflicting array literal count inference",
                        ));
                    }
                }
                element
            }
            TypePattern::Slice(element) if allow_conversion => element,
            _ => return Ok(()),
        };
        if let Some(explicit) = explicit {
            return infer(
                types,
                nominals,
                element,
                &ArgumentType::Known(*explicit),
                substitution,
                false,
                span,
            );
        }
        if let Some(value) = elements.first() {
            return infer(
                types,
                nominals,
                element,
                &value.ty,
                substitution,
                true,
                span,
            );
        }
        return Ok(());
    }
    let Some(source) = literals::default_type(source) else {
        // Scalar weak literals cannot reveal a nested pointer or array type.
        return Ok(());
    };
    let kind = types
        .kind(source)
        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
    if let TypePattern::Procedure(pattern) = pattern {
        return procedures::infer(types, nominals, pattern, source, substitution, span);
    }
    match (pattern, kind) {
        (TypePattern::Pointer(pattern), TypeKind::Pointer(source)) => infer(
            types,
            nominals,
            pattern,
            &ArgumentType::Known(*source),
            substitution,
            false,
            span,
        ),
        (TypePattern::Slice(pattern), TypeKind::Slice(source))
        | (TypePattern::DynamicArray(pattern), TypeKind::DynamicArray(source)) => infer(
            types,
            nominals,
            pattern,
            &ArgumentType::Known(*source),
            substitution,
            false,
            span,
        ),
        (
            TypePattern::Slice(pattern),
            TypeKind::FixedArray {
                element: source, ..
            }
            | TypeKind::DynamicArray(source),
        ) if allow_conversion => infer(
            types,
            nominals,
            pattern,
            &ArgumentType::Known(*source),
            substitution,
            false,
            span,
        ),
        (
            TypePattern::FixedArray {
                element,
                count,
            },
            TypeKind::FixedArray {
                element: source,
                count: actual,
            },
        ) => {
            if let CountPattern::Infer(name) = count {
                let value = Integer::checked(IntegerType::S64, i128::from(*actual))
                    .ok_or_else(|| Diagnostic::new(span, "array count exceeds s64"))?;
                if !substitution.bind_constant(*name, BakedValue::integer(value, types)) {
                    return Err(Diagnostic::new(span, "conflicting array count inference"));
                }
            }
            infer(
                types,
                nominals,
                element,
                &ArgumentType::Known(*source),
                substitution,
                false,
                span,
            )
        }
        (
            TypePattern::NominalApplication {
                declaration,
                arguments,
            },
            TypeKind::Record(_),
        ) => {
            let Some((origin, values)) = nominals.specialization(source) else {
                return Ok(());
            };
            if origin != *declaration {
                return Ok(());
            }
            for argument in arguments {
                match &argument.kind {
                    NominalArgumentKind::Type(pattern) => {
                        let ty = values.ty(argument.name).ok_or_else(|| {
                            Diagnostic::new(span, "record specialization has no type argument")
                        })?;
                        infer(
                            types,
                            nominals,
                            pattern,
                            &ArgumentType::Known(ty),
                            substitution,
                            false,
                            span,
                        )?;
                    }
                    NominalArgumentKind::InferValue(name) => {
                        let value = values.constant(argument.name).ok_or_else(|| {
                            Diagnostic::new(
                                span,
                                "record specialization has no baked value argument",
                            )
                        })?;
                        if !substitution.bind_constant(*name, value.clone()) {
                            return Err(Diagnostic::new(
                                span,
                                "conflicting record value inference",
                            ));
                        }
                    }
                    _ => {}
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn compatible(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    pattern: &TypePattern,
    source: &ArgumentType,
    substitution: &Substitution,
    allow_conversion: bool,
    span: Span,
) -> Result<ConversionRank, Diagnostic> {
    match pattern {
        TypePattern::Restricted {
            ty,
            restriction,
        } => {
            let target = match ty.as_ref() {
                TypePattern::Concrete(ty) => *ty,
                TypePattern::Infer(name) | TypePattern::Variable(name) => {
                    substitution.ty(*name).ok_or_else(|| {
                        Diagnostic::new(span, "restricted type variable could not be inferred")
                    })?
                }
                _ => {
                    return Err(Diagnostic::new(
                        span,
                        "restriction must apply to a type-variable leaf",
                    ));
                }
            };
            restrictions::validate(types, nominals, restriction, target, substitution, span)?;
            return compatible(
                types,
                nominals,
                ty,
                source,
                substitution,
                allow_conversion,
                span,
            );
        }
        TypePattern::Concrete(target) => {
            return concrete(types, nominals, *target, source, allow_conversion, span);
        }
        TypePattern::Infer(name) | TypePattern::Variable(name) => {
            let target = substitution.ty(*name).ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "type variable could not be inferred from its introducing argument",
                )
            })?;
            return concrete(types, nominals, target, source, allow_conversion, span);
        }
        _ => {}
    }
    if let ArgumentType::ContextualCast {
        mode,
        value,
    } = source
    {
        return casts::structural(types, nominals, pattern, value, substitution, *mode, span);
    }
    if let TypePattern::Procedure(pattern) = pattern {
        return procedures::compatible(types, nominals, pattern, source, substitution, span);
    }
    if allow_conversion
        && matches!(source, ArgumentType::StringLiteral(_))
        && let TypePattern::Pointer(element) = pattern
    {
        compatible(
            types,
            nominals,
            element,
            &ArgumentType::Known(types.scalar(ScalarType::Int(IntegerType::U8))),
            substitution,
            false,
            span,
        )?;
        return Ok(ConversionRank::Literal);
    }
    if let ArgumentType::ArrayLiteral {
        ..
    } = source
    {
        return literals::array_pattern(
            types,
            nominals,
            pattern,
            source,
            substitution,
            allow_conversion,
            span,
        );
    }
    if matches!(source, ArgumentType::Null) && matches!(pattern, TypePattern::Pointer(_)) {
        ensure_bound(pattern, substitution, span)?;
        return Ok(ConversionRank::Literal);
    }
    let source_literal = source;
    let Some(source) = literals::default_type(source) else {
        return Err(Diagnostic::new(span, "expected pointer or array argument"));
    };
    let kind = types
        .kind(source)
        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
    if let TypePattern::NominalApplication {
        declaration,
        arguments,
    } = pattern
    {
        let (origin, values) = nominals
            .specialization(source)
            .ok_or_else(|| Diagnostic::new(span, "argument is not a specialized nominal record"))?;
        if origin != *declaration {
            return Err(Diagnostic::new(
                span,
                "record argument belongs to a different template",
            ));
        }
        for argument in arguments {
            match &argument.kind {
                NominalArgumentKind::Type(pattern) => {
                    let ty = values.ty(argument.name).ok_or_else(|| {
                        Diagnostic::new(span, "record specialization has no type argument")
                    })?;
                    compatible(
                        types,
                        nominals,
                        pattern,
                        &ArgumentType::Known(ty),
                        substitution,
                        false,
                        span,
                    )?;
                }
                NominalArgumentKind::InferValue(name)
                | NominalArgumentKind::ValueVariable(name) => {
                    if substitution.constant(*name).is_none()
                        || substitution.constant(*name) != values.constant(argument.name)
                    {
                        return Err(Diagnostic::new(
                            span,
                            "record specialization's value argument differs",
                        ));
                    }
                }
                NominalArgumentKind::Value(expected)
                    if values.constant(argument.name) != Some(expected) =>
                {
                    return Err(Diagnostic::new(
                        span,
                        "record specialization's value argument differs",
                    ));
                }
                NominalArgumentKind::Default => {
                    let expected = nominals
                        .default_argument(source, argument.name)
                        .ok_or_else(|| {
                            Diagnostic::new(
                                span,
                                "record specialization's default argument is not ready",
                            )
                        })?;
                    if values.constant(argument.name) != Some(expected) {
                        return Err(Diagnostic::new(
                            span,
                            "record specialization's argument differs from the template default",
                        ));
                    }
                }
                _ => {}
            }
        }
        if matches!(source_literal, ArgumentType::RecordLiteral { .. }) {
            return literals::concrete(types, nominals, source, source_literal, span);
        }
        return Ok(ConversionRank::Exact);
    }
    let (element_pattern, element_source, rank) = match (pattern, kind) {
        (TypePattern::Pointer(pattern), TypeKind::Pointer(source))
        | (TypePattern::Slice(pattern), TypeKind::Slice(source))
        | (TypePattern::DynamicArray(pattern), TypeKind::DynamicArray(source)) => {
            (pattern.as_ref(), *source, ConversionRank::Exact)
        }
        (
            TypePattern::Slice(pattern),
            TypeKind::FixedArray {
                element: source, ..
            }
            | TypeKind::DynamicArray(source),
        ) if allow_conversion => (pattern.as_ref(), *source, ConversionRank::ArrayView),
        (
            TypePattern::FixedArray {
                element,
                count,
            },
            TypeKind::FixedArray {
                element: source,
                count: actual,
            },
        ) => {
            let expected = match count {
                CountPattern::Exact(count) => *count,
                CountPattern::Infer(name) | CountPattern::Variable(name) => substitution
                    .constant(*name)
                    .and_then(BakedValue::as_integer)
                    .and_then(|value| u64::try_from(value.value()).ok())
                    .ok_or_else(|| {
                        Diagnostic::new(span, "array count variable could not be inferred")
                    })?,
            };
            if expected != *actual {
                return Err(Diagnostic::new(
                    span,
                    "fixed array argument has the wrong count",
                ));
            }
            (element.as_ref(), *source, ConversionRank::Exact)
        }
        _ => {
            return Err(Diagnostic::new(
                span,
                "argument type does not match the pointer or array pattern",
            ));
        }
    };
    compatible(
        types,
        nominals,
        element_pattern,
        &ArgumentType::Known(element_source),
        substitution,
        false,
        span,
    )?;
    Ok(rank)
}
pub(crate) fn concrete(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    target: TypeId,
    source: &ArgumentType,
    allow_conversion: bool,
    span: Span,
) -> Result<ConversionRank, Diagnostic> {
    if let ArgumentType::ContextualProcedure {
        compatible_signatures,
    } = source
    {
        return if compatible_signatures.contains(&target)
            && matches!(types.kind(target), Ok(TypeKind::Procedure(_)))
        {
            Ok(ConversionRank::Literal)
        } else {
            Err(Diagnostic::new(
                span,
                "source lambda does not match this callback signature",
            ))
        };
    }
    if let ArgumentType::ContextualCast {
        mode,
        value,
    } = source
    {
        return casts::concrete(types, nominals, target, value, *mode, span);
    }
    let target_kind = types
        .kind(target)
        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
    if let ArgumentType::RecordLiteral {
        ..
    }
    | ArgumentType::ArrayLiteral {
        ..
    } = source
    {
        if allow_conversion && matches!(target_kind, TypeKind::Any(_)) {
            if matches!(source, ArgumentType::RecordLiteral { ty, .. } if ty.is_none_or(|ty| ty == target))
            {
                return literals::concrete(types, nominals, target, source, span);
            }
            let represented = literals::default_type(source).ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "aggregate literal needs a concrete represented type before Any boxing",
                )
            })?;
            literals::concrete(types, nominals, represented, source, span)?;
            boxing::runtime_payload(types, represented, span)?;
            return Ok(ConversionRank::Boxing);
        }
        return literals::concrete(types, nominals, target, source, span);
    }
    if allow_conversion && matches!(target_kind, TypeKind::Any(_)) {
        if let ArgumentType::Known(source) = source
            && *source == target
        {
            return Ok(ConversionRank::Exact);
        }
        let represented = match source {
            ArgumentType::ContextualCast {
                ..
            }
            | ArgumentType::ContextualProcedure {
                ..
            } => {
                unreachable!("contextual arguments checked first")
            }
            ArgumentType::Null => {
                return Err(Diagnostic::new(
                    span,
                    "null needs a concrete pointer type before boxing into Any",
                ));
            }
            ArgumentType::EnumMember(_) => {
                return Err(Diagnostic::new(
                    span,
                    "leading-dot member needs a concrete enum type before boxing into Any",
                ));
            }
            ArgumentType::RecordLiteral {
                ..
            }
            | ArgumentType::ArrayLiteral {
                ..
            } => {
                unreachable!("aggregate boxing checked before scalar dispatch")
            }
            ArgumentType::Known(source) | ArgumentType::StringLiteral(source) => *source,
            ArgumentType::WeakInteger {
                minimum,
                maximum,
            } if fits(IntegerType::S64, *minimum, *maximum) => {
                types.scalar(ScalarType::Int(IntegerType::S64))
            }
            ArgumentType::WeakInteger {
                ..
            } => {
                return Err(Diagnostic::new(
                    span,
                    "integer literal has no default s64 type for Any boxing",
                ));
            }
            ArgumentType::WeakFloat {
                spelling,
                default,
                ..
            } => {
                FloatValue::parse_decimal(*default, spelling)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                types.float(*default)
            }
            ArgumentType::WeakFloatExpression {
                default,
                permits_f32,
                permits_f64,
            } => {
                if !match default {
                    FloatType::F32 => *permits_f32,
                    FloatType::F64 => *permits_f64,
                } {
                    return Err(Diagnostic::new(
                        span,
                        "weak floating-point expression has no default type for Any boxing",
                    ));
                }
                types.float(*default)
            }
        };
        boxing::runtime_payload(types, represented, span)?;
        return Ok(ConversionRank::Boxing);
    }
    match source {
        ArgumentType::ContextualCast {
            ..
        }
        | ArgumentType::ContextualProcedure {
            ..
        } => {
            unreachable!("contextual arguments checked first")
        }
        ArgumentType::RecordLiteral {
            ..
        }
        | ArgumentType::ArrayLiteral {
            ..
        } => {
            unreachable!("aggregate literals checked before scalar dispatch")
        }
        ArgumentType::EnumMember(name)
            if matches!(target_kind, TypeKind::Enum(_))
                && nominals.enum_member(target, *name).is_some() =>
        {
            Ok(ConversionRank::Literal)
        }
        ArgumentType::EnumMember(_) => Err(Diagnostic::new(
            span,
            "leading-dot member does not belong to the contextual enum parameter",
        )),
        ArgumentType::Null
            if matches!(target_kind, TypeKind::Pointer(_) | TypeKind::Procedure(_)) =>
        {
            Ok(ConversionRank::Literal)
        }
        ArgumentType::Null => Err(Diagnostic::new(
            span,
            "null requires a pointer or procedure parameter",
        )),
        ArgumentType::StringLiteral(_)
            if allow_conversion
                && matches!(target_kind, TypeKind::Pointer(element)
                if matches!(types.kind(*element), Ok(TypeKind::Integer(IntegerType::U8)))) =>
        {
            Ok(ConversionRank::Literal)
        }
        ArgumentType::Known(source) | ArgumentType::StringLiteral(source) if *source == target => {
            Ok(ConversionRank::Exact)
        }
        ArgumentType::Known(source) | ArgumentType::StringLiteral(source) => {
            let source_kind = types
                .kind(*source)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            if allow_conversion {
                if nominals.implicit_conversion(*source, target, span)? {
                    return Ok(ConversionRank::Widening);
                }
                if let (TypeKind::Pointer(target), TypeKind::Pointer(_)) =
                    (target_kind, source_kind)
                    && matches!(types.kind(*target), Ok(TypeKind::Void))
                {
                    return Ok(ConversionRank::Widening);
                }
                if matches!(
                    (target_kind, source_kind),
                    (
                        TypeKind::Float(FloatType::F64),
                        TypeKind::Float(FloatType::F32)
                    )
                ) {
                    return Ok(ConversionRank::Widening);
                }
                if let (TypeKind::Integer(target), TypeKind::Integer(source)) =
                    (target_kind, source_kind)
                    && target.contains(*source)
                {
                    return Ok(ConversionRank::Widening);
                }
                if let (
                    TypeKind::Slice(target),
                    TypeKind::FixedArray {
                        element: source, ..
                    }
                    | TypeKind::DynamicArray(source),
                ) = (target_kind, source_kind)
                    && target == source
                {
                    return Ok(ConversionRank::ArrayView);
                }
            }
            Err(Diagnostic::new(
                span,
                "argument type cannot be implicitly converted to the parameter type",
            ))
        }
        ArgumentType::WeakInteger {
            minimum,
            maximum,
        } => {
            if let TypeKind::Float(ty) = *target_kind {
                integer_float(*minimum, ty, span)?;
                integer_float(*maximum, ty, span)?;
                return Ok(ConversionRank::Literal);
            }
            let TypeKind::Integer(target) = *target_kind else {
                return Err(Diagnostic::new(
                    span,
                    "integer literal requires a numeric parameter",
                ));
            };
            if !fits(target, *minimum, *maximum) {
                return Err(Diagnostic::new(
                    span,
                    "integer constant is out of range for its target type",
                ));
            }
            Ok(if target == IntegerType::S64 {
                ConversionRank::Exact
            } else {
                ConversionRank::Literal
            })
        }
        ArgumentType::WeakFloat {
            spelling,
            default,
            ..
        } => {
            let TypeKind::Float(target) = *target_kind else {
                return Err(Diagnostic::new(
                    span,
                    "floating-point literal requires a floating-point parameter",
                ));
            };
            FloatValue::parse_decimal(target, spelling)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            Ok(if target == *default {
                ConversionRank::Exact
            } else {
                ConversionRank::Literal
            })
        }
        ArgumentType::WeakFloatExpression {
            default,
            permits_f32,
            permits_f64,
        } => {
            let TypeKind::Float(target) = *target_kind else {
                return Err(Diagnostic::new(
                    span,
                    "weak floating-point expression requires a floating-point parameter",
                ));
            };
            if !match target {
                FloatType::F32 => *permits_f32,
                FloatType::F64 => *permits_f64,
            } {
                return Err(Diagnostic::new(
                    span,
                    "floating-point literal leaf is outside its target width",
                ));
            }
            Ok(if target == *default {
                ConversionRank::Exact
            } else {
                ConversionRank::Literal
            })
        }
    }
}
/// Share contextual result feasibility with source-only callback previews.
pub(crate) fn contextual_conversion(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    target: TypeId,
    source: &ArgumentType,
    span: Span,
) -> Result<ConversionRank, Diagnostic> {
    concrete(types, nominals, target, source, true, span)
}
/// An explicit cast retains its declared type even when constant materialization
/// fails. Ordinary checked casts may trap at runtime; baking validates the value.
pub(crate) fn explicit_cast_argument(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    target: TypeId,
    source: &ArgumentInfo,
    mode: CastMode,
    span: Span,
) -> Result<ArgumentInfo, Diagnostic> {
    casts::concrete(types, nominals, target, source, mode, span)?;
    if !source.is_compile_time_constant() || matches!(mode, CastMode::Force(_)) {
        return Ok(ArgumentInfo::typed(target));
    }
    Ok(
        match casts::bake(types, nominals, target, source, mode, span) {
            Ok(value) => ArgumentInfo::constant(value, target),
            // Preserve applicability for runtime checked failures and storage
            // recipes. Source constantness keeps these deferred when baking is asked.
            Err(_) => ArgumentInfo::typed(target),
        },
    )
}
fn fits(ty: IntegerType, minimum: i128, maximum: i128) -> bool {
    minimum <= maximum && minimum >= ty.min() && maximum <= ty.max()
}
pub(crate) fn bake(
    types: &dyn TypeView,
    pattern: &TypePattern,
    info: &ArgumentInfo,
    substitution: &Substitution,
    span: Span,
) -> Result<BakedValue, Diagnostic> {
    bake_with_nominals(types, &NoNominals, pattern, info, substitution, span)
}
fn bake_with_nominals(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    pattern: &TypePattern,
    info: &ArgumentInfo,
    substitution: &Substitution,
    span: Span,
) -> Result<BakedValue, Diagnostic> {
    if let TypePattern::Restricted {
        ty, ..
    } = pattern
    {
        compatible(types, nominals, pattern, &info.ty, substitution, true, span)?;
        return bake_with_nominals(types, nominals, ty, info, substitution, span);
    }
    if matches!(&info.ty, ArgumentType::ContextualProcedure { .. }) {
        return Err(Diagnostic::new(
            span,
            "source lambda cannot be baked without a stable checked procedure identity",
        ));
    }
    if let ArgumentType::ContextualCast {
        mode,
        value,
    } = &info.ty
    {
        let target = casts::target_type(types, pattern, substitution, span)?;
        return casts::bake(types, nominals, target, value, *mode, span);
    }
    if let ArgumentType::RecordLiteral {
        ..
    }
    | ArgumentType::ArrayLiteral {
        ..
    } = &info.ty
    {
        let target = match pattern {
            TypePattern::Concrete(ty) => Some(*ty),
            TypePattern::Infer(name) | TypePattern::Variable(name) => substitution.ty(*name),
            _ => literals::explicit_type(&info.ty),
        }
        .ok_or_else(|| {
            Diagnostic::new(
                span,
                "baked aggregate literal requires a canonical concrete type",
            )
        })?;
        return literals::bake(types, nominals, target, &info.ty, span);
    }
    let value = info
        .constant
        .as_ref()
        .ok_or_else(|| Diagnostic::new(span, "baked argument requires a compile-time constant"))?;
    let target = match pattern {
        TypePattern::Concrete(ty) => Some(*ty),
        TypePattern::Infer(name) | TypePattern::Variable(name) => substitution.ty(*name),
        _ => None,
    };
    if matches!(info.ty, ArgumentType::StringLiteral(_))
        && target.is_some_and(|ty| matches!(types.kind(ty), Ok(TypeKind::Pointer(_))))
    {
        return Err(Diagnostic::new(
            span,
            "baked literal string pointers require a published static data identity",
        ));
    }
    let integer = match value {
        ConstantArgument::IntegerLiteral(value) => Some(*value),
        ConstantArgument::Value(value) => value.as_integer().map(Integer::value),
        ConstantArgument::FloatLiteral {
            ..
        }
        | ConstantArgument::FloatExpression {
            ..
        }
        | ConstantArgument::Null
        | ConstantArgument::EnumMember(_)
        | ConstantArgument::CallerLocation
        | ConstantArgument::CodeNull
        | ConstantArgument::RuntimeRead(_) => None,
    };
    if let (Some(target), Some(integer)) = (target, integer) {
        if let Ok(TypeKind::Integer(ty)) = types.kind(target) {
            let integer = Integer::checked(*ty, integer)
                .ok_or_else(|| Diagnostic::new(span, "baked integer is out of range"))?;
            return Ok(BakedValue::integer(integer, types));
        }
        if let Ok(TypeKind::Float(ty)) = types.kind(target) {
            return Ok(BakedValue::Float(integer_float(integer, *ty, span)?));
        }
    }
    match value {
        ConstantArgument::RuntimeRead(_) => Err(Diagnostic::new(
            span,
            "runtime storage defaults cannot supply a baked argument",
        )),
        ConstantArgument::CodeNull => Err(Diagnostic::new(
            span,
            "#code, null is a compiler-only default and cannot supply a baked runtime argument",
        )),
        ConstantArgument::CallerLocation => Err(Diagnostic::new(
            span,
            "#caller_location is a deferred runtime default and cannot supply a baked argument",
        )),
        ConstantArgument::EnumMember(name) => {
            let ty = target
                .filter(|&ty| matches!(types.kind(ty), Ok(TypeKind::Enum(_))))
                .ok_or_else(|| {
                    Diagnostic::new(
                        span,
                        "baked leading-dot member requires a concrete enum type",
                    )
                })?;
            let value = nominals.enum_member(ty, *name).ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "baked leading-dot member does not belong to its contextual enum",
                )
            })?;
            BakedValue::runtime(
                jai_ir::ConstantValue {
                    ty,
                    kind: jai_ir::ConstantKind::Enum(value),
                },
                types,
            )
            .map_err(|error| Diagnostic::new(span, error.to_string()))
        }
        ConstantArgument::Null => {
            let target = target
                .filter(|&ty| {
                    matches!(
                        types.kind(ty),
                        Ok(TypeKind::Pointer(_) | TypeKind::Procedure(_))
                    )
                })
                .ok_or_else(|| {
                    Diagnostic::new(
                        span,
                        "baked null requires a concrete pointer or procedure type",
                    )
                })?;
            Ok(BakedValue::Value(jai_ir::ConstantValue {
                ty: target,
                kind: jai_ir::ConstantKind::Zero,
            }))
        }
        ConstantArgument::Value(BakedValue::Value(value)) => {
            if matches!(value.kind, jai_ir::ConstantKind::NativePointer(_)) {
                return Err(Diagnostic::new(
                    span,
                    "native address constants cannot supply baked VM pointer provenance",
                ));
            }
            BakedValue::runtime(value.clone(), types)
                .map_err(|error| Diagnostic::new(span, error.to_string()))
        }
        ConstantArgument::Value(BakedValue::Float(value)) => {
            match target.and_then(|target| types.kind(target).ok()) {
                Some(TypeKind::Float(ty)) => Ok(BakedValue::Float(value.convert(*ty))),
                _ => Err(Diagnostic::new(
                    span,
                    "baked floating-point parameter type could not be resolved",
                )),
            }
        }
        ConstantArgument::Value(value) => Ok(value.clone()),
        ConstantArgument::FloatLiteral {
            spelling,
            negative,
        } => {
            let target = target.ok_or_else(|| {
                Diagnostic::new(span, "baked floating-point type could not be resolved")
            })?;
            let TypeKind::Float(ty) = *types
                .kind(target)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
            else {
                return Err(Diagnostic::new(
                    span,
                    "baked floating-point literal requires a floating-point parameter",
                ));
            };
            let value = FloatValue::parse_decimal(ty, spelling)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            Ok(BakedValue::Float(if *negative {
                value.negate()
            } else {
                value
            }))
        }
        ConstantArgument::FloatExpression {
            f32,
            f64,
        } => {
            let value = match target.and_then(|target| types.kind(target).ok()) {
                Some(TypeKind::Float(FloatType::F32)) => f32,
                Some(TypeKind::Float(FloatType::F64)) => f64,
                _ => {
                    return Err(Diagnostic::new(
                        span,
                        "baked weak float requires a concrete floating-point type",
                    ));
                }
            }
            .ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "floating-point constant cannot be evaluated in its target width",
                )
            })?;
            Ok(BakedValue::Float(value))
        }
        ConstantArgument::IntegerLiteral(_) => Err(Diagnostic::new(
            span,
            "baked integer parameter type could not be resolved",
        )),
    }
}
pub(crate) fn pattern_bindings_ready(
    pattern: &TypePattern,
    substitution: &Substitution,
    span: Span,
) -> bool {
    ensure_bound(pattern, substitution, span).is_ok()
}

fn ensure_bound(
    pattern: &TypePattern,
    substitution: &Substitution,
    span: Span,
) -> Result<(), Diagnostic> {
    match pattern {
        TypePattern::Restricted {
            ty, ..
        } => ensure_bound(ty, substitution, span),
        TypePattern::Infer(name) | TypePattern::Variable(name)
            if substitution.ty(*name).is_none() =>
        {
            Err(Diagnostic::new(
                span,
                "null cannot determine the pointer's polymorphic pointee type",
            ))
        }
        TypePattern::Pointer(element)
        | TypePattern::Slice(element)
        | TypePattern::DynamicArray(element) => ensure_bound(element, substitution, span),
        TypePattern::Procedure(procedure) => {
            for pattern in procedure.parameters.iter().chain(&procedure.results) {
                ensure_bound(pattern, substitution, span)?;
            }
            Ok(())
        }
        TypePattern::FixedArray {
            element,
            count,
        } => {
            if let CountPattern::Infer(name) | CountPattern::Variable(name) = count
                && substitution.constant(*name).is_none()
            {
                return Err(Diagnostic::new(
                    span,
                    "null cannot determine the pointer's array count",
                ));
            }
            ensure_bound(element, substitution, span)
        }
        TypePattern::NominalApplication {
            arguments, ..
        } => {
            for argument in arguments {
                match &argument.kind {
                    NominalArgumentKind::Type(pattern) => {
                        ensure_bound(pattern, substitution, span)?
                    }
                    NominalArgumentKind::InferValue(name)
                    | NominalArgumentKind::ValueVariable(name)
                        if substitution.constant(*name).is_none() =>
                    {
                        return Err(Diagnostic::new(
                            span,
                            "record value argument could not be inferred",
                        ));
                    }
                    _ => {}
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
fn integer_float(value: i128, ty: FloatType, span: Span) -> Result<FloatValue, Diagnostic> {
    let integer = Integer::checked(IntegerType::S64, value)
        .or_else(|| Integer::checked(IntegerType::U64, value))
        .ok_or_else(|| {
            Diagnostic::new(
                span,
                "integer literal exceeds the supported float conversion range",
            )
        })?;
    Ok(FloatValue::from_integer(ty, integer))
}

mod boxing;
mod casts;
mod literals;
mod record_literals;
mod record_targets;
pub use record_targets::{RecordArgumentStep, RecordArgumentTarget};
mod procedures;
mod restrictions;
pub(crate) use procedures::infer_lambda_result;
pub(crate) use procedures::procedure_pattern;
#[cfg(test)]
mod tests;
