//! Resolve compiler models, links, prose and layout into template inputs.
use super::super::example;
use super::super::markdown::{self, Segment, Settings};
use super::super::model::{Item, Kind, Module, Overload, Support, TypeRef};
use super::super::site::{Site, item_line, signature_html, signature_line_html};
use super::{
    Badge, Category, Content, Crumb, Definition, DocSegment, Image, Link, ModuleView, NavCategory,
    NavGroup, NavLink, Outline, OutlineItem, OverloadView, PageHead, PageView, Platform, Row,
    RowGroup, Section, Sidebar, SiteView, Title,
};

pub(super) fn blocks(site: &Site, md: &str, from: &str) -> Vec<Segment> {
    blocks_at(site, md, from, 2)
}

fn blocks_at(site: &Site, md: &str, from: &str, shift: usize) -> Vec<Segment> {
    markdown::segments(
        md,
        &Settings {
            heading_shift: shift,
            heading_ids: false,
            link: &|d| site.doc_link(d, from),
        },
    )
}

pub(super) fn lead(segments: &[Segment]) -> Option<(usize, String)> {
    segments.iter().enumerate().find_map(|(i, s)| match s {
        Segment::Html(h) => h
            .strip_prefix("<p>")
            .and_then(|r| r.trim_end().strip_suffix("</p>"))
            .map(|h| (i, h.into())),
        Segment::Code {
            ..
        } => None,
    })
}

fn by_category(site: &Site) -> Vec<(&str, Vec<usize>)> {
    let mut out = vec![("", Vec::new())];
    out.extend(
        site.categories
            .iter()
            .map(|c| (c.name.as_str(), Vec::new())),
    );
    for (i, m) in site.modules.iter().enumerate() {
        out[m.category.map_or(0, |c| c + 1)].1.push(i);
    }
    out.retain(|(_, m)| !m.is_empty());
    out
}

fn by_group<'a>(site: &'a Site, members: &[usize]) -> Vec<(&'a str, Vec<usize>)> {
    if site.groups.is_empty() {
        return vec![("", members.to_vec())];
    }
    site.groups
        .iter()
        .map(|g| {
            (
                g.as_str(),
                members
                    .iter()
                    .copied()
                    .filter(|&i| site.modules[i].group == *g)
                    .collect::<Vec<_>>(),
            )
        })
        .filter(|(_, m)| !m.is_empty())
        .collect()
}

fn category(site: &Site, m: &Module) -> Option<Badge> {
    site.badge_of(m).map(|c| Badge {
        class: format!("cat-{c}"),
        label: site.categories[c].badge.clone(),
        tip: site.categories[c].tip.clone(),
    })
}

/// Shared module sidebar: configured docs precede API categories, e.g. Guides, Tools, Library.
pub(super) fn nav(site: &Site, current: Option<usize>) -> Vec<NavCategory> {
    let mut out = site.extra_nav.clone();
    out.extend(by_category(site).into_iter().map(|(category, members)| {
        NavCategory {
            label: category.into(),
            groups: by_group(site, &members)
                .into_iter()
                .map(|(group, of)| NavGroup {
                    label: group.into(),
                    key: format!("{category}/{group}"),
                    open: of.iter().any(|&i| current == Some(i)),
                    links: of
                        .iter()
                        .map(|&i| {
                            let m = &site.modules[i];
                            NavLink {
                                href: site.module_href(&m.id),
                                label: m.title().into(),
                                title: Some(m.id.clone()),
                                depth: of
                                    .iter()
                                    .filter(|&&p| {
                                        m.id.starts_with(&format!("{}/", site.modules[p].id))
                                    })
                                    .count(),
                                current: current == Some(i),
                                experimental: m.experimental,
                            }
                        })
                        .collect(),
                })
                .collect(),
        }
    }));
    out
}

fn platforms(m: &Module) -> Vec<Platform> {
    chips(m.support(), |name, s| match s {
        Support::Full => format!("Everything works on {name}"),
        Support::Partial(1) => format!("1 item is not available on {name}"),
        Support::Partial(n) => format!("{n} items are not available on {name}"),
        Support::None => format!("Not available on {name}"),
    })
}

