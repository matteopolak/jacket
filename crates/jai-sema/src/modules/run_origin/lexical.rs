//! Encode checked capture pools with source identities and structural types.
use super::*;
use crate::metaprogram::{RunBindingFact, RunCaptureFacts, RunMacroFact};
use jai_source::{DeclarationId, ModuleId, SourceSpan, Symbol};

const MAX_REPLAY_BYTES: usize = 8 * 1024 * 1024;

impl Encoder<'_, '_> {
    pub(super) fn lexical_facts(&mut self) -> Result<(), jai_vm::Error> {
        self.token(b"lexical-captures-v1");
        let facts = self.lexical;
        self.capture(&facts.current)?;
        self.count(facts.captures.len());
        for capture in &facts.captures {
            self.capture(capture)?;
        }
        self.count(facts.active_codes.len());
        for &capture in &facts.active_codes {
            self.capture_reference(capture)?;
        }
        self.count(facts.active_macros.len());
        for &macro_fact in &facts.active_macros {
            self.macro_fact(macro_fact)?;
        }
        self.token(b"active-callers");
        self.count(facts.active_callers.len());
        for &capture in &facts.active_callers {
            self.capture_reference(capture)?;
        }
        self.check_replay_size()
    }

    pub(super) fn specialized_type(&mut self, ty: TypeId) -> Result<bool, jai_vm::Error> {
        let records = &self.meta.record_specializations;
        let Some(record) = records.record(ty) else {
            return Ok(false);
        };
        let file = record.file;
        let origin = record.origin;
        let location = records.source_location(ty);
        let substitution = record.substitution.clone();
        self.token(b"specialized-nominal");
        self.file_origin(file)?;
        if let Some(origin) = origin {
            self.token(b"template");
            self.declaration(origin.0)?;
        } else {
            self.token(b"anonymous");
        }
        let location = location.ok_or(jai_vm::Error::InvalidIr(
            "specialized nominal has no retained source location",
        ))?;
        self.source_span(location)?;
        self.substitution(&substitution)?;
        Ok(true)
    }

    fn count(&mut self, value: usize) {
        self.token(&(value as u64).to_le_bytes());
    }

    fn check_replay_size(&self) -> Result<(), jai_vm::Error> {
        if self.bytes.len() > MAX_REPLAY_BYTES {
            return Err(jai_vm::Error::Limit(jai_vm::LimitKind::ValueCells));
        }
        Ok(())
    }

    fn symbol(&mut self, name: Symbol) {
        self.token(
            self.scope
                .declarations
                .graph
                .symbols()
                .name(name)
                .as_bytes(),
        );
    }

    fn file_origin(&mut self, file: FileInstanceId) -> Result<(), jai_vm::Error> {
        let graph = self.scope.declarations.graph;
        let source = graph
            .file(file)
            .and_then(|file| graph.sources().get(file.source()))
            .ok_or(jai_vm::Error::InvalidIr(
                "capture defining source is absent",
            ))?;
        let environment = graph
            .module_environment_origin(file)
            .map_err(|_| jai_vm::Error::InvalidIr("capture source environment is not ready"))?;
        self.token(source.path().as_os_str().as_encoded_bytes());
        self.token(&environment);
        Ok(())
    }

    pub(super) fn source_span(&mut self, location: SourceSpan) -> Result<(), jai_vm::Error> {
        let source = self
            .scope
            .declarations
            .graph
            .sources()
            .get(location.source)
            .ok_or(jai_vm::Error::InvalidIr(
                "capture source span belongs to another graph",
            ))?;
        let body = source
            .text()
            .as_bytes()
            .get(location.span.start..location.span.end)
            .ok_or(jai_vm::Error::InvalidIr("capture source span is invalid"))?;
        if body.len() > MAX_REPLAY_BYTES {
            return Err(jai_vm::Error::Limit(jai_vm::LimitKind::ValueCells));
        }
        self.token(source.path().as_os_str().as_encoded_bytes());
        self.count(location.span.start);
        self.count(location.span.end);
        self.token(body);
        self.check_replay_size()
    }

