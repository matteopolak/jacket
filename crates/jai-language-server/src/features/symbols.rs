//! Syntax-only features: workspace symbols, folding, lenses and polymorphs.
use super::clip;
use crate::analysis::{Span, TokenKind};
use crate::session::Session;
use crate::{
    CodeLens, Command, DocumentUri, Error, FoldingRange, Location, Position, Range,
    SymbolInformation,
};
use jaic::fxhash::HashSet;
use jaic::ide::meta::IdeClass;

impl Session {
    // Syntax-only features

    /// Declarations of every open document, then of the project's other files, whose name
    /// contains `query` (case-insensitive), at most `Limits::symbols` of them.
    pub fn workspace_symbols(&self, query: &str) -> Vec<SymbolInformation> {
        let query = query.to_lowercase();
        let mut out = Vec::new();
        for (uri, analysis) in &self.analyses {
            let analysis = match self.parsed_for(uri) {
                Some(parsed) if !analysis.complete => parsed,
                _ => analysis,
            };
            let doc = &self.documents[uri];
            for row in &analysis.rows {
                if row.local || !row.name.as_str().to_lowercase().contains(&query) {
                    continue;
                }
                if out.len() >= self.limits.symbols {
                    return out;
                }
                let Ok(range) = doc.index.range(&doc.text, row.selection) else {
                    continue;
                };
                out.push(SymbolInformation {
                    name: row.name.as_str().into(),
                    kind: row.kind,
                    location: Location {
                        uri: uri.as_str().into(),
                        range,
                    },
                    container: row
                        .parent
                        .and_then(|p| analysis.rows.get(p))
                        .map(|p| p.name.as_str().into()),
                });
            }
        }
        // The project's files that are not open: their top-level names, from tokens.
        let room = self.limits.symbols.saturating_sub(out.len());
        out.extend(self.project_symbols(&query, room));
        out
    }

