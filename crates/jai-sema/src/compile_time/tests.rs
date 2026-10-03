use super::*;
use jai_ir::{Block, Flow, IntExpr, Statement};
use jai_types::{
    CallingConvention, ContextMode, Integer, IntegerType, ProcedureType, TypeRegistry,
};

fn location() -> SourceSpan {
    let mut sources = jai_source::SourceMap::default();
    SourceSpan {
        source: sources.insert("run-test.jai".into(), "#run answer()".into()),
        span: jai_source::Span::new(0, 13),
    }
}

#[test]
fn pending_body_retries_into_a_typed_embeddable_constant() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([int]),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: jai_types::Variadic::None,
        })
        .unwrap();
    let id = ProcedureId::new(0);
    let signatures = HashMap::from([(id, signature)]);
    let places = Places::default();
    let mut ready = HashMap::new();
    let expression = ValueExpr::Call {
        call: Call::new(id, vec![]),
        ty: int,
    };
    {
        let provider = ReadyProcedures::new(&types, &ready, &signatures, &[], &places).unwrap();
        assert!(
            matches!(evaluate(&provider, jai_vm::NoEffects, &expression, int, location(), Limits::default()), RunOutcome::Pending(dependencies) if dependencies == [Dependency::Procedure(id)])
        );
    }
    ready.insert(
        id,
        Procedure {
            id,
            signature,
            parameters: vec![],
            locals: vec![],
            cleanups: vec![],
            body: Block {
                statements: vec![Statement::Exit(jai_ir::Exit {
                    cleanups: vec![],
                    transfer: jai_ir::Transfer::ReturnValues(vec![ValueExpr::Int(
                        IntExpr::constant(Integer::wrapping(IntegerType::S64, 42)),
                    )]),
                })],
                flow: Flow::Terminates,
            },
        },
    );
    let provider = ReadyProcedures::new(&types, &ready, &signatures, &[], &places).unwrap();
    let RunOutcome::Complete(Some(value)) = evaluate(
        &provider,
        jai_vm::NoEffects,
        &expression,
        int,
        location(),
        Limits::default(),
    ) else {
        panic!("checked call must finish");
    };
    assert_eq!(
        value,
        ConstantValue {
            ty: int,
            kind: ConstantKind::Int(Integer::wrapping(IntegerType::S64, 42))
        }
    );
    assert!(
        matches!(value.into_expression(), ValueExpr::Int(value) if matches!(value.kind(), jai_ir::IntExprKind::Constant(_)))
    );
}

#[test]
fn failed_execution_is_source_located_and_never_becomes_a_constant() {
    let types = TypeRegistry::new();
    let signatures = HashMap::new();
    let ready = HashMap::new();
    let places = Places::default();
    let provider = ReadyProcedures::new(&types, &ready, &signatures, &[], &places).unwrap();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let expression = ValueExpr::Int(IntExpr::constant(Integer::wrapping(IntegerType::S64, 42)));
    let RunOutcome::Failed(error) = evaluate(
        &provider,
        jai_vm::NoEffects,
        &expression,
        int,
        location(),
        Limits {
            fuel: 0,
            ..Limits::default()
        },
    ) else {
        panic!("zero fuel must fail");
    };
    assert_eq!(error.location, location());
    assert!(error.message.contains("Fuel"));
    let invalid = ValueExpr::Int(IntExpr::new(
        IntegerType::S64,
        jai_ir::IntExprKind::InvalidCheckedCast,
    ));
    let RunOutcome::Failed(error) = evaluate(
        &provider,
        jai_vm::NoEffects,
        &invalid,
        int,
        location(),
        Limits::default(),
    ) else {
        panic!("checked cast must fail");
    };
    assert!(error.message.contains("out of range"));
}

#[test]
fn invalid_staging_body_cannot_become_ready() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([int]),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: jai_types::Variadic::None,
        })
        .unwrap();
    let id = ProcedureId::new(0);
    let signatures = HashMap::from([(id, signature)]);
    let ready = HashMap::from([(
        id,
        Procedure {
            id,
            signature,
            parameters: vec![],
            locals: vec![],
            cleanups: vec![],
            body: Block {
                statements: vec![],
                flow: Flow::FallsThrough,
            },
        },
    )]);
    assert!(ReadyProcedures::new(&types, &ready, &signatures, &[], &Places::default()).is_err());
}

#[test]
fn unsupported_embedding_and_type_mismatch_reject_before_effect_transaction() {
    #[derive(Default)]
    struct Effects {
        began: usize,
    }
    impl CompilerEffects for Effects {
        fn begin(&mut self) {
            self.began += 1;
        }
        fn request(&mut self, _: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
            panic!("rejected result must never request effects");
        }
        fn finish(&mut self, _: bool) -> Result<(), jai_vm::Error> {
            panic!("rejected result must never start a transaction");
        }
    }
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer = types.pointer(int).unwrap();
    let signatures = HashMap::new();
    let ready = HashMap::new();
    let places = Places::default();
    let provider = ReadyProcedures::new(&types, &ready, &signatures, &[], &places).unwrap();
    let mut effects = Effects::default();
    assert!(matches!(
        evaluate(
            &provider,
            &mut effects,
            &ValueExpr::Zero(pointer),
            pointer,
            location(),
            Limits::default()
        ),
        RunOutcome::Failed(_)
    ));
    assert!(matches!(
        evaluate(
            &provider,
            &mut effects,
            &ValueExpr::Int(IntExpr::constant(Integer::wrapping(IntegerType::S64, 42))),
            types.scalar(ScalarType::Bool),
            location(),
            Limits::default()
        ),
        RunOutcome::Failed(_)
    ));
    assert_eq!(effects.began, 0);
}
