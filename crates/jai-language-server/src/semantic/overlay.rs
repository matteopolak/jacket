//! A file system that overlays open buffers on disk.
use super::observations::{Observation, Observations};
use jaic::sema::FileSystem;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

/// Every path a compile asked its file system about, by canonical key: files read or tested,
/// folders tested. Compile-time code reads through the same file system, so what `#run` read
/// is here too. A buffer that is none of these, and lies in none of these folders, cannot
/// change the compile.
#[derive(Clone, Debug, Default)]
pub struct Reads {
    files: BTreeSet<PathBuf>,
    dirs: BTreeSet<PathBuf>,
    identities: BTreeMap<PathBuf, PathBuf>,
    pub(crate) disk: Observations,
}

impl Reads {
    /// Exact native observations, independent of notifications and metadata fingerprints.
    pub(crate) fn is_current(&self, fs: &dyn FileSystem) -> bool {
        !fs.is_native() || self.disk.matches(fs)
    }

    pub(super) fn observe_canonical(&mut self, path: PathBuf, target: PathBuf) {
        #[cfg(test)]
        crate::reference_records::request_work_tests::outside_identity(&path);
        self.disk.record(path, Observation::Canonical(target));
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.files.is_empty()
            && self.dirs.is_empty()
            && self.identities.is_empty()
            && self.disk.is_empty()
    }

    /// Combine exact histories in an owned input or derived-output proof without changing their first responses.
    pub(crate) fn merge(&mut self, other: &Self) {
        self.files.extend(other.files.iter().cloned());
        self.dirs.extend(other.dirs.iter().cloned());
        self.identities.extend(
            other
                .identities
                .iter()
                .map(|(name, target)| (name.clone(), target.clone())),
        );
        self.disk.merge(&other.disk);
    }

    /// Whether a file appearing, going away or changing could affect the compile.
    pub fn concerns(&self, path: &Path) -> bool {
        self.directly_concerns(path)
            || self
                .identities
                .get(path)
                .is_some_and(|target| self.directly_concerns_tree(target))
    }

    fn directly_concerns(&self, path: &Path) -> bool {
        self.files.contains(path) || path.ancestors().any(|dir| self.dirs.contains(dir))
    }

    fn directly_concerns_tree(&self, path: &Path) -> bool {
        self.directly_concerns(path)
            || self
                .files
                .range(path.to_path_buf()..)
                .next()
                .is_some_and(|read| read.starts_with(path))
            || self
                .dirs
                .range(path.to_path_buf()..)
                .next()
                .is_some_and(|read| read.starts_with(path))
    }

    /// Watched directory removal also affects any observed descendant. Keep this search
    /// outside ordinary overlay comparisons, where only the changed file matters.
    pub(crate) fn concerns_tree(&self, path: &Path) -> bool {
        self.directly_concerns_tree(path)
            || self
                .identities
                .range(path.to_path_buf()..)
                .take_while(|(name, _)| name.starts_with(path))
                .any(|(_, target)| self.directly_concerns_tree(target))
    }

    /// Conservative retained dependency storage, including tree-node metadata.
    pub(crate) fn retained_bytes(&self) -> usize {
        self.disk.retained_bytes()
            + self
                .files
                .iter()
                .chain(&self.dirs)
                .map(|path| path.capacity() + std::mem::size_of::<PathBuf>() + 64)
                .sum::<usize>()
            + self
                .identities
                .iter()
                .map(|(name, target)| {
                    name.capacity() + target.capacity() + 2 * std::mem::size_of::<PathBuf>() + 64
                })
                .sum::<usize>()
    }
}

/// Open documents over the environment's file system.
pub(super) struct OverlayFs {
    pub(super) base: Rc<dyn FileSystem>,
    /// Where relative paths start (the root file's folder): the server's own working directory
    /// is wherever the editor started it, and a build metaprogram names its files and module
    /// folders relative to itself.
    pub(super) dir: PathBuf,
    pub(super) files: RefCell<BTreeMap<PathBuf, Arc<[u8]>>>,
    /// Synthetic program roots belong to the compile, independently of current open overlays.
    pub(super) generated: BTreeMap<PathBuf, Arc<[u8]>>,
    /// `canonical` answers so far: a compile asks about the same paths over and over.
    pub(super) canonical: jaic::paths::CanonCache,
    /// What was asked so far; shared with the analysis, since queries after the compile can
    /// load more.
    pub(super) reads: Rc<RefCell<Reads>>,
}