    /// Braces spanning lines, and runs of `#import`/`#load` lines.
    pub fn folding_ranges(&self, uri: &DocumentUri) -> Result<Vec<FoldingRange>, Error> {
        let doc = self.document(uri)?;
        let text = &doc.text;
        let analysis = &self
            .analysis_for(uri)
            .expect("opened document has analysis");
        let line = |byte: usize| doc.index.position(text, byte).map(|p| p.line);
        let mut out = Vec::new();
        let mut open = Vec::new();
        for token in &analysis.tokens {
            match token.spelling(text) {
                "{" | ".{" | "(" | "[" | ".[" => open.push(token.span.start),
                "}" | ")" | "]" => {
                    if let Some(start) = open.pop() {
                        let (a, b) = (line(start)?, line(token.span.start)?);
                        if b > a + 1 || (b > a && token.spelling(text) == "}") {
                            out.push(FoldingRange {
                                start_line: a,
                                end_line: b - 1,
                                imports: false,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        let mut run: Option<(u32, u32)> = None;
        for token in &analysis.tokens {
            if token.kind != TokenKind::Directive
                || !matches!(token.spelling(text), "#import" | "#load")
            {
                continue;
            }
            let l = line(token.span.start)?;
            run = match run {
                Some((a, b)) if l == b + 1 || l == b => Some((a, l)),
                Some((a, b)) => {
                    if b > a {
                        out.push(FoldingRange {
                            start_line: a,
                            end_line: b,
                            imports: true,
                        });
                    }
                    Some((l, l))
                }
                None => Some((l, l)),
            };
        }
        if let Some((a, b)) = run
            && b > a
        {
            out.push(FoldingRange {
                start_line: a,
                end_line: b,
                imports: true,
            });
        }
        out.sort_by_key(|r| (r.start_line, r.end_line));
        out.dedup_by_key(|r| r.start_line);
        Ok(out)
    }

    /// Each polymorphic procedure of the document: its name's range and its instances.
    fn polymorph_rows(&self, uri: &DocumentUri) -> Vec<(Range, Vec<String>)> {
        let Ok(doc) = self.document(uri) else {
            return Vec::new();
        };
        let text = &doc.text;
        let polys = self
            .checked(uri, |a, f| a.compiler.ide_polymorphs(f))
            .unwrap_or_default();
        let mut out = Vec::new();
        for (span, name, instances) in polys {
            let start = (span.start as usize).min(text.len());
            // The name is earlier on the line: `name :: (x: $T)`.
            let line_start = jaic::source::line_start(text, start);
            let at = text[line_start..start]
                .find(name.as_str())
                .map_or(start, |i| line_start + i);
            if let Some(range) = self.range_of(uri, Span::new(at, at + name.len())) {
                out.push((range, instances));
            }
        }
        out
    }

    /// Over each polymorphic procedure: how many instances checking created, and of what.
    pub fn code_lenses(&self, uri: &DocumentUri) -> Result<Vec<CodeLens>, Error> {
        self.document(uri)?;
        Ok(self
            .polymorph_rows(uri)
            .into_iter()
            .map(|(range, instances)| {
                let title = match instances.len() {
                    0 => "no polymorphs yet".to_string(),
                    1 => format!("1 polymorph: {}", clip(&instances[0], 60)),
                    n => format!("{n} polymorphs: {}", clip(&instances.join("; "), 80)),
                };
                CodeLens {
                    range,
                    command: Command {
                        title,
                        command: "jai.showPolymorphs".into(),
                        target: Some((uri.as_str().into(), range.start)),
                    },
                }
            })
            .collect())
    }

    /// Instances of the polymorphic procedure named at `position` (for `jai.showPolymorphs`).
    pub fn polymorphs(&self, uri: &DocumentUri, position: Position) -> Vec<String> {
        self.polymorph_rows(uri)
            .into_iter()
            .find(|(range, _)| range.start <= position && position <= range.end)
            .map(|(_, i)| i)
            .unwrap_or_default()
    }

    /// Classes of the identifiers the check recorded in the document, by span.
    pub(crate) fn identifier_classes(&self, uri: &DocumentUri) -> Vec<(Span, IdeClass)> {
        let classes: Vec<(Span, IdeClass)> = self
            .checked(uri, |a, f| a.compiler.ide_classes(f))
            .unwrap_or_default()
            .into_iter()
            .map(|(s, c)| (Span::new(s.start as usize, s.end as usize), c))
            .collect();
        // A callee the check recorded nothing for: its call is still being typed. Asking for the
        // call's target here, not leaving it to a signature help or completion request that may
        // never come, keeps the color from depending on which requests were made.
        let (Ok(doc), Some(analysis)) = (self.document(uri), self.analysis_for(uri)) else {
            return classes;
        };
        let text = doc.text.as_str();
        let known: HashSet<usize> = classes.iter().map(|(s, _)| s.start).collect();
        let callees: Vec<(Span, u32)> = analysis
            .tokens
            .windows(2)
            .filter(|w| {
                w[0].kind == TokenKind::Ident
                    && w[1].kind == TokenKind::Punctuation
                    && w[1].spelling(text) == "("
                    && !known.contains(&w[0].span.start)
            })
            .map(|w| (w[0].span, w[1].span.start as u32))
            .collect();
        if callees.is_empty() {
            return classes;
        }
        let found = self.checked(uri, |a, f| {
            callees
                .iter()
                .filter_map(|&(span, open)| Some((span, a.compiler.ide_callee_class(f, open)?)))
                .collect::<Vec<_>>()
        });
        let found = found.unwrap_or_default();
        // Resolving a callee checks the code around it and records what it found, even when it
        // is no procedure (`Shape(1, ` names a type; a top-level `#run` body is checked this
        // way too), so the classes are asked for again.
        let mut classes: Vec<(Span, IdeClass)> = self
            .checked(uri, |a, f| a.compiler.ide_classes(f))
            .unwrap_or_default()
            .into_iter()
            .map(|(s, c)| (Span::new(s.start as usize, s.end as usize), c))
            .collect();
        let known: HashSet<usize> = classes.iter().map(|(s, _)| s.start).collect();
        classes.extend(found.into_iter().filter(|(s, _)| !known.contains(&s.start)));
        classes
    }
}
