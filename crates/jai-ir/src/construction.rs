//! The only publication boundary for checked libraries and executable programs.
use crate::ForeignLibrary;
use crate::{
    ContextDefinition, EntryPoint, Global, IrError, Library, Places, Procedure, ProcedureId,
    ProcedurePrototype, Program, verify,
};
use jai_source::DeclarationId;
use jai_types::Types;
use std::collections::HashMap;

/// Collect staging nodes, then check the complete graph against its frozen types.
#[derive(Debug)]
pub struct ProgramBuilder {
    source_procedure_owners: crate::SourceProcedureOwners,
    source_warnings: crate::SourceWarnings,
    storage_alignments: crate::StorageAlignments,
    program_exports: Vec<crate::ProgramExport>,
    procedure_hints: HashMap<ProcedureId, jai_types::InlineHint>,
    procedure_phases: crate::ProcedurePhases,
    debug_sources: Option<crate::DebugSources>,
    foreign_libraries: Vec<ForeignLibrary>,
    types: Option<Types>,
    procedures: Vec<Procedure>,
    prototypes: Vec<ProcedurePrototype>,
    context: Option<ContextDefinition>,
    globals: Vec<Global>,
    places: Places,
    declarations: HashMap<DeclarationId, ProcedureId>,
}
impl ProgramBuilder {
    pub fn new(types: Types) -> Self {
        Self {
            source_procedure_owners: crate::SourceProcedureOwners::default(),
            source_warnings: crate::SourceWarnings::default(),
            storage_alignments: crate::StorageAlignments::default(),
            program_exports: vec![],
            procedure_hints: HashMap::new(),
            procedure_phases: crate::ProcedurePhases::default(),
            debug_sources: None,
            types: Some(types),
            procedures: vec![],
            prototypes: vec![],
            foreign_libraries: vec![],
            context: None,
            globals: vec![],
            places: Places::default(),
            declarations: HashMap::new(),
        }
    }
    pub fn source_warnings(mut self, warnings: crate::SourceWarnings) -> Self {
        self.source_warnings = warnings;
        self
    }
    pub fn source_procedure_owners(mut self, owners: crate::SourceProcedureOwners) -> Self {
        self.source_procedure_owners = owners;
        self
    }
    pub fn debug_sources(mut self, sources: crate::DebugSources) -> Self {
        self.debug_sources = Some(sources);
        self
    }
    pub fn storage_alignments(mut self, alignments: crate::StorageAlignments) -> Self {
        self.storage_alignments = alignments;
        self
    }
    /// Attach source policy by procedure identity, independently of debug information.
    pub fn procedure_hints(mut self, hints: HashMap<ProcedureId, jai_types::InlineHint>) -> Self {
        self.procedure_hints = hints;
        self
    }
    pub fn procedure_phases(mut self, phases: crate::ProcedurePhases) -> Self {
        self.procedure_phases = phases;
        self
    }
    pub fn procedures(mut self, procedures: Vec<Procedure>) -> Self {
        crate::disposal::procedures(std::mem::replace(&mut self.procedures, procedures));
        self
    }
    pub fn prototypes(mut self, prototypes: Vec<ProcedurePrototype>) -> Self {
        self.prototypes = prototypes;
        self
    }
    pub fn foreign_libraries(mut self, libraries: Vec<ForeignLibrary>) -> Self {
        self.foreign_libraries = libraries;
        self
    }
    pub fn program_exports(mut self, exports: Vec<crate::ProgramExport>) -> Self {
        self.program_exports = exports;
        self
    }
    pub fn context(mut self, definition: ContextDefinition) -> Self {
        if let Some(previous) = self.context.replace(definition) {
            crate::disposal::constant(previous.default);
        }
        self
    }
    pub fn globals(mut self, globals: Vec<Global>) -> Self {
        crate::disposal::globals(std::mem::replace(&mut self.globals, globals));
        self
    }
    pub fn places(mut self, places: Places) -> Self {
        crate::disposal::places(&mut self.places);
        self.places = places;
        self
    }
    pub fn declarations(mut self, declarations: HashMap<DeclarationId, ProcedureId>) -> Self {
        self.declarations = declarations;
        self
    }
    pub fn finish_library(mut self) -> Result<Library, IrError> {
        self.storage_alignments
            .validate(&self.procedures, &self.globals)?;
        crate::procedure_hints::validate(&self.procedure_hints, &self.procedures)?;
        self.procedure_phases.validate(&self.procedures)?;
        let signatures: HashMap<_, _> = self
            .procedures
            .iter()
            .map(|procedure| (procedure.id, procedure.signature))
            .chain(
                self.prototypes
                    .iter()
                    .map(|prototype| (prototype.id, prototype.signature)),
            )
            .collect();
        let procedure_indices = self
            .procedures
            .iter()
            .enumerate()
            .map(|(index, procedure)| (procedure.id, index))
            .collect::<HashMap<_, _>>();
        if let Some(sources) = &self.debug_sources {
            sources.validate(|id| signatures.contains_key(&id))?;
            sources.validate_type_sources(self.types.as_ref().expect("live builder types"))?;
            sources.validate_ir(|id| {
                procedure_indices
                    .get(&id)
                    .map(|index| &self.procedures[*index])
            })?;
        }
        let library = Library {
            source_procedure_owners: std::mem::take(&mut self.source_procedure_owners),
            source_warnings: std::mem::take(&mut self.source_warnings),
            storage_alignments: std::mem::take(&mut self.storage_alignments),
            program_exports: std::mem::take(&mut self.program_exports),
            procedure_hints: std::mem::take(&mut self.procedure_hints),
            procedure_phases: std::mem::take(&mut self.procedure_phases),
            debug_sources: self.debug_sources.take(),
            foreign_libraries: std::mem::take(&mut self.foreign_libraries),
            procedures: std::mem::take(&mut self.procedures),
            prototypes: std::mem::take(&mut self.prototypes),
            context: self.context.take(),
            procedure_indices,
            globals: std::mem::take(&mut self.globals),
            types: self
                .types
                .take()
                .expect("builder types are present until publication"),
            places: std::mem::take(&mut self.places),
            procedure_ids: std::mem::take(&mut self.declarations),
            signatures,
        };
        verify::library(&library)?;
        Ok(library)
    }
    pub fn finish(self, entry: EntryPoint) -> Result<Program, IrError> {
        self.finish_library()?.into_program(entry)
    }
}
impl Drop for ProgramBuilder {
    fn drop(&mut self) {
        crate::disposal::staging(
            std::mem::take(&mut self.procedures),
            std::mem::take(&mut self.globals),
            self.context.take(),
            &mut self.places,
        );
    }
}
