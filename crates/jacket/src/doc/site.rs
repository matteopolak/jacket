//! The site as a whole: every module, where each page lives, and the links between them.
use super::categories::Category;
use super::model::{Item, Kind, Module, Overload, Param, TypeRef};
use std::cell::RefCell;
use std::collections::BTreeMap;

pub struct Site {
    pub modules: Vec<Module>,
    /// The headings modules are listed under, in display order; empty for one plain list.
    pub categories: Vec<Category>,
    /// Group names, in display order.
    pub groups: Vec<String>,
    /// Where the site is served: `/docs/jacket/`.
    pub base: String,
    /// The name in the header and page titles.
    pub name: String,
    /// The front page's description.
    pub description: String,
    /// The header's version; empty for none.
    pub version: String,
    /// What Source links point at when the commit does not match; empty for none.
    pub source_ref: String,
    /// The commit the library was built from, when it is a clean checkout of this repository.
    pub commit: Option<String>,
    /// Files of the library with uncommitted changes, whose line numbers the commit does not match.
    pub dirty: std::collections::BTreeSet<String>,
    /// The repository's web address; empty for none, and then pages have no Source links.
    pub repo: String,
    /// Where the library's folder is in the repository, with a trailing slash (`stdlib/`).
    pub source_prefix: String,
    /// The playground's address; empty for none.
    pub playground: String,
    /// The logo's file below `assets/`, if the site has one.
    pub logo: Option<String>,
    /// The heading of the extra pages (and their folder, as a slug); empty when there are none.
    pub pages_title: String,
    /// The first extra page, for the header link.
    pub pages_href: Option<String>,
    /// Sidebar headings of the extra pages, after the modules'.
    pub extra_nav: Vec<super::navigation::NavCategory>,
    /// The first breadcrumb, linking to the front page.
    pub breadcrumb: String,
    /// Whether the site carries a browser compiler.
    pub runtime: bool,
    by_id: BTreeMap<String, usize>,
    /// `Module/name` of an item whose page folder is not its plain slug, because another item
    /// of its module differs from it only in case (`Head` and `head`) and the two folders
    /// would be one on a case-insensitive file system.
    folders: BTreeMap<String, String>,
    /// A type name that is declared once across the library leads to its page.
    decls: BTreeMap<(String, usize), Vec<(usize, String)>>,
    /// Explicit validation jobs deduplicate only identical program text and timeout.
    runnable: RefCell<BTreeMap<(String, Option<u64>), super::example_check::Program>>,
    /// Intra-doc links that lead nowhere, as `Module.item (from page)`.
    broken: RefCell<Vec<String>>,
}

/// Page folders for items whose slugs differ only in case from another item's in the same
/// module. The first such item in source order whose name has no capitals keeps its slug (the
/// lowercase name is the one people type); the others add their kind, then a number if needed.
fn case_folders(modules: &[Module]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for m in modules {
        let mut groups: BTreeMap<String, Vec<&Item>> = BTreeMap::new();
        for item in m.items.iter().chain(&m.private) {
            groups
                .entry(Site::slug(&item.name).to_lowercase())
                .or_default()
                .push(item);
        }
        let private = |item: &Item| m.private.iter().any(|p| std::ptr::eq(p, item));
        for (_, mut group) in groups.into_iter().filter(|(_, g)| g.len() > 1) {
            // A page that is not exported gives way: it never changes where a documented one is.
            group.sort_by_key(|i| (private(i), i.name.chars().any(|c| c.is_ascii_uppercase())));
            let mut taken = jaic::fxhash::HashSet::default();
            taken.insert(Site::slug(&group[0].name).to_lowercase());
            for item in &group[1..] {
                let mut folder = format!("{}-{}", Site::slug(&item.name), item.kind.label());
                let mut n = 2;
                while !taken.insert(folder.to_lowercase()) {
                    folder = format!("{}-{}-{n}", Site::slug(&item.name), item.kind.label());
                    n += 1;
                }
                out.insert(format!("{}/{}", m.id, item.name), folder);
            }
        }
    }
    out
}

pub(super) const BUILTIN_TYPES: &[&str] = &[
    "int", "float", "float32", "float64", "bool", "string", "void", "Type", "Any", "s8", "s16",
    "s32", "s64", "u8", "u16", "u32", "u64", "s128", "u128", "code", "Code",
];

