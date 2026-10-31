//! The search index the page loads on first use: every item by name and every overload by the
//! types it takes and returns, so `-> bool` and `string -> [] string` find procedures.
use super::markdown::{self, Segment, Settings, plain};
use super::model::Kind;
use super::site::{Site, item_line, signature_marked};
use serde_json::{Value, json};

/// A type with its spaces taken out and its case folded, the form both sides compare.
fn normal(ty: &str) -> String {
    ty.split_whitespace().collect::<String>().to_lowercase()
}

fn summary(site: &Site, md: &str, from: &str) -> String {
    let segs = markdown::segments(
        md,
        &Settings {
            heading_shift: 2,
            heading_ids: false,
            link: &|_| None,
        },
    );
    let _ = (site, from);
    segs.iter()
        .find_map(|s| match s {
            Segment::Html(h) if h.starts_with("<p>") => {
                Some(plain(h).split_whitespace().collect::<Vec<_>>().join(" "))
            }
            _ => None,
        })
        .map(|text| {
            if text.chars().count() > 140 {
                format!(
                    "{}...",
                    text.chars().take(137).collect::<String>().trim_end()
                )
            } else {
                text
            }
        })
        .unwrap_or_default()
}

pub fn index(site: &Site) -> String {
    let modules: Vec<Value> = site.modules.iter().map(|m| json!(m.id)).collect();
    let mut items: Vec<Value> = Vec::new();
    for (mi, m) in site.modules.iter().enumerate() {
        items.push(json!([
            mi,
            "",
            "module",
            summary(site, &m.docs, &m.id),
            "",
            []
        ]));
        for item in &m.items {
            let docs = if item.docs.is_empty() {
                item.overloads.first().map_or("", |o| o.docs.as_str())
            } else {
                item.docs.as_str()
            };
            let shapes: Vec<Value> = item
                .overloads
                .iter()
                .map(|o| {
                    json!([
                        o.params.iter().map(|p| normal(&p.ty)).collect::<Vec<_>>(),
                        o.returns.iter().map(|r| normal(&r.ty)).collect::<Vec<_>>()
                    ])
                })
                .collect();
            let kind = match item.kind {
                Kind::Procedure => "proc",
                Kind::Constant => "const",
                _ => "type",
            };
            items.push(json!([
                mi,
                item.name,
                kind,
                summary(site, docs, &item.name),
                match (item.kind, item.overloads.first()) {
                    (Kind::Procedure, Some(o)) => signature_marked(site, o, mi),
                    _ => item_line(item),
                },
                shapes
            ]));
        }
    }
    json!({ "v": 1, "modules": modules, "items": items }).to_string()
}
