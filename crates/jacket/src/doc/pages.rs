//! The pages of the API reference: the index, a module and an item.
use super::example;
use super::markdown::{self, Segment, Settings, esc, plain};
use super::model::{Item, Kind, Module, Overload, TypeRef};
use super::shell::{Page, render};
use super::site::{Site, item_line, signature_html, signature_line_html};

/// The blocks of a doc comment, links resolved against the site.
fn blocks(site: &Site, md: &str, from: &str) -> Vec<Segment> {
    markdown::segments(
        md,
        &Settings {
            heading_shift: 2,
            heading_ids: false,
            link: &|dest| site.doc_link(dest, from),
        },
    )
}

fn is_jai(info: &str) -> bool {
    info.split([',', ' ']).next() == Some("jai")
}

/// The first paragraph of `segments`, as the HTML inside its `<p>`.
fn lead(segments: &[Segment]) -> Option<(usize, String)> {
    segments.iter().enumerate().find_map(|(i, s)| match s {
        Segment::Html(html) => html
            .strip_prefix("<p>")
            .and_then(|rest| rest.trim_end().strip_suffix("</p>"))
            .map(|inner| (i, inner.to_owned())),
        Segment::Code {
            ..
        } => None,
    })
}

/// The first example of `segments`.
fn first_example(segments: &[Segment]) -> Option<(&str, &str)> {
    segments.iter().find_map(|s| match s {
        Segment::Code {
            info,
            code,
        } if is_jai(info) => Some((info.as_str(), code.as_str())),
        _ => None,
    })
}

fn clip(text: &str, max: usize) -> String {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() <= max {
        return text;
    }
    let cut: String = text.chars().take(max).collect();
    let cut = cut.rsplit_once(' ').map_or(cut.as_str(), |(a, _)| a);
    format!("{cut}...")
}

/// What the dots and badges for each flag say on hover.
const EXPERIMENTAL_TIP: &str = "Experimental: may change or go away";

/// The dot beside a sidebar entry that is experimental.
fn experimental_dot(experimental: bool) -> String {
    if experimental {
        format!(
            "<span class=\"dot exp\" role=\"img\" title=\"{EXPERIMENTAL_TIP}\" aria-label=\"{EXPERIMENTAL_TIP}\"></span>"
        )
    } else {
        String::new()
    }
}

/// The badge of the category `m` is listed under, beside its name or an item's.
fn category_badge(site: &Site, m: &Module, small: bool) -> String {
    let Some(c) = site.badge_of(m) else {
        return String::new();
    };
    let category = &site.categories[c];
    let sm = if small {
        " sm"
    } else {
        ""
    };
    format!(
        "<span class=\"badge cat-{c}{sm}\" title=\"{}\">{}</span>",
        esc(&category.tip),
        esc(&category.badge)
    )
}

/// The badge beside the name of a module or an item tagged `@experimental` (an item is, when its
/// module is: `Site::new`).
fn experimental_badge(experimental: bool, small: bool) -> &'static str {
    match (experimental, small) {
        (false, _) => "",
        (true, false) => {
            "<span class=\"badge exp\" title=\"Experimental: may change or go away\">experimental</span>"
        }
        (true, true) => {
            "<span class=\"badge exp sm\" title=\"Experimental: may change or go away\">experimental</span>"
        }
    }
}

fn platforms_line(m: &Module) -> String {
    let since = if m.since.is_empty() {
        String::new()
    } else {
        format!("<span class=\"meta-sep\">·</span>{}", since_note(&m.since))
    };
    format!(
        "<p class=\"meta\"><span class=\"meta-k\">Platforms</span>{}{since}</p>\n",
        esc(&m.systems().join(", "))
    )
}

/// "3 overloads", small and dim like the release note beside it.
fn overloads_note(item: &Item) -> String {
    if item.kind == Kind::Procedure && item.overloads.len() > 1 {
        format!(
            "<span class=\"since\">{} overloads</span>",
            item.overloads.len()
        )
    } else {
        String::new()
    }
}

/// "since 0.24.0", small and dim; nothing when the release is unknown.
/// The Source link at the right of a title, if the site knows where the source is.
fn source_link(site: &Site, file: &str, line: usize) -> String {
    site.source_url(file, line).map_or_else(String::new, |url| {
        format!(
            "<span class=\"spacer\"></span><a class=\"muted\" href=\"{}\">Source</a>",
            esc(&url)
        )
    })
}

