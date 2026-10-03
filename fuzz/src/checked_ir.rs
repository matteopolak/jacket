use crate::{Bytes, limits};
use jai_ir::*;
use jai_types::{
    CallingConvention, ContextMode, Integer, IntegerType, ProcedureType, RecordKind, ScalarType,
    TypeRegistry, Variadic,
};

fn integer(value: i128) -> IntExpr {
    IntExpr::constant(Integer::checked(IntegerType::S64, value).unwrap())
}

pub fn checked_ir_vm(data: &[u8]) {
    let mut bytes = Bytes::new(data);
    let flags = bytes.byte();
    let fields = usize::from(bytes.byte() % 8) + 1;
    let selected = usize::from(bytes.byte()) % fields;
    let values = (0..fields).map(|_| bytes.signed()).collect::<Vec<_>>();
    let replacement = bytes.signed();
    let operations = usize::from(bytes.byte() % 16);
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, vec![int; fields]).unwrap();
    let field = types.field(record, selected).unwrap().id;
    let id = ProcedureId::new(0);
    let original = Local::new_typed(id, 0, record, &types).unwrap();
    let copy = Local::new_typed(id, 1, record, &types).unwrap();
    let mut places = PlaceRegistry::new();
    let original_field = places.field(original.place(), field, &types).unwrap();
    let copy_field = places.field(copy.place(), field, &types).unwrap();
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([int]),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
            return_abi: jai_types::ForeignReturnAbi::Natural,
        })
        .unwrap();
    let mut expression = IntExpr::load(IntPlace::try_from_place(copy_field, &types).unwrap());
    let mut expected = values[selected];
    for _ in 0..operations {
        let right = bytes.signed();
        let operation = match bytes.byte() % 4 {
            0 => {
                expected += right;
                IntOp::Add
            }
            1 => {
                expected -= right;
                IntOp::Subtract
            }
            2 => {
                expected ^= right;
                IntOp::BitXor
            }
            _ => {
                expected |= right;
                IntOp::BitOr
            }
        };
        expression = IntExpr::new(
            IntegerType::S64,
            IntExprKind::Binary(operation, Box::new(expression), Box::new(integer(right))),
        );
    }
    let procedure = Procedure {
        id,
        signature,
        parameters: vec![],
        locals: vec![original, copy],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![
                Statement::Store(
                    original.place(),
                    ValueExpr::Record {
                        ty: record,
                        fields: values
                            .iter()
                            .map(|value| ValueExpr::Int(integer(*value)))
                            .collect(),
                    },
                ),
                Statement::Store(copy.place(), ValueExpr::Load(original.place())),
                Statement::Store(original_field, ValueExpr::Int(integer(replacement))),
                Statement::Exit(Exit {
                    cleanups: vec![],
                    transfer: Transfer::ReturnInt(expression),
                }),
            ],
        },
    };
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![procedure])
        .places(places.freeze())
        .finish(EntryPoint::Int(id))
        .expect("constructed typed program must pass the IR proof boundary");
    let target = if flags & 1 == 0 {
        jai_runtime::host_target()
    } else {
        jai_runtime::browser_target()
    };
    let byte_target = jai_vm::ByteTarget::from(&target);
    let mut vm = jai_vm::Vm::new_with_execution_phase(
        &program,
        jai_vm::NoEffects,
        limits(),
        byte_target,
        jai_vm::ExecutionPhase::Runtime,
    )
    .unwrap();
    for _ in 0..2 {
        let execution = vm.execute(id, vec![]);
        assert!(
            matches!(execution.outcome, jai_vm::Outcome::Complete(values)
            if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == expected)),
            "checked VM must preserve independent scalar arithmetic and record-copy semantics"
        );
        assert_eq!(
            vm.memory().allocation_count(),
            0,
            "completed context-free scalar request must release frame backing"
        );
    }
    // Exercise transactional cleanup on a separately named resource rejection.
    let mut denied = jai_vm::Vm::new_with_execution_phase(
        &program,
        jai_vm::NoEffects,
        jai_vm::Limits {
            fuel: 1,
            ..limits()
        },
        byte_target,
        jai_vm::ExecutionPhase::Runtime,
    )
    .unwrap();
    assert!(matches!(
        denied.execute(id, vec![]).outcome,
        jai_vm::Outcome::Failed(jai_vm::Error::Limit(jai_vm::LimitKind::Fuel))
    ));
    assert_eq!(denied.memory().allocation_count(), 0);
}
