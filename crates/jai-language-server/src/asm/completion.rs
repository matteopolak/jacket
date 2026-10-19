//! Completion inside `#asm` blocks: instructions, operands, register classes, feature
//! modifiers and the names in scope.
use super::{SUFFIXES, Slot, instruction_item, locate, registers, simple, slot};
use crate::session::{Session, contains};
use crate::{CompletionItem, CompletionKind, CompletionList, DocumentUri, SymbolKind};
use jaic::ide::{IdeKind, IdeName};
use jaic::lexer::{P, Tok};
use jaic::parser::asm_ide::{
    AsmBlockSyntax, AsmPlace, asm_blocks, parse_asm_statement, word_range,
};
use jaic::sema::asm_catalog as catalog;
use jaic::source::FileId;
use std::collections::BTreeMap;

impl Session {
    /// Completion inside an `#asm` block or its feature list; `None` elsewhere.
    pub(crate) fn asm_completion(
        &self,
        uri: &DocumentUri,
        text: &str,
        byte: usize,
    ) -> Option<CompletionList> {
        if !text.contains("#asm") {
            return None;
        }
        let blocks = asm_blocks(text);
        let (block, place) = locate(&blocks, byte)?;
        match place {
            AsmPlace::Features {
                word_start,
            } => {
                let typed = text[word_start as usize..byte].to_ascii_uppercase();
                return Some(CompletionList {
                    is_incomplete: false,
                    items: catalog::feature_modifiers()
                        .iter()
                        .filter(|f| f.starts_with(&typed))
                        .map(|f| simple(f, CompletionKind::Constant, "#asm feature modifier"))
                        .collect(),
                });
            }
            AsmPlace::Body => {}
        }
        let start = block.statement_at(byte)?.span.start as usize;
        let stmt = parse_asm_statement(text, start, byte);
        let (word, _) = word_range(text, byte);
        let prefix = &text[word..byte];
        let lower = prefix.to_ascii_lowercase();
        let mut incomplete = false;
        let items: Vec<CompletionItem> = match slot(text, &stmt)? {
            Slot::Mnemonic => {
                let mut items: Vec<CompletionItem> = Vec::new();
                for i in catalog::instructions() {
                    if !i.mnemonic.starts_with(&lower) {
                        continue;
                    }
                    if items.len() >= self.limits.symbols {
                        incomplete = true;
                        break;
                    }
                    items.push(instruction_item(i));
                }
                if prefix.is_empty() {
                    for class in catalog::register_classes() {
                        items.push(CompletionItem {
                            label: format!("name: {}", class.name),
                            kind: CompletionKind::Snippet,
                            detail: format!("declare a {} register", class.name),
                            documentation: Some(class.description.into()),
                            insert_text: Some(format!("${{1:name}}: {};", class.name)),
                            ..CompletionItem::default()
                        });
                    }
                }
                items
            }
            Slot::Suffix => SUFFIXES
                .iter()
                .filter(|(s, _)| s.starts_with(&lower))
                .map(|(s, d)| simple(s, CompletionKind::Keyword, d))
                .collect(),
            Slot::Class => catalog::register_classes()
                .into_iter()
                .filter(|c| c.name.starts_with(&lower))
                .map(|c| simple(c.name, CompletionKind::TypeAlias, c.description))
                .collect(),
            Slot::Pin => catalog::pin_names()
                .into_iter()
                .filter(|n| n.starts_with(&lower))
                .map(|n| {
                    simple(
                        &n,
                        CompletionKind::Constant,
                        "register pin (accepted and ignored)",
                    )
                })
                .collect(),
            Slot::Operand {
                ..
            } => {
                if prefix.bytes().next().is_some_and(|b| b.is_ascii_digit()) {
                    return Some(CompletionList::default());
                }
                let mut items = BTreeMap::new();
                for reg in registers(text, block) {
                    if reg.span.end as usize <= word && reg.name.starts_with(prefix) {
                        items.entry(reg.name.clone()).or_insert(CompletionItem {
                            label: reg.name,
                            kind: CompletionKind::Variable,
                            detail: format!("{} register (#asm)", reg.class),
                            ..CompletionItem::default()
                        });
                    }
                }
                for name in self.asm_scope_names(uri, text, block) {
                    if !name.name.starts_with(prefix) || items.len() >= self.limits.symbols {
                        continue;
                    }
                    let kind = if name.kind == IdeKind::Constant {
                        CompletionKind::Constant
                    } else {
                        CompletionKind::Variable
                    };
                    items.entry(name.name.clone()).or_insert(CompletionItem {
                        label: name.name,
                        kind,
                        detail: name.detail,
                        ..CompletionItem::default()
                    });
                }
                items.into_values().collect()
            }
        };
        Some(CompletionList {
            is_incomplete: incomplete,
            items,
        })
    }

    /// Jai variables and constants visible where the block starts: from the type checker with
    /// the block blanked out (so an unfinished block does not stop it), else from the syntax.
    pub(super) fn asm_scope_names(
        &self,
        uri: &DocumentUri,
        text: &str,
        block: &AsmBlockSyntax,
    ) -> Vec<IdeName> {
        let Ok((uri, doc)) = self.document_entry(uri) else {
            return Vec::new();
        };
        let at = block.directive.start as usize;
        let blank = |from: usize, to: usize| -> String {
            let mut bytes = text.as_bytes().to_vec();
            for b in &mut bytes[from..to] {
                if *b != b'\n' {
                    *b = b' ';
                }
            }
            String::from_utf8(bytes).unwrap_or_default()
        };
        if self.environment.is_some() {
            // Without the block; braces it left open (an unterminated block took the
            // procedure's `}`) are closed at the end, which keeps every offset.
            let mut probe = blank(at, (block.span().end as usize).min(text.len()));
            let open = jaic::lexer::lex_ide(FileId(0), &probe)
                .0
                .iter()
                .fold(0i64, |depth, t| match t.tok {
                    Tok::Punct(P::LBrace | P::DotBrace) => depth + 1,
                    Tok::Punct(P::RBrace) => depth - 1,
                    _ => depth,
                });
            if open > 0 {
                probe.push('\n');
                probe.push_str(&"}".repeat(open as usize));
            }
            if let Some(names) =
                self.with_semantic(uri, &probe, |a, path| a.complete(path, at, &[]))
            {
                return names
                    .into_iter()
                    .filter(|n| matches!(n.kind, IdeKind::Variable | IdeKind::Constant))
                    .collect();
            }
        }
        let analysis = match self.analysis_for(uri) {
            Some(a) if a.complete => a,
            _ => match self.parsed_for(uri).or_else(|| self.analysis_for(uri)) {
                Some(a) => a,
                None => return Vec::new(),
            },
        };
        analysis
            .rows
            .iter()
            .filter(|row| matches!(row.kind, SymbolKind::Variable | SymbolKind::Constant))
            .filter(|row| {
                if row.local {
                    contains(row.scope, at) && row.selection.start <= at
                } else {
                    row.parent.is_none()
                }
            })
            .map(|row| IdeName {
                name: row.name.as_str().into(),
                kind: if row.kind == SymbolKind::Constant {
                    IdeKind::Constant
                } else {
                    IdeKind::Variable
                },
                detail: Self::source_detail(doc, row).into(),
                decl: jaic::source::Span::NONE,
                call: None,
            })
            .collect()
    }
}
