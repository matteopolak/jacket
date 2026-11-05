//! Deterministic export orchestration. Complete bundles publish through an owned staging directory.
use super::contract::{
    ApiModule, Asset, AssetProvenance, Capability, Category, Declaration, DeclarationFormatProfile,
    DeclarationKind, DocumentKind, DocumentNode, DocumentPage, Formatter, InputDigest, Inventory,
    Link, Manifest, Member, NavigationEntry, NavigationGroup, PageKind, PageRoute, Parameter,
    Payload, Producer, Schema, SearchData, SearchEntry, SearchKind, SearchParameter, SearchPreview,
    SearchSignature, SiteData, SourceState, Target,
};
use super::{build, document, publication};
use crate::doc::{guides::Guides, site::Site};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn shard(kind: &str, id: &str) -> String {
    format!("{kind}/{}.json", &hash(id.as_bytes())[..24])
}

fn json(data: &impl serde::Serialize) -> Result<Vec<u8>, String> {
    serde_json::to_vec(data).map_err(|e| format!("documentation export JSON: {e}"))
}

struct Staging {
    path: PathBuf,
}

impl Drop for Staging {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

impl Staging {
    fn new(out: &Path) -> Result<Self, String> {
        let parent = out
            .parent()
            .ok_or("export output needs a parent directory")?;
        std::fs::create_dir_all(parent).map_err(|e| format!("create export parent: {e}"))?;
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let path = parent.join(format!(".jacket-doc-export-{}-{stamp}", std::process::id()));
        std::fs::create_dir(&path).map_err(|e| format!("create export staging directory: {e}"))?;
        Ok(Self {
            path,
        })
    }

    fn put(&self, path: &str, bytes: &[u8]) -> Result<(), String> {
        super::contract::validate_path(path)?;
        let file = self.path.join(path);
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&file, bytes).map_err(|e| format!("write export {path}: {e}"))
    }

    fn publish(&self, out: &Path) -> Result<(), String> {
        if out.exists() {
            let info = std::fs::symlink_metadata(out).map_err(|e| e.to_string())?;
            if info.file_type().is_symlink()
                || !info.is_dir()
                || (!out.join(".jacket-doc-export").is_file()
                    && std::fs::read_dir(out)
                        .map_err(|e| e.to_string())?
                        .next()
                        .is_some())
            {
                return Err(format!(
                    "{} exists and Jacket did not create it; choose another export output",
                    out.display()
                ));
            }
            let backup = self.path.with_extension("previous");
            std::fs::rename(out, &backup).map_err(|e| format!("stage previous export: {e}"))?;
            if let Err(error) = std::fs::rename(&self.path, out) {
                std::fs::rename(&backup, out).map_err(|e| {
                    format!("publish failed ({error}); restore previous export: {e}")
                })?;
                return Err(format!("publish export: {error}"));
            }
            std::fs::remove_dir_all(backup).map_err(|e| format!("remove previous export: {e}"))?;
        } else {
            std::fs::rename(&self.path, out).map_err(|e| format!("publish export: {e}"))?;
        }
        Ok(())
    }
}

fn heading_anchors(nodes: &[DocumentNode], out: &mut Vec<String>) {
    for node in nodes {
        match node {
            DocumentNode::Heading {
                id: Some(id),
                children,
                ..
            } => {
                out.push(id.clone());
                heading_anchors(children, out);
            }
            DocumentNode::Paragraph {
                children,
            }
            | DocumentNode::Heading {
                children, ..
            }
            | DocumentNode::Emphasis {
                children,
            }
            | DocumentNode::Strong {
                children,
            }
            | DocumentNode::Strikethrough {
                children,
            }
            | DocumentNode::Link {
                children, ..
            }
            | DocumentNode::List {
                children, ..
            }
            | DocumentNode::ListItem {
                children,
            }
            | DocumentNode::Quote {
                children,
            }
            | DocumentNode::TableCell {
                children,
            } => heading_anchors(children, out),
            DocumentNode::Table {
                head,
                rows,
                ..
            } => {
                heading_anchors(head, out);
                heading_anchors(rows, out);
            }
            DocumentNode::TableRow {
                cells,
            } => heading_anchors(cells, out),
            _ => {}
        }
    }
}

fn paragraph(nodes: &[DocumentNode]) -> String {
    nodes
        .iter()
        .find_map(|n| match n {
            DocumentNode::Paragraph {
                children,
            } => Some(document::plain(children)),
            _ => None,
        })
        .unwrap_or_default()
}

