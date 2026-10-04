use super::*;
use crate::polymorphism::{ConstantBinding, SpecializationKey, TypeBinding, materialize};
use jai_ir::{ConstantKind, ConstantValue};
use jai_source::{Identities, Symbols};
use jai_types::{RecordKind, TypeRegistry};
use std::collections::HashMap;
use std::slice::from_ref;

struct Names {
    symbols: Symbols,
    identities: Identities,
}
impl Names {
    fn new() -> Self {
        Self {
            symbols: Symbols::default(),
            identities: Identities::default(),
        }
    }
    fn symbol(&mut self, text: &str) -> Symbol {
        self.symbols.intern(text)
    }
    fn candidate(&mut self, parameters: Vec<Parameter>) -> Candidate {
        Candidate {
            result_type_parameters: Vec::new(),
            declaration: self.identities.declaration(),
            parameters,
            variadic: CandidateVariadic::None,
        }
    }
}
fn parameter(name: Symbol, ty: TypePattern) -> Parameter {
    Parameter {
        evaluation: jai_syntax::ParameterEvaluation::Evaluate,
        name,
        ty,
        default: None,
        baking: jai_syntax::ParameterBaking::None,
    }
}
fn argument(info: ArgumentInfo) -> Argument {
    Argument {
        name: None,
        spread: false,
        info,
        span: Span::default(),
    }
}

#[test]
fn native_addresses_are_ordinary_pointer_arguments_without_baked_vm_provenance() {
    let mut names = Names::new();
    let mut types = TypeRegistry::new();
    let pointer = types.pointer(types.void()).unwrap();
    let value = names.symbol("value");
    let mut candidate = names.candidate(vec![parameter(value, TypePattern::Concrete(pointer))]);
    let native = jai_ir::NativePointerConstant::new(
        pointer,
        Integer::wrapping(IntegerType::S64, -1),
        CastMode::Unchecked,
        &types,
    )
    .unwrap();
    let argument = argument(ArgumentInfo::constant(
        BakedValue::Value(ConstantValue {
            ty: pointer,
            kind: ConstantKind::NativePointer(native),
        }),
        pointer,
    ));
    let matched =
        match_candidate(&types, &candidate, from_ref(&argument), Span::default()).unwrap();
    assert!(matched.substitution.constants.is_empty());
    assert_eq!(matched.bindings[0].runtime_parameter, Some(0));
    for policy in [
        jai_syntax::ParameterBaking::Required,
        jai_syntax::ParameterBaking::Optional,
    ] {
        candidate.parameters[0].baking = policy;
        let error =
            match_candidate(&types, &candidate, from_ref(&argument), Span::default()).unwrap_err();
        assert!(
            error.message.contains("baked VM pointer provenance"),
            "{error:?}"
        );
    }
}

#[test]
fn nonscalar_distinct_casts_preserve_payloads_and_actual_wrapper_identity() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let array = types.fixed_array(int, 2).unwrap();
    let first = types.reserve_distinct(jai_types::DistinctKind::Distinct);
    let second = types.reserve_distinct(jai_types::DistinctKind::Distinct);
    types.define_distinct(first, array).unwrap();
    types.define_distinct(second, array).unwrap();
    let payload = ConstantValue {
        ty: array,
        kind: ConstantKind::Array(
            [20, 22]
                .into_iter()
                .map(|value| ConstantValue {
                    ty: int,
                    kind: ConstantKind::Int(Integer::checked(IntegerType::S64, value).unwrap()),
                })
                .collect(),
        ),
    };
    let source =
        ArgumentInfo::constant(BakedValue::runtime(payload.clone(), &types).unwrap(), array);
    let wrapped = explicit_cast_argument(
        &types,
        &NoNominals,
        first,
        &source,
        CastMode::Checked,
        Span::default(),
    )
    .unwrap();
    assert_eq!(
        wrapped.constant,
        Some(ConstantArgument::Value(BakedValue::Value(ConstantValue {
            ty: first,
            kind: ConstantKind::Distinct(Box::new(payload.clone())),
        })))
    );
    let rewrapped = explicit_cast_argument(
        &types,
        &NoNominals,
        second,
        &wrapped,
        CastMode::Checked,
        Span::default(),
    )
    .unwrap();
    assert_eq!(
        rewrapped.constant,
        Some(ConstantArgument::Value(BakedValue::Value(ConstantValue {
            ty: second,
            kind: ConstantKind::Distinct(Box::new(payload.clone())),
        })))
    );
    assert_ne!(wrapped.constant, rewrapped.constant);
    let unwrapped = explicit_cast_argument(
        &types,
        &NoNominals,
        array,
        &rewrapped,
        CastMode::Checked,
        Span::default(),
    )
    .unwrap();
    assert_eq!(
        unwrapped.constant,
        Some(ConstantArgument::Value(BakedValue::Value(payload)))
    );
    let wrong_extent = types.fixed_array(int, 3).unwrap();
    assert!(
        explicit_cast_argument(
            &types,
            &NoNominals,
            wrong_extent,
            &rewrapped,
            CastMode::Checked,
            Span::default(),
        )
        .is_err()
    );
}

#[test]
fn explicit_float_casts_round_weak_constants_in_the_declared_target_width() {
    let types = TypeRegistry::new();
    let target = types.float(FloatType::F64);
    let spelling = "0.10000000000000001";
    let source = ArgumentInfo::decimal_literal(spelling.into(), false, FloatType::F32);
    let description = explicit_cast_argument(
        &types,
        &NoNominals,
        target,
        &source,
        CastMode::Checked,
        Span::default(),
    )
    .unwrap();
    let Some(ConstantArgument::Value(BakedValue::Float(actual))) = description.constant else {
        panic!("cast must retain its checked constant: {description:?}");
    };
    let expected = FloatValue::parse_decimal(FloatType::F64, spelling).unwrap();
    assert_eq!(actual, expected);
    assert_ne!(
        actual,
        FloatValue::parse_decimal(FloatType::F32, spelling)
            .unwrap()
            .convert(FloatType::F64),
    );
}

#[test]
fn runtime_storage_defaults_remain_runtime_under_optional_baking() {
    let mut names = Names::new();
    let mut types = TypeRegistry::new();
    let integer = types.scalar(ScalarType::Int(IntegerType::S64));
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, vec![integer]).unwrap();
    let field = types.field(record, 0).unwrap().id;
    let mut sources = jai_source::SourceMap::default();
    let source = sources.insert("/runtime-default/main.jai".into(), "context.value".into());
    let read = crate::runtime_defaults::RuntimeDefaultRead::checked(
        crate::runtime_defaults::DefaultReadRoot::Context {
            ty: record,
        },
        vec![crate::runtime_defaults::DefaultReadStep::Field(field)],
        integer,
        jai_source::SourceSpan {
            source,
            span: Span::new(0, 13),
        },
        &types,
    )
    .unwrap();
    let info = ArgumentInfo::runtime_read(read);
    assert!(!info.is_compile_time_constant());
    let mut formal = parameter(names.symbol("value"), TypePattern::Concrete(integer));
    formal.default = Some(info);
    formal.baking = jai_syntax::ParameterBaking::Optional;
    let mut candidate = names.candidate(vec![formal]);
    let matched = match_candidate(&types, &candidate, &[], Span::default()).unwrap();
    assert!(matched.substitution.constants.is_empty());
    assert_eq!(matched.defaults, vec![0]);
    candidate.parameters[0].baking = jai_syntax::ParameterBaking::Required;
    let error = match_candidate(&types, &candidate, &[], Span::default()).unwrap_err();
    assert!(
        error.message.contains("runtime storage defaults"),
        "{error:?}"
    );
}