    fn capture(&mut self, capture: &RunCaptureFacts) -> Result<(), jai_vm::Error> {
        self.file_origin(capture.file)?;
        if self
            .scope
            .declarations
            .graph
            .file(capture.source_file)
            .is_none_or(|file| file.source() != capture.location.source)
        {
            return Err(jai_vm::Error::InvalidIr(
                "capture source instance does not own its source extent",
            ));
        }
        self.token(b"source-instance");
        self.file_origin(capture.source_file)?;
        self.source_span(capture.location)?;
        self.token(&[
            u8::from(capture.checks.array_bounds.enabled()),
            u8::from(capture.checks.arithmetic_overflow.enabled()),
            u8::from(capture.debug.emits()),
        ]);
        match &capture.substitution {
            Some(substitution) => {
                self.token(b"substitution");
                self.substitution(substitution)?;
            }
            None => self.token(b"no-substitution"),
        }
        self.count(capture.creation_origins.len());
        for &origin in &capture.creation_origins {
            self.source_span(origin)?;
        }
        self.count(capture.bindings.len());
        for frame in &capture.bindings {
            self.count(frame.len());
            let mut ordered = frame.iter().collect::<Vec<_>>();
            ordered.sort_by_key(|(name, _)| self.scope.declarations.graph.symbols().name(*name));
            for (name, binding) in ordered {
                self.symbol(*name);
                self.binding_fact(binding)?;
                self.check_replay_size()?;
            }
        }
        self.check_replay_size()
    }

    fn capture_reference(&mut self, index: usize) -> Result<(), jai_vm::Error> {
        if index >= self.lexical.captures.len() {
            return Err(jai_vm::Error::InvalidIr(
                "capture pool reference is invalid",
            ));
        }
        self.token(b"capture-reference");
        self.count(index);
        Ok(())
    }

    fn macro_fact(&mut self, source: RunMacroFact) -> Result<(), jai_vm::Error> {
        match source {
            RunMacroFact::Module(declaration) => {
                self.token(b"module-macro");
                self.declaration(declaration)
            }
            RunMacroFact::Local(capture) => {
                self.token(b"local-macro");
                self.capture_reference(capture)
            }
        }
    }

    pub(super) fn binding_fact(&mut self, binding: &RunBindingFact) -> Result<(), jai_vm::Error> {
        match binding {
            RunBindingFact::Discarded(ty) => {
                self.token(b"discarded");
                self.ty(*ty)?;
            }
            RunBindingFact::Type(ty) => {
                self.token(b"type");
                self.ty(*ty)?;
            }
            RunBindingFact::Constant(value) => {
                self.token(b"constant");
                self.constant(value)?;
            }
            RunBindingFact::WeakInteger(value) => {
                self.token(b"weak-integer");
                self.token(&value.to_le_bytes());
            }
            RunBindingFact::WeakFloat(value) => {
                self.token(b"weak-float");
                let bytes = value
                    .canonical_bytes(4096, MAX_REPLAY_BYTES)
                    .map_err(|_| jai_vm::Error::Limit(jai_vm::LimitKind::ValueCells))?;
                self.token(&bytes);
            }
            RunBindingFact::Code(index) => {
                self.token(b"code");
                self.capture_reference(*index)?;
            }
            RunBindingFact::NullCode => self.token(b"null-code"),
            RunBindingFact::Macro(source) => self.macro_fact(*source)?,
            RunBindingFact::Procedure { procedure, ty } => {
                self.token(b"procedure");
                self.ty(*ty)?;
                self.procedure_source(*procedure)?;
            }
            RunBindingFact::Namespace(module) => {
                self.token(b"namespace");
                self.module(*module)?;
            }
            RunBindingFact::Imported(binding) => {
                self.token(b"imported");
                self.imported(*binding)?;
            }
            RunBindingFact::Library(id) => {
                self.token(b"library");
                match id {
                    jai_ir::ForeignLibraryId::File(declaration) => {
                        self.declaration(*declaration)?
                    }
                    jai_ir::ForeignLibraryId::Local { procedure, index } => {
                        self.procedure_source(*procedure)?;
                        // The ordinal is assigned within this actual source procedure.
                        self.count(*index);
                    }
                }
            }
        }
        self.check_replay_size()
    }

