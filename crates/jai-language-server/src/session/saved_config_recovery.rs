//! One accounted native settings transaction advanced between client requests.
use super::{ConfigCapture, ConfigFailure, SavedConfigState};
use crate::{DocumentUri, semantic, session::Session};
use jaic::sema::FileSystem;
use std::{
    cell::RefCell,
    path::Path,
    rc::Rc,
    time::{Duration, Instant},
};

pub(crate) struct ConfigRecovery {
    pub(super) uri: DocumentUri,
    owner: Rc<()>,
    parent_revision: u64,
    fs: Rc<semantic::input_epoch::InputEpochFs>,
    proof: Rc<RefCell<semantic::Observations>>,
}

impl ConfigRecovery {
    fn new(session: &Session, uri: DocumentUri) -> Option<Self> {
        let parent = session.input_epoch.as_ref()?;
        let owner = session.saved_configs.get(&uri)?.owner.clone();
        let proof = Rc::new(RefCell::new(semantic::Observations::default()));
        let fs = Rc::new(parent.detached_recovery(&proof));
        if !fs.seal_after_validation() {
            return None;
        }
        // Unknown lookups in this sealed empty carrier only queue work: setup performs no IO.
        let path = Path::new(uri.path());
        let _ = fs.canonical(path);
        let _ = fs.read_shared(path);
        let _ = fs.missing_file_key(path);
        Some(Self {
            uri,
            owner,
            parent_revision: parent.event_revision(),
            fs,
            proof,
        })
    }

    pub(super) fn storage(&self) -> usize {
        std::mem::size_of::<Self>() + self.uri.as_str().len() + self.uri.path().len()
            + self.fs.retained_bytes() + self.proof.borrow().retained_bytes()
            // Reserve the future String before any other document can spend its space.
            + self.fs.captured_read_bytes(Path::new(self.uri.path()))
    }

    fn selected(&self, session: &Session) -> bool {
        session
            .input_epoch
            .as_ref()
            .is_some_and(|epoch| epoch.event_revision() == self.parent_revision)
            && session.saved_configs.get(&self.uri).is_some_and(|saved| {
                Rc::ptr_eq(&saved.owner, &self.owner)
                    && matches!(
                        saved.state,
                        SavedConfigState::Unavailable {
                            reason: ConfigFailure::Input,
                            ..
                        }
                    )
            })
    }
}

impl Session {
    pub(crate) fn has_config_recovery_work(&self) -> bool {
        self.config_recovery.borrow().is_some()
            || self
                .saved_configs
                .iter()
                .any(|(uri, saved)| match saved.state {
                    SavedConfigState::Unavailable {
                        reason: ConfigFailure::Input,
                        ..
                    } => true,
                    SavedConfigState::Unavailable {
                        reason:
                            ConfigFailure::Budget {
                                required,
                            },
                        ..
                    } => self.admit_saved_config_size(uri, required, 0).is_ok(),
                    _ => false,
                })
    }

