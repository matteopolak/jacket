use jai_ir::*;
use jai_types::{
    CallingConvention, ContextMode, RecordKind, ScalarType, TypeId, TypeRegistry, Types, Variadic,
};
use std::collections::HashMap;

struct Fixture {
    types: Types,
    schema: ContextDefinition,
    implicit: TypeId,
    contextless: TypeId,
    boolean: TypeId,
}

impl Fixture {
    fn new() -> Self {
        let mut types = TypeRegistry::new();
        let boolean = types.scalar(ScalarType::Bool);
        let record = types.reserve_record(RecordKind::Struct);
        types.define_record(record, [boolean]).unwrap();
        let pointer = types.pointer(record).unwrap();
        let mut signature = |convention, context| {
            types
                .procedure(jai_types::ProcedureType {
                    parameters: Box::new([]),
                    results: Box::new([]),
                    return_abi: jai_types::ForeignReturnAbi::Natural,
                    convention,
                    context,
                    variadic: Variadic::None,
                })
                .unwrap()
        };
        let implicit = signature(CallingConvention::Jai, ContextMode::Implicit);
        let contextless = signature(CallingConvention::C, ContextMode::None);
        let schema = ContextDefinition {
            record_type: record,
            pointer_type: pointer,
            default: ConstantValue {
                ty: record,
                kind: ConstantKind::Record(vec![ConstantValue {
                    ty: boolean,
                    kind: ConstantKind::Bool(false),
                }]),
            },
        };
        Self {
            types: types.freeze().unwrap(),
            schema,
            implicit,
            contextless,
            boolean,
        }
    }

    fn record(&self) -> ValueExpr {
        ValueExpr::Record {
            ty: self.schema.record_type,
            fields: vec![ValueExpr::Bool(BoolExpr::Constant(true))],
        }
    }

    fn procedure(&self, body: Block) -> Procedure {
        Procedure {
            id: ProcedureId::new(7),
            signature: self.contextless,
            parameters: vec![],
            locals: vec![],
            body,
            cleanups: vec![],
        }
    }

    fn push(&self, id: PushContextId, statements: Vec<Statement>) -> Statement {
        Statement::PushContext {
            id,
            value: self.record(),
            body: block(statements),
        }
    }

    fn verify(&self, procedure: &Procedure) -> Result<(), IrError> {
        verify_procedure_with_context(
            &self.types,
            procedure,
            &self.signatures(),
            &[],
            &Places::default(),
            Some(&self.schema),
        )
        .map(|_| ())
    }

    fn signatures(&self) -> HashMap<ProcedureId, TypeId> {
        HashMap::from([
            (ProcedureId::new(7), self.contextless),
            (ProcedureId::new(50), self.implicit),
        ])
    }
}

fn block(statements: Vec<Statement>) -> Block {
    Block {
        statements,
        flow: Flow::FallsThrough,
    }
}

fn implicit_call() -> Statement {
    Statement::CallVoid(Call::new(ProcedureId::new(50), vec![]))
}

fn push_id(index: usize) -> PushContextId {
    PushContextId::new(ProcedureId::new(7), index)
}

#[test]
fn context_schema_requires_pointer_to_its_record() {
    let fixture = Fixture::new();
    let mut schema = fixture.schema.clone();
    schema.pointer_type = fixture.boolean;
    assert!(matches!(
        ProgramBuilder::new(fixture.types)
            .context(schema)
            .finish_library(),
        Err(IrError::InvalidValue(_))
    ));
}

#[test]
fn context_schema_default_requires_record_type() {
    let fixture = Fixture::new();
    let mut schema = fixture.schema.clone();
    schema.default = ConstantValue {
        ty: fixture.boolean,
        kind: ConstantKind::Bool(false),
    };
    assert!(matches!(
        ProgramBuilder::new(fixture.types)
            .context(schema)
            .finish_library(),
        Err(IrError::TypeMismatch { .. })
    ));
}

#[test]
fn context_expression_is_record_snapshot_and_checked_proof_retains_schema() {
    let fixture = Fixture::new();
    let expression = ValueExpr::Context {
        ty: fixture.schema.record_type,
    };
    let signatures = HashMap::new();
    let places = Places::default();
    let proof = verify_expression_with_context(
        &fixture.types,
        &expression,
        &signatures,
        &[],
        &places,
        Some(&fixture.schema),
    )
    .unwrap();
    assert_eq!(
        proof.expression().type_id(proof.types()),
        fixture.schema.record_type
    );
    assert_eq!(proof.context(), Some(&fixture.schema));
    let pointer_expression = ValueExpr::Context {
        ty: fixture.schema.pointer_type,
    };
    assert!(matches!(
        verify_expression_with_context(
            &fixture.types,
            &pointer_expression,
            &signatures,
            &[],
            &places,
            Some(&fixture.schema)
        ),
        Err(IrError::TypeMismatch { .. })
    ));
}

#[test]
fn old_expression_entry_point_has_no_context_schema() {
    let fixture = Fixture::new();
    let expression = ValueExpr::Context {
        ty: fixture.schema.record_type,
    };
    assert!(matches!(
        verify_expression(
            &fixture.types,
            &expression,
            &HashMap::new(),
            &[],
            &Places::default()
        ),
        Err(IrError::MissingContext)
    ));
}

#[test]
fn contextless_procedure_cannot_call_implicit_procedure_directly() {
    let fixture = Fixture::new();
    assert!(matches!(
        fixture.verify(&fixture.procedure(block(vec![implicit_call()]))),
        Err(IrError::MissingContext)
    ));
}