impl Site {
    pub fn new(
        mut modules: Vec<Module>,
        groups: Vec<String>,
        base: String,
        version: String,
        source_ref: String,
        playground: String,
        runtime: bool,
    ) -> Site {
        // Everything in an experimental module is experimental, wherever it is shown.
        for m in modules.iter_mut().filter(|m| m.experimental) {
            for item in &mut m.items {
                item.experimental = true;
            }
        }
        let by_id = modules
            .iter()
            .enumerate()
            .map(|(i, m)| (m.id.clone(), i))
            .collect();
        // Where each documented item is declared, so a name the compiler resolved to a
        // declaration finds the page (or pages, when several modules re-export it).
        let mut decls: BTreeMap<(String, usize), Vec<(usize, String)>> = BTreeMap::new();
        for (m, module) in modules.iter().enumerate() {
            for item in module.items.iter().chain(&module.private) {
                let here = std::iter::once((item.file.clone(), item.line))
                    .chain(item.also.iter().cloned());
                for at in here {
                    decls.entry(at).or_default().push((m, item.name.clone()));
                }
            }
        }
        let folders = case_folders(&modules);
        Site {
            modules,
            categories: Vec::new(),
            groups,
            base,
            name: String::new(),
            description: String::new(),
            version,
            source_ref,
            commit: None,
            dirty: std::collections::BTreeSet::new(),
            repo: String::new(),
            source_prefix: String::new(),
            playground,
            logo: None,
            pages_title: String::new(),
            pages_href: None,
            extra_nav: Vec::new(),
            breadcrumb: String::new(),
            runtime,
            by_id,
            folders,
            decls,
            runnable: RefCell::new(BTreeMap::new()),
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
        format!("api/{id}.html")
    }

    /// The file of an item's page, in its module's folder. Its address is the same without
    /// `.html`, which GitHub Pages and the dev servers look up.
    pub fn item_path(&self, module: &str, name: &str) -> String {
        format!("api/{module}/{}.html", self.folder(module, name))
    }

    /// The address of a page from its file: the path without `.html`, and the front page
    /// (`index.html`) at the base URL itself.
    pub fn page_href(&self, path: &str) -> String {
        let stem = path.strip_suffix(".html").unwrap_or(path);
        self.href(if stem == "index" {
            ""
        } else {
            stem
        })
    }

    /// The folder of an item's page below its module's folder.
    pub fn folder(&self, module: &str, name: &str) -> String {
        self.folders
            .get(&format!("{module}/{name}"))
            .cloned()
            .unwrap_or_else(|| Self::slug(name))
    }

    /// The folder name of an item: its name, with each character that is not safe in a file
    /// name (`<` in `operator<`) written as `-` and its code in hex. The producer records
    /// it in the route table; `website/docs/src/lib/urls.ts` consumes those canonical paths.
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
        self.href(&format!("api/{id}"))
    }

    /// An item's page.
    pub fn item_href(&self, module: &str, name: &str) -> String {
        self.href(&format!("api/{module}/{}", self.folder(module, name)))
    }

    /// A link to `line` of a library file: at the commit the site was built from when that file is
    /// unchanged there, else at the source ref. None without a repository or anything to point at.
    pub fn source_url(&self, file: &str, line: usize) -> Option<String> {
        if self.repo.is_empty() {
            return None;
        }
        let at = match &self.commit {
            Some(commit) if !self.dirty.contains(file) => commit,
            _ if !self.source_ref.is_empty() => &self.source_ref,
            _ => return None,
        };
        Some(format!(
            "{}/blob/{at}/{}{file}#L{line}",
            self.repo, self.source_prefix
        ))
    }

    /// The folder of the extra pages: `tools` for `Tools`.
    pub fn pages_dir(&self) -> String {
        super::slug::slug(&self.pages_title)
    }