    /// Service at most one native operation; unfinished candidates retain their original proof.
    /// Returns `None` when ordinary epoch reconciliation should take this turn instead.
    pub(crate) fn service_config_recovery(&mut self, now: Instant) -> Option<bool> {
        if self.input_epoch.is_none() || jaic::interrupt::fired() {
            return None;
        }
        let current = self.config_recovery.borrow_mut().take();
        let candidate = if let Some(candidate) = current {
            if !candidate.selected(self) {
                return Some(false);
            }
            candidate
        } else {
            let cursor = self.config_recovery_cursor.get();
            let next = self
                .saved_configs
                .iter()
                .enumerate()
                .skip(cursor)
                .chain(self.saved_configs.iter().enumerate().take(cursor))
                .find(|(_, (uri, saved))| match saved.state {
                    SavedConfigState::Unavailable {
                        reason: ConfigFailure::Input,
                        recheck_at,
                    } => recheck_at <= now,
                    SavedConfigState::Unavailable {
                        reason:
                            ConfigFailure::Budget {
                                required,
                            },
                        ..
                    } => self.admit_saved_config_size(uri, required, 0).is_ok(),
                    _ => false,
                })
                .map(|(index, (uri, _))| (index, uri.clone()))?;
            self.config_recovery_cursor
                .set((next.0 + 1) % self.saved_configs.len());
            let next = next.1;
            if let Some(saved) = self.saved_configs.get_mut(&next) {
                saved.state = SavedConfigState::Unavailable {
                    reason: ConfigFailure::Input,
                    recheck_at: now,
                };
            }
            let candidate = ConfigRecovery::new(self, next)?;
            let needed = candidate.storage();
            if self.admit_config_recovery(needed).is_err() {
                self.unavailable_saved_config(
                    &candidate.uri,
                    ConfigFailure::Budget {
                        required: needed.saturating_add(
                            self.saved_configs
                                .get(&candidate.uri)
                                .map_or(0, |saved| saved.storage(&candidate.uri)),
                        ),
                    },
                );
                return Some(false);
            }
            // Setup has no IO; leave the first capture operation to the next service turn.
            *self.config_recovery.borrow_mut() = Some(candidate);
            return Some(false);
        };
        let changed = candidate.fs.reconcile_step(1, 256 * 1024);
        if !candidate.selected(self) || jaic::interrupt::fired() {
            return Some(false);
        }
        let needed = candidate.storage();
        if self.admit_config_recovery(needed).is_err() {
            self.unavailable_saved_config(
                &candidate.uri,
                ConfigFailure::Budget {
                    required: needed.saturating_add(
                        self.saved_configs
                            .get(&candidate.uri)
                            .map_or(0, |saved| saved.storage(&candidate.uri)),
                    ),
                },
            );
            return Some(false);
        }
        if changed.is_some()
            || (!candidate.fs.has_pending_coverage()
                && candidate.fs.input_epoch_valid() != Some(true))
        {
            self.defer_config_recovery(&candidate.uri, now);
            return Some(false);
        }
        if candidate.fs.input_epoch_valid() != Some(true) {
            *self.config_recovery.borrow_mut() = Some(candidate);
            return Some(false);
        }
        let snapshot = self.capture_saved_config_from(
            &candidate.uri,
            ConfigCapture::Disk,
            Some(candidate.fs.clone()),
        );
        // The final local operation, owner selection and workspace check precede all mutation.
        if !candidate.selected(self) || jaic::interrupt::should_stop() {
            return Some(false);
        }
        let snapshot = if let Ok(mut snapshot) = snapshot {
            if matches!(snapshot.state, SavedConfigState::Missing)
                && let Some(previous) = self.saved_configs.get(&candidate.uri)
            {
                snapshot.identity.clone_from(&previous.identity);
            }
            let needed = snapshot.storage(&candidate.uri);
            if self
                .admit_saved_config(&candidate.uri, &snapshot, 0)
                .is_err()
            {
                self.unavailable_saved_config(
                    &candidate.uri,
                    ConfigFailure::Budget {
                        required: needed,
                    },
                );
                return Some(false);
            }
            snapshot
        } else {
            self.defer_config_recovery(&candidate.uri, now);
            return Some(false);
        };
        Some(self.accept_recovered_settings(candidate.uri, snapshot))
    }

    fn defer_config_recovery(&mut self, uri: &DocumentUri, now: Instant) {
        if let Some(saved) = self.saved_configs.get_mut(uri) {
            saved.state = SavedConfigState::Unavailable {
                reason: ConfigFailure::Input,
                recheck_at: now + Duration::from_millis(250),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_charges_both_owned_uri_strings() {
        let (_temp, _fs, session, uri) =
            super::super::tests::unavailable_native_config("recovery-uri-accounting");
        let candidate = ConfigRecovery::new(&session, uri).unwrap();
        let operation_storage =
            candidate.fs.retained_bytes() + candidate.proof.borrow().retained_bytes();
        let non_operation =
            candidate.storage() - operation_storage - std::mem::size_of::<ConfigRecovery>();
        assert!(
            non_operation >= candidate.uri.as_str().len() + candidate.uri.path().len(),
            "both independent URI strings are owned heap storage beyond the inline struct"
        );
    }
}