#[test]
fn only_quoted_string_literals_match_a_nul_terminated_byte_pointer() {
    let mut names = Names::new();
    let mut types = TypeRegistry::default();
    let string = types.string();
    let pointer = types
        .pointer(types.scalar(ScalarType::Int(IntegerType::U8)))
        .unwrap();
    let x = names.symbol("value");
    let pointer_candidate = names.candidate(vec![parameter(x, TypePattern::Concrete(pointer))]);
    let literal = argument(ArgumentInfo::string_literal(
        Box::from(&b"A\0B"[..]),
        string,
    ));
    let matched = select(
        &types,
        from_ref(&pointer_candidate),
        from_ref(&literal),
        Span::default(),
    )
    .unwrap();
    assert_eq!(matched.conversions, vec![ConversionRank::Literal]);
    assert!(
        select(
            &types,
            from_ref(&pointer_candidate),
            &[argument(ArgumentInfo::typed(string))],
            Span::default(),
        )
        .is_err()
    );
    let string_candidate = names.candidate(vec![parameter(x, TypePattern::Concrete(string))]);
    let exact = select(
        &types,
        &[pointer_candidate, string_candidate.clone()],
        &[literal],
        Span::default(),
    )
    .unwrap();
    assert_eq!(exact.declaration, string_candidate.declaration);
}

#[test]
fn quoted_byte_backing_matches_structural_generic_pointer_parameters() {
    let mut names = Names::new();
    let types = TypeRegistry::default();
    let value = names.symbol("value");
    let t = names.symbol("T");
    let candidate = names.candidate(vec![parameter(
        value,
        TypePattern::Pointer(Box::new(TypePattern::Infer(t))),
    )]);
    let matched = select(
        &types,
        from_ref(&candidate),
        &[argument(ArgumentInfo::string_literal(
            Box::from(&b"A\0B"[..]),
            types.string(),
        ))],
        Span::default(),
    )
    .unwrap();
    assert_eq!(
        matched.substitution.ty(t),
        Some(types.scalar(ScalarType::Int(IntegerType::U8)))
    );
    assert_eq!(matched.conversions, vec![ConversionRank::Literal]);
    assert!(
        select(
            &types,
            &[candidate],
            &[argument(ArgumentInfo::typed(types.string()))],
            Span::default(),
        )
        .is_err()
    );
}

#[test]
fn force_cast_matching_requires_the_actual_layout_and_never_bakes_numeric_wrapping() {
    struct TargetLayout;
    impl NominalView for TargetLayout {
        fn specialization(&self, _: TypeId) -> Option<(DeclarationId, &Substitution)> {
            None
        }
        fn layout_policy(&self) -> Option<jai_types::LayoutPolicy> {
            Some(jai_types::LayoutPolicy::lp64())
        }
    }
    let mut names = Names::new();
    let mut types = TypeRegistry::default();
    let value = names.symbol("value");
    let equal = CastMode::Force(jai_types::StorageBitcastStrength::EqualSize);
    let prefix = CastMode::Force(jai_types::StorageBitcastStrength::Prefix);
    let f64 = names.candidate(vec![parameter(
        value,
        TypePattern::Concrete(types.float(FloatType::F64)),
    )]);
    let f32 = names.candidate(vec![parameter(
        value,
        TypePattern::Concrete(types.float(FloatType::F32)),
    )]);
    let args = [argument(ArgumentInfo::contextual_cast(
        equal,
        ArgumentInfo::integer_literal(12),
    ))];
    let selected = select_with_nominals(
        &types,
        &TargetLayout,
        from_ref(&f64),
        &args,
        Span::default(),
    )
    .unwrap();
    assert_eq!(selected.conversions, vec![ConversionRank::Literal]);
    let missing_layout = select(&types, from_ref(&f64), &args, Span::default()).unwrap_err();
    assert!(
        missing_layout
            .diagnostic(Span::default())
            .message
            .contains("target layout")
    );
    assert!(
        select_with_nominals(
            &types,
            &TargetLayout,
            from_ref(&f32),
            &args,
            Span::default()
        )
        .is_err()
    );
    assert!(
        select_with_nominals(
            &types,
            &TargetLayout,
            &[f32],
            &[argument(ArgumentInfo::contextual_cast(
                prefix,
                ArgumentInfo::integer_literal(12)
            ))],
            Span::default(),
        )
        .is_ok()
    );
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    types.pointer(byte).unwrap();
    let pointer = names.candidate(vec![parameter(
        value,
        TypePattern::Pointer(Box::new(TypePattern::Concrete(byte))),
    )]);
    assert!(
        select_with_nominals(&types, &TargetLayout, &[pointer], &args, Span::default()).is_ok()
    );
    let mut baked = f64;
    baked.parameters[0].baking = jai_syntax::ParameterBaking::Required;
    let error =
        select_with_nominals(&types, &TargetLayout, &[baked], &args, Span::default()).unwrap_err();
    assert!(
        error
            .diagnostic(Span::default())
            .message
            .contains("VM constant materialization")
    );
}

#[test]
fn pure_matching_preserves_a_non_graph_callable_origin() {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct LexicalOrigin {
        scope: usize,
        declaration: usize,
    }
    let mut names = Names::new();
    let types = TypeRegistry::default();
    let origin = LexicalOrigin {
        scope: 3,
        declaration: 7,
    };
    let candidate = Candidate {
        result_type_parameters: Vec::new(),
        declaration: origin,
        parameters: vec![parameter(
            names.symbol("value"),
            TypePattern::Concrete(integer(&types, IntegerType::U8)),
        )],
        variadic: CandidateVariadic::None,
    };
    let matched = select(
        &types,
        &[candidate],
        &[argument(ArgumentInfo::integer_literal(12))],
        Span::default(),
    )
    .unwrap();
    assert_eq!(matched.declaration, origin);
    assert_eq!(matched.conversions, vec![ConversionRank::Literal]);
}

#[test]
fn contextual_cast_targets_are_candidate_supplied_and_baked_casts_are_normalized() {
    let mut names = Names::new();
    let types = TypeRegistry::default();
    let x = names.symbol("x");
    let narrow = names.candidate(vec![parameter(
        x,
        TypePattern::Concrete(integer(&types, IntegerType::U8)),
    )]);
    let broad = names.candidate(vec![parameter(
        x,
        TypePattern::Concrete(integer(&types, IntegerType::U16)),
    )]);
    let cast = argument(ArgumentInfo::contextual_cast(
        CastMode::Checked,
        ArgumentInfo::integer_literal(300),
    ));
    assert!(matches!(
        select(
            &types,
            &[narrow.clone(), broad],
            from_ref(&cast),
            Span::default()
        ),
        Err(SelectionError::Ambiguous(_))
    ));
    assert!(select(&types, from_ref(&narrow), from_ref(&cast), Span::default()).is_ok());
    let mut baked = narrow;
    baked.parameters[0].baking = jai_syntax::ParameterBaking::Required;
    assert!(select(&types, from_ref(&baked), &[cast], Span::default()).is_err());
    let wrapping = argument(ArgumentInfo::contextual_cast(
        CastMode::Unchecked,
        ArgumentInfo::integer_literal(300),
    ));
    let matched = select(&types, &[baked], &[wrapping], Span::default()).unwrap();
    assert_eq!(
        matched
            .substitution
            .constant(x)
            .unwrap()
            .as_integer()
            .unwrap()
            .value(),
        44
    );
}

