//! Editor features built on the compiler's metaprogramming and call facts
//! (`jaic::sema::ide_meta`) and on the syntax layer: expansion hovers and documents, inlay
//! hints, code actions, format strings, references, signature help, folding, code lenses and
//! workspace symbols.
use crate::analysis::{Span, TokenKind};
use crate::hover::{Block, FormatRow, HoverText, Inline};
use crate::links::{Link, LinkSource};
use crate::semantic;
use crate::session::{Session, contains, repair};
use crate::{
    CodeAction, CodeLens, Command, DocumentUri, Error, Expansion, FoldingRange, InlayHint,
    InlayHintKind, Location, Position, Range, SignatureHelp, SignatureInformation,
    SymbolInformation, TextEdit,
};
use jaic::intern::Sym;
use jaic::sema::FileSystem;
use jaic::sema::ide_meta::{IdeCallInfo, IdeClass, IdeExpansion, IdeExpansionKind};
use jaic::source::FileId;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// URI scheme of expansion documents.
pub const EXPANSION_SCHEME: &str = "jai-expansion://";

/// Longest inlay hint label.
const HINT_CHARS: usize = 40;

fn kind_name(kind: IdeExpansionKind) -> &'static str {
    match kind {
        IdeExpansionKind::Insert => "insert",
        IdeExpansionKind::Run => "run",
        IdeExpansionKind::If => "if",
        IdeExpansionKind::Macro => "macro",
    }
}

fn span_of(e: &IdeExpansion) -> Span {
    Span::new(e.span.start as usize, e.span.end as usize)
}

/// The directive word at `at` of `text` (`insert` for `#insert`), if one starts there.
fn directive_at(text: &str, at: usize) -> Option<&str> {
    let rest = text.get(at..)?.strip_prefix('#')?;
    let end = rest
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    Some(&rest[..end])
}

fn clip(text: &str, chars: usize) -> String {
    let one_line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if one_line.chars().count() > chars {
        let cut: String = one_line.chars().take(chars).collect();
        format!("{cut}…")
    } else {
        one_line
    }
}

/// Rename edits grouped by document URI.
pub type RenameEdits = Vec<(String, Vec<TextEdit>)>;

impl Session {
    /// Run `query` on the type-checked program, the document's text repaired to parse.
    pub(crate) fn checked<T>(
        &self,
        uri: &DocumentUri,
        query: impl FnOnce(&mut semantic::Analysis, FileId) -> T,
    ) -> Option<T> {
        let doc = self.document(uri).ok()?;
        let probe = repair(&doc.text, None)?;
        self.with_semantic(uri, &probe, |a, path| {
            let file = a.file(path)?;
            Some(query(a, file))
        })
    }

    /// `span` of an open document as a range, if it lies on character boundaries.
    pub(crate) fn range_of(&self, uri: &DocumentUri, span: Span) -> Option<Range> {
        let doc = self.document(uri).ok()?;
        if span.end > doc.text.len() || span.start > span.end {
            return None;
        }
        doc.index.range(&doc.text, span).ok()
    }

    /// A location in any compiled file (an open document's own index when it is open).
    fn location_in(
        &self,
        path: &str,
        text: &Rc<str>,
        start: usize,
        end: usize,
    ) -> Option<Location> {
        let target = DocumentUri::parse(&format!("file://{path}")).ok()?;
        let span = Span::new(start, end);
        let range = match self.documents.get(&target) {
            Some(open) if *open.text == **text => open.index.range(&open.text, span).ok()?,
            _ => crate::position::LineIndex::new(text)
                .range(text, span)
                .ok()?,
        };
        Some(Location {
            uri: target.as_str().into(),
            range,
        })
    }

    fn expansions(&self, uri: &DocumentUri) -> Vec<IdeExpansion> {
        self.checked(uri, |a, f| a.compiler.ide_expansions(f))
            .unwrap_or_default()
    }

