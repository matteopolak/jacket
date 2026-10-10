//! `broken_doc_link`: a link in a declaration's doc comment that names nothing.
//!
//! Doc comments link to declarations with `[name]`, `` [`name`] ``, `[name()]`,
//! `[Module.name]`, `[Type.field]` and `[text](name)` (see `docs/compiler/doc-comments.md`).
//! The editor makes those clickable; a link that names nothing, because the declaration was
//! renamed or the name was misspelt, silently stops being one.
//!
//! Only comments directly above a declaration (or trailing one) are looked at, and only links
//! that look meant as links: `jaic::docs` decides (`Site::meant`, `docs::meant_as_link`), the
//! same text rules the language server uses. Code fences, inline code, `a[i]`, `[0]`, `[x][y]`,
//! URL links, `[ref]: url` definitions, footnotes and checkboxes are not links; one-letter and
//! all-capitals words (`[x]`, `[TODO]`) are prose, and so is a plain lowercase word (`[yet]`,
//! `[sic]`, `[keys]`) unless it is a close misspelling of a name in scope (`[prnt]`). Names resolve from the scope of the
//! documented declaration, with `Compiler::ide_resolve_path`, as hover does.
//!
//! When exactly one name in scope (or one member of what the path leads to) is a close
//! misspelling of the link's last segment, the fix replaces it. That is a guess, so it is
//! never machine applicable.
use crate::syntax::Cx;
use crate::{Edit, Finding, Fix};
use jaic::docs;
use jaic::intern::Sym;
use jaic::sema::Compiler;
use jaic::sema::ide::IdeReceiver;
use jaic::source::FileId;

/// A link that does not resolve.
pub(crate) struct Broken {
    /// The whole link.
    start: usize,
    end: usize,
    /// How the message names it.
    shown: String,
    /// The last segment of the name it spells.
    last: (usize, usize),
    /// A close match for it.
    suggestion: Option<String>,
}

/// Find the broken links in the doc comments of `file` (resolving needs the compiler mutably).
pub(crate) fn prepare(compiler: &mut Compiler, file: FileId, text: &str) -> Vec<Broken> {
    let mut out = Vec::new();
    // Most files have no comment with a link in it.
    if !text.contains("//") && !text.contains("/*") || !text.contains('[') {
        return out;
    }
    for site in docs::sites(text) {
        if !site.on_declaration || !site.meant {
            continue;
        }
        let Some(scope) = compiler.ide_scope_at(file, site.scope_at as u32) else {
            continue;
        };
        let names: Vec<Sym> = site.names.iter().map(|n| Sym::intern(n)).collect();
        if !compiler.ide_resolve_path(scope, &names).is_empty() {
            continue;
        }
        let Some((last, first)) = site.names.split_last() else {
            continue;
        };
        // `[table.header]`: a dotted word in plain brackets whose first part is no name at
        // all is prose; `[Type.nothing]` (a name that exists) is a link gone wrong.
        let whole = &text[site.whole.0..site.whole.1];
        let plain_path = !whole.contains('`') && !whole.ends_with(')');
        if plain_path
            && !first.is_empty()
            && first[0]
                .chars()
                .all(|c| c.is_lowercase() || c.is_ascii_digit())
            && compiler.ide_resolve_path(scope, &names[..1]).is_empty()
            && compiler.ide_receiver(scope, &names[..1]).is_none()
        {
            continue;
        }
        let wanted: Vec<String> = if first.is_empty() {
            compiler
                .ide_visible(scope, file, site.scope_at as u32)
                .into_iter()
                .map(|n| n.name)
                .collect()
        } else {
            compiler
                .ide_receiver(scope, &names[..names.len() - 1])
                .map(|r: IdeReceiver| compiler.ide_members(r))
                .unwrap_or_default()
                .into_iter()
                .map(|n| n.name)
                .collect()
        };
        let suggestion = jaic::suggest::closest_unique(last, wanted.iter().map(String::as_str))
            .map(str::to_owned);
        // `[sic]`, `[yet]`: a plain lowercase word with no near neighbour is prose.
        if site.plain && (suggestion.is_none() || last.chars().count() < 4) {
            continue;
        }
        let shown = if whole.contains('`') {
            format!("[{}]", &text[site.path.0..site.path.1])
        } else {
            whole.to_owned()
        };
        out.push(Broken {
            start: site.whole.0,
            end: site.whole.1,
            shown,
            last: (site.path.1 - last.len(), site.path.1),
            suggestion,
        });
    }
    out
}

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for b in &cx.doc_links {
        let (help, fix) = match &b.suggestion {
            Some(name) => (
                format!("did you mean `{name}`?"),
                Some(Fix {
                    title: format!("link to `{name}`"),
                    edits: vec![Edit {
                        start: b.last.0,
                        end: b.last.1,
                        text: name.clone(),
                    }],
                    machine_applicable: false,
                }),
            ),
            None => (
                "link a declaration that is in scope, or leave out the brackets if this is not \
                 a link"
                    .to_owned(),
                None,
            ),
        };
        out.push(Finding {
            start: b.start,
            end: b.end,
            message: format!("`{}` doesn't resolve to a declaration", b.shown),
            label: Some("not found from the documented declaration's scope".into()),
            help: Some(help),
            fix,
        });
    }
}
