//! Extra Markdown pages: `[[doc.page]]` files under one sidebar heading, and `[[doc.section]]`
//! files split into an overview and one page per `###` heading, each its own sidebar heading.
use super::super::highlight;
use super::super::markdown::{self, Segment, Settings, esc};
use super::super::site::Site;
use super::{
    Content, DocSegment, NavCategory, NavGroup, NavLink, Outline, PageHead, PageView, Sidebar,
};
use jacket_config::schema::{Doc, DocSection};
use jaic::fxhash::HashMap;
use std::path::Path;

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

/// One page to write.
struct Page {
    name: String,
    md: String,
    /// The file it came from, from the repository root, for resolving its links.
    repo: String,
    /// Where it is written: `tools/command-line.html`.
    path: String,
    /// The sidebar heading it is under.
    heading: String,
}

/// The path of the `[doc] home` page among the guides, until it takes the front page's place.
pub const HOME: &str = "index.html";

/// Every extra page and the sidebar headings that list them.
pub struct Guides {
    pages: Vec<Page>,
    pub nav: Vec<NavCategory>,
    /// A whole file's page.
    files: HashMap<String, String>,
    /// A heading of a split file: the page it became, or the overview's anchor.
    anchors: HashMap<(String, String), String>,
}

/// A `##` group of a split file.
struct Group {
    name: String,
    prose: String,
    entries: Vec<(String, String)>,
}

fn fence(line: &str) -> bool {
    line.trim_start().starts_with("```")
}

/// The text before the first `##`, and the groups after it. `###` before any `##` starts a group
/// with no name.
fn split(md: &str) -> (String, Vec<Group>) {
    let mut intro = String::new();
    let mut groups: Vec<Group> = Vec::new();
    let mut in_code = false;
    for line in md.lines() {
        if fence(line) {
            in_code = !in_code;
        }
        let heading = if in_code {
            None
        } else {
            line.strip_prefix("## ").map(|h| (2, h))
        }
        .or_else(|| {
            if in_code {
                None
            } else {
                line.strip_prefix("### ").map(|h| (3, h))
            }
        });
        match heading {
            Some((2, name)) => groups.push(Group {
                name: name.trim().into(),
                prose: String::new(),
                entries: Vec::new(),
            }),
            Some((_, name)) => {
                if groups.is_empty() {
                    groups.push(Group {
                        name: String::new(),
                        prose: String::new(),
                        entries: Vec::new(),
                    });
                }
                let g = groups.last_mut().unwrap();
                g.entries.push((name.trim().into(), String::new()));
            }
            None => {
                let to = match groups.last_mut() {
                    None => &mut intro,
                    Some(g) => match g.entries.last_mut() {
                        Some((_, body)) => body,
                        None => &mut g.prose,
                    },
                };
                to.push_str(line);
                to.push('\n');
            }
        }
    }
    (intro, groups)
}

/// The first paragraph of `md`, on one line.
fn first_paragraph(md: &str) -> String {
    let mut out = Vec::new();
    for line in md.lines() {
        let t = line.trim();
        if t.is_empty() {
            if out.is_empty() {
                continue;
            }
            break;
        }
        if t.starts_with(['#', '|', '`', '-', '>']) && out.is_empty() {
            continue;
        }
        out.push(t);
    }
    out.join(" ")
}

fn link(href: &str, label: &str) -> NavLink {
    NavLink {
        href: href.into(),
        label: label.into(),
        title: None,
        depth: 0,
        current: false,
        experimental: false,
    }
}

fn read(dir: &Path, file: &str) -> Result<String, String> {
    let path = dir.join(file);
    std::fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))
}