    fn calls(&self, uri: &DocumentUri) -> Vec<IdeCallInfo> {
        self.checked(uri, |a, f| a.compiler.ide_calls(f))
            .unwrap_or_default()
    }

    /// The expansion a cursor at `byte` is on: the smallest one containing it.
    fn expansion_at(&self, uri: &DocumentUri, byte: usize) -> Option<IdeExpansion> {
        self.expansions(uri)
            .into_iter()
            .filter(|e| contains(span_of(e), byte))
            .min_by_key(|e| e.span.end - e.span.start)
    }

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

    /// Hover over `#insert`, `#run`, `#if`, `#ifx` or `#assert`: what it produced.
    pub(crate) fn directive_hover(
        &self,
        uri: &DocumentUri,
        byte: usize,
    ) -> Option<(usize, usize, HoverText)> {
        let analysis = self.analyses.get(uri)?;
        let doc = self.document(uri).ok()?;
        let token = analysis.tokens.iter().find(|t| {
            t.kind == TokenKind::Directive && t.span.start <= byte && byte <= t.span.end
        })?;
        let word = directive_at(&doc.text, token.span.start)?;
        if !matches!(word, "insert" | "run" | "if" | "ifx" | "assert") {
            return None;
        }
        let e = self
            .expansions(uri)
            .into_iter()
            .filter(|e| e.span.start as usize == token.span.start)
            .min_by_key(|e| e.span.end - e.span.start)?;
        Some((token.span.start, token.span.end, self.describe(uri, &e)))
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

    /// Hover over the format string of a print-family call: each `%` with the argument it
    /// formats and that argument's type, the one under the cursor marked.
    pub(crate) fn format_hover(
        &self,
        uri: &DocumentUri,
        byte: usize,
    ) -> Option<(usize, usize, HoverText)> {
        let doc = self.document(uri).ok()?;
        let text = &doc.text;
        let call = self
            .analyses
            .get(uri)?
            .format_calls
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

    // ---------------------------------------------------------------------------------------
    // Expansion documents and code actions
    // ---------------------------------------------------------------------------------------

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
        let byte = doc.index.byte(&doc.text, position)?;
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

    /// Show an expansion; inline an `#insert` or a `#run` value; add an `#import` an unknown name
    /// needs; apply a lint's fix, or all.
    pub fn code_actions(&self, uri: &DocumentUri, range: Range) -> Result<Vec<CodeAction>, Error> {
        self.code_actions_in(uri, range, &crate::lints::ActionContext::default())
    }

    /// [`Session::code_actions`] limited to what `context` asks for.
    pub fn code_actions_in(
        &self,
        uri: &DocumentUri,
        range: Range,
        context: &crate::lints::ActionContext,
    ) -> Result<Vec<CodeAction>, Error> {
        let mut actions = self.all_code_actions(uri, range, context)?;
        actions.retain(|a| match a.kind {
            Some(kind) => context.wants(kind),
            None => context.only.is_none(),
        });
        Ok(actions)
    }

    fn all_code_actions(
        &self,
        uri: &DocumentUri,
        range: Range,
        context: &crate::lints::ActionContext,
    ) -> Result<Vec<CodeAction>, Error> {
        let doc = self.document(uri)?;
        let start = doc.index.byte(&doc.text, range.start)?;
        let end = doc.index.byte(&doc.text, range.end)?;
        let text = &doc.text;
        let mut fixes = self.import_actions(uri, start, end, context);
        fixes.extend(self.lint_actions(uri, start, end, context));
        if context
            .only
            .as_ref()
            .is_none_or(|only| only.iter().any(|k| k.starts_with("refactor")))
        {
            fixes.extend(self.refactor_actions(uri, start, end));
        }
        let Some(e) = self
            .expansions(uri)
            .into_iter()
            .filter(|e| {
                let s = span_of(e);
                contains(s, start) || (start <= s.start && s.end <= end && start < end)
            })
            .min_by_key(|e| e.span.end - e.span.start)
        else {
            return Ok(fixes);
        };
        let span = span_of(&e);
        let Some(at) = self.range_of(uri, span) else {
            return Ok(fixes);
        };
        let mut actions = Vec::new();
        if e.kind != IdeExpansionKind::If {
            let what = match e.kind {
                IdeExpansionKind::Macro => format!("Show expansion of {}", e.detail),
                IdeExpansionKind::Run => "Show #run result".into(),
                _ => "Show #insert expansion".into(),
            };
            actions.push(CodeAction {
                title: what.clone(),
                kind: None,
                edit: None,
                command: Some(Command {
                    title: what,
                    command: "jai.showExpansion".into(),
                    target: Some((uri.as_str().into(), at.start)),
                }),
                ..CodeAction::default()
            });
        }
        let single = e.texts.len() == 1;
        let edit = match e.kind {
            IdeExpansionKind::Insert if single => {
                inline_insert(text, span, &e.texts[0]).map(|(s, t)| ("Inline #insert", s, t))
            }
            IdeExpansionKind::Run if single => e
                .literal
                .clone()
                .filter(|_| e.output.is_empty())
                .map(|lit| ("Replace #run with its value", span, lit)),
            _ => None,
        };
        if let Some((title, span, new_text)) = edit
            && let Some(range) = self.range_of(uri, span)
        {
            actions.push(CodeAction {
                title: title.into(),
                kind: Some("refactor.inline"),
                edit: Some((
                    uri.as_str().into(),
                    vec![TextEdit {
                        range,
                        new_text,
                    }],
                )),
                command: None,
                ..CodeAction::default()
            });
        }
        actions.extend(fixes);
        Ok(actions)
    }

    // ---------------------------------------------------------------------------------------
    // Inlay hints
    // ---------------------------------------------------------------------------------------

    /// Inferred types of `x := value`, parameter names of literal arguments, and `#run` values.
    pub fn inlay_hints(&self, uri: &DocumentUri, range: Range) -> Result<Vec<InlayHint>, Error> {
        let doc = self.document(uri)?;
        let text = &doc.text;
        let first = doc.index.byte(text, range.start)?;
        let last = doc.index.byte(text, range.end)?;
        let Some((types, calls, expansions)) = self.checked(uri, |a, f| {
            (
                a.compiler.ide_declared_types(f),
                a.compiler.ide_calls(f),
                a.compiler.ide_expansions(f),
            )
        }) else {
            return Ok(Vec::new());
        };
        let mut hints: Vec<(usize, InlayHint)> = Vec::new();
        let mut hint = |at: usize, label: String, kind, left, right, tooltip| {
            if at < first || at > last || at > text.len() || !text.is_char_boundary(at) {
                return;
            }
            if let Ok(position) = doc.index.position(text, at) {
                hints.push((
                    at,
                    InlayHint {
                        position,
                        label,
                        kind,
                        padding_left: left,
                        padding_right: right,
                        tooltip,
                    },
                ));
            }
        };
        for (span, ty) in types {
            let end = span.end as usize;
            if end > text.len() || !inferred_declaration(text, end) {
                continue;
            }
            // `t := Thing.{...}` and `p := cast(*u8) q` already say the type.
            let value = text[end..]
                .trim_start()
                .trim_start_matches([',', ':', '='])
                .trim_start();
            let value = value.split(['\n', ';']).next().unwrap_or("");
            if value.starts_with(ty.as_str()) || value.contains(&format!("cast({ty})")) {
                continue;
            }
            hint(
                end,
                format!(": {}", clip(&ty, HINT_CHARS)),
                Some(InlayHintKind::Type),
                false,
                false,
                None,
            );
        }
        for call in &calls {
            let signature = &call.signatures[call.active];
            let ambiguous = ambiguous_params(&signature.params);
            for arg in &call.args {
                let (s, e) = (arg.span.start as usize, arg.span.end as usize);
                if arg.named
                    || arg.variadic
                    || arg.spread
                    || arg.param_name.is_empty()
                    || e > text.len()
                {
                    continue;
                }
                let source = &text[s..e];
                if !literal(source)
                    || source == arg.param_name
                    || !ambiguous.get(arg.param).copied().unwrap_or(false)
                {
                    continue;
                }
                let tooltip = signature.params.get(arg.param).cloned();
                hint(
                    s,
                    format!("{}:", arg.param_name),
                    Some(InlayHintKind::Parameter),
                    false,
                    true,
                    tooltip,
                );
            }
        }
        for e in &expansions {
            if e.kind != IdeExpansionKind::Run || e.texts.len() != 1 || e.texts[0].is_empty() {
                continue;
            }
            let span = span_of(e);
            if span.end > text.len() {
                continue;
            }
            // `#run 3` needs no hint.
            let operand = text[span.start..span.end].trim_start_matches("#run").trim();
            if operand == e.texts[0] {
                continue;
            }
            hint(
                span.end,
                format!("= {}", clip(&e.texts[0], HINT_CHARS)),
                None,
                true,
                false,
                Some(format!("computed at compile time ({})", e.detail)),
            );
        }
        hints.sort_by_key(|(at, _)| *at);
        Ok(hints.into_iter().map(|(_, h)| h).collect())
    }

    // ---------------------------------------------------------------------------------------
    // Navigation
    // ---------------------------------------------------------------------------------------

    /// Every use of what the name at `position` names, in the files the check recorded.
    pub fn references(
        &self,
        uri: &DocumentUri,
        position: Position,
        declaration: bool,
    ) -> Result<Vec<Location>, Error> {
        let doc = self.document(uri)?;
        let byte = doc.index.byte(&doc.text, position)?;
        let found = self
            .checked(uri, |a, f| {
                a.compiler
                    .ide_references(f, byte as u32)
                    .into_iter()
                    .filter(|(_, decl)| declaration || !decl)
                    .filter_map(|(span, _)| a.location(span))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Ok(found
            .into_iter()
            .filter_map(|(path, text, start, end)| self.location_in(&path, &text, start, end))
            .collect())
    }

    /// The name at `position`, when it can be renamed: the check recorded what it names (a
    /// struct field or enum member only when its declaration is in the project's files).
    pub fn prepare_rename(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Option<Range>, Error> {
        let doc = self.document(uri)?;
        let Some((_, token)) = self.word(uri, position)? else {
            return Ok(None);
        };
        if token.kind != TokenKind::Ident {
            return Ok(None);
        }
        let range = doc.index.range(&doc.text, token.span)?;
        let byte = doc.index.byte(&doc.text, position)?;
        let outside = self
            .checked(uri, |a, f| a.compiler.ide_member_declared(f, byte as u32))
            .flatten()
            == Some(false);
        if outside {
            return Ok(None);
        }
        let found = self.references(uri, position, true)?;
        Ok(found
            .iter()
            .any(|l| l.uri == uri.as_str() && l.range == range)
            .then_some(range))
    }

    /// Edits renaming what the name at `position` names, in every file the check recorded,
    /// grouped by document; `None` when it cannot be renamed.
    pub fn rename(
        &self,
        uri: &DocumentUri,
        position: Position,
        new_name: &str,
    ) -> Result<Option<RenameEdits>, Error> {
        let valid = new_name
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
            && new_name.chars().all(|c| c.is_alphanumeric() || c == '_');
        if !valid || crate::analysis::KEYWORDS.contains(&new_name) {
            return Err(Error::InvalidEdit("the new name is not an identifier"));
        }
        let Some(range) = self.prepare_rename(uri, position)? else {
            return Ok(None);
        };
        let doc = self.document(uri)?;
        let old = {
            let start = doc.index.byte(&doc.text, range.start)?;
            let end = doc.index.byte(&doc.text, range.end)?;
            doc.text[start..end].to_string()
        };
        let mut edits: Vec<(String, Vec<TextEdit>)> = Vec::new();
        for location in self.references(uri, position, true)? {
            // In an open document, edit only spans that spell the old name.
            if let Ok(target) = DocumentUri::parse(&location.uri)
                && let Some(open) = self.documents.get(&target)
            {
                let start = open.index.byte(&open.text, location.range.start)?;
                let end = open.index.byte(&open.text, location.range.end)?;
                if open.text.get(start..end) != Some(old.as_str()) {
                    continue;
                }
            }
            let edit = TextEdit {
                range: location.range,
                new_text: new_name.into(),
            };
            match edits.iter_mut().find(|(u, _)| *u == location.uri) {
                Some((_, list)) => list.push(edit),
                None => edits.push((location.uri, vec![edit])),
            }
        }
        Ok(Some(edits))
    }

    /// References within the document itself.
    pub fn document_highlights(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Vec<Range>, Error> {
        Ok(self
            .references(uri, position, true)?
            .into_iter()
            .filter(|l| l.uri == uri.as_str())
            .map(|l| l.range)
            .collect())
    }

    /// Where the type of the expression at `position` is declared.
    pub fn type_definition(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Vec<Location>, Error> {
        let doc = self.document(uri)?;
        let byte = doc.index.byte(&doc.text, position)?;
        let found = self
            .checked(uri, |a, f| {
                a.compiler
                    .ide_type_definition(f, byte as u32)
                    .into_iter()
                    .filter_map(|span| a.location(span))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Ok(found
            .into_iter()
            .filter_map(|(path, text, start, end)| {
                // Narrow `Name :: struct {...}` to the name when the span starts there.
                let line_start = text[..start].rfind('\n').map_or(0, |n| n + 1);
                let before = &text[line_start..start];
                let name = before.split("::").next().unwrap_or("").trim();
                if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    let at = line_start + before.find(name).unwrap_or(0);
                    return self.location_in(&path, &text, at, at + name.len());
                }
                self.location_in(&path, &text, start, end.min(start + 1).max(start))
            })
            .collect())
    }

    /// The signature of the call being typed at `position`, the chosen overload active.
    pub fn signature_help(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Option<SignatureHelp>, Error> {
        let doc = self.document(uri)?;
        let text = &doc.text;
        let byte = doc.index.byte(text, position)?;
        if let Some(help) = self.asm_signature_help(text, byte) {
            return Ok(Some(help));
        }
        let Some((open, chain, active_arg, named)) = open_call(text, byte) else {
            return Ok(None);
        };
        // Each name is followed by a `.` or the `(`; add the 1 first, since a callee at the start of
        // the document begins at byte 0.
        let callee_start = open + 1 - chain.iter().map(|c| c.len() + 1).sum::<usize>();
        // A call the check recorded at this position: its overloads, the chosen one active.
        let recorded = self
            .calls(uri)
            .into_iter()
            .find(|c| c.span.start as usize == callee_start && open < c.span.end as usize);
        let (signatures, active) = match recorded {
            Some(call) => (call.signatures, call.active),
            None => {
                // The unfinished call's line is blanked; the callee is looked up in the scope.
                let Some(probe) = repair(text, Some(byte)) else {
                    return Ok(None);
                };
                let names: Vec<Sym> = chain.iter().map(|n| Sym::intern(n)).collect();
                let found = self.with_semantic(uri, &probe, |a, path| {
                    let file = a.file(path)?;
                    let scope = a.compiler.ide_scope_at(file, callee_start as u32)?;
                    let procs = a.compiler.ide_callee(scope, &names);
                    Some(
                        procs
                            .into_iter()
                            .map(|p| a.compiler.ide_signature(p))
                            .collect::<Vec<_>>(),
                    )
                });
                match found {
                    Some(s) if !s.is_empty() => (s, 0),
                    _ => return Ok(None),
                }
            }
        };
        let parameter = |sig: &jaic::sema::ide_meta::IdeSignature| -> usize {
            if let Some(name) = &named
                && let Some(i) = sig.param_names.iter().position(|n| n == name)
            {
                return i;
            }
            let last = sig.params.len().saturating_sub(1);
            active_arg.min(last)
        };
        let active_parameter = signatures.get(active).map_or(0, parameter);
        Ok(Some(SignatureHelp {
            signatures: signatures
                .into_iter()
                .map(|s| SignatureInformation {
                    label: s.label,
                    parameters: s.params,
                })
                .collect(),
            active_signature: active,
            active_parameter,
        }))
    }

    // ---------------------------------------------------------------------------------------
    // `#load` / `#import` links
    // ---------------------------------------------------------------------------------------

    /// The file a `#load` or `#import` of `uri` brings in, resolved as the compiler does:
    /// `#load` relative to the loading file; a module in `modules/` next to the file, then on
    /// the import path (`Name.jai` before `Name/module.jai`); `,file` and `,dir` relative to
    /// the file. Open documents count as files.
    fn link_target(&self, uri: &DocumentUri, link: &Link) -> Option<DocumentUri> {
        let files = OpenFiles {
            session: self,
        };
        let path = match &link.source {
            LinkSource::Load(path) => PathBuf::from(uri.load(path).ok()?.path()),
            LinkSource::Import(source) => {
                let root = PathBuf::from(self.root(uri).path());
                let import_paths = self
                    .environment
                    .as_ref()
                    .map(|e| (e.options)(&root).import_paths)
                    .unwrap_or_default();
                let dir = Path::new(uri.path()).parent()?;
                jaic::sema::import_entry(&files, &import_paths, source, dir)?
            }
        };
        if !files.is_file(&path) {
            return None;
        }
        let path = self
            .environment
            .as_ref()
            .map_or(path.clone(), |e| e.fs.canonical(&path));
        DocumentUri::parse(&format!("file://{}", path.to_string_lossy())).ok()
    }

    /// Each `#load` / `#import` string of the document with the file it brings in.
    pub fn document_links(&self, uri: &DocumentUri) -> Result<Vec<(Range, String)>, Error> {
        self.document(uri)?;
        let links = self.analyses[uri].links.clone();
        Ok(links
            .iter()
            .filter_map(|link| {
                let target = self.link_target(uri, link)?;
                Some((
                    self.range_of(uri, link.string)?,
                    target.as_str().to_string(),
                ))
            })
            .collect())
    }

    /// Definition of a `#load` / `#import` directive or its string: the start of the file.
    pub(crate) fn link_definition(&self, uri: &DocumentUri, byte: usize) -> Option<Location> {
        let link = self
            .analyses
            .get(uri)?
            .links
            .iter()
            .find(|l| l.directive.start <= byte && byte <= l.directive.end)?;
        let target = self.link_target(uri, link)?;
        Some(Location {
            uri: target.as_str().into(),
            range: Range::default(),
        })
    }

    // ---------------------------------------------------------------------------------------
    // Syntax-only features
    // ---------------------------------------------------------------------------------------

    /// Declarations of every open document whose name contains `query` (case-insensitive).
    pub fn workspace_symbols(&self, query: &str) -> Vec<SymbolInformation> {
        let query = query.to_lowercase();
        let mut out = Vec::new();
        for (uri, analysis) in &self.analyses {
            let analysis = match self.parsed.get(uri) {
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
        out
    }

    /// Braces spanning lines, and runs of `#import`/`#load` lines.
    pub fn folding_ranges(&self, uri: &DocumentUri) -> Result<Vec<FoldingRange>, Error> {
        let doc = self.document(uri)?;
        let text = &doc.text;
        let analysis = &self.analyses[uri];
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
            let line_start = text[..start].rfind('\n').map_or(0, |n| n + 1);
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
        self.checked(uri, |a, f| a.compiler.ide_classes(f))
            .unwrap_or_default()
            .into_iter()
            .map(|(s, c)| (Span::new(s.start as usize, s.end as usize), c))
            .collect()
    }
}

/// The open documents over the environment's file system, for resolving links.
struct OpenFiles<'a> {
    session: &'a Session,
}

impl OpenFiles<'_> {
    fn open(&self, path: &Path) -> bool {
        self.session
            .documents
            .keys()
            .any(|d| Path::new(d.path()) == path)
    }
}

impl jaic::sema::FileSystem for OpenFiles<'_> {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        self.session.environment.as_ref()?.fs.read(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.open(path)
            || self
                .session
                .environment
                .as_ref()
                .is_some_and(|e| e.fs.is_file(path))
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.session
            .environment
            .as_ref()
            .is_some_and(|e| e.fs.is_dir(path))
    }
}

/// After a declared name ending at `end`: `:=`, or `, other :=` of a multiple declaration.
fn inferred_declaration(text: &str, end: usize) -> bool {
    let rest = text[end..].trim_start_matches([' ', '\t']);
    if rest.starts_with(":=") {
        return true;
    }
    if let Some(more) = rest.strip_prefix(',') {
        let line = more.split(['\n', ';']).next().unwrap_or("");
        if let Some(at) = line.find(":=") {
            return line[..at]
                .split(',')
                .all(|n| n.trim().chars().all(|c| c.is_alphanumeric() || c == '_'));
        }
    }
    false
}

/// A literal argument (a parameter name hint helps there).
fn literal(source: &str) -> bool {
    let s = source.trim_start_matches('-');
    s.starts_with(|c: char| c.is_ascii_digit())
        || s.starts_with('"')
        || matches!(s, "true" | "false" | "null")
        || s.starts_with("#char")
        || (s.starts_with('.') && s[1..].chars().all(|c| c.is_alphanumeric() || c == '_'))
}

/// `#insert` at `span` replaced by its code `text`: as statements (through the `;`) in
/// statement position, else as a parenthesized expression.
fn inline_insert(text: &str, span: Span, code: &str) -> Option<(Span, String)> {
    let before = text[..span.start].trim_end();
    let statement = before.is_empty() || before.ends_with([';', '{', '}']);
    if statement {
        let rest = &text[span.end..];
        let semi = rest.trim_start_matches([' ', '\t']);
        let end = if semi.starts_with(';') {
            span.end + (rest.len() - semi.len()) + 1
        } else {
            span.end
        };
        let line_start = text[..span.start].rfind('\n').map_or(0, |n| n + 1);
        let indent: String = text[line_start..span.start]
            .chars()
            .take_while(|c| c.is_whitespace())
            .collect();
        let body = jaic::sema::ide_meta::dedent(code.trim_matches('\n'));
        let body = body
            .lines()
            .collect::<Vec<_>>()
            .join(&format!("\n{indent}"));
        return Some((Span::new(span.start, end), body));
    }
    let code = code.trim().trim_end_matches(';').trim();
    if code.is_empty() {
        return None;
    }
    Some((span, format!("({code})")))
}

/// The call whose argument list the cursor at `byte` is in: the `(` offset, the callee chain
/// (`Module.name` → [Module, name]), the argument index, and the name of a `name =` argument.
fn open_call(text: &str, byte: usize) -> Option<(usize, Vec<&str>, usize, Option<String>)> {
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut commas = 0usize;
    let mut at = byte.min(bytes.len());
    let mut current_start = byte;
    let limit = byte.saturating_sub(4096);
    while at > limit {
        at -= 1;
        match bytes[at] {
            b')' | b']' | b'}' => depth += 1,
            b'(' | b'[' | b'{' if depth > 0 => depth -= 1,
            b'(' => break,
            b'[' | b'{' | b';' => return None,
            b',' if depth == 0 => {
                if commas == 0 {
                    current_start = at + 1;
                }
                commas += 1;
            }
            b'"' => {
                // Skip back over a string literal on this line.
                let line = text[..at].rfind('\n').map_or(0, |n| n + 1);
                at = text[line..at].rfind('"').map_or(at, |q| line + q);
            }
            _ => {}
        }
    }
    if bytes.get(at) != Some(&b'(') {
        return None;
    }
    if commas == 0 {
        current_start = at + 1;
    }
    let ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b >= 0x80;
    let mut chain = Vec::new();
    let mut end = at;
    loop {
        let mut s = end;
        while s > 0 && ident(bytes[s - 1]) {
            s -= 1;
        }
        if s == end || bytes[s].is_ascii_digit() {
            break;
        }
        chain.push(&text[s..end]);
        if s > 0 && bytes[s - 1] == b'.' {
            end = s - 1;
        } else {
            break;
        }
    }
    if chain.is_empty() || matches!(chain[0], "if" | "while" | "for" | "cast" | "case" | "ifx") {
        return None;
    }
    chain.reverse();
    let current = text[current_start..byte].trim_start();
    let named = current.split_once('=').and_then(|(n, rest)| {
        let n = n.trim();
        (!rest.starts_with('=')
            && !n.is_empty()
            && n.chars().all(|c| c.is_alphanumeric() || c == '_'))
        .then(|| n.to_string())
    });
    Some((at, chain, commas, named))
}

/// The type written in a parameter snippet (`name: Type = default`), or `None` for `..` varargs.
fn param_type(param: &str) -> Option<&str> {
    let ty = param.split_once(':').map_or(param, |(_, ty)| ty);
    let ty = ty.split_once('=').map_or(ty, |(ty, _)| ty).trim();
    (!ty.starts_with("..")).then_some(ty)
}

/// Which parameters share their type with another one. Only those get name hints: in
/// `print(format: string, args: ..Any)` the literal's role is obvious, in
/// `clamp(x: float, lo: float, hi: float)` it is not.
fn ambiguous_params(params: &[String]) -> Vec<bool> {
    let types: Vec<Option<&str>> = params.iter().map(|p| param_type(p)).collect();
    types
        .iter()
        .map(|ty| ty.is_some_and(|ty| types.iter().filter(|other| **other == Some(ty)).count() > 1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_calls_find_the_callee_and_argument() {
        let text = "x := Math.clamp(a, f(b, c), ";
        let (open, chain, arg, named) = open_call(text, text.len()).unwrap();
        assert_eq!(&text[open..open + 1], "(");
        assert_eq!(chain, ["Math", "clamp"]);
        assert_eq!(arg, 2);
        assert_eq!(named, None);
        let text = "f(1, scale = ";
        assert_eq!(
            open_call(text, text.len()).unwrap().3.as_deref(),
            Some("scale")
        );
        assert!(open_call("if (a", 5).is_none());
    }

    #[test]
    fn inline_insert_keeps_indentation() {
        let text = "f :: () {\n    #insert \"a := 1;\\nb := 2;\";\n}";
        let start = text.find("#insert").unwrap();
        let end = text.find(";\n}").unwrap();
        let (span, body) = inline_insert(text, Span::new(start, end), "a := 1;\nb := 2;").unwrap();
        assert_eq!(&text[span.end - 1..span.end], ";");
        assert_eq!(body, "a := 1;\n    b := 2;");
    }

    #[test]
    fn inferred_declarations() {
        assert!(inferred_declaration("x := 1", 1));
        assert!(inferred_declaration("a, b := f()", 1));
        assert!(!inferred_declaration("x : int = 1", 1));
        assert!(!inferred_declaration("x = 1", 1));
    }
}