fn description(nodes: &[DocumentNode]) -> String {
    paragraph(nodes)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn member_table(d: &Declaration, member: &Member) -> bool {
    !d.definitions.iter().any(|definition| {
        definition
            .anchors
            .iter()
            .any(|anchor| anchor.id == member.id)
    })
}

fn routes_for_module(m: &ApiModule, path: &str) -> Vec<PageRoute> {
    let output = Site::module_path(&m.id);
    let id = build::route_id(&output);
    let mut anchors = vec!["main-content".into()];
    if !m.parameters.is_empty() {
        anchors.push("parameters".into());
    }
    let is_type = |kind| {
        matches!(
            kind,
            DeclarationKind::Struct
                | DeclarationKind::Union
                | DeclarationKind::Enum
                | DeclarationKind::EnumFlags
                | DeclarationKind::Type
        )
    };
    for section in ["procedures", "types", "constants"] {
        let items = m
            .declarations
            .iter()
            .filter(|d| {
                !d.private
                    && match section {
                        "procedures" => d.kind == DeclarationKind::Procedure,
                        "types" => is_type(d.kind),
                        _ => d.kind == DeclarationKind::Constant,
                    }
            })
            .collect::<Vec<_>>();
        if items.is_empty() {
            continue;
        }
        anchors.push(section.into());
        if section == "procedures" {
            let mut names = Vec::new();
            for item in items.iter().filter(|d| !d.binding) {
                if !names.contains(&item.group.as_str()) {
                    names.push(item.group.as_str());
                }
            }
            let foreign = items.iter().any(|d| d.binding);
            if names.len() > 1 || (foreign && names.iter().any(|s| !s.is_empty())) {
                for name in names {
                    let name = if name.is_empty() {
                        "Other"
                    } else {
                        name
                    };
                    let slug = name
                        .chars()
                        .map(|c| {
                            if c.is_alphanumeric() {
                                c.to_ascii_lowercase()
                            } else {
                                '-'
                            }
                        })
                        .collect::<String>()
                        .split('-')
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join("-");
                    anchors.push(format!("g-{slug}"));
                }
                if foreign {
                    anchors.push("g-foreign".into());
                }
            }
        }
        anchors.extend(items.iter().map(|d| d.name.clone()));
    }
    let mut routes = vec![PageRoute {
        id: id.clone(),
        label: m.title.clone(),
        route: id,
        output,
        kind: PageKind::Module,
        anchors,
        aliases: vec![],
        private: false,
        noindex: m.metadata.noindex,
        shard: path.into(),
    }];
    for d in &m.declarations {
        let output = format!("{}.html", d.route);
        let mut anchors = vec!["main-content".into()];
        anchors.extend(d.members.iter().map(|m| m.id.clone()));
        for definition in &d.definitions {
            anchors.push(definition.id.clone());
            for anchor in &definition.anchors {
                if !anchors.contains(&anchor.id) {
                    anchors.push(anchor.id.clone());
                }
            }
        }
        anchors.extend(d.overloads.iter().map(|o| o.id.clone()));
        if !d.definitions.is_empty() {
            anchors.push("declaration".into());
        }
        if d.members.iter().any(|member| member_table(d, member)) {
            anchors.push("members".into());
        }
        routes.push(PageRoute {
            id: d.id.clone(),
            label: d.name.clone(),
            route: d.route.clone(),
            output,
            kind: PageKind::Declaration,
            anchors,
            aliases: vec![],
            private: d.private,
            noindex: d.metadata.noindex,
            shard: path.into(),
        });
    }
    routes
}

fn synthetic_document(
    site: &Site,
    publication: &publication::Publication,
    kind: DocumentKind,
    title: &str,
    description: &str,
) -> DocumentPage {
    let id = if kind == DocumentKind::Home {
        String::new()
    } else {
        "404".into()
    };
    let path = if id.is_empty() {
        "index.html".into()
    } else {
        format!("{id}.html")
    };
    let nodes = vec![DocumentNode::Paragraph {
        children: vec![DocumentNode::Text {
            text: description.into(),
        }],
    }];
    let metadata = publication.document(site, &path, kind, title, &nodes);
    DocumentPage {
        id,
        title: title.into(),
        kind,
        source: None,
        nodes,
        outline: vec![],
        metadata,
    }
}

fn navigation(site: &Site) -> Result<Vec<NavigationGroup>, String> {
    let mut nav = Vec::new();
    for (i, category) in site.extra_nav.iter().enumerate() {
        let children = category
            .groups
            .iter()
            .map(|group| {
                Ok(NavigationGroup {
                    id: group.key.clone(),
                    label: group.label.clone(),
                    children: vec![],
                    entries: group
                        .links
                        .iter()
                        .map(|link| {
                            Ok(NavigationEntry {
                                id: build::route_id(
                                    link.href.strip_prefix(&site.base).unwrap_or(&link.href),
                                ),
                                label: link.label.clone(),
                                title: link.title.clone(),
                                target: build::target(site, &link.href, "")?,
                                experimental: link.experimental,
                                depth: link.depth as u32,
                            })
                        })
                        .collect::<Result<_, String>>()?,
                })
            })
            .collect::<Result<_, String>>()?;
        nav.push(NavigationGroup {
            id: format!("documents-{i}"),
            label: category.label.clone(),
            children,
            entries: vec![],
        });
    }
    for category in std::iter::once(None).chain((0..site.categories.len()).map(Some)) {
        let modules = site
            .modules
            .iter()
            .filter(|m| m.category == category)
            .collect::<Vec<_>>();
        if modules.is_empty() {
            continue;
        }
        let mut children = Vec::new();
        for group in std::iter::once("").chain(site.groups.iter().map(String::as_str)) {
            let entries = modules
                .iter()
                .filter(|m| m.group == group)
                .map(|m| NavigationEntry {
                    id: build::route_id(&Site::module_path(&m.id)),
                    label: m.title().into(),
                    title: Some(m.id.clone()),
                    target: Target::Page {
                        id: build::route_id(&Site::module_path(&m.id)),
                        anchor: None,
                    },
                    experimental: m.experimental,
                    depth: modules
                        .iter()
                        .filter(|parent| m.id.starts_with(&format!("{}/", parent.id)))
                        .count() as u32,
                })
                .collect::<Vec<_>>();
            if !entries.is_empty() {
                children.push(NavigationGroup {
                    id: format!(
                        "api/{}/{group}",
                        category.map_or_else(String::new, |i| i.to_string())
                    ),
                    label: group.into(),
                    children: vec![],
                    entries,
                });
            }
        }
        nav.push(NavigationGroup {
            id: category.map_or_else(|| "api".into(), |i| format!("category-{i}")),
            label: category.map_or_else(String::new, |i| site.categories[i].name.clone()),
            children,
            entries: vec![],
        });
    }
    Ok(nav)
}

/// An asset keeps its owning input category until the public manifest is written.
#[derive(Clone)]
pub(in crate::doc) struct InputAsset {
    pub path: String,
    pub bytes: Vec<u8>,
    pub provenance: AssetProvenance,
}

pub(in crate::doc) struct Request<'a> {
    pub site: &'a Site,
    pub guides: &'a Guides,
    pub publication: &'a publication::Publication,
    pub out: &'a Path,
    pub compiler: &'a Path,
    pub source_root: &'a Path,
    pub stdlib: &'a Path,
    pub config: &'a Path,
    pub partial: bool,
    pub allow_broken_links: bool,
    pub assets: Vec<InputAsset>,
    pub source: SourceState,
}

