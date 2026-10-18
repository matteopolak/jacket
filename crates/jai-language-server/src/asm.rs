//! Editor help inside `#asm { ... }` blocks: instruction, operand, register-class and
//! feature-modifier completion, hover on mnemonics and declared registers, and operand
//! signature help.
//!
//! The instruction data is the compiler's own table (`jaic::sema::asm_catalog`), so what is
//! offered is exactly what compiles. Blocks, statements and operands come from
//! `jaic::parser::asm_ide`, which is tolerant of what is being typed: an unterminated
//! `#asm {` still gets help (its body runs to the brace that closes it, or to the end of the
//! file).
use crate::hover::{Block, HoverText, Inline};
use crate::session::{Session, contains};
use crate::{
    CompletionItem, CompletionKind, CompletionList, DocumentUri, SignatureHelp,
    SignatureInformation, SymbolKind,
};
use jaic::lexer::{P, Tok};
use jaic::parser::asm_ide::{
    AsmBlockSyntax, AsmPlace, AsmStmtKind, AsmStmtSyntax, asm_blocks, parse_asm_statement,
    word_range,
};
use jaic::sema::asm_catalog::{self as catalog, AsmInstruction};
use jaic::sema::ide::{IdeKind, IdeName};
use jaic::source::{FileId, Span};
use std::collections::BTreeMap;

fn slice(text: &str, span: Span) -> &str {
    text.get(span.start as usize..span.end as usize)
        .unwrap_or("")
}

/// The block and place `byte` is in.
fn locate(blocks: &[AsmBlockSyntax], byte: usize) -> Option<(&AsmBlockSyntax, AsmPlace)> {
    blocks
        .iter()
        .find_map(|b| b.place(byte).map(|place| (b, place)))
}

/// A register declared in a block.
struct Reg {
    name: String,
    class: &'static str,
    span: Span,
}

fn class_name(name: &str) -> &'static str {
    catalog::register_classes()
        .into_iter()
        .find(|c| c.name == name)
        .map_or("gpr", |c| c.name)
}

/// Registers the block declares: `x: class;` statements and inline `x:` operands.
fn registers(text: &str, block: &AsmBlockSyntax) -> Vec<Reg> {
    let mut out = Vec::new();
    for stmt in &block.statements {
        match &stmt.kind {
            AsmStmtKind::Decl {
                decl, ..
            } => out.push(Reg {
                name: slice(text, decl.name).into(),
                class: decl.class.map_or("gpr", |c| class_name(slice(text, c))),
                span: decl.name,
            }),
            AsmStmtKind::Inst(inst) => {
                let mnemonic = slice(text, inst.mnemonic);
                let width = block.width_for(text, inst.suffix);
                for (i, op) in inst.operands.iter().enumerate() {
                    let Some(decl) = op.decl else {
                        continue;
                    };
                    let class = match decl.class {
                        Some(c) => class_name(slice(text, c)),
                        None => catalog::inline_register_class(mnemonic, width, i).unwrap_or("gpr"),
                    };
                    out.push(Reg {
                        name: slice(text, decl.name).into(),
                        class,
                        span: decl.name,
                    });
                }
            }
            _ => {}
        }
    }
    out
}

/// What the cursor is completing inside a block.
enum Slot {
    /// The start of a statement: a mnemonic or a declaration.
    Mnemonic,
    /// After `mnemonic.`.
    Suffix,
    /// After `name:`.
    Class,
    /// After `===`.
    Pin,
    Operand {
        mnemonic: String,
        index: usize,
    },
}

