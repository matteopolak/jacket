//! Resolve compiler facts to semantic declarations, documents and tokens.
use super::contract::{
    ApiModule, Declaration, DeclarationKind, DefinitionAnchor, DocumentNode, FormattedDefinition,
    InputDigest, Location, Member, Overload, Parameter, ResolvedText, Target, TextSpan, TokenRole,
};
use super::document;
use crate::doc::{
    example, highlight, model,
    site::{self, Site},
};

/// Stable public route identity uses the existing collision-aware destination, never source lines.
pub(in crate::doc) fn route_id(output: &str) -> String {
    if output == "index.html" {
        String::new()
    } else {
        output.strip_suffix(".html").unwrap_or(output).into()
    }
}

/// Convert a resolved URL to a typed destination; script and native-file schemes are errors.
pub(in crate::doc) fn target(site: &Site, href: &str, from: &str) -> Result<Target, String> {
    if ["https://", "http://", "mailto:"].iter().any(|prefix| {
        href.get(..prefix.len())
            .is_some_and(|part| part.eq_ignore_ascii_case(prefix))
    }) {
        return Ok(Target::External {
            url: href.into(),
        });
    }
    if href.contains(':') || href.starts_with("//") || href.contains('\\') {
        return Err(format!("{from}: unsupported documentation URL {href:?}"));
    }
    let (path, anchor) = href.split_once('#').unwrap_or((href, ""));
    let path = if path.is_empty() {
        from
    } else {
        path.strip_prefix(&site.base).unwrap_or(path)
    };
    Ok(Target::Page {
        id: route_id(path),
        anchor: (!anchor.is_empty()).then(|| anchor.into()),
    })
}

/// Token spans use the shared lexical scanner and the compiler's resolved declaration identities.
pub(in crate::doc) fn text(
    site: &Site,
    mi: usize,
    source: &str,
    refs: &[model::TypeRef],
    own: Option<&str>,
) -> Result<ResolvedText, String> {
    let raw_tokens = highlight::linked_tokens(source, &|path| {
        for end in path
            .match_indices('.')
            .map(|(i, _)| i)
            .chain(std::iter::once(path.len()))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            let Some((module, name)) = site.resolved(&path[..end], refs, mi) else {
                continue;
            };
            if module == mi && own == Some(name) {
                return None;
            }
            return Some((end, site.item_href(&site.modules[module].id, name)));
        }
        None
    });
    resolved_text(site, source, raw_tokens)
}

fn resolved_text(
    site: &Site,
    source: &str,
    raw_tokens: Vec<highlight::Token>,
) -> Result<ResolvedText, String> {
    let mut tokens: Vec<highlight::Token> = Vec::new();
    for token in raw_tokens {
        if let Some(last) = tokens.last_mut()
            && last.end == token.start
            && last.role == token.role
            && last.target == token.target
        {
            last.end = token.end;
        } else {
            tokens.push(token);
        }
    }
    let spans = tokens
        .into_iter()
        .map(|token| {
            Ok(TextSpan {
                start: token.start as u64,
                end: token.end as u64,
                role: if token.target.is_some() {
                    TokenRole::Type
                } else {
                    match token.role {
                        Some("k") => TokenRole::Keyword,
                        Some("t") => TokenRole::Type,
                        Some("s") => TokenRole::String,
                        Some("n") => TokenRole::Number,
                        Some("c") => TokenRole::Comment,
                        Some("d") => TokenRole::Directive,
                        _ => TokenRole::Plain,
                    }
                },
                target: token
                    .target
                    .as_deref()
                    .map(|href| target(site, href, ""))
                    .transpose()?,
            })
        })
        .collect::<Result<_, String>>()?;
    Ok(ResolvedText {
        source: source.into(),
        spans,
    })
}

/// Resolve compiler-owned occurrences after the formatter has chosen every displayed byte.
fn formatted_definition(
    site: &Site,
    mi: usize,
    item: &model::Item,
    definition: &model::Definition,
    index: usize,
    batch: &crate::doc::declaration_format::Batch,
    route: &str,
) -> Result<FormattedDefinition, String> {
    let prepared = batch.get(definition)?;
    let mut occurrences = std::collections::BTreeMap::new();
    let mut anchors = Vec::new();
    let mut member_occurrences = std::collections::BTreeMap::<&str, usize>::new();
    for token in &definition.tokens {
        let (start, end) = prepared.range(definition, token.start, token.end)?;
        if token.reference {
            occurrences.insert(start, token.text.as_str());
        }
        for name in token.names.iter().filter(|name| {
            definition
                .members
                .iter()
                .any(|member| &member.name == *name)
        }) {
            let first = !item.definitions[..index]
                .iter()
                .any(|earlier| earlier.members.iter().any(|member| &member.name == name));
            let ordinal = member_occurrences.entry(name.as_str()).or_default();
            *ordinal += 1;
            let id = if first && *ordinal == 1 {
                format!("m-{name}")
            } else if *ordinal == 1 {
                format!("d{}-m-{name}", index + 1)
            } else {
                format!("d{}-m-{name}-{ordinal}", index + 1)
            };
            anchors.push(DefinitionAnchor {
                id,
                name: name.clone(),
                start: start as u64,
                end: end as u64,
            });
        }
    }
    let raw = highlight::resolved_tokens(&prepared.source, &|start, path| {
        let occurrence = occurrences.get(&start)?;
        if *occurrence != path {
            return None;
        }
        for end in path
            .match_indices('.')
            .map(|(offset, _)| offset)
            .chain(std::iter::once(path.len()))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            let Some((module, name)) = site.resolved(&path[..end], &definition.refs, mi) else {
                continue;
            };
            if module == mi && name == item.name {
                return None;
            }
            return Some((end, site.item_href(&site.modules[module].id, name)));
        }
        None
    });
    Ok(FormattedDefinition {
        id: format!("d{}", index + 1),
        kind: declaration_kind(definition.kind),
        source: location(&definition.file, definition.line),
        source_url: site.source_url(&definition.file, definition.line),
        platforms: definition.platforms.clone(),
        scope: definition.scope.clone(),
        docs: nodes_for_page(
            site,
            mi,
            &definition.docs,
            &definition.file,
            definition.line,
            &format!("{route}.html"),
            3,
        )?,
        code: resolved_text(site, &prepared.source, raw)?,
        anchors,
    })
}