fn chips(
    support: Vec<(&'static str, Support)>,
    tip: impl Fn(&str, Support) -> String,
) -> Vec<Platform> {
    support
        .into_iter()
        .map(|(name, s)| {
            let state = match s {
                Support::Full => "full",
                Support::Partial(_) => "partial",
                Support::None => "none",
            };
            Platform {
                name: name.into(),
                state: state.into(),
                tip: tip(name, s),
            }
        })
        .collect()
}

fn modules_sidebar(site: &Site, current: Option<usize>) -> Sidebar {
    Sidebar::Modules {
        categories: nav(site, current),
    }
}

/// An image written below `assets/`, linked with its type.
fn image(site: &Site, file: &str) -> Image {
    Image {
        url: site.href(&format!("assets/{file}")),
        mime: if file.ends_with(".svg") {
            "image/svg+xml"
        } else {
            "image/png"
        }
        .into(),
    }
}

pub fn site(site: &Site) -> SiteView {
    let mut header_links = Vec::new();
    if let Some(href) = &site.pages_href {
        header_links.push(Link {
            href: href.clone(),
            label: site.pages_title.clone(),
        });
    }
    if !site.playground.is_empty() {
        header_links.push(Link {
            href: site.playground.clone(),
            label: "Playground".into(),
        });
    }
    SiteView {
        format: 7,
        name: site.name.clone(),
        site_name: site.name.clone(),
        base: site.base.clone(),
        version: site.version.clone(),
        logo: site.logo.as_deref().map(|file| image(site, file)),
        icon: site.icon.as_deref().map(|file| image(site, file)),
        header_links,
        repo: site.repo.clone(),
        categories: site
            .categories
            .iter()
            .enumerate()
            .map(|(i, c)| Category {
                name: c.name.clone(),
                class: format!("cat-{i}"),
                badge: c.badge.clone(),
                tip: c.tip.clone(),
                color: c.color.clone(),
                color_dark: c.color_dark.clone(),
                styled: !c.badge.is_empty() && !c.color.is_empty(),
                dark_override: c.color_dark != c.color,
            })
            .collect(),
        groups: site.groups.clone(),
        nav: nav(site, None),
        not_found: None,
        index: PageView {
            head: PageHead::default(),
            not_found: false,
            title: site.name.clone(),
            description: site.description.clone(),
            path: String::new(),
            side_label: "Modules".into(),
            sidebar: modules_sidebar(site, None),
            outline: Vec::new(),
            content: Content::Index {},
        },
        module_files: Vec::new(),
        guide_files: Vec::new(),
        written_files: Vec::new(),
        assets: Vec::new(),
        resolve_links: true,
        footer_version: env!("CARGO_PKG_VERSION").into(),
    }
}

fn crumbs(site: &Site, m: &Module, last: Option<&str>) -> Vec<Crumb> {
    let mut out = vec![Crumb {
        label: if site.breadcrumb.is_empty() {
            "Home".into()
        } else {
            site.breadcrumb.clone()
        },
        href: Some(site.base.clone()),
        mono: false,
    }];
    let parts: Vec<_> = m.id.split('/').collect();
    let mut path = String::new();
    for (i, part) in parts.iter().enumerate() {
        if !path.is_empty() {
            path.push('/');
        }
        path.push_str(part);
        let linked =
            site.module_index(&path).is_some() && !(i + 1 == parts.len() && last.is_none());
        out.push(Crumb {
            label: (*part).into(),
            href: linked.then(|| site.module_href(&path)),
            mono: false,
        });
    }
    if let Some(last) = last {
        out.push(Crumb {
            label: last.into(),
            href: None,
            mono: true,
        });
    }
    out
}

fn overloads(item: &Item) -> Option<usize> {
    (item.kind == Kind::Procedure && item.overloads.len() > 1).then_some(item.overloads.len())
}

fn item_docs(item: &Item) -> &str {
    if item.docs.is_empty() {
        item.overloads.first().map_or("", |o| o.docs.as_str())
    } else {
        &item.docs
    }
}

fn docs_place(item: &Item) -> (&str, usize) {
    item.overloads
        .first()
        .map_or((&item.file, item.line), |o| (&o.file, o.line))
}

fn segments(site: &Site, m: &Module, file: &str, line: usize, segs: &[Segment]) -> Vec<DocSegment> {
    segs.iter()
        .map(|s| match s {
            Segment::Html(h) => DocSegment::Html {
                prose_html: h.clone(),
            },
            Segment::Code {
                info,
                code,
            } => DocSegment::Example {
                example: example::view(site, m, (file, line), (info, code), false),
            },
        })
        .collect()
}

/// A doc comment as the inline HTML of a description: one paragraph, links resolved, a code
/// block as a code span.
fn inline_html(site: &Site, doc: &str) -> String {
    if doc.is_empty() {
        return String::new();
    }
    markdown::segments(
        doc,
        &Settings {
            heading_shift: 3,
            heading_ids: false,
            link: &|d| site.doc_link(d, "a parameter"),
        },
    )
    .iter()
    .map(|s| match s {
        Segment::Html(h) => h
            .trim_end()
            .strip_prefix("<p>")
            .and_then(|r| r.strip_suffix("</p>"))
            .unwrap_or(h)
            .to_owned(),
        Segment::Code {
            code, ..
        } => format!("<code>{}</code>", markdown::esc(code.trim())),
    })
    .collect::<Vec<_>>()
    .join(" ")
}

fn definitions(
    site: &Site,
    mi: usize,
    rows: &[(String, String, String, Vec<TypeRef>)],
    anchor: &str,
) -> Vec<Definition> {
    rows.iter()
        .map(|(raw, ty, doc, refs)| {
            let (name, default) = raw.split_once(" := ").unwrap_or((raw, ""));
            let doc_html = inline_html(site, doc);
            Definition {
                id: (!anchor.is_empty()).then(|| format!("{anchor}{raw}")),
                name: name.into(),
                type_html: if ty.is_empty() {
                    String::new()
                } else {
                    site.type_html(ty, refs, mi)
                },
                default_html: if default.is_empty() || name.is_empty() {
                    String::new()
                } else {
                    site.expr_html(default, refs, mi)
                },
                default_separator: if ty.is_empty() {
                    " := "
                } else {
                    " = "
                }
                .into(),
                doc_html,
            }
        })
        .collect()
}

/// The fields or members the page lists in a table: those `keep` accepts.
fn members(
    site: &Site,
    mi: usize,
    item: &Item,
    anchor: &str,
    keep: &dyn Fn(usize) -> bool,
) -> Vec<Definition> {
    let enumerated = matches!(item.kind, Kind::Enum | Kind::EnumFlags);
    definitions(
        site,
        mi,
        &item
            .members
            .iter()
            .enumerate()
            .filter(|(k, _)| keep(*k))
            .map(|(_, f)| {
                (
                    f.name.clone(),
                    if enumerated {
                        String::new()
                    } else {
                        f.ty.clone()
                    },
                    f.doc.clone(),
                    f.refs.clone(),
                )
            })
            .collect::<Vec<_>>(),
        anchor,
    )
}

/// What of `doc` is written above a field in the declaration, and whether that is all of it: a
/// single paragraph of prose is, anything longer or with a code block is cut to its first
/// sentence.
fn comment_text(site: &Site, doc: &str) -> (String, bool) {
    let segments = blocks_at(site, doc, "a parameter", 3);
    let [Segment::Html(h)] = segments.as_slice() else {
        let first = doc.trim_start();
        if first.starts_with("```") || first.starts_with("    ") {
            return (String::new(), false);
        }
        return (first_sentence(doc), false);
    };
    let plain = h.matches("<p>").count() == 1
        && !["<pre", "<ul", "<ol", "<table", "<blockquote", "<h"]
            .iter()
            .any(|tag| h.contains(tag));
    if plain {
        (doc.to_owned(), true)
    } else {
        (first_sentence(doc), false)
    }
}

fn first_sentence(doc: &str) -> String {
    let paragraph = doc.trim().split("\n\n").next().unwrap_or_default();
    let paragraph = paragraph.split_whitespace().collect::<Vec<_>>().join(" ");
    match paragraph.find(". ") {
        Some(end) => paragraph[..=end].to_owned(),
        None => paragraph,
    }
}

/// A struct, union or enum as written with each described field's doc as comment lines above
/// it, and each field's line anchored as `m-<name>`; and which members still need a row of the
/// table, because their description does not fit in a comment or their line was not found.
fn commented_declaration(site: &Site, mi: usize, item: &Item) -> (String, Vec<bool>) {
    let html = declaration(site, mi, item);
    if html.is_empty() {
        return (html, vec![true; item.members.len()]);
    }
    let lines = item.member_lines();
    let mut table: Vec<bool> = item
        .members
        .iter()
        .zip(&lines)
        .map(|(m, line)| line.is_none() && !m.doc.is_empty())
        .collect();
    let rows: Vec<&str> = html.split('\n').collect();
    let source: Vec<&str> = item.declaration.split('\n').collect();
    if rows.len() != source.len() {
        return (html, table);
    }
    let mut at: Vec<Vec<usize>> = vec![Vec::new(); rows.len()];
    for (k, line) in lines.iter().enumerate() {
        if let Some(i) = line {
            at[*i].push(k);
        }
    }
    let mut out = String::with_capacity(html.len() + 256);
    for (i, row) in rows.iter().enumerate() {
        if at[i].is_empty() {
            out.push_str(row);
            if i + 1 < rows.len() {
                out.push('\n');
            }
            continue;
        }
        let indent = source[i].len() - source[i].trim_start().len();
        // Members sharing a line (`a, b: int;`) share its description.
        if let Some(&k) = at[i].iter().find(|&&k| !item.members[k].doc.is_empty()) {
            let (text, whole) = comment_text(site, &item.members[k].doc);
            for &j in &at[i] {
                if !item.members[j].doc.is_empty() {
                    table[j] |= !whole;
                }
            }
            if !text.is_empty() {
                out.push_str(&format!(
                    "<span class=\"fc\" style=\"--i:{indent}\">// {}</span>",
                    inline_html(site, &text)
                ));
            }
        }
        let names: Vec<String> = at[i]
            .iter()
            .map(|&k| markdown::esc(&item.members[k].name))
            .collect();
        let extra: String = names[1..]
            .iter()
            .map(|n| format!("<span id=\"m-{n}\"></span>"))
            .collect();
        out.push_str(&format!(
            "<span class=\"fl\" style=\"--i:{indent}\" id=\"m-{}\">{extra}{row}</span>",
            names[0]
        ));
    }
    (out, table)
}

/// A struct, union or enum as written, highlighted and linked; empty for anything else.
fn declaration(site: &Site, mi: usize, item: &Item) -> String {
    if item.declaration.is_empty() {
        return String::new();
    }
    site.declaration_html(&item.declaration, &item.decl_refs, mi, &item.name)
}

fn row(site: &Site, mi: usize, item: &Item) -> Row {
    let m = &site.modules[mi];
    let segs = blocks(site, item_docs(item), &format!("{}.{}", m.id, item.name));
    let (file, line) = docs_place(item);
    let sample = segs.iter().find_map(|s| match s {
        Segment::Code {
            info,
            code,
        } if info.split([',', ' ']).next() == Some("jai") => {
            Some(example::view(site, m, (file, line), (info, code), true))
        }
        _ => None,
    });
    let declaration_html = declaration(site, mi, item);
    let lines = if !declaration_html.is_empty() {
        Vec::new()
    } else if item.kind == Kind::Procedure {
        item.overloads
            .iter()
            .map(|o| signature_line_html(site, o, mi))
            .collect::<Vec<_>>()
    } else {
        vec![site.type_html(&item_line(item), &item.refs, mi)]
    };
    Row {
        id: item.name.clone(),
        href: site.item_href(&m.id, &item.name),
        kind_badge: if item.kind != Kind::Procedure && item.kind != Kind::Constant {
            item.kind.label().into()
        } else {
            String::new()
        },
        foreign: item.binding,
        experimental: item.experimental,
        overloads: overloads(item),
        platforms: m.item_missing(item).into_iter().map(String::from).collect(),
        since: item.since.clone(),
        source_url: site.source_url(file, line),
        lines_html: lines.into_iter().filter(|s| !s.is_empty()).collect(),
        summary_html: lead(&segs).map_or_else(String::new, |(_, h)| h),
        members: if declaration_html.is_empty() {
            members(site, mi, item, "", &|_| true)
        } else {
            Vec::new()
        },
        declaration_html,
        example: sample,
    }
}

fn heading_id(text: &str) -> String {
    let id: String = text
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    id.split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn outline_heading(id: &str, label: &str, sub: bool) -> Outline {
    Outline::Heading {
        id: id.into(),
        label: label.into(),
        sub,
    }
}

fn outline_items(items: &[&Item]) -> Outline {
    Outline::Items {
        items: items
            .iter()
            .map(|i| OutlineItem {
                id: i.name.clone(),
                label: i.name.clone(),
            })
            .collect(),
    }
}

fn group(
    site: &Site,
    mi: usize,
    items: &[&Item],
    heading: Option<Link>,
    outline: &mut Vec<Outline>,
) -> RowGroup {
    if let Some(h) = &heading {
        outline.push(outline_heading(&h.href, &h.label, true));
    }
    if !items.is_empty() {
        outline.push(outline_items(items));
    }
    RowGroup {
        heading,
        rows: items.iter().map(|i| row(site, mi, i)).collect(),
    }
}

pub fn module(site: &Site, mi: usize) -> ModuleView {
    let m = &site.modules[mi];
    let mut segs = blocks_at(site, &m.docs, &m.id, 1);
    let description = lead(&segs).map_or_else(String::new, |(_, h)| h);
    let mut first = true;
    for s in &mut segs {
        if let Segment::Html(h) = s
            && first
            && h.starts_with("<p>")
        {
            *h = h.replacen("<p>", "<p class=\"lead\">", 1);
            first = false;
        }
    }
    let parameters = definitions(
        site,
        mi,
        &m.parameters
            .iter()
            .map(|p| {
                (
                    if p.default.is_empty() {
                        p.name.clone()
                    } else {
                        format!("{} := {}", p.name, p.default)
                    },
                    p.ty.clone(),
                    p.doc.clone(),
                    p.refs.clone(),
                )
            })
            .collect::<Vec<_>>(),
        "",
    );
    let mut outline = Vec::new();
    if !parameters.is_empty() {
        outline.push(outline_heading("parameters", "Module parameters", false));
    }
    let mut sections = Vec::new();
    for (id, label) in [
        ("procedures", "Procedures"),
        ("types", "Types"),
        ("constants", "Constants"),
    ] {
        let mut items: Vec<_> = m
            .items
            .iter()
            .filter(|i| match id {
                "procedures" => i.kind == Kind::Procedure,
                "types" => i.kind.is_type(),
                _ => i.kind == Kind::Constant,
            })
            .collect();
        items.sort_by_key(|i| i.binding);
        if items.is_empty() {
            continue;
        }
        outline.push(outline_heading(id, label, false));
        // Procedures split by group, with foreign bindings in a group of their own, so each
        // item is listed once.
        let mut names = Vec::new();
        if id == "procedures" {
            for i in items.iter().filter(|i| !i.binding) {
                if !names.contains(&i.group.as_str()) {
                    names.push(i.group.as_str());
                }
            }
        }
        let bindings = items.iter().any(|i| i.binding);
        let mut groups = Vec::new();
        if names.len() > 1 || (bindings && names.iter().any(|n| !n.is_empty())) {
            for name in names {
                let selected: Vec<_> = items
                    .iter()
                    .copied()
                    .filter(|i| !i.binding && i.group == name)
                    .collect();
                let name = if name.is_empty() {
                    "Other"
                } else {
                    name
                };
                groups.push(group(
                    site,
                    mi,
                    &selected,
                    Some(Link {
                        href: format!("g-{}", heading_id(name)),
                        label: name.into(),
                    }),
                    &mut outline,
                ));
            }
            if bindings {
                let selected: Vec<_> = items.iter().copied().filter(|i| i.binding).collect();
                groups.push(group(
                    site,
                    mi,
                    &selected,
                    Some(Link {
                        href: "g-foreign".into(),
                        label: "Foreign".into(),
                    }),
                    &mut outline,
                ));
            }
        } else {
            groups.push(group(site, mi, &items, None, &mut outline));
        }
        sections.push(Section {
            id: id.into(),
            label: label.into(),
            groups,
        });
    }
    if outline
        .iter()
        .filter(|e| matches!(e, Outline::Heading { .. }))
        .count()
        < 2
    {
        outline.clear();
    }
    // Resolve each peer list once. Item pages carry only a group and window offsets.
    let mut peers: std::collections::BTreeMap<_, Vec<&Item>> = std::collections::BTreeMap::new();
    for item in &m.items {
        peers.entry(peer_key(item)).or_default().push(item);
    }
    let groups: Vec<_> = peers.values().collect();
    let group_of: std::collections::BTreeMap<_, _> =
        peers.keys().enumerate().map(|(n, key)| (*key, n)).collect();
    let mut nav: Vec<Vec<NavLink>> = groups
        .iter()
        .map(|same| {
            same.iter()
                .map(|i| NavLink {
                    href: site.item_href(&m.id, &i.name),
                    label: i.name.clone(),
                    title: None,
                    depth: 0,
                    current: false,
                    experimental: i.experimental,
                })
                .collect()
        })
        .collect();
    // Pages of types the module does not export list no neighbours, and no list has them.
    let private_nav = nav.len();
    if !m.private.is_empty() {
        nav.push(Vec::new());
    }
    let mut items: Vec<PageView> = m
        .items
        .iter()
        .map(|i| {
            let group = group_of[&peer_key(i)];
            let same = groups[group];
            let at = same.iter().position(|p| p.name == i.name).unwrap_or(0);
            item(site, mi, i, group, same, at)
        })
        .collect();
    items.extend(m.private.iter().map(|i| {
        let mut page = item(site, mi, i, private_nav, &[], 0);
        if let Content::Item {
            private, ..
        } = &mut page.content
        {
            *private = true;
        }
        if let Sidebar::Item {
            heading, ..
        } = &mut page.sidebar
        {
            heading.clear();
        }
        page
    }));
    ModuleView {
        id: m.id.clone(),
        page: PageView {
            head: PageHead::default(),
            not_found: false,
            title: site.title(&m.id.replace('/', ".")),
            description,
            path: Site::module_path(&m.id),
            side_label: "Modules".into(),
            sidebar: modules_sidebar(site, Some(mi)),
            outline,
            content: Content::Module {
                crumbs: crumbs(site, m, None),
                title: Title {
                    label: m.title().into(),
                    kind: "module".into(),
                    procedure: false,
                    foreign: false,
                    category: category(site, m),
                    experimental: m.experimental,
                    overloads: None,
                    since: String::new(),
                    source_url: site.source_url(&m.file, 1),
                    mono: false,
                },
                platforms: platforms(m),
                since: m.since.clone(),
                import: if m.implicit {
                    String::new()
                } else {
                    m.import_name().to_owned()
                },
                implicit: m.implicit,
                docs: segments(site, m, &m.file, 1, &segs),
                parameters,
                sections,
            },
        },
        items,
        nav,
    }
}

fn overload(
    site: &Site,
    mi: usize,
    item: &Item,
    n: usize,
    o: &Overload,
    skip_lead: bool,
) -> OverloadView {
    let m = &site.modules[mi];
    let segs = if o.docs == item.docs {
        Vec::new()
    } else {
        blocks(site, &o.docs, &format!("{}.{}", m.id, item.name))
    };
    let skip = if skip_lead {
        lead(&segs).map(|(i, _)| i)
    } else {
        None
    };
    let prose: Vec<_> = segs
        .iter()
        .enumerate()
        .filter(|(i, s)| matches!(s, Segment::Html(_)) && skip != Some(*i))
        .map(|(_, s)| s.clone())
        .collect();
    let examples: Vec<_> = segs
        .iter()
        .filter(|s| matches!(s, Segment::Code { .. }))
        .cloned()
        .collect();
    // The signature already shows every parameter and result; only what the docs say about one
    // is added, as a name and its description.
    let rows: Vec<_> = o
        .params
        .iter()
        .map(|p| (p, ""))
        .chain(o.returns.iter().map(|p| (p, "returns")))
        .filter(|(p, _)| !p.doc.trim().is_empty())
        .map(|(p, unnamed)| {
            let name = if p.name.is_empty() {
                unnamed
            } else {
                &p.name
            };
            (name.to_owned(), String::new(), p.doc.clone(), Vec::new())
        })
        .collect();
    let notes = definitions(site, mi, &rows, "");
    OverloadView {
        id: format!("o{}", n + 1),
        label: (item.overloads.len() > 1).then(|| format!("overload {}", n + 1)),
        card: item.overloads.len() > 1,
        signature_html: signature_html(site, &item.name, o, mi),
        prose: segments(site, m, &o.file, o.line, &prose),
        notes,
        examples: segments(site, m, &o.file, o.line, &examples),
    }
}

fn peer_key(item: &Item) -> (bool, &str, bool, bool) {
    (
        item.binding,
        &item.group,
        item.kind.is_type(),
        item.kind == Kind::Procedure,
    )
}

fn see_also(site: &Site, mi: usize, item: &Item, same: &[&Item], at: usize) -> Vec<Link> {
    let m = &site.modules[mi];
    let mut found: Vec<(String, String)> = Vec::new();
    let mut raw = item_docs(item).to_owned();
    for o in &item.overloads {
        raw.push_str(&o.docs);
    }
    let mut rest = raw.as_str();
    while let Some(at) = rest.find("jaidoc:") {
        let tail = &rest[at + 7..];
        let end = tail
            .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '/'))
            .unwrap_or(tail.len());
        if let Some((module, name)) = tail[..end].rsplit_once('/')
            && let Some(other) = site.module_for_link(module)
            && site.modules[other].item(name).is_some()
            && !(other == mi && name == item.name)
            && !found
                .iter()
                .any(|(a, b)| *a == site.modules[other].id && b == name)
        {
            found.push((site.modules[other].id.clone(), name.into()));
        }
        rest = &tail[end..];
    }
    {
        for i in [at.wrapping_sub(2), at.wrapping_sub(1), at + 1, at + 2] {
            if let Some(other) = same.get(i)
                && found.len() < 6
                && !found.iter().any(|(a, b)| *a == m.id && *b == other.name)
            {
                found.push((m.id.clone(), other.name.clone()));
            }
        }
    }
    found.truncate(6);
    found
        .into_iter()
        .map(|(m, n)| Link {
            href: site.item_href(&m, &n),
            label: n,
        })
        .collect()
}

