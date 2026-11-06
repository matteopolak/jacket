//! Saved settings own their accepted identity independently from unfinished editor text.
use super::Session;
use crate::{DocumentUri, Error, semantic};
use jaic::sema::{FileSystem, NativeFs};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Instant;

#[derive(Clone, Copy)]
pub(crate) enum ConfigFailure {
    Input,
    Budget { required: usize },
}

pub(crate) enum SavedConfigState {
    Text(String),
    ClientText(String),
    Missing,
    Unavailable {
        reason: ConfigFailure,
        recheck_at: Instant,
    },
}

pub(crate) struct SavedConfig {
    pub(crate) identity: PathBuf,
    pub(crate) state: SavedConfigState,
    proof: semantic::Observations,
    pub(super) actual_inputs: Option<semantic::input_epoch::ActualInputs>,
    owner: Rc<()>,
}

impl SavedConfig {
    pub(crate) fn text(&self) -> Option<&str> {
        match &self.state {
            SavedConfigState::Text(text) | SavedConfigState::ClientText(text) => Some(text),
            SavedConfigState::Missing
            | SavedConfigState::Unavailable {
                ..
            } => None,
        }
    }

    pub(crate) fn unavailable(&self) -> bool {
        matches!(self.state, SavedConfigState::Unavailable { .. })
    }

    pub(super) fn storage(&self, uri: &DocumentUri) -> usize {
        uri.as_str().len()
            + uri.path().len()
            + self.identity.capacity()
            + self.text().map_or(0, |_| match &self.state {
                SavedConfigState::Text(text) | SavedConfigState::ClientText(text) => {
                    text.capacity()
                }
                _ => 0,
            })
            + self.proof.retained_bytes()
            + self
                .actual_inputs
                .as_ref()
                .map_or(0, semantic::input_epoch::ActualInputs::retained_bytes)
            + std::mem::size_of::<(DocumentUri, Self)>()
            + 96
    }
}

