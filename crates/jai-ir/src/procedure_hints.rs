//! Check source policy ownership at the immutable program publication boundary.
use crate::{IrError, Procedure, ProcedureId};
use jai_types::InlineHint;
use std::collections::{HashMap, HashSet};

pub(crate) fn validate(
    hints: &HashMap<ProcedureId, InlineHint>,
    procedures: &[Procedure],
) -> Result<(), IrError> {
    let definitions: HashSet<_> = procedures.iter().map(|procedure| procedure.id).collect();
    for id in hints.keys() {
        if !definitions.contains(id) {
            return Err(IrError::UnknownIdentity {
                kind: "procedure inlining hint definition",
                index: id.index(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inlining_hint_cannot_target_a_bodyless_or_missing_procedure() {
        for origin in [
            None,
            Some(crate::PrototypeOrigin::Compiler),
            Some(crate::PrototypeOrigin::Foreign {
                symbol: "external".into(),
                library: None,
            }),
        ] {
            let mut types = jai_types::TypeRegistry::new();
            let signature = types
                .procedure(jai_types::ProcedureType {
                    parameters: vec![].into(),
                    results: vec![].into(),
                    return_abi: jai_types::ForeignReturnAbi::Natural,
                    convention: jai_types::CallingConvention::Jai,
                    context: jai_types::ContextMode::None,
                    variadic: jai_types::Variadic::None,
                })
                .unwrap();
            let id = ProcedureId::new(0);
            let error = crate::ProgramBuilder::new(types.freeze().unwrap())
                .procedure_hints(HashMap::from([(id, InlineHint::Always)]))
                .prototypes(
                    origin
                        .into_iter()
                        .map(|origin| crate::ProcedurePrototype {
                            id,
                            signature,
                            origin,
                        })
                        .collect(),
                )
                .finish_library()
                .unwrap_err();
            assert!(matches!(
                error,
                IrError::UnknownIdentity {
                    kind: "procedure inlining hint definition",
                    index: 0
                }
            ));
        }
    }
}
