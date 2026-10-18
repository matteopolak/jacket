//! Hovers over expansions, links, directives and casts.
use super::directive_at;
use crate::DocumentUri;
use crate::analysis::{DIRECTIVES, TokenKind};
use crate::hover::{Block, HoverText, Inline};
use crate::session::Session;
use jaic::sema::ide_meta::{IdeExpansion, IdeExpansionKind};
use std::path::Path;

impl Session {
    /// What an expansion produced, for a hover. Produced code comes last, in sections of
    /// its own (`expands to`, `prints`).
    fn describe(&self, uri: &DocumentUri, e: &IdeExpansion) -> HoverText {
        let text = self.document(uri).map(|d| d.text.as_str()).unwrap_or("");
        let produced = |label: &str| -> Vec<Block> {
            let count = e.texts.len();
            e.texts
                .iter()
                .enumerate()
                .flat_map(|(i, t)| {
                    let label = if count == 1 {
                        label.to_string()
                    } else {
                        format!("{label} ({} of {count})", i + 1)
                    };
                    [Block::Section(label), Block::Code(t.clone())]
                })
                .collect()
        };
        let line = |parts: Vec<Inline>| HoverText::new(vec![Block::Para(parts)]);
        let directive_said = |directive: &str, said: &str| {
            line(vec![
                Inline::Code(format!("#{directive}")),
                Inline::Text(said.into()),
            ])
        };
        match e.kind {
            IdeExpansionKind::Insert => {
                if e.texts.iter().all(|t| t.trim().is_empty()) {
                    directive_said("insert", " inserts nothing")
                } else {
                    let mut blocks = vec![Block::Code("#insert".into())];
                    blocks.extend(produced("expands to"));
                    HoverText::new(blocks)
                }
            }
            IdeExpansionKind::Macro => {
                let mut blocks = vec![Block::Code(e.detail.clone())];
                blocks.extend(produced("expands to"));
                HoverText::new(blocks)
            }
            IdeExpansionKind::Run => {
                let mut blocks = if e.texts.iter().all(String::is_empty) {
                    directive_said("run", " returns nothing").blocks
                } else if e.texts.len() > 1 {
                    vec![
                        Block::Para(vec![
                            Inline::Code("#run".into()),
                            Inline::Text(" values (".into()),
                            Inline::Code(e.detail.clone()),
                            Inline::Text("):".into()),
                        ]),
                        Block::Code(e.texts.join("\n")),
                    ]
                } else {
                    vec![Block::Code(format!("#run = {}: {}", e.texts[0], e.detail))]
                };
                if !e.output.is_empty() {
                    blocks.push(Block::Section("prints".into()));
                    blocks.push(Block::Output(e.output.trim_end().into()));
                }
                HoverText::new(blocks)
            }
            IdeExpansionKind::If => {
                let directive = directive_at(text, e.span.start as usize).unwrap_or("if");
                let said = if e.texts.len() > 1 {
                    ": the condition is true for some instances and false for others"
                } else if directive == "assert" {
                    match e.texts[0].as_str() {
                        "true" => ": the condition holds",
                        _ => ": the condition fails",
                    }
                } else {
                    match e.texts[0].as_str() {
                        "true" => ": the condition is true, the first branch is compiled",
                        _ => ": the condition is false, the else branch is compiled",
                    }
                };
                directive_said(directive, said)
            }
        }
    }

    /// The file a `#import` / `#load` string names, as the compiler reads it.
    pub(crate) fn link_hover(
        &self,
        uri: &DocumentUri,
        byte: usize,
    ) -> Option<(usize, usize, HoverText)> {
        let analysis = self.analysis_for(uri)?;
        let link = analysis
            .links
            .iter()
            .find(|l| l.string.start <= byte && byte <= l.string.end)?;
        let doc = self.document(uri).ok()?;
        let target = self.link_target(uri, link)?;
        let shown = doc
            .text
            .get(link.string.start..link.string.end)?
            .to_string();
        let path = Path::new(target.path());
        let shown_path = self
            .workspace_folders
            .iter()
            .find_map(|f| path.strip_prefix(f).ok())
            .unwrap_or(path)
            .display()
            .to_string();
        Some((
            link.string.start,
            link.string.end,
            HoverText::new(vec![
                Block::Code(shown),
                Block::Para(vec![Inline::Code(shown_path)]),
            ]),
        ))
    }

