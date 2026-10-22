//! Workspace-wide generic instances without retaining every compiler in memory.
use crate::{DocumentUri, Session};
use jaic::source::{FileId, Span};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

type Declaration = (PathBuf, u32, String);

#[derive(Clone, Default)]
struct Row {
    end: u32,
    instances: BTreeMap<String, (String, String)>,
}

struct Snapshot {
    dirs: Vec<PathBuf>,
    trusted: bool,
    checked_revision: u64,
    dependencies: BTreeMap<PathBuf, u64>,
    rows: BTreeMap<Declaration, Row>,
    complete: bool,
}

#[derive(Default)]
pub(crate) struct Index {
    programs: BTreeMap<PathBuf, Snapshot>,
}

impl Index {
    pub(crate) fn clear(&mut self) {
        self.programs.clear();
    }

    pub(crate) fn invalidate(&mut self, path: &Path, topology: bool) {
        // New files can satisfy a previously missing import; settings can change which
        // programs and dependencies exist. Ordinary edits invalidate only readers.
        if topology || path.extension().is_some_and(|e| e == "toml") {
            self.clear();
        } else {
            self.programs
                .retain(|_, p| !p.dependencies.contains_key(path));
        }
    }
}

impl Session {
    /// All current workspace programs contribute, regardless of the declaration's location.
    pub(crate) fn workspace_polymorphs(
        &self,
        uri: &DocumentUri,
    ) -> Vec<(Span, String, Vec<String>, bool)> {
        let Some(env) = &self.environment else {
            return Vec::new();
        };
        let sources = self.sources(env);
        let open: Vec<_> = self
            .documents
            .keys()
            .map(|u| PathBuf::from(u.path()))
            .collect();
        let mut roots =
            sources.workspace_entries(&mut self.index.borrow_mut(), &self.workspace_folders, &open);
        if roots.is_empty() {
            roots.insert(PathBuf::from(uri.path()));
        }
        let buffers = Rc::new(
            self.documents
                .iter()
                .map(|(u, d)| (PathBuf::from(u.path()), Rc::from(d.ide_text().as_bytes())))
                .collect(),
        );
        let path = sources.key(Path::new(uri.path()));
        let mut combined: BTreeMap<Declaration, Row> = BTreeMap::new();
        let mut complete = true;
        let mut used = std::collections::BTreeSet::new();
        for root in &roots {
            if self.index.borrow().builds.iter().any(|(build, facts)| {
                build != root
                    && roots.contains(build)
                    && facts.programs.iter().any(|p| p.files.contains(root))
            }) {
                continue;
            }
            used.insert(root.clone());
            let dirs = self.import_dirs(root);
            let valid = self
                .polymorph_index
                .borrow()
                .programs
                .get(root)
                .is_some_and(|p| {
                    p.dirs == dirs
                        && p.trusted == self.trusted()
                        && (p.checked_revision == self.semantic_revision()
                            || p.dependencies.iter().all(|(path, hash)| {
                                let text = self
                                    .open_file_uri(path)
                                    .and_then(|u| self.documents.get(&u))
                                    .map(|d| d.ide_text().into_owned())
                                    .or_else(|| sources.read(path));
                                text.is_some_and(|t| crate::auto_import::hash(&t) == *hash)
                            }))
                });
            if !valid {
                let trusted = self.trusted();
                let mut cache = self.semantic.borrow_mut();
                let analysis = cache.analyze(env, root, &dirs, Rc::clone(&buffers));
                let mut snapshot = Snapshot {
                    dirs,
                    trusted,
                    checked_revision: self.semantic_revision(),
                    dependencies: BTreeMap::new(),
                    rows: BTreeMap::new(),
                    complete: true,
                };
                for (compiler, ok) in analysis.polymorph_compilers() {
                    snapshot.complete &= ok;
                    for n in 0..compiler.src.sources.len() {
                        let file = compiler.src.sources.get(FileId(n as u32));
                        if let Some(path) = file.path.as_file() {
                            snapshot
                                .dependencies
                                .insert(sources.key(path), crate::auto_import::hash(&file.text));
                        }
                    }
                    for row in compiler.ide_polymorph_summaries() {
                        let Some(file) = compiler.src.sources.get(row.span.file).path.as_file()
                        else {
                            continue;
                        };
                        let at = snapshot
                            .rows
                            .entry((sources.key(file), row.span.start, row.name))
                            .or_default();
                        at.end = row.span.end;
                        at.instances.extend(
                            row.instances
                                .into_iter()
                                .map(|i| (i.identity, (i.bindings, i.detail))),
                        );
                    }
                }
                self.sync_builds(&cache);
                snapshot.dirs = self.import_dirs(root);
                self.polymorph_index
                    .borrow_mut()
                    .programs
                    .insert(root.clone(), snapshot);
            }
            self.polymorph_index
                .borrow_mut()
                .programs
                .get_mut(root)
                .unwrap()
                .checked_revision = self.semantic_revision();
        }
        // A metaprogram discovered later in the pass may own a root checked earlier.
        // Merge only after build discovery settles, so that standalone child's errors
        // and instances cannot leak into the workspace result.
        used.retain(|root| {
            !self.index.borrow().builds.iter().any(|(build, facts)| {
                build != root
                    && roots.contains(build)
                    && facts.programs.iter().any(|p| p.files.contains(root))
            })
        });
        for root in &used {
            let index = self.polymorph_index.borrow();
            let snapshot = &index.programs[root];
            complete &= snapshot.complete;
            for (key, row) in &snapshot.rows {
                if key.0 == path {
                    let at = combined.entry(key.clone()).or_default();
                    at.end = row.end;
                    at.instances.extend(row.instances.clone());
                }
            }
        }
        self.polymorph_index
            .borrow_mut()
            .programs
            .retain(|root, _| used.contains(root));
        combined
            .into_iter()
            .map(|((_, start, name), row)| {
                let mut counts = BTreeMap::<String, usize>::new();
                for (label, _) in row.instances.values() {
                    *counts.entry(label.clone()).or_default() += 1;
                }
                let mut values: Vec<_> = row
                    .instances
                    .into_values()
                    .map(|(label, detail)| {
                        if counts[&label] > 1 {
                            detail
                        } else {
                            label
                        }
                    })
                    .collect();
                values.sort();
                (
                    Span {
                        file: FileId(0),
                        start,
                        end: row.end,
                    },
                    name,
                    values,
                    complete,
                )
            })
            .collect()
    }
}
