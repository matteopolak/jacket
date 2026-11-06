//! Actual filesystem responses owned by an accepted input epoch.
//!
//! Comparison proofs normalize some responses. This carrier preserves the spelling and
//! ordering returned to the caller, and refuses contradictory actual captures.
use jaic::filesystem::ReadBuffer;
use jaic::sema::FileSystem;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Weak};

const MAX_INPUTS: usize = 32_768;
const MAX_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug)]
enum ReadBytes {
    Owned(Arc<ReadBuffer>),
    ProofOwned(Weak<ReadBuffer>),
}

impl ReadBytes {
    fn upgrade(&self) -> Option<Arc<ReadBuffer>> {
        match self {
            Self::Owned(bytes) => Some(bytes.clone()),
            Self::ProofOwned(bytes) => bytes.upgrade(),
        }
    }
}

#[derive(Clone, Debug)]
enum Response {
    Read(Option<ReadBytes>),
    File(bool),
    Directory(bool),
    Listing(Vec<(String, bool)>),
    Canonical(PathBuf),
    ExactCanonical(Option<PathBuf>),
    Stamp(Option<u64>),
    Missing(Option<PathBuf>),
}

impl PartialEq for Response {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Read(None), Self::Read(None)) => true,
            (Self::Read(Some(a)), Self::Read(Some(b))) => match (a.upgrade(), b.upgrade()) {
                (Some(a), Some(b)) => a == b,
                _ => false,
            },
            (Self::File(a), Self::File(b)) | (Self::Directory(a), Self::Directory(b)) => a == b,
            (Self::Listing(a), Self::Listing(b)) => a == b,
            (Self::Canonical(a), Self::Canonical(b)) => a.as_os_str() == b.as_os_str(),
            (Self::ExactCanonical(a), Self::ExactCanonical(b))
            | (Self::Missing(a), Self::Missing(b)) => {
                a.as_deref().map(Path::as_os_str) == b.as_deref().map(Path::as_os_str)
            }
            (Self::Stamp(a), Self::Stamp(b)) => a == b,
            _ => false,
        }
    }
}

impl Response {
    fn kind(&self) -> u8 {
        match self {
            Self::Read(_) => 0,
            Self::File(_) => 1,
            Self::Directory(_) => 2,
            Self::Listing(_) => 3,
            Self::Canonical(_) => 4,
            Self::ExactCanonical(_) => 5,
            Self::Stamp(_) => 6,
            Self::Missing(_) => 7,
        }
    }

    fn matches(&self, fs: &dyn FileSystem, path: &Path) -> bool {
        match self {
            Self::Read(None) => fs.read(path).is_none(),
            Self::Read(Some(value)) => value
                .upgrade()
                .is_some_and(|original| fs.read(path).as_deref() == Some(original.as_slice())),
            Self::File(value) => fs.is_file(path) == *value,
            Self::Directory(value) => fs.is_dir(path) == *value,
            Self::Listing(value) => fs.list_dir(path) == *value,
            Self::Canonical(value) => fs.canonical(path).as_os_str() == value.as_os_str(),
            Self::ExactCanonical(value) => {
                fs.canonical_exact(path).as_deref().map(Path::as_os_str)
                    == value.as_deref().map(Path::as_os_str)
            }
            Self::Stamp(value) => fs.stamp(path) == *value,
            Self::Missing(value) => {
                fs.missing_file_key(path).as_deref().map(Path::as_os_str)
                    == value.as_deref().map(Path::as_os_str)
            }
        }
    }

    fn capture(fs: &dyn FileSystem, path: &Path, kind: u8) -> Self {
        match kind {
            0 => Self::Read(fs.read_shared(path).map(ReadBytes::Owned)),
            1 => Self::File(fs.is_file(path)),
            2 => Self::Directory(fs.is_dir(path)),
            3 => Self::Listing(fs.list_dir(path)),
            4 => Self::Canonical(fs.canonical(path)),
            5 => Self::ExactCanonical(fs.canonical_exact(path)),
            6 => Self::Stamp(fs.stamp(path)),
            7 => Self::Missing(fs.missing_file_key(path)),
            _ => unreachable!("input operation kind"),
        }
    }

    fn available(&self) -> bool {
        !matches!(self, Self::Read(Some(ReadBytes::ProofOwned(bytes))) if bytes.upgrade().is_none())
    }

    fn borrow_proof_bytes(&mut self) {
        if let Self::Read(Some(ReadBytes::Owned(bytes))) = self {
            *self = Self::Read(Some(ReadBytes::ProofOwned(Arc::downgrade(bytes))));
        }
    }

