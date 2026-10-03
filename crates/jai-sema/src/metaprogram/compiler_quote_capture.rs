//! Native compiler slots become source facts while the selected frame is live.
use super::compiler_quotes::{CompilerQuoteBinding, CompilerQuoteTemplate};
use super::*;
use jai_ir::ConstantValue;
use jai_vm::{CompilerCodeSelection, CompilerEffects, Limits, ProcedureProvider, Vm};

#[derive(Clone)]
pub(crate) struct PublishedCompilerQuote {
    body: syntax::CodeBody,
    frames: Vec<Vec<(Symbol, PublishedBinding)>>,
    key: CaptureKey,
    scope: CapturedScope,
}

#[derive(Clone)]
enum PublishedBinding {
    Static(Binding),
    Native(ConstantValue),
}

impl Resolver<'_> {
    pub(crate) fn selected_compiler_quote<P, E>(
        &self,
        template: &CompilerQuoteTemplate,
        lexical: &crate::local_declarations::LocalScopes,
        vm: &Vm<'_, P, E>,
        selected: &CompilerCodeSelection<'_>,
        limits: Limits,
    ) -> Result<PublishedCompilerQuote, jai_vm::Error>
    where
        P: ProcedureProvider + ?Sized,
        E: CompilerEffects,
    {
        if selected.site() != template.site()
            || selected.frame().plan() != template.plan()
            || selected.captures() != template.captures()
        {
            return Err(jai_vm::Error::InvalidIr(
                "compiler quotation belongs to another selected plan or frame",
            ));
        }
        let limits = self.compiler_quote_publication_limits(template, vm, limits)?;
        let inputs = selected
            .captures()
            .iter()
            .map(|&slot| {
                let (ty, value) = selected.native_value(slot)?;
                Ok((slot, ty, value))
            })
            .collect::<Result<Vec<_>, jai_vm::Error>>()?;
        let snapshots: HashMap<_, _> = crate::compile_time::materialize_compiler_captures(
            vm,
            self.types,
            &inputs,
            limits,
            selected.publication_occurrence(),
        )?
        .into_iter()
        .collect();
        let mut frames = Vec::with_capacity(template.frames().len());
        for frame in template.frames() {
            let mut published = Vec::with_capacity(frame.len());
            for (name, binding) in frame {
                let binding = match binding {
                    CompilerQuoteBinding::Static(binding) => {
                        PublishedBinding::Static(binding.clone())
                    }
                    CompilerQuoteBinding::Lexical(storage) => {
                        PublishedBinding::Static(Binding::Storage(*storage))
                    }
                    CompilerQuoteBinding::Native {
                        slot,
                        ty,
                    } => {
                        let value = snapshots.get(slot).ok_or(jai_vm::Error::InvalidIr(
                            "selected compiler quotation capture is absent",
                        ))?;
                        if value.ty != *ty {
                            return Err(jai_vm::Error::InvalidIr(
                                "selected compiler quotation capture type changed",
                            ));
                        }
                        self.compiler_charge_constant_publication(
                            vm,
                            value,
                            &mut 0,
                            &mut 0,
                            template.source().location.span,
                        )?;
                        PublishedBinding::Native(value.clone())
                    }
                };
                published.push((*name, binding));
            }
            frames.push(published);
        }
        let source = template.source();
        let source_scope = self
            .graph_scope
            .ok_or(jai_vm::Error::InvalidIr(
                "compiler quotation lost its source scope",
            ))?
            .in_file(source.file);
        let (file, file_scope, _) = source_scope.code_origin();
        if file != source.file {
            return Err(jai_vm::Error::InvalidIr(
                "compiler quotation changed its source owner",
            ));
        }
        // Inserted syntax keeps its original SourceId even though its bindings
        // live in the destination file's genuine scope.
        if !source_scope.has_compiler_quote_source(source.source_file, source.location) {
            return Err(jai_vm::Error::InvalidIr(
                "compiler quotation changed its retained source file",
            ));
        }
        template
            .admit_publication_body()
            .and_then(|()| template.admit_publication_scope(lexical, source_scope.substitution))
            .map_err(|error| jai_vm::Error::IrValidation(error.message))?;
        let substitution = source_scope.substitution.cloned();
        let mut nodes = frames.len();
        let mut bytes = 0usize;
        let mut binding_keys = Vec::with_capacity(frames.len());
        for frame in &frames {
            let mut keys = Vec::with_capacity(frame.len());
            for (name, binding) in frame {
                let key = match binding {
                    PublishedBinding::Native(value) => {
                        self.compiler_charge_constant_publication(
                            vm,
                            value,
                            &mut nodes,
                            &mut bytes,
                            source.location.span,
                        )?;
                        CaptureBinding::Constant(value.clone())
                    }
                    PublishedBinding::Static(binding) => self.compiler_static_binding_key(
                        vm,
                        binding,
                        &mut nodes,
                        &mut bytes,
                        source.location.span,
                    )?,
                };
                lexical_keys::charge(
                    &mut nodes,
                    1,
                    lexical_keys::MAX_LEXICAL_NODES,
                    source.location.span,
                )
                .map_err(|error| jai_vm::Error::IrValidation(error.message))?;
                keys.push((*name, key));
            }
            binding_keys.push(keys);
        }
        let key = CaptureKey {
            checks: source.checks,
            debug: source.debug,
            file: source.file,
            source_file: source.source_file,
            procedure: self.procedure,
            source: source.location.source,
            start: source.location.span.start,
            end: source.location.span.end,
            bindings: binding_keys,
            substitution: substitution.clone(),
            lexical_scopes: lexical.capture_identity(),
            expansion_origins: source
                .origins
                .iter()
                .map(|origin| (origin.source, origin.span.start, origin.span.end))
                .collect(),
        };
        let scope = CapturedScope {
            checks: source.checks,
            debug: source.debug,
            file: source.file,
            source_file: source.source_file,
            file_scope,
            procedure: self.procedure,
            location: source.location,
            substitution,
            expansion_origins: source.origins.clone(),
            frames: vec![],
            local_scopes: lexical.clone(),
        };
        Ok(PublishedCompilerQuote {
            body: template.body().clone(),
            frames,
            key,
            scope,
        })
    }

    pub(crate) fn compiler_quote_publication_limits<P, E>(
        &self,
        template: &CompilerQuoteTemplate,
        vm: &Vm<'_, P, E>,
        mut limits: Limits,
    ) -> Result<Limits, jai_vm::Error>
    where
        P: ProcedureProvider + ?Sized,
        E: CompilerEffects,
    {
        let fail = || jai_vm::Error::Limit(jai_vm::LimitKind::ValueCells);
        let residual = vm
            .publication_value_cell_limit()
            .checked_sub(vm.publication_retained_cells()?)
            .ok_or_else(fail)?;
        let entries = template.frames().iter().try_fold(0usize, |used, frame| {
            used.checked_add(frame.len()).ok_or_else(fail)
        })?;
        // The slot input vector, snapshot table, published frames, capture key,
        // and final source binding maps coexist. These metadata units bound
        // their headers, bucket slack, and symbol/type/slot fields.
        let metadata = entries
            .checked_mul(32)
            .and_then(|cells| cells.checked_add(template.frames().len().checked_mul(16)?))
            .ok_or_else(fail)?;
        let mut remaining = residual.checked_sub(metadata).ok_or_else(fail)?;
        vm.charge_publication_work(entries)?;
        let mut aliases = HashMap::new();
        for (_, binding) in template.frames().iter().flat_map(|frame| frame.iter()) {
            if let CompilerQuoteBinding::Native {
                slot, ..
            } = binding
            {
                let count = aliases.entry(*slot).or_insert(0usize);
                *count = count.checked_add(1).ok_or_else(fail)?;
            } else if let CompilerQuoteBinding::Static(Binding::TypedConstant(id)) = binding {
                let value = self.meta.constant(*id).ok_or(jai_vm::Error::InvalidIr(
                    "compiler constant capture belongs to another source arena",
                ))?;
                let mut nodes = 0;
                let mut bytes = 0;
                self.compiler_charge_constant_publication(
                    vm,
                    value,
                    &mut nodes,
                    &mut bytes,
                    template.source().location.span,
                )?;
                remaining = remaining
                    .checked_sub(nodes.checked_add(bytes).ok_or_else(fail)?)
                    .ok_or_else(fail)?;
            }
        }
        let mut copies = 1usize;
        for aliases in aliases.into_values() {
            // One snapshot plus one published fact and one key fact for each
            // alias. The batch converter also reserves two conversion forests.
            copies = copies.max(
                aliases
                    .checked_mul(2)
                    .and_then(|n| n.checked_add(1))
                    .ok_or_else(fail)?,
            );
        }
        limits.value_cells = limits.value_cells.min(remaining / copies);
        Ok(limits)
    }

    /// Inspection itself shares the selected VM's work meter. The bounded
    /// call stack avoids first allocating a pending vector for a large tree.
    pub(crate) fn compiler_charge_constant_publication<P, E>(
        &self,
        vm: &Vm<'_, P, E>,
        value: &ConstantValue,
        nodes: &mut usize,
        bytes: &mut usize,
        span: Span,
    ) -> Result<(), jai_vm::Error>
    where
        P: ProcedureProvider + ?Sized,
        E: CompilerEffects,
    {
        fn inspect<P, E>(
            vm: &Vm<'_, P, E>,
            value: &ConstantValue,
            nodes: &mut usize,
            bytes: &mut usize,
            span: Span,
            depth: usize,
        ) -> Result<(), jai_vm::Error>
        where
            P: ProcedureProvider + ?Sized,
            E: CompilerEffects,
        {
            if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
                return Err(jai_vm::Error::Limit(jai_vm::LimitKind::EvaluationDepth));
            }
            // Admit the inspection, clone, and eventual capture-key hash and
            // equality work before looking at each retained node.
            vm.charge_publication_work(5)?;
            lexical_keys::charge(nodes, 1, lexical_keys::MAX_LEXICAL_NODES, span)
                .map_err(|error| jai_vm::Error::IrValidation(error.message))?;
            match &value.kind {
                jai_ir::ConstantKind::Array(values) | jai_ir::ConstantKind::Record(values) => {
                    for value in values {
                        inspect(vm, value, nodes, bytes, span, depth + 1)?;
                    }
                }
                jai_ir::ConstantKind::Distinct(value)
                | jai_ir::ConstantKind::Union {
                    value, ..
                } => {
                    inspect(vm, value, nodes, bytes, span, depth + 1)?;
                }
                jai_ir::ConstantKind::StringBytes(value) => {
                    vm.charge_publication_work(
                        value
                            .len()
                            .checked_mul(4)
                            .ok_or(jai_vm::Error::Limit(jai_vm::LimitKind::ValueCells))?,
                    )?;
                    lexical_keys::charge(bytes, value.len(), lexical_keys::MAX_LEXICAL_BYTES, span)
                        .map_err(|error| jai_vm::Error::IrValidation(error.message))?;
                }
                _ => {}
            }
            Ok(())
        }
        inspect(vm, value, nodes, bytes, span, 0)
    }

    fn compiler_static_binding_key<P, E>(
        &self,
        vm: &Vm<'_, P, E>,
        binding: &Binding,
        nodes: &mut usize,
        bytes: &mut usize,
        span: Span,
    ) -> Result<CaptureBinding, jai_vm::Error>
    where
        P: ProcedureProvider + ?Sized,
        E: CompilerEffects,
    {
        let diagnostic = |error: Diagnostic| jai_vm::Error::IrValidation(error.message);
        Ok(match binding {
            Binding::CompilerInput {
                ..
            }
            | Binding::Discarded(_)
            | Binding::LambdaPreview(_) => {
                return Err(jai_vm::Error::InvalidIr(
                    "compiler quotation contains an unowned source fact",
                ));
            }
            Binding::Macro(id) => CaptureBinding::Macro(*id),
            Binding::Namespace(module) => CaptureBinding::Namespace(*module),
            Binding::Imported(binding) => match binding {
                jai_modules::Binding::Declaration(id) => CaptureBinding::ImportedDeclaration(*id),
                jai_modules::Binding::OverloadSet(id) => CaptureBinding::ImportedOverloadSet(*id),
                jai_modules::Binding::Module(id) => CaptureBinding::ImportedModule(*id),
                jai_modules::Binding::Parameter(id) => CaptureBinding::ImportedParameter(*id),
                jai_modules::Binding::SourceMember {
                    declaration,
                    member,
                } => CaptureBinding::ImportedSourceMember(*declaration, *member),
                jai_modules::Binding::StorageMember(id) => {
                    CaptureBinding::ImportedStorageMember(*id)
                }
            },
            Binding::Library(id) => CaptureBinding::Library(*id),
            Binding::Code(id) => {
                if self.meta.codes.get(*id).is_none() {
                    return Err(jai_vm::Error::InvalidIr(
                        "compiler Code capture belongs to another source arena",
                    ));
                }
                CaptureBinding::Code(*id)
            }
            Binding::TypedConstant(id) => {
                let value = self.meta.constant(*id).ok_or(jai_vm::Error::InvalidIr(
                    "compiler constant capture belongs to another source arena",
                ))?;
                self.compiler_charge_constant_publication(vm, value, nodes, bytes, span)?;
                CaptureBinding::Constant(value.clone())
            }
            Binding::Type(ty) => CaptureBinding::Type(*ty),
            Binding::Storage(storage) => {
                let key = storage_keys::StorageKey::new(self.places, storage.place(), span)
                    .map_err(diagnostic)?;
                lexical_keys::charge(nodes, key.nodes(), lexical_keys::MAX_LEXICAL_NODES, span)
                    .map_err(diagnostic)?;
                lexical_keys::charge(bytes, key.bytes(), lexical_keys::MAX_LEXICAL_BYTES, span)
                    .map_err(diagnostic)?;
                CaptureBinding::Storage(key)
            }
            Binding::Procedure {
                procedure,
                ty,
            } => CaptureBinding::Procedure(*procedure, *ty),
            Binding::Constant(crate::ScalarConstant::Literal(value)) => {
                CaptureBinding::WeakInteger(*value)
            }
            Binding::Constant(crate::ScalarConstant::Int(value)) => CaptureBinding::Scalar(
                self.types.scalar(crate::ScalarType::Int(value.ty())),
                value.bits(),
            ),
            Binding::Constant(crate::ScalarConstant::Bool(value)) => CaptureBinding::Scalar(
                self.types.scalar(crate::ScalarType::Bool),
                u64::from(*value),
            ),
            Binding::Constant(crate::ScalarConstant::Float(value)) => {
                CaptureBinding::Constant(ConstantValue {
                    ty: self.types.float(value.ty()),
                    kind: jai_ir::ConstantKind::Float(*value),
                })
            }
            Binding::Constant(crate::ScalarConstant::WeakFloat(value)) => {
                CaptureBinding::WeakFloat(value.request_key().clone())
            }
            Binding::Enum(value) => CaptureBinding::Enum(value.ty, value.value),
        })
    }

    /// Called only after the VM's sole transaction commit. This allocates a
    /// compiler Code identity, with no VM value or native procedure signature.
    pub(crate) fn retain_compiler_quote(&mut self, quote: PublishedCompilerQuote) -> CodeValueId {
        let mut scope = quote.scope;
        for frame in quote.frames {
            let mut bindings = HashMap::new();
            for (name, binding) in frame {
                let binding = match binding {
                    PublishedBinding::Static(binding) => binding,
                    PublishedBinding::Native(value) => match value.kind {
                        jai_ir::ConstantKind::Int(value) => {
                            Binding::Constant(crate::ScalarConstant::Int(value))
                        }
                        jai_ir::ConstantKind::Float(value) => {
                            Binding::Constant(crate::ScalarConstant::Float(value))
                        }
                        jai_ir::ConstantKind::Bool(value) => {
                            Binding::Constant(crate::ScalarConstant::Bool(value))
                        }
                        jai_ir::ConstantKind::Enum(integer) => {
                            Binding::Enum(crate::modules::aggregates::EnumConstant {
                                ty: value.ty,
                                value: integer,
                            })
                        }
                        jai_ir::ConstantKind::RuntimeType(value) => {
                            Binding::Type(value.identity().ty())
                        }
                        kind => Binding::TypedConstant(self.meta.intern_constant(ConstantValue {
                            ty: value.ty,
                            kind,
                        })),
                    },
                };
                bindings.insert(name, binding);
            }
            scope.frames.push(bindings);
        }
        self.meta.codes.capture(quote.key, quote.body, scope)
    }
}
