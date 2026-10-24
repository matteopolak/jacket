//! Format-call hovers and expansion documents.
use super::{EXPANSION_SCHEME, clip, kind_name, span_of};
use crate::analysis::Span;
use crate::hover::{Block, FormatRow, HoverText, Inline};
use crate::session::{Session, contains};
use crate::{DocumentUri, Error, Expansion, Location, Position};
use jaic::ide::meta::IdeExpansionKind;
use std::path::Path;

impl Session {
    /// The calls of the document with a literal format string whose callee is declared
    /// `@PrintLike`: from the compile's resolved callee, else (no compile available) from this
    /// document's own `@PrintLike` declarations.
    pub(crate) fn format_calls(&self, uri: &DocumentUri) -> Vec<crate::format::FormatCall> {
        let Some(analysis) = self.analysis_for(uri) else {
            return Vec::new();
        };
        if analysis.format_calls.is_empty() {
            return Vec::new();
        }
        if self.environment.is_none() {
            let local = &analysis.print_like;
            return analysis
                .format_calls
                .iter()
                .filter(|c| local.contains(&c.callee))
                .cloned()
                .collect();
        }
        let semantic = self.calls(uri);
        analysis
            .format_calls
            .iter()
            .filter(|call| {
                semantic
                    .iter()
                    .filter(|c| {
                        c.span.start as usize <= call.string.start
                            && call.string.end <= c.span.end as usize
                    })
                    .min_by_key(|c| c.span.end - c.span.start)
                    .is_some_and(|info| {
                        info.format_param.is_some_and(|param| {
                            info.args.iter().any(|a| {
                                a.span.start as usize == call.string.start
                                    && !a.named
                                    && a.param == param
                            })
                        })
                    })
            })
            .cloned()
            .collect()
    }

    /// Hover over the format string of a `@PrintLike` call: each `%` with the argument it
    /// formats and that argument's type, the one under the cursor marked.
    pub(crate) fn format_hover(
        &self,
        uri: &DocumentUri,
        byte: usize,
    ) -> Option<(usize, usize, HoverText)> {
        let doc = self.document(uri).ok()?;
        let text = &doc.text;
        let calls = self.format_calls(uri);
        let call = calls
            .iter()
            .find(|c| c.string.start <= byte && byte < c.string.end)?;
        let semantic = self
            .calls(uri)
            .into_iter()
            .filter(|c| {
                c.span.start as usize <= call.string.start && call.string.end <= c.span.end as usize
            })
            .min_by_key(|c| c.span.end - c.span.start);
        let type_of = |arg: Span| -> Option<String> {
            semantic
                .as_ref()?
                .args
                .iter()
                .find(|a| a.span.start as usize == arg.start)?
                .ty
                .clone()
        };
        let mut blocks = vec![Block::Code(clip(
            &text[call.string.start..call.string.end],
            120,
        ))];
        if call.specs.is_empty() {
            blocks.push(Block::Para(vec![Inline::Text(
                "No format arguments.".into(),
            )]));
        }
        let rows = call
            .specs
            .iter()
            .map(|spec| {
                let target = match spec.index {
                    None => Inline::Text("prints nothing".into()),
                    Some(i) => match call.args.get(i) {
                        Some(arg) => {
                            let source = clip(&text[arg.start..arg.end], 60);
                            Inline::Code(match type_of(*arg) {
                                Some(ty) => format!("{source}: {ty}"),
                                None => source,
                            })
                        }
                        None if call.spread => {
                            Inline::Text(format!("argument {} of the spread", i + 1))
                        }
                        None => Inline::Text(format!("missing argument {}", i + 1)),
                    },
                };
                FormatRow {
                    current: contains(spec.span, byte) && call.specs.len() > 1,
                    spec: text[spec.span.start..spec.span.end].to_string(),
                    target: vec![target],
                }
            })
            .collect::<Vec<_>>();
        if !rows.is_empty() {
            blocks.push(Block::FormatRows(rows));
        }
        Some((call.string.start, call.string.end, HoverText::new(blocks)))
    }

    // Expansion documents and code actions

    fn expansion_uri(&self, uri: &DocumentUri, at: Position) -> String {
        format!(
            "{EXPANSION_SCHEME}{}?{}:{}",
            uri.as_str().trim_start_matches("file://"),
            at.line,
            at.character
        )
    }

    /// The generated code of the `#insert`, `#run`, `#if` or macro call at `position`.
    pub fn expansion(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Option<Expansion>, Error> {
        let doc = self.document(uri)?;
        let byte = doc.index.byte(&doc.text, position);
        let Some(e) = self.expansion_at(uri, byte) else {
            return Ok(None);
        };
        let Some(range) = self.range_of(uri, span_of(&e)) else {
            return Ok(None);
        };
        let name = Path::new(uri.path())
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        let place = format!(
            "{name}:{}:{}",
            range.start.line + 1,
            range.start.character + 1
        );
        let what = match e.kind {
            IdeExpansionKind::Macro => format!("the call of {} at {place}", e.detail),
            kind => format!("the #{} at {place}", kind_name(kind)),
        };
        let body = match e.kind {
            IdeExpansionKind::Run => {
                let mut out = format!("// Value of {what} ({}):\n", e.detail);
                out.push_str(&e.texts.join("\n"));
                if !e.output.is_empty() {
                    out.push_str("\n\n// Printed at compile time:\n");
                    for line in e.output.trim_end().lines() {
                        out.push_str("// ");
                        out.push_str(line);
                        out.push('\n');
                    }
                }
                out
            }
            IdeExpansionKind::If => format!(
                "// The condition of {what} is {}.\n",
                e.texts.join(" for some instances and ")
            ),
            _ => {
                let mut out = format!("// Expansion of {what}\n");
                for (i, t) in e.texts.iter().enumerate() {
                    if e.texts.len() > 1 {
                        out.push_str(&format!("\n// Instance {}:\n", i + 1));
                    }
                    out.push_str(t);
                    out.push('\n');
                }
                out
            }
        };
        Ok(Some(Expansion {
            uri: self.expansion_uri(uri, range.start),
            source: Location {
                uri: uri.as_str().into(),
                range,
            },
            kind: kind_name(e.kind),
            text: body,
        }))
    }

    /// The text of a `jai-expansion:` document.
    pub fn expansion_source(&self, uri: &str) -> Option<String> {
        let rest = uri.strip_prefix(EXPANSION_SCHEME)?;
        let (path, at) = rest.rsplit_once('?')?;
        let (line, character) = at.split_once(':')?;
        let document = DocumentUri::parse(&format!("file://{path}")).ok()?;
        let position = Position {
            line: line.parse().ok()?,
            character: character.parse().ok()?,
        };
        Some(self.expansion(&document, position).ok()??.text)
    }
}
