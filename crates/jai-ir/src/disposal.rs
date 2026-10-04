//! Dispose rejected staging graphs without recursively dropping their children.
mod expressions;
use crate::*;

pub(super) enum Work {
    Value(ValueExpr),
    Int(IntExpr),
    Bool(BoolExpr),
    Float(FloatExpr),
    Call(Call),
    Constant(ConstantValue),
    Block(Block),
    Statement(Statement),
    Exit(Exit),
}

pub(crate) fn constant(value: ConstantValue) {
    run(vec![Work::Constant(value)]);
}

pub(crate) fn simd_instructions(instructions: Vec<SimdInstruction>) {
    let mut pending = Vec::new();
    push_simd(instructions, &mut pending);
    run(pending);
}

fn push_simd(instructions: Vec<SimdInstruction>, pending: &mut Vec<Work>) {
    for instruction in instructions {
        match instruction {
            SimdInstruction::Load {
                address, ..
            }
            | SimdInstruction::Store {
                address, ..
            } => {
                pending.push(Work::Value(address));
            }
            SimdInstruction::Add {
                ..
            }
            | SimdInstruction::DebugTrap
            | SimdInstruction::Arm64DebugTrap => {}
        }
    }
}

pub(crate) fn procedures(procedures: Vec<Procedure>) {
    let mut pending = Vec::new();
    push_procedures(procedures, &mut pending);
    run(pending);
}

pub(crate) fn globals(globals: Vec<Global>) {
    let mut pending = Vec::new();
    push_globals(globals, &mut pending);
    run(pending);
}

pub(crate) fn places(places: &mut Places) {
    let mut pending = Vec::new();
    places.drain_operands(&mut pending);
    run(pending);
}

pub(crate) fn staging(
    procedures: Vec<Procedure>,
    globals: Vec<Global>,
    context: Option<ContextDefinition>,
    places: &mut Places,
) {
    let mut pending = Vec::new();
    push_procedures(procedures, &mut pending);
    push_globals(globals, &mut pending);
    if let Some(context) = context {
        pending.push(Work::Constant(context.default));
    }
    places.drain_operands(&mut pending);
    run(pending);
}

fn push_procedures(procedures: Vec<Procedure>, pending: &mut Vec<Work>) {
    for procedure in procedures {
        pending.push(Work::Block(procedure.body));
        pending.extend(
            procedure
                .cleanups
                .into_iter()
                .map(|cleanup| Work::Block(cleanup.body)),
        );
    }
}

fn push_globals(globals: Vec<Global>, pending: &mut Vec<Work>) {
    for global in globals {
        let runtime_initializer = global.runtime_initializer().cloned();
        if let GlobalInitializer::Value(value) = global.into_initializer() {
            pending.push(Work::Constant(value));
        }
        if let Some(value) = runtime_initializer {
            pending.push(Work::Constant(value));
        }
    }
}

fn run(mut pending: Vec<Work>) {
    while let Some(work) = pending.pop() {
        match work {
            Work::Value(value) => expressions::value(value, &mut pending),
            Work::Int(value) => expressions::integer(value, &mut pending),
            Work::Bool(value) => expressions::boolean(value, &mut pending),
            Work::Float(value) => expressions::float(value, &mut pending),
            Work::Call(value) => expressions::call(value, &mut pending),
            Work::Constant(value) => match value.kind {
                ConstantKind::Record(values) | ConstantKind::Array(values) => {
                    pending.extend(values.into_iter().map(Work::Constant));
                }
                ConstantKind::Union {
                    value, ..
                }
                | ConstantKind::Distinct(value) => {
                    pending.push(Work::Constant(*value));
                }
                ConstantKind::Int(_)
                | ConstantKind::NativePointer(_)
                | ConstantKind::Float(_)
                | ConstantKind::Bool(_)
                | ConstantKind::StringBytes(_)
                | ConstantKind::Enum(_)
                | ConstantKind::Procedure(_)
                | ConstantKind::RuntimeType(_)
                | ConstantKind::Zero => {}
            },
            Work::Block(block) => {
                pending.extend(block.statements.into_iter().map(Work::Statement));
            }
            Work::Exit(exit) => match exit.transfer {
                Transfer::ReturnValues(values) => {
                    pending.extend(values.into_iter().map(Work::Value));
                }
                Transfer::ReturnInt(value) => pending.push(Work::Int(value)),
                Transfer::ReturnBool(value) => pending.push(Work::Bool(value)),
                Transfer::ReturnVoid | Transfer::Break(_) | Transfer::Continue(_) => {}
            },
            Work::Statement(statement) => match statement {
                Statement::Simd(mut block) => {
                    push_simd(block.take_instructions(), &mut pending);
                }
                Statement::PushContext {
                    value,
                    body,
                    ..
                } => {
                    pending.push(Work::Value(value));
                    pending.push(Work::Block(body));
                }
                Statement::IndirectCallResults {
                    callee,
                    arguments,
                    ..
                } => {
                    pending.push(Work::Value(*callee));
                    pending.extend(arguments.into_iter().map(|(_, value)| Work::Value(value)));
                }
                Statement::Store(_, value) | Statement::DiscardValue(value) => {
                    pending.push(Work::Value(value));
                }
                Statement::CallResults {
                    call, ..
                }
                | Statement::CallVoid(call) => {
                    pending.push(Work::Call(call));
                }
                Statement::StoreInt(_, value) | Statement::DiscardInt(value) => {
                    pending.push(Work::Int(value));
                }
                Statement::StoreBool(_, value) | Statement::DiscardBool(value) => {
                    pending.push(Work::Bool(value));
                }
                Statement::Exit(exit) => pending.push(Work::Exit(exit)),
                Statement::Cleanup(_) => {}
                Statement::If(condition, then_body, else_body) => {
                    pending.push(Work::Bool(condition));
                    pending.push(Work::Block(then_body));
                    pending.push(Work::Block(else_body));
                }
                Statement::Cases(cases) => {
                    pending.push(Work::Statement(*cases.subject));
                    for arm in cases.arms {
                        pending.push(Work::Bool(arm.condition));
                        pending.push(Work::Block(arm.body));
                    }
                    if let Some(body) = cases.default {
                        pending.push(Work::Block(body));
                    }
                }
                Statement::While {
                    condition,
                    body,
                    ..
                } => {
                    match condition {
                        LoopCondition::Value(value) | LoopCondition::BoundBool(_, value) => {
                            pending.push(Work::Bool(value));
                        }
                        LoopCondition::BoundInt(_, value) => pending.push(Work::Int(value)),
                    }
                    pending.push(Work::Block(body));
                }
                Statement::Range(range) => {
                    pending.push(Work::Int(range.start));
                    pending.push(Work::Int(range.end));
                    pending.push(Work::Block(range.body));
                }
                Statement::Block(block) => pending.push(Work::Block(block)),
            },
        }
    }
}

pub(crate) mod compiler_roots;
