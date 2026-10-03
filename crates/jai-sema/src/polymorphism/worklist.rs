use super::SpecializationKey;
use jai_ir::ProcedureId;
use jai_modules::FileInstanceId;
use jai_source::{Diagnostic, Span};
use jai_types::TypeId;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpecializationId(usize);
impl SpecializationId {
    pub fn index(self) -> usize {
        self.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecializationState {
    SignaturePending,
    ResolvingSignature,
    BodyPending,
    ResolvingBody,
    Ready,
    Failed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reservation {
    pub specialization: SpecializationId,
    pub procedure: ProcedureId,
    pub newly_reserved: bool,
}
#[derive(Debug)]
pub struct Specialization {
    pub key: SpecializationKey,
    pub defining_file: FileInstanceId,
    pub procedure: ProcedureId,
    state: SpecializationState,
    signature: Option<TypeId>,
    failure: Option<Diagnostic>,
}
impl Specialization {
    pub fn state(&self) -> SpecializationState {
        self.state
    }
    pub fn signature(&self) -> Option<TypeId> {
        self.signature
    }
}
#[derive(Debug)]
pub enum Readiness<'a> {
    SignaturePending(ProcedureId),
    BodyPending {
        procedure: ProcedureId,
        signature: TypeId,
    },
    Ready {
        procedure: ProcedureId,
        signature: TypeId,
    },
    Failed(&'a Diagnostic),
}

/// Owns reservation and scheduling; completed bodies live in the checked IR arena.
#[derive(Debug)]
pub struct Specializations {
    by_key: HashMap<SpecializationKey, SpecializationId>,
    entries: Vec<Specialization>,
    signatures: VecDeque<SpecializationId>,
    bodies: VecDeque<SpecializationId>,
    next_procedure: usize,
    callback_revision: usize,
    pending_callback_rechecks: HashSet<ProcedureId>,
    callback_changed_while_binding: HashSet<ProcedureId>,
}
impl Specializations {
    pub fn new(first_procedure: usize) -> Self {
        Self {
            by_key: HashMap::new(),
            entries: Vec::new(),
            signatures: VecDeque::new(),
            bodies: VecDeque::new(),
            next_procedure: first_procedure,
            callback_revision: 0,
            pending_callback_rechecks: HashSet::new(),
            callback_changed_while_binding: HashSet::new(),
        }
    }
    pub fn reserve(
        &mut self,
        key: SpecializationKey,
        defining_file: FileInstanceId,
    ) -> Result<Reservation, Diagnostic> {
        if let Some(&id) = self.by_key.get(&key) {
            let entry = &self.entries[id.0];
            if entry.defining_file != defining_file {
                return Err(Diagnostic::new(
                    Span::default(),
                    "specialization definition scope changed",
                ));
            }
            return Ok(Reservation {
                specialization: id,
                procedure: entry.procedure,
                newly_reserved: false,
            });
        }
        let procedure = self.reserve_procedure_identity()?;
        let id = SpecializationId(self.entries.len());
        self.by_key.insert(key.clone(), id);
        self.entries.push(Specialization {
            key,
            defining_file,
            procedure,
            state: SpecializationState::SignaturePending,
            signature: None,
            failure: None,
        });
        self.signatures.push_back(id);
        Ok(Reservation {
            specialization: id,
            procedure,
            newly_reserved: true,
        })
    }
    pub(crate) fn reserve_procedure_identity(&mut self) -> Result<ProcedureId, Diagnostic> {
        let procedure = ProcedureId::new(self.next_procedure);
        self.next_procedure = self.next_procedure.checked_add(1).ok_or_else(|| {
            Diagnostic::new(Span::default(), "procedure identity space exhausted")
        })?;
        Ok(procedure)
    }
    pub fn get(&self, id: SpecializationId) -> Option<&Specialization> {
        self.entries.get(id.0)
    }
    pub fn entries(&self) -> &[Specialization] {
        &self.entries
    }
    pub fn query(&self, id: SpecializationId) -> Option<Readiness<'_>> {
        let entry = self.entries.get(id.0)?;
        Some(match entry.state {
            SpecializationState::SignaturePending | SpecializationState::ResolvingSignature => {
                Readiness::SignaturePending(entry.procedure)
            }
            SpecializationState::BodyPending | SpecializationState::ResolvingBody => {
                Readiness::BodyPending {
                    procedure: entry.procedure,
                    signature: entry.signature.expect("body work has a ready signature"),
                }
            }
            SpecializationState::Ready => Readiness::Ready {
                procedure: entry.procedure,
                signature: entry.signature.expect("ready body has a ready signature"),
            },
            SpecializationState::Failed => Readiness::Failed(
                entry
                    .failure
                    .as_ref()
                    .expect("failed work stores its error"),
            ),
        })
    }
    pub fn next_signature(&mut self) -> Option<SpecializationId> {
        while let Some(id) = self.signatures.pop_front() {
            if self.entries[id.0].state == SpecializationState::SignaturePending {
                self.entries[id.0].state = SpecializationState::ResolvingSignature;
                return Some(id);
            }
        }
        None
    }
    pub fn publish_signature(
        &mut self,
        id: SpecializationId,
        signature: TypeId,
    ) -> Result<(), Diagnostic> {
        let entry = self.entry_mut(id)?;
        if !matches!(
            entry.state,
            SpecializationState::SignaturePending | SpecializationState::ResolvingSignature
        ) {
            return Err(Diagnostic::new(
                Span::default(),
                "specialization signature was already resolved",
            ));
        }
        entry.signature = Some(signature);
        entry.state = SpecializationState::BodyPending;
        self.bodies.push_back(id);
        Ok(())
    }
    pub fn next_body(&mut self) -> Option<SpecializationId> {
        while let Some(id) = self.bodies.pop_front() {
            if self.entries[id.0].state == SpecializationState::BodyPending {
                self.entries[id.0].state = SpecializationState::ResolvingBody;
                self.callback_changed_while_binding
                    .remove(&self.entries[id.0].procedure);
                return Some(id);
            }
        }
        None
    }
    /// A bodyless declaration publishes checked metadata without entering the
    /// body compiler's queue. Its signature has the same recursion identity.
    pub fn publish_prototype(
        &mut self,
        id: SpecializationId,
        signature: TypeId,
    ) -> Result<(), Diagnostic> {
        let entry = self.entry_mut(id)?;
        if !matches!(
            entry.state,
            SpecializationState::SignaturePending | SpecializationState::ResolvingSignature
        ) {
            return Err(Diagnostic::new(
                Span::default(),
                "specialization prototype was already resolved",
            ));
        }
        entry.signature = Some(signature);
        entry.state = SpecializationState::Ready;
        Ok(())
    }
    pub fn complete(&mut self, id: SpecializationId) -> Result<(), Diagnostic> {
        let entry = self.entry_mut(id)?;
        if entry.state != SpecializationState::ResolvingBody {
            return Err(Diagnostic::new(
                Span::default(),
                "specialization body was not being resolved",
            ));
        }
        let procedure = entry.procedure;
        if self.callback_changed_while_binding.remove(&procedure) {
            self.entries[id.0].state = SpecializationState::BodyPending;
            self.bodies.push_back(id);
        } else {
            self.entries[id.0].state = SpecializationState::Ready;
            self.pending_callback_rechecks.remove(&procedure);
        }
        Ok(())
    }
    /// Retry a body whose typed compile-time dependencies are not ready yet.
    pub fn retry_body(&mut self, id: SpecializationId) -> Result<(), Diagnostic> {
        let entry = self.entry_mut(id)?;
        if entry.state != SpecializationState::ResolvingBody {
            return Err(Diagnostic::new(
                Span::default(),
                "specialization body was not being resolved",
            ));
        }
        entry.state = SpecializationState::BodyPending;
        self.bodies.push_back(id);
        Ok(())
    }
    pub(crate) fn recheck_body(&mut self, procedure: ProcedureId) -> Result<bool, Diagnostic> {
        let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.procedure == procedure)
        else {
            return Ok(false);
        };
        let state = self.entries[index].state;
        if !matches!(
            state,
            SpecializationState::Ready | SpecializationState::ResolvingBody
        ) {
            return Ok(false);
        }
        if state == SpecializationState::ResolvingBody
            && self.callback_changed_while_binding.contains(&procedure)
        {
            return Ok(false);
        }
        self.callback_revision = self.callback_revision.checked_add(1).ok_or_else(|| {
            Diagnostic::new(Span::default(), "callback readiness revision exhausted")
        })?;
        self.pending_callback_rechecks.insert(procedure);
        if state == SpecializationState::ResolvingBody {
            self.callback_changed_while_binding.insert(procedure);
        } else {
            self.entries[index].state = SpecializationState::BodyPending;
            self.bodies.push_back(SpecializationId(index));
        }
        Ok(true)
    }
    pub(crate) fn callback_readiness_revision(&self) -> usize {
        self.callback_revision
    }
    pub(crate) fn callback_body_ready(&self, procedure: ProcedureId) -> Option<bool> {
        self.entries
            .iter()
            .find(|entry| entry.procedure == procedure)
            .map(|entry| entry.state == SpecializationState::Ready)
    }
    pub(crate) fn callback_body_failure(&self, procedure: ProcedureId) -> Option<&Diagnostic> {
        self.entries
            .iter()
            .find(|entry| {
                entry.procedure == procedure && entry.state == SpecializationState::Failed
            })
            .and_then(|entry| entry.failure.as_ref())
    }
    pub(crate) fn pending_callback_rechecks(&self) -> Vec<ProcedureId> {
        let mut procedures = self
            .pending_callback_rechecks
            .iter()
            .copied()
            .collect::<Vec<_>>();
        procedures.sort_by_key(|procedure| procedure.index());
        procedures
    }
    pub fn fail(&mut self, id: SpecializationId, error: Diagnostic) -> Result<(), Diagnostic> {
        let entry = self.entry_mut(id)?;
        if matches!(
            entry.state,
            SpecializationState::Ready | SpecializationState::Failed
        ) {
            return Err(Diagnostic::new(
                Span::default(),
                "specialization has already finished",
            ));
        }
        entry.failure = Some(error);
        entry.state = SpecializationState::Failed;
        let procedure = entry.procedure;
        // A failed recheck cannot validate a previously committed #run receipt.
        self.callback_changed_while_binding.remove(&procedure);
        Ok(())
    }
    fn entry_mut(&mut self, id: SpecializationId) -> Result<&mut Specialization, Diagnostic> {
        self.entries
            .get_mut(id.0)
            .ok_or_else(|| Diagnostic::new(Span::default(), "unknown specialization identity"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polymorphism::{Substitution, TypeBinding};
    use jai_modules::{GraphOptions, ModuleGraph};
    use jai_source::{Identities, Symbols};
    use jai_types::{IntegerType, ScalarType, TypeRegistry};
    use std::{
        fs,
        sync::atomic::{AtomicUsize, Ordering},
    };

    static NEXT: AtomicUsize = AtomicUsize::new(0);
    fn file() -> FileInstanceId {
        let path = std::env::temp_dir().join(format!(
            "jai-specialization-{}-{}.jai",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&path, "main :: () {}").unwrap();
        let graph = ModuleGraph::load(&path, GraphOptions::default()).unwrap();
        fs::remove_file(path).unwrap();
        graph.module(graph.root()).unwrap().entry()
    }
    fn key(types: &TypeRegistry) -> SpecializationKey {
        let mut names = Symbols::default();
        SpecializationKey {
            declaration: Identities::default().declaration(),
            substitution: Substitution {
                types: vec![TypeBinding {
                    name: names.intern("T"),
                    ty: types.scalar(ScalarType::Int(IntegerType::S64)),
                }],
                constants: vec![],
                callables: vec![],
            },
        }
    }
    #[test]
    fn recursive_request_keeps_reserved_identity_and_never_duplicates_body_work() {
        let types = TypeRegistry::new();
        let key = key(&types);
        let file = file();
        let mut work = Specializations::new(4);
        let first = work.reserve(key.clone(), file).unwrap();
        assert!(first.newly_reserved);
        assert_eq!(first.procedure.index(), 4);
        assert!(
            matches!(work.query(first.specialization), Some(Readiness::SignaturePending(id)) if id == first.procedure)
        );
        assert_eq!(work.next_signature(), Some(first.specialization));
        work.publish_signature(
            first.specialization,
            types.scalar(ScalarType::Int(IntegerType::S64)),
        )
        .unwrap();
        assert_eq!(work.next_body(), Some(first.specialization));
        let recursive = work.reserve(key, file).unwrap();
        assert!(!recursive.newly_reserved);
        assert_eq!(recursive.procedure, first.procedure);
        assert_eq!(work.entries().len(), 1);
        assert!(
            matches!(work.query(recursive.specialization), Some(Readiness::BodyPending { procedure, .. }) if procedure == first.procedure)
        );
        work.complete(first.specialization).unwrap();
        assert!(matches!(
            work.query(first.specialization),
            Some(Readiness::Ready { .. })
        ));
        assert_eq!(work.next_signature(), None);
        assert_eq!(work.next_body(), None);
    }
    #[test]
    fn failed_specialization_keeps_error_and_is_not_requeued() {
        let types = TypeRegistry::new();
        let key = key(&types);
        let file = file();
        let mut work = Specializations::new(0);
        let reservation = work.reserve(key.clone(), file).unwrap();
        let diagnostic = Diagnostic::new(Span::new(8, 12), "generic body is invalid for this type");
        work.fail(reservation.specialization, diagnostic.clone())
            .unwrap();
        assert!(
            matches!(work.query(reservation.specialization), Some(Readiness::Failed(error)) if error == &diagnostic)
        );
        assert!(!work.reserve(key, file).unwrap().newly_reserved);
        assert_eq!(work.next_signature(), None);
        assert_eq!(work.next_body(), None);
        assert!(
            work.publish_signature(reservation.specialization, types.scalar(ScalarType::Bool))
                .is_err()
        );
    }
    #[test]
    fn strengthened_callback_contract_rechecks_the_same_identity() {
        let types = TypeRegistry::new();
        let mut work = Specializations::new(7);
        let reservation = work.reserve(key(&types), file()).unwrap();
        work.publish_signature(reservation.specialization, types.scalar(ScalarType::Bool))
            .unwrap();
        assert_eq!(work.next_body(), Some(reservation.specialization));
        work.complete(reservation.specialization).unwrap();
        assert_eq!(work.callback_body_ready(reservation.procedure), Some(true));

        assert!(work.recheck_body(reservation.procedure).unwrap());
        assert_eq!(work.callback_readiness_revision(), 1);
        assert_eq!(work.callback_body_ready(reservation.procedure), Some(false));
        assert_eq!(work.pending_callback_rechecks(), [reservation.procedure]);
        assert!(!work.recheck_body(reservation.procedure).unwrap());
        assert_eq!(work.next_body(), Some(reservation.specialization));
        work.complete(reservation.specialization).unwrap();
        assert_eq!(work.entries().len(), 1);
        assert_eq!(work.callback_body_ready(reservation.procedure), Some(true));
        assert!(work.pending_callback_rechecks().is_empty());
        assert_eq!(work.next_body(), None);
    }
    #[test]
    fn callback_contract_changed_during_binding_requires_another_body_proof() {
        let types = TypeRegistry::new();
        let mut work = Specializations::new(0);
        let reservation = work.reserve(key(&types), file()).unwrap();
        work.publish_signature(reservation.specialization, types.scalar(ScalarType::Bool))
            .unwrap();
        assert_eq!(work.next_body(), Some(reservation.specialization));
        assert!(work.recheck_body(reservation.procedure).unwrap());
        assert!(!work.recheck_body(reservation.procedure).unwrap());
        assert_eq!(work.callback_body_failure(reservation.procedure), None);
        work.complete(reservation.specialization).unwrap();
        assert_eq!(work.callback_body_ready(reservation.procedure), Some(false));
        assert_eq!(work.next_body(), Some(reservation.specialization));
        let diagnostic = Diagnostic::new(Span::new(8, 12), "required callback result discarded");
        work.fail(reservation.specialization, diagnostic.clone())
            .unwrap();
        assert_eq!(work.pending_callback_rechecks(), [reservation.procedure]);
        assert_eq!(work.callback_body_ready(reservation.procedure), Some(false));
        assert_eq!(
            work.callback_body_failure(reservation.procedure),
            Some(&diagnostic)
        );
        assert!(
            matches!(work.query(reservation.specialization), Some(Readiness::Failed(error)) if error == &diagnostic)
        );
        assert_eq!(work.next_body(), None);
    }
    #[test]
    fn a_bodyless_specialization_publishes_readiness_without_body_work() {
        let types = TypeRegistry::new();
        let key = key(&types);
        let file = file();
        let mut work = Specializations::new(5);
        let reservation = work.reserve(key.clone(), file).unwrap();
        let signature = types.scalar(ScalarType::Bool);
        work.publish_prototype(reservation.specialization, signature)
            .unwrap();
        assert!(
            matches!(work.query(reservation.specialization), Some(Readiness::Ready { procedure, signature: ty }) if procedure == reservation.procedure && ty == signature)
        );
        assert_eq!(work.next_body(), None);
        assert_eq!(work.next_signature(), None);
        assert_eq!(
            work.reserve(key, file).unwrap().procedure,
            reservation.procedure
        );
        assert!(
            work.publish_signature(reservation.specialization, signature)
                .is_err()
        );
    }
    #[test]
    fn different_origin_and_types_get_distinct_specializations() {
        let types = TypeRegistry::new();
        let first_key = key(&types);
        let mut other_type = first_key.clone();
        other_type.substitution.types[0].ty = types.scalar(ScalarType::Int(IntegerType::U8));
        let mut origins = Identities::default();
        origins.declaration();
        let mut other_origin = first_key.clone();
        other_origin.declaration = origins.declaration();
        let file = file();
        let mut work = Specializations::new(1);
        let first = work.reserve(first_key, file).unwrap();
        let second = work.reserve(other_type, file).unwrap();
        let third = work.reserve(other_origin, file).unwrap();
        assert_eq!(
            [
                first.procedure.index(),
                second.procedure.index(),
                third.procedure.index()
            ],
            [1, 2, 3]
        );
        assert_eq!(work.entries().len(), 3);
    }
}