    fn work_bytes(&self) -> usize {
        match self {
            Self::Read(Some(bytes)) => bytes.upgrade().map_or(0, |bytes| bytes.len()),
            _ => self.retained_bytes(),
        }
    }

    fn retained_bytes(&self) -> usize {
        match self {
            Self::Read(Some(ReadBytes::Owned(bytes))) => {
                bytes.len() + 3 * std::mem::size_of::<usize>()
            }
            Self::Read(Some(ReadBytes::ProofOwned(_))) => std::mem::size_of::<ReadBuffer>() + 64,
            Self::Read(None) => 0,
            Self::Listing(rows) => {
                rows.capacity() * std::mem::size_of::<(String, bool)>()
                    + rows.iter().map(|(name, _)| name.capacity()).sum::<usize>()
            }
            Self::Canonical(path) => path.capacity(),
            Self::ExactCanonical(path) | Self::Missing(path) => {
                path.as_ref().map_or(0, PathBuf::capacity)
            }
            _ => 0,
        }
    }
}

type Responses = BTreeMap<(std::ffi::OsString, u8), Response>;

#[derive(Clone)]
pub(crate) struct ActualInputs(Rc<Responses>);

impl ActualInputs {
    pub(crate) fn agrees_with(&self, other: &Self) -> bool {
        self.0
            .iter()
            .all(|(key, value)| other.0.get(key).is_none_or(|other| value == other))
    }

    /// A known event revokes captured history, including unrelated prefix operations.
    /// This is conservative input ownership, not semantic dependency precision.
    pub(crate) fn concerns_tree(&self, path: &Path) -> bool {
        self.concerns_tree_since(path, None)
    }

    /// A fresh event capture can prove an affected response unchanged without granting reuse.
    pub(crate) fn concerns_tree_since(&self, path: &Path, current: Option<&Self>) -> bool {
        self.concerns_tree_with(path, current, jaic::paths::identity_spelling)
    }

    fn concerns_tree_with(
        &self,
        path: &Path,
        current: Option<&Self>,
        spelling: impl Fn(&Path) -> PathBuf,
    ) -> bool {
        let changed = spelling(path);
        self.0.iter().any(|((raw, kind), response)| {
            if response.available()
                && current
                    .and_then(|current| current.0.get(&(raw.clone(), *kind)))
                    .is_some_and(|fresh| fresh.available() && fresh == response)
            {
                return false;
            }
            let key = spelling(Path::new(raw));
            key.starts_with(&changed)
                || (*kind == 3 && changed.parent() == Some(key.as_path()))
                || (*kind == 6 && changed.starts_with(&key))
                || match response {
                    Response::Canonical(target) => spelling(target).starts_with(&changed),
                    Response::ExactCanonical(Some(target)) | Response::Missing(Some(target)) => {
                        spelling(target).starts_with(&changed)
                    }
                    _ => false,
                }
        })
    }

    /// Charge shared read payload once when this exact proof remains in the same owner.
    /// Equal bytes from another allocation or raw spelling do not qualify.
    pub(crate) fn retained_bytes_with_proof(&self, proof: &super::Observations) -> usize {
        let shared = self
            .0
            .iter()
            .map(|((path, _), response)| match response {
                Response::Read(Some(ReadBytes::Owned(owner)))
                    if proof.owns_read(Path::new(path), owner) =>
                {
                    owner.len()
                }
                _ => 0,
            })
            .sum::<usize>();
        self.retained_bytes().saturating_sub(shared)
    }

    pub(crate) fn retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self
                .0
                .iter()
                .map(|((path, _), value)| {
                    path.capacity()
                        + value.retained_bytes()
                        + std::mem::size_of::<((std::ffi::OsString, u8), Response)>()
                        + 64
                })
                .sum::<usize>()
    }
}

#[derive(Default)]
struct Capture {
    responses: Rc<Responses>,
    bytes: usize,
    inconsistent: bool,
}

/// Native-derived input authority. Sealed reads never delegate to its native provider.
/// For example, capture a read, seal after native proof acceptance, then reuse its bytes.
pub(crate) struct InputEpochFs {
    native: Rc<dyn FileSystem>,
    read_owner: std::rc::Weak<RefCell<super::Observations>>,
    capture: RefCell<Capture>,
    sealed: Cell<bool>,
    unavailable: Cell<bool>,
    coverage_miss: Cell<bool>,
    pending: RefCell<BTreeSet<(std::ffi::OsString, u8)>>,
    pending_bytes: Cell<usize>,
    native_required: RefCell<BTreeSet<(std::ffi::OsString, u8)>>,
    native_required_bytes: Cell<usize>,
    hydrating: Cell<bool>,
    updates_pending: Cell<bool>,
    revision: Cell<u64>,
    event_revision: Cell<u64>,
    capture_token: Cell<u64>,
    hint_role: RefCell<Option<Rc<Self>>>,
    hint_service_turn: Cell<bool>,
    hint_uses: Cell<usize>,
    hint_parent: Option<(std::rc::Weak<Self>, u64)>,
    reconcile_after: RefCell<Option<(std::ffi::OsString, u8)>>,
    reconcile_rounds: Cell<u64>,
}

