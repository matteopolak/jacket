//! API page templates. All choices, links and fragments come from the view model.
use super::markdown::esc;
use super::view::{
    Badge, Content, Crumb, Definition, DocSegment, NavLink, Outline, OverloadView, PageView, Row,
    Sidebar, SiteView, Title,
};
use super::{example, shell};

const EXPERIMENTAL_TIP: &str = "Experimental: may change or go away";


/// Ends every `.fold`, so a short one is opened before the page first paints. Without it, a
/// long one would show whole until `site.js` ran (`fold` in `shell.rs`).
const FOLD: &str = "<script>fold()</script>";

fn experimental_dot(experimental: bool) -> String {
    if experimental {
        format!(
            "<span class=\"dot exp\" role=\"img\" title=\"{EXPERIMENTAL_TIP}\" aria-label=\"{EXPERIMENTAL_TIP}\"></span>"
        )
    } else {
        String::new()
    }
}

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

fn category_badge(category: Option<&Badge>, small: bool) -> String {
    category.map_or_else(String::new, |c| {
        format!(
            "<span class=\"badge {}{}\" title=\"{}\">{}</span>",
            esc(&c.class),
            if small {
                " sm"
            } else {
                ""
            },
            esc(&c.tip),
            esc(&c.label)
        )
    })
}

fn foreign_badge(foreign: bool, small: bool) -> String {
    if foreign {
        format!(
            "<span class=\"badge{} foreign\" title=\"Declared by a foreign library this module binds\">foreign</span>",
            if small {
                " sm"
            } else {
                ""
            }
        )
    } else {
        String::new()
    }
}

fn since_note(version: &str) -> String {
    if version.is_empty() {
        String::new()
    } else {
        format!("<span class=\"since\">since {}</span>", esc(version))
    }
}

fn overloads_note(count: Option<usize>) -> String {
    count.map_or_else(String::new, |n| {
        format!("<span class=\"since\">{n} overloads</span>")
    })
}

fn platforms_note(platforms: &str) -> String {
    if platforms.is_empty() {
        String::new()
    } else {
        format!(
            "<span class=\"since\" title=\"Available only on these systems\">{} only</span>",
            esc(platforms)
        )
    }
}

fn source_link(url: Option<&str>) -> String {
    url.map_or_else(String::new, |u| {
        format!(
            "<span class=\"spacer\"></span><a class=\"muted\" href=\"{}\">Source</a>",
            esc(u)
        )
    })
}

fn crumbs(crumbs: &[Crumb]) -> String {
    let mut out = String::from("<nav class=\"crumbs\" aria-label=\"Breadcrumb\">");
    for (n, c) in crumbs.iter().enumerate() {
        if n > 0 {
            out.push_str(" / ");
        }
        if let Some(href) = &c.href {
            out.push_str(&format!("<a href=\"{}\">{}</a>", esc(href), esc(&c.label)));
        } else {
            out.push_str(&format!(
                "<span{}>{}</span>",
                if c.mono {
                    " class=\"mono\""
                } else {
                    ""
                },
                esc(&c.label)
            ));
        }
    }
    out.push_str("</nav>\n");
    out
}

fn title(title: &Title) -> String {
    let kind = if title.procedure {
        "<span class=\"badge proc\">procedure</span>".into()
    } else {
        format!("<span class=\"badge\">{}</span>", esc(&title.kind))
    };
    format!(
        "<div class=\"titlebar\"><h1{}>{}</h1>{kind}{}{}{}{}{}{}{}</div>\n",
        if title.mono {
            " class=\"mono\""
        } else {
            ""
        },
        esc(&title.label),
        foreign_badge(title.foreign, false),
        category_badge(title.category.as_ref(), false),
        experimental_badge(title.experimental, false),
        overloads_note(title.overloads),
        platforms_note(&title.platforms),
        since_note(&title.since),
        source_link(title.source_url.as_deref())
    )
}

