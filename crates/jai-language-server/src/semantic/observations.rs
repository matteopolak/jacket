//! Exact native filesystem responses captured by one accepted semantic context.
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
        }
    }

    fn matches(&self, fs: &dyn FileSystem, path: &Path) -> bool {
        match self {
            Self::Read(bytes) => fs.read(path).as_deref() == bytes.as_deref(),
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
        }
    }

    fn bytes(&self) -> usize {
        match self {
            Self::Read(bytes) => bytes.as_ref().map_or(0, |b| b.len()),
            Self::Canonical(path) => path.capacity(),
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
    consistent: bool,
}

impl Default for Observations {
    fn default() -> Self {
        Self {
            first: BTreeMap::new(),
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
        self.consistent = true;
        changed.into_iter().collect()
    }

    pub(crate) fn merge(&mut self, other: &Self) {
        self.consistent &= other.consistent;
        for ((path, _), value) in &other.first {
            self.record(path.clone(), value.clone());
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
        #[cfg(test)]
        let _work =
            crate::reference_records::request_work_tests::scope("history_replay", self.first.len());
        self.consistent
            && self
                .first
                .iter()
                .all(|((path, _), value)| value.matches(fs, path))
    }

    pub(crate) fn retained_bytes(&self) -> usize {
        self.first
            .iter()
            .map(|((path, _), value)| {
                path.capacity()
                    + std::mem::size_of::<((PathBuf, u8), Observation)>()
                    + value.bytes()
                    + 64
            })
            .sum()
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
        if self.base.is_native()
            && let Some(proof) = self.proof
        {
            proof.borrow_mut().record(path.to_path_buf(), value.clone());
        }
        if self.base.is_native()
            && let Some(proof) = self.secondary
        {
            proof.borrow_mut().record(path.to_path_buf(), value);
        }
    }
}

impl FileSystem for ObservedFs<'_> {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        let bytes = self.base.read(path);
        self.record(path, Observation::Read(bytes.as_deref().map(Arc::from)));
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

    fn is_native(&self) -> bool {
        self.base.is_native()
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
