//! Header preparation follows typed dependencies instead of binding unrelated recipes.
use super::*;
use jai_types::FieldId;
use std::collections::HashSet;

#[derive(Default)]
pub(super) struct HeaderPrerequisites {
    pub(super) procedures: HashSet<ProcedureId>,
    pub(super) constants: HashSet<DeclarationId>,
    fields: HashSet<FieldId>,
}

impl HeaderPrerequisites {
    pub(super) fn include_fields(&mut self, fields: &HashSet<FieldId>) {
        self.fields.extend(fields);
    }

    pub(super) fn observe(
        &mut self,
        dependencies: &[jai_vm::Dependency],
        constants: &[DeclarationId],
        fields: &[FieldId],
    ) {
        self.procedures.extend(
            dependencies
                .iter()
                .filter_map(|dependency| match dependency {
                    jai_vm::Dependency::Procedure(id) => Some(*id),
                    _ => None,
                }),
        );
        self.constants.extend(constants);
        self.fields.extend(fields);
    }

    pub(super) fn observe_fields(
        &mut self,
        jobs: &super::super::field_default_jobs::FieldDefaultJobs,
    ) {
        use super::super::field_default_jobs::FieldDefaultDependency;
        loop {
            let before = self.len();
            for (field, dependencies, _) in jobs.waiting() {
                if !self.fields.contains(&field) {
                    continue;
                }
                for dependency in dependencies {
                    match dependency {
                        FieldDefaultDependency::Field(field) => {
                            self.fields.insert(*field);
                        }
                        FieldDefaultDependency::Constant(declaration) => {
                            self.constants.insert(*declaration);
                        }
                        FieldDefaultDependency::Vm(jai_vm::Dependency::Procedure(id)) => {
                            self.procedures.insert(*id);
                        }
                        FieldDefaultDependency::Vm(_) => {}
                    }
                }
            }
            if before == self.len() {
                break;
            }
        }
    }

    pub(super) fn ready(&self, jobs: &super::super::field_default_jobs::FieldDefaultJobs) -> bool {
        use super::super::field_default_jobs::FieldDefaultReadiness;
        self.fields.iter().all(|field| {
            matches!(
                jobs.readiness(*field),
                Some(FieldDefaultReadiness::Ready(_) | FieldDefaultReadiness::NoWrite(_))
            )
        })
    }

    pub(super) fn len(&self) -> usize {
        self.procedures.len() + self.constants.len() + self.fields.len()
    }
}
