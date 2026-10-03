//! Preserve one ordered copy for each scalar/spread variadic segment.
use super::*;
use jai_ir::SequencePackPart;

impl Resolver<'_> {
    pub(crate) fn concat_variadic_pack(
        &mut self,
        prefix: &[&syntax::CallArgument],
        spread: &syntax::CallArgument,
        element: TypeId,
        slice: TypeId,
    ) -> Result<ValueExpr, Diagnostic> {
        let mut parts = Vec::with_capacity(prefix.len() + 1);
        for argument in prefix {
            let value = self.expr_expected(&argument.value, element)?;
            parts.push(SequencePackPart::Element(self.coerce_value(
                value,
                element,
                argument.value.span,
            )?));
        }
        let value = self.expr_expected(&spread.value, slice)?;
        parts.push(SequencePackPart::Spread(self.coerce_value(
            value,
            slice,
            spread.value.span,
        )?));
        Ok(ValueExpr::SequenceConcat {
            ty: slice,
            parts,
        })
    }
}

impl Resolver<'_> {
    /// Diagnose a proven direct alias before relying on the native frame guard.
    pub(crate) fn reject_returned_pack_alias(
        &self,
        value: &ValueExpr,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let mut pending = vec![value];
        while let Some(value) = pending.pop() {
            match value {
                ValueExpr::Bind {
                    bindings,
                    body,
                    ..
                } => {
                    pending.push(body);
                    pending.extend(bindings.iter().map(|(_, value)| value));
                }
                ValueExpr::Call {
                    call, ..
                } => {
                    let procedure = self
                        .meta
                        .local_declarations
                        .ready_procedure(call.procedure)
                        .or_else(|| {
                            self.compile_time
                                .and_then(|context| context.procedures.get(&call.procedure))
                        });
                    if let Some(parameter) = self
                        .graph_scope
                        .and_then(|scope| scope.forwarding_parameter(call.procedure))
                        && call.arguments.iter().any(|(id, value)| {
                            id.index() == parameter
                                && matches!(value, ValueExpr::SequenceConcat { .. })
                        })
                    {
                        return Err(Diagnostic::new(
                            span,
                            "cannot return a view or data address of caller-frame variadic pack storage",
                        ));
                    }
                    let Some(procedure) = procedure else {
                        continue;
                    };
                    let [
                        Statement::Exit(jai_ir::Exit {
                            transfer: jai_ir::Transfer::ReturnValues(results),
                            ..
                        }),
                    ] = procedure.body.statements.as_slice()
                    else {
                        continue;
                    };
                    for result in results {
                        let mut alias = result;
                        loop {
                            alias = match alias {
                                ValueExpr::SequenceField {
                                    base,
                                    field: jai_ir::SequenceField::Data,
                                    ..
                                }
                                | ValueExpr::PointerCast {
                                    value: base, ..
                                }
                                | ValueExpr::SequenceView {
                                    sequence: base, ..
                                } => base,
                                _ => break,
                            };
                        }
                        let ValueExpr::Load(place) = alias else {
                            continue;
                        };
                        let Some(parameter) = procedure
                            .parameters
                            .iter()
                            .position(|local| local.place() == *place)
                        else {
                            continue;
                        };
                        if call.arguments.iter().any(|(id, value)| {
                            id.index() == parameter
                                && matches!(value, ValueExpr::SequenceConcat { .. })
                        }) {
                            return Err(Diagnostic::new(
                                span,
                                "cannot return a view or data address of caller-frame variadic pack storage",
                            ));
                        }
                    }
                }
                ValueExpr::PointerCast {
                    value, ..
                }
                | ValueExpr::Distinct {
                    value, ..
                }
                | ValueExpr::Union {
                    value, ..
                }
                | ValueExpr::UnwrapDistinct {
                    value, ..
                } => pending.push(value),
                ValueExpr::SequenceView {
                    sequence, ..
                } => pending.push(sequence),
                ValueExpr::SequenceField {
                    base,
                    field: jai_ir::SequenceField::Data,
                    ..
                }
                | ValueExpr::Field {
                    base, ..
                } => pending.push(base),
                ValueExpr::Conditional {
                    expression, ..
                } => {
                    pending.push(&expression.then_value);
                    pending.push(&expression.else_value);
                }
                ValueExpr::Array {
                    elements, ..
                }
                | ValueExpr::Record {
                    fields: elements, ..
                } => pending.extend(elements),
                ValueExpr::OrderedRecord {
                    initializers, ..
                } => {
                    pending.extend(initializers.iter().map(|(_, value)| value));
                }
                ValueExpr::RecordBuild {
                    initializers, ..
                } => pending.extend(initializers.iter().map(|(_, value)| value)),
                ValueExpr::SequenceBuild {
                    initializers, ..
                } => pending.extend(initializers.iter().map(|(_, value)| value)),
                _ => {}
            }
        }
        Ok(())
    }
}
