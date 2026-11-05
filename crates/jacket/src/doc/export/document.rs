//! `CommonMark` events become data, never executable templates or producer-owned markup.
use super::contract::{Alignment, DocumentNode as Node, Target};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};

struct Frame<'a> {
    tag: Tag<'a>,
    children: Vec<Node>,
    heading_text: String,
}

/// Convert Markdown with the same table/strikethrough options as the documentation parser.
/// For example, raw `<script>` text remains a text node and is escaped by the component.
pub(in crate::doc) fn parse(
    source: &str,
    shift: u32,
    heading_ids: bool,
    resolve: &dyn Fn(&str) -> Result<Option<Target>, String>,
    fence: &dyn Fn(&str, &str) -> Result<Node, String>,
) -> Result<Vec<Node>, String> {
    let mut roots = Vec::new();
    let mut used_ids = std::collections::BTreeSet::new();
    let mut frames: Vec<Frame<'_>> = Vec::new();
    let mut code: Option<(String, String)> = None;
    let push = |node, frames: &mut Vec<Frame<'_>>, roots: &mut Vec<Node>| {
        if let Some(frame) = frames.last_mut() {
            frame.children.push(node);
        } else {
            roots.push(node);
        }
    };
    for event in Parser::new_ext(
        source,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH,
    ) {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                let info = match kind {
                    CodeBlockKind::Fenced(info) => info.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                code = Some((info, String::new()));
            }
            Event::End(pulldown_cmark::TagEnd::CodeBlock) => {
                let (info, text) = code.take().ok_or("unbalanced code fence")?;
                let destination = frames
                    .last_mut()
                    .map_or(&mut roots, |frame| &mut frame.children);
                example_fence(&info, &text, destination, fence)?;
            }
            Event::Text(text) if code.is_some() => {
                if let Some((_, body)) = code.as_mut() {
                    body.push_str(&text);
                }
            }
            Event::Start(tag) => frames.push(Frame {
                tag,
                children: Vec::new(),
                heading_text: String::new(),
            }),
            Event::End(_) => {
                let Frame {
                    tag,
                    children,
                    heading_text,
                } = frames.pop().ok_or("unbalanced Markdown node")?;
                let node = match tag {
                    Tag::HtmlBlock => {
                        for child in children {
                            push(child, &mut frames, &mut roots);
                        }
                        continue;
                    }
                    Tag::Paragraph => Node::Paragraph {
                        children,
                    },
                    Tag::Heading {
                        level, ..
                    } => {
                        let level = (level as u32 + shift).min(6);
                        let id = heading_ids.then(|| {
                            let base = super::slug::slug(&heading_text);
                            let mut id = base.clone();
                            let mut suffix = 1;
                            while !used_ids.insert(id.clone()) {
                                id = format!("{base}-{suffix}");
                                suffix += 1;
                            }
                            id
                        });
                        Node::Heading {
                            id,
                            level,
                            children,
                        }
                    }
                    Tag::Emphasis => Node::Emphasis {
                        children,
                    },
                    Tag::Strong => Node::Strong {
                        children,
                    },
                    Tag::Strikethrough => Node::Strikethrough {
                        children,
                    },
                    Tag::BlockQuote(_) => Node::Quote {
                        children,
                    },
                    Tag::List(start) => Node::List {
                        start,
                        children,
                    },
                    Tag::Item => Node::ListItem {
                        children,
                    },
                    Tag::Link {
                        dest_url,
                        title,
                        link_type,
                        ..
                    } => {
                        let destination = if link_type == pulldown_cmark::LinkType::Email {
                            format!("mailto:{dest_url}")
                        } else {
                            dest_url.to_string()
                        };
                        let Some(target) = resolve(&destination)? else {
                            for child in children {
                                push(child, &mut frames, &mut roots);
                            }
                            continue;
                        };
                        Node::Link {
                            target,
                            title: title.to_string(),
                            children,
                        }
                    }
                    Tag::Image {
                        dest_url,
                        title,
                        ..
                    } => {
                        let target = resolve(&dest_url)?
                            .ok_or_else(|| format!("unresolved image {dest_url}"))?;
                        Node::Image {
                            target,
                            title: title.to_string(),
                            alternative: plain(&children),
                        }
                    }
                    Tag::Table(alignments) => {
                        let mut rows = children.into_iter();
                        let head = match rows.next() {
                            Some(Node::TableRow {
                                cells,
                            }) => cells,
                            None => Vec::new(),
                            _ => return Err("table header is not a row".into()),
                        };
                        let alignments = alignments
                            .into_iter()
                            .map(|a| match a {
                                pulldown_cmark::Alignment::None => Alignment::None,
                                pulldown_cmark::Alignment::Left => Alignment::Left,
                                pulldown_cmark::Alignment::Center => Alignment::Center,
                                pulldown_cmark::Alignment::Right => Alignment::Right,
                            })
                            .collect();
                        Node::Table {
                            alignments,
                            head,
                            rows: rows.collect(),
                        }
                    }
                    Tag::TableHead | Tag::TableRow => Node::TableRow {
                        cells: children,
                    },
                    Tag::TableCell => Node::TableCell {
                        children,
                    },
                    other => return Err(format!("unsupported Markdown node {other:?}")),
                };
                push(node, &mut frames, &mut roots);
            }
            Event::Text(text) => {
                for frame in &mut frames {
                    if matches!(frame.tag, Tag::Heading { .. }) {
                        frame.heading_text.push_str(&text);
                    }
                }
                push(
                    Node::Text {
                        text: text.to_string(),
                    },
                    &mut frames,
                    &mut roots,
                );
            }
            Event::Html(text) | Event::InlineHtml(text) => push(
                Node::Text {
                    text: text.to_string(),
                },
                &mut frames,
                &mut roots,
            ),
            Event::Code(text) => {
                for frame in &mut frames {
                    if matches!(frame.tag, Tag::Heading { .. }) {
                        frame.heading_text.push_str(&text);
                    }
                }
                push(
                    Node::Code {
                        text: text.to_string(),
                    },
                    &mut frames,
                    &mut roots,
                );
            }
            Event::SoftBreak => push(
                Node::Break {
                    hard: false,
                },
                &mut frames,
                &mut roots,
            ),
            Event::HardBreak => push(
                Node::Break {
                    hard: true,
                },
                &mut frames,
                &mut roots,
            ),
            Event::Rule => push(Node::Rule {}, &mut frames, &mut roots),
            other => return Err(format!("unsupported Markdown event {other:?}")),
        }
    }
    if !frames.is_empty() || code.is_some() {
        return Err("unterminated Markdown node".into());
    }
    combine_comparisons(&mut roots);
    Ok(roots)
}