/// A `###` entry's body as a page of its own: `####` headings become `##`, `#####` become `###`.
fn promote(body: &str) -> String {
    let mut out = String::new();
    let mut in_code = false;
    for line in body.lines() {
        if fence(line) {
            in_code = !in_code;
        }
        let line = match line.strip_prefix("##") {
            Some(rest) if !in_code && (rest.starts_with("## ") || rest.starts_with("### ")) => rest,
            _ => line,
        };
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Reads every page and works out the sidebar headings. `prefix` is the config folder's path in
/// the repository.
pub fn collect(site: &Site, dir: &Path, prefix: &str, doc: &Doc) -> Result<Guides, String> {
    let (home, pages, sections) = (doc.home.as_deref(), &doc.page[..], &doc.section[..]);
    let mut out = Guides {
        pages: Vec::new(),
        nav: Vec::new(),
        files: HashMap::default(),
        anchors: HashMap::default(),
    };
    if let Some(file) = home {
        let repo = repo_path(&format!("{prefix}x"), file);
        out.files.insert(repo.clone(), site.href(""));
        out.pages.push(Page {
            name: site.name.clone(),
            md: read(dir, file)?,
            repo,
            path: HOME.into(),
            heading: String::new(),
        });
    }
    if !pages.is_empty() {
        let mut links = Vec::new();
        for page in pages {
            let stem = Path::new(&page.file)
                .file_stem()
                .map_or_else(|| page.name.clone(), |s| s.to_string_lossy().into_owned());
            let folder = format!("{}/{}", site.pages_dir(), markdown::slug(&stem));
            let repo = repo_path(&format!("{prefix}x"), &page.file);
            out.files.insert(repo.clone(), site.href(&folder));
            links.push(link(&site.href(&folder), &page.name));
            out.pages.push(Page {
                name: page.name.clone(),
                md: read(dir, &page.file)?,
                repo,
                path: format!("{folder}.html"),
                heading: site.pages_title.clone(),
            });
        }
        out.nav.push(NavCategory {
            label: site.pages_title.clone(),
            groups: vec![NavGroup {
                label: String::new(),
                key: format!("{}/", site.pages_title),
                open: true,
                links,
            }],
        });
    }
    for section in sections {
        let md = read(dir, &section.file)?;
        let repo = repo_path(&format!("{prefix}x"), &section.file);
        let folder = markdown::slug(&section.name);
        let (intro, groups) = split(&md);
        let overview = site.href(&folder);
        out.files.insert(repo.clone(), overview.clone());
        if section.one_page {
            // One page: the sidebar links to each heading on it.
            nav_one_page(&mut out, section, &repo, &overview, &groups);
            out.pages.push(Page {
                name: section.name.clone(),
                md,
                repo,
                path: format!("{folder}.html"),
                heading: section.name.clone(),
            });
            continue;
        }
        let mut nav = NavCategory {
            label: section.name.clone(),
            groups: vec![NavGroup {
                label: String::new(),
                key: format!("{}/", section.name),
                open: true,
                links: vec![link(&overview, "Overview")],
            }],
        };
        // The overview: the intro, then each group's text and a list of its pages.
        let mut summary = intro;
        for g in &groups {
            if !g.name.is_empty() {
                let id = markdown::slug(&g.name);
                out.anchors
                    .insert((repo.clone(), id.clone()), format!("{overview}#{id}"));
                summary.push_str(&format!("## {}\n", g.name));
            }
            summary.push_str(&g.prose);
            summary.push('\n');
            let mut links = Vec::new();
            for (name, body) in &g.entries {
                let id = markdown::slug(name);
                let page = format!("{folder}/{id}");
                out.anchors
                    .insert((repo.clone(), id.clone()), site.href(&page));
                let lead = first_paragraph(body);
                summary.push_str(&format!("- [{name}](#{id})"));
                if !lead.is_empty() {
                    summary.push_str(&format!(": {lead}"));
                }
                summary.push('\n');
                links.push(link(&site.href(&page), name));
                out.pages.push(Page {
                    name: name.clone(),
                    md: format!("# {name}\n\n{}", promote(body)),
                    repo: repo.clone(),
                    path: format!("{page}.html"),
                    heading: section.name.clone(),
                });
            }
            summary.push('\n');
            if !links.is_empty() {
                nav.groups.push(NavGroup {
                    label: g.name.clone(),
                    key: format!("{}/{}", section.name, g.name),
                    open: false,
                    links,
                });
            }
        }
        out.pages.push(Page {
            name: section.name.clone(),
            md: summary,
            repo,
            path: format!("{folder}.html"),
            heading: section.name.clone(),
        });
        out.nav.push(nav);
    }
    Ok(out)
}

/// The sidebar of a `one_page` section: the page, then each `##` group's headings as anchors.
fn nav_one_page(
    out: &mut Guides,
    section: &DocSection,
    repo: &str,
    overview: &str,
    groups: &[Group],
) {
    let mut nav = NavCategory {
        label: section.name.clone(),
        groups: vec![NavGroup {
            label: String::new(),
            key: format!("{}/", section.name),
            open: true,
            links: vec![link(overview, "Overview")],
        }],
    };
    for g in groups {
        let mut anchor = |name: &str| {
            let id = markdown::slug(name);
            let href = format!("{overview}#{id}");
            out.anchors.insert((repo.to_string(), id), href.clone());
            href
        };
        if !g.name.is_empty() {
            anchor(&g.name);
        }
        let links: Vec<NavLink> = g
            .entries
            .iter()
            .map(|(name, _)| link(&anchor(name), name))
            .collect();
        if !links.is_empty() {
            nav.groups.push(NavGroup {
                label: g.name.clone(),
                key: format!("{}/{}", section.name, g.name),
                open: false,
                links,
            });
        }
    }
    out.nav.push(nav);
}

/// `nav` with the link to `href` marked, and its group open.
fn marked(nav: &[NavCategory], href: &str) -> Vec<NavCategory> {
    let mut nav = nav.to_vec();
    for g in nav.iter_mut().flat_map(|c| c.groups.iter_mut()) {
        for l in &mut g.links {
            if l.href == href {
                l.current = true;
                g.open = true;
            }
        }
    }
    nav
}

/// The `## ` and `### ` headings of a page, for its outline.
fn outline(md: &str) -> Vec<Outline> {
    let mut out = Vec::new();
    let mut in_code = false;
    for line in md.lines() {
        if fence(line) {
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        let (sub, text) = if let Some(t) = line.strip_prefix("## ") {
            (false, t)
        } else if let Some(t) = line.strip_prefix("### ") {
            (true, t)
        } else {
            continue;
        };
        let label = text.trim().replace('`', "");
        out.push(Outline::Heading {
            id: markdown::slug(text),
            label,
            sub,
        });
    }
    out
}

/// A fence's code, coloured when it is Jai.
fn code_html(info: &str, code: &str) -> String {
    let code = code.trim_end();
    let lang = info.split([' ', ',']).next().unwrap_or("");
    if lang == "jai" {
        highlight::jai(code, &|_| None)
    } else {
        highlight::other(lang, code).unwrap_or_else(|| esc(code))
    }
}

/// `jai before` and `jai after` fences next to each other become one side-by-side block. Words
/// after `after` (`indent_width=2`) are the settings the after side was formatted with.
fn segments(segs: &[Segment]) -> Vec<DocSegment> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < segs.len() {
        match &segs[i] {
            Segment::Html(h) => out.push(DocSegment::Html {
                prose_html: h.clone(),
            }),
            Segment::Code {
                info,
                code,
            } => {
                let words: Vec<&str> = info.split_whitespace().collect();
                let next = segs.get(i + 1).and_then(|s| match s {
                    Segment::Code {
                        info,
                        code,
                    } => Some((info, code)),
                    Segment::Html(_) => None,
                });
                match (words.get(1), next) {
                    (Some(&"before"), Some((after_info, after)))
                        if after_info.split_whitespace().nth(1) == Some("after") =>
                    {
                        let settings: Vec<&str> = after_info.split_whitespace().skip(2).collect();
                        out.push(DocSegment::Compare {
                            before_html: code_html(info, code),
                            after_html: code_html(after_info, after),
                            settings: settings.join(" ").replace('=', " = "),
                        });
                        i += 1;
                    }
                    _ => out.push(DocSegment::Code {
                        code_html: code_html(info, code),
                        wrap: words.first() != Some(&"jai"),
                    }),
                }
            }
        }
        i += 1;
    }
    out
}

impl Guides {
    /// The page to link a header button to: the first `[[doc.page]]`, else the overview of the
    /// section named like `pages_title`.
    pub fn first_href(&self, site: &Site) -> Option<String> {
        let overview = format!("{}.html", site.pages_dir());
        self.pages
            .iter()
            .find(|p| p.heading == site.pages_title && p.path != overview)
            .or_else(|| self.pages.iter().find(|p| p.path == overview))
            .map(|p| site.page_href(&p.path))
    }

    pub fn pages(&self, site: &Site, nav: &[NavCategory]) -> Vec<PageView> {
        self.pages
            .iter()
            .map(|page| {
                let href = site.page_href(&page.path);
                let here = &page.repo;
                let resolve = |dest: &str| -> Option<String> {
                    if dest.contains("://") || dest.starts_with('/') {
                        return Some(dest.to_owned());
                    }
                    let (target, anchor) = dest.split_once('#').unwrap_or((dest, ""));
                    let target = if target.is_empty() {
                        here.clone()
                    } else {
                        repo_path(here, target)
                    };
                    if let Some(to) = self.anchors.get(&(target.clone(), anchor.to_owned())) {
                        return Some(if *to == href {
                            format!("#{anchor}")
                        } else {
                            to.clone()
                        });
                    }
                    let anchor = if anchor.is_empty() {
                        String::new()
                    } else {
                        format!("#{anchor}")
                    };
                    if target == *here {
                        return Some(anchor);
                    }
                    if let Some(to) = self.files.get(&target) {
                        return Some(format!("{to}{anchor}"));
                    }
                    // A module's folder or its `module.jai` opens its API page.
                    let module = site.modules.iter().find(|m| {
                        let file = format!("{}{}", site.source_prefix, m.file);
                        file == target
                            || file
                                .rsplit_once('/')
                                .is_some_and(|(dir, _)| dir == target.trim_end_matches('/'))
                    });
                    if let Some(m) = module {
                        return Some(format!("{}{anchor}", site.module_href(&m.id)));
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
                    &page.md,
                    &Settings {
                        heading_shift: 0,
                        heading_ids: true,
                        link: &resolve,
                    },
                );
                let description = segs
                    .iter()
                    .find_map(|s| match s {
                        Segment::Html(h) if h.starts_with("<p>") => {
                            Some(markdown::plain(h).trim().to_owned())
                        }
                        _ => None,
                    })
                    .unwrap_or_default();
                PageView {
                    head: PageHead::default(),
                    not_found: false,
                    title: site.title(&page.name),
                    description,
                    path: page.path.clone(),
                    side_label: page.heading.clone(),
                    sidebar: Sidebar::Modules {
                        categories: marked(nav, &href),
                    },
                    outline: outline(&page.md),
                    content: Content::Guide {
                        segments: segments(&segs),
                    },
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{promote, repo_path, split};

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

    #[test]
    fn sections_split_at_their_headings() {
        let (intro, groups) = split(
            "# T\n\nIntro.\n\n## A\n\nAbout A.\n\n### one\n\nOne.\n\n```\n### not\n```\n\n### two\n\nTwo.\n",
        );
        assert_eq!(intro, "# T\n\nIntro.\n\n");
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].name, "A");
        assert_eq!(groups[0].prose, "\nAbout A.\n\n");
        let names: Vec<_> = groups[0].entries.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["one", "two"]);
        assert!(groups[0].entries[0].1.contains("### not"));
    }

    #[test]
    fn entry_headings_move_up_two_levels() {
        assert_eq!(
            promote("#### Options\n##### Deep\n```\n#### kept\n```\n## Kept\n"),
            "## Options\n### Deep\n```\n#### kept\n```\n## Kept\n"
        );
    }
}