fn nav_link(link: &NavLink) -> String {
    format!(
        "<a href=\"{}\"{}{}{}>{}{}</a>\n",
        esc(&link.href),
        link.title
            .as_ref()
            .map_or_else(String::new, |t| format!(" title=\"{}\"", esc(t))),
        if link.depth > 0 {
            format!(" style=\"--depth:{}\"", link.depth)
        } else {
            String::new()
        },
        if link.current {
            " aria-current=\"page\""
        } else {
            ""
        },
        esc(&link.label),
        experimental_dot(link.experimental)
    )
}

pub fn sidebar(side: &Sidebar) -> String {
    let mut out = String::new();
    match side {
        Sidebar::Modules {
            categories,
        } => {
            for c in categories {
                if !c.label.is_empty() {
                    out.push_str(&format!("<div class=\"nav-h\">{}</div>\n", esc(&c.label)));
                }
                for g in &c.groups {
                    let links: String = g.links.iter().map(nav_link).collect();
                    if g.label.is_empty() {
                        out.push_str(&format!("<div class=\"nav-list\">\n{links}</div>\n"));
                    } else {
                        out.push_str(&format!("<details class=\"nav-cat\" data-key=\"{}\"{}><summary>{}</summary>\n<div class=\"nav-list\">\n{links}</div>\n</details>\n",esc(&g.key),if g.open { " open" } else { "" },esc(&g.label)));
                    }
                }
            }
        }
        Sidebar::Item {
            back,
            heading,
            items,
            anchors,
        } => {
            out.push_str(&format!(
                "<a class=\"back\" href=\"{}\">&larr; {}</a>\n",
                esc(&back.href),
                esc(&back.label)
            ));
            out.push_str(&format!(
                "<div class=\"nav-h\">{}</div>\n<div class=\"nav-list mono\">\n",
                esc(heading)
            ));
            for link in items {
                out.push_str(&nav_link(link));
            }
            out.push_str("</div>\n");
            if !anchors.is_empty() {
                out.push_str("<div class=\"nav-h\">On this page</div>\n<div class=\"nav-list\">\n");
                for a in anchors {
                    out.push_str(&format!(
                        "<a href=\"{}\">{}</a>\n",
                        esc(&a.href),
                        esc(&a.label)
                    ));
                }
                out.push_str("</div>\n");
            }
        }
        Sidebar::Guides {
            heading,
            links,
        } => {
            out.push_str(&format!(
                "<div class=\"nav-h\">{}</div>\n<div class=\"nav-list\">\n",
                esc(heading)
            ));
            for link in links {
                out.push_str(&nav_link(link));
            }
            out.push_str("</div>\n");
        }
    }
    out
}

pub fn outline(entries: &[Outline]) -> String {
    if entries.is_empty() {
        return String::new();
    }
    let mut out = String::from("<div class=\"toc-h\">On this page</div>\n<ul>\n");
    for e in entries {
        match e {
            Outline::Heading {
                id,
                label,
                sub,
            } => out.push_str(&format!(
                "<li{}><a href=\"#{}\">{}</a></li>\n",
                if *sub {
                    " class=\"sub\""
                } else {
                    ""
                },
                esc(id),
                esc(label)
            )),
            Outline::Items {
                items,
                show_all,
            } => {
                out.push_str("<li class=\"items\"><ul>\n");
                for i in items {
                    out.push_str(&format!(
                        "<li{}><a href=\"#{}\">{}</a></li>\n",
                        if i.extra {
                            " class=\"extra\""
                        } else {
                            ""
                        },
                        esc(&i.id),
                        esc(&i.label)
                    ));
                }
                if let Some(n) = show_all {
                    out.push_str(&format!("<li class=\"more js-only\"><button type=\"button\">Show all {n}</button></li>\n"));
                }
                out.push_str("</ul></li>\n");
            }
        }
    }
    out.push_str("</ul>\n");
    out
}

fn heading(level: u8, id: &str, label: &str) -> String {
    format!(
        "<h{level} id=\"{id}\">{}<a class=\"anchor\" href=\"#{id}\" aria-label=\"Link to this heading\">#</a></h{level}>\n",
        esc(label)
    )
}

