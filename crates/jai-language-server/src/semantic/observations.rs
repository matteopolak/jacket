//! Exact native filesystem responses captured by one accepted semantic context.
use jaic::filesystem::ReadBuffer;
use jaic::sema::FileSystem;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Observation {
    Read(Option<Arc<[u8]>>),
    File(bool),
    Directory(bool),
    Listing(Vec<(String, bool)>),
    Canonical(PathBuf),
    ExactCanonical(Option<PathBuf>),
    Stamp(Option<u64>),
    Absence(PathBuf),
}

impl Observation {
    fn kind(&self) -> u8 {
        match self {
            Self::Read(_) => 0,
            Self::File(_) => 1,
            Self::Directory(_) => 2,
            Self::Listing(_) => 3,
            Self::Canonical(_) => 4,
            Self::Stamp(_) => 5,
            Self::ExactCanonical(_) => 6,
            Self::Absence(_) => 7,
        }
    }

    fn matches(&self, fs: &dyn FileSystem, path: &Path) -> bool {
        match self {
            Self::Read(bytes) => {
                #[cfg(test)]
                crate::reference_records::request_work_tests::logical_read(path);
                fs.read_shared(path).as_deref().map(ReadBuffer::as_slice) == bytes.as_deref()
            }
            Self::File(value) => fs.is_file(path) == *value,
            Self::Directory(value) => fs.is_dir(path) == *value,
            Self::Listing(entries) => {
                let mut current = fs.list_dir(path);
                current.sort();
                &current == entries
            }
            Self::Canonical(value) => jaic::paths::file_identity(fs, path) == *value,
            Self::Stamp(value) => fs.stamp(path) == *value,
            Self::ExactCanonical(value) => fs.canonical_exact(path) == *value,
            Self::Absence(key) => fs.missing_file_key(path).as_ref() == Some(key),
        }
    }

