//! Publication policy consumes semantic facts and emits typed metadata, never page markup.
use super::contract::{
    BreadcrumbEntry, DocumentKind, DocumentNode, OpenGraphType, PageKind, PageMetadata,
    StructuredData, TwitterCard,
};
use super::document;
use crate::cli::DocArgs;
use crate::doc::{metadata, site::Site};
use jacket_config::schema::Doc;
use std::path::Path;

pub(in crate::doc) struct Breadcrumb {
    pub name: String,
    pub href: Option<String>,
}

pub(in crate::doc) struct Page<'a> {
    pub path: &'a str,
    pub kind: PageKind,
    pub title: &'a str,
    pub description: &'a [DocumentNode],
    pub private: bool,
    pub breadcrumbs: &'a [Breadcrumb],
}

pub(in crate::doc) struct Publication {
    pub origin: String,
    pub base: String,
    pub name: String,
    pub image: String,
    pub version: String,
    pub canonical_version: Option<String>,
    pub image_file: Option<(String, Vec<u8>)>,
}

/// Resolve flags over configuration without constructing a rendered page.
/// For example, a configured `og_image = "card.png"` is read relative to jacket.toml.
pub(in crate::doc) fn prepare(
    args: &DocArgs,
    doc: &Doc,
    config_dir: &Path,
    cwd: &Path,
    site: &Site,
) -> Result<Publication, String> {
    let pick = |flag: &Option<String>, setting: &Option<String>| {
        flag.clone().or_else(|| setting.clone()).unwrap_or_default()
    };
    let origin = pick(&args.site_url, &doc.url)
        .trim_end_matches('/')
        .to_owned();
    if origin.is_empty() {
        eprintln!(
            "jacket: warning: no [doc] url or --site-url; canonical URLs and sitemaps are omitted"
        );
    } else if !metadata::valid_origin(&origin) {
        return Err("site URL must be an absolute HTTP(S) origin".into());
    }
    if let Some(version) = &doc.canonical_version
        && (version.is_empty()
            || !version
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
            || matches!(version.as_str(), "." | ".."))
    {
        return Err("canonical_version must be a version path segment".into());
    }
    let image = pick(&args.og_image, &doc.og_image);
    let mut image_file = None;
    let image = if image.is_empty() || image.starts_with("https://") || image.starts_with("http://")
    {
        image
    } else {
        let root = if args.og_image.is_some() {
            cwd
        } else {
            config_dir
        };
        let path = root.join(&image);
        let name = path
            .file_name()
            .ok_or("og_image needs a file name")?
            .to_string_lossy();
        let dest = format!("assets/og-{name}");
        image_file = Some((
            dest.clone(),
            std::fs::read(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?,
        ));
        if origin.is_empty() {
            String::new()
        } else {
            format!("{origin}{}{dest}", site.base)
        }
    };
    Ok(Publication {
        origin,
        base: site.base.clone(),
        name: doc.site_name.clone().unwrap_or_else(|| site.name.clone()),
        image,
        version: site.version.clone(),
        canonical_version: doc.canonical_version.clone(),
        image_file,
    })
}

/// Bound plain semantic text at the same word boundary as the previous metadata policy.
fn description(nodes: &[DocumentNode]) -> String {
    let text = document::plain(nodes)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if text.chars().count() <= 155 {
        return text;
    }
    let cut: String = text.chars().take(154).collect();
    format!("{}…", cut.rsplit_once(' ').map_or("", |(a, _)| a))
}

fn first_paragraph(nodes: &[DocumentNode]) -> Vec<DocumentNode> {
    nodes
        .iter()
        .find(|n| matches!(n, DocumentNode::Paragraph { .. }))
        .cloned()
        .into_iter()
        .collect()
}

fn breadcrumbs(site: &Site, module: &str, declaration: Option<&str>) -> Vec<Breadcrumb> {
    let mut out = vec![Breadcrumb {
        name: if site.breadcrumb.is_empty() {
            "Home".into()
        } else {
            site.breadcrumb.clone()
        },
        href: Some(site.base.clone()),
    }];
    let mut prefix = String::new();
    let parts = module.split('/').collect::<Vec<_>>();
    for (index, part) in parts.iter().enumerate() {
        if !prefix.is_empty() {
            prefix.push('/');
        }
        prefix.push_str(part);
        let linked = site.module_index(&prefix).is_some()
            && (declaration.is_some() || index + 1 < parts.len());
        out.push(Breadcrumb {
            name: (*part).into(),
            href: linked.then(|| site.module_href(&prefix)),
        });
    }
    if let Some(name) = declaration {
        out.push(Breadcrumb {
            name: name.into(),
            href: None,
        });
    }
    out
}

impl Publication {
    pub(in crate::doc) fn module(
        &self,
        site: &Site,
        id: &str,
        docs: &[DocumentNode],
    ) -> PageMetadata {
        self.page(&Page {
            path: &Site::module_path(id),
            kind: PageKind::Module,
            title: &site.title(&id.replace('/', ".")),
            description: &first_paragraph(docs),
            private: false,
            breadcrumbs: &breadcrumbs(site, id, None),
        })
    }

    pub(in crate::doc) fn declaration(
        &self,
        site: &Site,
        module: &str,
        name: &str,
        docs: &[DocumentNode],
        private: bool,
    ) -> PageMetadata {
        self.page(&Page {
            path: &site.item_path(module, name),
            kind: PageKind::Declaration,
            title: &site.title(&format!("{}.{}", module.replace('/', "."), name)),
            description: &first_paragraph(docs),
            private,
            breadcrumbs: &breadcrumbs(site, module, Some(name)),
        })
    }

    pub(in crate::doc) fn document(
        &self,
        site: &Site,
        path: &str,
        kind: DocumentKind,
        title: &str,
        docs: &[DocumentNode],
    ) -> PageMetadata {
        self.page(&Page {
            path,
            kind: kind.into(),
            title: &site.title(title),
            description: &first_paragraph(docs),
            private: false,
            breadcrumbs: &[],
        })
    }

    /// Resolve one route's metadata; for example, a private declaration retains its canonical
    /// destination and structured facts while receiving `noindex`.
    pub(in crate::doc) fn page(&self, page: &Page<'_>) -> PageMetadata {
        let missing = page.kind == PageKind::NotFound;
        let title = if page.kind == PageKind::Home {
            self.name.clone()
        } else {
            page.title.into()
        };
        let description = description(page.description);
        let url = if missing {
            None
        } else {
            self.url(page.path, false)
        };
        let canonical = if missing {
            None
        } else {
            self.url(page.path, true)
        };
        let mut structured = Vec::new();
        if page.kind == PageKind::Declaration {
            let module = page
                .breadcrumbs
                .iter()
                .rev()
                .nth(1)
                .map_or("", |crumb| crumb.name.as_str());
            structured.push(StructuredData::TechArticle {
                headline: title.clone(),
                description: description.clone(),
                url: url.clone(),
                website_name: self.name.clone(),
                website_url: self.url("", false),
                module_name: module.into(),
            });
        }
        if matches!(page.kind, PageKind::Declaration | PageKind::Module) {
            let entries = page
                .breadcrumbs
                .iter()
                .enumerate()
                .map(|(i, crumb)| BreadcrumbEntry {
                    position: (i + 1) as u64,
                    name: crumb.name.clone(),
                    url: crumb.href.as_ref().map_or_else(
                        || url.clone(),
                        |href| {
                            if href.starts_with("http://") || href.starts_with("https://") {
                                Some(href.clone())
                            } else if self.origin.is_empty() {
                                None
                            } else {
                                Some(format!("{}{href}", self.origin))
                            }
                        },
                    ),
                })
                .collect();
            structured.push(StructuredData::BreadcrumbList {
                entries,
            });
        }
        PageMetadata {
            title,
            description,
            url,
            canonical,
            og_type: if page.kind == PageKind::Home {
                OpenGraphType::Website
            } else {
                OpenGraphType::Article
            },
            image: (!self.image.is_empty()).then(|| self.image.clone()),
            twitter_card: if self.image.is_empty() {
                TwitterCard::Summary
            } else {
                TwitterCard::SummaryLargeImage
            },
            noindex: missing
                || page.private
                || self
                    .canonical_version
                    .as_deref()
                    .is_some_and(|v| v != self.version),
            structured,
        }
    }

    /// Publication URL from an output path, e.g. `api/Example.html` becomes `/api/Example`.
    pub(in crate::doc) fn url(&self, path: &str, canonical: bool) -> Option<String> {
        if self.origin.is_empty() {
            return None;
        }
        let version = if canonical {
            self.canonical_version
                .as_deref()
                .filter(|v| *v != self.version)
        } else {
            None
        };
        let folder = version.map_or_else(String::new, |v| format!("versions/{v}/"));
        let path = path.strip_suffix(".html").unwrap_or(path);
        let path = if path == "index" {
            ""
        } else {
            path
        };
        let archived = format!("versions/{}/", self.version);
        let base = if version.is_some() {
            self.base.strip_suffix(&archived).unwrap_or(&self.base)
        } else {
            &self.base
        };
        Some(format!("{}{base}{folder}{path}", self.origin))
    }

    /// Auxiliary publication assets from already filtered public routes, e.g. robots and sitemap.
    /// An archived build's `noindex` does not remove its public routes from this inventory.
    pub(in crate::doc) fn files(&self, urls: &[String]) -> Vec<super::InputAsset> {
        use super::{InputAsset, contract::AssetProvenance};
        let mut files: Vec<_> = self
            .image_file
            .clone()
            .into_iter()
            .map(|(path, bytes)| InputAsset {
                path,
                bytes,
                provenance: AssetProvenance::ConfiguredImage,
            })
            .collect();
        if let Some(root) = self.url("", false) {
            files.extend(
                metadata::sitemaps(urls, &root, 50_000, 50 * 1024 * 1024)
                    .into_iter()
                    .map(|(path, bytes)| InputAsset {
                        path,
                        bytes,
                        provenance: AssetProvenance::GeneratedMetadata,
                    }),
            );
            files.push(InputAsset {
                path: "robots.txt".into(),
                bytes: format!("Sitemap: {root}sitemap.xml\n").into_bytes(),
                provenance: AssetProvenance::GeneratedMetadata,
            });
        }
        files
    }
}

#[cfg(test)]
mod tests;
