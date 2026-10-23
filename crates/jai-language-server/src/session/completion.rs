//! Completion items and auto imports.
use super::{Completions, Session, contains, enclosing_declaration_start};
use crate::analysis::{Span, TokenKind};
use crate::semantic::{self};
use crate::{
    CompletionData, CompletionItem, CompletionKind, CompletionList, DocumentUri, Error, Position,
    SymbolKind,
};
use jaic::ide::{IdeCall, IdeKind, IdeName};
use jaic::lexer::Lexed;
use jaic::parser::ide_facts::{self, CursorContext};
use jaic::source::FileId;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

impl Session {
    /// Completion from the type checker. The word being typed (and `a.b.` before it) is cut
    /// out, so the text asked about is the same while a word is typed and its compile is reused.
    fn semantic_completion(
        &self,
        uri: &DocumentUri,
        text: &str,
        byte: usize,
        context: &CursorContext,
    ) -> Option<Completions> {
        let (start, member) = if let CursorContext::Member(m) = context {
            (m.word.start as usize, Some(m))
        } else {
            let ident = jaic::lexicon::is_identifier_char;
            let start = text[..byte]
                .char_indices()
                .rfind(|&(_, c)| !ident(c))
                .map_or(0, |(i, c)| i + c.len_utf8());
            (start, None)
        };
        let prefix = text[start..byte].to_owned();
        let (at, chain) = match member {
            // `.Member` with an inferred type: the enum its context expects.
            Some(m) if m.receiver.is_none() => {
                return self.complete_member_at(uri, text, byte, m.dot as usize, prefix, false);
            }
            // `f().x`, `a[i].x`: no names lead to the receiver; its type is what the check
            // records for it.
            Some(m) if m.chain.is_empty() => {
                return self.complete_member_at(uri, text, byte, m.dot as usize, prefix, true);
            }
            Some(m) => (
                m.chain[0].start as usize,
                m.chain
                    .iter()
                    .map(|s| &text[s.start as usize..s.end as usize])
                    .collect::<Vec<_>>(),
            ),
            None => (start, Vec::new()),
        };
        // The program as last compiled knows everything before the expression being typed.
        // Declarations below the cursor are where the compiled text had them; their docs are
        // read from the document as it is now.
        let ask = |a: &mut semantic::Analysis, path: &Path| {
            let mut names = a.complete(path, at, &chain)?;
            // A receiver with no members may be a stand-in left by a failed attempt: what the
            // check recorded for the member name has the rest.
            if names.is_empty() && !chain.is_empty() {
                return None;
            }
            a.shift_to(path, text, &mut names);
            let files = a.decl_files(&names);
            Some((names, files))
        };
        if let Some((names, files)) = self.with_compiled_before(uri, text, at, ask) {
            return Some((prefix, names, files));
        }
        if let Some((names, files)) = self.with_semantic(uri, text, ask) {
            return Some((prefix, names, files));
        }
        // Names a lookup from the scope cannot follow: a macro's parameter (checked where the
        // macro expands), `context`, a member reached through `using`.
        let m = member?;
        self.complete_member_at(uri, text, byte, m.dot as usize, prefix, true)
    }

    /// Members for `.NAME` typed with `dot` the offset of the `.`: the text asked about holds a
    /// placeholder name in place of the word, so the compiler records the type its context
    /// expects or, with `recorded`, the type of the receiver before the dot. The editor parser
    /// recovers from whatever is left open on the line, so the one text serves wherever the dot
    /// is.
    fn complete_member_at(
        &self,
        uri: &DocumentUri,
        text: &str,
        byte: usize,
        dot: usize,
        prefix: String,
        recorded: bool,
    ) -> Option<Completions> {
        const PLACEHOLDER: &str = "__jailsp_member";
        let ident = jaic::lexicon::is_identifier_char;
        let tail = byte
            + text[byte..]
                .find(|c: char| !ident(c))
                .unwrap_or(text.len() - byte);
        let candidate = format!("{}{PLACEHOLDER}{}", &text[..=dot], &text[tail..]);
        let found = self.with_semantic(uri, &candidate, |a, path| {
            let mut names = if recorded {
                a.complete_recorded(path, dot + 1)?
            } else {
                a.complete_inferred(path, dot + 1)?
            };
            a.shift_to(path, text, &mut names);
            let files = a.decl_files(&names);
            Some((names, files))
        });
        let (names, files) = found.unwrap_or_default();
        Some((prefix, names, files))
    }