#[test]
fn contextual_casts_use_later_introductions_and_never_infer_their_target() {
    let mut names = Names::new();
    let types = TypeRegistry::default();
    let x = names.symbol("x");
    let y = names.symbol("y");
    let t = names.symbol("T");
    let candidate = names.candidate(vec![
        parameter(x, TypePattern::Variable(t)),
        parameter(y, TypePattern::Infer(t)),
    ]);
    let arguments = [
        argument(ArgumentInfo::contextual_cast(
            CastMode::Unchecked,
            ArgumentInfo::integer_literal(300),
        )),
        argument(ArgumentInfo::typed(integer(&types, IntegerType::U8))),
    ];
    let matched = select(&types, &[candidate], &arguments, Span::default()).unwrap();
    assert_eq!(
        matched.substitution.ty(t),
        Some(integer(&types, IntegerType::U8))
    );
    let candidate = names.candidate(vec![parameter(x, TypePattern::Infer(t))]);
    assert!(select(&types, &[candidate], &arguments[..1], Span::default()).is_err());
}

#[test]
fn truncate_casts_keep_integer_bits_and_reject_float_and_boolean_domains() {
    let mut names = Names::new();
    let types = TypeRegistry::default();
    let name = names.symbol("value");
    let mut narrow = names.candidate(vec![parameter(
        name,
        TypePattern::Concrete(integer(&types, IntegerType::U8)),
    )]);
    narrow.parameters[0].baking = jai_syntax::ParameterBaking::Required;
    let value = argument(ArgumentInfo::contextual_cast(
        CastMode::Truncate,
        ArgumentInfo::integer_literal(300),
    ));
    let matched = select(&types, &[narrow], from_ref(&value), Span::default()).unwrap();
    assert_eq!(
        matched
            .substitution
            .constant(name)
            .unwrap()
            .as_integer()
            .unwrap()
            .value(),
        44
    );
    let boolean = names.candidate(vec![parameter(
        name,
        TypePattern::Concrete(types.scalar(ScalarType::Bool)),
    )]);
    let float = names.candidate(vec![parameter(
        name,
        TypePattern::Concrete(types.float(FloatType::F32)),
    )]);
    assert!(select(&types, &[boolean, float], &[value], Span::default()).is_err());
}

#[test]
fn callback_patterns_infer_results_and_keep_abi_context_and_parameter_types_exact() {
    use jai_types::{CallingConvention, ContextMode, ProcedureType, Variadic};
    let mut names = Names::new();
    let mut types = TypeRegistry::default();
    let t = names.symbol("T");
    let r = names.symbol("R");
    let input = names.symbol("input");
    let callback = names.symbol("callback");
    let u8 = integer(&types, IntegerType::U8);
    let u16 = integer(&types, IntegerType::U16);
    let signature = ProcedureType {
        parameters: vec![u8, u8].into_boxed_slice(),
        results: vec![u16].into_boxed_slice(),
        return_abi: jai_types::ForeignReturnAbi::Natural,
        convention: CallingConvention::Jai,
        context: ContextMode::Implicit,
        variadic: Variadic::None,
    };
    let procedure = types.procedure(signature.clone()).unwrap();
    let candidate = names.candidate(vec![
        parameter(input, TypePattern::Infer(t)),
        parameter(
            callback,
            TypePattern::Procedure(Box::new(ProcedurePattern {
                parameters: vec![TypePattern::Variable(t), TypePattern::Variable(t)],
                results: vec![TypePattern::Infer(r)],
                return_abi: signature.return_abi,
                convention: signature.convention,
                context: signature.context,
                variadic: CandidateVariadic::None,
            })),
        ),
    ]);
    let arguments = [
        argument(ArgumentInfo::typed(u8)),
        argument(ArgumentInfo::typed(procedure)),
    ];
    let matched = select(&types, from_ref(&candidate), &arguments, Span::default()).unwrap();
    assert_eq!(matched.substitution.ty(r), Some(u16));
    let pattern = &candidate.parameters[1].ty;
    assert_eq!(
        materialize(&mut types, pattern, &matched.substitution).unwrap(),
        procedure
    );
    let changed = [
        ProcedureType {
            context: ContextMode::None,
            ..signature.clone()
        },
        ProcedureType {
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::C,
            context: ContextMode::None,
            ..signature.clone()
        },
        ProcedureType {
            parameters: vec![u16, u16].into_boxed_slice(),
            ..signature
        },
    ];
    for signature in changed {
        let procedure = types.procedure(signature).unwrap();
        let arguments = [
            argument(ArgumentInfo::typed(u8)),
            argument(ArgumentInfo::typed(procedure)),
        ];
        assert!(select(&types, from_ref(&candidate), &arguments, Span::default()).is_err());
    }
}

#[test]
fn contextual_callback_inference_requires_known_parameters_and_a_unique_result() {
    use jai_types::{CallingConvention, ContextMode, ProcedureType, Variadic};
    let mut names = Names::new();
    let mut types = TypeRegistry::default();
    let t = names.symbol("T");
    let r = names.symbol("R");
    let callback = names.symbol("callback");
    let input = names.symbol("input");
    let u8 = integer(&types, IntegerType::U8);
    let u16 = integer(&types, IntegerType::U16);
    let pattern = TypePattern::Procedure(Box::new(ProcedurePattern {
        parameters: vec![TypePattern::Variable(t)],
        results: vec![TypePattern::Infer(r)],
        return_abi: jai_types::ForeignReturnAbi::Natural,
        convention: CallingConvention::Jai,
        context: ContextMode::Implicit,
        variadic: CandidateVariadic::None,
    }));
    let candidate = names.candidate(vec![
        parameter(callback, pattern.clone()),
        parameter(input, TypePattern::Infer(t)),
    ]);
    let descriptor = ProcedureType {
        parameters: vec![u8].into_boxed_slice(),
        results: vec![u16].into_boxed_slice(),
        return_abi: jai_types::ForeignReturnAbi::Natural,
        convention: CallingConvention::Jai,
        context: ContextMode::Implicit,
        variadic: Variadic::None,
    };
    let signature = types.procedure(descriptor.clone()).unwrap();
    let lambda = |signatures: Vec<TypeId>| {
        argument(ArgumentInfo {
            ty: ArgumentType::ContextualProcedure {
                compatible_signatures: signatures.into_boxed_slice(),
            },
            constant: None,
        })
    };
    let matched = match_candidate(
        &types,
        &candidate,
        &[lambda(vec![signature]), argument(ArgumentInfo::typed(u8))],
        Span::default(),
    )
    .unwrap();
    assert_eq!(matched.substitution.ty(r), Some(u16));
    let other = types
        .procedure(ProcedureType {
            results: vec![types.scalar(ScalarType::Bool)].into_boxed_slice(),
            ..descriptor
        })
        .unwrap();
    assert!(
        match_candidate(
            &types,
            &candidate,
            &[
                lambda(vec![signature, other]),
                argument(ArgumentInfo::typed(u8))
            ],
            Span::default()
        )
        .unwrap_err()
        .message
        .contains("ambiguous")
    );
    let unknown = names.candidate(vec![parameter(callback, pattern)]);
    assert!(
        match_candidate(
            &types,
            &unknown,
            &[lambda(vec![signature])],
            Span::default()
        )
        .is_err(),
        "a contextual target must not guess its own parameter type"
    );
}