pub fn segments(segments: &[DocSegment]) -> String {
    segments
        .iter()
        .map(|s| match s {
            DocSegment::Html {
                prose_html,
            } => prose_html.clone(),
            DocSegment::Example {
                example: e,
            } => example::render(e),
            DocSegment::Code {
                code_html,
            } => format!("<pre class=\"code\"><code>{code_html}</code></pre>\n"),
        })
        .collect()
}

fn definitions(rows: &[Definition]) -> String {
    let mut out = String::from("<dl class=\"defs\">\n");
    for r in rows {
        let id =
            r.id.as_ref()
                .map_or_else(String::new, |id| format!(" id=\"{}\"", esc(id)));
        let name = if r.name.is_empty() {
            String::new()
        } else {
            format!("<span class=\"pname\">{}</span>", esc(&r.name))
        };
        let typed = if r.type_html.is_empty() || r.name.is_empty() {
            r.type_html.clone()
        } else {
            format!("<span class=\"colon\">: </span>{}", r.type_html)
        };
        let default = if r.default_html.is_empty() {
            String::new()
        } else {
            format!(
                "<span class=\"colon\">{}</span>{}",
                r.default_separator, r.default_html
            )
        };
        out.push_str(&format!("<div class=\"def\"{id}><dt class=\"mono\">{name}{typed}{default}</dt><dd>{}</dd></div>\n",r.doc_html));
    }
    out.push_str("</dl>\n");
    out
}

fn row(r: &Row) -> String {
    let pills = if r.kind_badge.is_empty() {
        String::new()
    } else {
        format!("<span class=\"badge sm\">{}</span>", esc(&r.kind_badge))
    };
    let line: String = r
        .lines_html
        .iter()
        .map(|l| format!("<div class=\"sig-line\">{l}</div>\n"))
        .collect();
    let summary = if r.summary_html.is_empty() {
        String::new()
    } else {
        format!("<p class=\"summary\">{}</p>\n", r.summary_html)
    };
    let members = if r.members.is_empty() {
        String::new()
    } else {
        format!(
            "<div class=\"fold members\" data-fold=\"240\">\n{}{FOLD}</div>\n",
            definitions(&r.members)
        )
    };
    let sample = r.example.as_ref().map_or_else(String::new, example::render);
    format!(
        "<article class=\"row\" id=\"{name}\">\n<div class=\"row-head\"><a class=\"name\" href=\"{href}\">{name}</a>{pills}{foreign}{experimental}{overloads}{platforms}{since}{source}</div>\n{line}{summary}{members}{sample}</article>\n",
        name = esc(&r.id),
        href = esc(&r.href),
        foreign = foreign_badge(r.foreign, true),
        experimental = experimental_badge(r.experimental, true),
        overloads = overloads_note(r.overloads),
        platforms = platforms_note(&r.platforms),
        since = since_note(&r.since),
        source = source_link(r.source_url.as_deref())
    )
}

fn overload(o: &OverloadView) -> String {
    let mut out = format!(
        "<section id=\"{}\"{}>\n",
        esc(&o.id),
        if o.card {
            " class=\"overload\""
        } else {
            ""
        }
    );
    if let Some(label) = &o.label {
        out.push_str(&format!(
            "<div class=\"overload-head\"><a href=\"#{}\">{}</a></div>\n",
            esc(&o.id),
            esc(label)
        ));
    }
    out.push_str(&format!(
        "<pre class=\"sig\"><code>{}</code></pre>\n",
        o.signature_html
    ));
    out.push_str(&segments(&o.prose));
    if !o.parameters.is_empty() {
        out.push_str("<h3>Parameters</h3>\n");
        out.push_str(&definitions(&o.parameters));
    }
    if !o.returns.is_empty() {
        out.push_str("<h3>Returns</h3>\n");
        out.push_str(&definitions(&o.returns));
    }
    out.push_str(&segments(&o.examples));
    out.push_str("</section>\n");
    out
}