impl InputEpochFs {
    pub(crate) fn new(native: Rc<dyn FileSystem>) -> Self {
        Self {
            native,
            read_owner: std::rc::Weak::new(),
            capture: RefCell::new(Capture {
                bytes: std::mem::size_of::<Self>(),
                ..Capture::default()
            }),
            sealed: Cell::new(false),
            unavailable: Cell::new(false),
            coverage_miss: Cell::new(false),
            pending: RefCell::default(),
            pending_bytes: Cell::new(0),
            native_required: RefCell::default(),
            native_required_bytes: Cell::new(0),
            hydrating: Cell::new(false),
            updates_pending: Cell::new(false),
            revision: Cell::new(0),
            event_revision: Cell::new(0),
            capture_token: Cell::new(0),
            hint_role: RefCell::default(),
            hint_service_turn: Cell::new(false),
            hint_uses: Cell::new(0),
            hint_parent: None,
            reconcile_after: RefCell::default(),
            reconcile_rounds: Cell::new(0),
        }
    }

    /// A detached between-request transaction, sharing the existing byte-proof owner.
    pub(crate) fn detached_recovery(&self, proof: &Rc<RefCell<super::Observations>>) -> Self {
        let mut recovery = Self::new(Rc::new(super::observations::OwnedObservedFs {
            base: self.native.clone(),
            proof: proof.clone(),
        }));
        recovery.read_owner = Rc::downgrade(proof);
        recovery
    }

    fn own_bytes(&self, capture: &Capture) -> usize {
        capture.bytes
            + self.pending_bytes.get()
            + self.native_required_bytes.get()
            + self.read_owner.upgrade().map_or(0, |_| {
                std::mem::size_of::<super::observations::OwnedObservedFs>()
                    + std::mem::size_of::<RefCell<super::Observations>>()
                    + 96
            })
    }

    fn shared_read_bytes(capture: &Capture, parent: &Capture) -> usize {
        capture
            .responses
            .iter()
            .filter_map(|(key, value)| {
                let Response::Read(Some(ReadBytes::Owned(bytes))) = value else {
                    return None;
                };
                match parent.responses.get(key) {
                    Some(Response::Read(Some(ReadBytes::Owned(other))))
                        if std::sync::Arc::ptr_eq(bytes.bytes(), other.bytes()) =>
                    {
                        Some(bytes.len())
                    }
                    _ => None,
                }
            })
            .sum()
    }

    fn extra_against(&self, parent: &Capture) -> usize {
        let capture = self.capture.borrow();
        self.own_bytes(&capture)
            .saturating_sub(Self::shared_read_bytes(&capture, parent))
    }

    fn remaining_for(&self, capture: &Capture) -> usize {
        let own = self.own_bytes(capture);
        let used = if let Some((parent, _)) = &self.hint_parent {
            if let Some(parent) = parent.upgrade() {
                let parent_capture = parent.capture.borrow();
                own.saturating_add(parent.own_bytes(&parent_capture))
                    .saturating_sub(Self::shared_read_bytes(capture, &parent_capture))
            } else {
                usize::MAX
            }
        } else {
            own.saturating_add(
                self.hint_role
                    .borrow()
                    .as_ref()
                    .map_or(0, |hint| hint.extra_against(capture)),
            )
        };
        MAX_BYTES.saturating_sub(used)
    }

    pub(crate) fn retained_bytes(&self) -> usize {
        let capture = self.capture.borrow();
        self.own_bytes(&capture)
            + self
                .hint_role
                .borrow()
                .as_ref()
                .map_or(0, |hint| hint.extra_against(&capture))
    }

