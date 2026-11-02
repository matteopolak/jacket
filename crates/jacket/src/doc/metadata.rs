//! Publication metadata is resolved before rendering, so themes need no URL or JSON logic.
use super::markdown::{esc, plain};
use super::view::{Content, PageView};
use serde_json::json;
mod config;
pub use config::{Publication, prepare};

pub struct Metadata {
    pub origin: String,
    pub base: String,
    pub name: String,
    pub image: String,
    pub version: String,
    pub canonical_version: Option<String>,
}

/// `url` is an origin, not a deployment path (that belongs in --base-url).
pub fn valid_origin(origin: &str) -> bool {
    let Some(host) = origin
        .strip_prefix("https://")
        .or_else(|| origin.strip_prefix("http://"))
    else {
        return false;
    };
    !host.is_empty()
        && !host
            .chars()
            .any(|c| c.is_whitespace() || "/?#@\\\"'<>".contains(c))
}

/// Collapse plain text and reserve one character for the ellipsis, cutting at a word boundary.
pub fn description(text: &str) -> String {
    let text = plain(text).split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() <= 155 {
        return text;
    }
    let cut: String = text.chars().take(154).collect();
    format!("{}…", cut.rsplit_once(' ').map_or("", |(a, _)| a))
}

impl Metadata {
    pub fn files(
        &self,
        urls: &[String],
        image: Option<(String, Vec<u8>)>,
    ) -> Vec<(String, Vec<u8>)> {
        let mut files: Vec<_> = image.into_iter().collect();
        if !self.origin.is_empty() {
            let root = self.url("", false);
            files.extend(sitemaps(urls, &root, 50_000, 50 * 1024 * 1024));
            files.push((
                "robots.txt".into(),
                format!("Sitemap: {root}sitemap.xml\n").into_bytes(),
            ));
        }
        files
    }

    pub fn url(&self, path: &str, canonical: bool) -> String {
        if self.origin.is_empty() {
            return String::new();
        }
        // Current releases live at url + base. Archived versions live at
        // url + base + versions/<version>/. An archived build passes that directory
        // as --base-url; strip its suffix when targeting a different canonical version.
        let version = if canonical {
            self.canonical_version
                .as_deref()
                .filter(|v| *v != self.version)
        } else {
            None
        };
        let folder = version.map_or_else(String::new, |v| format!("versions/{v}/"));
        let path = path.strip_suffix("index.html").unwrap_or(path);
        let archived = format!("versions/{}/", self.version);
        let base = if version.is_some() {
            self.base.strip_suffix(&archived).unwrap_or(&self.base)
        } else {
            &self.base
        };
        format!("{}{base}{folder}{path}", self.origin)
    }

    pub fn apply(&self, page: &mut PageView) {
        page.description = description(&page.description);
        page.head.url = if page.not_found {
            String::new()
        } else {
            self.url(&page.path, false)
        };
        page.head.canonical = if page.not_found {
            String::new()
        } else {
            self.url(&page.path, true)
        };
        page.head.og_type = if page.path.is_empty() {
            "website"
        } else {
            "article"
        }
        .into();
        page.head.image.clone_from(&self.image);
        page.head.twitter_card = if self.image.is_empty() {
            "summary"
        } else {
            "summary_large_image"
        }
        .into();
        page.head.noindex = page.not_found
            || self
                .canonical_version
                .as_deref()
                .is_some_and(|v| v != self.version);
        let url = &page.head.url;
        let mut data = Vec::new();
        let crumbs = match &page.content {
            Content::Item {
                crumbs, ..
            }
            | Content::Module {
                crumbs, ..
            } => Some(crumbs),
            _ => None,
        };
        if let Content::Item {
            crumbs, ..
        } = &page.content
        {
            let module = crumbs.iter().rev().nth(1).map_or("", |c| c.label.as_str());
            data.push(
                json!({"@context":"https://schema.org", "@type":"TechArticle",
                "headline":page.title, "description":page.description, "url":url,
                "isPartOf":{"@type":"WebSite", "name":self.name, "url":self.url("", false)},
                "about":{"@type":"SoftwareSourceCode", "name":module}}),
            );
        }
        if let Some(crumbs) = crumbs {
            let entries: Vec<_> = crumbs
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let href = c.href.as_ref().map_or_else(
                        || url.to_owned(),
                        |h| {
                            if h.starts_with("http://") || h.starts_with("https://") {
                                h.clone()
                            } else {
                                format!("{}{h}", self.origin)
                            }
                        },
                    );
                    json!({"@type":"ListItem", "position":i+1, "name":c.label, "item":href})
                })
                .collect();
            data.push(json!({"@context":"https://schema.org", "@type":"BreadcrumbList", "itemListElement":entries}));
        }
        page.head.json_ld = if data.is_empty() {
            String::new()
        } else {
            serde_json::to_string(&data).unwrap().replace("</", "<\\/")
        };
    }
}

/// Limits include XML markup and escaped URLs; no shard exceeds either sitemap protocol limit.
pub fn sitemaps(
    urls: &[String],
    root: &str,
    count_limit: usize,
    byte_limit: usize,
) -> Vec<(String, Vec<u8>)> {
    const START: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">";
    const END: &str = "</urlset>";
    let mut shards = Vec::new();
    let mut text = START.to_owned();
    let mut count = 0;
    for url in urls {
        let entry = format!("<url><loc>{}</loc></url>", esc(url));
        if count > 0 && (count >= count_limit || text.len() + entry.len() + END.len() > byte_limit)
        {
            text.push_str(END);
            shards.push(std::mem::replace(&mut text, START.to_owned()));
            count = 0;
        }
        text.push_str(&entry);
        count += 1;
    }
    text.push_str(END);
    shards.push(text);
    if shards.len() == 1 {
        return vec![("sitemap.xml".into(), shards.remove(0).into_bytes())];
    }
    let mut index = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><sitemapindex xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">",
    );
    let mut files = Vec::new();
    for (i, shard) in shards.into_iter().enumerate() {
        let name = format!("sitemap-{}.xml", i + 1);
        index.push_str(&format!(
            "<sitemap><loc>{}{name}</loc></sitemap>",
            esc(root)
        ));
        files.push((name, shard.into_bytes()));
    }
    index.push_str("</sitemapindex>");
    files.push(("sitemap.xml".into(), index.into_bytes()));
    files
}

#[cfg(test)]
mod tests;
