//! Resolve compiler models, links, prose and layout into template inputs.
use super::super::example;
use super::super::markdown::{self, Segment, Settings, plain};
use super::super::model::{Item, Kind, Module, Overload, TypeRef};
use super::super::site::{Site, item_line, signature_html, signature_line_html};
use super::{
    Badge, Card, CardCategory, CardGroup, Category, Content, Crumb, Definition, DocSegment, Link,
    Logo, ModuleView, NavCategory, NavGroup, NavLink, Outline, OutlineItem, OverloadView, PageHead,
    PageView, Row, RowGroup, Section, Sidebar, SiteView, Title,
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

fn clip(text: &str, max: usize) -> String {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() <= max {
        return text;
    }
    let cut: String = text.chars().take(max).collect();
    format!(
        "{}...",
        cut.rsplit_once(' ').map_or(cut.as_str(), |(a, _)| a)
    )
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

fn nav(site: &Site, current: Option<usize>) -> Vec<NavCategory> {
    by_category(site)
        .into_iter()
        .map(|(category, members)| NavCategory {
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
        })
        .collect()
}

fn modules_sidebar(site: &Site, current: Option<usize>) -> Sidebar {
    Sidebar::Modules {
        categories: nav(site, current),
    }
}

fn cards(site: &Site) -> Vec<CardCategory> {
    by_category(site)
        .into_iter()
        .map(|(label, members)| CardCategory {
            label: label.into(),
            groups: by_group(site, &members)
                .into_iter()
                .map(|(label, of)| CardGroup {
                    label: label.into(),
                    cards: of
                        .iter()
                        .map(|&i| {
                            let m = &site.modules[i];
                            let summary = lead(&blocks(site, &m.docs, &m.id))
                                .map_or_else(String::new, |(_, h)| clip(&plain(&h), 140));
                            Card {
                                href: site.module_href(&m.id),
                                label: m.title().into(),
                                experimental: m.experimental,
                                category: category(site, m),
                                summary_html: markdown::esc(&summary),
                            }
                        })
                        .collect(),
                })
                .collect(),
        })
        .collect()
}

pub fn site(site: &Site) -> SiteView {
    let mut header_links = Vec::new();
    if !site.pages_title.is_empty() {
        header_links.push(Link {
            href: site.href(&site.pages_dir()),
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
        format: 4,
        name: site.name.clone(),
        site_name: site.name.clone(),
        base: site.base.clone(),
        version: site.version.clone(),
        logo: site.logo.as_ref().map(|file| Logo {
            url: site.href(&format!("assets/{file}")),
            mime: if file.ends_with(".svg") {
                "image/svg+xml"
            } else {
                "image/png"
            }
            .into(),
        }),
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
        guides: Vec::new(),
        guides_index: None,
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
            content: Content::Index {
                categories: cards(site),
            },
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

fn definitions(
    site: &Site,
    mi: usize,
    rows: &[(String, String, String, Vec<TypeRef>)],
    anchor: &str,
) -> Vec<Definition> {
    rows.iter()
        .map(|(raw, ty, doc, refs)| {
            let (name, default) = raw.split_once(" := ").unwrap_or((raw, ""));
            let doc_html = if doc.is_empty() {
                String::new()
            } else {
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
            };
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

fn members(site: &Site, mi: usize, item: &Item, anchor: &str) -> Vec<Definition> {
    let enumerated = matches!(item.kind, Kind::Enum | Kind::EnumFlags);
    definitions(
        site,
        mi,
        &item
            .members
            .iter()
            .map(|f| {
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
    let lines = if item.kind == Kind::Procedure {
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
        platforms: m.item_systems(item).join(", "),
        since: item.since.clone(),
        source_url: site.source_url(file, line),
        lines_html: lines.into_iter().filter(|s| !s.is_empty()).collect(),
        summary_html: lead(&segs).map_or_else(String::new, |(_, h)| h),
        members: members(site, mi, item, ""),
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
        let mut names = Vec::new();
        if id == "procedures" {
            for i in &items {
                if !i.group.is_empty() && !names.contains(&i.group.as_str()) {
                    names.push(i.group.as_str());
                }
            }
        }
        let mut groups = Vec::new();
        if names.len() > 1 {
            for name in names {
                let selected: Vec<_> = items.iter().copied().filter(|i| i.group == name).collect();
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
            if items.iter().any(|i| i.binding) {
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
    let nav = groups
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
    let items = m
        .items
        .iter()
        .map(|i| {
            let group = group_of[&peer_key(i)];
            let same = groups[group];
            let at = same.iter().position(|p| p.name == i.name).unwrap_or(0);
            item(site, mi, i, group, same, at)
        })
        .collect();
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
                    platforms: String::new(),
                    since: String::new(),
                    source_url: site.source_url(&m.file, 1),
                    mono: false,
                },
                available: m.systems().join(", "),
                since: m.since.clone(),
                import: m.id.clone(),
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
    let parameters = definitions(
        site,
        mi,
        &o.params
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
    let returns = definitions(
        site,
        mi,
        &o.returns
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone(), p.doc.clone(), p.refs.clone()))
            .collect::<Vec<_>>(),
        "",
    );
    OverloadView {
        id: format!("o{}", n + 1),
        label: (item.overloads.len() > 1).then(|| format!("overload {}", n + 1)),
        card: item.overloads.len() > 1,
        signature_html: signature_html(site, &item.name, o, mi),
        prose: segments(site, m, &o.file, o.line, &prose),
        parameters,
        returns,
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
            && let Some(other) = site.module_index(module)
            && site.modules[other].item(name).is_some()
            && !(module == m.id && name == item.name)
            && !found.iter().any(|(a, b)| a == module && b == name)
        {
            found.push((module.into(), name.into()));
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
    let summary_segs = if lead_owner {
        own.clone()
    } else {
        item.overloads
            .first()
            .map_or_else(Vec::new, |o| blocks(site, &o.docs, &from))
    };
    let summary_html = lead(&summary_segs).map_or_else(String::new, |(_, h)| h);
    // A lone overload shows its whole description under its signature, so it has no lead.
    let single = item.kind == Kind::Procedure && item.overloads.len() == 1 && !lead_owner;
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
            overload_views.push(overload(
                site,
                mi,
                item,
                n,
                o,
                n == 0 && !lead_owner && !single,
            ));
            if item.overloads.len() > 1 {
                anchors.push(Link {
                    href: format!("#o{}", n + 1),
                    label: labels[n].clone(),
                });
            }
        }
    }
    let member_defs = if item.kind == Kind::Procedure {
        Vec::new()
    } else {
        members(site, mi, item, "m-")
    };
    let members_label = if matches!(item.kind, Kind::Enum | Kind::EnumFlags) {
        "Members"
    } else {
        "Fields"
    }
    .to_owned();
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
            title: Title {
                label: item.name.clone(),
                kind: item.kind.label().into(),
                procedure: item.kind == Kind::Procedure,
                foreign: item.binding,
                category: category(site, m),
                experimental: item.experimental,
                overloads: overloads(item),
                platforms: m.item_systems(item).join(", "),
                since: item.since.clone(),
                source_url: site.source_url(file, line),
                mono: true,
            },
            summary_html: if single {
                String::new()
            } else {
                summary_html
            },
            docs: segments(site, m, file, line, &remaining),
            overloads: overload_views,
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
