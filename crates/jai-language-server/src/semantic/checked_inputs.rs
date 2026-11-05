//! Immutable input authority for compiler-independent exact records.
use super::{Analysis, Buffers, Environment, Reads};
use std::{collections::BTreeMap, path::PathBuf, rc::Rc, sync::Arc};

#[derive(Clone)]
pub(crate) struct CheckedInputs {
    authority: Rc<()>,
    pub(crate) root: PathBuf,
    pub(crate) dirs: Vec<PathBuf>,
    pub(crate) os: Option<jaic::target::TargetOs>,
    pub(crate) reads: Rc<Reads>,
    overlays: BTreeMap<PathBuf, Arc<[u8]>>,
    pub(crate) bytes: usize,
}

impl CheckedInputs {
    pub(crate) fn capture(analysis: &Analysis) -> Self {
        Self::capture_reads(analysis, Rc::new(analysis.reads.borrow().clone()))
    }

    /// Freeze separate derived-output observations without changing compiler input authority.
    fn capture_reads(analysis: &Analysis, reads: Rc<Reads>) -> Self {
        Self::capture_with_files(analysis, reads, &analysis.files)
    }

    /// Use only within the same live query that captured `files`; this does not promote
    /// another compiler's old context into current retained authority.
    pub(super) fn capture_with_files(
        analysis: &Analysis,
        reads: Rc<Reads>,
        files: &BTreeMap<PathBuf, Arc<[u8]>>,
    ) -> Self {
        let overlays: BTreeMap<_, _> = files
            .iter()
            .filter(|(path, _)| reads.concerns(path))
            .map(|(path, bytes)| (path.clone(), bytes.clone()))
            .collect();
        let bytes = analysis.root.capacity()
            + analysis.dirs.capacity() * std::mem::size_of::<PathBuf>()
            + analysis.dirs.iter().map(PathBuf::capacity).sum::<usize>()
            + reads.retained_bytes()
            + std::mem::size_of::<Self>()
            + overlays
                .iter()
                .map(|(path, bytes)| {
                    path.capacity() + bytes.len() + std::mem::size_of::<(PathBuf, Arc<[u8]>)>() + 64
                })
                .sum::<usize>();
        Self {
            authority: analysis.authority.clone(),
            root: analysis.root.clone(),
            dirs: analysis.dirs.clone(),
            os: analysis.os,
            reads,
            overlays,
            bytes,
        }
    }

    /// Selected analyses already own canonical overlay keys.
    pub(super) fn matches_selected(
        &self,
        analysis: &Analysis,
        fs: &dyn jaic::sema::FileSystem,
    ) -> bool {
        Rc::ptr_eq(&self.authority, &analysis.authority)
            && self.reads.is_current(fs)
            && analysis
                .files
                .iter()
                .filter(|(path, _)| self.reads.concerns(path))
                .eq(self.overlays.iter())
    }

    pub(crate) fn is_current(
        &self,
        env: &Environment,
        files: &Buffers,
        authority: &Rc<()>,
    ) -> bool {
        self.is_current_except(env, files, authority, None)
    }

    pub(crate) fn is_current_except(
        &self,
        env: &Environment,
        files: &Buffers,
        authority: &Rc<()>,
        ignored: Option<&std::path::Path>,
    ) -> bool {
        self.is_current_projected(
            env,
            files,
            authority,
            ignored,
            &super::Observations::default(),
        )
    }

    /// Validate retained authority together with this answer's actual projection history.
    /// Example: a call graph stages target URI identities before replaying the union.
    pub(crate) fn is_current_projected(
        &self,
        env: &Environment,
        files: &Buffers,
        authority: &Rc<()>,
        ignored: Option<&std::path::Path>,
        output: &super::Observations,
    ) -> bool {
        self.projected_reads(env, files, authority, ignored, output)
            .is_some_and(|reads| reads.is_current(&*env.fs))
    }