/// The slot at the end of `stmt`, a statement cut at the cursor.
fn slot(text: &str, stmt: &AsmStmtSyntax) -> Option<Slot> {
    let byte = stmt.span.end;
    Some(match &stmt.kind {
        AsmStmtKind::Empty => Slot::Mnemonic,
        AsmStmtKind::Decl {
            pin: Some(_), ..
        }
        | AsmStmtKind::Pin => Slot::Pin,
        // Only while the class itself is typed: nothing may follow it.
        AsmStmtKind::Decl {
            decl, ..
        } => decl.class_open.then_some(Slot::Class)?,
        AsmStmtKind::Inst(inst) => match inst.operands.last() {
            None if inst.suffix.is_some_and(|s| s.end == byte) => Slot::Suffix,
            None if inst.mnemonic.end == byte => Slot::Mnemonic,
            None => return None,
            Some(op) => {
                if op.pin.is_some() {
                    Slot::Pin
                } else if let Some(decl) = op.decl {
                    // `tmp: cl|` names the class of an inline declaration.
                    decl.class_open.then_some(Slot::Class)?
                } else {
                    Slot::Operand {
                        mnemonic: slice(text, inst.mnemonic).into(),
                        index: inst.operands.len() - 1,
                    }
                }
            }
        },
        AsmStmtKind::Other => return None,
    })
}

/// The Markdown documentation of an instruction.
fn instruction_doc(i: &AsmInstruction) -> String {
    format!(
        "{}\n\n```jai\n{}\n```\n\nRequires `{}`.",
        i.description,
        i.forms.join("\n"),
        i.feature
    )
}

fn instruction_item(i: &AsmInstruction) -> CompletionItem {
    CompletionItem {
        label: i.mnemonic.clone(),
        kind: CompletionKind::Instruction,
        detail: format!("{} ({})", i.forms[0], i.feature),
        documentation: Some(instruction_doc(i)),
        insert_text: None,
        ..CompletionItem::default()
    }
}

fn simple(label: &str, kind: CompletionKind, detail: &str) -> CompletionItem {
    CompletionItem {
        label: label.into(),
        kind,
        detail: detail.into(),
        ..CompletionItem::default()
    }
}

