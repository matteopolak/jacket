//! Checked source bodies and explicit typed compile-time capabilities.
use super::*;

/// Only fully bound bodies are Ready. Other source procedures remain dependencies.
pub struct ReadyProcedures<'a> {
    locals: Option<&'a crate::local_declarations::LocalDeclarationRegistry>,
    callback_proof: Option<&'a CallbackProof>,
    pub(super) storage_alignments: Option<&'a jai_ir::StorageAlignments>,
    pub(super) types: &'a dyn TypeView,
    pub(super) procedures: &'a HashMap<ProcedureId, Procedure>,
    pub(super) signatures: &'a HashMap<ProcedureId, TypeId>,
    pub(super) globals: &'a [Global],
    pub(super) places: &'a Places,
    pub(super) foreign: Option<&'a std::collections::HashSet<ProcedureId>>,
    pub(super) context: Option<&'a jai_ir::ContextDefinition>,
    pub(super) compiler: Option<&'a HashMap<ProcedureId, jai_vm::CompilerProcedure>>,
    pub(super) runtime: Option<&'a HashMap<ProcedureId, jai_vm::RuntimeProcedure>>,
    pub(super) file_abi: Option<&'a HashMap<ProcedureId, jai_vm::file_abi::FileAbiProcedure>>,
    pub(super) process_abi:
        Option<&'a HashMap<ProcedureId, jai_vm::process_abi::ProcessAbiProcedure>>,
    pub(super) heap_abi: Option<&'a HashMap<ProcedureId, jai_vm::heap_abi::HeapAbiProcedure>>,
    pub(super) pending_global_alignments: Option<&'a std::collections::HashSet<jai_ir::GlobalId>>,
    pub(super) generics: Option<&'a RefCell<crate::polymorphism::integration::GenericContext>>,
}
impl<'a> ReadyProcedures<'a> {
    pub fn new(
        types: &'a dyn TypeView,
        procedures: &'a HashMap<ProcedureId, Procedure>,
        signatures: &'a HashMap<ProcedureId, TypeId>,
        globals: &'a [Global],
        places: &'a Places,
    ) -> Result<Self, jai_ir::IrError> {
        Self::new_with_context(types, procedures, signatures, globals, places, None)
    }
    pub fn new_with_context(
        types: &'a dyn TypeView,
        procedures: &'a HashMap<ProcedureId, Procedure>,
        signatures: &'a HashMap<ProcedureId, TypeId>,
        globals: &'a [Global],
        places: &'a Places,
        context: Option<&'a jai_ir::ContextDefinition>,
    ) -> Result<Self, jai_ir::IrError> {
        for procedure in procedures.values() {
            match jai_ir::verify_procedure_with_context(
                types, procedure, signatures, globals, places, context,
            ) {
                Ok(_) | Err(jai_ir::IrError::Type(jai_types::TypeError::Incomplete(_))) => {}
                Err(error) => return Err(error),
            }
        }
        Ok(Self {
            locals: None,
            callback_proof: None,
            storage_alignments: None,
            types,
            procedures,
            signatures,
            globals,
            places,
            context,
            foreign: None,
            compiler: None,
            runtime: None,
            file_abi: None,
            heap_abi: None,
            process_abi: None,
            pending_global_alignments: None,
            generics: None,
        })
    }
    pub fn with_foreign(mut self, foreign: &'a std::collections::HashSet<ProcedureId>) -> Self {
        self.foreign = Some(foreign);
        self
    }
    pub fn with_pending_global_alignments(
        mut self,
        pending: &'a std::collections::HashSet<jai_ir::GlobalId>,
    ) -> Self {
        self.pending_global_alignments = Some(pending);
        self
    }
    pub fn with_storage_alignments(mut self, alignments: &'a jai_ir::StorageAlignments) -> Self {
        self.storage_alignments = Some(alignments);
        self
    }
    pub(crate) fn with_generic_readiness(
        mut self,
        generics: &'a RefCell<crate::polymorphism::integration::GenericContext>,
    ) -> Self {
        self.generics = Some(generics);
        self
    }
    pub(crate) fn with_local_readiness(
        mut self,
        locals: &'a crate::local_declarations::LocalDeclarationRegistry,
    ) -> Self {
        self.locals = Some(locals);
        self
    }
    pub(super) fn with_callback_proof(mut self, proof: &'a CallbackProof) -> Self {
        self.callback_proof = Some(proof);
        self
    }
    pub fn with_context(mut self, context: Option<&'a jai_ir::ContextDefinition>) -> Self {
        self.context = context;
        self
    }
    fn check_capability(&self, id: ProcedureId, signature: TypeId) -> Result<(), jai_ir::IrError> {
        if self
            .compiler
            .is_some_and(|bindings| bindings.contains_key(&id))
            || self
                .runtime
                .is_some_and(|bindings| bindings.contains_key(&id))
            || self
                .file_abi
                .is_some_and(|bindings| bindings.contains_key(&id))
            || self
                .heap_abi
                .is_some_and(|bindings| bindings.contains_key(&id))
            || self
                .process_abi
                .is_some_and(|bindings| bindings.contains_key(&id))
        {
            return Err(jai_ir::IrError::DuplicateIdentity {
                kind: "compile-time capability",
                index: id.index(),
            });
        }
        let expected = self
            .signatures
            .get(&id)
            .ok_or(jai_ir::IrError::UnknownIdentity {
                kind: "compile-time capability signature",
                index: id.index(),
            })?;
        if *expected != signature {
            return Err(jai_ir::IrError::TypeMismatch {
                expected: *expected,
                actual: signature,
            });
        }
        Ok(())
    }
    pub fn with_file_abi(
        mut self,
        bindings: &'a HashMap<ProcedureId, jai_vm::file_abi::FileAbiProcedure>,
    ) -> Result<Self, jai_ir::IrError> {
        for (id, procedure) in bindings {
            if procedure.procedure() != *id {
                return Err(jai_ir::IrError::UnknownIdentity {
                    kind: "file ABI procedure",
                    index: id.index(),
                });
            }
            self.check_capability(*id, procedure.signature)?;
        }
        self.file_abi = Some(bindings);
        Ok(self)
    }
    pub fn with_heap_abi(
        mut self,
        bindings: &'a HashMap<ProcedureId, jai_vm::heap_abi::HeapAbiProcedure>,
    ) -> Result<Self, jai_ir::IrError> {
        for (id, procedure) in bindings {
            if procedure.procedure() != *id {
                return Err(jai_ir::IrError::UnknownIdentity {
                    kind: "heap ABI procedure",
                    index: id.index(),
                });
            }
            self.check_capability(*id, procedure.signature)?;
        }
        self.heap_abi = Some(bindings);
        Ok(self)
    }
    pub fn with_process_abi(
        mut self,
        bindings: &'a HashMap<ProcedureId, jai_vm::process_abi::ProcessAbiProcedure>,
    ) -> Result<Self, jai_ir::IrError> {
        for (id, procedure) in bindings {
            if procedure.procedure() != *id {
                return Err(jai_ir::IrError::UnknownIdentity {
                    kind: "process ABI procedure",
                    index: id.index(),
                });
            }
            self.check_capability(*id, procedure.signature())?;
        }
        self.process_abi = Some(bindings);
        Ok(self)
    }
    pub fn with_runtime(
        mut self,
        runtime: &'a HashMap<ProcedureId, jai_vm::RuntimeProcedure>,
    ) -> Result<Self, jai_ir::IrError> {
        for (id, procedure) in runtime {
            let signature = self
                .signatures
                .get(id)
                .ok_or(jai_ir::IrError::UnknownIdentity {
                    kind: "runtime intrinsic",
                    index: id.index(),
                })?;
            if *signature != procedure.signature {
                return Err(jai_ir::IrError::TypeMismatch {
                    expected: *signature,
                    actual: procedure.signature,
                });
            }
            if self
                .compiler
                .is_some_and(|compiler| compiler.contains_key(id))
                || self
                    .file_abi
                    .is_some_and(|bindings| bindings.contains_key(id))
                || self
                    .heap_abi
                    .is_some_and(|bindings| bindings.contains_key(id))
                || self
                    .process_abi
                    .is_some_and(|bindings| bindings.contains_key(id))
            {
                return Err(jai_ir::IrError::DuplicateIdentity {
                    kind: "intrinsic binding",
                    index: id.index(),
                });
            }
        }
        self.runtime = Some(runtime);
        Ok(self)
    }
    pub fn with_compiler(
        mut self,
        compiler: &'a HashMap<ProcedureId, jai_vm::CompilerProcedure>,
    ) -> Result<Self, jai_ir::IrError> {
        for (id, procedure) in compiler {
            if self.runtime.is_some_and(|runtime| runtime.contains_key(id))
                || self
                    .file_abi
                    .is_some_and(|bindings| bindings.contains_key(id))
                || self
                    .heap_abi
                    .is_some_and(|bindings| bindings.contains_key(id))
                || self
                    .process_abi
                    .is_some_and(|bindings| bindings.contains_key(id))
            {
                return Err(jai_ir::IrError::DuplicateIdentity {
                    kind: "intrinsic binding",
                    index: id.index(),
                });
            }
            let signature = self
                .signatures
                .get(id)
                .ok_or(jai_ir::IrError::UnknownIdentity {
                    kind: "compiler procedure",
                    index: id.index(),
                })?;
            if *signature != procedure.signature {
                return Err(jai_ir::IrError::TypeMismatch {
                    expected: *signature,
                    actual: procedure.signature,
                });
            }
        }
        self.compiler = Some(compiler);
        Ok(self)
    }
}
impl ProcedureProvider for ReadyProcedures<'_> {
    fn context(&self) -> Option<&jai_ir::ContextDefinition> {
        self.context
    }
    fn signatures(&self) -> &HashMap<ProcedureId, TypeId> {
        self.signatures
    }
    fn types(&self) -> &dyn TypeView {
        self.types
    }
    fn procedure(&self, id: ProcedureId) -> ProcedureAvailability<'_> {
        if let Some(proof) = self.callback_proof
            && let Err(error) = proof.record(id)
        {
            return ProcedureAvailability::Failed(error);
        }
        if self
            .locals
            .is_some_and(|locals| locals.callback_body_ready(id) == Some(false))
        {
            return ProcedureAvailability::Pending(Dependency::Procedure(id));
        }
        if self
            .generics
            .is_some_and(|generics| generics.borrow().callback_body_ready(id) == Some(false))
        {
            return ProcedureAvailability::Pending(Dependency::Procedure(id));
        }
        if let Some(procedure) = self.compiler.and_then(|compiler| compiler.get(&id)) {
            return ProcedureAvailability::Compiler(*procedure);
        }
        if let Some(procedure) = self.runtime.and_then(|runtime| runtime.get(&id)) {
            return ProcedureAvailability::Runtime(*procedure);
        }
        if let Some(procedure) = self.file_abi.and_then(|bindings| bindings.get(&id)) {
            return ProcedureAvailability::FileAbi(*procedure);
        }
        if let Some(procedure) = self.heap_abi.and_then(|bindings| bindings.get(&id)) {
            return ProcedureAvailability::HeapAbi(*procedure);
        }
        if let Some(procedure) = self.process_abi.and_then(|bindings| bindings.get(&id)) {
            return ProcedureAvailability::ProcessAbi(procedure.clone());
        }
        if let Some(procedure) = self.procedures.get(&id) {
            match jai_ir::verify_procedure_with_context(
                self.types,
                procedure,
                self.signatures,
                self.globals,
                self.places,
                self.context,
            ) {
                Ok(checked) => ProcedureAvailability::Ready(checked),
                Err(jai_ir::IrError::Type(jai_types::TypeError::Incomplete(ty))) => {
                    ProcedureAvailability::Pending(Dependency::Type(ty))
                }
                Err(_) => ProcedureAvailability::Failed(jai_vm::Error::InvalidIr(
                    "ready procedure failed verification",
                )),
            }
        } else if self.foreign.is_some_and(|foreign| foreign.contains(&id)) {
            ProcedureAvailability::Foreign
        } else if self.signatures.contains_key(&id) {
            ProcedureAvailability::Pending(Dependency::Procedure(id))
        } else {
            ProcedureAvailability::Missing
        }
    }
    fn globals(&self) -> &[Global] {
        self.globals
    }
    fn global_alignment_pending(&self, id: jai_ir::GlobalId) -> bool {
        self.pending_global_alignments
            .is_some_and(|pending| pending.contains(&id))
    }
    fn storage_alignments(&self) -> Option<&jai_ir::StorageAlignments> {
        self.storage_alignments
    }
    fn places(&self) -> Option<&Places> {
        Some(self.places)
    }
}