#[test]
fn pushing_record_copy_enables_implicit_call_in_contextless_procedure() {
    let fixture = Fixture::new();
    let procedure = fixture.procedure(block(vec![fixture.push(
        push_id(3),
        vec![
            implicit_call(),
            Statement::DiscardValue(ValueExpr::Context {
                ty: fixture.schema.record_type,
            }),
        ],
    )]));
    assert!(fixture.verify(&procedure).is_ok());
}

#[test]
fn push_context_value_requires_record_copy_type() {
    let fixture = Fixture::new();
    let procedure = fixture.procedure(block(vec![Statement::PushContext {
        id: push_id(0),
        value: ValueExpr::Zero(fixture.schema.pointer_type),
        body: block(vec![]),
    }]));
    assert!(matches!(
        fixture.verify(&procedure),
        Err(IrError::TypeMismatch { .. })
    ));
}

#[test]
fn push_context_identity_is_branded_to_its_procedure() {
    let fixture = Fixture::new();
    let foreign = PushContextId::new(ProcedureId::new(99), 0);
    let procedure = fixture.procedure(block(vec![fixture.push(foreign, vec![])]));
    assert!(
        matches!(fixture.verify(&procedure), Err(IrError::LocalOwner { expected, actual }) if expected == ProcedureId::new(7) && actual == ProcedureId::new(99))
    );
}

#[test]
fn duplicate_push_context_identity_is_rejected() {
    let fixture = Fixture::new();
    let procedure = fixture.procedure(block(vec![
        fixture.push(push_id(0), vec![]),
        fixture.push(push_id(0), vec![]),
    ]));
    assert!(matches!(
        fixture.verify(&procedure),
        Err(IrError::DuplicateIdentity {
            kind: "push context",
            index: 0
        })
    ));
}

#[test]
fn cleanup_cannot_capture_unknown_push() {
    let fixture = Fixture::new();
    let mut procedure = fixture.procedure(block(vec![]));
    procedure.cleanups.push(Cleanup {
        body: block(vec![]),
        context: CleanupContext::Push(push_id(8)),
    });
    assert!(matches!(
        fixture.verify(&procedure),
        Err(IrError::UnknownIdentity {
            kind: "captured push context",
            index: 8
        })
    ));
}

#[test]
fn cleanup_push_capture_ancestry_cannot_cycle() {
    let fixture = Fixture::new();
    let mut procedure = fixture.procedure(block(vec![]));
    procedure.cleanups.push(Cleanup {
        body: block(vec![fixture.push(push_id(1), vec![])]),
        context: CleanupContext::Push(push_id(1)),
    });
    assert!(matches!(
        fixture.verify(&procedure),
        Err(IrError::UnknownIdentity {
            kind: "push capture cycle",
            index: 1
        })
    ));
}

#[test]
fn procedure_cleanup_does_not_inherit_call_site_push_context() {
    let fixture = Fixture::new();
    let mut procedure = fixture.procedure(block(vec![
        fixture.push(push_id(0), vec![Statement::Cleanup(CleanupId::new(0))]),
    ]));
    procedure.cleanups.push(Cleanup {
        body: block(vec![Statement::DiscardValue(ValueExpr::Context {
            ty: fixture.schema.record_type,
        })]),
        context: CleanupContext::Procedure,
    });
    assert!(matches!(
        fixture.verify(&procedure),
        Err(IrError::MissingContext)
    ));
}

#[test]
fn cleanup_can_use_captured_context_while_inner_push_is_active() {
    let fixture = Fixture::new();
    let mut procedure = fixture.procedure(block(vec![fixture.push(
        push_id(0),
        vec![fixture.push(push_id(1), vec![Statement::Cleanup(CleanupId::new(0))])],
    )]));
    procedure.cleanups.push(Cleanup {
        body: block(vec![
            Statement::DiscardValue(ValueExpr::Context {
                ty: fixture.schema.record_type,
            }),
            implicit_call(),
        ]),
        context: CleanupContext::Push(push_id(0)),
    });
    assert!(fixture.verify(&procedure).is_ok());
}

#[test]
fn captured_push_must_dominate_cleanup_invocation() {
    let fixture = Fixture::new();
    let mut procedure = fixture.procedure(block(vec![
        fixture.push(push_id(0), vec![]),
        Statement::Cleanup(CleanupId::new(0)),
    ]));
    procedure.cleanups.push(Cleanup {
        body: block(vec![]),
        context: CleanupContext::Push(push_id(0)),
    });
    assert!(matches!(
        fixture.verify(&procedure),
        Err(IrError::MissingContext)
    ));
}

#[test]
fn context_schema_does_not_hide_cleanup_cycle_in_cases_subject() {
    let fixture = Fixture::new();
    let mut procedure = fixture.procedure(block(vec![]));
    procedure.cleanups.push(Cleanup {
        body: block(vec![Statement::Cases(Cases {
            default_position: None,
            default_through: false,
            subject: Box::new(Statement::Cleanup(CleanupId::new(0))),
            arms: vec![],
            default: Some(block(vec![])),
            flow: Flow::FallsThrough,
            exhaustive: false,
        })]),
        context: CleanupContext::Procedure,
    });
    assert!(
        matches!(fixture.verify(&procedure), Err(IrError::CleanupCycle(id)) if id == CleanupId::new(0))
    );
}

#[test]
fn root_call_with_context_retains_schema_in_checked_proof() {
    let fixture = Fixture::new();
    let call = Call::new(ProcedureId::new(50), vec![]);
    let signatures = fixture.signatures();
    let places = Places::default();
    let proof = verify_call_with_context(
        &fixture.types,
        &call,
        &signatures,
        &[],
        &places,
        Some(&fixture.schema),
    )
    .unwrap();
    assert_eq!(proof.context(), Some(&fixture.schema));
    assert!(proof.results().is_empty());
}