fn since_note(version: &str) -> String {
    if version.is_empty() {
        String::new()
    } else {
        format!("<span class=\"since\">since {}</span>", esc(version))
    }
}

/// "macOS, Linux only", small and dim, for an item that is not available everywhere its module
/// is.
fn item_badges(m: &Module, item: &Item) -> String {
    let systems = m.item_systems(item);
    if systems.is_empty() {
        return String::new();
    }
    format!(
        "<span class=\"since\" title=\"Platforms\">{} only</span>",
        esc(&systems.join(", "))
    )
}

/// `Jacket / Android / Toolchain / item`: the site's name, then one crumb per level of the module's path, linked when
/// that level is a page of its own.
fn crumbs(site: &Site, m: &Module, last: Option<&str>) -> String {
    let mut out = format!(
        "<nav class=\"crumbs\" aria-label=\"Breadcrumb\"><a href=\"{}\">{}</a>",
        esc(&site.base),
        esc(if site.name.is_empty() {
            "Home"
        } else {
            &site.name
        })
    );
    let parts: Vec<&str> = m.id.split('/').collect();
    let mut path = String::new();
    for (i, part) in parts.iter().enumerate() {
        if !path.is_empty() {
            path.push('/');
        }
        path.push_str(part);
        let is_module = i + 1 == parts.len();
        let linked = (site.module_index(&path).is_some()) && !(is_module && last.is_none());
        if linked {
            out.push_str(&format!(
                " / <a href=\"{}\">{}</a>",
                esc(&site.module_href(&path)),
                esc(part)
            ));
        } else {
            out.push_str(&format!(" / <span>{}</span>", esc(part)));
        }
    }
    if let Some(last) = last {
        out.push_str(&format!(" / <span class=\"mono\">{}</span>", esc(last)));
    }
    out.push_str("</nav>\n");
    out
}

// The index.

/// One module's link in a list. `depth` indents a nested module under its parent.
fn module_link(site: &Site, m: &Module, current: Option<usize>, i: usize, depth: usize) -> String {
    format!(
        "<a href=\"{}\" title=\"{}\"{}{}>{}{}</a>\n",
        esc(&site.module_href(&m.id)),
        esc(&m.id),
        if depth > 0 {
            format!(" style=\"--depth:{depth}\"")
        } else {
            String::new()
        },
        if current == Some(i) {
            " aria-current=\"page\""
        } else {
            ""
        },
        esc(m.title()),
        experimental_dot(m.experimental)
    )
}

/// The modules of each category, in display order: those under no heading first, with an
/// empty name.
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
    out.retain(|(_, members)| !members.is_empty());
    out
}

/// `members` split by group, in the groups' order; one unnamed group when there are none.
fn by_group<'a>(site: &'a Site, members: &[usize]) -> Vec<(&'a str, Vec<usize>)> {
    if site.groups.is_empty() {
        return vec![("", members.to_vec())];
    }
    site.groups
        .iter()
        .map(|g| {
            let of: Vec<usize> = members
                .iter()
                .copied()
                .filter(|&i| site.modules[i].group == *g)
                .collect();
            (g.as_str(), of)
        })
        .filter(|(_, of)| !of.is_empty())
        .collect()
}

/// The modules by category and group. Each group folds; the current module's is open.
fn modules_side(site: &Site, current: Option<usize>) -> String {
    let mut out = String::new();
    for (category, members) in by_category(site) {
        if !category.is_empty() {
            out.push_str(&format!("<div class=\"nav-h\">{}</div>\n", esc(category)));
        }
        for (group, of) in by_group(site, &members) {
            let links: String = of
                .iter()
                .map(|&i| {
                    let m = &site.modules[i];
                    // Count the parents listed alongside, so Android/Toolchain sits under Android.
                    let depth = of
                        .iter()
                        .filter(|&&p| m.id.starts_with(&format!("{}/", site.modules[p].id)))
                        .count();
                    module_link(site, m, current, i, depth)
                })
                .collect();
            if group.is_empty() {
                out.push_str(&format!("<div class=\"nav-list\">\n{links}</div>\n"));
                continue;
            }
            let open = of.iter().any(|&i| current == Some(i));
            out.push_str(&format!(
                "<details class=\"nav-cat\" data-key=\"{key}\"{open}><summary>{name}</summary>\n<div class=\"nav-list\">\n{links}</div>\n</details>\n",
                key = esc(&format!("{category}/{group}")),
                open = if open { " open" } else { "" },
                name = esc(group)
            ));
        }
    }
    out
}