    /// Optional rendering owns a separate input role and never records into semantic input.
    pub(crate) fn hint_fork(self: &Rc<Self>) -> Option<Rc<Self>> {
        if let Some(fork) = self.hint_role.borrow().as_ref() {
            return Some(fork.clone());
        }
        if self.input_epoch_valid() == Some(false) {
            return None;
        }
        let mut fork = Self::new(self.native.clone());
        fork.hint_parent = Some((Rc::downgrade(self), self.capture_token.get()));
        let parent = self.capture.borrow();
        *fork.capture.borrow_mut() = Capture {
            responses: parent.responses.clone(),
            bytes: parent.bytes,
            inconsistent: parent.inconsistent,
        };
        fork.sealed.set(self.sealed.get());
        if self
            .own_bytes(&parent)
            .saturating_add(fork.extra_against(&parent))
            > MAX_BYTES
        {
            return None;
        }
        drop(parent);
        let fork = Rc::new(fork);
        *self.hint_role.borrow_mut() = Some(fork.clone());
        Some(fork)
    }

    /// The original provider is available only to nonbinding, notification-governed
    /// catalogues. Exact discovery and compiler hints must retain epoch authority.
    pub(crate) fn advisory_provider(&self) -> Rc<dyn FileSystem> {
        self.native.clone()
    }

    /// Keep the optional carrier stable until the outer render transaction finishes.
    pub(crate) fn enter_hint_query(&self) {
        self.hint_uses.set(self.hint_uses.get() + 1);
    }

    pub(crate) fn leave_hint_query(&self) {
        self.hint_uses.set(self.hint_uses.get() - 1);
    }

    pub(crate) fn hint_query_available(&self) -> bool {
        self.hint_parent_current()
            && !self.unavailable.get()
            && self.hint_parent.as_ref().is_some_and(|(parent, _)| {
                parent
                    .upgrade()
                    .is_some_and(|parent| !parent.sealed.get() || self.sealed.get())
            })
    }

    // A changed optional response starts a successor of the same semantic epoch. Keep
    // only its required keys; fresh values are acquired by later bounded service turns.
    fn restart_hint(&self, parent: &Self) {
        let prefix = parent.capture.borrow();
        let mut keys = self.pending.borrow().clone();
        keys.extend(
            self.capture
                .borrow()
                .responses
                .keys()
                .filter(|key| !prefix.responses.contains_key(*key))
                .cloned(),
        );
        self.reset_capture();
        *self.capture.borrow_mut() = Capture {
            responses: prefix.responses.clone(),
            bytes: prefix.bytes,
            inconsistent: prefix.inconsistent,
        };
        self.sealed.set(true);
        drop(prefix);
        for (path, kind) in keys {
            self.queue_missing(Path::new(&path), kind);
        }
    }

    pub(crate) fn hint_parent_current(&self) -> bool {
        self.hint_parent.as_ref().is_some_and(|(parent, token)| {
            parent
                .upgrade()
                .is_some_and(|parent| parent.capture_token.get() == *token)
        })
    }

    pub(crate) fn accept_hint_inputs(&self) -> bool {
        let Some((parent, token)) = &self.hint_parent else {
            return false;
        };
        let Some(parent) = parent.upgrade() else {
            return false;
        };
        if parent.capture_token.get() != *token {
            return false;
        }
        let semantic = parent.actual_inputs();
        if !self.actual_inputs().agrees_with(&semantic) {
            return false;
        }
        if !parent.sealed.get() {
            // Retain earlier hint responses and add newly observed semantic inputs before
            // a native union replay. Reopening occurs only at final query acceptance.
            self.sealed.set(false);
            if !self.reuse_actual(&semantic) {
                return false;
            }
        }
        self.seal_after_validation() && self.hint_parent_current()
    }

    pub(crate) fn capture_identity(&self) -> u64 {
        self.capture_token.get()
    }

    pub(crate) fn event_revision(&self) -> u64 {
        self.event_revision.get()
    }

    pub(crate) fn captured_read_bytes(&self, path: &Path) -> usize {
        match self
            .capture
            .borrow()
            .responses
            .get(&(path.as_os_str().to_owned(), 0))
        {
            Some(Response::Read(Some(bytes))) => bytes.upgrade().map_or(0, |bytes| bytes.len()),
            _ => 0,
        }
    }

    /// Seal only an actual-consistent capture, after the caller's full native proof succeeds.
    /// This carrier alone does not validate source, selection, events or semantic owner state.
    pub(crate) fn seal_after_validation(&self) -> bool {
        if self.sealed.get() {
            return self.ready() && !jaic::interrupt::fired();
        }
        if !self.validate_actual() {
            return false;
        }
        let capture = self.capture.borrow();
        self.native_required
            .borrow_mut()
            .retain(|key| !capture.responses.contains_key(key));
        self.native_required_bytes.set(
            self.native_required
                .borrow()
                .iter()
                .map(|(path, _)| {
                    path.capacity() + std::mem::size_of::<(std::ffi::OsString, u8)>() + 64
                })
                .sum(),
        );
        if !self.native_required.borrow().is_empty() {
            // Full native proof can accept an answer while an oversized optional operation
            // still needs a durable exact owner. Keep strict native capture, never seal it.
            return true;
        }
        self.reconcile_after.borrow_mut().take();
        self.sealed.set(true);
        true
    }

