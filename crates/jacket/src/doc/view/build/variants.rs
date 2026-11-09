//! The definitions of a name that differ by system, as an item page and a module row show them.
use super::super::super::model::{Item, Kind, system_name};
use super::super::super::site::Site;
use super::super::VariantView;
use super::{blocks_at, docs_place, members_label, segments, written};

/// The systems of `platforms` in words: `macOS`, `Linux and Windows`.
pub(super) fn system_list(platforms: &[String]) -> String {
    if platforms.is_empty() {
        return "Every system".into();
    }
    let names: Vec<&str> = platforms
        .iter()
        .map(|p| system_name(p).unwrap_or(p.as_str()))
        .collect();
    match names.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} and {last}", rest.join(", ")),
        _ => names.join(""),
    }
}

/// Every definition of an item with the systems it is for, the item's own first.
pub(super) fn definitions_of(item: &Item) -> Vec<(&Item, &[String])> {
    std::iter::once((item, item.defined_on.as_slice()))
        .chain(item.variants.iter().map(|v| (v, v.platforms.as_slice())))
        .collect()
}

/// Each definition of an item that differs by system, the item's own first, under the systems it
/// is for; empty for an item with one definition. Anchors start with `d<n>-`.
pub(super) fn variant_views(site: &Site, mi: usize, item: &Item) -> Vec<VariantView> {
    if item.variants.is_empty() {
        return Vec::new();
    }
    let m = &site.modules[mi];
    let from = format!("{}.{}", m.id, item.name);
    definitions_of(item)
        .iter()
        .enumerate()
        .map(|(n, (d, platforms))| {
            let (declaration_html, member_defs) = written(site, mi, d, &format!("d{}-m-", n + 1));
            let (file, line) = docs_place(d);
            // The first definition's docs are the page's own.
            let docs = if n == 0 {
                Vec::new()
            } else {
                segments(site, m, file, line, &blocks_at(site, &d.docs, &from, 1))
            };
            VariantView {
                id: format!("d{}", n + 1),
                label: system_list(platforms),
                declaration_html,
                value_html: if d.kind != Kind::Procedure && !d.value.is_empty() {
                    site.type_html(&d.value, &d.refs, mi)
                } else {
                    String::new()
                },
                docs,
                members_label: members_label(d),
                members: member_defs,
            }
        })
        .collect()
}