/// Explicit fence metadata groups generated examples without interpreting prose or DOM position.
fn example_fence(
    info: &str,
    text: &str,
    nodes: &mut Vec<Node>,
    fence: &dyn Fn(&str, &str) -> Result<Node, String>,
) -> Result<(), String> {
    use super::contract::{ExampleResult, ExampleResultKind, ExampleTitle};
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SourceMeta {
        kind: ExampleTitle,
        #[serde(default)]
        note: Option<String>,
    }
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ResultMeta {
        kind: ExampleResultKind,
    }
    let mut fields = info.splitn(3, char::is_whitespace);
    let language = fields.next().unwrap_or("");
    let marker = fields.next().unwrap_or("");
    if !marker.starts_with("jacket-example/") && !marker.starts_with("jacket-result/") {
        nodes.push(fence(info, text)?);
        return Ok(());
    }
    let metadata = fields.next().unwrap_or("");
    match marker {
        "jacket-example/1" => {
            let meta: SourceMeta = serde_json::from_str(metadata)
                .map_err(|error| format!("invalid example metadata: {error}"))?;
            let source = fence(language, text)?;
            if !matches!(source, Node::CodeBlock { .. } | Node::Fence { .. }) {
                return Err("example source is not a code fence".into());
            }
            nodes.push(Node::ExampleGroup {
                title: meta.kind,
                note: meta.note,
                source: Box::new(source),
                result: None,
            });
        }
        "jacket-result/1" => {
            let meta: ResultMeta = serde_json::from_str(metadata)
                .map_err(|error| format!("invalid example result metadata: {error}"))?;
            let body = fence(language, text)?;
            if !matches!(body, Node::CodeBlock { .. }) {
                return Err("example result must be a static code block".into());
            }
            let Some(Node::ExampleGroup {
                title: ExampleTitle::Example,
                result,
                ..
            }) = nodes.last_mut()
            else {
                return Err("example result has no immediately preceding example source".into());
            };
            if result.is_some() {
                return Err("example has more than one result".into());
            }
            *result = Some(ExampleResult {
                kind: meta.kind,
                body: Box::new(body),
            });
        }
        _ => return Err(format!("unsupported example metadata version {marker}")),
    }
    Ok(())
}

