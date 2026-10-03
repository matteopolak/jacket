//! Context schemas and lexical availability are checked before backend or VM use.
use super::*;

pub(super) fn definition(
    context: &ContextDefinition,
    types: &dyn TypeView,
    signatures: &HashMap<ProcedureId, TypeId>,
) -> Result<(), IrError> {
    if types.record_definition(context.record_type)?.kind != jai_types::RecordKind::Struct {
        return Err(IrError::InvalidValue(context.record_type));
    }
    let TypeKind::Pointer(record) = *types.kind(context.pointer_type)? else {
        return Err(IrError::InvalidValue(context.pointer_type));
    };
    same_type(context.record_type, record)?;
    same_type(context.record_type, context.default.ty)?;
    constant(types, &context.default)?;
    verify_constant_procedures(types, &context.default, signatures)
}

impl Context<'_> {
    pub(super) fn context_type(&self) -> Result<TypeId, IrError> {
        if !self.context_available {
            return Err(IrError::MissingContext);
        }
        self.context
            .map(|context| context.record_type)
            .ok_or(IrError::MissingContext)
    }
    pub(super) fn call_context(&self, signature: &jai_types::ProcedureType) -> Result<(), IrError> {
        if signature.context == jai_types::ContextMode::Implicit && !self.context_available {
            Err(IrError::MissingContext)
        } else {
            Ok(())
        }
    }
}

pub(super) fn push_definitions(
    procedure: &Procedure,
) -> Result<HashMap<PushContextId, Option<PushContextId>>, IrError> {
    let mut pushes = HashMap::new();
    let mut pending: Vec<(&Statement, Option<PushContextId>)> = procedure
        .body
        .statements
        .iter()
        .map(|statement| (statement, None))
        .collect();
    for cleanup in &procedure.cleanups {
        let parent = match cleanup.context {
            CleanupContext::Procedure => None,
            CleanupContext::Push(id) => Some(id),
        };
        pending.extend(
            cleanup
                .body
                .statements
                .iter()
                .map(|statement| (statement, parent)),
        );
    }
    while let Some((statement, parent)) = pending.pop() {
        match statement {
            Statement::PushContext {
                id,
                body,
                ..
            } => {
                if id.procedure() != procedure.id {
                    return Err(IrError::LocalOwner {
                        expected: procedure.id,
                        actual: id.procedure(),
                    });
                }
                if pushes.insert(*id, parent).is_some() {
                    return Err(IrError::DuplicateIdentity {
                        kind: "push context",
                        index: id.index(),
                    });
                }
                pending.extend(
                    body.statements
                        .iter()
                        .map(|statement| (statement, Some(*id))),
                );
            }
            Statement::If(_, yes, no) => {
                pending.extend(
                    yes.statements
                        .iter()
                        .chain(&no.statements)
                        .map(|statement| (statement, parent)),
                );
            }
            Statement::Block(body)
            | Statement::While {
                body, ..
            } => pending.extend(body.statements.iter().map(|statement| (statement, parent))),
            Statement::Range(range) => pending.extend(
                range
                    .body
                    .statements
                    .iter()
                    .map(|statement| (statement, parent)),
            ),
            Statement::Cases(cases) => {
                pending.push((&cases.subject, parent));
                for arm in &cases.arms {
                    pending.extend(
                        arm.body
                            .statements
                            .iter()
                            .map(|statement| (statement, parent)),
                    );
                }
                if let Some(default) = &cases.default {
                    pending.extend(
                        default
                            .statements
                            .iter()
                            .map(|statement| (statement, parent)),
                    );
                }
            }
            _ => {}
        }
    }
    for &id in pushes.keys() {
        capture_chain(CleanupContext::Push(id), &pushes)?;
    }
    for cleanup in &procedure.cleanups {
        capture_chain(cleanup.context, &pushes)?;
    }
    Ok(pushes)
}

pub(super) fn capture_chain(
    capture: CleanupContext,
    pushes: &HashMap<PushContextId, Option<PushContextId>>,
) -> Result<Vec<PushContextId>, IrError> {
    let mut id = match capture {
        CleanupContext::Procedure => None,
        CleanupContext::Push(id) => Some(id),
    };
    let mut chain = Vec::new();
    while let Some(current) = id {
        if chain.len() >= MAX_VERIFICATION_DEPTH {
            return Err(IrError::VerificationDepth);
        }
        if chain.contains(&current) {
            return Err(unknown("push capture cycle", current.index()));
        }
        chain.push(current);
        id = *pushes
            .get(&current)
            .ok_or_else(|| unknown("captured push context", current.index()))?;
    }
    chain.reverse();
    Ok(chain)
}
