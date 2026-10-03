//! Allocation policy keyed by checked storage identity, independently of types.
use crate::{Global, GlobalId, IrError, LocalId, Procedure, ProcedureId};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Default)]
pub struct StorageAlignments {
    locals: HashMap<LocalId, u32>,
    globals: HashMap<GlobalId, u32>,
}
impl StorageAlignments {
    pub fn set_local(&mut self, id: LocalId, alignment: u32) -> Result<(), IrError> {
        valid(alignment)?;
        self.locals.insert(id, alignment);
        Ok(())
    }
    pub fn set_global(&mut self, id: GlobalId, alignment: u32) -> Result<(), IrError> {
        valid(alignment)?;
        self.globals.insert(id, alignment);
        Ok(())
    }
    pub fn local(&self, id: LocalId) -> Option<u32> {
        self.locals.get(&id).copied()
    }
    pub fn global(&self, id: GlobalId) -> Option<u32> {
        self.globals.get(&id).copied()
    }
    /// A retried source body must replace its previous allocation requests.
    pub fn clear_procedure(&mut self, procedure: ProcedureId) {
        self.locals.retain(|id, _| id.procedure() != procedure);
    }
    pub(crate) fn validate(
        &self,
        procedures: &[Procedure],
        globals: &[Global],
    ) -> Result<(), IrError> {
        let locals: HashSet<_> = procedures
            .iter()
            .flat_map(|procedure| {
                procedure
                    .parameters
                    .iter()
                    .chain(&procedure.locals)
                    .map(|local| local.id())
            })
            .collect();
        for (id, alignment) in &self.locals {
            valid(*alignment)?;
            if !locals.contains(id) {
                return Err(IrError::UnknownIdentity {
                    kind: "local storage alignment",
                    index: id.index(),
                });
            }
        }
        for (id, alignment) in &self.globals {
            valid(*alignment)?;
            if !globals
                .get(id.index())
                .is_some_and(|global| global.id() == *id)
            {
                return Err(IrError::UnknownIdentity {
                    kind: "global storage alignment",
                    index: id.index(),
                });
            }
        }
        Ok(())
    }
}