fn card(site: &Site, m: &Module) -> String {
    let segs = blocks(site, &m.docs, &m.id);
    let summary = lead(&segs).map_or_else(String::new, |(_, html)| clip(&plain(&html), 140));
    format!(
        "<li><a class=\"card\" href=\"{}\"><span class=\"card-name mono\">{}{}{}</span><span class=\"card-sum\">{}</span></a></li>\n",
        esc(&site.module_href(&m.id)),
        esc(m.title()),
        experimental_badge(m.experimental, true),
        category_badge(site, m, true),
        esc(&summary)
    )
}

pub fn index_page(site: &Site) -> String {
    let mut main = String::new();
    for (category, members) in by_category(site) {
        if !category.is_empty() {
            main.push_str(&format!("<h2 class=\"section\">{}</h2>\n", esc(category)));
        }
        for (group, of) in by_group(site, &members) {
            main.push_str("<section class=\"cat\">\n");
            if !group.is_empty() {
                main.push_str(&format!("<h3>{}</h3>\n", esc(group)));
            }
            main.push_str("<ul class=\"cards\">\n");
            for &i in &of {
                main.push_str(&card(site, &site.modules[i]));
            }
            main.push_str("</ul>\n</section>\n");
        }
    }
    render(
        site,
        &Page {
            title: &site.title("API documentation"),
            description: &site.description,
            side: modules_side(site, None),
            side_label: "Modules",
            main,
            toc: String::new(),
            path: "",
        },
    )
}

// A module.

/// An id for a heading made from `text`: lower case, other characters as dashes.
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

/// A heading that links to itself, so its address can be copied.
fn heading(level: u8, id: &str, text: &str) -> String {
    format!(
        "<h{level} id=\"{id}\">{}<a class=\"anchor\" href=\"#{id}\" aria-label=\"Link to this heading\">#</a></h{level}>\n",
        esc(text)
    )
}