fn item(site: &Site, mi: usize, item: &Item, nav: usize, same: &[&Item], at: usize) -> PageView {
    let m = &site.modules[mi];
    let from = format!("{}.{}", m.id, item.name);
    let own = blocks_at(site, &item.docs, &from, 1);
    let (file, line) = docs_place(item);
    let lead_owner = lead(&own).is_some();
    let leads: Vec<_> = item
        .overloads
        .iter()
        .map(|o| lead(&blocks(site, &o.docs, &from)).map(|(_, h)| h))
        .collect();
    let summary_html = if lead_owner {
        lead(&own).map_or_else(String::new, |(_, h)| h)
    } else {
        leads.first().cloned().flatten().unwrap_or_default()
    };
    // When every overload opens with the same lead it becomes the page's summary and leaves the
    // cards; when they differ each card keeps its own and the page shows none, so nothing repeats.
    let lifted = item.kind == Kind::Procedure
        && !lead_owner
        && leads.first().is_some_and(Option::is_some)
        && leads.iter().all(|l| *l == leads[0]);
    let skip = if lead_owner {
        lead(&own).map(|(i, _)| i)
    } else {
        None
    };
    let remaining: Vec<_> = own
        .into_iter()
        .enumerate()
        .filter(|(i, _)| skip != Some(*i))
        .map(|(_, s)| s)
        .collect();
    let mut anchors = Vec::new();
    let mut overload_views = Vec::new();
    if item.kind == Kind::Procedure {
        let labels = super::super::site::overload_labels(item);
        for (n, o) in item.overloads.iter().enumerate() {
            overload_views.push(overload(site, mi, item, n, o, lifted));
            if item.overloads.len() > 1 {
                anchors.push(Link {
                    href: format!("#o{}", n + 1),
                    label: labels[n].clone(),
                });
            }
        }
    }
    let (declaration_html, table) = commented_declaration(site, mi, item);
    // With the declaration shown, the table is for the fields it could not describe.
    let member_defs = if item.kind == Kind::Procedure {
        Vec::new()
    } else {
        let lines = item.member_lines();
        let mut defs = members(site, mi, item, "m-", &|k| table[k]);
        let shown = (0..item.members.len()).filter(|&k| table[k]);
        for (def, k) in defs.iter_mut().zip(shown) {
            // Its anchor is on its line in the declaration.
            if lines[k].is_some() && !declaration_html.is_empty() {
                def.id = None;
            }
        }
        defs
    };
    let members_label = if matches!(item.kind, Kind::Enum | Kind::EnumFlags) {
        "Members"
    } else {
        "Fields"
    }
    .to_owned();
    if !declaration_html.is_empty() {
        anchors.push(Link {
            href: "#declaration".into(),
            label: "Declaration".into(),
        });
    }
    if !member_defs.is_empty() {
        anchors.push(Link {
            href: "#members".into(),
            label: members_label.clone(),
        });
    }
    let also = see_also(site, mi, item, same, at);
    if !also.is_empty() {
        anchors.push(Link {
            href: "#see".into(),
            label: "See also".into(),
        });
    }
    let (start, end) = if same.len() > 40 {
        let start = at.saturating_sub(20);
        (start, (start + 40).min(same.len()))
    } else {
        (0, same.len())
    };
    let heading = if !item.group.is_empty() {
        item.group.clone()
    } else if item.kind == Kind::Procedure {
        "Procedures".into()
    } else if item.kind.is_type() {
        "Types".into()
    } else {
        "Constants".into()
    };
    PageView {
        head: PageHead::default(),
        not_found: false,
        title: site.title(&format!("{}.{}", m.id.replace('/', "."), item.name)),
        description: summary_html.clone(),
        path: site.item_path(&m.id, &item.name),
        side_label: format!("{} module", m.title()),
        sidebar: Sidebar::Item {
            back: Link {
                href: site.module_href(&m.id),
                label: m.title().into(),
            },
            heading,
            nav,
            start,
            end,
            current: at,
            anchors,
        },
        outline: Vec::new(),
        content: Content::Item {
            crumbs: crumbs(site, m, Some(&item.name)),
            platforms: chips(m.item_support(item), |name, s| match s {
                Support::None => format!("{} is not available on {name}", item.name),
                _ => format!("{} works on {name}", item.name),
            }),
            title: Title {
                label: item.name.clone(),
                kind: item.kind.label().into(),
                procedure: item.kind == Kind::Procedure,
                foreign: item.binding,
                category: category(site, m),
                experimental: item.experimental,
                overloads: overloads(item),
                since: item.since.clone(),
                source_url: site.source_url(file, line),
                mono: true,
            },
            summary_html: if lead_owner || lifted || item.kind != Kind::Procedure {
                summary_html
            } else {
                String::new()
            },
            docs: segments(site, m, file, line, &remaining),
            overloads: overload_views,
            private: false,
            declaration_html,
            value_html: if item.kind != Kind::Procedure && !item.value.is_empty() {
                site.type_html(&item.value, &item.refs, mi)
            } else {
                String::new()
            },
            members_label,
            members: member_defs,
            see_also: also,
        },
    }
}