fn valid(alignment: u32) -> Result<(), IrError> {
    if alignment.is_power_of_two() {
        Ok(())
    } else {
        Err(IrError::InvalidStorageAlignment(alignment))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Block, Flow, GlobalInitializer, Local, ProgramBuilder};
    use jai_types::{
        CallingConvention, ContextMode, IntegerType, ProcedureType, ScalarType, TypeRegistry,
        Variadic,
    };

    #[test]
    fn invalid_requests_are_rejected_before_publication() {
        let types = TypeRegistry::new();
        let local = Local::new(
            ProcedureId::new(0),
            0,
            ScalarType::Int(IntegerType::S64),
            &types,
        );
        let mut metadata = StorageAlignments::default();
        for invalid in [0, 3, 6, u32::MAX] {
            assert!(matches!(
                metadata.set_local(local.id(), invalid),
                Err(IrError::InvalidStorageAlignment(_))
            ));
            assert!(matches!(
                metadata.set_global(GlobalId::new(0), invalid),
                Err(IrError::InvalidStorageAlignment(_))
            ));
        }
        assert_eq!(metadata.local(local.id()), None);
    }

    #[test]
    fn sidecar_publication_checks_exact_local_owners_and_global_indices() {
        let mut types = TypeRegistry::new();
        let signature = types
            .procedure(ProcedureType {
                parameters: vec![].into(),
                results: vec![].into(),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::Jai,
                context: ContextMode::None,
                variadic: Variadic::None,
            })
            .unwrap();
        let scalar = ScalarType::Int(IntegerType::S64);
        let local = Local::new(ProcedureId::new(0), 0, scalar, &types);
        let foreign = Local::new(ProcedureId::new(1), 0, scalar, &types);
        let missing = Local::new(ProcedureId::new(0), 1, scalar, &types);
        let procedure = Procedure {
            id: ProcedureId::new(0),
            signature,
            parameters: vec![],
            locals: vec![local],
            body: Block {
                statements: vec![],
                flow: Flow::FallsThrough,
            },
            cleanups: vec![],
        };
        let global = Global::new(0, GlobalInitializer::Bool(false), &types);
        for bad in [foreign.id(), missing.id()] {
            let mut metadata = StorageAlignments::default();
            metadata.set_local(bad, 64).unwrap();
            assert!(matches!(
                metadata.validate(
                    std::slice::from_ref(&procedure),
                    std::slice::from_ref(&global)
                ),
                Err(IrError::UnknownIdentity {
                    kind: "local storage alignment",
                    ..
                })
            ));
        }
        let mut metadata = StorageAlignments::default();
        metadata.set_global(GlobalId::new(1), 64).unwrap();
        assert!(matches!(
            metadata.validate(
                std::slice::from_ref(&procedure),
                std::slice::from_ref(&global)
            ),
            Err(IrError::UnknownIdentity {
                kind: "global storage alignment",
                ..
            })
        ));
        let mut metadata = StorageAlignments::default();
        metadata.set_local(local.id(), 128).unwrap();
        metadata.set_global(global.id(), 64).unwrap();
        let library = ProgramBuilder::new(types.freeze().unwrap())
            .procedures(vec![procedure])
            .globals(vec![global])
            .storage_alignments(metadata)
            .finish_library()
            .unwrap();
        assert_eq!(library.storage_alignments().local(local.id()), Some(128));
        assert_eq!(
            library.storage_alignments().global(GlobalId::new(0)),
            Some(64)
        );
    }

    #[test]
    fn retries_replace_only_the_selected_procedures_requests() {
        let types = TypeRegistry::new();
        let mut metadata = StorageAlignments::default();
        let first = Local::new(ProcedureId::new(0), 0, ScalarType::Bool, &types);
        let second = Local::new(ProcedureId::new(1), 0, ScalarType::Bool, &types);
        metadata.set_local(first.id(), 64).unwrap();
        metadata.set_local(second.id(), 128).unwrap();
        metadata.set_global(GlobalId::new(0), 256).unwrap();
        metadata.clear_procedure(first.id().procedure());
        assert_eq!(metadata.local(first.id()), None);
        assert_eq!(metadata.local(second.id()), Some(128));
        assert_eq!(metadata.global(GlobalId::new(0)), Some(256));
    }

    #[test]
    fn parameter_alignment_keeps_its_exact_owner() {
        let mut types = TypeRegistry::new();
        let integer = types.scalar(ScalarType::Int(IntegerType::S64));
        let signature = types
            .procedure(ProcedureType {
                parameters: vec![integer].into(),
                results: vec![].into(),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::Jai,
                context: ContextMode::None,
                variadic: Variadic::None,
            })
            .unwrap();
        let parameter = Local::new(
            ProcedureId::new(0),
            0,
            ScalarType::Int(IntegerType::S64),
            &types,
        );
        let procedure = Procedure {
            id: ProcedureId::new(0),
            signature,
            parameters: vec![parameter],
            locals: vec![parameter],
            body: Block {
                statements: vec![],
                flow: Flow::FallsThrough,
            },
            cleanups: vec![],
        };
        let mut foreign = StorageAlignments::default();
        foreign
            .set_local(
                Local::new(
                    ProcedureId::new(1),
                    0,
                    ScalarType::Int(IntegerType::S64),
                    &types,
                )
                .id(),
                64,
            )
            .unwrap();
        assert!(matches!(
            foreign.validate(std::slice::from_ref(&procedure), &[]),
            Err(IrError::UnknownIdentity {
                kind: "local storage alignment",
                ..
            })
        ));
        let mut metadata = StorageAlignments::default();
        metadata.set_local(parameter.id(), 64).unwrap();
        let library = ProgramBuilder::new(types.freeze().unwrap())
            .procedures(vec![procedure])
            .storage_alignments(metadata)
            .finish_library()
            .unwrap();
        assert_eq!(library.storage_alignments().local(parameter.id()), Some(64));
    }
}