fn declaration_kind(kind: model::Kind) -> DeclarationKind {
    match kind {
        model::Kind::Procedure => DeclarationKind::Procedure,
        model::Kind::Struct => DeclarationKind::Struct,
        model::Kind::Union => DeclarationKind::Union,
        model::Kind::Enum => DeclarationKind::Enum,
        model::Kind::EnumFlags => DeclarationKind::EnumFlags,
        model::Kind::Type => DeclarationKind::Type,
        model::Kind::Constant => DeclarationKind::Constant,
    }
}

fn called_text(site: &Site, mi: usize, source: &str) -> Result<ResolvedText, String> {
    let spans = highlight::called_tokens(source, &|name| site.call_link(&site.modules[mi], name))
        .into_iter()
        .map(|token| {
            Ok(TextSpan {
                start: token.start as u64,
                end: token.end as u64,
                role: match token.role {
                    Some("k") => TokenRole::Keyword,
                    Some("t") => TokenRole::Type,
                    Some("s") => TokenRole::String,
                    Some("n") => TokenRole::Number,
                    Some("c") => TokenRole::Comment,
                    Some("d") => TokenRole::Directive,
                    _ => TokenRole::Plain,
                },
                target: token
                    .target
                    .as_deref()
                    .map(|href| target(site, href, ""))
                    .transpose()?,
            })
        })
        .collect::<Result<_, String>>()?;
    Ok(ResolvedText {
        source: source.into(),
        spans,
    })
}

/// Plain code shares the documentation token scanner without resolving compiler names.
pub(in crate::doc) fn lexical_text(source: &str) -> ResolvedText {
    let spans = highlight::linked_tokens(source, &|_| None)
        .into_iter()
        .map(|token| TextSpan {
            start: token.start as u64,
            end: token.end as u64,
            target: None,
            role: match token.role {
                Some("k") => TokenRole::Keyword,
                Some("t") => TokenRole::Type,
                Some("s") => TokenRole::String,
                Some("n") => TokenRole::Number,
                Some("c") => TokenRole::Comment,
                Some("d") => TokenRole::Directive,
                _ => TokenRole::Plain,
            },
        })
        .collect();
    ResolvedText {
        source: source.into(),
        spans,
    }
}

pub(in crate::doc) fn nodes_for_page(
    site: &Site,
    mi: usize,
    md: &str,
    file: &str,
    line: usize,
    page: &str,
    shift: u32,
) -> Result<Vec<DocumentNode>, String> {
    let module = &site.modules[mi];
    document::parse(
        md,
        shift,
        false,
        &|href| {
            site.doc_link(href, page)
                .map(|href| target(site, &href, page))
                .transpose()
        },
        &|info, code| {
            let provenance = location(file, line);
            Ok(DocumentNode::Fence {
                example: Box::new(example::exported(
                    site,
                    module,
                    (&provenance.file, line),
                    (info, code),
                    called_text(site, mi, code)?,
                )),
            })
        },
    )
}

fn location(file: &str, line: usize) -> Location {
    // Compiler-local provenance stays private. Out-of-root imports retain a filename only.
    let file = if super::contract::validate_path(file).is_ok() {
        file.into()
    } else {
        format!(
            "external/{}",
            std::path::Path::new(file)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        )
    };
    Location {
        file,
        line: line as u64,
    }
}

fn parameter(
    site: &Site,
    mi: usize,
    p: &model::Param,
    file: &str,
    line: usize,
    page: &str,
) -> Result<Parameter, String> {
    Ok(Parameter {
        name: p.name.clone(),
        ty: text(site, mi, &p.ty, &p.refs, None)?,
        default: text(site, mi, &p.default, &p.refs, None)?,
        docs: nodes_for_page(site, mi, &p.doc, file, line, page, 3)?,
    })
}

