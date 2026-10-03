//! Admit the exact lexical snapshot before compiler Code retains its clone.
use super::*;
use crate::metaprogram::CompilerQuoteBudget;

impl LocalScopes {
    pub(crate) fn admit_compiler_clone(
        &self,
        budget: &mut CompilerQuoteBudget,
        span: Span,
    ) -> Result<(), Diagnostic> {
        // Map clone retains its allocated buckets, including unused capacity.
        // Sixteen metadata units cover each bucket and its keys/source facts;
        // AST payloads are walked separately below, before their deep clones.
        let mut entries = self.active.capacity().saturating_add(self.frames.len());
        for frame in &self.frames {
            for count in [
                frame.declarations.capacity(),
                frame.operators.len(),
                frame.operator_imports.len(),
                frame.runtime.capacity(),
                frame.external.capacity(),
                frame.runtime_symbols.capacity(),
                frame.imports.capacity(),
                frame.imports.placeholders().count(),
                frame.import_aliases.capacity(),
                frame.using_bindings.capacity(),
                frame.using_pending.capacity(),
                frame.using_placeholders.capacity(),
                frame.using_origins.capacity(),
                frame.using_operator_declarations.len(),
            ] {
                entries = entries.saturating_add(count);
            }
            for declaration in frame.declarations.values().chain(&frame.operators) {
                entries = entries
                    .saturating_add(declaration.imports.capacity())
                    .saturating_add(declaration.imports.placeholders().count())
                    .saturating_add(declaration.operator_imports.len())
                    .saturating_add(declaration.using.compiler_retention_entries());
            }
        }
        budget.retain_metadata(entries.saturating_mul(16), 0, span)?;
        for frame in &self.frames {
            for declaration in frame.declarations.values().chain(&frame.operators) {
                match &declaration.syntax {
                    DeclarationSyntax::Library(value) => {
                        budget.retain_metadata(1, value.target.len(), span)?
                    }
                    DeclarationSyntax::Record(value) => budget.admit_record(value, span)?,
                    DeclarationSyntax::Enum(value) => budget.admit_enum(value, span)?,
                    DeclarationSyntax::Alias(value) => budget.admit_type(&value.ty, span)?,
                    DeclarationSyntax::Constant(value) => budget.admit_constant(value, span)?,
                    // Its retained original AST is shared by Arc; cloning the
                    // projection copies only the already charged wrapper.
                    DeclarationSyntax::ConstantResult(_) => {}
                    DeclarationSyntax::Procedure(value) => budget.admit_procedure(value, span)?,
                    DeclarationSyntax::Prototype(value) => budget.admit_prototype(value, span)?,
                }
            }
            for value in frame.runtime.values() {
                budget.admit_declaration(value, span)?;
            }
        }
        Ok(())
    }
}
