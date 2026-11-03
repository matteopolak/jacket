//! `#load` and `#import` links.
use super::OpenFiles;
use crate::analysis::Span;
use crate::session::Session;
use crate::{DocumentUri, Error, Location, Range};
use jaic::project::{DirectiveLink as Link, LinkSource};
use jaic::sema::FileSystem;
use std::cell::OnceCell;
use std::path::{Path, PathBuf};

impl Session {
    // `#load` / `#import` links

    /// The file a `#load` or `#import` of `uri` brings in, resolved as the compiler does:
    /// `#load` relative to the loading file; a module in `modules/` next to the file, then on
    /// the import path (`Name.jai` before `Name/module.jai`); `,file` and `,dir` relative to
    /// the file. Open documents count as files.
    pub(super) fn link_target(&self, uri: &DocumentUri, link: &Link) -> Option<DocumentUri> {
        self.link_target_with_paths(uri, link, &OnceCell::new())
    }

    fn link_target_with_paths(
        &self,
        uri: &DocumentUri,
        link: &Link,
        paths: &OnceCell<Vec<PathBuf>>,
    ) -> Option<DocumentUri> {
        let files = OpenFiles {
            session: self,
        };
        let path = match &link.source {
            LinkSource::Load(path) => PathBuf::from(uri.load(path).ok()?.path()),
            LinkSource::Import(source) => {
                let import_paths = paths.get_or_init(|| {
                    let root = PathBuf::from(self.root(uri).path());
                    self.options_for(&root)
                        .map(|o| o.import_paths)
                        .unwrap_or_default()
                });
                let dir = Path::new(uri.path()).parent()?;
                jaic::sema::import_entry(&files, import_paths, source, dir)?
            }
        };
        if !files.is_file(&path) {
            return None;
        }
        let path = self
            .environment
            .as_ref()
            .map_or(path.clone(), |e| jaic::paths::file_identity(&*e.fs, &path));
        self.client_uri(&path)
    }

    /// Each `#load` / `#import` string of the document with the file it brings in.
    pub fn document_links(&self, uri: &DocumentUri) -> Result<Vec<(Range, String)>, Error> {
        self.document(uri)?;
        let links = self
            .analysis_for(uri)
            .expect("opened document has analysis")
            .links
            .clone();
        // All links share the same program and options during this request. Keep the
        // paths only for this call, so edits and watched-file changes need no invalidation.
        let paths = OnceCell::new();
        let mut out: Vec<(Range, String)> = links
            .iter()
            .filter_map(|link| {
                let target = self.link_target_with_paths(uri, link, &paths)?;
                Some((
                    self.range_of(uri, Span::new(link.string.start, link.string.end))?,
                    target.as_str().to_string(),
                ))
            })
            .collect();
        out.extend(self.doc_comment_links(uri));
        Ok(out)
    }

    /// Definition of a `#load` / `#import` directive or its string: the start of the file.
    pub(crate) fn link_definition(&self, uri: &DocumentUri, byte: usize) -> Option<Location> {
        let link = self
            .analysis_for(uri)?
            .links
            .iter()
            .find(|l| l.directive.start <= byte && byte <= l.directive.end)?;
        let target = self.link_target(uri, link)?;
        Some(Location {
            uri: target.as_str().into(),
            range: Range::default(),
        })
    }
}