#[test]
fn discarded_arguments_keep_inference_and_ranking_but_have_no_runtime_destination() {
    let mut names = Names::new();
    let types = TypeRegistry::default();
    let t = names.symbol("T");
    let ignored = names.symbol("ignored");
    let value = names.symbol("value");
    let mut discarded = parameter(ignored, TypePattern::Infer(t));
    discarded.evaluation = jai_syntax::ParameterEvaluation::Discard;
    let candidate = names.candidate(vec![discarded, parameter(value, TypePattern::Variable(t))]);
    let arguments = [
        argument(ArgumentInfo::typed(integer(&types, IntegerType::U8))),
        argument(ArgumentInfo::integer_literal(5)),
    ];
    let matched = select(&types, &[candidate], &arguments, Span::default()).unwrap();
    assert_eq!(
        matched
            .bindings
            .iter()
            .map(|binding| binding.runtime_parameter)
            .collect::<Vec<_>>(),
        vec![None, Some(0)]
    );
    assert_eq!(
        matched.conversions,
        vec![ConversionRank::Exact, ConversionRank::Literal]
    );
    assert_eq!(
        matched.substitution.ty(t),
        Some(integer(&types, IntegerType::U8))
    );
}
fn named(name: Symbol, info: ArgumentInfo) -> Argument {
    Argument {
        name: Some(name),
        spread: false,
        info,
        span: Span::default(),
    }
}
fn integer(types: &TypeRegistry, ty: IntegerType) -> TypeId {
    types.scalar(ScalarType::Int(ty))
}

struct NominalInstances(HashMap<TypeId, (DeclarationId, Substitution)>);
impl NominalView for NominalInstances {
    fn specialization(&self, ty: TypeId) -> Option<(DeclarationId, &Substitution)> {
        self.0
            .get(&ty)
            .map(|(origin, substitution)| (*origin, substitution))
    }
}

struct LiteralMetadata {
    fields: HashMap<TypeId, Vec<LiteralField>>,
    defaults: HashMap<jai_types::FieldId, ConstantValue>,
}
impl NominalView for LiteralMetadata {
    fn specialization(&self, _: TypeId) -> Option<(DeclarationId, &Substitution)> {
        None
    }
    fn record_fields(&self, ty: TypeId, span: Span) -> Result<Vec<LiteralField>, Diagnostic> {
        self.fields
            .get(&ty)
            .cloned()
            .ok_or_else(|| Diagnostic::new(span, "unknown literal record"))
    }
    fn field_default(
        &self,
        field: jai_types::FieldId,
        span: Span,
    ) -> Result<ConstantValue, Diagnostic> {
        self.defaults
            .get(&field)
            .cloned()
            .ok_or_else(|| Diagnostic::new(span, "required literal field"))
    }
}

#[test]
fn contextual_records_validate_fields_and_canonicalize_baked_defaults() {
    let mut types = TypeRegistry::new();
    let mut names = Names::new();
    let value = names.symbol("value");
    let extra = names.symbol("extra");
    let config = names.symbol("config");
    let record = types.reserve_record(RecordKind::Struct);
    let wrong = types.reserve_record(RecordKind::Struct);
    let u8 = integer(&types, IntegerType::U8);
    let s64 = integer(&types, IntegerType::S64);
    types.define_record(record, [u8, s64]).unwrap();
    types
        .define_record(wrong, [types.scalar(ScalarType::Bool)])
        .unwrap();
    let fields = [
        types.field(record, 0).unwrap(),
        types.field(record, 1).unwrap(),
    ];
    let metadata = LiteralMetadata {
        fields: HashMap::from([
            (
                record,
                vec![
                    LiteralField {
                        name: Some(value),
                        id: fields[0].id,
                        ty: u8,
                    },
                    LiteralField {
                        name: Some(extra),
                        id: fields[1].id,
                        ty: s64,
                    },
                ],
            ),
            (
                wrong,
                vec![LiteralField {
                    name: Some(value),
                    id: types.field(wrong, 0).unwrap().id,
                    ty: types.scalar(ScalarType::Bool),
                }],
            ),
        ]),
        defaults: HashMap::from([(
            fields[1].id,
            ConstantValue {
                ty: s64,
                kind: ConstantKind::Int(Integer::checked(IntegerType::S64, 6).unwrap()),
            },
        )]),
    };
    let literal = |ty, fields| ArgumentInfo {
        ty: ArgumentType::RecordLiteral {
            ty,
            fields,
        },
        constant: None,
    };
    let field = |name, value| RecordArgumentField {
        target: RecordArgumentTarget::field(name),
        value: ArgumentInfo::integer_literal(value),
        span: Span::default(),
    };
    let contextual = literal(None, vec![field(value, 7)]);
    let candidate = names.candidate(vec![parameter(config, TypePattern::Concrete(record))]);
    let mismatch = names.candidate(vec![parameter(config, TypePattern::Concrete(wrong))]);
    assert_eq!(
        select_with_nominals(
            &types,
            &metadata,
            &[mismatch, candidate.clone()],
            &[argument(contextual.clone())],
            Span::default()
        )
        .unwrap()
        .declaration,
        candidate.declaration
    );
    let mut baked = candidate;
    baked.parameters[0].baking = jai_syntax::ParameterBaking::Required;
    let a = match_candidate_with_nominals(
        &types,
        &metadata,
        &baked,
        &[argument(contextual)],
        Span::default(),
    )
    .unwrap();
    let b = match_candidate_with_nominals(
        &types,
        &metadata,
        &baked,
        &[argument(literal(
            Some(record),
            vec![field(extra, 6), field(value, 7)],
        ))],
        Span::default(),
    )
    .unwrap();
    assert_eq!(
        a.substitution.key(a.declaration),
        b.substitution.key(b.declaration)
    );
    assert!(
        match_candidate_with_nominals(
            &types,
            &metadata,
            &baked,
            &[argument(literal(None, vec![field(value, 256)]))],
            Span::default()
        )
        .is_err()
    );
    assert!(
        match_candidate_with_nominals(
            &types,
            &metadata,
            &baked,
            &[argument(literal(
                None,
                vec![field(value, 7), field(value, 7)]
            ))],
            Span::default()
        )
        .is_err()
    );
}

#[test]
fn contextual_array_count_inference_is_independent_of_a_default_element_type() {
    let types = TypeRegistry::new();
    let mut names = Names::new();
    let values = names.symbol("values");
    let count = names.symbol("Count");
    let candidate = names.candidate(vec![parameter(
        values,
        TypePattern::FixedArray {
            element: Box::new(TypePattern::Concrete(integer(&types, IntegerType::U8))),
            count: CountPattern::Infer(count),
        },
    )]);
    let empty = ArgumentInfo {
        ty: ArgumentType::ArrayLiteral {
            explicit: None,
            default: None,
            elements: vec![],
        },
        constant: None,
    };
    let matched = match_candidate(&types, &candidate, &[argument(empty)], Span::default()).unwrap();
    assert_eq!(
        matched
            .substitution
            .constant(count)
            .unwrap()
            .as_integer()
            .unwrap()
            .value(),
        0
    );
    let invalid = ArgumentInfo {
        ty: ArgumentType::ArrayLiteral {
            explicit: None,
            default: None,
            elements: vec![
                ArgumentInfo::integer_literal(7),
                ArgumentInfo::integer_literal(256),
            ],
        },
        constant: None,
    };
    assert!(match_candidate(&types, &candidate, &[argument(invalid)], Span::default()).is_err());
}