    /// Validate an accepted producer's actual history without closing its enclosing query.
    pub(crate) fn validate_actual(&self) -> bool {
        if jaic::interrupt::fired() {
            return false;
        }
        if self.sealed.get() {
            return self.ready();
        }
        if self.capture.borrow().inconsistent || self.unavailable.get() || self.coverage_miss.get()
        {
            return false;
        }
        let revision = self.revision.get();
        let responses = self.capture.borrow().responses.clone();
        if responses.values().any(|value| !value.available()) {
            self.unavailable.set(true);
            self.updates_pending.set(true);
            return false;
        }
        let current = responses
            .iter()
            .all(|((path, _), value)| value.matches(&*self.native, Path::new(path)));
        if !current
            || revision != self.revision.get()
            || self.capture.borrow().inconsistent
            || jaic::interrupt::fired()
        {
            self.unavailable.set(true);
            return false;
        }
        true
    }

    /// Start a later capture transaction after a known event or a coverage miss.
    pub(crate) fn begin_capture(&self) {
        self.event_revision
            .set(self.event_revision.get().wrapping_add(1));
        self.reset_capture();
    }

    fn reset_capture(&self) {
        self.capture_token
            .set(self.capture_token.get().wrapping_add(1));
        self.hint_role.borrow_mut().take();
        self.reconcile_after.borrow_mut().take();
        self.sealed.set(false);
        self.unavailable.set(false);
        self.coverage_miss.set(false);
        self.pending.borrow_mut().clear();
        self.pending_bytes.set(0);
        self.native_required.borrow_mut().clear();
        self.native_required_bytes.set(0);
        self.hydrating.set(false);
        self.updates_pending.set(false);
        self.revision.set(self.revision.get().wrapping_add(1));
        *self.capture.borrow_mut() = Capture {
            bytes: std::mem::size_of::<Self>(),
            ..Capture::default()
        };
    }

    /// A later request can fall back to native proof until required oversized reads have
    /// acquired an exact proof owner. Known file/document events use `begin_capture` instead.
    pub(crate) fn begin_request_capture(&self) {
        let required = self.native_required.replace(BTreeSet::new());
        let bytes = self.native_required_bytes.get();
        self.reset_capture();
        self.native_required.replace(required);
        self.native_required_bytes.set(bytes);
    }

    /// Start input-only recovery for retained raw buffer notifications between requests.
    /// Promotion still requires the ordinary bounded native replay; no answer is accepted here.
    pub(crate) fn prepare_buffer_identity_hydration(&self, retry_failed: bool) -> bool {
        if self.unavailable.get() || self.capture.borrow().inconsistent {
            if !retry_failed {
                return false;
            }
            self.begin_request_capture();
        }
        if !self.sealed.get() {
            self.sealed.set(true);
            self.coverage_miss.set(true);
            self.hydrating.set(true);
            self.reconcile_after.borrow_mut().take();
        }
        true
    }

    /// Check a bounded slice of every captured API kind; never mutate sealed response data.
    /// One operation is allowed before observing cancellation so busy traffic makes progress.
    pub(crate) fn reconcile_step(&self, max_ops: usize, max_bytes: usize) -> Option<PathBuf> {
        let hint_turn = self
            .hint_service_turn
            .replace(!self.hint_service_turn.get());
        if hint_turn && self.ready() {
            let hint = self.hint_role.borrow().clone();
            if let Some(hint) =
                hint.filter(|hint| hint.hint_uses.get() == 0 && hint.hint_parent_current())
            {
                if !hint.sealed.get() {
                    hint.restart_hint(self);
                }
                let token = self.capture_token.get();
                let revision = self.revision.get();
                let changed = hint.reconcile_step_inner(max_ops, max_bytes);
                if self.capture_token.get() != token
                    || self.revision.get() != revision
                    || !self.ready()
                {
                    hint.unavailable.set(true);
                    return None;
                }
                if let Some(key) = changed {
                    if self.capture.borrow().responses.contains_key(&key) {
                        self.unavailable.set(true);
                        return Some(PathBuf::from(key.0));
                    }
                    hint.restart_hint(self);
                }
                if hint.take_updates_pending() {
                    self.updates_pending.set(true);
                }
                return None;
            }
        }
        self.reconcile_step_inner(max_ops, max_bytes)
            .map(|key| PathBuf::from(key.0))
    }

