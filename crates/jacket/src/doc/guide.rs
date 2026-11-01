//! The extra pages of `[[doc.page]]`: Markdown files rendered as part of the site, under
//! `pages_title` (the standard library's are the generated `docs/reference/*.md`).
use super::markdown::{self, Segment, Settings, esc, plain};
use super::shell::{Page, render};
use super::site::Site;
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

fn side(site: &Site, pages: &[DocPage], current: Option<usize>) -> String {
    let mut out = format!(
        "<div class=\"nav-h\">{}</div>\n<div class=\"nav-list\">\n",
        esc(&site.pages_title)
    );
    for (i, page) in pages.iter().enumerate() {
        out.push_str(&format!(
            "<a href=\"{}\"{}>{}</a>\n",
            esc(&site.href(&page_dir(site, page))),
            if current == Some(i) {
                " aria-current=\"page\""
            } else {
                ""
            },
            esc(&page.name)
        ));
    }
    out.push_str("</div>\n");
    out
}

pub fn index(site: &Site, pages: &[DocPage]) -> String {
    let mut main = format!(
        "<h1 class=\"page-title\">{}</h1>\n<ul class=\"cards\">\n",
        esc(&site.pages_title)
    );
    for page in pages {
        main.push_str(&format!(
            "<li><a class=\"card\" href=\"{}\"><span>{}</span><span class=\"card-sum\">{}</span></a></li>\n",
            esc(&site.href(&page_dir(site, page))),
            esc(&page.name),
            esc(&page.summary)
        ));
    }
    main.push_str("</ul>\n");
    let description = format!("{} for {}.", site.pages_title, site.name);
    render(
        site,
        &Page {
            title: &site.title(&site.pages_title),
            description: &description,
            side: side(site, pages, None),
            side_label: &site.pages_title,
            main,
            toc: String::new(),
            path: &format!("{}index.html", site.pages_dir()),
        },
    )
}

/// The pages, read from `dir` (the folder of the `jacket.toml`), as (path below the site,
/// HTML). `prefix` is where `dir` is in the repository, with a trailing slash, for links to
/// other files of it.
pub fn pages(
    site: &Site,
    dir: &Path,
    prefix: &str,
    pages: &[DocPage],
) -> Result<Vec<(String, String)>, String> {
    let mut out = vec![(
        format!("{}index.html", site.pages_dir()),
        index(site, pages),
    )];
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
        let mut main = String::from("<div class=\"prose guide\">\n");
        for seg in &segs {
            match seg {
                Segment::Html(html) => main.push_str(html),
                Segment::Code {
                    code, ..
                } => main.push_str(&format!(
                    "<pre class=\"code\"><code>{}</code></pre>\n",
                    esc(code.trim_end())
                )),
            }
        }
        main.push_str("</div>\n");
        let description = segs
            .iter()
            .find_map(|s| match s {
                Segment::Html(h) if h.starts_with("<p>") => Some(plain(h)),
                _ => None,
            })
            .unwrap_or_else(|| page.summary.clone());
        let description: String = description.chars().take(160).collect();
        let html = render(
            site,
            &Page {
                title: &site.title(&page.name),
                description: &description,
                side: side(site, pages, Some(n)),
                side_label: &site.pages_title,
                main,
                toc: String::new(),
                path: &page_path(site, page),
            },
        );
        out.push((page_path(site, page), html));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

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