/// Index the module and its public declarations; a private helper contributes no search row.
fn module_search(module: &ApiModule) -> Vec<SearchEntry> {
    let mut search = Vec::new();
    search.push(SearchEntry {
        id: build::route_id(&Site::module_path(&module.id)),
        name: module.title.clone(),
        qualified_name: module.id.clone(),
        module: Some(module.id.clone()),
        target: Target::Page {
            id: build::route_id(&Site::module_path(&module.id)),
            anchor: None,
        },
        summary: description(&module.docs),
        kind: SearchKind::Module,
        binding: false,
        experimental: module.experimental,
        category: module.category.clone(),
        overloads: vec![],
        preview: None,
    });
    for d in module.declarations.iter().filter(|d| !d.private) {
        search.push(SearchEntry {
            id: d.id.clone(),
            name: d.name.clone(),
            qualified_name: format!("{}.{}", module.id, d.name),
            module: Some(module.id.clone()),
            target: Target::Page {
                id: d.route.clone(),
                anchor: None,
            },
            summary: description(if d.docs.is_empty() {
                d.overloads.first().map_or(&[], |o| o.docs.as_slice())
            } else {
                &d.docs
            }),
            kind: SearchKind::Declaration,
            binding: d.binding,
            experimental: d.experimental,
            category: module.category.clone(),
            preview: Some(if let Some(overload) = d.overloads.first() {
                let parameters = |values: &[Parameter]| {
                    values
                        .iter()
                        .map(|value| SearchParameter {
                            name: value.name.clone(),
                            ty: value.ty.source.clone(),
                            default: value.default.source.clone(),
                        })
                        .collect()
                };
                SearchPreview::Procedure {
                    parameters: parameters(&overload.parameters),
                    results: parameters(&overload.results),
                }
            } else if !d.value.source.is_empty() {
                SearchPreview::Value {
                    value: d.value.source.clone(),
                }
            } else {
                SearchPreview::DeclarationKind {
                    declaration_kind: d.kind,
                }
            }),
            overloads: if d.binding {
                vec![]
            } else {
                d.overloads
                    .iter()
                    .map(|o| SearchSignature {
                        parameters: o.parameters.iter().map(|p| p.ty.source.clone()).collect(),
                        results: o.results.iter().map(|p| p.ty.source.clone()).collect(),
                    })
                    .collect()
            },
        });
    }
    search
}