/// One line of a module page's outline: a section, a group of procedures, or an item.
struct Outline {
    id: String,
    label: String,
    level: Level,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Level {
    Section,
    Group,
    Item,
}

/// How many items of a list the outline shows before the rest fold away.
const OUTLINE_ITEMS: usize = 5;

/// The right-hand outline of a module page: its sections and groups, each with its items. A long
/// list of items shows its first few until the reader reaches it, or asks for the rest.
fn outline_html(entries: &[Outline]) -> String {
    if entries.iter().filter(|e| e.level != Level::Item).count() < 2 {
        return String::new();
    }
    let mut out = String::from("<div class=\"toc-h\">On this page</div>\n<ul>\n");
    let mut i = 0;
    while i < entries.len() {
        let e = &entries[i];
        if e.level == Level::Item {
            let run = entries[i..]
                .iter()
                .take_while(|e| e.level == Level::Item)
                .count();
            let items = &entries[i..i + run];
            out.push_str("<li class=\"items\"><ul>\n");
            for (n, item) in items.iter().enumerate() {
                out.push_str(&format!(
                    "<li{}><a href=\"#{}\">{}</a></li>\n",
                    if n >= OUTLINE_ITEMS {
                        " class=\"extra\""
                    } else {
                        ""
                    },
                    esc(&item.id),
                    esc(&item.label)
                ));
            }
            if run > OUTLINE_ITEMS {
                out.push_str(&format!(
                    "<li class=\"more js-only\"><button type=\"button\">Show all {run}</button></li>\n"
                ));
            }
            out.push_str("</ul></li>\n");
            i += run;
            continue;
        }
        out.push_str(&format!(
            "<li{}><a href=\"#{}\">{}</a></li>\n",
            if e.level == Level::Group {
                " class=\"sub\""
            } else {
                ""
            },
            esc(&e.id),
            esc(&e.label)
        ));
        i += 1;
    }
    out.push_str("</ul>\n");
    out
}

/// The outline lines of `items`, linking to their rows.
fn outline_items(items: &[&Item]) -> impl Iterator<Item = Outline> {
    items.iter().map(|i| Outline {
        id: if i.binding {
            Site::slug(&i.name)
        } else {
            i.name.clone()
        },
        label: i.name.clone(),
        level: Level::Item,
    })
}

/// A type's fields or enum members as a full-width band of its row, folded when long.
fn members_band(site: &Site, mi: usize, item: &Item) -> String {
    if item.members.is_empty() {
        return String::new();
    }
    let enumerated = matches!(item.kind, Kind::Enum | Kind::EnumFlags);
    let rows: Vec<_> = item
        .members
        .iter()
        .map(|f| {
            let ty = if enumerated {
                String::new()
            } else {
                f.ty.clone()
            };
            (f.name.clone(), ty, f.doc.clone(), f.refs.clone())
        })
        .collect();
    format!(
        "<div class=\"fold members\" data-fold=\"240\">\n{}</div>\n",
        definition_list(site, mi, &rows, "")
    )
}

/// One entry of a module's list: name, first signature, summary, a type's members and the
/// first example.
fn row(site: &Site, mi: usize, item: &Item) -> String {
    let m = &site.modules[mi];
    let from = format!("{}.{}", m.id, item.name);
    let md = item_docs(item);
    let segs = blocks(site, md, &from);
    let summary = lead(&segs).map_or_else(String::new, |(_, html)| html);
    let overloads = overloads_note(item);
    let mut pills = String::new();
    if item.kind != Kind::Procedure && item.kind != Kind::Constant {
        pills.push_str(&format!(
            "<span class=\"badge sm\">{}</span>",
            item.kind.label()
        ));
    }
    // Every overload gets a line of its own.
    let lines: Vec<String> = match item.kind {
        Kind::Procedure => item
            .overloads
            .iter()
            .map(|o| signature_line_html(site, o, mi))
            .collect(),
        _ => vec![site.type_html(&item_line(item), &item.refs, mi)],
    };
    let line: String = lines
        .iter()
        .filter(|l| !l.is_empty())
        .map(|l| format!("<div class=\"sig-line\">{l}</div>\n"))
        .collect();
    let summary = if summary.is_empty() {
        String::new()
    } else {
        format!("<p class=\"summary\">{summary}</p>\n")
    };
    let sample = first_example(&segs).map_or_else(String::new, |(info, code)| {
        let (file, line) = docs_place(item);
        example::fence(site, m, (&file, line), (info, code), true)
    });
    let members = members_band(site, mi, item);
    if item.binding {
        return foreign_row(site, mi, item, &pills, &line, &members);
    }
    format!(
        "<article class=\"row\" id=\"{name}\">\n<div class=\"row-head\"><a class=\"name\" href=\"{href}\">{name}</a>{pills}{experimental}{overloads}{platforms}{since}</div>\n{line}{summary}{members}{sample}</article>\n",
        name = esc(&item.name),
        href = esc(&site.item_href(&m.id, &item.name)),
        experimental = experimental_badge(item.experimental, true),
        platforms = item_badges(m, item),
        since = since_note(&item.since),
    )
}

/// A declaration of a foreign library the module binds. It has no page of its own, so the row
/// is the whole entry: its signature, and a type's members.
fn foreign_row(
    site: &Site,
    mi: usize,
    item: &Item,
    pills: &str,
    line: &str,
    members: &str,
) -> String {
    let m = &site.modules[mi];
    format!(
        "<article class=\"row\" id=\"{id}\">\n<div class=\"row-head\"><span class=\"name\">{name}</span>{pills}<span class=\"badge sm foreign\" title=\"Declared by a foreign library this module binds\">foreign</span>{platforms}</div>\n{line}{members}</article>\n",
        id = esc(&Site::slug(&item.name)),
        name = esc(&item.name),
        platforms = item_badges(m, item),
    )
}

/// The docs a list shows for an item: its own, else its first overload's.
fn item_docs(item: &Item) -> &str {
    if item.docs.is_empty() {
        item.overloads.first().map_or("", |o| o.docs.as_str())
    } else {
        &item.docs
    }
}

/// The file and line a program built from the item's docs names in its header.
fn docs_place(item: &Item) -> (String, usize) {
    match item.overloads.first() {
        Some(o) => (o.file.clone(), o.line),
        None => (item.file.clone(), item.line),
    }
}

pub fn module_page(site: &Site, mi: usize) -> String {
    let m = &site.modules[mi];
    let mut main = crumbs(site, m, None);
    main.push_str(&format!(
        "<div class=\"titlebar\"><h1>{}</h1><span class=\"badge\">module</span>{}{}{}</div>\n",
        esc(m.title()),
        category_badge(site, m, false),
        experimental_badge(m.experimental, false),
        source_link(site, &m.file, 1),
    ));
    main.push_str(&platforms_line(m));
    main.push_str(&format!(
        "<p class=\"import\"><code>#import \"{}\";</code></p>\n",
        esc(&m.id)
    ));
    let segs = blocks(site, &m.docs, &m.id);
    let mut description = String::new();
    if !segs.is_empty() {
        // A long description is shown faded past a height, with a button to expand it
        // (`.fold` in site.js); without scripts it is shown whole.
        if let Some((_, html)) = lead(&segs) {
            description = clip(&plain(&html), 160);
        }
        main.push_str("<div class=\"fold\">\n");
        let mut first = true;
        for seg in &segs {
            match seg {
                Segment::Html(html) if first && html.starts_with("<p>") => {
                    main.push_str(&html.replacen("<p>", "<p class=\"lead\">", 1));
                    first = false;
                }
                Segment::Html(html) => main.push_str(html),
                Segment::Code {
                    info,
                    code,
                } => main.push_str(&example::fence(site, m, (&m.file, 1), (info, code), false)),
            }
        }
        main.push_str("</div>\n");
    }

    let mut outline: Vec<Outline> = Vec::new();
    let section_line = |id: &str, label: &str, level| Outline {
        id: id.into(),
        label: label.into(),
        level,
    };
    if !m.parameters.is_empty() {
        main.push_str(&heading(2, "parameters", "Module parameters"));
        outline.push(section_line(
            "parameters",
            "Module parameters",
            Level::Section,
        ));
        let rows: Vec<_> = m
            .parameters
            .iter()
            .map(|p| {
                let name = if p.default.is_empty() {
                    p.name.clone()
                } else {
                    format!("{} := {}", p.name, p.default)
                };
                (name, p.ty.clone(), p.doc.clone(), Vec::new())
            })
            .collect();
        main.push_str(&definition_list(site, mi, &rows, ""));
    }
    // Each section lists the module's own items first, then those of a foreign library it binds.
    let section = |keep: &dyn Fn(&Item) -> bool| -> Vec<&Item> {
        let mut items: Vec<&Item> = m.items.iter().filter(|i| keep(i)).collect();
        items.sort_by_key(|i| i.binding);
        items
    };
    let procs = section(&|i| i.kind == Kind::Procedure);
    if !procs.is_empty() {
        main.push_str(&heading(2, "procedures", "Procedures"));
        outline.push(section_line("procedures", "Procedures", Level::Section));
        let mut groups: Vec<&str> = Vec::new();
        for p in &procs {
            if !p.group.is_empty() && !groups.contains(&p.group.as_str()) {
                groups.push(&p.group);
            }
        }
        if groups.len() > 1 {
            for g in &groups {
                let id = format!("g-{}", heading_id(g));
                main.push_str(&format!(
                    "<section class=\"group\">\n{}",
                    heading(3, &id, g)
                ));
                outline.push(section_line(&id, g, Level::Group));
                let members: Vec<&Item> = procs.iter().copied().filter(|p| p.group == *g).collect();
                outline.extend(outline_items(&members));
                for p in procs.iter().filter(|p| p.group == *g) {
                    main.push_str(&row(site, mi, p));
                }
                main.push_str("</section>\n");
            }
            if procs.iter().any(|p| p.binding) {
                main.push_str(&format!(
                    "<section class=\"group\">\n{}",
                    heading(3, "g-foreign", "Foreign")
                ));
                outline.push(section_line("g-foreign", "Foreign", Level::Group));
                let foreign: Vec<&Item> = procs.iter().copied().filter(|p| p.binding).collect();
                outline.extend(outline_items(&foreign));
                for p in procs.iter().filter(|p| p.binding) {
                    main.push_str(&row(site, mi, p));
                }
                main.push_str("</section>\n");
            }
        } else {
            outline.extend(outline_items(&procs));
            main.push_str("<section class=\"group\">\n");
            for p in &procs {
                main.push_str(&row(site, mi, p));
            }
            main.push_str("</section>\n");
        }
    }
    let types = section(&|i| i.kind.is_type());
    if !types.is_empty() {
        main.push_str(&heading(2, "types", "Types"));
        outline.push(section_line("types", "Types", Level::Section));
        outline.extend(outline_items(&types));
        main.push_str("<section class=\"group\">\n");
        for t in types {
            main.push_str(&row(site, mi, t));
        }
        main.push_str("</section>\n");
    }
    let constants = section(&|i| i.kind == Kind::Constant);
    if !constants.is_empty() {
        main.push_str(&heading(2, "constants", "Constants"));
        outline.push(section_line("constants", "Constants", Level::Section));
        outline.extend(outline_items(&constants));
        main.push_str("<section class=\"group\">\n");
        for c in constants {
            main.push_str(&row(site, mi, c));
        }
        main.push_str("</section>\n");
    }
    render(
        site,
        &Page {
            title: &site.title(m.title()),
            description: &description,
            side: modules_side(site, Some(mi)),
            side_label: "Modules",
            main,
            toc: outline_html(&outline),
            path: &Site::module_path(&m.id),
        },
    )
}

/// `name: type` and what it means, two columns that stack on a phone.
fn definition_list(
    site: &Site,
    here: usize,
    rows: &[(String, String, String, Vec<TypeRef>)],
    anchor: &str,
) -> String {
    let mut out = String::from("<dl class=\"defs\">\n");
    for (name, ty, doc, refs) in rows {
        let id = if anchor.is_empty() {
            String::new()
        } else {
            format!(" id=\"{anchor}{}\"", esc(name))
        };
        let typed = if ty.is_empty() {
            String::new()
        } else {
            format!(
                "<span class=\"colon\">: </span>{}",
                site.type_html(ty, refs, here)
            )
        };
        let name_html = if name.is_empty() {
            String::new()
        } else {
            format!("<span class=\"pname\">{}</span>", esc(name))
        };
        let (name_html, typed) = if name.is_empty() {
            (String::new(), site.type_html(ty, refs, here))
        } else {
            (name_html, typed)
        };
        let doc = if doc.is_empty() {
            String::new()
        } else {
            let segs = markdown::segments(
                doc,
                &Settings {
                    heading_shift: 3,
                    heading_ids: false,
                    link: &|d| site.doc_link(d, "a parameter"),
                },
            );
            segs.iter()
                .map(|s| match s {
                    Segment::Html(h) => h
                        .trim_end()
                        .strip_prefix("<p>")
                        .and_then(|r| r.strip_suffix("</p>"))
                        .unwrap_or(h)
                        .to_owned(),
                    Segment::Code {
                        code, ..
                    } => format!("<code>{}</code>", esc(code.trim())),
                })
                .collect::<Vec<_>>()
                .join(" ")
        };
        out.push_str(&format!(
            "<div class=\"def\"{id}><dt class=\"mono\">{name_html}{typed}</dt><dd>{doc}</dd></div>\n"
        ));
    }
    out.push_str("</dl>\n");
    out
}

fn overload_section(
    site: &Site,
    mi: usize,
    item: &Item,
    n: usize,
    o: &Overload,
    skip_lead: bool,
) -> String {
    let m = &site.modules[mi];
    let from = format!("{}.{}", m.id, item.name);
    // A lone overload is the whole page, so it needs no card of its own.
    let class = if item.overloads.len() > 1 {
        " class=\"overload\""
    } else {
        ""
    };
    let mut out = format!("<section id=\"o{}\"{class}>\n", n + 1);
    if item.overloads.len() > 1 {
        out.push_str(&format!(
            "<div class=\"overload-head\"><a href=\"#o{0}\">overload {0}</a></div>\n",
            n + 1
        ));
    }
    out.push_str(&format!(
        "<pre class=\"sig\"><code>{}</code></pre>\n",
        signature_html(site, &item.name, o, mi)
    ));
    // The page already shows the item's own docs, which a lone overload often shares.
    let segs = if o.docs == item.docs {
        Vec::new()
    } else {
        blocks(site, &o.docs, &from)
    };
    let lead_at = skip_lead.then(|| lead(&segs).map(|(i, _)| i)).flatten();
    for (i, seg) in segs.iter().enumerate() {
        if let Segment::Html(html) = seg
            && lead_at != Some(i)
        {
            out.push_str(html);
        }
    }
    if !o.params.is_empty() {
        let rows: Vec<_> = o
            .params
            .iter()
            .map(|p| {
                let ty = if p.ty.is_empty() && !p.default.is_empty() {
                    String::new()
                } else {
                    p.ty.clone()
                };
                let name = if p.default.is_empty() {
                    p.name.clone()
                } else {
                    format!("{} := {}", p.name, p.default)
                };
                let (name, ty) = if ty.is_empty() {
                    (name, ty)
                } else {
                    (p.name.clone(), ty)
                };
                (name, ty, p.doc.clone(), p.refs.clone())
            })
            .collect();
        out.push_str("<h3>Parameters</h3>\n");
        out.push_str(&definition_list(site, mi, &rows, ""));
    }
    if !o.returns.is_empty() {
        let rows: Vec<_> = o
            .returns
            .iter()
            .map(|r| (r.name.clone(), r.ty.clone(), r.doc.clone(), r.refs.clone()))
            .collect();
        out.push_str("<h3>Returns</h3>\n");
        out.push_str(&definition_list(site, mi, &rows, ""));
    }
    for seg in &segs {
        if let Segment::Code {
            info,
            code,
        } = seg
        {
            out.push_str(&example::fence(
                site,
                m,
                (&o.file, o.line),
                (info, code),
                false,
            ));
        }
    }
    out.push_str("</section>\n");
    out
}

/// The items an item's docs link to, then its neighbours, as pills.
fn see_also(site: &Site, mi: usize, item: &Item) -> String {
    let m = &site.modules[mi];
    let mut found: Vec<(String, String)> = Vec::new();
    let mut raw = String::from(item_docs(item));
    for o in &item.overloads {
        raw.push_str(&o.docs);
    }
    let mut rest = raw.as_str();
    while let Some(at) = rest.find("jaidoc:") {
        let tail = &rest[at + "jaidoc:".len()..];
        let end = tail
            .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '/'))
            .unwrap_or(tail.len());
        if let Some((module, name)) = tail[..end].rsplit_once('/')
            && let Some(other) = site.module_index(module)
            && site.modules[other].item(name).is_some()
            && !(module == m.id && name == item.name)
            && !found.iter().any(|(a, b)| a == module && b == name)
        {
            found.push((module.to_owned(), name.to_owned()));
        }
        rest = &tail[end..];
    }
    // The neighbours in the same group make up the list.
    let same: Vec<&Item> = m
        .items
        .iter()
        .filter(|i| {
            !i.binding
                && i.group == item.group
                && i.kind.is_type() == item.kind.is_type()
                && (i.kind == Kind::Procedure) == (item.kind == Kind::Procedure)
        })
        .collect();
    if let Some(at) = same.iter().position(|i| i.name == item.name) {
        let around = [at.wrapping_sub(2), at.wrapping_sub(1), at + 1, at + 2];
        for i in around {
            if let Some(other) = same.get(i)
                && found.len() < 6
                && !found.iter().any(|(a, b)| *a == m.id && *b == other.name)
            {
                found.push((m.id.clone(), other.name.clone()));
            }
        }
    }
    found.truncate(6);
    if found.is_empty() {
        return String::new();
    }
    let pills: String = found
        .iter()
        .map(|(module, name)| {
            format!(
                "<a class=\"pill link mono\" href=\"{}\">{}</a>",
                esc(&site.item_href(module, name)),
                esc(name)
            )
        })
        .collect();
    format!(
        "<section id=\"see\" class=\"see\">\n<h2>See also</h2>\n<div class=\"pills\">{pills}</div>\n</section>\n"
    )
}