/// Formatter fences retain their raw code while declaring a paired comparison to presentation.
fn combine_comparisons(nodes: &mut Vec<Node>) {
    for node in nodes.iter_mut() {
        match node {
            Node::Paragraph {
                children,
            }
            | Node::Heading {
                children, ..
            }
            | Node::Emphasis {
                children,
            }
            | Node::Strong {
                children,
            }
            | Node::Strikethrough {
                children,
            }
            | Node::Link {
                children, ..
            }
            | Node::List {
                children, ..
            }
            | Node::ListItem {
                children,
            }
            | Node::Quote {
                children,
            }
            | Node::TableCell {
                children,
            } => combine_comparisons(children),
            Node::Table {
                head,
                rows,
                ..
            } => {
                combine_comparisons(head);
                combine_comparisons(rows);
            }
            Node::TableRow {
                cells,
            } => combine_comparisons(cells),
            _ => {}
        }
    }
    let mut input = std::mem::take(nodes).into_iter().peekable();
    while let Some(node) = input.next() {
        let before = matches!(&node,Node::CodeBlock {info,..} if info.split_whitespace().nth(1)==Some("before"));
        let after = matches!(input.peek(),Some(Node::CodeBlock {info,..}) if info.split_whitespace().nth(1)==Some("after"));
        if before && after {
            if let (
                Node::CodeBlock {
                    source: before, ..
                },
                Some(Node::CodeBlock {
                    source: after,
                    info,
                    ..
                }),
            ) = (node, input.next())
            {
                nodes.push(Node::Compare {
                    before,
                    after,
                    settings: info
                        .split_whitespace()
                        .skip(2)
                        .collect::<Vec<_>>()
                        .join(" ")
                        .replace('=', " = "),
                });
            }
        } else {
            nodes.push(node);
        }
    }
}