    pub(super) fn declaration(&mut self, id: DeclarationId) -> Result<(), jai_vm::Error> {
        let declaration =
            self.scope
                .declarations
                .graph
                .declaration(id)
                .ok_or(jai_vm::Error::InvalidIr(
                    "capture declaration belongs to another graph",
                ))?;
        self.file_origin(declaration.file())?;
        self.symbol(declaration.name());
        self.source_span(declaration.location())
    }

    fn module(&mut self, id: ModuleId) -> Result<(), jai_vm::Error> {
        let entry = self
            .scope
            .declarations
            .graph
            .module(id)
            .and_then(|module| module.discovered_entry())
            .ok_or(jai_vm::Error::InvalidIr(
                "capture module source is not ready",
            ))?;
        self.file_origin(entry)
    }

    fn imported(&mut self, binding: jai_modules::Binding) -> Result<(), jai_vm::Error> {
        match binding {
            jai_modules::Binding::Declaration(id) => {
                self.token(b"declaration");
                self.declaration(id)
            }
            jai_modules::Binding::OverloadSet(id) => {
                self.token(b"overloads");
                let graph = self.scope.declarations.graph;
                let overloads = graph
                    .overload_set(id)
                    .ok_or(jai_vm::Error::InvalidIr("capture overload set is absent"))?;
                let mut declarations = overloads.declarations().to_vec();
                declarations.sort_by_key(|id| {
                    graph.declaration(*id).map(|decl| {
                        (
                            graph
                                .sources()
                                .get(decl.location().source)
                                .map(|source| source.path()),
                            decl.location().span.start,
                            decl.location().span.end,
                            graph.symbols().name(decl.name()),
                        )
                    })
                });
                self.count(declarations.len());
                for declaration in declarations {
                    self.declaration(declaration)?;
                }
                Ok(())
            }
            jai_modules::Binding::Module(id) => {
                self.token(b"module");
                self.module(id)
            }
            jai_modules::Binding::Parameter(id) => {
                self.token(b"parameter");
                let parameter =
                    self.scope
                        .declarations
                        .graph
                        .parameter(id)
                        .ok_or(jai_vm::Error::InvalidIr(
                            "capture module parameter is absent",
                        ))?;
                self.module(parameter.module)?;
                self.symbol(parameter.name);
                self.source_span(parameter.location)
            }
            jai_modules::Binding::StorageMember(id) => {
                self.token(b"storage-member");
                let member = self
                    .scope
                    .declarations
                    .graph
                    .source_storage_member(id)
                    .ok_or(jai_vm::Error::InvalidIr("capture storage member is absent"))?;
                self.declaration(member.owner())?;
                self.count(member.path().len());
                for &name in member.path() {
                    self.symbol(name);
                }
                self.source_span(member.location())
            }
            jai_modules::Binding::SourceMember {
                declaration,
                member,
            } => {
                self.token(b"source-member");
                self.declaration(declaration)?;
                self.symbol(member);
                Ok(())
            }
        }
    }

    pub(super) fn procedure_source(
        &mut self,
        procedure: jai_ir::ProcedureId,
    ) -> Result<(), jai_vm::Error> {
        if let Some(origin) = self.local.stable_procedure_origin(procedure) {
            self.token(b"local-procedure");
            self.token(origin);
            self.origin_environments(self.local.procedure_origin_files(procedure))?;
            if let Some(substitution) = self.local.procedure_origin_substitution(procedure).cloned()
            {
                self.substitution(&substitution)?;
            }
            return Ok(());
        }
        if let Some(id) = self
            .scope
            .declarations
            .signatures
            .iter()
            .find_map(|(id, signature)| (signature.id == procedure).then_some(*id))
        {
            self.token(b"source-procedure");
            return self.declaration(id);
        }
        let source = {
            let generics = self.scope.declarations.generics.borrow();
            generics
                .callback_source_origin(procedure)
                .map(|(declaration, _)| (declaration, generics.callback_substitution(procedure)))
        };
        let Some((declaration, substitution)) = source else {
            return Err(jai_vm::Error::InvalidIr(
                "capture procedure has no stable source origin",
            ));
        };
        self.token(b"specialized-procedure");
        self.declaration(declaration)?;
        if let Some(substitution) = substitution {
            self.substitution(&substitution)?;
        }
        Ok(())
    }
}
