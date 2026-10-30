//! The site as a whole: every module, where each page lives, and the links between them.
use super::markdown::esc;
use super::model::{Item, Kind, Module, Overload, Param, TypeRef};
use std::cell::RefCell;
use std::collections::BTreeMap;

pub struct Site {
    pub modules: Vec<Module>,
    /// Category names, in display order.
    pub categories: Vec<String>,
    /// Where the site is served: `/docs/jacket/`.
    pub base: String,
    pub version: String,
    pub source_ref: String,
    /// The commit the library was built from, when it is a clean checkout of this repository.
    pub commit: Option<String>,
    /// Files below `stdlib/` with uncommitted changes, whose line numbers the commit does not match.
    pub dirty: std::collections::BTreeSet<String>,
    pub repo: String,
    pub playground: String,
    /// Whether the site carries a browser compiler.
    pub runtime: bool,
    by_id: BTreeMap<String, usize>,
    /// A type name that is declared once across the library leads to its page.
    decls: BTreeMap<(String, usize), Vec<(usize, String)>>,
    /// Example programs, by the id in their file name.
    examples: RefCell<BTreeMap<String, String>>,
    /// Intra-doc links that lead nowhere, as `Module.item (from page)`.
    broken: RefCell<Vec<String>>,
}

const BUILTIN_TYPES: &[&str] = &[
    "int", "float", "float32", "float64", "bool", "string", "void", "Type", "Any", "s8", "s16",
    "s32", "s64", "u8", "u16", "u32", "u64", "s128", "u128", "code", "Code",
];

impl Site {
    pub fn new(
        modules: Vec<Module>,
        categories: Vec<String>,
        base: String,
        version: String,
        source_ref: String,
        playground: String,
        runtime: bool,
    ) -> Site {
        let by_id = modules
            .iter()
            .enumerate()
            .map(|(i, m)| (m.id.clone(), i))
            .collect();
        // Where each documented type is declared, so a name the compiler resolved to a
        // declaration finds the page (or pages, when several modules re-export it).
        let mut decls: BTreeMap<(String, usize), Vec<(usize, String)>> = BTreeMap::new();
        for (m, module) in modules.iter().enumerate() {
            for item in module.items.iter().filter(|i| i.kind.is_type()) {
                let here = std::iter::once((item.file.clone(), item.line))
                    .chain(item.also.iter().cloned());
                for at in here {
                    decls.entry(at).or_default().push((m, item.name.clone()));
                }
            }
        }
        Site {
            modules,
            categories,
            base,
            version,
            source_ref,
            commit: None,
            dirty: std::collections::BTreeSet::new(),
            repo: "https://github.com/matteopolak/jacket".into(),
            playground,
            runtime,
            by_id,
            decls,
            examples: RefCell::new(BTreeMap::new()),
            broken: RefCell::new(Vec::new()),
        }
    }

    pub fn module_index(&self, id: &str) -> Option<usize> {
        self.by_id.get(id).copied()
    }

    // Where pages are.