impl OverlayFs {
    /// An isolated observer over the same selected inputs; reads never enter compiler proof.
    pub(super) fn separate_observer(&self) -> Self {
        Self {
            base: self.base.clone(),
            dir: self.dir.clone(),
            files: RefCell::new(self.files.borrow().clone()),
            generated: self.generated.clone(),
            canonical: jaic::paths::CanonCache::default(),
            reads: Rc::default(),
        }
    }

    /// Called only after checked dependencies prove that changed buffers are unrelated.
    pub(super) fn replace_files(&self, files: BTreeMap<PathBuf, Arc<[u8]>>) {
        *self.files.borrow_mut() = files;
    }

    pub(super) fn forget_paths(&self) {
        self.canonical.clear();
    }


    fn at<'a>(&self, path: &'a Path) -> std::borrow::Cow<'a, Path> {
        if path.is_relative() {
            self.dir.join(path).into()
        } else {
            path.into()
        }
    }

    /// Keep the observed leaf name as well as its resolved target: removing or retargeting a
    /// file symlink changes a load even when the old target still exists.
    fn observed_name(&self, path: &Path) -> PathBuf {
        let absolute = self.at(path);
        match (absolute.parent(), absolute.file_name()) {
            (Some(parent), Some(name)) => self.canonical_of(parent).join(name),
            _ => self.canonical_of(path),
        }
    }

    fn record_file(&self, path: &Path, key: PathBuf) {
        self.observe_identity(path);
        let name = self.observed_name(path);
        let mut reads = self.reads.borrow_mut();
        reads.files.insert(key);
        reads.files.insert(name);
    }

    fn record_directory(&self, path: &Path, key: PathBuf) {
        self.observe_identity(path);
        let name = self.observed_name(path);
        let mut reads = self.reads.borrow_mut();
        reads.dirs.insert(key);
        reads.dirs.insert(name);
    }

    fn observe_identity(&self, path: &Path) -> PathBuf {
        #[cfg(test)]
        crate::reference_records::request_work_tests::overlay_identity(|| {
            (self.at(path).into_owned(), false)
        });
        let known = self.canonical.get(path);
        let key = known.clone().unwrap_or_else(|| self.canonical_of(path));
        let absolute = self.at(path);
        if known.is_none() {
            // The loader may resolve a symlink before reading its target. Retain those names,
            // but invalidate only when the resolved target was actually observed by the check.
            for name in absolute.ancestors() {
                let target = self.canonical_of(name);
                let leaf = self.observed_name(name);
                let mut reads = self.reads.borrow_mut();
                reads.identities.insert(name.to_path_buf(), target.clone());
                reads.identities.insert(leaf, target);
            }
        }
        key
    }

    fn observe(&self, path: &Path, value: Observation) {
        if self.base.is_native() {
            self.reads
                .borrow_mut()
                .disk
                .record(self.at(path).into_owned(), value);
        }
    }

    fn canonical_of(&self, path: &Path) -> PathBuf {
        #[cfg(test)]
        crate::reference_records::request_work_tests::overlay_identity(|| {
            (self.at(path).into_owned(), false)
        });
        self.canonical.get_or_insert_with(path, |path| {
            let resolved = jaic::paths::file_identity(&*self.base, &self.at(path));
            self.observe(path, Observation::Canonical(resolved.clone()));
            resolved
        })
    }
}