    fn bytes(&self) -> usize {
        match self {
            Self::Read(bytes) => bytes.as_ref().map_or(0, |b| b.len()),
            Self::Canonical(path) | Self::Absence(path) => path.capacity(),
            Self::ExactCanonical(path) => path.as_ref().map_or(0, PathBuf::capacity),
            Self::Listing(entries) => {
                entries.capacity() * std::mem::size_of::<(String, bool)>()
                    + entries
                        .iter()
                        .map(|(name, _)| name.capacity())
                        .sum::<usize>()
            }
            _ => 0,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Observations {
    first: BTreeMap<(PathBuf, u8), Observation>,
    owners: BTreeMap<std::ffi::OsString, Arc<ReadBuffer>>,
    consistent: bool,
}

impl Default for Observations {
    fn default() -> Self {
        Self {
            first: BTreeMap::new(),
            owners: BTreeMap::new(),
            consistent: true,
        }
    }
}

impl Observations {
    pub(crate) fn remove_invalid(&mut self, fs: &dyn FileSystem) -> Vec<PathBuf> {
        #[cfg(test)]
        let _work = crate::reference_records::request_work_tests::scope(
            "discovery_retirement",
            self.first.len(),
        );
        let all = !self.consistent;
        let mut changed = std::collections::BTreeSet::new();
        self.first.retain(|(path, _), value| {
            let valid = !all && value.matches(fs, path);
            if !valid {
                changed.insert(path.clone());
            }
            valid
        });
        let first = &self.first;
        self.owners
            .retain(|path, _| first.contains_key(&(PathBuf::from(path), 0)));
        self.consistent = true;
        changed.into_iter().collect()
    }

    pub(crate) fn merge(&mut self, other: &Self) {
        self.consistent &= other.consistent;
        for ((path, _), value) in &other.first {
            self.record(path.clone(), value.clone());
        }
        for (path, owner) in &other.owners {
            self.record_read_owner(PathBuf::from(path), owner.clone());
        }
    }

    /// Keep the actual read owner with its first byte proof, including after history merges.
    pub(crate) fn record_read_owner(&mut self, path: PathBuf, owner: Arc<ReadBuffer>) {
        if matches!(self.first.get(&(path.clone(), 0)), Some(Observation::Read(Some(bytes))) if bytes.as_ref() == owner.as_slice())
        {
            self.owners.entry(path.into_os_string()).or_insert(owner);
        }
    }

    pub(crate) fn record(&mut self, path: PathBuf, value: Observation) {
        match self.first.entry((path, value.kind())) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(value);
            }
            std::collections::btree_map::Entry::Occupied(entry) => {
                self.consistent &= *entry.get() == value;
            }
        }
    }

    /// The consistency of their union, without allocating another copy of either history.
    pub(crate) fn agrees_with(&self, other: &Self) -> bool {
        self.consistent
            && other.consistent
            && other
                .first
                .iter()
                .all(|(key, value)| self.first.get(key).is_none_or(|first| first == value))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.first.is_empty() && self.consistent
    }

    pub(crate) fn is_consistent(&self) -> bool {
        self.consistent
    }

    pub(crate) fn matches(&self, fs: &dyn FileSystem) -> bool {
        if fs.input_epoch_valid() == Some(false) {
            return false;
        }
        #[cfg(test)]
        let _work =
            crate::reference_records::request_work_tests::scope("history_replay", self.first.len());
        self.consistent
            && self
                .first
                .iter()
                .all(|((path, _), value)| value.matches(fs, path))
            && fs.input_epoch_valid() != Some(false)
    }

    pub(crate) fn owns_read(&self, path: &Path, owner: &Arc<ReadBuffer>) -> bool {
        self.owners
            .get(path.as_os_str())
            .is_some_and(|actual| Arc::ptr_eq(actual, owner))
    }

    pub(crate) fn retained_bytes(&self) -> usize {
        let owner_bytes: usize = self.owners.iter().map(|(path, owner)| {
            let shared = matches!(self.first.get(&(PathBuf::from(path), 0)), Some(Observation::Read(Some(bytes))) if Arc::ptr_eq(bytes, owner.bytes()));
            path.capacity() + std::mem::size_of::<(std::ffi::OsString, Arc<ReadBuffer>)>() + 64
                + std::mem::size_of::<ReadBuffer>() + 64
                + if shared { 0 } else { owner.len() }
        }).sum::<usize>();
        owner_bytes
            + self
                .first
                .iter()
                .map(|((path, _), value)| {
                    path.capacity()
                        + std::mem::size_of::<((PathBuf, u8), Observation)>()
                        + value.bytes()
                        + 64
                })
                .sum::<usize>()
    }
}

/// Capture actual base responses during configuration/discovery without changing precedence.
pub(crate) struct ObservedFs<'a> {
    pub(crate) base: &'a dyn FileSystem,
    pub(crate) proof: Option<&'a std::cell::RefCell<Observations>>,
    pub(crate) secondary: Option<&'a std::cell::RefCell<Observations>>,
}

impl ObservedFs<'_> {
    fn record(&self, path: &Path, value: Observation) {
        if self.base.tracks_native_inputs()
            && let Some(proof) = self.proof
        {
            proof.borrow_mut().record(path.to_path_buf(), value.clone());
        }
        if self.base.tracks_native_inputs()
            && let Some(proof) = self.secondary
        {
            proof.borrow_mut().record(path.to_path_buf(), value);
        }
    }
}

