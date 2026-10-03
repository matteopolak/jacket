//! Capture all outputs before installing new bindings or updating existing places.
use super::*;
impl Resolver<'_> {
    pub(crate) fn mixed_results(
        &mut self,
        bindings: &[syntax::ResultTargetBinding],
        annotation: Option<&syntax::TypeSyntax>,
        source: &[syntax::Expression],
    ) -> Result<Statement, Diagnostic> {
        if source.is_empty() {
            return Err(self.error("mixed result bindings require values"));
        }
        let annotated = annotation
            .map(|ty| self.lexical_annotation(ty, self.span))
            .transpose()?;
        let mut statements = Vec::new();
        let mut existing = Vec::new();
        let mut contract_places = Vec::new();
        let mut expected = Vec::new();
        let mut used = Vec::new();
        for binding in bindings {
            match binding {
                syntax::ResultTargetBinding::New {
                    name, ..
                } => {
                    existing.push(None);
                    contract_places.push(None);
                    expected.push(annotated);
                    used.push(self.symbols.name(*name) != "_");
                }
                syntax::ResultTargetBinding::Existing(target) => {
                    if self.is_result_discard_target(target) {
                        existing.push(None);
                        contract_places.push(None);
                        expected.push(None);
                        used.push(false);
                        continue;
                    }
                    let place = self.resolve_place(target)?;
                    contract_places.push(Some(place));
                    self.reject_iteration_write(place, target.span)?;
                    let pointer = self
                        .types
                        .pointer(place.ty())
                        .map_err(|error| Diagnostic::new(target.span, error.to_string()))?;
                    let address = self.allocate_typed(pointer)?;
                    statements.push(Statement::Store(
                        address.place(),
                        ValueExpr::AddressOf {
                            place,
                            ty: pointer,
                        },
                    ));
                    let place = self
                        .places
                        .dereference(ValueExpr::Load(address.place()), self.types)
                        .map_err(|error| Diagnostic::new(target.span, error.to_string()))?;
                    existing.push(Some(place));
                    expected.push(Some(place.ty()));
                    used.push(true);
                }
            }
        }
        let values = self.capture_results(source, &expected, &used, &mut statements)?;
        if values.len() != bindings.len() {
            return Err(self.error("assignment result count does not match binding count"));
        }
        for (index, binding) in bindings.iter().enumerate() {
            if !used[index] {
                continue;
            }
            let (place, span) = match binding {
                syntax::ResultTargetBinding::New {
                    name,
                    span,
                } => (
                    self.declare_typed(*name, annotated.unwrap_or(values[index].ty()))?
                        .place(),
                    *span,
                ),
                syntax::ResultTargetBinding::Existing(target) => (
                    existing[index].expect("existing destination captured"),
                    target.span,
                ),
            };
            let value =
                self.typed_value(ValueExpr::Load(values[index]), values[index].ty(), span)?;
            let value = self.implicit_field_pointer(value, place.ty(), span)?;
            let value = self.coerce_value(value, place.ty(), span)?;
            let contract = match (binding, annotation) {
                (
                    syntax::ResultTargetBinding::New {
                        ..
                    },
                    Some(annotation),
                ) => self.annotation_value_contract(place.ty(), annotation, span)?,
                (syntax::ResultTargetBinding::Existing(_), _) => {
                    let original =
                        contract_places[index].expect("existing binding retains its source place");
                    match self.callback_value_contract(&ValueExpr::Load(original), span)? {
                        Some(contract) => Some(contract),
                        None => self.callback_value_contract(&value, span)?,
                    }
                }
                _ => self.callback_value_contract(&value, span)?,
            };
            if let Some(original) = contract_places[index] {
                self.bind_value_contract(original, contract.clone(), span)?;
            }
            self.bind_value_contract(place, contract, span)?;
            statements.push(Statement::Store(place, value));
        }
        Ok(Statement::Block(Block {
            statements,
            flow: Flow::FallsThrough,
        }))
    }
}