/// Keep publication config separate from canonical routes, for example the `/reference/` base.
fn site_data(
    site: &Site,
    publication: &publication::Publication,
    routes: Vec<PageRoute>,
) -> Result<SiteData, String> {
    Ok(SiteData {
        name: site.name.clone(),
        description: site.description.clone(),
        version: site.version.clone(),
        footer_version: env!("CARGO_PKG_VERSION").into(),
        breadcrumb: site.breadcrumb.clone(),
        page_base: site.base.clone(),
        social_site_name: publication.name.clone(),
        canonical_version: publication.canonical_version.clone(),
        social_image: (!publication.image.is_empty()).then(|| publication.image.clone()),
        canonical_origin: (!publication.origin.is_empty()).then(|| publication.origin.clone()),
        playground_url: (!site.playground.is_empty()).then(|| site.playground.clone()),
        logo: site.logo.as_ref().map(|l| format!("assets/{l}")),
        repository: (!site.repo.is_empty()).then(|| site.repo.clone()),
        header_links: {
            let mut links = site
                .pages_href
                .as_ref()
                .map(|href| {
                    Ok::<Link, String>(Link {
                        label: site.pages_title.clone(),
                        target: build::target(site, href, "")?,
                    })
                })
                .transpose()?
                .into_iter()
                .collect::<Vec<_>>();
            if !site.playground.is_empty() {
                links.push(Link {
                    label: "Playground".into(),
                    target: build::target(site, &site.playground, "")?,
                });
            }
            links
        },
        categories: site
            .categories
            .iter()
            .enumerate()
            .map(|(i, c)| {
                Ok(Category {
                    id: format!("category-{i}"),
                    name: c.name.clone(),
                    badge: c.badge.clone(),
                    tip: c.tip.clone(),
                    color: crate::doc::categories::export_color(&c.color)?,
                    dark_color: crate::doc::categories::export_color(&c.color_dark)?,
                })
            })
            .collect::<Result<_, String>>()?,
        navigation: navigation(site)?,
        routes,
    })
}