#[test]
fn baked_union_constants_retain_the_selected_alternative() {
    let mut types = TypeRegistry::new();
    let union = types.reserve_record(RecordKind::Union);
    let u8 = integer(&types, IntegerType::U8);
    types.define_record(union, [u8, u8]).unwrap();
    let value = || ConstantValue {
        ty: u8,
        kind: ConstantKind::Int(Integer::checked(IntegerType::U8, 7).unwrap()),
    };
    let a = BakedValue::runtime(
        ConstantValue {
            ty: union,
            kind: ConstantKind::Union {
                field: types.field(union, 0).unwrap().id,
                value: Box::new(value()),
            },
        },
        &types,
    )
    .unwrap();
    let b = BakedValue::runtime(
        ConstantValue {
            ty: union,
            kind: ConstantKind::Union {
                field: types.field(union, 1).unwrap().id,
                value: Box::new(value()),
            },
        },
        &types,
    )
    .unwrap();
    assert_ne!(a, b);
}

#[test]
fn nominal_patterns_infer_typed_arguments_and_preserve_the_origin() {
    let mut types = TypeRegistry::new();
    let mut names = Names::new();
    let value = names.symbol("value");
    let element = names.symbol("Element");
    let count = names.symbol("Count");
    let t = names.symbol("T");
    let n = names.symbol("N");
    let origin = names.identities.declaration();
    let unrelated_origin = names.identities.declaration();
    let record = types.reserve_record(RecordKind::Struct);
    let unrelated = types.reserve_record(RecordKind::Struct);
    types
        .define_record(record, [integer(&types, IntegerType::U8)])
        .unwrap();
    types
        .define_record(unrelated, [integer(&types, IntegerType::U8)])
        .unwrap();
    let mut nominal_substitution = Substitution::default();
    nominal_substitution.bind_constant(element, BakedValue::Type(integer(&types, IntegerType::U8)));
    nominal_substitution.bind_constant(
        count,
        BakedValue::integer(Integer::checked(IntegerType::S64, 3).unwrap(), &types),
    );
    let instances = NominalInstances(HashMap::from([
        (record, (origin, nominal_substitution.clone())),
        (unrelated, (unrelated_origin, nominal_substitution)),
    ]));
    let pattern = TypePattern::NominalApplication {
        declaration: origin,
        arguments: vec![
            NominalArgumentPattern {
                name: element,
                kind: NominalArgumentKind::Type(Box::new(TypePattern::Infer(t))),
            },
            NominalArgumentPattern {
                name: count,
                kind: NominalArgumentKind::InferValue(n),
            },
        ],
    };
    let candidate = names.candidate(vec![parameter(value, pattern.clone())]);
    let matched = select_with_nominals(
        &types,
        &instances,
        from_ref(&candidate),
        &[argument(ArgumentInfo::typed(record))],
        Span::default(),
    )
    .unwrap();
    assert_eq!(
        matched.substitution.ty(t),
        Some(integer(&types, IntegerType::U8))
    );
    assert_eq!(
        matched
            .substitution
            .constant(n)
            .unwrap()
            .as_integer()
            .unwrap()
            .value(),
        3
    );
    assert!(
        select_with_nominals(
            &types,
            &instances,
            &[candidate],
            &[argument(ArgumentInfo::typed(unrelated))],
            Span::default()
        )
        .is_err()
    );
    let mut materialized = None;
    let ty = crate::polymorphism::materialize_with_nominals(
        &mut types,
        &pattern,
        &matched.substitution,
        &mut |_, declaration, arguments| {
            materialized = Some((declaration, arguments));
            Ok(record)
        },
    )
    .unwrap();
    assert_eq!(ty, record);
    let (declaration, arguments) = materialized.unwrap();
    assert_eq!(declaration, origin);
    assert_eq!(arguments.ty(element), matched.substitution.ty(t));
    assert_eq!(arguments.constant(count), matched.substitution.constant(n));
}