impl FileSystem for ObservedFs<'_> {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        if !self.base.tracks_native_inputs() {
            return self.base.read(path);
        }
        self.read_shared(path).map(|bytes| bytes.to_vec())
    }

    fn read_shared(&self, path: &Path) -> Option<Arc<ReadBuffer>> {
        let bytes = self.base.read_shared(path);
        self.record(
            path,
            Observation::Read(bytes.as_ref().map(|bytes| bytes.bytes().clone())),
        );
        if self.base.tracks_native_inputs()
            && let Some(owner) = &bytes
        {
            for proof in [self.proof, self.secondary].into_iter().flatten() {
                proof
                    .borrow_mut()
                    .record_read_owner(path.to_path_buf(), owner.clone());
            }
        }
        bytes
    }

    fn is_file(&self, path: &Path) -> bool {
        let value = self.base.is_file(path);
        self.record(path, Observation::File(value));
        value
    }

    fn is_dir(&self, path: &Path) -> bool {
        let value = self.base.is_dir(path);
        self.record(path, Observation::Directory(value));
        value
    }

    fn canonical(&self, path: &Path) -> PathBuf {
        #[cfg(test)]
        crate::reference_records::request_work_tests::outside_identity(path);
        let value = self.base.canonical(path);
        self.record(
            path,
            Observation::Canonical(jaic::paths::identity_spelling(&value)),
        );
        value
    }

    fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
        let value = self.base.canonical_exact(path);
        self.record(path, Observation::ExactCanonical(value.clone()));
        value
    }

    fn missing_file_key(&self, path: &Path) -> Option<PathBuf> {
        let value = self.base.missing_file_key(path);
        if let Some(key) = &value {
            self.record(path, Observation::Absence(key.clone()));
        }
        value
    }

    fn is_native(&self) -> bool {
        self.base.is_native()
    }

    fn tracks_native_inputs(&self) -> bool {
        self.base.tracks_native_inputs()
    }

    fn input_epoch_valid(&self) -> Option<bool> {
        self.base.input_epoch_valid()
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        let value = self.base.list_dir(path);
        let mut sorted = value.clone();
        sorted.sort();
        self.record(path, Observation::Listing(sorted));
        value
    }

    fn stamp(&self, path: &Path) -> Option<u64> {
        let value = self.base.stamp(path);
        self.record(path, Observation::Stamp(value));
        value
    }
}

/// An owned form of the existing observer for a between-request capture transaction.
/// It preserves actual return spelling/order and retains the exact shared read owner.
pub(crate) struct OwnedObservedFs {
    pub(crate) base: std::rc::Rc<dyn FileSystem>,
    pub(crate) proof: std::rc::Rc<std::cell::RefCell<Observations>>,
}

impl OwnedObservedFs {
    fn observer(&self) -> ObservedFs<'_> {
        ObservedFs {
            base: &*self.base,
            proof: Some(&self.proof),
            secondary: None,
        }
    }
}