/// Export semantic data and declared assets without executing examples or requiring Node.
pub(in crate::doc) fn write(request: Request<'_>) -> Result<(), String> {
    let Request {
        site,
        guides,
        publication,
        out,
        compiler,
        source_root,
        stdlib,
        config,
        partial,
        allow_broken_links,
        assets,
        source,
    } = request;
    let stage = Staging::new(out)?;
    let formatted =
        crate::doc::declaration_format::Batch::prepare(site, &stage.path.join("formatter"))?;
    let mut inventory = Vec::new();
    let mut routes = Vec::new();
    let mut search = Vec::new();
    let mut private_routes = std::collections::BTreeSet::new();
    let mut add = |path: String, payload: Payload| -> Result<(), String> {
        let bytes = json(&payload)?;
        stage.put(&path, &bytes)?;
        inventory.push(Inventory {
            path,
            sha256: hash(&bytes),
            bytes: bytes.len() as u64,
        });
        Ok(())
    };
    for mi in 0..site.modules.len() {
        let module = build::module(
            site,
            mi,
            &|file, sha256| {
                Ok(InputDigest {
                    path: super::inputs::logical_path(file, source_root, stdlib, config, sha256)?,
                    sha256: sha256.into(),
                })
            },
            publication,
            &formatted,
        )?;
        private_routes.extend(
            module
                .declarations
                .iter()
                .filter(|d| d.private)
                .map(|d| d.id.clone()),
        );
        let path = shard("modules", &module.id);
        routes.extend(routes_for_module(&module, &path));
        search.extend(module_search(&module));
        add(path, Payload::Module(module))?;
    }
    let mut documents = guides.documents(site, publication)?;
    if !documents.iter().any(|d| d.id.is_empty()) {
        documents.push(synthetic_document(
            site,
            publication,
            DocumentKind::Home,
            &site.name,
            &site.description,
        ));
    }
    documents.push(synthetic_document(
        site,
        publication,
        DocumentKind::NotFound,
        "Page not found",
        "Search the docs or choose a module.",
    ));
    for document in documents {
        let path = shard("documents", &document.id);
        let output = if document.id.is_empty() {
            "index.html".into()
        } else {
            format!("{}.html", document.id)
        };
        let kind: PageKind = document.kind.into();
        let mut anchors = vec!["main-content".into()];
        heading_anchors(&document.nodes, &mut anchors);
        routes.push(PageRoute {
            id: document.id.clone(),
            label: document.title.clone(),
            route: document.id.clone(),
            output,
            kind,
            anchors,
            aliases: vec![],
            private: false,
            noindex: document.metadata.noindex,
            shard: path.clone(),
        });
        if !matches!(kind, PageKind::Home | PageKind::NotFound) {
            search.push(SearchEntry {
                id: document.id.clone(),
                name: document.title.clone(),
                qualified_name: document.title.clone(),
                module: None,
                target: Target::Page {
                    id: document.id.clone(),
                    anchor: None,
                },
                summary: description(&document.nodes),
                kind: if kind == PageKind::Tool {
                    SearchKind::Tool
                } else {
                    SearchKind::Guide
                },
                binding: false,
                experimental: false,
                category: None,
                overloads: vec![],
                preview: None,
            });
        }
        add(path, Payload::Document(document))?;
    }
    add(
        "search.json".into(),
        Payload::Search(SearchData {
            schema_version: 1,
            entries: search,
        }),
    )?;
    let site_data = site_data(site, publication, routes)?;
    let urls = site_data
        .routes
        .iter()
        .filter(|r| r.kind != PageKind::NotFound && !private_routes.contains(&r.id))
        .filter_map(|r| publication.url(&r.output, false))
        .collect::<Vec<_>>();
    add("site.json".into(), Payload::Site(site_data))?;
    let mut declared = Vec::new();
    for InputAsset {
        path,
        bytes,
        provenance,
    } in assets.into_iter().chain(publication.files(&urls))
    {
        stage.put(&path, &bytes)?;
        let media_type = match path.rsplit('.').next().unwrap_or("") {
            "json" => "application/json",
            "mjs" => "text/javascript",
            "wasm" => "application/wasm",
            "svg" => "image/svg+xml",
            "png" => "image/png",
            "xml" => "application/xml",
            _ => "text/plain",
        };
        declared.push(Asset {
            path,
            media_type: media_type.into(),
            sha256: hash(&bytes),
            bytes: bytes.len() as u64,
            provenance,
        });
    }
    let manifest = Manifest {
        schema: Schema::JacketDocSite,
        schema_version: 1,
        producer: Producer {
            jacket: format!(
                "{}:{}",
                env!("CARGO_PKG_VERSION"),
                hash(
                    &std::fs::read(std::env::current_exe().map_err(|e| e.to_string())?)
                        .map_err(|e| e.to_string())?
                )
            ),
            compiler: hash(&std::fs::read(compiler).map_err(|e| e.to_string())?),
            formatter: formatted.identity.as_ref().map(|identity| Formatter {
                version: identity.version.clone(),
                sha256: identity.sha256.clone(),
                profile: DeclarationFormatProfile::DocumentationDeclarationsV1,
                max_width: crate::doc::declaration_format::MAX_WIDTH,
                indent_width: crate::doc::declaration_format::INDENT_WIDTH,
            }),
        },
        source,
        partial,
        capabilities: if site.runtime {
            vec![Capability::BrowserRuntime, Capability::EditableExamples]
        } else {
            vec![]
        },
        shards: inventory,
        assets: declared,
    };
    manifest.validate()?;
    stage.put("manifest.json", &json(&Payload::Manifest(manifest))?)?;
    stage.put(".jacket-doc-export", b"")?;
    let clashes = site.path_clashes();
    if !clashes.is_empty() {
        return Err(clashes.join("\n"));
    }
    if !partial {
        let unlinked = site.unlinked_types();
        if !unlinked.is_empty() {
            return Err(format!(
                "{} unlinked type name(s): {}",
                unlinked.len(),
                unlinked.join("; ")
            ));
        }
        if !allow_broken_links {
            let mut broken = site.broken_links();
            broken.extend(
                site.modules
                    .iter()
                    .flat_map(|m| m.unresolved.iter().cloned()),
            );
            if !broken.is_empty() {
                return Err(format!(
                    "{} unresolved documentation link(s): {}",
                    broken.len(),
                    broken.join("; ")
                ));
            }
        }
    }
    stage.publish(out)
}
