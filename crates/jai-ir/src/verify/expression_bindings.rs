//! Producers initialize once in order; the proof environment never escapes.
use super::*;
use crate::expression_bindings::MAX_EXPRESSION_BINDINGS;

struct BindingScope<'a> {
    values: &'a RefCell<HashMap<ExpressionBindingId, TypeId>>,
    installed: Vec<ExpressionBindingId>,
}
impl Drop for BindingScope<'_> {
    fn drop(&mut self) {
        let mut values = self.values.borrow_mut();
        for binding in &self.installed {
            values.remove(binding);
        }
    }
}
impl Context<'_> {
    fn binding_owner(&self, id: ExpressionBindingId) -> Result<(), IrError> {
        let expected = self
            .binding_owner
            .ok_or_else(|| unknown("ownerless expression binding", id.index()))?;
        if id.procedure() != expected {
            return Err(IrError::LocalOwner {
                expected,
                actual: id.procedure(),
            });
        }
        Ok(())
    }
    pub(super) fn bound_value(&self, id: ExpressionBindingId, ty: TypeId) -> Result<(), IrError> {
        self.binding_owner(id)?;
        let actual = self
            .expression_bindings
            .borrow()
            .get(&id)
            .copied()
            .ok_or_else(|| unknown("unbound expression binding", id.index()))?;
        same_type(actual, ty)
    }
    pub(super) fn bound_scope(
        &self,
        bindings: &[(ExpressionBindingId, ValueExpr)],
        body: &ValueExpr,
        ty: TypeId,
    ) -> Result<(), IrError> {
        if bindings
            .len()
            .checked_add(self.expression_bindings.borrow().len())
            .is_none_or(|count| count > MAX_EXPRESSION_BINDINGS)
        {
            return Err(IrError::VerificationDepth);
        }
        let mut scope = BindingScope {
            values: &self.expression_bindings,
            installed: Vec::with_capacity(bindings.len()),
        };
        for (id, value) in bindings {
            self.binding_owner(*id)?;
            if self.expression_bindings.borrow().contains_key(id) {
                return Err(IrError::DuplicateIdentity {
                    kind: "active expression binding",
                    index: id.index(),
                });
            }
            let actual = self.value(value)?;
            self.expression_bindings.borrow_mut().insert(*id, actual);
            scope.installed.push(*id);
        }
        same_type(ty, self.value(body)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::{ScalarType, TypeRegistry};

    fn id(index: usize) -> ExpressionBindingId {
        ExpressionBindingId::new(ProcedureId::new(7), index)
    }
    fn types() -> (TypeRegistry, TypeId) {
        let types = TypeRegistry::new();
        let ty = types.scalar(ScalarType::Int(IntegerType::S64));
        (types, ty)
    }
    fn bound(id: ExpressionBindingId, ty: TypeId) -> ValueExpr {
        ValueExpr::Bound {
            binding: id,
            ty,
        }
    }
    fn scoped(
        bindings: Vec<(ExpressionBindingId, ValueExpr)>,
        body: ValueExpr,
        ty: TypeId,
    ) -> ValueExpr {
        ValueExpr::Bind {
            bindings,
            body: Box::new(body),
            ty,
        }
    }
    fn valid(types: &TypeRegistry, value: &ValueExpr) -> Result<(), IrError> {
        verify_owned_expression_with_context(
            types,
            value,
            &HashMap::new(),
            &[],
            &Places::default(),
            ProcedureId::new(7),
            None,
        )
        .map(|_| ())
    }
    #[test]
    fn sequential_typed_captures_and_nested_scopes_prove() {
        let (types, ty) = types();
        let value = scoped(
            vec![(id(0), ValueExpr::Zero(ty)), (id(1), bound(id(0), ty))],
            scoped(vec![(id(2), bound(id(1), ty))], bound(id(0), ty), ty),
            ty,
        );
        valid(&types, &value).unwrap();
        assert!(
            verify_expression(&types, &value, &HashMap::new(), &[], &Places::default()).is_err()
        );
        assert!(valid(&types, &bound(id(0), ty)).is_err());
    }
    #[test]
    fn forward_cross_owner_duplicate_and_mismatched_uses_reject() {
        let (types, ty) = types();
        let bool_ty = types.scalar(ScalarType::Bool);
        for value in [
            scoped(vec![(id(0), bound(id(0), ty))], bound(id(0), ty), ty),
            scoped(
                vec![(id(0), ValueExpr::Zero(ty))],
                bound(id(0), bool_ty),
                bool_ty,
            ),
            scoped(
                vec![(id(0), ValueExpr::Zero(ty)), (id(0), ValueExpr::Zero(ty))],
                bound(id(0), ty),
                ty,
            ),
            scoped(
                vec![(
                    ExpressionBindingId::new(ProcedureId::new(8), 0),
                    ValueExpr::Zero(ty),
                )],
                ValueExpr::Zero(ty),
                ty,
            ),
            scoped(
                vec![(id(0), ValueExpr::Zero(ty))],
                scoped(vec![(id(0), ValueExpr::Zero(ty))], bound(id(0), ty), ty),
                ty,
            ),
        ] {
            assert!(valid(&types, &value).is_err());
        }
    }
    #[test]
    fn failures_restore_only_inner_installed_ids() {
        let (types, ty) = types();
        let signatures = HashMap::new();
        let places = Places::default();
        let mut proof = Context::root(&types, &signatures, &[], &places, None);
        proof.binding_owner = Some(ProcedureId::new(7));
        proof.expression_bindings.borrow_mut().insert(id(0), ty);
        let failed = proof.bound_scope(
            &[(id(1), bound(id(0), ty)), (id(2), bound(id(2), ty))],
            &bound(id(1), ty),
            ty,
        );
        assert!(failed.is_err());
        assert_eq!(proof.expression_bindings.borrow().len(), 1);
        assert_eq!(proof.expression_bindings.borrow().get(&id(0)), Some(&ty));
        proof
            .bound_scope(&[(id(1), bound(id(0), ty))], &bound(id(1), ty), ty)
            .unwrap();
        assert_eq!(proof.expression_bindings.borrow().len(), 1);
    }
    #[test]
    fn disjoint_scopes_may_reuse_an_id_but_cannot_escape() {
        let (mut types, ty) = types();
        let array_ty = types.fixed_array(ty, 2).unwrap();
        let producer = || scoped(vec![(id(0), ValueExpr::Zero(ty))], bound(id(0), ty), ty);
        let value = ValueExpr::Array {
            ty: array_ty,
            elements: vec![producer(), producer()],
        };
        valid(&types, &value).unwrap();
        let escaped = ValueExpr::Array {
            ty: array_ty,
            elements: vec![producer(), bound(id(0), ty)],
        };
        assert!(valid(&types, &escaped).is_err());
    }
    #[test]
    fn wide_vectors_are_bounded_without_artificial_expression_depth() {
        let (types, ty) = types();
        let value = scoped(
            (0..MAX_EXPRESSION_BINDINGS)
                .map(|index| (id(index), ValueExpr::Zero(ty)))
                .collect(),
            bound(id(MAX_EXPRESSION_BINDINGS - 1), ty),
            ty,
        );
        valid(&types, &value).unwrap();
        let too_wide = scoped(
            (0..=MAX_EXPRESSION_BINDINGS)
                .map(|index| (id(index), ValueExpr::Zero(ty)))
                .collect(),
            ValueExpr::Zero(ty),
            ty,
        );
        assert!(matches!(
            valid(&types, &too_wide),
            Err(IrError::VerificationDepth)
        ));
    }

    #[test]
    fn indexed_places_are_checked_under_each_active_capture_scope() {
        let (mut types, ty) = types();
        let array = types.fixed_array(ty, 2).unwrap();
        let global = Global::new_typed(
            0,
            ConstantValue {
                ty: array,
                kind: ConstantKind::Zero,
            },
            &types,
        )
        .unwrap();
        let mut registry = PlaceRegistry::new();
        let indexed = registry
            .index(
                global.place(),
                IntExpr::new(
                    IntegerType::S64,
                    IntExprKind::Value(Box::new(bound(id(0), ty))),
                ),
                &types,
            )
            .unwrap();
        let places = registry.freeze();
        let signatures = HashMap::new();
        let globals = [global];
        let mut proof = Context::root(&types, &signatures, &globals, &places, None);
        proof.binding_owner = Some(ProcedureId::new(7));
        proof
            .value(&scoped(
                vec![(id(0), ValueExpr::Zero(ty))],
                ValueExpr::Load(indexed),
                ty,
            ))
            .unwrap();
        let boolean = types.scalar(ScalarType::Bool);
        assert!(
            proof
                .value(&scoped(
                    vec![(id(0), ValueExpr::Zero(boolean))],
                    ValueExpr::Load(indexed),
                    ty
                ))
                .is_err()
        );
        assert!(proof.expression_bindings.borrow().is_empty());
    }

    #[test]
    fn root_call_uses_its_explicit_context_owner() {
        let (mut types, ty) = types();
        let callable = types
            .procedure(jai_types::ProcedureType {
                parameters: vec![ty].into(),
                results: vec![ty].into(),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: jai_types::CallingConvention::C,
                context: jai_types::ContextMode::None,
                variadic: jai_types::Variadic::None,
            })
            .unwrap();
        let signatures = HashMap::from([(ProcedureId::new(1), callable)]);
        let call = Call::new(
            ProcedureId::new(1),
            vec![(
                ParameterId::new(0),
                scoped(vec![(id(0), ValueExpr::Zero(ty))], bound(id(0), ty), ty),
            )],
        );
        let places = Places::default();
        let checked = verify_owned_call_with_context(
            &types,
            &call,
            &signatures,
            &[],
            &places,
            ProcedureId::new(7),
            None,
        )
        .unwrap();
        assert_eq!(checked.binding_owner(), Some(ProcedureId::new(7)));
        assert_eq!(checked.results(), &[ty]);
        assert!(verify_call(&types, &call, &signatures, &[], &places).is_err());
        assert!(
            verify_owned_call_with_context(
                &types,
                &call,
                &signatures,
                &[],
                &places,
                ProcedureId::new(8),
                None
            )
            .is_err()
        );
    }
}
