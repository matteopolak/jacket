//! Source capture conversion occurs while the selected compiler frame is live.
use super::compiler_quotes::{CompilerQuoteBinding, CompilerQuoteTemplate};
use super::declaration_capture::SourceBinding;
use super::*;
use jai_modules::DeclarationInsertionCode;
use jai_vm::{CompilerCodeSelection, CompilerEffects, Limits, ProcedureProvider, Vm};

pub(crate) enum CompilerQuoteFinishError {
    Source(Diagnostic),
    Vm(jai_vm::Error),
}
impl From<Diagnostic> for CompilerQuoteFinishError {
    fn from(error: Diagnostic) -> Self {
        Self::Source(error)
    }
}
impl From<jai_vm::Error> for CompilerQuoteFinishError {
    fn from(error: jai_vm::Error) -> Self {
        Self::Vm(error)
    }
}

impl Resolver<'_> {
    pub(crate) fn compiler_declaration_insertion_code<P, E>(
        &self,
        template: &CompilerQuoteTemplate,
        vm: &Vm<'_, P, E>,
        selected: &CompilerCodeSelection<'_>,
        visibility: syntax::Visibility,
        mode: syntax::InsertScope,
        limits: Limits,
    ) -> Result<DeclarationInsertionCode, CompilerQuoteFinishError>
    where
        P: ProcedureProvider + ?Sized,
        E: CompilerEffects,
    {
        let source = template.source();
        let location = source.location;
        if selected.site() != template.site()
            || selected.frame().plan() != template.plan()
            || selected.captures() != template.captures()
        {
            return Err(Diagnostic::at_source(
                location,
                "compiler quotation was selected by another plan, frame, or return site",
            )
            .into());
        }
        let limits = self.compiler_quote_publication_limits(template, vm, limits)?;
        template.admit_publication_body()?;
        let items = super::declaration_members::literal_file_items(
            template.body(),
            location.source,
            visibility,
        )
        .map_err(|error| error.with_fallback_source(location.source))?;
        let capture_scope = self
            .graph_scope
            .ok_or_else(|| {
                Diagnostic::at_source(
                    location,
                    "compiler quotation capture requires its defining source graph",
                )
            })?
            .in_file(source.file);
        let mut bindings = Vec::new();
        let mut values = Vec::new();
        if mode == syntax::InsertScope::Captured {
            let mut effective = HashMap::new();
            for frame in template.frames() {
                for (name, binding) in frame.iter() {
                    effective.insert(*name, binding);
                }
            }
            let mut effective = effective.into_iter().collect::<Vec<_>>();
            effective.sort_by_key(|(name, _)| self.symbols.name(*name));
            let mut nodes = 0;
            let mut bytes = 0;
            let mut native_slots = Vec::new();
            let mut native_seen = std::collections::HashSet::new();
            for (_, binding) in &effective {
                if let CompilerQuoteBinding::Native { slot, ty } = binding {
                    if !native_seen.insert(*slot) {
                        continue;
                    }
                    let (actual, value) = selected.native_value(*slot)?;
                    if actual != *ty {
                        return Err(Diagnostic::at_source(
                            location,
                            "compiler quotation selected a native capture with another type",
                        )
                        .into());
                    }
                    native_slots.push((*slot, *ty, value));
                }
            }
            let snapshots = crate::compile_time::materialize_compiler_captures(
                vm,
                self.types,
                &native_slots,
                limits,
                selected.publication_occurrence(),
            )?
            .into_iter()
            .collect::<HashMap<_, _>>();
            for (name, binding) in effective {
                lexical_keys::charge(
                    &mut nodes,
                    1,
                    lexical_keys::MAX_LEXICAL_NODES,
                    location.span,
                )?;
                let binding = match binding {
                    CompilerQuoteBinding::Lexical(storage) => self
                        .source_insertion_binding_in_scope(
                            Binding::Storage(*storage),
                            location,
                            &mut nodes,
                            &mut bytes,
                            capture_scope,
                        )?,
                    CompilerQuoteBinding::Static(binding) => self
                        .source_insertion_binding_in_scope(
                            binding.clone(),
                            location,
                            &mut nodes,
                            &mut bytes,
                            capture_scope,
                        )?,
                    CompilerQuoteBinding::Native { slot, .. } => {
                        let value = snapshots.get(slot).expect("selected slot was materialized");
                        // Materialize one live slot once, but charge every
                        // retained source alias before its portable clone.
                        self.compiler_charge_constant_publication(
                            vm,
                            value,
                            &mut nodes,
                            &mut bytes,
                            location.span,
                        )?;
                        self.source_insertion_constant_in_scope(value, location, capture_scope)?
                    }
                };
                match binding {
                    SourceBinding::Graph(binding) => bindings.push((name, binding)),
                    SourceBinding::Value(value) => values.push((name, value)),
                }
            }
        }
        let policy = |mode: jai_ir::CheckMode| {
            if mode.enabled() {
                syntax::CheckPolicy::Inherited
            } else {
                syntax::CheckPolicy::Disabled
            }
        };
        Ok(DeclarationInsertionCode {
            file: source.file,
            source_file: source.source_file,
            location,
            items,
            bindings,
            values,
            checks: syntax::SafetyChecks {
                array_bounds: policy(source.checks.array_bounds),
                arithmetic_overflow: policy(source.checks.arithmetic_overflow),
            },
            debug: source.debug,
            origins: source.origins.clone(),
        })
    }
}