    /// Stage one retained context for a combined answer's single final native replay.
    /// No accepted authority is changed; the returned history still requires validation.
    pub(crate) fn projected_reads(
        &self,
        env: &Environment,
        files: &Buffers,
        authority: &Rc<()>,
        ignored: Option<&std::path::Path>,
        output: &super::Observations,
    ) -> Option<Reads> {
        if !Rc::ptr_eq(&self.authority, authority) {
            return None;
        }
        // Identity projection is part of this derived answer, not work after acceptance.
        // Preserve the retained first responses and add only actual current mapping reads.
        let projection = std::cell::RefCell::new(super::Observations::default());
        let fs = super::ObservedFs {
            base: &*env.fs,
            proof: Some(&projection),
            secondary: None,
        };
        let ignored = ignored.map(|path| jaic::paths::file_identity(&fs, path));
        let ignored = ignored.as_deref();
        let current: BTreeMap<_, _> = files
            .iter()
            .map(|(path, bytes)| (jaic::paths::file_identity(&fs, path), bytes.clone()))
            .filter(|(path, _)| self.reads.concerns(path))
            .collect();
        let overlays_match = current
            .iter()
            .filter(|(path, _)| Some(path.as_path()) != ignored)
            .eq(self
                .overlays
                .iter()
                .filter(|(path, _)| Some(path.as_path()) != ignored));
        let mut derived = (*self.reads).clone();
        derived.disk.merge(output);
        derived.disk.merge(&projection.into_inner());
        overlays_match.then_some(derived)
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use jaic::sema::{FileSystem, NativeFs};
    use std::{cell::Cell, path::Path};

    struct RetargetFs {
        alias: PathBuf,
        second: PathBuf,
        armed: Cell<bool>,
    }

    impl FileSystem for RetargetFs {
        fn is_native(&self) -> bool {
            true
        }

        fn read(&self, path: &Path) -> Option<Vec<u8>> {
            NativeFs.read(path)
        }

        fn is_file(&self, path: &Path) -> bool {
            NativeFs.is_file(path)
        }

        fn is_dir(&self, path: &Path) -> bool {
            NativeFs.is_dir(path)
        }

        fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
            NativeFs.list_dir(path)
        }

        fn stamp(&self, path: &Path) -> Option<u64> {
            NativeFs.stamp(path)
        }

        fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
            NativeFs.canonical_exact(path)
        }

        fn canonical(&self, path: &Path) -> PathBuf {
            let answer = NativeFs.canonical(path);
            if path == self.alias && self.armed.replace(false) {
                std::fs::remove_file(&self.alias).unwrap();
                std::os::unix::fs::symlink(&self.second, &self.alias).unwrap();
            }
            answer
        }
    }

    #[test]
    fn ignored_alias_mapping_is_inside_the_retained_input_guard() {
        let temp = test_support::TempDir::new("checked-input-projection");
        let first = temp.path().join("first");
        let second = temp.path().join("second");
        let alias = temp.path().join("alias");
        std::fs::write(&first, "first").unwrap();
        std::fs::write(&second, "second").unwrap();
        std::os::unix::fs::symlink(&first, &alias).unwrap();
        let fs = Rc::new(RetargetFs {
            alias: alias.clone(),
            second,
            armed: Cell::new(false),
        });
        let env = Environment {
            fs: fs.clone(),
            services: None,
            options: Box::new(|_, _| jaic::sema::Options::host()),
        };
        let authority = Rc::new(());
        let mut reads = Reads::default();
        reads.observe_canonical(alias.clone(), jaic::paths::file_identity(&NativeFs, &alias));
        let checked = CheckedInputs {
            authority: authority.clone(),
            root: first,
            dirs: Vec::new(),
            os: None,
            reads: Rc::new(reads),
            overlays: BTreeMap::new(),
            bytes: 0,
        };
        let files = Rc::new(BTreeMap::new());
        assert!(checked.is_current_except(&env, &files, &authority, Some(&alias)));
        fs.armed.set(true);
        assert!(
            !checked.is_current_except(&env, &files, &authority, Some(&alias)),
            "mapping must not run after the final proof replay"
        );
        assert!(
            !fs.armed.get(),
            "the actual alias response must trigger the retarget"
        );
    }
}
