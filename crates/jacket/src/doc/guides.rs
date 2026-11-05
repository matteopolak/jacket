//! Extra Markdown pages: `[[doc.page]]` files under one sidebar heading, and `[[doc.section]]`
//! files split into an overview and one page per `###` heading, each its own sidebar heading.
use super::site::Site;
use super::slug;

use super::navigation::{NavCategory, NavGroup, NavLink};
use jacket_config::schema::Doc;
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
    kind: crate::doc::export::contract::DocumentKind,
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

fn document_kind(
    kind: jacket_config::schema::DocPageKind,
) -> crate::doc::export::contract::DocumentKind {
    match kind {
        jacket_config::schema::DocPageKind::Guide => {
            crate::doc::export::contract::DocumentKind::Guide
        }
        jacket_config::schema::DocPageKind::Tool => {
            crate::doc::export::contract::DocumentKind::Tool
        }
    }
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
            kind: crate::doc::export::contract::DocumentKind::Home,
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
            let folder = format!("{}/{}", site.pages_dir(), slug::slug(&stem));
            let repo = repo_path(&format!("{prefix}x"), &page.file);
            out.files.insert(repo.clone(), site.href(&folder));
            links.push(link(&site.href(&folder), &page.name));
            out.pages.push(Page {
                kind: document_kind(page.kind),
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
        let folder = slug::slug(&section.name);
        let (intro, groups) = split(&md);
        let overview = site.href(&folder);
        out.files.insert(repo.clone(), overview.clone());
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
                let id = slug::slug(&g.name);
                out.anchors
                    .insert((repo.clone(), id.clone()), format!("{overview}#{id}"));
                summary.push_str(&format!("## {}\n", g.name));
            }
            summary.push_str(&g.prose);
            summary.push('\n');
            let mut links = Vec::new();
            for (name, body) in &g.entries {
                let id = slug::slug(name);
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
                    kind: document_kind(section.kind),
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
            kind: document_kind(section.kind),
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

    fn resolve(&self, site: &Site, page: &Page, dest: &str) -> Option<String> {
        if dest.contains("://") || dest.starts_with('/') {
            return Some(dest.to_owned());
        }
        let (target, anchor) = dest.split_once('#').unwrap_or((dest, ""));
        let target = if target.is_empty() {
            page.repo.clone()
        } else {
            repo_path(&page.repo, target)
        };
        if let Some(to) = self.anchors.get(&(target.clone(), anchor.to_owned())) {
            return Some(if *to == site.page_href(&page.path) {
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
        if target == page.repo {
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
    }

    /// Resolved Markdown data shares guide splitting, heading IDs and link maps with the site graph.
    pub(in crate::doc) fn documents(
        &self,
        site: &Site,
        publication: &crate::doc::export::publication::Publication,
    ) -> Result<Vec<crate::doc::export::contract::DocumentPage>, String> {
        use crate::doc::export::{
            build,
            contract::{DocumentNode, DocumentPage, Heading, Location, ResolvedText},
            document,
        };
        self.pages
            .iter()
            .map(|page| {
                let id = build::route_id(&page.path);
                let nodes = document::parse(
                    &page.md,
                    0,
                    true,
                    &|dest| {
                        self.resolve(site, page, dest)
                            .map(|href| build::target(site, &href, &page.path))
                            .transpose()
                    },
                    &|info, code| {
                        Ok(DocumentNode::CodeBlock {
                            source: if info.split([' ', ',']).next() == Some("jai") {
                                build::lexical_text(code)
                            } else {
                                ResolvedText {
                                    source: code.into(),
                                    spans: Vec::new(),
                                }
                            },
                            language: info.split([' ', ',']).next().unwrap_or("").into(),
                            info: info.into(),
                        })
                    },
                )?;
                let outline = nodes
                    .iter()
                    .filter_map(|node| match node {
                        DocumentNode::Heading {
                            id: Some(id),
                            level,
                            children,
                        } if *level >= 2 => Some(Heading {
                            id: id.clone(),
                            label: document::plain(children),
                            level: *level,
                        }),
                        _ => None,
                    })
                    .collect();
                let metadata =
                    publication.document(site, &page.path, page.kind, &page.name, &nodes);
                Ok(DocumentPage {
                    id,
                    kind: page.kind,
                    metadata,
                    title: page.name.clone(),
                    source: Some(Location {
                        file: page.repo.clone(),
                        line: 0,
                    }),
                    nodes,
                    outline,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{collect, promote, repo_path, split};
    use crate::doc::site::Site;

    #[test]
    fn semantic_guides_preserve_documents_navigation_and_nested_resolution() {
        use clap::Parser;
        let dir = test_support::TempDir::new("semantic-guides");
        dir.write("home.md", "# Home\n\n[Guide](guide.md#destination).\n");
        dir.write(
            "guide.md",
            "# Guide\n\n## Destination\n\n[Home](home.md).\n",
        );
        dir.write("tools.md", "# Tools\n\nOverview.\n\n## Group\n\n### command\n\nA command.\n\n#### Options\n\n| Key | Value |\n| --- | --- |\n| a | b |\n");
        let doc: jacket_config::schema::Doc = toml::from_str(
            r#"home="home.md"
url="https://fixture.invalid"
[[page]]
file="guide.md"
name="Guide"
[[section]]
file="tools.md"
name="Tools"
kind="tool"
"#,
        )
        .unwrap();
        let mut site = Site::new(
            vec![],
            vec![],
            "/nested/docs/".into(),
            String::new(),
            String::new(),
            String::new(),
            false,
        );
        site.name = "Fixture".into();
        site.pages_title = "Guides".into();
        let current = collect(&site, &dir, "", &doc).unwrap();
        let crate::cli::Command::Doc(args) =
            crate::cli::Cli::try_parse_from(["jacket", "doc", "--export-only"])
                .unwrap()
                .command
        else {
            panic!("doc command")
        };
        let publication =
            crate::doc::export::publication::prepare(&args, &doc, &dir, &dir, &site).unwrap();
        let documents = current.documents(&site, &publication).unwrap();
        use crate::doc::export::contract::{DocumentKind, DocumentNode};
        assert_eq!(
            documents
                .iter()
                .map(|page| (page.title.as_str(), page.kind))
                .collect::<Vec<_>>(),
            [
                ("Fixture", DocumentKind::Home),
                ("Guide", DocumentKind::Guide),
                ("command", DocumentKind::Tool),
                ("Tools", DocumentKind::Tool)
            ]
        );
        assert_eq!(
            current.first_href(&site).as_deref(),
            Some("/nested/docs/guides/guide")
        );
        let links = current
            .nav
            .iter()
            .flat_map(|category| category.groups.iter())
            .flat_map(|group| {
                group.links.iter().map(|link| {
                    (
                        link.href.as_str(),
                        link.label.as_str(),
                        link.depth,
                        link.current,
                        link.experimental,
                    )
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(
            links,
            [
                ("/nested/docs/guides/guide", "Guide", 0, false, false),
                ("/nested/docs/tools", "Overview", 0, false, false),
                ("/nested/docs/tools/command", "command", 0, false, false)
            ]
        );
        assert_eq!(
            current
                .resolve(&site, &current.pages[0], "guide.md#destination")
                .as_deref(),
            Some("/nested/docs/guides/guide#destination")
        );
        assert_eq!(
            current
                .resolve(&site, &current.pages[1], "home.md")
                .as_deref(),
            Some("/nested/docs/")
        );
        assert_eq!(
            current
                .resolve(&site, &current.pages[2], "tools.md#group")
                .as_deref(),
            Some("/nested/docs/tools#group")
        );
        assert!(
            documents[2]
                .nodes
                .iter()
                .any(|node| matches!(node, DocumentNode::Table { .. }))
        );
        assert_eq!(
            documents[2]
                .outline
                .iter()
                .map(|heading| (heading.id.as_str(), heading.label.as_str(), heading.level))
                .collect::<Vec<_>>(),
            [("options", "Options", 2)]
        );
    }

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