    fn reconcile_step_inner(
        &self,
        max_ops: usize,
        max_bytes: usize,
    ) -> Option<(std::ffi::OsString, u8)> {
        use std::ops::Bound::{Excluded, Unbounded};
        if !self.has_work() || self.unavailable.get() || max_ops == 0 {
            return None;
        }
        let revision = self.revision.get();
        // Hydration is a successor transaction. Queries remain sealed and unavailable
        // throughout it, including while native responses are being captured.
        let next = self.pending.borrow_mut().pop_first();
        if let Some((path, kind)) = next {
            self.pending_bytes
                .set(self.pending_bytes.get().saturating_sub(
                    path.capacity() + std::mem::size_of::<(std::ffi::OsString, u8)>() + 64,
                ));
            self.hydrating.set(true);
            let mut response = Response::capture(&*self.native, Path::new(&path), kind);
            if let Response::Read(Some(ReadBytes::Owned(owner))) = &response
                && self
                    .read_owner
                    .upgrade()
                    .is_some_and(|proof| proof.borrow().owns_read(Path::new(&path), owner))
            {
                response.borrow_proof_bytes();
            }
            if revision == self.revision.get() && self.sealed.get() {
                let required = response.retained_bytes().saturating_add(
                    path.capacity()
                        + std::mem::size_of::<((std::ffi::OsString, u8), Response)>()
                        + 64,
                );
                let mut remaining = self.remaining_for(&self.capture.borrow());
                if required > remaining && self.hint_parent.is_none() {
                    self.discard_optional_hint();
                    remaining = self.remaining_for(&self.capture.borrow());
                }
                if matches!(&response, Response::Read(Some(ReadBytes::Owned(_))))
                    && required > remaining
                {
                    // No exact consumer owns hydration bytes. Do not keep an oversized
                    // strong temporary or repeatedly recapture it without an owner.
                    let bytes =
                        path.capacity() + std::mem::size_of::<(std::ffi::OsString, u8)>() + 64;
                    if self.native_required.borrow_mut().insert((path, kind)) {
                        self.native_required_bytes
                            .set(self.native_required_bytes.get() + bytes);
                    }
                    self.hydrating.set(false);
                    self.unavailable.set(true);
                    self.updates_pending.set(true);
                } else {
                    self.record(Path::new(&path), response);
                }
            }
            return None;
        }
        if self.coverage_miss.get() && !self.hydrating.get() {
            return None;
        }
        let responses = self.capture.borrow().responses.clone();
        let started = std::time::Instant::now();
        let mut after = self.reconcile_after.borrow().clone();
        let mut bytes = 0usize;
        for op in 0..max_ops {
            let next = match &after {
                Some(key) => responses.range((Excluded(key), Unbounded)).next(),
                None => responses.iter().next(),
            };
            let Some((key, response)) = next else {
                if self.hydrating.get() && jaic::interrupt::should_stop() {
                    return None;
                }
                self.reconcile_rounds
                    .set(self.reconcile_rounds.get().wrapping_add(1));
                after = None;
                if self.hydrating.get() && self.pending.borrow().is_empty() {
                    self.hydrating.set(false);
                    self.coverage_miss.set(false);
                    self.capture_token
                        .set(self.capture_token.get().wrapping_add(1));
                    self.hint_role.borrow_mut().take();
                    self.updates_pending.set(true);
                }
                break;
            };
            if !response.available() {
                self.hydrating.set(false);
                self.unavailable.set(true);
                self.updates_pending.set(true);
                return Some(key.clone());
            }
            let expected = response.work_bytes();
            if op > 0
                && (expected > max_bytes.saturating_sub(bytes)
                    || started.elapsed() >= std::time::Duration::from_millis(1))
            {
                break;
            }
            let matches = response.matches(&*self.native, Path::new(&key.0));
            if revision != self.revision.get() || !self.sealed.get() {
                return None;
            }
            if !matches {
                self.hydrating.set(false);
                self.unavailable.set(true);
                self.reconcile_after.borrow_mut().take();
                return Some(key.clone());
            }
            bytes = bytes.saturating_add(expected);
            after = Some(key.clone());
            if jaic::interrupt::should_stop() {
                break;
            }
        }
        *self.reconcile_after.borrow_mut() = after;
        None
    }

    pub(crate) fn actual_inputs(&self) -> ActualInputs {
        ActualInputs(self.capture.borrow().responses.clone())
    }