    /// A page title: `what — name`.
    pub fn title(&self, what: &str) -> String {
        if self.name.is_empty() {
            what.to_owned()
        } else {
            format!("{what} — {}", self.name)
        }
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
            for item in module.items.iter().chain(&module.private) {
                let id = format!("{}/{}", module.id, item.name);
                if self.by_id.contains_key(&id) {
                    found.push(format!(
                        "`{}` is both an item and a module (api/{id}.html would hold both pages)",
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

    /// Deduplicate explicit checks by exact executable and budget, without a truncated hash.
    /// For example, one program with one- and two-second timeouts remains two checks.
    pub fn add_runnable(&self, program: super::example_check::Program) {
        self.runnable
            .borrow_mut()
            .entry((program.text.clone(), program.timeout))
            .or_insert(program);
    }

    /// Pending explicit checks; identical exports with the same budget contribute one check.
    pub fn runnable(&self) -> Vec<super::example_check::Program> {
        self.runnable.borrow().values().cloned().collect()
    }

    /// The documented declaration a written name (`Kind`, `Hash.Table`) was resolved to, if any:
    /// its module and name.
    pub(super) fn resolved(
        &self,
        head: &str,
        refs: &[TypeRef],
        here: usize,
    ) -> Option<(usize, &str)> {
        // References with the same written path are ordered alias hops. Skip private
        // aliases and thin re-exports until a declaration with a page is reached.
        let targets = refs
            .iter()
            .filter(|r| r.path == head)
            .find_map(|r| self.decls.get(&(r.file.clone(), r.line)))?;
        let (m, name) = targets
            .iter()
            .find(|(m, _)| *m == here)
            .or_else(|| targets.first())?;
        Some((*m, name.as_str()))
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
            private: false,
        }];
        use super::super::export::{build, contract::Target};
        let linked = build::text(&site, 0, "*Thing", &to_b, None).unwrap();
        assert_eq!(linked.source, "*Thing");
        assert!(linked.spans.iter().any(|span| span.start == 1
            && span.end == 6
            && matches!(&span.target,Some(Target::Page {id,anchor:None}) if id=="api/B/Thing")));
        let unresolved = build::text(&site, 0, "*Thing", &[], None).unwrap();
        assert!(unresolved.spans.iter().all(|span| span.target.is_none()));
        let member = [TypeRef {
            path: "Thing".into(),
            file: "A/thing.jai".into(),
            line: 3,
            private: false,
        }];
        let member_text = build::text(&site, 1, ":= Thing.ONE", &member, None).unwrap();
        assert!(member_text.spans.iter().any(|span| span.start == 3
            && span.end == 8
            && matches!(&span.target,Some(Target::Page {id,anchor:None}) if id=="api/A/Thing")));
        let own = build::text(&site, 1, "Thing", &to_b, Some("Thing")).unwrap();
        assert!(own.spans.iter().all(|span| span.target.is_none()));
    }

    #[test]
    fn names_that_differ_only_in_case_get_their_own_folders() {
        let item = |name: &str, kind| Item {
            name: name.into(),
            kind,
            ..Item::default()
        };
        let site = Site::new(
            vec![Module {
                id: "Http".into(),
                items: vec![
                    item("Head", Kind::Struct),
                    item("head", Kind::Procedure),
                    item("HEAD", Kind::Constant),
                    item("body", Kind::Procedure),
                ],
                ..Module::default()
            }],
            Vec::new(),
            "/".into(),
            "0.0.0".into(),
            String::new(),
            String::new(),
            false,
        );
        assert_eq!(site.item_href("Http", "head"), "/api/Http/head");
        assert_eq!(site.item_href("Http", "Head"), "/api/Http/Head-struct");
        assert_eq!(site.item_href("Http", "HEAD"), "/api/Http/HEAD-constant");
        assert_eq!(site.item_href("Http", "body"), "/api/Http/body");
    }

    #[test]
    fn a_page_that_is_not_exported_never_moves_an_exported_one() {
        let item = |name: &str, kind| Item {
            name: name.into(),
            kind,
            ..Item::default()
        };
        let site = Site::new(
            vec![Module {
                id: "Http".into(),
                items: vec![item("Head", Kind::Struct)],
                private: vec![item("head", Kind::Struct)],
                ..Module::default()
            }],
            Vec::new(),
            "/".into(),
            "0.0.0".into(),
            String::new(),
            String::new(),
            false,
        );
        assert_eq!(site.item_href("Http", "Head"), "/api/Http/Head");
        assert_eq!(site.item_href("Http", "head"), "/api/Http/head-struct");
    }

    #[test]
    fn configured_breadcrumb_does_not_become_part_of_the_site_brand() {
        let config: jacket_config::schema::Config =
            toml::from_str(include_str!("../../../../jacket.toml")).unwrap();
        let mut site = Site::new(
            vec![],
            vec![],
            "/".into(),
            String::new(),
            String::new(),
            String::new(),
            false,
        );
        site.name = config.doc.name.unwrap();
        site.breadcrumb = config.doc.breadcrumb.unwrap();
        assert_eq!(site.title("Basic"), "Basic — Jacket");
        assert_eq!(site.title("Testing"), "Testing — Jacket");
        site.name = "Another library".into();
        site.breadcrumb = "A separate root crumb".into();
        assert_eq!(site.title("Usage"), "Usage — Another library");
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
