//! The documentation model as JSON.
use super::*;

/// `field` as a JSON string literal.
pub(in crate::doc_comments) fn json(field: &str) -> String {
    let mut out = String::with_capacity(field.len() + 2);
    out.push('"');
    for c in field.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A file as it is named in the model: canonical, so one file has one name whichever way it
/// was reached.
pub(in crate::doc_comments) fn path(p: &Path) -> String {
    json(&super::super::canonical(p).to_string_lossy())
}

fn params_json(list: &[Param]) -> String {
    let rows: Vec<String> = list
        .iter()
        .map(|p| {
            format!(
                "{{\"name\":{},\"type\":{},\"default\":{},\"doc\":{}{}}}",
                json(&p.name),
                json(&p.ty),
                json(&p.default),
                json(&p.doc),
                refs_json(&p.refs)
            )
        })
        .collect();
    format!("[{}]", rows.join(","))
}

/// One item as JSON; `listed` is false for a private type, whose links are not checked.
fn item_json(
    item: &Item,
    listed: bool,
    md_at: &dyn Fn(&str, &Path, usize, bool) -> String,
) -> String {
    let overloads: Vec<String> = item
        .overloads
        .iter()
        .map(|o| {
            format!(
                "{{\"file\":{},\"line\":{},\"signature\":{},\"params\":{},\"returns\":{},\"docs\":{},\"docs_line\":{},\"flags\":[{}],\"platforms\":[{}]}}",
                path(&o.file),
                o.line,
                json(&o.signature),
                params_json(&o.params),
                params_json(&o.returns),
                json(&md_at(&o.docs, &o.file, if o.docs_line > 0 { o.docs_line } else { o.line }, listed)),
                o.docs_line,
                o.flags.iter().map(|f| json(f)).collect::<Vec<_>>().join(","),
                o.platforms.iter().map(|f| json(f)).collect::<Vec<_>>().join(",")
            )
        })
        .collect();
    let members: Vec<String> = item
        .members
        .iter()
        .map(|m| {
            format!(
                "{{\"name\":{},\"type\":{},\"doc\":{}{}{}}}",
                json(&m.name),
                json(&m.ty),
                json(&md_at(&m.doc, &item.file, item.line, listed)),
                m.line
                    .map_or_else(String::new, |l| format!(",\"line\":{l}")),
                refs_json(&m.refs)
            )
        })
        .collect();
    let written = format!(
        "{}{}{}",
        if item.declaration.is_empty() {
            String::new()
        } else {
            format!(",\"declaration\":{}", json(&item.declaration))
        },
        refs_named("decl_refs", &item.decl_refs),
        if item.private {
            ",\"private\":true"
        } else {
            ""
        }
    );
    let variants = if item.variants.is_empty() {
        String::new()
    } else {
        format!(
            ",\"defined_on\":[{}],\"variants\":[{}]",
            item.defined_on
                .iter()
                .map(|f| json(f))
                .collect::<Vec<_>>()
                .join(","),
            item.variants
                .iter()
                .map(|v| item_json(v, listed, md_at))
                .collect::<Vec<_>>()
                .join(",")
        )
    };
    format!(
        "{{\"name\":{},\"kind\":{},\"file\":{},\"line\":{},\"docs\":{},\"value\":{},\"platforms\":[{}],\"experimental\":{}{},\"overloads\":[{}],\"members\":[{}]{}{},\"scope\":[{}]{}{}}}",
        json(&item.name),
        json(item.kind),
        path(&item.file),
        item.line,
        json(&md_at(&item.docs, &item.file, item.line, listed)),
        json(&item.value),
        item.platforms
            .iter()
            .map(|f| json(f))
            .collect::<Vec<_>>()
            .join(","),
        item.experimental,
        if item.binding {
            ",\"binding\":true"
        } else {
            ""
        },
        overloads.join(","),
        members.join(","),
        if item.also.is_empty() {
            String::new()
        } else {
            format!(
                ",\"also\":[{}]",
                item.also
                    .iter()
                    .map(|(f, l)| format!("[{},{l}]", path(f)))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        },
        refs_json(&item.refs),
        item.scope
            .iter()
            .map(|n| json(n))
            .collect::<Vec<_>>()
            .join(","),
        written,
        variants
    )
}

/// The model as JSON. `resolve` gives a URL for a link's names, as for [`docs::markdown`]; docs
/// come out as Markdown with the links it knows rewritten.
pub fn to_json(model: &Module, resolve: &dyn Fn(&[String]) -> Option<String>) -> String {
    let known: Vec<&str> = model
        .items
        .iter()
        .filter(|i| !i.private)
        .map(|i| i.name.as_str())
        .collect();
    let missed = std::cell::RefCell::new(Vec::<String>::new());
    // `file` and `line` say where the comment is, for the links that lead nowhere.
    let md_at = |doc: &str, file: &Path, line: usize, listed: bool| {
        if listed {
            for text in docs::unresolved::unresolved(doc, resolve, &known) {
                let at = docs::unresolved::located_line(file, line, &text);
                missed.borrow_mut().push(format!(
                    "{{\"file\":{},\"line\":{at},\"text\":{}}}",
                    path(file),
                    json(&text)
                ));
            }
        }
        docs::linked(doc, resolve)
    };
    let items: Vec<String> = model
        .items
        .iter()
        .map(|item| item_json(item, !item.private, &md_at))
        .collect();
    format!(
        "{{\"file\":{},\"implicit\":{},\"docs\":{},\"docs_line\":{},\"platforms\":[{}],\"experimental\":{},\"parameters\":[{}],\"items\":[\n{}\n],\"unresolved\":[{}]}}\n",
        path(&model.file),
        model.implicit,
        json(&md_at(
            &model.docs,
            &model.file,
            model.docs_line.max(1),
            true
        )),
        model.docs_line,
        model
            .platforms
            .iter()
            .map(|p| json(p))
            .collect::<Vec<_>>()
            .join(","),
        model.experimental,
        model
            .parameters
            .iter()
            .map(|p| format!(
                "{{\"name\":{},\"type\":{},\"default\":{},\"doc\":{}{}}}",
                json(&p.name),
                json(&p.ty),
                json(&p.default),
                json(&md_at(&p.doc, &model.file, model.docs_line.max(1), true)),
                refs_json(&p.refs)
            ))
            .collect::<Vec<_>>()
            .join(","),
        items.join(",\n"),
        missed.borrow().join(",")
    )
}
