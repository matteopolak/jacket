//! A machine described by a table, for tests of the lookups. Nothing here touches the real one.
use crate::host::{Host, Os};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// A host built from the files it has, the folders of its `PATH` and what its tools print.
pub struct FakeHost {
    pub os: Os,
    pub arch: String,
    pub path: Vec<PathBuf>,
    pub program_files: Vec<PathBuf>,
    pub cache: Option<PathBuf>,
    files: BTreeSet<PathBuf>,
    dirs: BTreeSet<PathBuf>,
    output: BTreeMap<(PathBuf, String), String>,
}

impl FakeHost {
    pub fn new(os: Os, arch: &str) -> FakeHost {
        FakeHost {
            os,
            arch: arch.to_string(),
            path: Vec::new(),
            program_files: Vec::new(),
            cache: Some(PathBuf::from("/cache")),
            files: BTreeSet::new(),
            dirs: BTreeSet::new(),
            output: BTreeMap::new(),
        }
    }

    pub fn mac() -> FakeHost {
        let mut host = FakeHost::new(Os::MacOs, "aarch64");
        host.path = vec!["/usr/bin".into(), "/opt/homebrew/bin".into()];
        host.cache = Some("/cache/mac".into());
        host.file("/usr/bin/cc");
        host
    }

    pub fn debian() -> FakeHost {
        let mut host = FakeHost::new(Os::Linux, "x86_64");
        host.path = vec!["/usr/bin".into()];
        host.cache = Some("/cache/linux".into());
        host.file("/usr/bin/cc");
        host.file("/usr/bin/apt-get");
        host
    }

    /// Adds a file, and the folders above it.
    pub fn file(&mut self, path: &str) -> &mut Self {
        let path = PathBuf::from(path);
        for ancestor in path.ancestors().skip(1) {
            self.dirs.insert(ancestor.to_path_buf());
        }
        self.files.insert(path);
        self
    }

    pub fn dir(&mut self, path: &str) -> &mut Self {
        for ancestor in Path::new(path).ancestors() {
            self.dirs.insert(ancestor.to_path_buf());
        }
        self
    }

    /// What `program args` prints.
    pub fn prints(&mut self, program: &str, args: &[&str], text: &str) -> &mut Self {
        self.output
            .insert((PathBuf::from(program), args.join(" ")), text.to_string());
        self
    }

    pub fn without(&mut self, path: &str) -> &mut Self {
        self.files.remove(Path::new(path));
        self
    }
}

impl Host for FakeHost {
    fn os(&self) -> Os {
        self.os
    }

    fn arch(&self) -> &str {
        &self.arch
    }

    fn path_dirs(&self) -> Vec<PathBuf> {
        self.path.clone()
    }

    fn program_files(&self) -> Vec<PathBuf> {
        self.program_files.clone()
    }

    fn cache_root(&self) -> Option<PathBuf> {
        self.cache.clone()
    }

    fn is_file(&self, path: &Path) -> bool {
        self.files.contains(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.dirs.contains(path)
    }

    fn read_dir(&self, path: &Path) -> Vec<PathBuf> {
        self.files
            .iter()
            .chain(&self.dirs)
            .filter(|entry| entry.parent() == Some(path))
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    fn run(&self, program: &Path, args: &[&str]) -> Option<String> {
        self.output
            .get(&(program.to_path_buf(), args.join(" ")))
            .cloned()
    }
}