#[test]
fn forwarded_generic_pack_infers_its_element_and_binds_trailing_arguments() {
    let mut types = TypeRegistry::new();
    let mut names = Names::new();
    let rest = names.symbol("rest");
    let last = names.symbol("last");
    let t = names.symbol("T");
    let slice = types.slice(integer(&types, IntegerType::U16)).unwrap();
    let mut candidate = names.candidate(vec![
        parameter(rest, TypePattern::Infer(t)),
        parameter(last, TypePattern::Concrete(types.scalar(ScalarType::Bool))),
    ]);
    candidate.variadic = CandidateVariadic::Jai {
        parameter: 0,
    };
    let mut forwarded = argument(ArgumentInfo::typed(slice));
    forwarded.spread = true;
    let args = [
        forwarded.clone(),
        argument(ArgumentInfo::typed(types.scalar(ScalarType::Bool))),
    ];
    let matched = select(&types, from_ref(&candidate), &args, Span::default()).unwrap();
    assert_eq!(
        matched.substitution.ty(t),
        Some(integer(&types, IntegerType::U16))
    );
    assert_eq!(
        matched
            .bindings
            .iter()
            .map(|binding| binding.parameter)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    let args = [
        argument(ArgumentInfo::typed(integer(&types, IntegerType::U16))),
        forwarded.clone(),
        argument(ArgumentInfo::typed(types.scalar(ScalarType::Bool))),
    ];
    let matched = match_candidate(&types, &candidate, &args, Span::default()).unwrap();
    assert_eq!(
        matched
            .bindings
            .iter()
            .map(|binding| binding.parameter)
            .collect::<Vec<_>>(),
        [0, 0, 1]
    );
    assert_eq!(
        matched.substitution.ty(t),
        Some(integer(&types, IntegerType::U16))
    );
    let args = [forwarded.clone(), forwarded];
    assert!(
        match_candidate(&types, &candidate, &args, Span::default())
            .unwrap_err()
            .message
            .contains("spread")
    );
}

#[test]
fn universal_boxing_ranks_after_typed_conversions_and_rejects_metadata() {
    let mut types = TypeRegistry::new();
    let header = types.reserve_record(RecordKind::Struct);
    types.define_record(header, []).unwrap();
    let any = types.reserve_any();
    types.define_any(any, header).unwrap();
    let mut names = Names::new();
    let value = names.symbol("value");
    let concrete = names.candidate(vec![parameter(
        value,
        TypePattern::Concrete(integer(&types, IntegerType::S32)),
    )]);
    let universal = names.candidate(vec![parameter(value, TypePattern::Concrete(any))]);
    let args = [argument(ArgumentInfo::typed(integer(
        &types,
        IntegerType::S16,
    )))];
    assert_eq!(
        select(
            &types,
            &[universal.clone(), concrete.clone()],
            &args,
            Span::default()
        )
        .unwrap()
        .declaration,
        concrete.declaration
    );
    assert_eq!(
        match_candidate(
            &types,
            &universal,
            &[argument(ArgumentInfo::integer_literal(7))],
            Span::default()
        )
        .unwrap()
        .conversions,
        [ConversionRank::Boxing]
    );
    assert_eq!(
        match_candidate(
            &types,
            &universal,
            &[argument(ArgumentInfo::typed(any))],
            Span::default()
        )
        .unwrap()
        .conversions,
        [ConversionRank::Exact]
    );
    assert_eq!(
        match_candidate(
            &types,
            &universal,
            &[argument(ArgumentInfo::typed(types.meta_type()))],
            Span::default()
        )
        .unwrap()
        .conversions,
        [ConversionRank::Boxing],
        "runtime Type storage is a canonical descriptor pointer"
    );
    for metadata in [types.code_type(), types.void()] {
        assert!(
            match_candidate(
                &types,
                &universal,
                &[argument(ArgumentInfo::typed(metadata))],
                Span::default()
            )
            .unwrap_err()
            .message
            .contains("metadata")
        );
    }
    let incomplete = types.reserve_record(RecordKind::Struct);
    assert!(
        match_candidate(
            &types,
            &universal,
            &[argument(ArgumentInfo::typed(incomplete))],
            Span::default()
        )
        .is_err()
    );
}

#[test]
fn exact_match_dominates_widening_independently_of_declaration_order() {
    let types = TypeRegistry::new();
    let mut names = Names::new();
    let a = names.symbol("a");
    let exact = names.candidate(vec![parameter(
        a,
        TypePattern::Concrete(integer(&types, IntegerType::U8)),
    )]);
    let wider = names.candidate(vec![parameter(
        a,
        TypePattern::Concrete(integer(&types, IntegerType::S64)),
    )]);
    let args = [argument(ArgumentInfo::typed(integer(
        &types,
        IntegerType::U8,
    )))];
    for candidates in [
        vec![exact.clone(), wider.clone()],
        vec![wider.clone(), exact.clone()],
    ] {
        assert_eq!(
            select(&types, &candidates, &args, Span::default())
                .unwrap()
                .declaration,
            exact.declaration
        );
    }
}

#[test]
fn weak_literals_keep_default_type_and_checked_adaptation() {
    let types = TypeRegistry::new();
    let mut names = Names::new();
    let a = names.symbol("a");
    let narrow = names.candidate(vec![parameter(
        a,
        TypePattern::Concrete(integer(&types, IntegerType::U8)),
    )]);
    let default = names.candidate(vec![parameter(
        a,
        TypePattern::Concrete(integer(&types, IntegerType::S64)),
    )]);
    assert_eq!(
        select(
            &types,
            &[narrow.clone(), default.clone()],
            &[argument(ArgumentInfo::integer_literal(42))],
            Span::default()
        )
        .unwrap()
        .declaration,
        default.declaration
    );
    assert!(
        match_candidate(
            &types,
            &narrow,
            &[argument(ArgumentInfo::integer_literal(256))],
            Span::default()
        )
        .unwrap_err()
        .message
        .contains("out of range")
    );
    let conditional = ArgumentInfo {
        ty: ArgumentType::WeakInteger {
            minimum: 1,
            maximum: 255,
        },
        constant: None,
    };
    assert_eq!(
        match_candidate(&types, &narrow, &[argument(conditional)], Span::default())
            .unwrap()
            .conversions,
        vec![ConversionRank::Literal]
    );
}

#[test]
fn generic_inference_uses_introducing_parameter_not_named_argument_order() {
    let types = TypeRegistry::new();
    let mut names = Names::new();
    let a = names.symbol("a");
    let b = names.symbol("b");
    let t = names.symbol("T");
    let candidate = names.candidate(vec![
        parameter(a, TypePattern::Infer(t)),
        parameter(b, TypePattern::Variable(t)),
    ]);
    let args = [
        named(b, ArgumentInfo::integer_literal(2)),
        named(a, ArgumentInfo::typed(integer(&types, IntegerType::S8))),
    ];
    let matched = match_candidate(&types, &candidate, &args, Span::default()).unwrap();
    assert_eq!(
        matched.substitution.ty(t),
        Some(integer(&types, IntegerType::S8))
    );
    assert_eq!(
        matched
            .bindings
            .iter()
            .map(|binding| (
                binding.argument,
                binding.parameter,
                binding.runtime_parameter
            ))
            .collect::<Vec<_>>(),
        vec![(0, 1, Some(1)), (1, 0, Some(0))]
    );
    let args = [
        named(b, ArgumentInfo::typed(integer(&types, IntegerType::S16))),
        named(a, ArgumentInfo::typed(integer(&types, IntegerType::S8))),
    ];
    assert!(match_candidate(&types, &candidate, &args, Span::default()).is_err());
}

#[test]
fn a_type_use_may_precede_its_introduction_and_does_not_bind_it() {
    let types = TypeRegistry::new();
    let mut names = Names::new();
    let a = names.symbol("a");
    let b = names.symbol("b");
    let t = names.symbol("T");
    let candidate = names.candidate(vec![
        parameter(a, TypePattern::Variable(t)),
        parameter(b, TypePattern::Infer(t)),
    ]);
    let args = [
        argument(ArgumentInfo::typed(integer(&types, IntegerType::U8))),
        argument(ArgumentInfo::typed(integer(&types, IntegerType::U16))),
    ];
    let matched = match_candidate(&types, &candidate, &args, Span::default()).unwrap();
    assert_eq!(
        matched.substitution.ty(t),
        Some(integer(&types, IntegerType::U16))
    );
    assert_eq!(
        matched.conversions,
        vec![ConversionRank::Widening, ConversionRank::Exact]
    );
}

#[test]
fn null_accepts_a_pointer_context_but_does_not_infer_a_pointee() {
    let mut types = TypeRegistry::new();
    let mut names = Names::new();
    let pointer = types.pointer(integer(&types, IntegerType::S64)).unwrap();
    let a = names.symbol("a");
    let b = names.symbol("b");
    let t = names.symbol("T");
    let concrete = names.candidate(vec![parameter(a, TypePattern::Concrete(pointer))]);
    assert!(
        match_candidate(
            &types,
            &concrete,
            &[argument(ArgumentInfo::null())],
            Span::default()
        )
        .is_ok()
    );
    let generic = names.candidate(vec![
        parameter(a, TypePattern::Pointer(Box::new(TypePattern::Infer(t)))),
        parameter(b, TypePattern::Variable(t)),
    ]);
    assert!(
        match_candidate(
            &types,
            &generic,
            &[
                argument(ArgumentInfo::null()),
                argument(ArgumentInfo::integer_literal(3))
            ],
            Span::default()
        )
        .is_err()
    );
    let dependent = names.candidate(vec![
        parameter(a, TypePattern::Pointer(Box::new(TypePattern::Variable(t)))),
        parameter(b, TypePattern::Infer(t)),
    ]);
    assert!(
        match_candidate(
            &types,
            &dependent,
            &[
                argument(ArgumentInfo::null()),
                argument(ArgumentInfo::integer_literal(3))
            ],
            Span::default()
        )
        .is_ok()
    );
}

#[test]
fn decimal_context_rounds_directly_to_the_selected_width() {
    let types = TypeRegistry::new();
    let mut names = Names::new();
    let a = names.symbol("a");
    let narrow = names.candidate(vec![parameter(
        a,
        TypePattern::Concrete(types.float(FloatType::F32)),
    )]);
    let wide = names.candidate(vec![parameter(
        a,
        TypePattern::Concrete(types.float(FloatType::F64)),
    )]);
    let args = [argument(ArgumentInfo::decimal_literal(
        "3.141592653589793".into(),
        false,
        FloatType::F64,
    ))];
    let matched = select(&types, &[narrow, wide.clone()], &args, Span::default()).unwrap();
    assert_eq!(matched.declaration, wide.declaration);
    let value = bake(
        &types,
        &wide.parameters[0].ty,
        &args[0].info,
        &matched.substitution,
        Span::default(),
    )
    .unwrap();
    assert_eq!(
        value,
        BakedValue::Float(FloatValue::parse_decimal(FloatType::F64, "3.141592653589793").unwrap())
    );
}

#[test]
fn unrestricted_generic_and_concrete_exact_match_remain_ambiguous() {
    let types = TypeRegistry::new();
    let mut names = Names::new();
    let a = names.symbol("a");
    let t = names.symbol("T");
    let generic = names.candidate(vec![parameter(a, TypePattern::Infer(t))]);
    let concrete = names.candidate(vec![parameter(
        a,
        TypePattern::Concrete(integer(&types, IntegerType::S64)),
    )]);
    assert!(
        matches!(select(&types, &[generic, concrete], &[argument(ArgumentInfo::typed(integer(&types, IntegerType::S64)))], Span::default()), Err(SelectionError::Ambiguous(found)) if found.len() == 2)
    );
}

#[test]
fn incomparable_argument_conversions_are_ambiguous() {
    let types = TypeRegistry::new();
    let mut names = Names::new();
    let a = names.symbol("a");
    let b = names.symbol("b");
    let u8 = TypePattern::Concrete(integer(&types, IntegerType::U8));
    let u16 = TypePattern::Concrete(integer(&types, IntegerType::U16));
    let left = names.candidate(vec![parameter(a, u8.clone()), parameter(b, u16.clone())]);
    let right = names.candidate(vec![parameter(a, u16), parameter(b, u8)]);
    let args = [
        argument(ArgumentInfo::typed(integer(&types, IntegerType::U8))),
        argument(ArgumentInfo::typed(integer(&types, IntegerType::U8))),
    ];
    assert!(matches!(
        select(&types, &[left, right], &args, Span::default()),
        Err(SelectionError::Ambiguous(_))
    ));
}

#[test]
fn repeated_introductions_and_duplicate_named_arguments_reject() {
    let types = TypeRegistry::new();
    let mut names = Names::new();
    let a = names.symbol("a");
    let b = names.symbol("b");
    let t = names.symbol("T");
    let invalid = names.candidate(vec![
        parameter(a, TypePattern::Infer(t)),
        parameter(b, TypePattern::Infer(t)),
    ]);
    assert!(
        match_candidate(&types, &invalid, &[], Span::default())
            .unwrap_err()
            .message
            .contains("introduced more than once")
    );
    let valid = names.candidate(vec![parameter(
        a,
        TypePattern::Concrete(integer(&types, IntegerType::S64)),
    )]);
    let args = [
        named(a, ArgumentInfo::integer_literal(1)),
        named(a, ArgumentInfo::integer_literal(2)),
    ];
    assert!(
        match_candidate(&types, &valid, &args, Span::default())
            .unwrap_err()
            .message
            .contains("duplicate argument")
    );
}

#[test]
fn omitted_default_can_introduce_a_type_before_other_arguments_match() {
    let types = TypeRegistry::new();
    let mut names = Names::new();
    let a = names.symbol("a");
    let b = names.symbol("b");
    let t = names.symbol("T");
    let mut introduction = parameter(a, TypePattern::Infer(t));
    introduction.default = Some(ArgumentInfo::integer_literal(4));
    let candidate = names.candidate(vec![introduction, parameter(b, TypePattern::Variable(t))]);
    let matched = match_candidate(
        &types,
        &candidate,
        &[named(
            b,
            ArgumentInfo::typed(integer(&types, IntegerType::U8)),
        )],
        Span::default(),
    )
    .unwrap();
    assert_eq!(
        matched.substitution.ty(t),
        Some(integer(&types, IntegerType::S64))
    );
    assert_eq!(matched.defaults, vec![0]);
}

#[test]
fn nested_pointer_and_array_patterns_preserve_nominal_identity_and_no_element_widening() {
    let mut types = TypeRegistry::new();
    let mut names = Names::new();
    let a = names.symbol("a");
    let t = names.symbol("T");
    let n = names.symbol("N");
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, vec![]).unwrap();
    let array = types.fixed_array(record, 3).unwrap();
    let pointer = types.pointer(array).unwrap();
    let pattern = TypePattern::Pointer(Box::new(TypePattern::FixedArray {
        element: Box::new(TypePattern::Infer(t)),
        count: CountPattern::Infer(n),
    }));
    let candidate = names.candidate(vec![parameter(a, pattern.clone())]);
    let matched = match_candidate(
        &types,
        &candidate,
        &[argument(ArgumentInfo::typed(pointer))],
        Span::default(),
    )
    .unwrap();
    assert_eq!(matched.substitution.ty(t), Some(record));
    assert_eq!(
        matched
            .substitution
            .constant(n)
            .unwrap()
            .as_integer()
            .unwrap()
            .value(),
        3
    );
    assert_eq!(
        materialize(&mut types, &pattern, &matched.substitution).unwrap(),
        pointer
    );
    let source = types.pointer(integer(&types, IntegerType::U8)).unwrap();
    let candidate = names.candidate(vec![parameter(
        a,
        TypePattern::Pointer(Box::new(TypePattern::Concrete(integer(
            &types,
            IntegerType::U16,
        )))),
    )]);
    assert!(
        match_candidate(
            &types,
            &candidate,
            &[argument(ArgumentInfo::typed(source))],
            Span::default()
        )
        .is_err()
    );
}

