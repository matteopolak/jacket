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
use crate::session::Session;
use crate::{CompletionItem, CompletionKind, SignatureHelp, SignatureInformation};
use jaic::parser::asm_ide::{
    AsmBlockSyntax, AsmPlace, AsmStmtKind, AsmStmtSyntax, asm_blocks, parse_asm_statement,
    word_range,
};
use jaic::sema::asm_catalog::{self as catalog, AsmInstruction};
use jaic::source::Span;

mod completion;

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
    /// Hover on a mnemonic or a declared register inside an `#asm` block.
    pub(crate) fn asm_hover(&self, text: &str, byte: usize) -> Option<(usize, usize, HoverText)> {
        if !text.contains(jaic::lexicon::spelled::ASM) {
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
        if !text.contains(jaic::lexicon::spelled::ASM) {
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