    /// Reused facts retain the actual responses that originally produced them.
    pub(crate) fn reuse_actual(&self, original: &ActualInputs) -> bool {
        if original.0.values().any(|value| !value.available()) {
            self.unavailable.set(true);
            self.updates_pending.set(true);
            return false;
        }
        if self.sealed.get() {
            let capture = self.capture.borrow();
            let mut matches = true;
            for (key, value) in &*original.0 {
                match capture.responses.get(key) {
                    Some(current) if current == value => {}
                    Some(_) => {
                        // Two accepted histories disagree: this is not missing coverage.
                        self.unavailable.set(true);
                        self.updates_pending.set(true);
                        matches = false;
                    }
                    None => {
                        self.queue_missing(Path::new(&key.0), key.1);
                        matches = false;
                    }
                }
            }
            return matches && self.ready();
        }
        for ((path, _), value) in &*original.0 {
            self.record(Path::new(path), value.clone());
        }
        !self.capture.borrow().inconsistent && !self.unavailable.get()
    }

    pub(crate) fn has_pending_coverage(&self) -> bool {
        !self.unavailable.get() && (self.hydrating.get() || !self.pending.borrow().is_empty())
    }

    pub(crate) fn has_work(&self) -> bool {
        self.updates_pending.get()
            || self.buffer_identity_inputs_ready()
            || self.has_pending_coverage()
    }

    pub(crate) fn take_updates_pending(&self) -> bool {
        self.updates_pending.replace(false)
    }

    fn queue_missing(&self, path: &Path, kind: u8) {
        self.coverage_miss.set(true);
        let key = (path.as_os_str().to_owned(), kind);
        let mut pending = self.pending.borrow_mut();
        if pending.contains(&key) {
            return;
        }
        let bytes = key.0.capacity() + std::mem::size_of::<(std::ffi::OsString, u8)>() + 64;
        let capture = self.capture.borrow();
        if bytes > self.remaining_for(&capture) && self.hint_parent.is_none() {
            self.discard_optional_hint();
        }
        if capture.responses.len() + pending.len() + self.native_required.borrow().len()
            >= MAX_INPUTS
            || bytes > self.remaining_for(&capture)
        {
            self.unavailable.set(true);
            return;
        }
        self.pending_bytes.set(self.pending_bytes.get() + bytes);
        pending.insert(key);
        self.reconcile_after.borrow_mut().take();
    }

    pub(crate) fn ready(&self) -> bool {
        self.buffer_identity_inputs_ready() && self.native_required.borrow().is_empty()
    }

    /// Completed input replay can resolve buffer identities without accepting an answer.
    /// An oversized required read still needs the existing strict native transaction.
    pub(crate) fn buffer_identity_inputs_ready(&self) -> bool {
        self.sealed.get()
            && !self.unavailable.get()
            && !self.coverage_miss.get()
            && !self.capture.borrow().inconsistent
    }

    /// Hand the same replayed history to strict capture after identity admission finishes.
    /// For example, an oversized options input can acquire its exact proof owner there.
    pub(crate) fn finish_buffer_identity_hydration(&self) {
        if self.buffer_identity_inputs_ready() && !self.native_required.borrow().is_empty() {
            self.sealed.set(false);
        }
    }

    pub(crate) fn needs_recapture(&self) -> bool {
        self.coverage_missed() || self.unavailable.get() || self.capture.borrow().inconsistent
    }

    pub(crate) fn coverage_missed(&self) -> bool {
        self.coverage_miss.get()
    }

    fn lookup(&self, path: &Path, kind: u8) -> Option<Response> {
        if !self.sealed.get() {
            return None;
        }
        let mut value = self
            .capture
            .borrow()
            .responses
            .get(&(path.as_os_str().to_owned(), kind))
            .cloned();
        if value.as_ref().is_some_and(|response| !response.available()) {
            value = None;
        }
        if value.is_none() {
            self.queue_missing(path, kind);
        }
        value
    }

    fn discard_optional_hint(&self) {
        if self.hint_parent.is_some() {
            return;
        }
        if let Some(hint) = self.hint_role.borrow_mut().take() {
            hint.unavailable.set(true);
            hint.revision.set(hint.revision.get().wrapping_add(1));
            *hint.capture.borrow_mut() = Capture {
                bytes: std::mem::size_of::<Self>(),
                inconsistent: true,
                ..Capture::default()
            };
            hint.pending.borrow_mut().clear();
            hint.pending_bytes.set(0);
            hint.native_required.borrow_mut().clear();
            hint.native_required_bytes.set(0);
            hint.hydrating.set(false);
        }
    }