    /// The visible names whose type fits the one the position expects (a parameter, a typed
    /// declaration, a return value, a condition), or `None` when no type is expected there. The
    /// word being typed is replaced by a placeholder `.NAME`, for which the compiler records
    /// the type its context expected.
    fn fitting_names(
        &self,
        uri: &DocumentUri,
        text: &str,
        start: usize,
    ) -> Option<BTreeSet<String>> {
        const PLACEHOLDER: &str = "__jailsp_member";
        let ident = jaic::lexicon::is_identifier_char;
        let tail = start
            + text[start..]
                .find(|c: char| !ident(c))
                .unwrap_or(text.len() - start);
        let candidate = format!("{}.{PLACEHOLDER}{}", &text[..start], &text[tail..]);
        self.with_semantic(uri, &candidate, |a, path| a.complete_fitting(path, start))
    }

    pub fn completion(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<CompletionList, Error> {
        let (uri, doc) = self.document_entry(uri)?;
        let byte = doc.index.byte(&doc.text, position)?;
        if let Some(list) = self.asm_completion(uri, &doc.text, byte) {
            return Ok(list);
        }
        let lexed = Lexed::new(FileId(0), &doc.text);
        let context = ide_facts::cursor_context_in(&doc.text, byte as u32, &lexed);
        let mut list = self.completion_items(uri, byte, &context, &lexed)?;
        let mut parameters = self.named_argument_completions(uri, byte);
        // In a call's arguments the callee's parameter names come first (see `rank`).
        for item in &mut parameters {
            item.sort_text = Some(format!("!{}", item.label));
        }
        if !parameters.is_empty() {
            // Both alternatives remain useful when a local expression shares a parameter name.
            list.items.extend(parameters);
            if list.items.len() > self.limits.symbols {
                list.items.truncate(self.limits.symbols);
                list.is_incomplete = true;
            }
        }
        // Naming a field of a struct literal: its fields come first; once one is named the values
        // are not expressions here, so only the fields remain.
        if let Some((fields, positional)) = self.struct_literal_completions(uri, byte)
            && !fields.is_empty()
        {
            if positional {
                list.items.extend(fields);
            } else {
                list.items = fields;
                list.is_incomplete = false;
            }
        }
        // Accepting an item replaces the whole word around the cursor, the letters after it
        // included, so confirming in the middle of a word does not leave its tail behind.
        let text = &doc.text;
        let word = |c: char| c.is_alphanumeric() || c == '_';
        let mut start = text[..byte]
            .char_indices()
            .rfind(|&(_, c)| !word(c))
            .map_or(0, |(i, c)| i + c.len_utf8());
        let mut end = byte
            + text[byte..]
                .find(|c: char| !word(c))
                .unwrap_or(text.len() - byte);
        if start > 0
            && text.as_bytes()[start - 1] == b'#'
            && list.items.iter().any(|i| i.label.starts_with('#'))
        {
            // A directive's label includes its `#`.
            start -= 1;
        }
        if let CursorContext::String(string) = &context
            && string.directive.is_some()
        {
            // Inside `#load "..."` / `#import "..."`: the entry after the last `/`.
            let from = string.start as usize;
            start = from + text[from..byte].rfind('/').map_or(0, |s| s + 1);
            end = byte
                + text[byte..]
                    .find(|c: char| c == '"' || c == '/' || c.is_whitespace())
                    .unwrap_or(text.len() - byte);
        }
        if let Ok(range) = doc.index.range(text, Span::new(start, end)) {
            for item in &mut list.items {
                if item.replace.is_none() {
                    item.replace = Some(range);
                }
            }
        }
        if self.snippet_support && matches!(context, CursorContext::Code | CursorContext::Member(_))
        {
            crate::call_snippets::apply(
                &mut list.items,
                text,
                &lexed,
                start,
                end,
                self.call_snippets,
            );
        }
        Ok(list)
    }

    fn completion_items(
        &self,
        uri: &DocumentUri,
        byte: usize,
        context: &CursorContext,
        lexed: &Lexed,
    ) -> Result<CompletionList, Error> {
        let (uri, doc) = self.document_entry(uri)?;
        let position = doc.index.position(&doc.text, byte)?;
        // `#` and the start of a directive: offer directives (labels include the `#`).
        // Past the whole separator: it may be multi-byte (a no-break space).
        let word_start = doc.text[..byte]
            .char_indices()
            .rfind(|&(_, c)| !(c.is_alphanumeric() || c == '_'))
            .map_or(0, |(i, c)| i + c.len_utf8());
        if doc.text[..word_start].ends_with('#') {
            let typed = doc.text[word_start..byte].to_lowercase();
            return Ok(CompletionList {
                is_incomplete: false,
                items: jaic::lexicon::described_directives()
                    .filter(|(name, _)| name.starts_with(&typed))
                    .map(|(name, detail)| CompletionItem {
                        label: format!("#{name}"),
                        kind: CompletionKind::Keyword,
                        detail: detail.into(),
                        ..CompletionItem::default()
                    })
                    .collect(),
            });
        }
        if let Some(list) = self.path_completion(uri, &doc.text, context, byte) {
            return Ok(list);
        }
        if let Some((prefix, names, files)) =
            self.semantic_completion(uri, &doc.text, byte, context)
        {
            let lower = prefix.to_lowercase();
            let mut items = BTreeMap::new();
            let mut incomplete = false;
            let keywords = jaic::lexicon::reserved().map(|k| IdeName {
                name: k.into(),
                kind: IdeKind::Constant,
                detail: "keyword".into(),
                decl: jaic::source::Span::NONE,
                call: crate::call_snippets::CALL_KEYWORDS
                    .contains(&k)
                    .then(IdeCall::unknown),
                ty: None,
            });
            let member = {
                let before = &doc.text.as_bytes()[..byte - prefix.len()];
                before.ends_with(b".") && !before.ends_with(b"..")
            };
            let all = names.into_iter().map(|n| (n, false));
            let all: Vec<(IdeName, bool)> = if member {
                all.collect()
            } else {
                all.chain(keywords.map(|k| (k, true))).collect()
            };
            let here = files
                .iter()
                .find(|(_, path)| path.as_str() == uri.path())
                .map(|(id, _)| *id);
            let declaration_start = enclosing_declaration_start(&doc.text, byte);
            let fitting = if member {
                None
            } else {
                self.fitting_names(uri, &doc.text, byte - prefix.len())
            };
            for (name, keyword) in all {
                if !name.name.to_lowercase().starts_with(&lower) {
                    continue;
                }
                let rank = rank(&name, keyword, here, declaration_start..byte);
                if items.len() >= self.limits.symbols {
                    incomplete = true;
                    break;
                }
                let kind = match name.kind {
                    _ if keyword => CompletionKind::Keyword,
                    IdeKind::Variable => CompletionKind::Variable,
                    IdeKind::Constant => CompletionKind::Constant,
                    IdeKind::Function => CompletionKind::Function,
                    IdeKind::Type => CompletionKind::Struct,
                    IdeKind::Module => CompletionKind::Module,
                    IdeKind::Field => CompletionKind::Field,
                    IdeKind::EnumMember => CompletionKind::EnumMember,
                };
                let data = files.get(&name.decl.file.0).map(|path| CompletionData {
                    document: uri.as_str().to_owned(),
                    path: path.clone(),
                    offset: name.decl.start as usize,
                });
                // Names that fit the expected type come first; the order within each group is
                // the usual one.
                let fit = match &fitting {
                    Some(set) if set.contains(&name.name) => "0",
                    Some(_) => "1",
                    None => "",
                };
                items.entry(name.name.clone()).or_insert(CompletionItem {
                    sort_text: Some(format!("{fit}{rank}{}", name.name)),
                    label: name.name,
                    kind,
                    detail: name.detail,
                    data,
                    call: name.call,
                    ..CompletionItem::default()
                });
            }
            let mut items: Vec<CompletionItem> = items.into_values().collect();
            // A short list carries its documentation; a long one waits for the client to
            // ask about the item it highlights.
            if items.len() <= crate::doc_view::EAGER_DOCS {
                for item in &mut items {
                    if let Some(data) = &item.data {
                        item.documentation = self.completion_documentation(data);
                    }
                }
            }
            if !member {
                incomplete |= self.add_auto_imports(uri, &doc.text, &prefix, &mut items);
            } else if self.snippet_support
                && let CursorContext::Member(m) = context
            {
                items.extend(self.postfix_items(uri, &doc.text, lexed, byte, m, &prefix));
            }
            return Ok(CompletionList {
                is_incomplete: incomplete,
                items,
            });
        }
        let analysis = match self.analysis_for(uri) {
            Some(a) if a.complete => a,
            _ => self.parsed_for(uri).unwrap_or(
                self.analysis_for(uri)
                    .expect("opened document has analysis"),
            ),
        };
        let prefix = self
            .word(uri, position)?
            .filter(|(_, token)| token.kind == TokenKind::Ident)
            .map_or("", |(_, token)| {
                &doc.text[token.span.start..byte.min(token.span.end)]
            });
        if self.word(uri, position)?.is_some_and(|(at, _)| {
            at > 0
                && self
                    .analysis_for(uri)
                    .expect("opened document has analysis")
                    .tokens[at - 1]
                    .kind
                    == TokenKind::Dot
        }) {
            return Ok(CompletionList {
                is_incomplete: true,
                items: vec![],
            });
        }
        let mut items = BTreeMap::new();
        let mut incomplete = !analysis.complete;
        for candidate in self.reachable(uri) {
            let rows = if candidate == uri {
                &analysis.rows
            } else {
                &self.analyses[candidate].rows
            };
            for row in rows {
                let visible = if row.local {
                    candidate == uri && contains(row.scope, byte) && row.selection.start <= byte
                } else {
                    row.parent.is_none() && (candidate == uri || !row.file_private)
                };
                let name = row.name.as_str();
                if visible && name.starts_with(prefix) && items.len() < self.limits.symbols {
                    let kind = match row.kind {
                        SymbolKind::Function => CompletionKind::Function,
                        SymbolKind::Struct => CompletionKind::Struct,
                        SymbolKind::Enum => CompletionKind::Enum,
                        SymbolKind::Constant => CompletionKind::Constant,
                        SymbolKind::TypeAlias => CompletionKind::TypeAlias,
                        _ => CompletionKind::Variable,
                    };
                    items.entry(name.to_owned()).or_insert(CompletionItem {
                        label: name.into(),
                        kind,
                        detail: Self::source_detail(&self.documents[candidate], row).into(),
                        ..CompletionItem::default()
                    });
                } else if visible && items.len() >= self.limits.symbols {
                    incomplete = true;
                }
            }
        }
        for name in jaic::lexicon::reserved() {
            if name.starts_with(prefix) && items.len() < self.limits.symbols {
                items.entry(name.into()).or_insert(CompletionItem {
                    label: name.into(),
                    kind: CompletionKind::Keyword,
                    detail: "keyword".into(),
                    ..CompletionItem::default()
                });
            }
        }
        let mut items: Vec<CompletionItem> = items.into_values().collect();
        incomplete |= self.add_auto_imports(uri, &doc.text, prefix, &mut items);
        Ok(CompletionList {
            is_incomplete: incomplete,
            items,
        })
    }

    /// Append the auto-import items for `prefix` (names not among `items`); whether the client
    /// should ask again as the word grows.
    fn add_auto_imports(
        &self,
        uri: &DocumentUri,
        text: &str,
        prefix: &str,
        items: &mut Vec<CompletionItem>,
    ) -> bool {
        let visible: BTreeSet<String> = items.iter().map(|i| i.label.clone()).collect();
        let (auto, incomplete) = self.auto_import_items(uri, text, prefix, &visible);
        items.extend(auto);
        incomplete
    }
}

/// A completion's `sortText` prefix. Editors order equally good matches by it (else by label,
/// where `_` sorts before letters): named arguments (`!`) first, then locals of the enclosing
/// declaration, the rest of this file, everything else, compiler internals (`__`) and last
/// auto-imports (`~`).
fn rank(name: &IdeName, keyword: bool, here: Option<u32>, local: std::ops::Range<usize>) -> char {
    let in_file = here == Some(name.decl.file.0) && name.decl != jaic::source::Span::NONE;
    match () {
        () if keyword => '3',
        // Strictly after the declaration's start: the declaration itself is a file name.
        () if in_file
            && local.start < name.decl.start as usize
            && local.contains(&(name.decl.start as usize)) =>
        {
            '1'
        }
        () if in_file => '2',
        () if name.name.starts_with("__") => '}',
        () => '3',
    }
}
