//! Validate body phase policy before publishing immutable procedure identities.
use crate::{IrError, Procedure, ProcedureId};
use jai_types::ProcedureExecution;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Default)]
pub struct ProcedurePhases(HashMap<ProcedureId, ProcedureExecution>);

impl ProcedurePhases {
    pub fn insert(&mut self, id: ProcedureId, phase: ProcedureExecution) {
        self.0.insert(id, phase);
    }
    pub fn get(&self, id: ProcedureId) -> ProcedureExecution {
        self.0.get(&id).copied().unwrap_or_default()
    }
    pub(crate) fn validate(&self, procedures: &[Procedure]) -> Result<(), IrError> {
        let definitions: HashSet<_> = procedures.iter().map(|procedure| procedure.id).collect();
        for id in self.0.keys() {
            if !definitions.contains(id) {
                return Err(IrError::UnknownIdentity {
                    kind: "procedure execution phase definition",
                    index: id.index(),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn phase_policy_requires_a_defined_body_not_a_bodyless_signature() {
        let mut phases = ProcedurePhases::default();
        phases.insert(ProcedureId::new(123), ProcedureExecution::CompileTimeOnly);
        assert!(matches!(
            phases.validate(&[]),
            Err(IrError::UnknownIdentity { index: 123, .. })
        ));
        assert_eq!(
            ProcedurePhases::default().get(ProcedureId::new(123)),
            ProcedureExecution::RuntimeAndCompileTime
        );
    }

    #[test]
    fn publication_preserves_sparse_body_phase_and_rejects_bodyless_metadata() {
        let mut types = jai_types::TypeRegistry::new();
        let signature = types
            .procedure(jai_types::ProcedureType {
                parameters: vec![].into(),
                results: vec![].into(),
                convention: jai_types::CallingConvention::Jai,
                context: jai_types::ContextMode::None,
                variadic: jai_types::Variadic::None,
            })
            .unwrap();
        let id = ProcedureId::new(123);
        let mut phases = ProcedurePhases::default();
        phases.insert(id, ProcedureExecution::CompileTimeOnly);
        let error = crate::ProgramBuilder::new(types.freeze().unwrap())
            .procedure_phases(phases.clone())
            .prototypes(vec![crate::ProcedurePrototype {
                id,
                signature,
                origin: crate::PrototypeOrigin::Compiler,
            }])
            .finish_library()
            .unwrap_err();
        assert!(matches!(error, IrError::UnknownIdentity { index: 123, .. }));
        let mut types = jai_types::TypeRegistry::new();
        let signature = types
            .procedure(jai_types::ProcedureType {
                parameters: vec![].into(),
                results: vec![].into(),
                convention: jai_types::CallingConvention::Jai,
                context: jai_types::ContextMode::None,
                variadic: jai_types::Variadic::None,
            })
            .unwrap();
        let body = crate::Procedure {
            id,
            signature,
            parameters: vec![],
            locals: vec![],
            cleanups: vec![],
            body: crate::Block {
                statements: vec![crate::Statement::Exit(crate::Exit {
                    cleanups: vec![],
                    transfer: crate::Transfer::ReturnVoid,
                })],
                flow: crate::Flow::Terminates,
            },
        };
        let library = crate::ProgramBuilder::new(types.freeze().unwrap())
            .procedure_phases(phases)
            .procedures(vec![body])
            .finish_library()
            .unwrap();
        assert_eq!(
            library.procedure_phase(id),
            ProcedureExecution::CompileTimeOnly
        );
        assert_eq!(
            library.procedure_phase(ProcedureId::new(0)),
            ProcedureExecution::RuntimeAndCompileTime
        );
    }
}
