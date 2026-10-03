//! Shared source contracts for graph and retained lexical specializations.
use super::*;

impl Resolver<'_> {
    pub(crate) fn callback_argument_contract_for_source(
        &mut self,
        source: Option<&syntax::Expression>,
        value: &ValueExpr,
        span: Span,
        depth: usize,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        match source {
            Some(source) => self.callback_expression_contract_inner(source, value, span, depth),
            None => self.callback_value_contract_inner(value, span, depth),
        }
    }

    pub(crate) fn capture_generic_callback_variables(
        &self,
        source: &syntax::TypeSyntax,
        contract: Option<&ValueContract>,
        bindings: &mut HashMap<Symbol, Option<ValueContract>>,
        span: Span,
    ) -> Result<(), Diagnostic> {
        super::generics::capture_variables(source, contract, bindings, span, 0)
    }

    pub(crate) fn merge_generic_callback_bindings(
        &mut self,
        procedure: ProcedureId,
        bindings: &HashMap<Symbol, Option<ValueContract>>,
    ) -> bool {
        let mut changed = false;
        for (&name, contract) in bindings {
            let Some(contract) = contract else {
                continue;
            };
            let key = (procedure, name);
            let merged = match self.meta.callbacks.generic_type_arguments.get(&key) {
                Some(previous) => {
                    let merged = previous.clone().merge(contract.clone());
                    changed |= !previous.same_binding_contract(&merged);
                    merged
                }
                None => {
                    changed = true;
                    contract.clone()
                }
            };
            self.meta
                .callbacks
                .generic_type_arguments
                .insert(key, merged);
        }
        changed
    }

    pub(crate) fn merge_generic_callback_parameter(
        &mut self,
        procedure: ProcedureId,
        name: Symbol,
        contract: ValueContract,
    ) -> bool {
        let key = (procedure, name);
        let (contract, changed) = match self.meta.callbacks.generic_parameters.get(&key) {
            Some(previous) => {
                let merged = previous.clone().merge(contract);
                let changed = !previous.same_binding_contract(&merged);
                (merged, changed)
            }
            None => (contract, true),
        };
        self.meta.callbacks.generic_parameters.insert(key, contract);
        changed
    }

    pub(crate) fn bound_generic_callback_contract(
        &self,
        ty: TypeId,
        source: &ContractSyntax,
        bindings: &HashMap<Symbol, Option<ValueContract>>,
        span: Span,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        self.bound_syntax_contract(ty, source, bindings, span, 0)
    }

    /// The cached body must satisfy every checked use of its source results.
    /// A call's inferred result policy is still derived from that call's inputs.
    pub(crate) fn merge_generic_callback_results(
        &mut self,
        procedure: ProcedureId,
        contracts: &[Option<ValueContract>],
    ) -> bool {
        let previous = self
            .meta
            .callbacks
            .returned_contracts
            .entry(procedure)
            .or_default();
        let mut changed = previous.len() != contracts.len();
        previous.resize(contracts.len(), None);
        for (previous, contract) in previous.iter_mut().zip(contracts) {
            let Some(contract) = contract else {
                continue;
            };
            let merged = match previous.as_ref() {
                Some(previous) => previous.clone().merge(contract.clone()),
                None => contract.clone(),
            };
            changed |= previous
                .as_ref()
                .is_none_or(|previous| !previous.same_binding_contract(&merged));
            *previous = Some(merged);
        }
        changed
    }
}