    fn record(&self, path: &Path, mut response: Response) {
        let mut capture = self.capture.borrow_mut();
        let key = (path.as_os_str().to_owned(), response.kind());
        if let Some(first) = capture.responses.get(&key) {
            if first != &response {
                capture.inconsistent = true;
            }
            return;
        }
        let overhead =
            key.0.capacity() + std::mem::size_of::<((std::ffi::OsString, u8), Response)>() + 64;
        let mut remaining = self.remaining_for(&capture);
        if overhead.saturating_add(response.retained_bytes()) > remaining
            && self.hint_parent.is_none()
        {
            self.discard_optional_hint();
            remaining = self.remaining_for(&capture);
        }
        if overhead.saturating_add(response.retained_bytes()) > remaining {
            response.borrow_proof_bytes();
        }
        let bytes = overhead.saturating_add(response.retained_bytes());
        if capture.responses.len() >= MAX_INPUTS || bytes > remaining {
            capture.inconsistent = true;
            self.unavailable.set(true);
            return;
        }
        capture.bytes += bytes;
        Rc::make_mut(&mut capture.responses).insert(key, response);
        if self.hydrating.get() {
            self.reconcile_after.borrow_mut().take();
        }
        self.revision.set(self.revision.get().wrapping_add(1));
    }
}

impl FileSystem for InputEpochFs {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        self.read_shared(path).map(|bytes| bytes.to_vec())
    }

    fn read_shared(&self, path: &Path) -> Option<Arc<ReadBuffer>> {
        if self.sealed.get() {
            return match self.lookup(path, 0) {
                Some(Response::Read(bytes)) => bytes.and_then(|bytes| bytes.upgrade()),
                _ => None,
            };
        }
        let value = self.native.read_shared(path);
        self.record(path, Response::Read(value.clone().map(ReadBytes::Owned)));
        if let Some(returned) = &value {
            let original = self
                .capture
                .borrow()
                .responses
                .get(&(path.as_os_str().to_owned(), 0))
                .and_then(|response| match response {
                    Response::Read(Some(bytes)) => bytes.upgrade(),
                    _ => None,
                });
            if let Some(original) = original
                && original.as_slice() == returned.as_slice()
            {
                // Every exact proof must retain the owner behind this first response,
                // rather than an equal temporary returned by a later native replay.
                return Some(original);
            }
        }
        value
    }

    fn is_file(&self, path: &Path) -> bool {
        if self.sealed.get() {
            return matches!(self.lookup(path, 1), Some(Response::File(true)));
        }
        let value = self.native.is_file(path);
        self.record(path, Response::File(value));
        value
    }

    fn is_dir(&self, path: &Path) -> bool {
        if self.sealed.get() {
            return matches!(self.lookup(path, 2), Some(Response::Directory(true)));
        }
        let value = self.native.is_dir(path);
        self.record(path, Response::Directory(value));
        value
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        if self.sealed.get() {
            return match self.lookup(path, 3) {
                Some(Response::Listing(value)) => value,
                _ => Vec::new(),
            };
        }
        let value = self.native.list_dir(path);
        self.record(path, Response::Listing(value.clone()));
        value
    }

    fn canonical(&self, path: &Path) -> PathBuf {
        if self.sealed.get() {
            return match self.lookup(path, 4) {
                Some(Response::Canonical(value)) => value,
                _ => jaic::paths::normalize(path),
            };
        }
        let value = self.native.canonical(path);
        self.record(path, Response::Canonical(value.clone()));
        value
    }

    fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
        if self.sealed.get() {
            return match self.lookup(path, 5) {
                Some(Response::ExactCanonical(value)) => value,
                _ => None,
            };
        }
        let value = self.native.canonical_exact(path);
        self.record(path, Response::ExactCanonical(value.clone()));
        value
    }

    fn stamp(&self, path: &Path) -> Option<u64> {
        if self.sealed.get() {
            return match self.lookup(path, 6) {
                Some(Response::Stamp(value)) => value,
                _ => None,
            };
        }
        let value = self.native.stamp(path);
        self.record(path, Response::Stamp(value));
        value
    }

    fn missing_file_key(&self, path: &Path) -> Option<PathBuf> {
        if self.sealed.get() {
            return match self.lookup(path, 7) {
                Some(Response::Missing(value)) => value,
                _ => None,
            };
        }
        let value = self.native.missing_file_key(path);
        self.record(path, Response::Missing(value.clone()));
        value
    }

    fn is_native(&self) -> bool {
        false
    }

    fn tracks_native_inputs(&self) -> bool {
        true
    }

    fn input_epoch_valid(&self) -> Option<bool> {
        if self.needs_recapture() {
            Some(false)
        } else {
            self.sealed.get().then(|| self.ready())
        }
    }
}

#[cfg(test)]
#[path = "input_epoch_tests.rs"]
mod tests;