    pub fn href(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    pub fn module_path(id: &str) -> String {
        format!("api/{id}/index.html")
    }

    /// The file of an item's page: every page is `index.html` in its own folder, so its address
    /// is the folder with no extension, and a plain static server serves it.
    pub fn item_path(module: &str, name: &str) -> String {
        format!("api/{module}/{}/index.html", Self::slug(name))
    }

    /// The folder name of an item: its name, with each character that is not safe in a file
    /// name (`<` in `operator<`) written as `-` and its code in hex. `site.js` does the same.
    pub fn slug(name: &str) -> String {
        let mut out = String::new();
        for c in name.chars() {
            if c.is_ascii_alphanumeric() || c == '_' {
                out.push(c);
            } else {
                out.push_str(&format!("-{:02x}", c as u32));
            }
        }
        out
    }

    /// A link to a module's page.
    pub fn module_href(&self, id: &str) -> String {
        self.href(&format!("api/{id}/"))
    }

    pub fn item_href(&self, module: &str, name: &str) -> String {
        self.href(&format!("api/{module}/{}/", Self::slug(name)))
    }

    /// A link to `line` of a library file: at the commit the site was built from when that file is
    /// unchanged there, else at the version's tag.
    pub fn source_url(&self, file: &str, line: usize) -> String {
        let at = match &self.commit {
            Some(commit) if !self.dirty.contains(file) => commit,
            _ => &self.source_ref,
        };
        format!("{}/blob/{at}/stdlib/{file}#L{line}", self.repo)
    }

    // Links in doc comments.

    /// Where a link of a doc comment goes. `jaidoc:Module/item[#member]` is an intra-doc link
    /// from `jaic -doc_model`; it must lead to a page, or it is dropped (and recorded).
    pub fn doc_link(&self, dest: &str, from: &str) -> Option<String> {
        let Some(rest) = dest.strip_prefix("jaidoc:") else {
            return Some(dest.to_owned());
        };
        let (path, member) = rest
            .split_once('#')
            .map_or((rest, None), |(p, m)| (p, Some(m)));
        let found = path.rsplit_once('/').and_then(|(module, name)| {
            let m = self.module_index(module)?;
            let item = self.modules[m].item(name)?;
            Some((module, item))
        });
        let Some((module, item)) = found else {
            self.broken.borrow_mut().push(format!("{path} (in {from})"));
            return None;
        };
        let href = self.item_href(module, &item.name);
        Some(match member {
            Some(member) if item.members.iter().any(|m| m.name == member) => {
                format!("{href}#m-{member}")
            }
            Some(member) => {
                self.broken
                    .borrow_mut()
                    .push(format!("{path}.{member} (in {from})"));
                href
            }
            None => href,
        })
    }

    /// Pages that would be written to the same file: an item named like a module below it.
    pub fn path_clashes(&self) -> Vec<String> {
        let mut found = Vec::new();
        for module in &self.modules {
            for item in &module.items {
                let id = format!("{}/{}", module.id, item.name);
                if self.by_id.contains_key(&id) {
                    found.push(format!(
                        "`{}` is both an item and a module (api/{id}/ would hold both pages)",
                        item.name
                    ));
                }
            }
        }
        found
    }

    pub fn broken_links(&self) -> Vec<String> {
        let mut found = self.broken.borrow().clone();
        found.sort();
        found.dedup();
        found
    }

    /// The page of a procedure of `module` that an example calls by name.
    pub fn call_link(&self, module: &Module, name: &str) -> Option<String> {
        let item = module.item(name).filter(|i| i.kind == Kind::Procedure)?;
        Some(self.item_href(&module.id, &item.name))
    }

    // Examples.

    pub fn add_example(&self, program: &str) -> String {
        let id = super::example::program_id(program);
        self.examples
            .borrow_mut()
            .insert(id.clone(), program.to_owned());
        id
    }

    pub fn examples(&self) -> BTreeMap<String, String> {
        self.examples.borrow().clone()
    }

    // Signatures.

    /// `ty` as HTML: a name the compiler resolved (`refs`) to a type the library documents links
    /// to its page, builtin types are coloured. Nothing is matched by name alone, so two
    /// modules with a type of the same name each link to their own.
    pub fn type_html(&self, ty: &str, refs: &[TypeRef], here: usize) -> String {
        let mut out = String::new();
        let chars: Vec<(usize, char)> = ty.char_indices().collect();
        let mut i = 0;
        while i < chars.len() {
            let (start, c) = chars[i];
            if c.is_ascii_digit() {
                // A number: `[4] u8`, `1.5`.
                let mut j = i;
                while j < chars.len() && (chars[j].1.is_alphanumeric() || chars[j].1 == '_') {
                    j += 1;
                }
                let end = chars.get(j).map_or(ty.len(), |&(b, _)| b);
                out.push_str(&esc(&ty[start..end]));
                i = j;
            } else if c.is_alphabetic() || c == '_' {
                // A name, with the `.name`s after it: `Hash.Table`.
                let mut j = i;
                loop {
                    while j < chars.len() && (chars[j].1.is_alphanumeric() || chars[j].1 == '_') {
                        j += 1;
                    }
                    let more = j + 1 < chars.len()
                        && chars[j].1 == '.'
                        && (chars[j + 1].1.is_alphabetic() || chars[j + 1].1 == '_');
                    if !more {
                        break;
                    }
                    j += 1;
                }
                let end = chars.get(j).map_or(ty.len(), |&(b, _)| b);
                out.push_str(&self.path_html(&ty[start..end], refs, here));
                i = j;
            } else {
                out.push_str(&esc(&ty[start..start + c.len_utf8()]));
                i += 1;
            }
        }
        out
    }

    /// A written name: its longest start that the compiler resolved to a documented type is
    /// the link (`Kind` of `Kind.MEMBER`), and what follows is plain.
    fn path_html(&self, path: &str, refs: &[TypeRef], here: usize) -> String {
        let cuts = path
            .match_indices('.')
            .map(|(i, _)| i)
            .chain(std::iter::once(path.len()));
        for end in cuts.collect::<Vec<_>>().into_iter().rev() {
            let head = &path[..end];
            if let Some(r) = refs.iter().find(|r| r.path == head)
                && let Some(targets) = self.decls.get(&(r.file.clone(), r.line))
                && let Some((m, name)) = targets
                    .iter()
                    .find(|(m, _)| *m == here)
                    .or_else(|| targets.first())
            {
                return format!(
                    "<a class=\"ty\" href=\"{}\">{}</a>{}",
                    esc(&self.item_href(&self.modules[*m].id, name)),
                    esc(head),
                    esc(&path[end..])
                );
            }
        }
        if BUILTIN_TYPES.contains(&path) {
            return format!("<span class=\"ty\">{}</span>", esc(path));
        }
        esc(path)
    }
}

/// `name: type`, `name := default` or `name: type = default`.
pub fn param_text(p: &Param) -> String {
    match (p.ty.is_empty(), p.default.is_empty()) {
        (false, true) => format!("{}: {}", p.name, p.ty),
        (true, false) => format!("{} := {}", p.name, p.default),
        (false, false) => format!("{}: {} = {}", p.name, p.ty, p.default),
        (true, true) => p.name.clone(),
    }
}

fn param_html(site: &Site, p: &Param, here: usize) -> String {
    let name = if p.name.is_empty() {
        String::new()
    } else {
        esc(&p.name)
    };
    match (p.ty.is_empty(), p.default.is_empty()) {
        (false, true) if p.name.is_empty() => site.type_html(&p.ty, &p.refs, here),
        (false, true) => format!("{name}: {}", site.type_html(&p.ty, &p.refs, here)),
        (true, false) => format!("{name} := {}", esc(&p.default)),
        (false, false) => format!(
            "{name}: {} = {}",
            site.type_html(&p.ty, &p.refs, here),
            esc(&p.default)
        ),
        (true, true) => name,
    }
}

/// The results as written after `->`: one bare, or several in parentheses.
fn returns_plain(returns: &[Param]) -> String {
    match returns {
        [] => String::new(),
        [one] if one.name.is_empty() => one.ty.clone(),
        [one] => param_text(one),
        many => format!(
            "({})",
            many.iter().map(param_text).collect::<Vec<_>>().join(", ")
        ),
    }
}

/// `(s: string, byte: u8) -> (found: bool, left: string, right: string)`, for a list or a
/// search result.
pub fn signature_plain(o: &Overload) -> String {
    let params = o
        .params
        .iter()
        .map(param_text)
        .collect::<Vec<_>>()
        .join(", ");
    let returns = returns_plain(&o.returns);
    if returns.is_empty() {
        format!("({params})")
    } else {
        format!("({params}) -> {returns}")
    }
}

/// What names each overload of `item` in the page outline: its parameters, `(s: string, n: int)`;
/// with the results when two overloads take the same parameters, and `Overload 2` when even
/// those match.
pub fn overload_labels(item: &super::model::Item) -> Vec<String> {
    let clip = |text: String| -> String {
        if text.chars().count() <= 48 {
            return text;
        }
        let cut: String = text.chars().take(45).collect();
        format!("{cut}...")
    };
    let params: Vec<String> = item
        .overloads
        .iter()
        .map(|o| {
            let list = o
                .params
                .iter()
                .map(param_text)
                .collect::<Vec<_>>()
                .join(", ");
            format!("({list})")
        })
        .collect();
    let full: Vec<String> = item.overloads.iter().map(signature_plain).collect();
    (0..params.len())
        .map(|i| {
            let count = |list: &[String]| list.iter().filter(|p| **p == list[i]).count();
            if count(&params) == 1 {
                clip(params[i].clone())
            } else if count(&full) == 1 {
                clip(full[i].clone())
            } else {
                format!("Overload {}", i + 1)
            }
        })
        .collect()
}

/// `(s: string, n := 1) -> bool` as HTML, for a list: the parameters on one line, the types
/// linked, wrapping at spaces when the line is too long for its card.
pub fn signature_line_html(site: &Site, o: &Overload, here: usize) -> String {
    let params = o
        .params
        .iter()
        .map(|p| param_html(site, p, here))
        .collect::<Vec<_>>()
        .join(", ");
    let returns = match o.returns.as_slice() {
        [] => String::new(),
        [one] if one.name.is_empty() => format!(" -> {}", site.type_html(&one.ty, &one.refs, here)),
        [one] => format!(" -> {}", param_html(site, one, here)),
        many => format!(
            " -> ({})",
            many.iter()
                .map(|p| param_html(site, p, here))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    };
    format!("({params}){returns}")
}

/// The declaration of an overload as a code block's content: the name, then the parameters,
/// broken over lines when they are long, and the results with links to their types.
pub fn signature_html(site: &Site, name: &str, o: &Overload, here: usize) -> String {
    let plain = format!("{name} :: {}", signature_plain(o));
    let head = format!("<span class=\"fn\">{}</span> :: (", esc(name));
    let tail = |returns: &[Param]| -> String {
        match returns {
            [] => String::new(),
            [one] if one.name.is_empty() => {
                format!(" -> {}", site.type_html(&one.ty, &one.refs, here))
            }
            [one] => format!(" -> {}", param_html(site, one, here)),
            many => format!(
                " -> ({})",
                many.iter()
                    .map(|p| param_html(site, p, here))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    };
    let flags: String = o.flags.iter().map(|f| format!(" {}", esc(f))).collect();
    let params = o
        .params
        .iter()
        .map(|p| param_html(site, p, here))
        .collect::<Vec<_>>()
        .join(", ");
    if plain.chars().count() <= 80 {
        return format!("{head}{params}){}{flags}", tail(&o.returns));
    }
    // Too wide for a line: one parameter to a line, the results after the closing parenthesis.
    let mut out = head;
    if !o.params.is_empty() {
        out.push('\n');
        for (i, p) in o.params.iter().enumerate() {
            out.push_str("    ");
            out.push_str(&param_html(site, p, here));
            out.push_str(if i + 1 < o.params.len() {
                ",\n"
            } else {
                "\n"
            });
        }
    }
    out.push(')');
    out.push_str(&tail(&o.returns));
    out.push_str(&flags);
    out
}

/// The one-line form of what an item is, for a list: a procedure's first signature, or what a
/// type or constant is set to.
pub fn item_line(item: &Item) -> String {
    match item.kind {
        Kind::Procedure => item
            .overloads
            .first()
            .map(signature_plain)
            .unwrap_or_default(),
        Kind::Constant | Kind::Type => {
            if item.value.is_empty() {
                String::new()
            } else {
                format!(":: {}", item.value)
            }
        }
        kind => format!(":: {}", kind.label().replace(' ', "_")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn overload(params: &[(&str, &str, &str)], returns: &[(&str, &str)]) -> Overload {
        Overload {
            params: params
                .iter()
                .map(|(n, t, d)| Param {
                    name: (*n).into(),
                    ty: (*t).into(),
                    default: (*d).into(),
                    doc: String::new(),
                    ..Param::default()
                })
                .collect(),
            returns: returns
                .iter()
                .map(|(n, t)| Param {
                    name: (*n).into(),
                    ty: (*t).into(),
                    ..Param::default()
                })
                .collect(),
            ..Overload::default()
        }
    }

    #[test]
    fn a_type_links_to_the_declaration_the_compiler_found() {
        fn module(id: &str, file: &str) -> Module {
            Module {
                id: id.into(),
                items: vec![Item {
                    name: "Thing".into(),
                    kind: Kind::Struct,
                    file: file.into(),
                    line: 3,
                    ..Item::default()
                }],
                ..Module::default()
            }
        }
        let site = Site::new(
            vec![module("A", "A/thing.jai"), module("B", "B/thing.jai")],
            Vec::new(),
            "/".into(),
            "0.0.0".into(),
            String::new(),
            String::new(),
            false,
        );
        let to_b = [TypeRef {
            path: "Thing".into(),
            file: "B/thing.jai".into(),
            line: 3,
        }];
        // Written the same in module A, but resolved to the other module's type.
        assert_eq!(
            site.type_html("*Thing", &to_b, 0),
            "*<a class=\"ty\" href=\"/api/B/Thing/\">Thing</a>"
        );
        // Not resolved to anything documented: no link, whatever else is named `Thing`.
        assert_eq!(site.type_html("*Thing", &[], 0), "*Thing");
        let member = [TypeRef {
            path: "Thing".into(),
            file: "A/thing.jai".into(),
            line: 3,
        }];
        assert_eq!(
            site.type_html(":= Thing.ONE", &member, 1),
            ":= <a class=\"ty\" href=\"/api/A/Thing/\">Thing</a>.ONE"
        );
    }

    #[test]
    fn signatures_read_like_the_source() {
        let o = overload(
            &[("s", "string", ""), ("n", "", "1")],
            &[("found", "bool"), ("rest", "string")],
        );
        assert_eq!(
            signature_plain(&o),
            "(s: string, n := 1) -> (found: bool, rest: string)"
        );
        let o = overload(&[("s", "string", "")], &[("", "bool")]);
        assert_eq!(signature_plain(&o), "(s: string) -> bool");
        assert_eq!(signature_plain(&overload(&[], &[])), "()");
    }
}