impl FileSystem for OverlayFs {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        #[cfg(test)]
        let _work = crate::reference_records::request_work_tests::overlay_scope("read");
        let key = self.observe_identity(path);
        let found = self
            .generated
            .get(&key)
            .cloned()
            .or_else(|| self.files.borrow().get(&key).cloned());
        self.record_file(path, key);
        if let Some(bytes) = found {
            Some(bytes.to_vec())
        } else {
            let bytes = self.base.read(&self.at(path));
            self.observe(path, Observation::Read(bytes.as_deref().map(Arc::from)));
            bytes
        }
    }

    fn is_file(&self, path: &Path) -> bool {
        #[cfg(test)]
        let _work = crate::reference_records::request_work_tests::overlay_scope("file");
        #[cfg(test)]
        crate::reference_records::request_work_tests::overlay_identity(|| {
            (
                self.at(path).into_owned(),
                self.canonical.get(path).is_none(),
            )
        });
        // A cached identity can still select an open file after its disk alias disappears.
        // Only a new name may use exact absence, and an unsaved/generated fallback wins.
        if self.base.is_native()
            && self.canonical.get(path).is_none()
            && let Some(key) = self.base.missing_file_key(&self.at(path))
            && !self.generated.contains_key(&key)
            && !self.files.borrow().contains_key(&key)
        {
            self.observe(path, Observation::Absence(key.clone()));
            let raw = self.at(path).into_owned();
            let mut reads = self.reads.borrow_mut();
            reads.files.insert(raw);
            reads.files.insert(key);
            return false;
        }
        let key = self.observe_identity(path);
        let open = self.generated.contains_key(&key) || self.files.borrow().contains_key(&key);
        self.record_file(path, key);
        open || {
            let value = self.base.is_file(&self.at(path));
            self.observe(path, Observation::File(value));
            value
        }
    }

    fn is_dir(&self, path: &Path) -> bool {
        #[cfg(test)]
        let _work = crate::reference_records::request_work_tests::overlay_scope("directory");
        let p = self.observe_identity(path);
        self.record_directory(path, p.clone());
        self.files
            .borrow()
            .keys()
            .chain(self.generated.keys())
            .any(|k| k.starts_with(&p) && *k != p)
            || {
                let value = self.base.is_dir(&self.at(path));
                self.observe(path, Observation::Directory(value));
                value
            }
    }

    fn canonical(&self, path: &Path) -> PathBuf {
        #[cfg(test)]
        let _work = crate::reference_records::request_work_tests::overlay_scope("canonical");
        self.observe_identity(path)
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        #[cfg(test)]
        let _work = crate::reference_records::request_work_tests::overlay_scope("listing");
        let key = self.observe_identity(path);
        self.record_directory(path, key.clone());
        let mut base = self.base.list_dir(&self.at(path));
        base.sort();
        self.observe(path, Observation::Listing(base.clone()));
        let mut entries: BTreeMap<String, bool> = base.into_iter().collect();
        // An open document is a file even before it is saved. Its ancestors are directories;
        // only the immediate child belongs in this listing. Exact overlays replace disk entries.
        for file in self.files.borrow().keys().chain(self.generated.keys()) {
            let Ok(relative) = file.strip_prefix(&key) else {
                continue;
            };
            let mut components = relative.components();
            let Some(std::path::Component::Normal(name)) = components.next() else {
                continue;
            };
            entries.insert(
                name.to_string_lossy().into_owned(),
                components.next().is_some(),
            );
        }
        entries.into_iter().collect()
    }

    fn stamp(&self, path: &Path) -> Option<u64> {
        #[cfg(test)]
        let _work = crate::reference_records::request_work_tests::overlay_scope("stamp");
        let key = self.observe_identity(path);
        let open = self
            .generated
            .get(&key)
            .cloned()
            .or_else(|| self.files.borrow().get(&key).cloned())
            .map(|bytes| jaic::sema::content_stamp(&bytes));
        if open.is_none() && self.is_dir(path) {
            use std::hash::{Hash, Hasher};
            let mut hash = jaic::fxhash::FxHasher::default();
            self.list_dir(path).hash(&mut hash);
            return Some(hash.finish());
        }
        // What is stamped counts as read: the compile it belongs to depends on it.
        self.record_file(path, key);
        if let Some(stamp) = open {
            Some(stamp)
        } else {
            let stamp = self.base.stamp(&self.at(path));
            self.observe(path, Observation::Stamp(stamp));
            stamp
        }
    }
}

#[cfg(test)]
#[path = "overlay_tests.rs"]
mod tests;
