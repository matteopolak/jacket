use super::*;
use jai_types::{
    CallingConvention, ContextMode, Integer, ProcedureType, ScalarType, TypeRegistry, Variadic,
};

fn fixture() -> (TypeRegistry, Procedure, HashMap<ProcedureId, TypeId>) {
    let mut types = TypeRegistry::new();
    let integer = types.scalar(ScalarType::Int(IntegerType::S64));
    let signature = types
        .procedure(ProcedureType {
            parameters: [].into(),
            results: [integer].into(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let id = ProcedureId::new(0);
    let local = Local::new_typed(id, 0, integer, &types).unwrap();
    let place = local.integer(&types).unwrap().place();
    let procedure = Procedure {
        id,
        signature,
        parameters: vec![],
        locals: vec![local],
        body: Block {
            statements: vec![
                Statement::StoreInt(
                    place,
                    IntExpr::constant(Integer::wrapping(IntegerType::S64, 7)),
                ),
                Statement::Exit(Exit {
                    cleanups: vec![],
                    transfer: Transfer::ReturnInt(IntExpr::load(place)),
                }),
            ],
            flow: Flow::Terminates,
        },
        cleanups: vec![],
    };
    (types, procedure, HashMap::from([(id, signature)]))
}

#[test]
fn procedure_plan_owns_source_and_addresses_control_with_stable_ids() {
    let (types, mut procedure, signatures) = fixture();
    let places = Places::default();
    let checked = verify_procedure(&types, &procedure, &signatures, &[], &places).unwrap();
    let plan = compile_checked_procedure(checked, Limits::default()).unwrap();
    procedure.body.statements.clear();
    drop(procedure);
    assert_eq!(plan.source.body.statements.len(), 2);
    let block = &plan.code.blocks[plan.body.index()];
    let StatementCode::Store {
        destination,
        value,
    } = block.statements[0]
    else {
        panic!("expected store")
    };
    assert!(destination.index() < value.index());
    assert!(matches!(
        plan.code.nodes[destination.index()].kind,
        NodeKind::Place {
            op: PlaceOp::Local(_),
            ..
        }
    ));
    let StatementCode::Exit(exit) = &block.statements[1] else {
        panic!("expected return")
    };
    assert!(matches!(exit.transfer, TransferCode::Return));
    assert_eq!(exit.values.len(), 1);
    assert_eq!(block.flow, Flow::Terminates);
}

#[test]
fn procedure_source_clone_obeys_the_cumulative_budget() {
    let (types, procedure, signatures) = fixture();
    let places = Places::default();
    let checked = verify_procedure(&types, &procedure, &signatures, &[], &places).unwrap();
    let complete = compile_checked_procedure(checked, Limits::default()).unwrap();
    let lowered_budget = complete.code.metadata_cells / 2;
    assert!(lowered_budget > 0);
    let checked = verify_procedure(&types, &procedure, &signatures, &[], &places).unwrap();
    assert!(matches!(
        compile_checked_procedure(
            checked,
            Limits {
                value_cells: lowered_budget,
                ..Limits::default()
            }
        ),
        Err(Error::Limit(LimitKind::ValueCells))
    ));
}

#[test]
fn procedure_planning_work_obeys_fuel_before_owned_source_copy() {
    let (types, procedure, signatures) = fixture();
    let places = Places::default();
    let checked = verify_procedure(&types, &procedure, &signatures, &[], &places).unwrap();
    assert!(matches!(
        compile_checked_procedure(
            checked,
            Limits {
                fuel: 1,
                ..Limits::default()
            }
        ),
        Err(Error::Limit(LimitKind::Fuel))
    ));
}