/// Text content for headings and search; it never strips tags from an HTML string.
pub(in crate::doc) fn plain(nodes: &[Node]) -> String {
    let mut result = String::new();
    for node in nodes {
        match node {
            Node::Text {
                text,
            }
            | Node::Code {
                text,
            } => result.push_str(text),
            Node::Paragraph {
                children,
            }
            | Node::Heading {
                children, ..
            }
            | Node::Emphasis {
                children,
            }
            | Node::Strong {
                children,
            }
            | Node::Strikethrough {
                children,
            }
            | Node::Link {
                children, ..
            }
            | Node::List {
                children, ..
            }
            | Node::ListItem {
                children,
            }
            | Node::Quote {
                children,
            }
            | Node::TableCell {
                children,
            } => result.push_str(&plain(children)),
            Node::Image {
                alternative, ..
            } => result.push_str(alternative),
            Node::Break {
                ..
            } => result.push('\n'),
            _ => {}
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_example_metadata_pairs_source_and_result_and_preserves_fixed_source() {
        let source = "```jai jacket-example/1 {\"kind\":\"example\"}\nvalue := 0;\n```\n\n```diagnostic jacket-result/1 {\"kind\":\"diagnostic\"}\nwarning: literal 0\n 1 - old;\n```\n\n```jai jacket-example/1 {\"kind\":\"fixed\"}\nvalue := 1;\n```\n";
        let infos = std::cell::RefCell::new(Vec::new());
        let nodes = parse(source, 0, true, &|_| Ok(None), &|info, code| {
            infos.borrow_mut().push(info.to_owned());
            Ok(Node::CodeBlock {
                language: info.split_whitespace().next().unwrap_or("").into(),
                info: info.into(),
                source: super::super::contract::ResolvedText {
                    source: code.into(),
                    spans: Vec::new(),
                },
            })
        })
        .unwrap();
        let value = serde_json::to_value(&nodes).unwrap();
        assert_eq!(value.as_array().unwrap().len(), 2);
        assert_eq!(value[0]["kind"], "example_group");
        assert_eq!(value[0]["title"], "example");
        assert_eq!(value[0]["source"]["source"]["source"], "value := 0;\n");
        assert_eq!(value[0]["result"]["kind"], "diagnostic");
        assert_eq!(
            value[0]["result"]["body"]["source"]["source"],
            "warning: literal 0\n 1 - old;\n"
        );
        assert_eq!(value[1]["kind"], "example_group");
        assert_eq!(value[1]["title"], "fixed");
        assert!(value[1]["result"].is_null());
        assert_eq!(*infos.borrow(), ["jai", "diagnostic", "jai"]);
    }

    #[test]
    fn example_metadata_is_explicit_versioned_and_result_ownership_is_checked() {
        let fence = |info: &str, code: &str| {
            Ok(Node::CodeBlock {
                language: info.split_whitespace().next().unwrap_or("").into(),
                info: info.into(),
                source: super::super::contract::ResolvedText {
                    source: code.into(),
                    spans: Vec::new(),
                },
            })
        };
        let ordinary = parse(
            "Example:\n\n```jai\nvalue := 0;\n```\n\n```diagnostic\nwarning: ordinary\n```\n",
            0,
            true,
            &|_| Ok(None),
            &fence,
        )
        .unwrap();
        assert!(matches!(&ordinary[0], Node::Paragraph { .. }));
        assert_eq!(
            ordinary.len(),
            3,
            "unmarked authored content must not become an example pair"
        );
        for invalid in [
            "```diagnostic jacket-result/1 {\"kind\":\"diagnostic\"}\nwarning: orphan\n```\n",
            "```jai jacket-example/2 {\"kind\":\"example\"}\nx := 0;\n```\n",
            "```jai jacket-example/1 {\"kind\":\"unknown\"}\nx := 0;\n```\n",
            "```jai jacket-example/1 {\"kind\":\"example\",\"unknown\":true}\nx := 0;\n```\n",
        ] {
            assert!(
                parse(invalid, 0, true, &|_| Ok(None), &fence).is_err(),
                "invalid explicit example metadata was accepted: {invalid}"
            );
        }
    }


    #[test]
    fn nested_images_keep_alternative_text_and_email_autolinks_keep_their_scheme() {
        let nodes = parse(
            "![foo ![bar](/url)](/url2)\n\n<foo@example.com>\n",
            0,
            false,
            &|url| {
                Ok(Some(Target::External {
                    url: url.into(),
                }))
            },
            &|_, _| Err("unexpected fence".into()),
        )
        .unwrap();
        let Node::Paragraph {
            children,
        } = &nodes[0]
        else {
            panic!("image paragraph")
        };
        assert!(
            matches!(&children[0], Node::Image { alternative, .. } if alternative == "foo bar")
        );
        let Node::Paragraph {
            children,
        } = &nodes[1]
        else {
            panic!("email paragraph")
        };
        assert!(
            matches!(&children[0], Node::Link { target: Target::External { url }, .. } if url == "mailto:foo@example.com")
        );
    }

    #[test]
    fn repeated_heading_labels_keep_the_first_fragment_and_get_unique_suffixes() {
        let nodes = parse(
            "# Title\n\n## Options\n\nFirst.\n\n## Options\n\nSecond.\n",
            0,
            true,
            &|_| Ok(None),
            &|_, _| Err("unexpected fence".into()),
        )
        .unwrap();
        let ids = nodes
            .iter()
            .filter_map(|node| match node {
                Node::Heading {
                    id: Some(id), ..
                } => Some(id.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(ids, ["title", "options", "options-1"]);
    }

    #[test]
    fn comments_are_typed_escaped_content_with_exact_commonmark_options() {
        let nodes = parse(
            "# Header\n\n<script>alert(1)</script>\n\n~~gone~~ and `code`\n",
            2,
            true,
            &|_| Ok(None),
            &|_, _| Err("unexpected fence".into()),
        )
        .unwrap();
        assert!(
            matches!(&nodes[0], Node::Heading { level: 3, id: Some(id), .. } if id == "header")
        );
        assert!(
            nodes
                .iter()
                .any(|n| matches!(n, Node::Text { text } if text.contains("<script>")))
        );
        assert!(plain(&nodes).contains("gone and code"));
    }
}
