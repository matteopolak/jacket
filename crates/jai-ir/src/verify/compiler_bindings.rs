//! Typed native inputs admitted from a separate checked compiler frame.
use super::*;

fn prefix(proof: &Context<'_>, bindings: &[(ExpressionBindingId, TypeId)]) -> Result<(), IrError> {
    if bindings.len() > crate::expression_bindings::MAX_EXPRESSION_BINDINGS {
        return Err(IrError::VerificationDepth);
    }
    for &(id, ty) in bindings {
        let owner = proof.binding_owner.expect("owned compiler input proof");
        if id.procedure() != owner {
            return Err(IrError::LocalOwner {
                expected: owner,
                actual: id.procedure(),
            });
        }
        storage::runtime_type(proof.types, ty)?;
        if proof
            .expression_bindings
            .borrow_mut()
            .insert(id, ty)
            .is_some()
        {
            return Err(IrError::DuplicateIdentity {
                kind: "compiler input binding",
                index: id.index(),
            });
        }
    }
    Ok(())
}

/// Prove the native domain of one compiler slot, including nested type constraints.
pub fn verify_compiler_slot_type(types: &dyn TypeView, ty: TypeId) -> Result<(), IrError> {
    storage::runtime_type(types, ty)
}

/// Bindings are type facts from a checked compiler slot schema, never fabricated values.
#[allow(clippy::too_many_arguments)]
pub fn verify_compiler_bound_expression_with_context<'a>(
    types: &'a dyn TypeView,
    expression: &'a ValueExpr,
    signatures: &'a HashMap<ProcedureId, TypeId>,
    globals: &'a [Global],
    places: &'a Places,
    owner: ProcedureId,
    context: Option<&'a ContextDefinition>,
    bindings: &[(ExpressionBindingId, TypeId)],
) -> Result<CheckedExpression<'a>, IrError> {
    if let Some(context) = context {
        super::context::definition(context, types, signatures)?;
    }
    global_identities(types, globals)?;
    let mut proof = Context::root(types, signatures, globals, places, context);
    proof.binding_owner = Some(owner);
    prefix(&proof, bindings)?;
    proof.value(expression)?;
    Ok(CheckedExpression {
        binding_owner: Some(owner),
        expression,
        types,
        signatures,
        globals,
        places,
        context,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn verify_compiler_bound_call_with_context<'a>(
    types: &'a dyn TypeView,
    call: &'a Call,
    signatures: &'a HashMap<ProcedureId, TypeId>,
    globals: &'a [Global],
    places: &'a Places,
    owner: ProcedureId,
    context: Option<&'a ContextDefinition>,
    bindings: &[(ExpressionBindingId, TypeId)],
) -> Result<CheckedCall<'a>, IrError> {
    if let Some(context) = context {
        super::context::definition(context, types, signatures)?;
    }
    global_identities(types, globals)?;
    let mut proof = Context::root(types, signatures, globals, places, context);
    proof.binding_owner = Some(owner);
    prefix(&proof, bindings)?;
    let results = proof.call(call)?.to_vec();
    Ok(CheckedCall {
        binding_owner: Some(owner),
        call,
        results,
        types,
        signatures,
        globals,
        places,
        context,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::TypeRegistry;
    #[test]
    fn typed_prefix_requires_exact_owner_without_creating_native_signature() {
        let types = TypeRegistry::new();
        let owner = ProcedureId::new(42);
        let ty = types.scalar(ScalarType::Int(IntegerType::S64));
        let id = ExpressionBindingId::new(owner, 0);
        let expression = ValueExpr::Bound { binding: id, ty };
        let signatures = HashMap::new();
        let places = Places::default();
        assert!(
            verify_compiler_bound_expression_with_context(
                &types,
                &expression,
                &signatures,
                &[],
                &places,
                owner,
                None,
                &[(id, ty)]
            )
            .is_ok()
        );
        assert!(
            verify_compiler_bound_expression_with_context(
                &types,
                &expression,
                &signatures,
                &[],
                &places,
                owner,
                None,
                &[]
            )
            .is_err()
        );
        assert!(
            verify_compiler_bound_expression_with_context(
                &types,
                &expression,
                &signatures,
                &[],
                &places,
                ProcedureId::new(43),
                None,
                &[(id, ty)]
            )
            .is_err()
        );
        assert!(
            verify_compiler_bound_expression_with_context(
                &types,
                &expression,
                &signatures,
                &[],
                &places,
                owner,
                None,
                &[(id, ty), (id, ty)]
            )
            .is_err()
        );
        assert!(
            verify_compiler_bound_expression_with_context(
                &types,
                &expression,
                &signatures,
                &[],
                &places,
                owner,
                None,
                &[(id, types.scalar(ScalarType::Bool))]
            )
            .is_err()
        );
    }
}