    /// Hover over `#insert`, `#run`, `#if`, `#ifx` or `#assert`: what it produced.
    pub(crate) fn directive_hover(
        &self,
        uri: &DocumentUri,
        byte: usize,
    ) -> Option<(usize, usize, HoverText)> {
        let analysis = self.analysis_for(uri)?;
        let doc = self.document(uri).ok()?;
        let (_, token) = crate::session::tokens_at(&analysis.tokens, byte)
            .find(|(_, t)| t.kind == TokenKind::Directive)?;
        let word = directive_at(&doc.text, token.span.start)?;
        let expansion = matches!(word, "insert" | "run" | "if" | "ifx" | "assert")
            .then(|| {
                self.expansions(uri)
                    .into_iter()
                    .filter(|e| e.span.start as usize == token.span.start)
                    .min_by_key(|e| e.span.end - e.span.start)
            })
            .flatten();
        let text = match expansion {
            Some(e) => self.describe(uri, &e),
            // Any other directive (or one whose expansion was not recorded): what it is.
            None => {
                let (_, about) = DIRECTIVES.iter().find(|(name, _)| *name == word)?;
                HoverText::new(vec![
                    Block::Code(format!("#{word}")),
                    Block::Para(vec![Inline::Text(format!("{about}."))]),
                ])
            }
        };
        Some((token.span.start, token.span.end, text))
    }

    /// Hover over a cast flag: `trunc` of `cast,trunc(u8) x` or `x.(u8, trunc)`.
    pub(crate) fn cast_flag_hover(
        &self,
        text: &str,
        byte: usize,
    ) -> Option<(usize, usize, HoverText)> {
        let word = |c: char| c.is_alphanumeric() || c == '_';
        if !text.is_char_boundary(byte) {
            return None;
        }
        let start = text[..byte]
            .char_indices()
            .rfind(|&(_, c)| !word(c))
            .map_or(0, |(i, c)| i + c.len_utf8());
        let end = text[byte..]
            .find(|c: char| !word(c))
            .map_or(text.len(), |i| byte + i);
        let flag = &text[start..end];
        let about = match flag {
            "trunc" | "truncate" => "keeps the low bits of a value too large for the target type",
            "no_check" => "skips the check that the value fits the target type",
            "force" => "reinterprets the bits as the target type, which has the same size",
            _ => return None,
        };
        let before = text[..start].trim_end().strip_suffix(',')?.trim_end();
        let after = text[end..].trim_start();
        let chained = ["cast", "xx", "trunc", "truncate", "no_check", "force"]
            .iter()
            .any(|w| before.ends_with(w) && !before[..before.len() - w.len()].ends_with(word));
        let line = jaic::source::line_start(text, start);
        let in_dot_cast = after.starts_with([')', ',']) && text[line..start].contains(".(");
        if !chained && !in_dot_cast {
            return None;
        }
        Some((
            start,
            end,
            HoverText::new(vec![
                Block::Code(format!("cast,{flag}")),
                Block::Para(vec![Inline::Text(format!("Cast flag: {about}."))]),
            ]),
        ))
    }

    /// Hover inside an expansion's operand where no name is (an `#insert` string, say).
    pub(crate) fn expansion_hover(
        &self,
        uri: &DocumentUri,
        byte: usize,
    ) -> Option<(usize, usize, HoverText)> {
        let e = self.expansion_at(uri, byte)?;
        Some((
            e.span.start as usize,
            e.span.end as usize,
            self.describe(uri, &e),
        ))
    }

    /// `hover` (of the name at `start`) followed by the expansion of the macro call it starts.
    pub(crate) fn with_macro_expansion(
        &self,
        uri: &DocumentUri,
        start: usize,
        mut hover: HoverText,
    ) -> HoverText {
        if let Some(e) = self
            .expansions(uri)
            .into_iter()
            .find(|e| e.kind == IdeExpansionKind::Macro && e.span.start as usize == start)
        {
            hover
                .blocks
                .extend_from_slice(self.describe(uri, &e).sections());
        }
        hover
    }
}
