//! Guide input loading and link resolution.
use super::super::markdown::{self, Segment, Settings, esc, plain};
use super::super::site::Site;
use super::{Content, DocSegment, GuideLink, NavLink, PageView, Sidebar, SiteView};
use jacket_config::schema::DocPage;
use std::path::Path;

/// The folder a page is served from, as a link: `tools/command-line/`.
fn page_dir(site: &Site, page: &DocPage) -> String {
    let stem = Path::new(&page.file)
        .file_stem()
        .map_or_else(|| page.name.clone(), |s| s.to_string_lossy().into_owned());
    format!("{}{}/", site.pages_dir(), Site::slug(&stem))
}

fn page_path(site: &Site, page: &DocPage) -> String {
    format!("{}index.html", page_dir(site, page))
}

/// `from` (a path in `docs/`) joined with the relative `link`, normalised.
fn repo_path(from: &str, link: &str) -> String {
    let mut parts: Vec<&str> = from.split('/').collect();
    parts.pop();
    for part in link.split('/') {
        match part {
            "." | "" => {}
            ".." => {
                parts.pop();
            }
            part => parts.push(part),
        }
    }
    parts.join("/")
}

fn side(site: &Site, cards: &[GuideLink], current: Option<usize>) -> Sidebar {
    Sidebar::Guides {
        heading: site.pages_title.clone(),
        links: cards
            .iter()
            .enumerate()
            .map(|(n, c)| NavLink {
                href: c.href.clone(),
                label: c.label.clone(),
                title: None,
                depth: 0,
                current: current == Some(n),
                experimental: false,
            })
            .collect(),
    }
}

pub fn pages(
    site: &Site,
    shared: &mut SiteView,
    dir: &Path,
    prefix: &str,
    pages: &[DocPage],
) -> Result<Vec<PageView>, String> {
    let cards: Vec<_> = pages
        .iter()
        .map(|p| GuideLink {
            href: site.href(&page_dir(site, p)),
            label: p.name.clone(),
            summary: p.summary.clone(),
        })
        .collect();
    shared.guides.clone_from(&cards);
    shared.guides_index = Some(PageView {
        title: site.title(&site.pages_title),
        description: format!("{} for {}.", site.pages_title, site.name),
        path: format!("{}index.html", site.pages_dir()),
        side_label: site.pages_title.clone(),
        sidebar: side(site, &cards, None),
        outline: Vec::new(),
        content: Content::GuidesIndex {
            title: site.pages_title.clone(),
            cards: cards.clone(),
        },
    });
    let mut out = Vec::new();
    let in_repo: Vec<String> = pages
        .iter()
        .map(|p| repo_path(&format!("{prefix}x"), &p.file))
        .collect();
    for (n, page) in pages.iter().enumerate() {
        let path = dir.join(&page.file);
        let md = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let here = &in_repo[n];
        let link = |dest: &str| -> Option<String> {
            if dest.contains("://") || dest.starts_with(['#', '/']) {
                return Some(dest.to_owned());
            }
            let (target, anchor) = dest.split_once('#').map_or((dest, ""), |(t, a)| (t, a));
            let anchor = if anchor.is_empty() {
                String::new()
            } else {
                format!("#{anchor}")
            };
            let target = repo_path(here, target);
            if let Some(own) = in_repo.iter().position(|p| *p == target) {
                return Some(format!(
                    "{}{anchor}",
                    site.href(&page_dir(site, &pages[own]))
                ));
            }
            let at = if site.source_ref.is_empty() {
                site.commit.as_deref()?
            } else {
                site.source_ref.as_str()
            };
            if site.repo.is_empty() {
                return None;
            }
            Some(format!("{}/blob/{at}/{target}{anchor}", site.repo))
        };
        let segs = markdown::segments(
            &md,
            &Settings {
                heading_shift: 0,
                heading_ids: true,
                link: &link,
            },
        );
        let segments = segs
            .iter()
            .map(|s| match s {
                Segment::Html(h) => DocSegment::Html {
                    prose_html: h.clone(),
                },
                Segment::Code {
                    code, ..
                } => DocSegment::Code {
                    code_html: esc(code.trim_end()),
                },
            })
            .collect();
        let description = segs
            .iter()
            .find_map(|s| match s {
                Segment::Html(h) if h.starts_with("<p>") => Some(plain(h)),
                _ => None,
            })
            .unwrap_or_else(|| page.summary.clone());
        let description: String = description.chars().take(160).collect();
        out.push(PageView {
            title: site.title(&page.name),
            description,
            path: page_path(site, page),
            side_label: site.pages_title.clone(),
            sidebar: side(site, &cards, Some(n)),
            outline: Vec::new(),
            content: Content::Guide {
                segments,
            },
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::repo_path;

    #[test]
    fn repo_links_resolve_from_the_page() {
        assert_eq!(
            repo_path("docs/reference/lint-rules.md", "../tools/jailint.md"),
            "docs/tools/jailint.md"
        );
        assert_eq!(
            repo_path("docs/reference/a.md", "b.md"),
            "docs/reference/b.md"
        );
    }
}