fn item_side(site: &Site, mi: usize, item: &Item, anchors: &[(String, String)]) -> String {
    let m = &site.modules[mi];
    let mut out = format!(
        "<a class=\"back\" href=\"{}\">&larr; {}</a>\n",
        esc(&site.module_href(&m.id)),
        esc(m.title())
    );
    let same: Vec<&Item> = m
        .items
        .iter()
        .filter(|i| {
            !i.binding
                && i.group == item.group
                && (i.kind == Kind::Procedure) == (item.kind == Kind::Procedure)
                && i.kind.is_type() == item.kind.is_type()
        })
        .collect();
    let at = same.iter().position(|i| i.name == item.name).unwrap_or(0);
    let (from, to) = if same.len() > 40 {
        let from = at.saturating_sub(20);
        (from, (from + 40).min(same.len()))
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
    out.push_str(&format!(
        "<div class=\"nav-h\">{}</div>\n<div class=\"nav-list mono\">\n",
        esc(&heading)
    ));
    for other in &same[from..to] {
        let here = other.name == item.name;
        out.push_str(&format!(
            "<a href=\"{}\"{}>{}{}</a>\n",
            esc(&site.item_href(&m.id, &other.name)),
            if here {
                " aria-current=\"page\""
            } else {
                ""
            },
            esc(&other.name),
            experimental_dot(other.experimental)
        ));
    }
    out.push_str("</div>\n");
    if !anchors.is_empty() {
        out.push_str("<div class=\"nav-h\">On this page</div>\n<div class=\"nav-list\">\n");
        for (id, label) in anchors {
            out.push_str(&format!("<a href=\"#{}\">{}</a>\n", esc(id), esc(label)));
        }
        out.push_str("</div>\n");
    }
    out
}

pub fn item_page(site: &Site, mi: usize, item: &Item) -> String {
    let m = &site.modules[mi];
    let from = format!("{}.{}", m.id, item.name);
    let own = blocks(site, &item.docs, &from);
    let (source_file, source_line) = docs_place(item);
    let mut main = crumbs(site, m, Some(&item.name));
    let kind = match item.kind {
        Kind::Procedure => "<span class=\"badge proc\">procedure</span>".to_owned(),
        other => format!("<span class=\"badge\">{}</span>", other.label()),
    };
    main.push_str(&format!(
        "<div class=\"titlebar\"><h1 class=\"mono\">{}</h1>{kind}{extension}{experimental}{overloads}{platforms}{since}{}</div>\n",
        esc(&item.name),
        source_link(site, &source_file, source_line),
        extension = category_badge(site, m, false),
        experimental = experimental_badge(item.experimental, false),
        overloads = overloads_note(item),
        platforms = item_badges(m, item),
        since = since_note(&item.since),
    ));
    let mut anchors: Vec<(String, String)> = Vec::new();
    let mut description = String::new();
    // The summary is the first paragraph of the item's docs, or of its first overload.
    let lead_owner_is_item = lead(&own).is_some();
    let summary_segs = if lead_owner_is_item {
        own.clone()
    } else {
        item.overloads
            .first()
            .map_or_else(Vec::new, |o| blocks(site, &o.docs, &from))
    };
    if let Some((_, html)) = lead(&summary_segs) {
        description = clip(&plain(&html), 160);
        main.push_str(&format!("<p class=\"lead\">{html}</p>\n"));
    }
    // The rest of the item's own docs.
    let skip = if lead_owner_is_item {
        lead(&own).map(|(i, _)| i)
    } else {
        None
    };
    for (i, seg) in own.iter().enumerate() {
        match seg {
            Segment::Html(html) if skip != Some(i) => main.push_str(html),
            Segment::Code {
                info,
                code,
            } => main.push_str(&example::fence(
                site,
                m,
                (&source_file, source_line),
                (info, code),
                false,
            )),
            Segment::Html(_) => {}
        }
    }
    if matches!(item.kind, Kind::Procedure) {
        let labels = super::site::overload_labels(item);
        for (n, o) in item.overloads.iter().enumerate() {
            let skip_lead = n == 0 && !lead_owner_is_item;
            main.push_str(&overload_section(site, mi, item, n, o, skip_lead));
            if item.overloads.len() > 1 {
                anchors.push((format!("o{}", n + 1), labels[n].clone()));
            }
        }
    } else {
        if !item.value.is_empty() {
            main.push_str(&format!(
                "<pre class=\"sig\"><code><span class=\"fn\">{}</span> :: {}</code></pre>\n",
                esc(&item.name),
                site.type_html(&item.value, &item.refs, mi)
            ));
        }
        if !item.members.is_empty() {
            let label = if matches!(item.kind, Kind::Enum | Kind::EnumFlags) {
                "Members"
            } else {
                "Fields"
            };
            let rows: Vec<_> = item
                .members
                .iter()
                .map(|f| {
                    (
                        f.name.clone(),
                        if matches!(item.kind, Kind::Enum | Kind::EnumFlags) {
                            String::new()
                        } else {
                            f.ty.clone()
                        },
                        f.doc.clone(),
                        f.refs.clone(),
                    )
                })
                .collect();
            main.push_str(&format!("<section id=\"members\">\n<h2>{label}</h2>\n"));
            main.push_str(&definition_list(site, mi, &rows, "m-"));
            main.push_str("</section>\n");
            anchors.push(("members".into(), label.into()));
        }
    }
    let also = see_also(site, mi, item);
    if !also.is_empty() {
        anchors.push(("see".into(), "See also".into()));
    }
    main.push_str(&also);
    render(
        site,
        &Page {
            title: &site.title(&format!("{}.{}", m.title(), item.name)),
            description: &description,
            side: item_side(site, mi, item, &anchors),
            side_label: &format!("{} module", m.title()),
            main,
            toc: String::new(),
            path: &site.item_path(&m.id, &item.name),
        },
    )
}

#[cfg(test)]
mod tests;