impl FileSystem for OwnedObservedFs {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        self.observer().read(path)
    }

    fn read_shared(&self, path: &Path) -> Option<Arc<ReadBuffer>> {
        self.observer().read_shared(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.observer().is_file(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.observer().is_dir(path)
    }

    fn canonical(&self, path: &Path) -> PathBuf {
        self.observer().canonical(path)
    }

    fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
        self.observer().canonical_exact(path)
    }

    fn missing_file_key(&self, path: &Path) -> Option<PathBuf> {
        self.observer().missing_file_key(path)
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        self.observer().list_dir(path)
    }

    fn stamp(&self, path: &Path) -> Option<u64> {
        self.observer().stamp(path)
    }

    fn is_native(&self) -> bool {
        false
    }

    fn tracks_native_inputs(&self) -> bool {
        self.base.tracks_native_inputs()
    }

    fn input_epoch_valid(&self) -> Option<bool> {
        self.base.input_epoch_valid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SharedFs(Arc<ReadBuffer>);

    impl FileSystem for SharedFs {
        fn read(&self, _: &Path) -> Option<Vec<u8>> {
            Some(self.0.to_vec())
        }

        fn read_shared(&self, _: &Path) -> Option<Arc<ReadBuffer>> {
            Some(self.0.clone())
        }

        fn is_file(&self, _: &Path) -> bool {
            true
        }

        fn is_dir(&self, _: &Path) -> bool {
            false
        }

        fn is_native(&self) -> bool {
            true
        }
    }

    #[test]
    fn observed_shared_read_keeps_the_actual_arc_in_its_proof() {
        let bytes: Arc<[u8]> = Arc::from(&b"shared input"[..]);
        let base = SharedFs(Arc::new(ReadBuffer::new(bytes.clone())));
        let proof = std::cell::RefCell::new(Observations::default());
        let fs = ObservedFs {
            base: &base,
            proof: Some(&proof),
            secondary: None,
        };
        let path = Path::new("/shared/data");
        let returned = fs.read_shared(path).unwrap();
        assert!(Arc::ptr_eq(&bytes, returned.bytes()));
        let proof = proof.borrow();
        let Observation::Read(Some(owned)) = &proof.first[&(path.to_path_buf(), 0)] else {
            panic!("read proof missing")
        };
        assert!(
            Arc::ptr_eq(returned.bytes(), owned),
            "proof must retain the exact returned allocation"
        );
    }

    #[test]
    fn overlay_shared_read_keeps_native_and_overlay_ownership() {
        use std::{cell::RefCell, rc::Rc};
        let bytes: Arc<[u8]> = Arc::from(&b"native input"[..]);
        let open: Arc<[u8]> = Arc::from(&b"open input"[..]);
        let generated: Arc<[u8]> = Arc::from(&b"generated input"[..]);
        let path = PathBuf::from("/shared/data");
        let reads = Rc::new(RefCell::new(super::super::Reads::default()));
        let mut fs = super::super::overlay::OverlayFs {
            base: Rc::new(SharedFs(Arc::new(ReadBuffer::new(bytes.clone())))),
            dir: PathBuf::from("/shared"),
            files: RefCell::default(),
            generated: BTreeMap::new(),
            canonical: jaic::paths::CanonCache::default(),
            reads: reads.clone(),
        };
        let returned = fs.read_shared(Path::new("data")).unwrap();
        assert!(Arc::ptr_eq(&bytes, returned.bytes()));
        {
            let proof = reads.borrow();
            let Observation::Read(Some(owned)) = &proof.disk.first[&(path.clone(), 0)] else {
                panic!("read proof missing")
            };
            assert!(Arc::ptr_eq(returned.bytes(), owned));
        }
        fs.files.borrow_mut().insert(path.clone(), open.clone());
        assert!(Arc::ptr_eq(
            fs.read_shared(Path::new("data")).unwrap().bytes(),
            &open
        ));
        fs.generated.insert(path, generated.clone());
        assert!(Arc::ptr_eq(
            fs.read_shared(Path::new("data")).unwrap().bytes(),
            &generated
        ));
    }

    #[test]
    fn discovery_proxy_preserves_missing_exact_canonical_answers() {
        let temp = test_support::TempDir::new("exact-canonical-proof");
        let path = temp.path().join("missing.jai");
        let proof = std::cell::RefCell::default();
        let fs = ObservedFs {
            base: &jaic::sema::NativeFs,
            proof: Some(&proof),
            secondary: None,
        };
        assert_eq!(fs.canonical_exact(&path), None);
        assert!(proof.borrow().matches(&jaic::sema::NativeFs));
        std::fs::write(&path, "").unwrap();
        assert!(!proof.borrow().matches(&jaic::sema::NativeFs));
    }

    #[test]
    fn absence_proof_accounts_for_its_owned_fallback_key() {
        let mut key = PathBuf::with_capacity(2 * 1024 * 1024);
        key.push("/absent");
        let allocated = key.capacity();
        let mut proof = Observations::default();
        proof.record(PathBuf::from("/observed"), Observation::Absence(key));
        assert!(
            proof.retained_bytes() >= allocated,
            "compiler and retained-record budgets must include the owned absence key"
        );
    }
}
