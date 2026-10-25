//! A project file system backed by the index.
use super::{Index, Sources};
use crate::project::{self};
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// The sources and index of a request as the project walk's file system: scans are cached
/// in the index, open documents win over the disk, and a build metaprogram's files count as
/// loaded by it.
pub(super) struct IndexFs<'a, 's> {
    sources: &'a Sources<'s>,
    index: std::cell::RefCell<&'a mut Index>,
}

impl<'a, 's> IndexFs<'a, 's> {
    pub(super) fn new(sources: &'a Sources<'s>, index: &'a mut Index) -> Self {
        IndexFs {
            sources,
            index: std::cell::RefCell::new(index),
        }
    }
}

impl project::ProjectFs for IndexFs<'_, '_> {
    fn key(&self, path: &Path) -> PathBuf {
        self.sources.key(path)
    }

    fn read(&self, path: &Path) -> Option<String> {
        self.sources.read(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.sources.is_file(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.sources.is_dir(path)
    }

    fn list_dir(&self, dir: &Path) -> Vec<(String, bool)> {
        self.sources.list_dir(dir)
    }

    fn loads(&self, file: &Path) -> Vec<PathBuf> {
        let dir = file.parent().unwrap_or(Path::new("/"));
        let scan = self.sources.scan(&mut self.index.borrow_mut(), file);
        scan.loads
            .iter()
            .map(|load| self.sources.key(&project::normalize(&dir.join(&load.path))))
            .collect()
    }

    /// The loads, plus what a build metaprogram compiles belongs to its program. After a run,
    /// the files it added; before the first run (or without a compile), the literal paths of
    /// its `add_build_file` calls, relative to the metaprogram's folder, where
    /// `set_working_directory(#filepath)` usually puts it.
    fn edges(&self, file: &Path) -> Vec<PathBuf> {
        let mut out = self.loads(file);
        let dir = file.parent().unwrap_or(Path::new("/"));
        let mut index = self.index.borrow_mut();
        if let Some(facts) = index.builds.get(file) {
            self.sources.note_own();
            out.extend(facts.programs.iter().flat_map(|p| p.files.iter().cloned()));
        } else {
            let scan = self.sources.scan(&mut index, file);
            out.extend(
                scan.builds
                    .iter()
                    .map(|build| project::normalize(&dir.join(build))),
            );
        }
        out
    }

    fn project_files(&self, folder: &Path) -> Vec<PathBuf> {
        self.sources
            .project_files(&mut self.index.borrow_mut(), folder)
    }

    /// Kept until a file's `#load`s change (an edit that leaves them alone keeps it), so the
    /// program of a document is found without reading the whole tree again.
    fn loader_graph(&self, folder: &Path) -> Option<project::LoaderGraph> {
        let files = self.project_files(folder);
        if let Some((epoch, seen, graph)) = self.index.borrow().graphs.get(folder)
            && *epoch == self.index.borrow().graph_epoch
            && *seen == files
        {
            return Some(graph.clone());
        }
        let graph = Rc::new(project::build_loader_graph(self, &files));
        let mut index = self.index.borrow_mut();
        let epoch = index.graph_epoch;
        index
            .graphs
            .insert(folder.to_path_buf(), (epoch, files, graph.clone()));
        Some(graph)
    }
}