#[test]
fn baked_type_and_count_parameters_bind_before_dependent_array_patterns() {
    let mut types = TypeRegistry::new();
    let mut names = Names::new();
    let values = names.symbol("values");
    let t = names.symbol("T");
    let n = names.symbol("N");
    let type_parameter = Parameter {
        evaluation: jai_syntax::ParameterEvaluation::Evaluate,
        name: t,
        ty: TypePattern::Concrete(types.meta_type()),
        default: None,
        baking: jai_syntax::ParameterBaking::Required,
    };
    let count_parameter = Parameter {
        evaluation: jai_syntax::ParameterEvaluation::Evaluate,
        name: n,
        ty: TypePattern::Concrete(integer(&types, IntegerType::S64)),
        default: None,
        baking: jai_syntax::ParameterBaking::Required,
    };
    let candidate = names.candidate(vec![
        parameter(
            values,
            TypePattern::FixedArray {
                element: Box::new(TypePattern::Variable(t)),
                count: CountPattern::Variable(n),
            },
        ),
        type_parameter,
        count_parameter,
    ]);
    let u8 = integer(&types, IntegerType::U8);
    let array = types.fixed_array(u8, 3).unwrap();
    let args = [
        argument(ArgumentInfo::typed(array)),
        argument(ArgumentInfo::constant(
            BakedValue::Type(u8),
            types.meta_type(),
        )),
        argument(ArgumentInfo::integer_literal(3)),
    ];
    let matched = match_candidate(&types, &candidate, &args, Span::default()).unwrap();
    assert_eq!(matched.substitution.ty(t), Some(u8));
    assert_eq!(
        matched
            .bindings
            .iter()
            .map(|binding| binding.runtime_parameter)
            .collect::<Vec<_>>(),
        vec![Some(0), None, None]
    );
    assert_eq!(
        matched
            .substitution
            .constant(n)
            .unwrap()
            .as_integer()
            .unwrap()
            .value(),
        3
    );
}

