//! Completion inside the string of `#load` and `#import`.
use crate::session::Session;
use crate::{CompletionItem, CompletionKind, CompletionList, DocumentUri};
use jaic::parser::ide_facts::{CursorContext, PathDirective};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

impl Session {
    /// Inside the string of `#load "..."` (files and folders relative to the document) or
    /// `#import "..."` (modules on the import path): the entries of the folder typed so far.
    pub(crate) fn path_completion(
        &self,
        uri: &DocumentUri,
        text: &str,
        context: &CursorContext,
        byte: usize,
    ) -> Option<CompletionList> {
        let CursorContext::String(string) = context else {
            return None;
        };
        let load = match string.directive? {
            PathDirective::Load => true,
            PathDirective::Import => false,
        };
        let typed = text.get(string.start as usize..byte)?;
        let (folder, partial) = typed.rsplit_once('/').unwrap_or(("", typed));
        let here = Path::new(uri.path()).parent().unwrap_or(Path::new("/"));
        let roots: Vec<PathBuf> = if load {
            vec![here.join(folder)]
        } else {
            let options = self.options_for(&PathBuf::from(self.root(uri).path()));
            let mut roots: Vec<PathBuf> = options
                .map(|o| o.import_paths)
                .unwrap_or_default()
                .into_iter()
                .map(|p| p.join(folder))
                .collect();
            roots.push(here.join("modules").join(folder));
            roots
        };
        // (name, is_folder): the environment's files and the open documents under each root.
        let mut entries = BTreeSet::new();
        for root in &roots {
            if let Some(environment) = &self.environment {
                entries.extend(environment.fs.list_dir(root));
            }
            for open in self.documents.keys() {
                let Ok(rest) = Path::new(open.path()).strip_prefix(root) else {
                    continue;
                };
                let mut parts = rest.iter();
                let Some(first) = parts.next() else {
                    continue;
                };
                entries.insert((first.to_string_lossy().into_owned(), parts.next().is_some()));
            }
        }
        // jaic's own modules, imported as `Extensions/Name` (stdlib/Extensions).
        let extensions = jaic::STDLIB_EXTENSIONS_DIR;
        let module_detail = if folder == extensions {
            "jaic extension module"
        } else {
            "module"
        };
        let items = entries
            .into_iter()
            .filter(|(name, _)| name.starts_with(partial) && !name.starts_with('.'))
            .filter_map(|(name, is_folder)| {
                let jai = name.strip_suffix(".jai");
                match (is_folder, jai) {
                    (true, _) if !load && folder.is_empty() && name == extensions => {
                        Some(CompletionItem {
                            label: name,
                            kind: CompletionKind::Folder,
                            detail: "jaic extension modules (not official Jai)".into(),
                            ..CompletionItem::default()
                        })
                    }
                    (true, _) => Some(CompletionItem {
                        label: if load {
                            format!("{name}/")
                        } else {
                            name
                        },
                        kind: if load {
                            CompletionKind::Folder
                        } else {
                            CompletionKind::Module
                        },
                        detail: if load {
                            "folder"
                        } else {
                            module_detail
                        }
                        .into(),
                        ..CompletionItem::default()
                    }),
                    (false, Some(stem)) => Some(CompletionItem {
                        label: if load {
                            name.clone()
                        } else {
                            stem.into()
                        },
                        kind: if load {
                            CompletionKind::File
                        } else {
                            CompletionKind::Module
                        },
                        detail: if load {
                            "file"
                        } else {
                            module_detail
                        }
                        .into(),
                        ..CompletionItem::default()
                    }),
                    _ => None,
                }
            })
            // Loading the file being edited would include it twice.
            .filter(|item| !load || here.join(folder).join(&item.label) != Path::new(uri.path()))
            .collect::<Vec<_>>();
        let mut seen = BTreeSet::new();
        Some(CompletionList {
            is_incomplete: false,
            items: items
                .into_iter()
                .filter(|i| seen.insert(i.label.clone()))
                .collect(),
        })
    }
}
