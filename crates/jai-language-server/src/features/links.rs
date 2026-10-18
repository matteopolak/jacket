//! `#load` and `#import` links.
use super::OpenFiles;
use crate::analysis::Span;
use crate::session::Session;
use crate::{DocumentUri, Error, Location, Range};
use jaic::project::{DirectiveLink as Link, LinkSource};
use jaic::sema::FileSystem;
use std::path::{Path, PathBuf};

impl Session {
    // `#load` / `#import` links

    /// The file a `#load` or `#import` of `uri` brings in, resolved as the compiler does:
    /// `#load` relative to the loading file; a module in `modules/` next to the file, then on
    /// the import path (`Name.jai` before `Name/module.jai`); `,file` and `,dir` relative to
    /// the file. Open documents count as files.
    pub(super) fn link_target(&self, uri: &DocumentUri, link: &Link) -> Option<DocumentUri> {
        let files = OpenFiles {
            session: self,
        };
        let path = match &link.source {
            LinkSource::Load(path) => PathBuf::from(uri.load(path).ok()?.path()),
            LinkSource::Import(source) => {
                let root = PathBuf::from(self.root(uri).path());
                let import_paths = self
                    .options_for(&root)
                    .map(|o| o.import_paths)
                    .unwrap_or_default();
                let dir = Path::new(uri.path()).parent()?;
                jaic::sema::import_entry(&files, &import_paths, source, dir)?
            }
        };
        if !files.is_file(&path) {
            return None;
        }
        let path = self
            .environment
            .as_ref()
            .map_or(path.clone(), |e| crate::document::file_key(&*e.fs, &path));
        DocumentUri::from_path(&path).ok()
    }

    /// Each `#load` / `#import` string of the document with the file it brings in.
    pub fn document_links(&self, uri: &DocumentUri) -> Result<Vec<(Range, String)>, Error> {
        self.document(uri)?;
        let links = self
            .analysis_for(uri)
            .expect("opened document has analysis")
            .links
            .clone();
        let mut out: Vec<(Range, String)> = links
            .iter()
            .filter_map(|link| {
                let target = self.link_target(uri, link)?;
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
