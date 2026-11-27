//! A machine described by a table, for tests of the lookups. Nothing here touches the real one.
use crate::host::{Host, Os};
use jacket_config::env::Var;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
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
    output: BTreeMap<(PathBuf, String), (bool, String)>,
    visual_studio: BTreeMap<(String, String), PathBuf>,
    texts: BTreeMap<PathBuf, String>,
    links: BTreeMap<PathBuf, PathBuf>,
    vars: BTreeMap<&'static str, OsString>,
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
            visual_studio: BTreeMap::new(),
            texts: BTreeMap::new(),
            links: BTreeMap::new(),
            vars: BTreeMap::new(),
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

    /// What `program args` prints when it succeeds.
    pub fn prints(&mut self, program: &str, args: &[&str], text: &str) -> &mut Self {
        self.output.insert(
            (PathBuf::from(program), args.join(" ")),
            (true, text.to_string()),
        );
        self
    }

    /// What `program args` prints when it fails.
    pub fn fails_printing(&mut self, program: &str, args: &[&str], text: &str) -> &mut Self {
        self.output.insert(
            (PathBuf::from(program), args.join(" ")),
            (false, text.to_string()),
        );
        self
    }

    /// A Visual Studio installation's `tool` for the CPU folder `arch`.
    pub fn visual_studio(&mut self, tool: &str, arch: &str, path: &str) -> &mut Self {
        self.file(path);
        self.visual_studio
            .insert((tool.to_string(), arch.to_string()), PathBuf::from(path));
        self
    }

    /// Adds a file holding `text`.
    pub fn text(&mut self, path: &str, text: &str) -> &mut Self {
        self.file(path);
        self.texts.insert(PathBuf::from(path), text.to_string());
        self
    }

    /// Adds a symbolic link at `path` to `target`.
    pub fn link(&mut self, path: &str, target: &str) -> &mut Self {
        self.links
            .insert(PathBuf::from(path), PathBuf::from(target));
        self
    }

    /// Sets an environment variable.
    pub fn set(&mut self, var: &Var, value: &str) -> &mut Self {
        self.vars.insert(var.name, value.into());
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

    fn read_to_string(&self, path: &Path) -> Option<String> {
        self.texts.get(path).cloned()
    }

    fn read_link(&self, path: &Path) -> Option<PathBuf> {
        self.links.get(path).cloned()
    }

    fn var(&self, var: &Var) -> Option<OsString> {
        self.vars.get(var.name).cloned()
    }

    fn output(&self, program: &Path, args: &[&str]) -> Option<(bool, String)> {
        self.output
            .get(&(program.to_path_buf(), args.join(" ")))
            .cloned()
    }

    /// `path` with `.` and `..` folded away, when it names a file or folder of the host. The
    /// fake host has no links of folders, so that is its canonical form.
    fn canonicalize(&self, path: &Path) -> Option<PathBuf> {
        let mut folded = PathBuf::new();
        for part in path.components() {
            match part {
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    folded.pop();
                }
                other => folded.push(other),
            }
        }
        (self.is_file(&folded) || self.is_dir(&folded)).then_some(folded)
    }

    fn visual_studio_tool(&self, tool: &str, arch: &str) -> Option<PathBuf> {
        self.visual_studio
            .get(&(tool.to_string(), arch.to_string()))
            .cloned()
    }
}
