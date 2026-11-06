//! Finding what an alias names from the library's own files, for `jaic -doc_comments`: the
//! rule for which declaration supplies an alias's docs is [`docs::docs_site`]; this is only the
//! resolver that reads files instead of asking the type checker.
use crate::ast::{ImportSource, UsingFilter};
use crate::docs::Resolver;
use crate::fxhash::HashMap;
use crate::parser::ide_facts::{self, TopLevelKind};
use crate::source::FileId;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Reads and parses files of a library and the standard library, each once.
pub(super) struct Files<'a> {
    stdlib: &'a Path,
    texts: HashMap<PathBuf, Option<Arc<str>>>,
    tops: HashMap<PathBuf, Option<Arc<Top>>>,
    loaders: HashMap<PathBuf, Vec<PathBuf>>,
}

/// The top level of one file: what its names stand for, and the files it brings in.
struct Top {
    /// The first declaration or named import of each name.
    names: HashMap<String, Name>,
    /// Loaded files, in order.
    loads: Vec<PathBuf>,
    /// Modules imported without a name, in order.
    unnamed: Vec<(PathBuf, UsingFilter)>,
}

/// What one name at the top of a file stands for.
#[derive(Clone)]
enum Name {
    /// A constant declared in this file, at this offset.
    Decl(PathBuf, usize),
    /// A module or file imported under this name.
    Module(PathBuf),
}

impl<'a> Files<'a> {
    pub(super) fn new(stdlib: &'a Path) -> Self {
        Self {
            stdlib,
            texts: HashMap::default(),
            tops: HashMap::default(),
            loaders: HashMap::default(),
        }
    }

    /// The source files successfully read during documentation resolution, in path order.
    pub(super) fn dependencies(&self) -> Vec<(PathBuf, String)> {
        use sha2::{Digest, Sha256};
        let mut files = self
            .texts
            .iter()
            .filter_map(|(path, text)| {
                text.as_ref().map(|text| {
                    (
                        path.clone(),
                        Sha256::digest(text.as_bytes())
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect(),
                    )
                })
            })
            .collect::<Vec<_>>();
        files.sort();
        files
    }

    /// These modules supply language and implementation declarations, not public pages.
    pub(super) fn unpublished(&self, file: &Path) -> bool {
        file.starts_with(self.stdlib.join("prelude"))
            || file.starts_with(self.stdlib.join("internal"))
    }

    fn read(&mut self, file: &Path) -> Option<Arc<str>> {
        self.texts
            .entry(file.to_path_buf())
            .or_insert_with(|| std::fs::read_to_string(file).ok().map(Arc::from))
            .clone()
    }

    /// The file an import of `source` in a file of `base` leads to.
    fn imported(&self, base: &Path, source: &ImportSource) -> Option<PathBuf> {
        let candidates: Vec<PathBuf> = match source {
            ImportSource::File(path) => vec![base.join(&**path)],
            ImportSource::Module(name) => {
                let local = base.join("modules").join(&**name);
                let stdlib = self.stdlib.join(&**name);
                vec![
                    local.with_extension("jai"),
                    crate::paths::module_entry(&local),
                    stdlib.with_extension("jai"),
                    crate::paths::module_entry(&stdlib),
                ]
            }
            ImportSource::Dir(_) | ImportSource::String(_) => return None,
        };
        candidates
            .into_iter()
            .find(|p| p.is_file())
            .map(|p| super::canonical(&p))
    }

    /// The top level of `file`, parsed the first time it is asked for.
    fn top(&mut self, file: &Path) -> Option<Arc<Top>> {
        if let Some(top) = self.tops.get(file) {
            return top.clone();
        }
        let top = self.read(file).map(|text| {
            let parsed = crate::parser::parse_file_ide(FileId(0), &text);
            let base = file.parent().unwrap_or(Path::new("."));
            let mut top = Top {
                names: HashMap::default(),
                loads: Vec::new(),
                unnamed: Vec::new(),
            };
            let tops = ide_facts::top_level(&parsed.file);
            // Bindings in a #module_parameters body are listed among the top level.
            for item in tops {
                match item.kind {
                    TopLevelKind::Decl(decl) => {
                        for n in &decl.names {
                            top.names
                                .entry(n.name.as_str().to_owned())
                                .or_insert_with(|| {
                                    Name::Decl(file.to_path_buf(), n.span.start as usize)
                                });
                        }
                    }
                    TopLevelKind::Import(import) => {
                        let found = self.imported(base, &import.source);
                        match (&import.name, found) {
                            (Some(n), Some(found)) => {
                                if let Some(filter) = &import.using {
                                    top.unnamed.push((found.clone(), filter.clone()));
                                }
                                top.names
                                    .entry(n.name.as_str().to_owned())
                                    .or_insert(Name::Module(found));
                            }
                            (None, Some(found)) => top.unnamed.push((found, UsingFilter::None)),
                            (_, None) => {}
                        }
                    }
                    TopLevelKind::Load(path, _) => {
                        let loaded = super::canonical(&base.join(path));
                        self.loaders
                            .entry(loaded.clone())
                            .or_default()
                            .push(file.to_path_buf());
                        top.loads.push(loaded);
                    }
                    TopLevelKind::Assert(_) | TopLevelKind::ModuleParameters(_) => {}
                }
            }
            Arc::new(top)
        });
        self.tops.insert(file.to_path_buf(), top.clone());
        top
    }