pub(in crate::doc) fn module(
    site: &Site,
    mi: usize,
    dependency: &dyn Fn(&std::path::Path, &str) -> Result<InputDigest, String>,
    publication: &super::publication::Publication,
    formatted: &crate::doc::declaration_format::Batch,
) -> Result<ApiModule, String> {
    let m = &site.modules[mi];
    let mut declarations = Vec::new();
    for (i, item) in m.items.iter().chain(&m.private).enumerate() {
        let route = route_id(&site.item_path(&m.id, &item.name));
        let labels = site::overload_labels(item);
        let overloads: Vec<Overload> = item
            .overloads
            .iter()
            .enumerate()
            .map(|(n, o)| {
                let refs = o
                    .params
                    .iter()
                    .chain(&o.returns)
                    .flat_map(|p| p.refs.iter().cloned())
                    .collect::<Vec<_>>();
                Ok(Overload {
                    id: format!("o{}", n + 1),
                    label: (item.overloads.len() > 1).then(|| labels[n].clone()),
                    source: location(&o.file, o.line),
                    signature: text(
                        site,
                        mi,
                        &format!("{} :: {}", item.name, o.signature),
                        &refs,
                        Some(&item.name),
                    )?,
                    docs: nodes_for_page(
                        site,
                        mi,
                        &o.docs,
                        &o.file,
                        o.line,
                        &format!("{route}.html"),
                        2,
                    )?,
                    parameters: o
                        .params
                        .iter()
                        .map(|p| parameter(site, mi, p, &o.file, o.line, &format!("{route}.html")))
                        .collect::<Result<_, _>>()?,
                    results: o
                        .returns
                        .iter()
                        .map(|p| parameter(site, mi, p, &o.file, o.line, &format!("{route}.html")))
                        .collect::<Result<_, _>>()?,
                    flags: o.flags.clone(),
                    platforms: o.platforms.clone(),
                })
            })
            .collect::<Result<_, String>>()?;
        // Complete formatted definitions already contain every conditional field and its docs.
        // A second canonical table would duplicate content and conflate conditional names.
        let members = item
            .members
            .iter()
            .filter(|_| item.definitions.is_empty())
            .map(|member| {
                Ok(Member {
                    id: format!("m-{}", member.name),
                    name: member.name.clone(),
                    ty: text(site, mi, &member.ty, &member.refs, None)?,
                    docs: nodes_for_page(
                        site,
                        mi,
                        &member.doc,
                        &item.file,
                        item.line,
                        &format!("{route}.html"),
                        3,
                    )?,
                })
            })
            .collect::<Result<_, String>>()?;
        let docs = nodes_for_page(
            site,
            mi,
            &item.docs,
            &item.file,
            item.line,
            &format!("{route}.html"),
            1,
        )?;
        let metadata = publication.declaration(
            site,
            &m.id,
            &item.name,
            if docs.is_empty() {
                overloads.first().map_or(&[], |o| o.docs.as_slice())
            } else {
                &docs
            },
            i >= m.items.len(),
        );
        declarations.push(Declaration {
            id: route.clone(),
            route: route.clone(),
            name: item.name.clone(),
            kind: declaration_kind(item.kind),
            source: location(&item.file, item.line),
            source_url: item.overloads.first().map_or_else(
                || site.source_url(&item.file, item.line),
                |o| site.source_url(&o.file, o.line),
            ),
            private: i >= m.items.len(),
            binding: item.binding,
            experimental: item.experimental,
            since: item.since.clone(),
            platforms: item.platforms.clone(),
            group: item.group.clone(),
            docs,
            metadata,
            definitions: item
                .definitions
                .iter()
                .enumerate()
                .map(|(index, definition)| {
                    formatted_definition(site, mi, item, definition, index, formatted, &route)
                })
                .collect::<Result<_, String>>()?,
            value: text(site, mi, &item.value, &item.refs, Some(&item.name))?,
            members,
            overloads,
        });
    }
    let module_docs = nodes_for_page(site, mi, &m.docs, &m.file, 1, &Site::module_path(&m.id), 1)?;
    let metadata = publication.module(site, &m.id, &module_docs);
    Ok(ApiModule {
        id: m.id.clone(),
        title: m.title().into(),
        source: location(&m.file, m.docs_line),
        source_url: site.source_url(&m.file, 1),
        docs: module_docs,
        metadata,
        implicit: m.implicit,
        experimental: m.experimental,
        wasm: m.wasm,
        since: m.since.clone(),
        platforms: m.platforms.clone(),
        category: m.category.map(|i| format!("category-{i}")),
        group: m.group.clone(),
        parameters: m
            .parameters
            .iter()
            .map(|p| {
                parameter(
                    site,
                    mi,
                    &model::Param {
                        name: p.name.clone(),
                        ty: p.ty.clone(),
                        default: p.default.clone(),
                        doc: p.doc.clone(),
                        refs: p.refs.clone(),
                    },
                    &m.file,
                    m.docs_line,
                    &Site::module_path(&m.id),
                )
            })
            .collect::<Result<_, _>>()?,
        declarations,
        dependencies: m
            .dependencies
            .iter()
            .map(|(file, sha256)| dependency(file, sha256))
            .collect::<Result<_, _>>()?,
    })
}