pub(super) enum ConfigCapture<'a> {
    Initial(&'a str),
    Buffer(&'a str),
    Disk,
}

impl Session {
    /// Capture identity and contents together, without sealing an enclosing semantic query.
    /// For example, a watcher refresh reads through the stored alias, not the event target.
    pub(super) fn capture_saved_config(
        &self,
        uri: &DocumentUri,
        source: ConfigCapture<'_>,
    ) -> Result<SavedConfig, Error> {
        self.capture_saved_config_from(uri, source, None)
    }

    fn capture_saved_config_from(
        &self,
        uri: &DocumentUri,
        source: ConfigCapture<'_>,
        accepted: Option<Rc<semantic::input_epoch::InputEpochFs>>,
    ) -> Result<SavedConfig, Error> {
        let native = NativeFs;
        let independent = accepted.is_some();
        let parent_identity = self
            .input_epoch
            .as_ref()
            .map(|epoch| (epoch.event_revision(), epoch.capture_identity()));
        let proof = Rc::new(RefCell::new(semantic::Observations::default()));
        // Event candidates are provisional until workspace admission. Capture their own
        // native history without publishing rejected bytes into the parent epoch.
        let local = self.input_epoch.as_ref().map(|epoch| {
            accepted.as_ref().map_or_else(
                || epoch.detached_recovery(&proof),
                |candidate| semantic::input_epoch::InputEpochFs::new(candidate.clone()),
            )
        });
        let base: &dyn FileSystem = local.as_ref().map_or_else(
            || {
                self.environment
                    .as_ref()
                    .map_or(&native as &dyn FileSystem, |env| &*env.fs)
            },
            |fs| fs as &dyn FileSystem,
        );
        let fs = semantic::ObservedFs {
            base,
            proof: Some(&proof),
            secondary: None,
        };
        let path = Path::new(uri.path());
        let identity = jaic::paths::file_identity(&fs, path);
        let state = match source {
            ConfigCapture::Buffer(text) => {
                // Client authority is explicit. The disk response is retained only as the
                // baseline whose later change reconciliation must notice.
                let _ = fs.read(path);
                SavedConfigState::ClientText(text.to_owned())
            }
            ConfigCapture::Initial(text) => match fs
                .read(path)
                .and_then(|bytes| String::from_utf8(bytes).ok())
            {
                Some(text) => SavedConfigState::Text(text),
                None => SavedConfigState::ClientText(text.to_owned()),
            },
            ConfigCapture::Disk => match fs.read(path) {
                Some(bytes) => match String::from_utf8(bytes) {
                    Ok(text) => SavedConfigState::Text(text),
                    Err(_) => SavedConfigState::Unavailable {
                        reason: ConfigFailure::Input,
                        recheck_at: Instant::now(),
                    },
                },
                None if fs.missing_file_key(path).is_some() => SavedConfigState::Missing,
                None => SavedConfigState::Unavailable {
                    reason: ConfigFailure::Input,
                    recheck_at: Instant::now(),
                },
            },
        };
        // Native replay records into the shared owner too; release the RefCell borrow
        // before delegating filesystem operations during validation.
        let captured_proof = proof.borrow().clone();
        if jaic::interrupt::fired()
            || (!independent
                && self
                    .input_epoch
                    .as_ref()
                    .map(|epoch| (epoch.event_revision(), epoch.capture_identity()))
                    != parent_identity)
            || !captured_proof.matches(base)
            || local.as_ref().is_some_and(|epoch| !epoch.validate_actual())
            || (!independent
                && self
                    .input_epoch
                    .as_ref()
                    .is_some_and(|epoch| !epoch.validate_actual()))
            || (!independent
                && self
                    .input_epoch
                    .as_ref()
                    .map(|epoch| (epoch.event_revision(), epoch.capture_identity()))
                    != parent_identity)
            || !proof.borrow().is_consistent()
            || jaic::interrupt::fired()
        {
            return Err(Error::DiagnosticsUnavailable);
        }
        if matches!(state, SavedConfigState::Unavailable { .. }) {
            return Err(Error::DiagnosticsUnavailable);
        }
        let actual_inputs = local
            .as_ref()
            .map(semantic::input_epoch::InputEpochFs::actual_inputs);
        let proof = proof.borrow().clone();
        Ok(SavedConfig {
            identity,
            state,
            proof,
            actual_inputs,
            owner: Rc::new(()),
        })
    }

    pub(crate) fn reject_config_inputs(&self) {
        self.configuration_input_failed.set(true);
    }

    /// Restore the original event capture before treating saved settings as an overlay.
    pub(crate) fn saved_config_for_use(&self, uri: &DocumentUri) -> Option<&SavedConfig> {
        let saved = self.saved_configs.get(uri)?;
        if saved.unavailable()
            || !self.restore_input_epoch(saved.actual_inputs.as_ref())
            || self
                .environment
                .as_ref()
                .is_some_and(|env| !saved.proof.matches(&*env.fs))
        {
            self.reject_config_inputs();
            return None;
        }
        self.input_proof.borrow_mut().merge(&saved.proof);
        if self.input_queries.get() > 0 {
            self.request_input_proof.borrow_mut().merge(&saved.proof);
        }
        Some(saved)
    }

    pub(super) fn saved_config_storage(&self) -> Result<usize, Error> {
        let saved = self
            .saved_configs
            .iter()
            .try_fold(0usize, |sum, (uri, saved)| {
                sum.checked_add(saved.storage(uri))
                    .ok_or(Error::Limit("workspace byte overflow"))
            })?;
        saved
            .checked_add(
                self.config_recovery
                    .borrow()
                    .as_ref()
                    .map_or(0, ConfigRecovery::storage),
            )
            .ok_or(Error::Limit("workspace byte overflow"))
    }

    /// Include a replacement saved snapshot exactly once in the workspace budget.
    /// `extra_buffer` reserves an opening document before its buffer enters the map.
    pub(super) fn admit_saved_config(
        &self,
        uri: &DocumentUri,
        saved: &SavedConfig,
        extra_buffer: usize,
    ) -> Result<(), Error> {
        self.admit_saved_config_size(uri, saved.storage(uri), extra_buffer)
    }

    fn admit_saved_config_size(
        &self,
        uri: &DocumentUri,
        needed: usize,
        extra_buffer: usize,
    ) -> Result<(), Error> {
        let buffers = self.buffer_storage()?;
        let old = self
            .saved_configs
            .get(uri)
            .map_or(0, |old| old.storage(uri));
        let own_recovery = self
            .config_recovery
            .borrow()
            .as_ref()
            .filter(|candidate| &candidate.uri == uri)
            .map_or(0, ConfigRecovery::storage);
        let total = buffers
            .checked_add(self.saved_config_storage()?)
            .and_then(|sum| sum.checked_sub(own_recovery))
            .and_then(|sum| sum.checked_sub(old))
            .and_then(|sum| sum.checked_add(needed))
            .and_then(|sum| sum.checked_add(extra_buffer))
            .ok_or(Error::Limit("workspace byte overflow"))?;
        if total > self.limits.workspace_bytes {
            return Err(Error::Limit("workspace byte budget exceeded"));
        }
        Ok(())
    }

    pub(super) fn buffer_storage(&self) -> Result<usize, Error> {
        self.documents
            .iter()
            .chain(&self.lint_configs)
            .try_fold(0usize, |sum, (uri, doc)| {
                sum.checked_add(uri.as_str().len() + doc.text.len())
            })
            .ok_or(Error::Limit("workspace byte overflow"))
    }

    fn admit_config_recovery(&self, needed: usize) -> Result<(), Error> {
        // The old unavailable owner remains live until commit: this is addition, not replacement.
        let total = self
            .buffer_storage()?
            .checked_add(self.saved_config_storage()?)
            .and_then(|sum| sum.checked_add(needed))
            .ok_or(Error::Limit("workspace byte overflow"))?;
        if total > self.limits.workspace_bytes {
            return Err(Error::Limit("workspace byte budget exceeded"));
        }
        Ok(())
    }

    /// Preserve a bounded prior association without retaining stale settings text.
    pub(super) fn unavailable_saved_config(&mut self, uri: &DocumentUri, reason: ConfigFailure) {
        if let Some(saved) = self.saved_configs.get_mut(uri) {
            saved.state = SavedConfigState::Unavailable {
                reason,
                recheck_at: Instant::now(),
            };
            saved.proof = semantic::Observations::default();
            saved.actual_inputs = None;
            saved.owner = Rc::new(());
        }
        self.reject_config_inputs();
    }
}

#[cfg(test)]
#[path = "saved_configs_tests.rs"]
mod tests;

#[path = "saved_config_recovery.rs"]
mod recovery;
pub(super) use recovery::ConfigRecovery;
