//! `jacket.toml` in the editor: its problems, completion of keys and paths, hover from the
//! schema, and a hint on each dependency (see [`dependencies`]).
//!
//! The text is read a line at a time, not parsed: a file being typed rarely parses, and the
//! cursor is wanted exactly then. Problems are another matter: they are `jacket_config::issues`,
//! as the tools report them, so nothing here judges the file.
pub mod dependencies;
mod lines;
#[cfg(feature = "dependency-hints")]
pub mod live;

use std::path::Path;

use lines::{PathKind, Place};

use crate::analysis::Span;
use crate::model::{
    CompletionItem, CompletionKind, CompletionList, Diagnostic, DiagnosticCode, DiagnosticSeverity,
    Hover, InlayHint, MarkupContent, MarkupKind,
};
use crate::session::{Document, Session};
use crate::{DocumentUri, Error, Position, Range};

impl Session {
    /// The open `jacket.toml` named by `uri`.
    pub(crate) fn config_document(&self, uri: &DocumentUri) -> Option<(&DocumentUri, &Document)> {
        let key = self.document_uri(uri)?;
        self.lint_configs.get_key_value(&key)
    }

    /// What is wrong with the open `jacket.toml` `uri`, each where it is.
    pub(crate) fn manifest_diagnostics(&self, uri: &DocumentUri) -> Vec<Diagnostic> {
        let Some((_, doc)) = self.config_document(uri) else {
            return Vec::new();
        };
        let dir = Path::new(uri.path()).parent().unwrap_or(Path::new("."));

        jacket_config::issues(&doc.text, dir, jailint::config::CHECKS)
            .into_iter()
            .filter_map(|issue| {
                let span = issue.span.unwrap_or_else(|| {
                    let end = doc.text.find('\n').unwrap_or(doc.text.len());
                    0..end
                });

                Some(Diagnostic {
                    range: doc
                        .index
                        .range(&doc.text, Span::new(span.start, span.end))
                        .ok()?,
                    severity: DiagnosticSeverity::Error,
                    code: DiagnosticCode::Config,
                    message: issue.message,
                    related: Vec::new(),
                })
            })
            .collect()
    }

    /// The publication of the open `jacket.toml` `uri` alone: its version and problems, for the
    /// edits that must not touch what the Jai files show until the file is saved.
    pub fn config_publication(&self, uri: &DocumentUri) -> Option<(String, i32, Vec<Diagnostic>)> {
        let (key, doc) = self.config_document(uri)?;

        Some((
            key.as_str().to_string(),
            doc.version,
            self.manifest_diagnostics(uri),
        ))
    }

    /// The description of the key or table under `position` of the open `jacket.toml`.
    pub(crate) fn manifest_hover(
        &self,
        uri: &DocumentUri,
        position: Position,
        kind: MarkupKind,
    ) -> Result<Option<Hover>, Error> {
        let (_, doc) = self.config_document(uri).ok_or(Error::MissingDocument)?;
        let at = doc.index.byte(&doc.text, position);
        let Some((span, name, text)) = lines::describe_at(&doc.text, at) else {
            return Ok(None);
        };
        let value = match kind {
            MarkupKind::Markdown => format!("`{name}`\n\n{text}"),
            MarkupKind::PlainText => format!("{name}\n\n{text}"),
        };

        Ok(Some(Hover {
            contents: MarkupContent {
                kind,
                value,
            },
            range: doc.index.range(&doc.text, span)?,
        }))
    }