/// Operand size and vector width suffixes after `mnemonic.`.
const SUFFIXES: &[(&str, &str)] = &[
    ("b", "8-bit operation"),
    ("w", "16-bit operation"),
    ("d", "32-bit operation"),
    ("q", "64-bit operation (or the 8-byte MMX form)"),
    ("x", "128-bit vector (xmm)"),
    ("y", "256-bit vector (ymm)"),
    ("z", "512-bit vector (zmm)"),
];

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
    fn asm_scope_names(
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
            })
            .collect()
    }

    /// Hover on a mnemonic or a declared register inside an `#asm` block.
    pub(crate) fn asm_hover(&self, text: &str, byte: usize) -> Option<(usize, usize, HoverText)> {
        if !text.contains("#asm") {
            return None;
        }
        let blocks = asm_blocks(text);
        let (block, AsmPlace::Body) = locate(&blocks, byte)? else {
            return None;
        };
        if let AsmStmtKind::Inst(inst) = &block.statement_at(byte)?.kind
            && inst.mnemonic.start as usize <= byte
            && byte <= inst.mnemonic.end as usize
        {
            let (ms, me) = (inst.mnemonic.start as usize, inst.mnemonic.end as usize);
            let i = catalog::instruction(slice(text, inst.mnemonic))?;
            let text = HoverText::new(vec![
                Block::Code(i.forms.join("\n")),
                Block::Para(vec![Inline::Text(i.description.clone())]),
                Block::Para(vec![
                    Inline::Text("Requires ".into()),
                    Inline::Code(i.feature.into()),
                ]),
            ]);
            return Some((ms, me, text));
        }
        let (ws, we) = word_range(text, byte);
        let word = &text[ws..we];
        let reg = registers(text, block)
            .into_iter()
            .find(|r| r.name == word && !word.is_empty())?;
        let description = catalog::register_classes()
            .into_iter()
            .find(|c| c.name == reg.class)
            .map_or("", |c| c.description);
        Some((
            ws,
            we,
            HoverText::new(vec![
                Block::Code(format!("{}: {}", reg.name, reg.class)),
                Block::Para(vec![Inline::Text(format!("#asm {description}"))]),
            ]),
        ))
    }

    /// The operand forms of the instruction whose operands are being typed.
    pub(crate) fn asm_signature_help(&self, text: &str, byte: usize) -> Option<SignatureHelp> {
        if !text.contains("#asm") {
            return None;
        }
        let blocks = asm_blocks(text);
        let (block, AsmPlace::Body) = locate(&blocks, byte)? else {
            return None;
        };
        let start = block.statement_at(byte)?.span.start as usize;
        let Slot::Operand {
            mnemonic,
            index,
        } = slot(text, &parse_asm_statement(text, start, byte))?
        else {
            return None;
        };
        let i = catalog::instruction(&mnemonic)?;
        let signatures: Vec<SignatureInformation> = i
            .forms
            .iter()
            .map(|form| SignatureInformation {
                label: form.clone(),
                parameters: form
                    .split_once(' ')
                    .map(|(_, ops)| ops.split(", ").map(String::from).collect())
                    .unwrap_or_default(),
                documentation: None,
                parameter_docs: Vec::new(),
            })
            .collect();
        let active = signatures
            .iter()
            .position(|s| s.parameters.len() > index)
            .unwrap_or(0);
        Some(SignatureHelp {
            signatures,
            active_signature: active,
            active_parameter: index,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn place(text: &str, marker: &str) -> Option<(Vec<AsmBlockSyntax>, AsmPlace)> {
        let byte = text.find(marker).unwrap() + marker.len();
        let blocks = asm_blocks(text);
        let place = locate(&blocks, byte)?.1;
        Some((blocks, place))
    }

    #[test]
    fn blocks_are_found_without_parsing() {
        let text = "f :: () {\n    #asm AVX2 {\n        vpad\n}\n";
        let (blocks, here) = place(text, "vpad").unwrap();
        assert_eq!((here, blocks[0].width), (AsmPlace::Body, 32));
        assert!(matches!(
            place(text, "#asm AV"),
            Some((_, AsmPlace::Features { .. }))
        ));
        assert!(place(text, "f ::").is_none());
        let commented = "// #asm { add\nx := 1;";
        assert!(place(commented, "x :=").is_none());
        let after = "#asm { add x, 1; }\ny := 2;";
        assert!(place(after, "y :=").is_none());
    }

    #[test]
    fn slots_follow_the_statement_shape() {
        let text = "#asm { mov.q x, ab; t: gp";
        let at = |m: &str| text.find(m).unwrap() + m.len();
        let blocks = asm_blocks(text);
        let slot_at = |m: &str| {
            let byte = at(m);
            let start = blocks[0].statement_at(byte).unwrap().span.start as usize;
            slot(text, &parse_asm_statement(text, start, byte))
        };
        assert!(matches!(
            slot_at("ab"),
            Some(Slot::Operand {
                index: 1,
                ..
            })
        ));
        assert!(matches!(slot_at("mov"), Some(Slot::Mnemonic)));
        assert!(matches!(slot_at("mov.q"), Some(Slot::Suffix)));
        assert!(matches!(slot_at("gp"), Some(Slot::Class)));
    }

    #[test]
    fn an_unterminated_block_still_has_a_body() {
        let text = "f :: () {\n    #asm {\n        mov.q x, 1;\n        ad";
        let blocks = asm_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert!(!blocks[0].closed);
        assert_eq!(blocks[0].body.end as usize, text.len());
        let byte = text.len();
        assert_eq!(blocks[0].place(byte), Some(AsmPlace::Body));
        let start = blocks[0].statement_at(byte).unwrap().span.start as usize;
        let stmt = parse_asm_statement(text, start, byte);
        assert!(matches!(slot(text, &stmt), Some(Slot::Mnemonic)));
    }
}
