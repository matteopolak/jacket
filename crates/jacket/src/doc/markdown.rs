//! Markdown to HTML for doc comments and the guide pages. Code blocks are not rendered here:
//! they come back as segments, so a page can turn a ```` ```jai ```` block into an example with
//! its own buttons.
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

/// One block of rendered HTML (a paragraph, a list, a heading, a table), or a fenced code block.
#[derive(Clone, Debug)]
pub enum Segment {
    Html(String),
    Code {
        /// The text after the opening fence, `jai,skip_on(wasm)`; empty for a bare fence.
        info: String,
        code: String,
    },
}

pub struct Settings<'a> {
    /// Added to the level of every heading (`#` in a doc comment is a third-level heading).
    pub heading_shift: usize,
    /// Give headings an `id` (GitHub's rule) so they can be linked.
    pub heading_ids: bool,
    /// Where a link goes: the target to write, or `None` to keep its text without the link.
    pub link: &'a dyn Fn(&str) -> Option<String>,
}

pub fn esc(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
    out
}

/// The plain text of HTML produced here: tags dropped, entities undone.
pub fn plain(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

fn level(shift: usize, base: HeadingLevel) -> HeadingLevel {
    let n = (base as usize + shift).min(6);
    match n {
        1 => HeadingLevel::H1,
        2 => HeadingLevel::H2,
        3 => HeadingLevel::H3,
        4 => HeadingLevel::H4,
        5 => HeadingLevel::H5,
        _ => HeadingLevel::H6,
    }
}

/// GitHub's anchor for a heading: lower case, spaces to hyphens, punctuation dropped.
pub fn slug(text: &str) -> String {
    text.trim()
        .to_lowercase()
        .chars()
        .filter_map(|c| match c {
            c if c.is_alphanumeric() || c == '_' || c == '-' => Some(c),
            ' ' => Some('-'),
            _ => None,
        })
        .collect()
}

fn flush(buf: &mut Vec<Event<'_>>, out: &mut Vec<Segment>) {
    if buf.is_empty() {
        return;
    }
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, std::mem::take(buf).into_iter());
    out.push(Segment::Html(html));
}

/// The first line of each indented code block of `md`: Markdown makes one of a paragraph's
/// worth of lines after a blank line that start four spaces in, and that is almost never what a
/// doc comment means, since only a fenced block can be run.
pub fn indented_blocks(md: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut open = false;
    for event in Parser::new_ext(md, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH) {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)) => open = true,
            Event::End(TagEnd::CodeBlock) => open = false,
            Event::Text(t) if open => {
                out.push(t.lines().next().unwrap_or("").trim().to_owned());
                open = false;
            }
            _ => {}
        }
    }
    out
}

pub fn segments(md: &str, settings: &Settings<'_>) -> Vec<Segment> {
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    let events: Vec<Event<'_>> = Parser::new_ext(md, options).collect();
    let mut out = Vec::new();
    let mut buf: Vec<Event<'_>> = Vec::new();
    let mut kept_links: Vec<bool> = Vec::new();
    // How many tags are open: a block ends when this is back to 0.
    let mut depth = 0usize;
    let mut i = 0;
    while i < events.len() {
        let event = events[i].clone();
        i += 1;
        match &event {
            Event::Start(Tag::CodeBlock(_)) => {}
            Event::Start(_) => depth += 1,
            Event::End(_) => depth = depth.saturating_sub(1),
            _ => {}
        }
        let block_ends = depth == 0 && matches!(event, Event::End(_) | Event::Rule);
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                flush(&mut buf, &mut out);
                let info = match kind {
                    CodeBlockKind::Fenced(info) => info.trim().to_owned(),
                    CodeBlockKind::Indented => String::new(),
                };
                let mut code = String::new();
                while i < events.len() {
                    let inner = events[i].clone();
                    i += 1;
                    match inner {
                        Event::End(TagEnd::CodeBlock) => break,
                        Event::Text(t) => code.push_str(&t),
                        _ => {}
                    }
                }
                out.push(Segment::Code {
                    info,
                    code,
                });
            }
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => match (settings.link)(&dest_url) {
                Some(target) => {
                    kept_links.push(true);
                    buf.push(Event::Start(Tag::Link {
                        link_type,
                        dest_url: target.into(),
                        title,
                        id,
                    }));
                }
                None => kept_links.push(false),
            },
            Event::End(TagEnd::Link) => {
                if kept_links.pop().unwrap_or(true) {
                    buf.push(Event::End(TagEnd::Link));
                }
            }
            Event::Html(text) | Event::InlineHtml(text) => buf.push(Event::Text(text)),
            Event::Start(Tag::Heading {
                level: base,
                id: _,
                classes,
                attrs,
            }) => {
                let mut text = String::new();
                for later in &events[i..] {
                    match later {
                        Event::End(TagEnd::Heading(_)) => break,
                        Event::Text(t) | Event::Code(t) => text.push_str(t),
                        _ => {}
                    }
                }
                buf.push(Event::Start(Tag::Heading {
                    level: level(settings.heading_shift, base),
                    id: settings.heading_ids.then(|| slug(&text).into()),
                    classes,
                    attrs,
                }));
            }
            Event::End(TagEnd::Heading(base)) => {
                buf.push(Event::End(TagEnd::Heading(level(
                    settings.heading_shift,
                    base,
                ))));
            }
            other => buf.push(other),
        }
        if block_ends {
            flush(&mut buf, &mut out);
        }
    }
    flush(&mut buf, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(md: &str) -> Vec<Segment> {
        let link = |dest: &str| {
            dest.strip_prefix("jaidoc:").map_or_else(
                || Some(dest.to_owned()),
                |rest| (rest != "gone").then(|| format!("/{rest}.html")),
            )
        };
        segments(
            md,
            &Settings {
                heading_shift: 2,
                heading_ids: false,
                link: &link,
            },
        )
    }

    #[test]
    fn code_blocks_split_the_text_and_bad_links_lose_their_target() {
        let parts = render(
            "Use [`trim`](jaidoc:trim) or [x](jaidoc:gone).\n\n```jai,no_run\na := 1;\n```\n\n# Title\n",
        );
        assert_eq!(parts.len(), 3);
        let Segment::Html(first) = &parts[0] else {
            panic!("{parts:?}");
        };
        assert!(
            first.contains("<a href=\"/trim.html\"><code>trim</code></a>"),
            "{first}"
        );
        assert!(first.contains(" or x."), "{first}");
        assert!(
            matches!(&parts[1], Segment::Code { info, code } if info == "jai,no_run" && code == "a := 1;\n")
        );
        assert!(matches!(&parts[2], Segment::Html(h) if h.contains("<h3>Title</h3>")));
    }

    #[test]
    fn raw_html_is_text() {
        let parts = render("a <b>bold</b> claim");
        assert!(matches!(&parts[0], Segment::Html(h) if h.contains("&lt;b&gt;")));
    }

    #[test]
    fn anchors_follow_githubs_rule() {
        assert_eq!(slug("Per-test options (v2)"), "per-test-options-v2");
    }

    #[test]
    fn indented_blocks_are_found_and_fenced_ones_are_not() {
        let md =
            "Text.\n\n    a := 1;\n    b := 2;\n\n```jai\nc := 3;\n```\n\n- item\n\n  continued\n";
        assert_eq!(indented_blocks(md), ["a := 1;"]);
    }
}