    /// What `name` is at the top level of `file` or of a file it loads.
    fn top_level(&mut self, file: &Path, name: &str, depth: usize) -> Option<Name> {
        self.top_level_in(file, name, depth, &mut BTreeSet::new())
    }

    fn top_level_in(
        &mut self,
        file: &Path,
        name: &str,
        depth: usize,
        seen: &mut BTreeSet<(PathBuf, String)>,
    ) -> Option<Name> {
        if !seen.insert((file.to_path_buf(), name.to_owned())) {
            return None;
        }
        let top = self.top(file)?;
        if let Some(found) = top.names.get(name) {
            return Some(found.clone());
        }
        if depth == 0 {
            return None;
        }
        if let Some(found) = top
            .loads
            .iter()
            .find_map(|loaded| self.top_level_in(loaded, name, depth - 1, seen))
        {
            return Some(found);
        }
        top.unnamed.iter().find_map(|(module, filter)| {
            let original = match filter {
                UsingFilter::None => name,
                UsingFilter::Only(names) if names.iter().any(|n| n.name.as_str() == name) => name,
                UsingFilter::Except(names) if !names.iter().any(|n| n.name.as_str() == name) => {
                    name
                }
                UsingFilter::Map(names) => names
                    .iter()
                    .find(|(_, to)| to.name.as_str() == name)?
                    .0
                    .name
                    .as_str(),
                _ => return None,
            };
            self.top_level_in(module, original, depth - 1, seen)
        })
    }
}

impl Files<'_> {
    /// The declaration `path` names where it is written in `file`: through the file's named
    /// imports (`Mod.Type`), and for a lone name also through the modules it imports without a
    /// name (`#import "Basic";`), whose exports are in scope.
    pub(super) fn type_site(&mut self, file: &Path, path: &[String]) -> Option<(PathBuf, usize)> {
        self.scope_site(file, path, &mut BTreeSet::new())
    }

    fn scope_site(
        &mut self,
        file: &Path,
        path: &[String],
        seen: &mut BTreeSet<PathBuf>,
    ) -> Option<(PathBuf, usize)> {
        if !seen.insert(file.to_path_buf()) {
            return None;
        }
        self.resolve(&file.to_path_buf(), 0, path).or_else(|| {
            // A loaded file shares the loading module's scope, including sibling declarations.
            let owners = self.loaders.get(file).cloned().unwrap_or_default();
            owners
                .iter()
                .find_map(|owner| self.scope_site(owner, path, seen))
        })
    }

    pub(super) fn implicit_site(&mut self, path: &[String]) -> Option<(PathBuf, usize)> {
        ["prelude", "Runtime_Support"].iter().find_map(|module| {
            self.resolve(
                &crate::paths::module_entry(&self.stdlib.join(module)),
                0,
                path,
            )
        })
    }
}

/// Files loaded from files loaded from ... are looked through this deep.
const LOAD_DEPTH: usize = 32;

impl Resolver<PathBuf> for Files<'_> {
    fn text(&mut self, file: &PathBuf) -> Option<Arc<str>> {
        self.read(file)
    }

    fn resolve(&mut self, file: &PathBuf, _at: usize, path: &[String]) -> Option<(PathBuf, usize)> {
        let (last, modules) = path.split_last()?;
        let mut here = file.clone();
        for module in modules {
            let Name::Module(next) = self.top_level(&here, module, LOAD_DEPTH)? else {
                return None;
            };
            here = next;
        }
        match self.top_level(&here, last, LOAD_DEPTH)? {
            Name::Decl(owner, at) => Some((owner, at)),
            Name::Module(_) => None,
        }
    }
}