pub fn content(content: &Content) -> String {
    let mut out = String::new();
    match content {
        Content::Index {
            categories,
        } => {
            for c in categories {
                if !c.label.is_empty() {
                    out.push_str(&format!("<h2 class=\"section\">{}</h2>\n", esc(&c.label)));
                }
                for g in &c.groups {
                    out.push_str("<section class=\"cat\">\n");
                    if !g.label.is_empty() {
                        out.push_str(&format!("<h3>{}</h3>\n", esc(&g.label)));
                    }
                    out.push_str("<ul class=\"cards\">\n");
                    for c in &g.cards {
                        out.push_str(&format!("<li><a class=\"card\" href=\"{}\"><span class=\"card-name mono\">{}{}{}</span><span class=\"card-sum\">{}</span></a></li>\n",esc(&c.href),esc(&c.label),experimental_badge(c.experimental,true),category_badge(c.category.as_ref(),true),c.summary_html));
                    }
                    out.push_str("</ul>\n</section>\n");
                }
            }
        }
        Content::Module {
            crumbs: c,
            title: t,
            available,
            since,
            import,
            docs,
            parameters,
            sections,
        } => {
            out.push_str(&crumbs(c));
            out.push_str(&title(t));
            let since = if since.is_empty() {
                String::new()
            } else {
                format!("<span class=\"meta-sep\">·</span>{}", since_note(since))
            };
            out.push_str(&format!(
                "<p class=\"meta\">Available on {}{since}</p>\n",
                esc(available)
            ));
            out.push_str(&format!(
                "<p class=\"import\"><code>#import \"{}\";</code></p>\n",
                esc(import)
            ));
            if !docs.is_empty() {
                out.push_str("<div class=\"fold\">\n");
                out.push_str(&segments(docs));
                out.push_str(FOLD);
                out.push_str("</div>\n");
            }
            if !parameters.is_empty() {
                out.push_str(&heading(2, "parameters", "Module parameters"));
                out.push_str(&definitions(parameters));
            }
            for s in sections {
                out.push_str(&heading(2, &s.id, &s.label));
                for g in &s.groups {
                    out.push_str("<section class=\"group\">\n");
                    if let Some(h) = &g.heading {
                        out.push_str(&heading(3, &h.href, &h.label));
                    }
                    for r in &g.rows {
                        out.push_str(&row(r));
                    }
                    out.push_str("</section>\n");
                }
            }
        }
        Content::Item {
            crumbs: c,
            title: t,
            summary_html,
            docs,
            overloads,
            value_html,
            members_label,
            members,
            see_also,
        } => {
            out.push_str(&crumbs(c));
            out.push_str(&title(t));
            if !summary_html.is_empty() {
                out.push_str(&format!("<p class=\"lead\">{summary_html}</p>\n"));
            }
            out.push_str(&segments(docs));
            for o in overloads {
                out.push_str(&overload(o));
            }
            if !value_html.is_empty() {
                out.push_str(&format!("<pre class=\"sig\"><code><span class=\"fn\">{}</span> :: {value_html}</code></pre>\n",esc(&t.label)));
            }
            if !members.is_empty() {
                out.push_str(&format!(
                    "<section id=\"members\">\n<h2>{members_label}</h2>\n"
                ));
                out.push_str(&definitions(members));
                out.push_str("</section>\n");
            }
            if !see_also.is_empty() {
                out.push_str(
                    "<section id=\"see\" class=\"see\">\n<h2>See also</h2>\n<div class=\"pills\">",
                );
                for p in see_also {
                    out.push_str(&format!(
                        "<a class=\"pill link mono\" href=\"{}\">{}</a>",
                        esc(&p.href),
                        esc(&p.label)
                    ));
                }
                out.push_str("</div>\n</section>\n");
            }
        }
        Content::Guide {
            ..
        }
        | Content::GuidesIndex {
            ..
        } => out.push_str(&super::guide::content(content)),
    }
    out
}

pub fn render(site: &SiteView, page: &PageView) -> String {
    shell::render(site, page)
}

#[cfg(test)]
mod tests;
