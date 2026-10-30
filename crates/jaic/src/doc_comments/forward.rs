//! Finding what an alias names from the library's own files, for `jaic -doc_comments`: the
//! rule for which declaration supplies an alias's docs is [`docs::docs_site`]; this is only the
//! resolver that reads files instead of asking the type checker.
use crate::ast::ImportSource;
use crate::docs::Resolver;
use crate::fxhash::HashMap;
use crate::parser::ide_facts::{self, TopLevelKind};
use crate::source::FileId;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Reads files of a library and the standard library, each once.
pub(super) struct Files<'a> {
    stdlib: &'a Path,
    texts: HashMap<PathBuf, Option<Rc<str>>>,
}

/// What one name at the top of a file stands for.
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
        }
    }

    fn read(&mut self, file: &Path) -> Option<Rc<str>> {
        self.texts
            .entry(file.to_path_buf())
            .or_insert_with(|| std::fs::read_to_string(file).ok().map(Rc::from))
            .clone()
    }

    /// The file an import of `source` in a file of `base` leads to.
    fn imported(&self, base: &Path, source: &ImportSource) -> Option<PathBuf> {
        let candidates: Vec<PathBuf> = match source {
            ImportSource::File(path) => vec![base.join(&**path)],
            ImportSource::Module(name) => {
                let local = base.join(&**name);
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
        candidates.into_iter().find(|p| p.is_file())
    }

    /// What `name` is at the top level of `file` or of a file it loads.
    fn top_level(&mut self, file: &Path, name: &str, depth: usize) -> Option<Name> {
        let text = self.read(file)?;
        let parsed = crate::parser::parse_file_ide(FileId(0), &text);
        let base = file.parent().unwrap_or(Path::new("."));
        let mut loads = Vec::new();
        for item in ide_facts::top_level(&parsed.file) {
            match item.kind {
                TopLevelKind::Decl(decl) => {
                    if let Some(found) = decl.names.iter().find(|n| n.name.as_str() == name) {
                        return Some(Name::Decl(file.to_path_buf(), found.span.start as usize));
                    }
                }
                TopLevelKind::Import(import) => {
                    if import
                        .name
                        .as_ref()
                        .is_some_and(|n| n.name.as_str() == name)
                        && let Some(found) = self.imported(base, &import.source)
                    {
                        return Some(Name::Module(found));
                    }
                }
                TopLevelKind::Load(path, _) => loads.push(base.join(path)),
                TopLevelKind::Assert(_) => {}
            }
        }
        if depth == 0 {
            return None;
        }
        loads
            .iter()
            .find_map(|loaded| self.top_level(loaded, name, depth - 1))
    }
}

/// Files loaded from files loaded from ... are looked through this deep.
const LOAD_DEPTH: usize = 4;

impl Resolver<PathBuf> for Files<'_> {
    fn text(&mut self, file: &PathBuf) -> Option<Rc<str>> {
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
