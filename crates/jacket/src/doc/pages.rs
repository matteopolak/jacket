//! The pages of the API reference: the index, a module and an item.
use super::example;
use super::markdown::{self, Segment, Settings, esc, plain};
use super::model::{Item, Kind, Module, Overload};
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

fn badges(m: &Module) -> String {
    let names: Vec<String> = if m.platforms.is_empty() {
        vec!["macOS".into(), "Linux".into(), "Windows".into()]
    } else {
        m.platforms
            .iter()
            .map(|p| match p.as_str() {
                "MACOS" => "macOS".to_owned(),
                "LINUX" => "Linux".to_owned(),
                "WINDOWS" => "Windows".to_owned(),
                "ANDROID" => "Android".to_owned(),
                "WASM" => "wasm".to_owned(),
                "IOS" => "iOS".to_owned(),
                other => other.to_owned(),
            })
            .collect()
    };
    let spans: String = names
        .iter()
        .map(|n| format!("<span class=\"badge\">{}</span>", esc(n)))
        .collect();
    format!("<div class=\"badges\" aria-label=\"Platforms\">{spans}</div>\n")
}

/// `std / Android / Toolchain / item`: one crumb per level of the module's path, linked when
/// that level is a page of its own.
fn crumbs(site: &Site, m: &Module, last: Option<&str>) -> String {
    let mut out = format!(
        "<nav class=\"crumbs\" aria-label=\"Breadcrumb\"><a href=\"{}\">std</a>",
        esc(&site.base)
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

/// One module's link in a list.
fn module_link(site: &Site, m: &Module, current: Option<usize>, i: usize) -> String {
    format!(
        "<a href=\"{}\" title=\"{}\"{}>{}</a>\n",
        esc(&site.module_href(&m.id)),
        esc(&m.id),
        if current == Some(i) {
            " aria-current=\"page\""
        } else {
            ""
        },
        esc(m.title())
    )
}

/// The modules by section and category. Each category folds; the current module's is open.
fn modules_side(site: &Site, current: Option<usize>) -> String {
    let mut out = String::new();
    for section in ["Standard library", "Extensions"] {
        out.push_str(&format!("<div class=\"nav-h\">{section}</div>\n"));
        for category in &site.categories {
            let members: Vec<(usize, &Module)> = site
                .modules
                .iter()
                .enumerate()
                .filter(|(_, m)| m.section() == section && m.category == *category)
                .collect();
            if members.is_empty() {
                continue;
            }
            let open = members.iter().any(|(i, _)| current == Some(*i));
            out.push_str(&format!(
                "<details class=\"nav-cat\" data-key=\"{key}\"{open}><summary>{name}</summary>\n<div class=\"nav-list\">\n",
                key = esc(&format!("{section}/{category}")),
                open = if open { " open" } else { "" },
                name = esc(category)
            ));
            for (i, m) in members {
                out.push_str(&module_link(site, m, current, i));
            }
            out.push_str("</div>\n</details>\n");
        }
    }
    out
}

fn card(site: &Site, m: &Module) -> String {
    let segs = blocks(site, &m.docs, &m.id);
    let summary = lead(&segs).map_or_else(String::new, |(_, html)| clip(&plain(&html), 140));
    format!(
        "<li><a class=\"card\" href=\"{}\"><span class=\"card-name mono\">{}</span><span class=\"card-sum\">{}</span></a></li>\n",
        esc(&site.module_href(&m.id)),
        esc(m.title()),
        esc(&summary)
    )
}

pub fn index_page(site: &Site) -> String {
    let mut main = String::new();
    for section in ["Standard library", "Extensions"] {
        main.push_str(&format!("<h2 class=\"section\">{section}</h2>\n"));
        for category in &site.categories {
            let members: Vec<&Module> = site
                .modules
                .iter()
                .filter(|m| m.section() == section && m.category == *category)
                .collect();
            if members.is_empty() {
                continue;
            }
            main.push_str(&format!(
                "<section class=\"cat\">\n<h3>{}</h3>\n<ul class=\"cards\">\n",
                esc(category)
            ));
            for m in members {
                main.push_str(&card(site, m));
            }
            main.push_str("</ul>\n</section>\n");
        }
    }
    render(
        site,
        &Page {
            title: "Jai standard library - Jacket",
            description: "API documentation for the Jai standard library that ships with Jacket.",
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

/// The right-hand outline of a module page: its sections, with the groups of procedures below.
fn outline_html(entries: &[(String, String, bool)]) -> String {
    if entries.len() < 2 {
        return String::new();
    }
    let mut out = String::from("<div class=\"toc-h\">On this page</div>\n<ul>\n");
    for (id, label, sub) in entries {
        out.push_str(&format!(
            "<li{}><a href=\"#{}\">{}</a></li>\n",
            if *sub {
                " class=\"sub\""
            } else {
                ""
            },
            esc(id),
            esc(label)
        ));
    }
    out.push_str("</ul>\n");
    out
}

/// One entry of a module's list: name, first signature, summary and the first example.
fn row(site: &Site, mi: usize, item: &Item) -> String {
    let m = &site.modules[mi];
    let from = format!("{}.{}", m.id, item.name);
    let md = item_docs(item);
    let segs = blocks(site, md, &from);
    let summary = lead(&segs).map_or_else(String::new, |(_, html)| html);
    let mut pills = String::new();
    if item.kind == Kind::Procedure && item.overloads.len() > 1 {
        pills.push_str(&format!(
            "<span class=\"pill sm\">{} overloads</span>",
            item.overloads.len()
        ));
    } else if item.kind != Kind::Procedure && item.kind != Kind::Constant {
        pills.push_str(&format!(
            "<span class=\"pill sm\">{}</span>",
            item.kind.label()
        ));
    }
    let line = match item.kind {
        Kind::Procedure => item
            .overloads
            .first()
            .map(|o| signature_line_html(site, o, mi))
            .unwrap_or_default(),
        _ => esc(&item_line(item)),
    };
    let line = if line.is_empty() {
        String::new()
    } else {
        format!("<div class=\"sig-line\">{line}</div>\n")
    };
    let summary = if summary.is_empty() {
        String::new()
    } else {
        format!("<p class=\"summary\">{summary}</p>\n")
    };
    let sample = first_example(&segs).map_or_else(String::new, |(info, code)| {
        let (file, line) = docs_place(item);
        example::fence(site, m, (&file, line), (info, code), true)
    });
    format!(
        "<article class=\"row\" id=\"{name}\">\n<div class=\"row-head\"><a class=\"name\" href=\"{href}\">{name}</a>{pills}</div>\n{line}{summary}{sample}</article>\n",
        name = esc(&item.name),
        href = esc(&site.item_href(&m.id, &item.name)),
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
        "<div class=\"titlebar\"><h1>{}</h1><span class=\"pill\">module</span><span class=\"spacer\"></span><a class=\"muted\" href=\"{}\">Source</a></div>\n",
        esc(m.title()),
        esc(&site.source_url(&m.file, 1)),
    ));
    main.push_str(&badges(m));
    main.push_str(&format!(
        "<p class=\"import\"><code>#import \"{}\";</code></p>\n",
        esc(&m.id)
    ));
    let segs = blocks(site, &m.docs, &m.id);
    let mut description = String::new();
    if !segs.is_empty() {
        // Show the text up to the first example, or two blocks; the rest waits behind a button.
        let shown = segs
            .iter()
            .position(|s| matches!(s, Segment::Code { info, .. } if is_jai(info)))
            .filter(|&i| i < 4)
            .map_or(segs.len().min(2), |i| i + 1);
        if let Some((_, html)) = lead(&segs) {
            description = clip(&plain(&html), 160);
        }
        let mut first = true;
        for (i, seg) in segs.iter().enumerate() {
            if i == shown {
                main.push_str("<details class=\"more\"><summary><span class=\"when-closed\">Show more</span><span class=\"when-open\">Show less</span></summary>\n");
            }
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
        if segs.len() > shown {
            main.push_str("</details>\n");
        }
    }

    let mut outline: Vec<(String, String, bool)> = Vec::new();
    let procs: Vec<&Item> = m
        .items
        .iter()
        .filter(|i| i.kind == Kind::Procedure)
        .collect();
    if !procs.is_empty() {
        main.push_str(&heading(2, "procedures", "Procedures"));
        outline.push(("procedures".into(), "Procedures".into(), false));
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
                outline.push((id, (*g).to_owned(), true));
                for p in procs.iter().filter(|p| p.group == *g) {
                    main.push_str(&row(site, mi, p));
                }
                main.push_str("</section>\n");
            }
        } else {
            main.push_str("<section class=\"group\">\n");
            for p in &procs {
                main.push_str(&row(site, mi, p));
            }
            main.push_str("</section>\n");
        }
    }
    let types: Vec<&Item> = m.items.iter().filter(|i| i.kind.is_type()).collect();
    if !types.is_empty() {
        main.push_str(&heading(2, "types", "Types"));
        outline.push(("types".into(), "Types".into(), false));
        main.push_str("<section class=\"group\">\n");
        for t in types {
            main.push_str(&row(site, mi, t));
        }
        main.push_str("</section>\n");
    }
    let constants: Vec<&Item> = m
        .items
        .iter()
        .filter(|i| i.kind == Kind::Constant)
        .collect();
    if !constants.is_empty() {
        main.push_str(&heading(2, "constants", "Constants"));
        outline.push(("constants".into(), "Constants".into(), false));
        main.push_str("<section class=\"group\">\n");
        for c in constants {
            main.push_str(&row(site, mi, c));
        }
        main.push_str("</section>\n");
    }
    render(
        site,
        &Page {
            title: &format!("{} - Jai standard library", m.title()),
            description: &description,
            side: modules_side(site, Some(mi)),
            side_label: "Modules",
            main,
            toc: outline_html(&outline),
            path: &Site::module_path(&m.id),
        },
    )
}

// An item.

fn plural(n: usize, one: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {one}s")
    }
}

/// `name: type` and what it means, two columns that stack on a phone.
fn definition_list(
    site: &Site,
    here: usize,
    rows: &[(String, String, String)],
    anchor: &str,
) -> String {
    let mut out = String::from("<dl class=\"defs\">\n");
    for (name, ty, doc) in rows {
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
                site.type_html(ty, here)
            )
        };
        let name_html = if name.is_empty() {
            String::new()
        } else {
            format!("<span class=\"pname\">{}</span>", esc(name))
        };
        let (name_html, typed) = if name.is_empty() {
            (String::new(), site.type_html(ty, here))
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
    let many = item.overloads.len() > 1;
    let from = format!("{}.{}", m.id, item.name);
    let mut out = format!("<section id=\"o{}\" class=\"overload\">\n", n + 1);
    if many {
        out.push_str(&format!("<h2>Overload {}</h2>\n", n + 1));
    }
    out.push_str(&format!(
        "<pre class=\"sig\"><code>{}</code></pre>\n",
        signature_html(site, &item.name, o, mi)
    ));
    let segs = blocks(site, &o.docs, &from);
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
                (name, ty, p.doc.clone())
            })
            .collect();
        out.push_str("<h3>Parameters</h3>\n");
        out.push_str(&definition_list(site, mi, &rows, ""));
    }
    if !o.returns.is_empty() {
        let rows: Vec<_> = o
            .returns
            .iter()
            .map(|r| (r.name.clone(), r.ty.clone(), r.doc.clone()))
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
            i.group == item.group
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
            i.group == item.group
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
            "<a href=\"{}\"{}>{}</a>\n",
            esc(&site.item_href(&m.id, &other.name)),
            if here {
                " aria-current=\"page\""
            } else {
                ""
            },
            esc(&other.name)
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
        Kind::Procedure => format!(
            "procedure &middot; {}",
            plural(item.overloads.len().max(1), "overload")
        ),
        other => other.label().to_owned(),
    };
    main.push_str(&format!(
        "<div class=\"titlebar\"><h1 class=\"mono\">{}</h1><span class=\"pill\">{kind}</span><span class=\"spacer\"></span><a class=\"muted\" href=\"{}\">Source</a></div>\n",
        esc(&item.name),
        esc(&site.source_url(&source_file, source_line)),
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
        for (n, o) in item.overloads.iter().enumerate() {
            let skip_lead = n == 0 && !lead_owner_is_item;
            main.push_str(&overload_section(site, mi, item, n, o, skip_lead));
            if item.overloads.len() > 1 {
                anchors.push((format!("o{}", n + 1), format!("Overload {}", n + 1)));
            }
        }
    } else {
        if !item.value.is_empty() {
            main.push_str(&format!(
                "<pre class=\"sig\"><code><span class=\"fn\">{}</span> :: {}</code></pre>\n",
                esc(&item.name),
                esc(&item.value)
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
            title: &format!("{} - {} - Jai standard library", item.name, m.title()),
            description: &description,
            side: item_side(site, mi, item, &anchors),
            side_label: &format!("{} module", m.title()),
            main,
            toc: String::new(),
            path: &Site::item_path(&m.id, &item.name),
        },
    )
}
