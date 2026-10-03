//! Actual immutable graph owners; no semantic registry or preview reconstruction.
use super::*;
use std::{collections::HashSet, mem::size_of};

/// Every variant borrows a real owned root. Declarations and directives are
/// independent clones of parsed-file syntax, so both roots must be inspected.
pub enum GraphSyntaxStorage<'a> {
    ParsedFile(&'a jai_syntax::ParsedFile),
    FileDeclaration(&'a jai_syntax::FileDeclaration),
    FileItem(&'a jai_syntax::FileItem),
    RunDirective(&'a jai_syntax::RunDirective),
    InsertDirective(&'a jai_syntax::InsertDirective),
    ContextFieldDeclaration(&'a jai_syntax::ContextFieldDeclaration),
    ScalarValue(&'a jai_eval::Value),
}
pub(super) fn vector<T, E>(
    values: &Vec<T>,
    charge: &mut impl FnMut(usize) -> Result<(), E>,
    metadata: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<(), E> {
    charge(values.capacity().saturating_add(1))?;
    metadata(values.capacity().saturating_mul(size_of::<T>()))
}
pub(super) fn map<K, V, S, E>(
    values: &HashMap<K, V, S>,
    charge: &mut impl FnMut(usize) -> Result<(), E>,
    metadata: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<(), E> {
    charge(values.capacity().saturating_add(1))?;
    metadata(table(values.capacity(), size_of::<(K, V)>()))
}
fn set<T, S, E>(
    values: &HashSet<T, S>,
    charge: &mut impl FnMut(usize) -> Result<(), E>,
    metadata: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<(), E> {
    charge(values.capacity().saturating_add(1))?;
    metadata(table(values.capacity(), size_of::<T>()))
}
fn table(capacity: usize, slot: usize) -> usize {
    if capacity == 0 {
        0
    } else {
        capacity
            .saturating_mul(2)
            .saturating_mul(slot.saturating_add(1))
            .saturating_add(32)
    }
}
fn argument<E>(
    value: &ModuleBoundArgument,
    charge: &mut impl FnMut(usize) -> Result<(), E>,
    metadata: &mut impl FnMut(usize) -> Result<(), E>,
    ty: &mut impl FnMut(&ModuleType) -> Result<(), E>,
) -> Result<(), E> {
    charge(1)?;
    match value {
        ModuleBoundArgument::Type(value) => ty(value),
        ModuleBoundArgument::String(value) => metadata(value.len()),
        ModuleBoundArgument::Integer(_)
        | ModuleBoundArgument::Boolean(_)
        | ModuleBoundArgument::Float(_)
        | ModuleBoundArgument::Enumeration(_) => Ok(()),
    }
}
fn specialization<E>(
    key: &SourceSpecializationKey,
    charge: &mut impl FnMut(usize) -> Result<(), E>,
    metadata: &mut impl FnMut(usize) -> Result<(), E>,
    ty: &mut impl FnMut(&ModuleType) -> Result<(), E>,
) -> Result<(), E> {
    charge(1)?;
    metadata(
        key.arguments()
            .len()
            .saturating_mul(size_of::<(Symbol, ModuleBoundArgument)>()),
    )?;
    for (_, value) in key.arguments() {
        argument(value, charge, metadata, ty)?;
    }
    Ok(())
}
fn parameter<E>(
    value: &ParameterValue,
    charge: &mut impl FnMut(usize) -> Result<(), E>,
    metadata: &mut impl FnMut(usize) -> Result<(), E>,
    syntax: &mut impl FnMut(GraphSyntaxStorage<'_>) -> Result<(), E>,
    ty: &mut impl FnMut(&ModuleType) -> Result<(), E>,
) -> Result<(), E> {
    charge(1)?;
    match value {
        ParameterValue::Scalar(value) => syntax(GraphSyntaxStorage::ScalarValue(value)),
        ParameterValue::Type(value) => ty(value),
        ParameterValue::String(value) => metadata(value.capacity()),
        ParameterValue::Enumeration(_) | ParameterValue::ContextualMember(_) => Ok(()),
    }
}
impl ModuleGraph {
    /// Inspect the graph's real container, source, syntax and publication forest
    /// before output ownership copies. Callbacks borrow under this graph guard.
    /// This excludes the separate mutable Builder/request/append-plan owner.
    pub fn visit_retained_source_storage<E>(
        &self,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
        metadata: &mut impl FnMut(usize) -> Result<(), E>,
        source: &mut impl FnMut(&jai_source::SourceRecord) -> Result<(), E>,
        syntax: &mut impl FnMut(GraphSyntaxStorage<'_>) -> Result<(), E>,
        capture: &mut impl FnMut(&SourceCaptureValue) -> Result<(), E>,
        ty: &mut impl FnMut(&ModuleType) -> Result<(), E>,
    ) -> Result<(), E> {
        charge(1)?;
        metadata(size_of::<Self>())?;
        if let Some(target) = &self.target {
            charge(1)?;
            if let jai_types::OperatingSystem::Other(value) = &target.operating_system {
                metadata(value.capacity())?;
            }
            if let jai_types::Architecture::Other(value) = &target.architecture {
                metadata(value.capacity())?;
            }
        }
        self.symbols.visit_retained_metadata(&mut |work, bytes| {
            charge(work)?;
            metadata(bytes)
        })?;
        self.sources.visit_retained_record_storage(
            &mut |work, bytes| {
                charge(work)?;
                metadata(bytes)
            },
            source,
        )?;
        vector(&self.files, charge, metadata)?;
        for file in &self.files {
            charge(1)?;
            map(&file.private, charge, metadata)?;
            vector(&file.declarations, charge, metadata)?;
            syntax(GraphSyntaxStorage::ParsedFile(&file.syntax))?;
        }
        vector(&self.modules, charge, metadata)?;
        for module in &self.modules {
            charge(1)?;
            vector(&module.files, charge, metadata)?;
            map(&module.bindings, charge, metadata)?;
            map(&module.exports, charge, metadata)?;
        }
        vector(&self.declarations, charge, metadata)?;
        for declaration in &self.declarations {
            charge(1)?;
            syntax(GraphSyntaxStorage::FileDeclaration(&declaration.syntax))?;
        }
        self.placeholders
            .visit_retained_metadata(charge, metadata)?;
        self.storage_members
            .visit_retained_metadata(charge, metadata)?;
        vector(&self.imports, charge, metadata)?;
        vector(&self.loads, charge, metadata)?;
        vector(&self.dependency_templates, charge, metadata)?;
        vector(&self.scoped_imports, charge, metadata)?;
        for item in &self.scoped_imports {
            charge(1)?;
            if let Some(key) = item.specialization() {
                specialization(key, charge, metadata, ty)?;
            }
        }
        vector(&self.source_conditions, charge, metadata)?;
        for item in &self.source_conditions {
            charge(1)?;
            if let Some(key) = item.specialization() {
                specialization(key, charge, metadata, ty)?;
            }
        }
        vector(&self.source_cases, charge, metadata)?;
        for item in &self.source_cases {
            charge(1)?;
            if let Some(key) = &item.specialization {
                specialization(key, charge, metadata, ty)?;
            }
        }
        vector(&self.source_specializations, charge, metadata)?;
        for key in &self.source_specializations {
            specialization(key, charge, metadata, ty)?;
        }
        vector(&self.using_publications, charge, metadata)?;
        for publication in &self.using_publications {
            charge(1)?;
            if let Some(key) = &publication.specialization {
                specialization(key, charge, metadata, ty)?;
            }
            vector(&publication.bindings, charge, metadata)?;
            vector(
                &publication.selected_operator_declarations,
                charge,
                metadata,
            )?;
            vector(&publication.aliases, charge, metadata)?;
            vector(&publication.placeholders, charge, metadata)?;
        }
        vector(&self.runs, charge, metadata)?;
        for run in &self.runs {
            charge(1)?;
            syntax(GraphSyntaxStorage::RunDirective(&run.syntax))?;
        }
        vector(&self.insertions, charge, metadata)?;
        for insertion in &self.insertions {
            charge(1)?;
            syntax(GraphSyntaxStorage::InsertDirective(&insertion.directive))?;
        }
        vector(&self.context_fields, charge, metadata)?;
        for field in &self.context_fields {
            charge(1)?;
            syntax(GraphSyntaxStorage::ContextFieldDeclaration(&field.syntax))?;
        }
        vector(&self.insertion_publications, charge, metadata)?;
        for publication in &self.insertion_publications {
            charge(1)?;
            vector(&publication.declarations, charge, metadata)?;
            set(&publication.own_names, charge, metadata)?;
            // Conservative full charge for each retained code Arc. No invented
            // sharing credit is inferred from equal source spans or contents.
            metadata(size_of::<DeclarationInsertionCode>() + 3 * size_of::<usize>())?;
            let code = &publication.code;
            vector(&code.items, charge, metadata)?;
            for item in &code.items {
                charge(1)?;
                syntax(GraphSyntaxStorage::FileItem(item))?;
            }
            vector(&code.bindings, charge, metadata)?;
            vector(&code.values, charge, metadata)?;
            for (_, value) in &code.values {
                charge(1)?;
                capture(value)?;
            }
            vector(&code.origins, charge, metadata)?;
        }
        vector(&self.parameters, charge, metadata)?;
        for item in &self.parameters {
            parameter(&item.value, charge, metadata, syntax, ty)?;
        }
        map(&self.source_requests, charge, metadata)?;
        for request in self.source_requests.values() {
            charge(1)?;
            metadata(request.path.capacity())?;
            if let Some(arguments) = &request.arguments {
                vector(arguments, charge, metadata)?;
                for item in arguments {
                    parameter(&item.value, charge, metadata, syntax, ty)?;
                }
            }
        }
        vector(&self.overload_sets, charge, metadata)?;
        for set in &self.overload_sets {
            charge(1)?;
            metadata(
                set.declarations
                    .len()
                    .saturating_mul(size_of::<DeclarationId>()),
            )?;
        }
        Ok(())
    }
}
