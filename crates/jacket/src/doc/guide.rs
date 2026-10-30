//! The Tools pages: the reference pages of the repository (`docs/reference/*.md`), which the
//! tools generate from their own definitions, rendered as part of the site.
use super::markdown::{self, Segment, Settings, esc, plain};
use super::shell::{Page, render};
use super::site::Site;
use std::path::Path;

/// (file, page name, one line for the index).
const PAGES: [(&str, &str, &str); 3] = [
    (
        "command-line.md",
        "Command line",
        "Every flag of jaic, jailint, and jacket, as their --help prints it.",
    ),
    (
        "environment-variables.md",
        "Environment variables",
        "What each JAIC_ and JACKET_ variable does and who reads it.",
    ),
    (
        "lint-rules.md",
        "Lint rules",
        "Every jailint rule with its group and default level.",
    ),
];

/// The folder a reference page is served from, as a link.
fn page_dir(file: &str) -> String {
    format!("tools/{}/", file.trim_end_matches(".md"))
}

fn page_path(file: &str) -> String {
    format!("{}index.html", page_dir(file))
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

fn side(site: &Site, current: Option<&str>) -> String {
    let mut out = String::from("<div class=\"nav-h\">Tools</div>\n<div class=\"nav-list\">\n");
    for (file, name, _) in PAGES {
        out.push_str(&format!(
            "<a href=\"{}\"{}>{}</a>\n",
            esc(&site.href(&page_dir(file))),
            if current == Some(file) {
                " aria-current=\"page\""
            } else {
                ""
            },
            esc(name)
        ));
    }
    out.push_str("</div>\n");
    out
}

pub fn index(site: &Site) -> String {
    let mut main = String::from("<h1 class=\"page-title\">Tools</h1>\n<ul class=\"cards\">\n");
    for (file, name, about) in PAGES {
        main.push_str(&format!(
            "<li><a class=\"card\" href=\"{}\"><span>{}</span><span class=\"card-sum\">{}</span></a></li>\n",
            esc(&site.href(&page_dir(file))),
            esc(name),
            esc(about)
        ));
    }
    main.push_str("</ul>\n");
    render(
        site,
        &Page {
            title: "Tools - Jacket",
            description: "Reference pages for jaic, jailint, jacket and their settings.",
            side: side(site, None),
            side_label: "Tools",
            main,
            toc: String::new(),
            path: "tools/index.html",
        },
    )
}

/// The pages found in `docs/reference` of `root`, as (path below the site, HTML).
pub fn pages(site: &Site, root: &Path) -> Result<Vec<(String, String)>, String> {
    let mut out = vec![("tools/index.html".to_owned(), index(site))];
    for (file, name, about) in PAGES {
        let path = root.join("docs/reference").join(file);
        let md = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let here = format!("docs/reference/{file}");
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
            if let Some((own, _, _)) = PAGES.iter().find(|(f, _, _)| *f == target) {
                return Some(format!("{}{anchor}", site.href(&page_dir(own))));
            }
            Some(format!(
                "{}/blob/{}/{}{anchor}",
                site.repo,
                site.source_ref,
                repo_path(&here, target)
            ))
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
            .unwrap_or_else(|| about.to_owned());
        let description: String = description.chars().take(160).collect();
        let html = render(
            site,
            &Page {
                title: &format!("{name} - Jacket"),
                description: &description,
                side: side(site, Some(file)),
                side_label: "Tools",
                main,
                toc: String::new(),
                path: &page_path(file),
            },
        );
        out.push((page_path(file), html));
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
