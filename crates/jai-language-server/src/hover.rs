//! Hover contents. Each hover is built once as a list of blocks and then written in the
//! format the client asked for: Markdown (code in fenced `jai` blocks, sections after a
//! thematic break with an emphasized label) or plain text (the same content with a
//! `─── label ───` line between sections).
use crate::MarkupKind;

/// Inline text of a paragraph or a format row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Inline {
    Text(String),
    Code(String),
}

/// One row of a format-string hover: a `%` and what it prints.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FormatRow {
    /// The specifier under the cursor (marked when there is more than one).
    pub(crate) current: bool,
    pub(crate) spec: String,
    pub(crate) target: Vec<Inline>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Block {
    /// Jai source: a signature, a declaration, produced code, a `#run` value.
    Code(String),
    /// Text a program printed.
    Output(String),
    /// A line of prose.
    Para(Vec<Inline>),
    /// Prose set apart from the code above it (a blank line before it in plain text).
    Note(Vec<Inline>),
    FormatRows(Vec<FormatRow>),
    /// Starts a new section, such as `expands to` or `prints`.
    Section(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HoverText {
    pub(crate) blocks: Vec<Block>,
}

impl HoverText {
    pub(crate) fn new(blocks: Vec<Block>) -> Self {
        Self {
            blocks,
        }
    }

    pub(crate) fn code(text: impl Into<String>) -> Self {
        Self::new(vec![Block::Code(text.into())])
    }

    /// The sections after the first divider (what a macro produced, say), if any.
    pub(crate) fn sections(&self) -> &[Block] {
        let first = self
            .blocks
            .iter()
            .position(|b| matches!(b, Block::Section(_)));
        first.map_or(&[], |at| &self.blocks[at..])
    }

    pub(crate) fn render(&self, kind: MarkupKind) -> String {
        match kind {
            MarkupKind::PlainText => self.plain(),
            MarkupKind::Markdown => self.markdown(),
        }
    }

    fn plain(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        for block in &self.blocks {
            match block {
                Block::Code(text) | Block::Output(text) => lines.push(text.clone()),
                Block::Para(inline) => lines.push(plain_inline(inline)),
                Block::Note(inline) => lines.push(format!("\n{}", plain_inline(inline))),
                Block::FormatRows(rows) => {
                    let width = rows
                        .iter()
                        .map(|r| r.spec.chars().count())
                        .max()
                        .unwrap_or(1);
                    for row in rows {
                        let marker = if row.current {
                            "▸"
                        } else {
                            " "
                        };
                        lines.push(format!(
                            "{marker} {:<width$} → {}",
                            row.spec,
                            plain_inline(&row.target)
                        ));
                    }
                }
                Block::Section(label) => lines.push(format!("─── {label} ───")),
            }
        }
        lines.join("\n")
    }

    fn markdown(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        for block in &self.blocks {
            match block {
                Block::Code(text) => parts.push(fenced("jai", text)),
                Block::Output(text) => parts.push(fenced("text", text)),
                Block::Para(inline) | Block::Note(inline) => parts.push(markdown_inline(inline)),
                Block::FormatRows(rows) => parts.push(
                    rows.iter()
                        .map(|row| {
                            let spec = code_span(&row.spec);
                            let spec = if row.current {
                                format!("**{spec}**")
                            } else {
                                spec
                            };
                            format!("- {spec} → {}", markdown_inline(&row.target))
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                ),
                Block::Section(label) => parts.push(format!("---\n\n*{}*", escape(label))),
            }
        }
        parts.join("\n\n")
    }
}

fn plain_inline(inline: &[Inline]) -> String {
    inline
        .iter()
        .map(|part| match part {
            Inline::Text(t) | Inline::Code(t) => t.as_str(),
        })
        .collect()
}

fn markdown_inline(inline: &[Inline]) -> String {
    let mut out = String::new();
    for part in inline {
        match part {
            Inline::Text(t) => out.push_str(&escape(t)),
            Inline::Code(t) => out.push_str(&code_span(t)),
        }
    }
    // A paragraph starting with `-`, `+` or `=` could read as a list or a heading underline.
    if out.starts_with(['-', '+', '=']) {
        out.insert(0, '\\');
    }
    out
}

/// Backslash-escapes the characters that could start emphasis, code, links, HTML, tables
/// or strikethrough. Escaping any ASCII punctuation is valid CommonMark, so the rendered
/// text is unchanged.
pub(crate) fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(
            c,
            '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '>' | '&' | '|' | '~' | '#'
        ) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

fn longest_backtick_run(text: &str) -> usize {
    text.split(|c| c != '`').map(str::len).max().unwrap_or(0)
}

/// An inline code span holding `text` verbatim: the fence is longer than any run of
/// backticks inside, and padded when the text starts or ends with one.
pub(crate) fn code_span(text: &str) -> String {
    let text = text.replace('\n', " ");
    let ticks = "`".repeat(longest_backtick_run(&text) + 1);
    let pad = if text.starts_with('`') || text.ends_with('`') {
        " "
    } else {
        ""
    };
    format!("{ticks}{pad}{text}{pad}{ticks}")
}

/// A fenced code block that no line of `text` can close early.
pub(crate) fn fenced(language: &str, text: &str) -> String {
    let ticks = "`".repeat((longest_backtick_run(text) + 1).max(3));
    format!("{ticks}{language}\n{text}\n{ticks}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_and_fences_outlast_backticks_inside() {
        assert_eq!(code_span("x"), "`x`");
        assert_eq!(code_span("`total"), "`` `total ``");
        assert_eq!(code_span("a``b"), "```a``b```");
        assert_eq!(fenced("jai", "`x += 1;"), "```jai\n`x += 1;\n```");
        assert_eq!(
            fenced("jai", "s := \"```\";"),
            "````jai\ns := \"```\";\n````"
        );
    }

    #[test]
    fn prose_is_escaped() {
        assert_eq!(
            escape("a *b* [c](d) <e> _f_"),
            r"a \*b\* \[c\](d) \<e\> \_f\_"
        );
        assert_eq!(markdown_inline(&[Inline::Text("- x".into())]), r"\- x");
    }

    #[test]
    fn sections_render_as_breaks_or_divider_lines() {
        let hover = HoverText::new(vec![
            Block::Code("f :: () #expand".into()),
            Block::Section("expands to".into()),
            Block::Code("x += 1;".into()),
        ]);
        assert_eq!(
            hover.render(MarkupKind::PlainText),
            "f :: () #expand\n─── expands to ───\nx += 1;"
        );
        assert_eq!(
            hover.render(MarkupKind::Markdown),
            "```jai\nf :: () #expand\n```\n\n---\n\n*expands to*\n\n```jai\nx += 1;\n```"
        );
    }
}