    /// Keys, table names and paths at `position` of the open `jacket.toml`.
    pub(crate) fn manifest_completion(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<CompletionList, Error> {
        let (_, doc) = self.config_document(uri).ok_or(Error::MissingDocument)?;
        let at = doc.index.byte(&doc.text, position);
        let replace =
            |start: usize, end: usize| doc.index.range(&doc.text, Span::new(start, end)).ok();
        let items = match lines::place(&doc.text, at) {
            Place::Header {
                start,
                end,
                prefix,
            } => lines::table_names()
                .into_iter()
                .filter(|name| name.starts_with(&prefix))
                .map(|name| CompletionItem {
                    documentation: jacket_config::schema::describe_table(&name).map(String::from),
                    label: name,
                    kind: CompletionKind::Module,
                    detail: "table".into(),
                    replace: replace(start, end),
                    ..CompletionItem::default()
                })
                .collect(),
            Place::Key {
                start,
                end,
                table,
                prefix,
                present,
                assign,
            } => jacket_config::schema::known_keys(&table)
                .iter()
                .filter(|key| key.starts_with(&prefix) && !present.iter().any(|p| p == *key))
                .map(|key| CompletionItem {
                    label: (*key).into(),
                    kind: CompletionKind::Field,
                    detail: table.clone(),
                    documentation: jacket_config::schema::describe(&table, key).map(String::from),
                    insert_text: assign.then(|| format!("{key} = ")),
                    replace: replace(start, end),
                    ..CompletionItem::default()
                })
                .collect(),
            Place::Path {
                start,
                end,
                folder,
                kind,
            } => self.path_items(uri, &folder, kind, replace(start, end)),
            Place::Nothing => Vec::new(),
        };

        Ok(CompletionList {
            items,
            is_incomplete: false,
        })
    }

    /// The folders and files of `folder` (relative to the project) that `kind` accepts.
    fn path_items(
        &self,
        uri: &DocumentUri,
        folder: &str,
        kind: PathKind,
        replace: Option<Range>,
    ) -> Vec<CompletionItem> {
        let Some(environment) = &self.environment else {
            return Vec::new();
        };
        let base = Path::new(uri.path()).parent().unwrap_or(Path::new("/"));
        let root = base.join(folder);

        environment
            .fs
            .list_dir(&root)
            .into_iter()
            .filter(|(name, _)| !name.starts_with('.'))
            .filter(|(name, is_folder)| match kind {
                PathKind::Files => *is_folder || name.ends_with(".jai"),
                PathKind::Folders => *is_folder,
                PathKind::Modules => {
                    *is_folder && environment.fs.is_file(&root.join(name).join("module.jai"))
                }
            })
            .map(|(name, is_folder)| CompletionItem {
                insert_text: is_folder.then(|| format!("{name}/")),
                label: name,
                kind: if is_folder {
                    CompletionKind::Folder
                } else {
                    CompletionKind::File
                },
                replace,
                ..CompletionItem::default()
            })
            .collect()
    }

    /// A hint on each dependency of the open `jacket.toml` in `range`: what it is pinned at,
    /// and the releases beyond that.
    pub(crate) fn manifest_inlay_hints(
        &self,
        uri: &DocumentUri,
        range: Range,
    ) -> Result<Vec<InlayHint>, Error> {
        let (_, doc) = self.config_document(uri).ok_or(Error::MissingDocument)?;
        let Some(source) = &self.dependency_source else {
            return Ok(Vec::new());
        };
        let Ok(config) = jacket_config::parse(&doc.text) else {
            return Ok(Vec::new());
        };
        let dir = Path::new(uri.path()).parent().unwrap_or(Path::new("/"));
        let mut hints = Vec::new();

        for (name, entry) in &config.dependencies {
            let Some(end) = lines::dependency_line_end(&doc.text, name) else {
                continue;
            };
            let position = doc.index.range(&doc.text, Span::new(end, end))?.start;

            if position.line < range.start.line || position.line > range.end.line {
                continue;
            }

            let request = dependencies::Request {
                dir: dir.to_path_buf(),
                name: name.clone(),
                entry: entry.clone(),
                resolver: config.resolver.clone(),
            };
            let Some(standing) = source.lookup(&request) else {
                continue;
            };

            hints.extend(standing.hints(position));
        }

        Ok(hints)
    }
}
