use super::*;
use jaic::sema::{FileSystem, NativeFs};
use std::cell::Cell;

struct UnavailableRead {
    path: PathBuf,
    fail: Cell<bool>,
}

impl FileSystem for UnavailableRead {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        if path == self.path && self.fail.get() {
            None
        } else {
            NativeFs.read(path)
        }
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

    fn canonical(&self, path: &Path) -> PathBuf {
        NativeFs.canonical(path)
    }

    fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
        NativeFs.canonical_exact(path)
    }

    fn missing_file_key(&self, path: &Path) -> Option<PathBuf> {
        NativeFs.missing_file_key(path)
    }

    fn stamp(&self, path: &Path) -> Option<u64> {
        NativeFs.stamp(path)
    }

    fn is_native(&self) -> bool {
        true
    }
}

#[test]
fn advisory_absence_is_complete_but_unknown_read_does_not_commit_a_catalogue() {
    let temp = test_support::TempDir::new("advisory-absence-and-unknown-read");
    let main = temp.path().join("main.jai");
    let probe = temp.path().join("build.jai");
    std::fs::write(&main, "main :: () {}").unwrap();
    std::fs::write(&probe, "helper :: () {}").unwrap();
    let fs = Rc::new(UnavailableRead {
        path: probe.clone(),
        fail: Cell::new(true),
    });
    let mut session = Session::with_environment(
        crate::Limits::default(),
        Environment {
            fs: fs.clone(),
            services: None,
            options: Box::new(|_, _| {
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options
            }),
        },
    );
    let uri = crate::DocumentUri::from_path(&main).unwrap();
    session
        .open(uri.clone(), 1, "main :: () {}".into())
        .unwrap();
    let env = session.environment.as_ref().unwrap();
    let absent = session.advisory_sources(env);
    assert!(absent.read(&temp.path().join("absent.jai")).is_none());
    assert!(
        absent.accept_advisory(),
        "certified missing candidates are valid negative discovery"
    );
    let unknown = session.advisory_sources(env);
    assert!(unknown.read(&probe).is_none());
    assert!(
        !unknown.accept_advisory(),
        "an existing unreadable input is not certified absence"
    );
    let before = session.index.borrow().clone();
    session.warm_auto_import();
    let after = session.index.borrow();
    assert!(before.files.keys().eq(after.files.keys()));
    assert!(before.walked.keys().eq(after.walked.keys()));
    assert!(before.graphs.keys().eq(after.graphs.keys()));
    assert!(before.import_dirs.keys().eq(after.import_dirs.keys()));
    drop(after);
    fs.fail.set(false);
    let recovered = session.advisory_sources(env);
    assert_eq!(recovered.read(&probe).as_deref(), Some("helper :: () {}"));
    assert!(recovered.accept_advisory());
}