#[test]
fn runtime_values_cannot_supply_baked_arguments_even_when_their_ranges_are_known() {
    let types = TypeRegistry::new();
    let mut names = Names::new();
    let n = names.symbol("N");
    let candidate = names.candidate(vec![Parameter {
        evaluation: jai_syntax::ParameterEvaluation::Evaluate,
        name: n,
        ty: TypePattern::Concrete(integer(&types, IntegerType::S64)),
        default: None,
        baking: jai_syntax::ParameterBaking::Required,
    }]);
    let value = ArgumentInfo {
        ty: ArgumentType::WeakInteger {
            minimum: 3,
            maximum: 3,
        },
        constant: None,
    };
    assert!(
        match_candidate(&types, &candidate, &[argument(value)], Span::default())
            .unwrap_err()
            .message
            .contains("compile-time constant")
    );
}

#[test]
fn baked_keys_normalize_zero_and_retain_scalar_and_nominal_type_identities() {
    let mut types = TypeRegistry::new();
    let mut names = Names::new();
    let n = names.symbol("N");
    let declaration = names.identities.declaration();
    let u8 = integer(&types, IntegerType::U8);
    let u16 = integer(&types, IntegerType::U16);
    let zero = BakedValue::runtime(
        ConstantValue {
            ty: u8,
            kind: ConstantKind::Zero,
        },
        &types,
    )
    .unwrap();
    let explicit = BakedValue::integer(Integer::checked(IntegerType::U8, 0).unwrap(), &types);
    assert_eq!(zero, explicit);
    let key = |value| SpecializationKey {
        declaration,
        substitution: Substitution {
            types: vec![],
            constants: vec![ConstantBinding {
                name: n,
                value,
            }],
            callables: vec![],
        },
    };
    assert_ne!(
        key(explicit),
        key(BakedValue::integer(
            Integer::checked(IntegerType::U16, 0).unwrap(),
            &types
        ))
    );
    let first = types.reserve_record(RecordKind::Struct);
    let second = types.reserve_record(RecordKind::Struct);
    types.define_record(first, vec![u8]).unwrap();
    types.define_record(second, vec![u8]).unwrap();
    let record_zero = BakedValue::runtime(
        ConstantValue {
            ty: first,
            kind: ConstantKind::Zero,
        },
        &types,
    )
    .unwrap();
    let explicit_zero = BakedValue::runtime(
        ConstantValue {
            ty: first,
            kind: ConstantKind::Record(vec![ConstantValue {
                ty: u8,
                kind: ConstantKind::Zero,
            }]),
        },
        &types,
    )
    .unwrap();
    assert_eq!(record_zero, explicit_zero);
    assert_ne!(
        key(record_zero),
        key(BakedValue::runtime(
            ConstantValue {
                ty: second,
                kind: ConstantKind::Zero
            },
            &types
        )
        .unwrap())
    );
    assert_ne!(
        Substitution {
            types: vec![TypeBinding {
                name: n,
                ty: u8
            }],
            constants: vec![],
            callables: vec![],
        }
        .key(declaration),
        Substitution {
            types: vec![TypeBinding {
                name: n,
                ty: u16
            }],
            constants: vec![],
            callables: vec![],
        }
        .key(declaration)
    );
}


#[test]
fn result_type_inputs_keep_original_formal_and_argument_positions() {
    let mut names = Names::new();
    let types = TypeRegistry::new();
    let t = names.symbol("T");
    let value = names.symbol("value");
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let mut candidate = names.candidate(vec![parameter(value, TypePattern::Variable(t))]);
    candidate.result_type_parameters.push(ResultTypeParameter {
        name: t,
        result: 0,
        span: Span::default(),
        pattern: TypePattern::Infer(t),
    });
    let mut ty = argument(ArgumentInfo::constant(
        BakedValue::Type(int),
        types.meta_type(),
    ));
    ty.name = Some(t);
    let arguments = [argument(ArgumentInfo::integer_literal(42)), ty];
    let matched = match_candidate(&types, &candidate, &arguments, Span::default()).unwrap();
    assert_eq!(matched.substitution.ty(t), Some(int));
    assert!(matched.substitution.constants.is_empty());
    assert_eq!(
        matched.bindings,
        [ArgumentBinding {
            argument: 0,
            parameter: 0,
            runtime_parameter: Some(0)
        }]
    );
    assert_eq!(
        matched.result_type_arguments,
        [ResultTypeArgument {
            argument: 1,
            parameter: 0
        }]
    );
    recheck_match_with_nominals(
        &types,
        &NoNominals,
        &candidate,
        &arguments,
        matched,
        Span::default(),
    )
    .unwrap();
}
#[test]
fn result_type_input_order_is_canonical_and_named_order_is_irrelevant() {
    let mut names = Names::new();
    let types = TypeRegistry::new();
    let a = names.symbol("A");
    let b = names.symbol("B");
    let mut candidate = names.candidate(Vec::new());
    candidate.result_type_parameters = [a, b]
        .into_iter()
        .enumerate()
        .map(|(result, name)| ResultTypeParameter {
            name,
            result,
            span: Span::default(),
            pattern: TypePattern::Infer(name),
        })
        .collect();
    let mut arg_a = argument(ArgumentInfo::constant(
        BakedValue::Type(types.scalar(ScalarType::Int(IntegerType::S64))),
        types.meta_type(),
    ));
    arg_a.name = Some(a);
    let mut arg_b = argument(ArgumentInfo::constant(
        BakedValue::Type(types.scalar(ScalarType::Bool)),
        types.meta_type(),
    ));
    arg_b.name = Some(b);
    let first = match_candidate(
        &types,
        &candidate,
        &[arg_a.clone(), arg_b.clone()],
        Span::default(),
    )
    .unwrap();
    let second = match_candidate(&types, &candidate, &[arg_b, arg_a], Span::default()).unwrap();
    assert_eq!(first.substitution, second.substitution);
    assert!(first.bindings.is_empty());
    assert!(second.bindings.is_empty());
}
#[test]
fn result_type_inputs_cannot_replace_the_original_accepted_projection() {
    let mut names = Names::new();
    let types = TypeRegistry::new();
    let t = names.symbol("T");
    let mut candidate = names.candidate(Vec::new());
    candidate.result_type_parameters.push(ResultTypeParameter {
        name: t,
        result: 0,
        span: Span::default(),
        pattern: TypePattern::Infer(t),
    });
    let mut input = argument(ArgumentInfo::constant(
        BakedValue::Type(types.scalar(ScalarType::Bool)),
        types.meta_type(),
    ));
    input.name = Some(t);
    let mut matched =
        match_candidate(&types, &candidate, from_ref(&input), Span::default()).unwrap();
    matched.result_type_arguments[0].parameter = 1;
    let error = recheck_match_with_nominals(
        &types,
        &NoNominals,
        &candidate,
        from_ref(&input),
        matched,
        Span::default(),
    )
    .unwrap_err();
    assert!(
        error.message.contains("inconsistent argument bindings"),
        "{error:?}"
    );
}
