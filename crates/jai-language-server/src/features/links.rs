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
        let _query = self.discovery_query();
        let history = std::cell::RefCell::new(crate::semantic::Observations::default());
        let base = self.config_filesystem();
        let fs = base.as_ref().map(|base| crate::semantic::ObservedFs {
            base,
            proof: Some(&history),
            secondary: None,
        });
        let (uri, _) = match &fs {
            Some(fs) => self.projected_document_entry(fs, uri).ok()?,
            None => self.document_entry(uri).ok()?,
        };
        let paths = OnceCell::new();
        let target = self.link_target_with_paths(uri, link, &paths, fs.as_ref());
        history
            .borrow_mut()
            .merge(&self.request_input_proof.borrow());
        if paths.get().is_some() {
            history.borrow_mut().merge(&self.input_proof.borrow());
        }
        if self
            .environment
            .as_ref()
            .is_some_and(|env| !history.borrow().matches(&*env.fs))
        {
            return None;
        }
        target
    }

    fn link_target_with_paths(
        &self,
        uri: &DocumentUri,
        link: &Link,
        paths: &OnceCell<Option<Vec<PathBuf>>>,
        fs: Option<&crate::semantic::ObservedFs<'_>>,
    ) -> Option<DocumentUri> {
        let files = OpenFiles {
            session: self,
            fs,
        };
        let path = match &link.source {
            LinkSource::Load(path) => PathBuf::from(uri.load(path).ok()?.path()),
            LinkSource::Import(source) => {
                let import_paths = if matches!(source, jaic::ast::ImportSource::Module(_)) {
                    paths
                        .get_or_init(|| self.named_link_import_paths(fs?, uri))
                        .as_deref()?
                } else {
                    &[]
                };
                let dir = Path::new(uri.path()).parent()?;
                jaic::sema::import_entry(&files, import_paths, source, dir)?
            }
        };
        if !files.is_file(&path) {
            return None;
        }
        let path = match fs {
            Some(fs) => jaic::paths::file_identity(fs, &path),
            None => path,
        };
        match fs {
            Some(fs) => self.projected_uri(fs, &path),
            None => self.client_uri(&path),
        }
    }

    /// Named modules require a currently proven program/import context, not advisory indexes.
    /// Deferred discovery queues the ordinary background check instead of using guessed dirs.
    fn named_link_import_paths(
        &self,
        fs: &crate::semantic::ObservedFs<'_>,
        uri: &DocumentUri,
    ) -> Option<Vec<PathBuf>> {
        let (uri, doc) = self.projected_document_entry(fs, uri).ok()?;
        let Some((_, root, dirs)) = self.compile_inputs(uri, &doc.text) else {
            if !self.may_compile() {
                self.want_compile(uri, &doc.text);
            }
            return None;
        };
        let mut options = (self.environment.as_ref()?.options)(fs, &root);
        jaic::project::add_import_dirs(&mut options.import_paths, &dirs);
        Some(options.import_paths)
    }

    /// Each `#load` / `#import` string of the document with the file it brings in.
    pub fn document_links(&self, uri: &DocumentUri) -> Result<Vec<(Range, String)>, Error> {
        let _query = self.discovery_query();
        let history = std::cell::RefCell::new(crate::semantic::Observations::default());
        let base = self.config_filesystem();
        let fs = base.as_ref().map(|base| crate::semantic::ObservedFs {
            base,
            proof: Some(&history),
            secondary: None,
        });
        let (uri, _) = match &fs {
            Some(fs) => self.projected_document_entry(fs, uri)?,
            None => self.document_entry(uri)?,
        };
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
                let target = self.link_target_with_paths(uri, link, &paths, fs.as_ref())?;
                Some((
                    self.range_of(uri, Span::new(link.string.start, link.string.end))?,
                    target.as_str().to_string(),
                ))
            })
            .collect();
        out.extend(self.doc_comment_links(uri));
        history
            .borrow_mut()
            .merge(&self.request_input_proof.borrow());
        if paths.get().is_some() {
            history.borrow_mut().merge(&self.input_proof.borrow());
        }
        if self
            .environment
            .as_ref()
            .is_some_and(|env| !history.borrow().matches(&*env.fs))
        {
            return Ok(Vec::new());
        }
        Ok(out)
    }

    /// Definition of a `#load` / `#import` directive or its string: the start of the file.
    pub(crate) fn link_definition(
        &self,
        uri: &DocumentUri,
        byte: usize,
    ) -> Option<Option<Location>> {
        let link = self
            .analysis_for(uri)?
            .links
            .iter()
            .find(|l| l.directive.start <= byte && byte <= l.directive.end)?;
        Some(self.link_target(uri, link).map(|target| Location {
            uri: target.as_str().into(),
            range: Range::default(),
        }))
    }
}
