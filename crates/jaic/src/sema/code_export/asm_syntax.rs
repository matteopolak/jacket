//! Exporting inline assembly.
use super::Exporter;
use super::constants::node;
use crate::builtin_keys::SyntaxKey;
use crate::records::RecordTag;
use crate::sema::{Span, ast};

impl Exporter<'_> {
    fn asm_decl_syntax(&mut self, decl: &ast::AsmDecl, span: Span) -> i64 {
        let pin = decl.pin.map_or(String::new(), |pin| match pin {
            ast::AsmPin::Name(name) => name.name.to_string(),
            ast::AsmPin::Index(index) => index.to_string(),
        });
        let mut rec = self.node(RecordTag::CodeNode, node::PLACEHOLDER, span);
        rec.str(SyntaxKey::AsmKind, b"declaration")
            .str(SyntaxKey::Name, decl.name.name.as_str().as_bytes())
            .str(
                SyntaxKey::Class,
                decl.class
                    .map_or(String::new(), |class| class.name.to_string())
                    .as_bytes(),
            )
            .str(SyntaxKey::Pin, pin.as_bytes())
            .int(SyntaxKey::Colon, i64::from(decl.colon));
        self.add(rec)
    }

    fn asm_operand_syntax(&mut self, operand: &ast::AsmOperand, span: Span) -> i64 {
        match operand {
            ast::AsmOperand::Value(value) => self.expr(value),
            ast::AsmOperand::Decl(decl) => self.asm_decl_syntax(decl, span),
            ast::AsmOperand::Mem(memory) => {
                let terms: Vec<_> = memory
                    .terms
                    .iter()
                    .map(|term| {
                        let value = self.expr(&term.value);
                        let scale = self.opt_expr(term.scale.as_ref());
                        let mut rec =
                            self.node(RecordTag::CodeNode, node::PLACEHOLDER, memory.span);
                        rec.ptr(SyntaxKey::Value, value)
                            .ptr(SyntaxKey::Scale, scale)
                            .int(SyntaxKey::Negate, i64::from(term.negate));
                        self.add(rec)
                    })
                    .collect();
                let terms = self.node_list(&terms, memory.span);
                let mut rec = self.node(RecordTag::CodeNode, node::PLACEHOLDER, memory.span);
                rec.str(SyntaxKey::AsmKind, b"memory")
                    .ptr(SyntaxKey::Terms, terms)
                    .int(SyntaxKey::Broadcast, i64::from(memory.broadcast));
                self.add(rec)
            }
        }
    }

    pub(super) fn asm_syntax(&mut self, block: &ast::AsmBlock, span: Span) -> i64 {
        let items: Vec<_> = block
            .items
            .iter()
            .map(|item| match item {
                ast::AsmItem::Decl(decl) => self.asm_decl_syntax(decl, span),
                ast::AsmItem::Inst(inst) => {
                    let operands: Vec<_> = inst
                        .operands
                        .iter()
                        .map(|operand| self.asm_operand_syntax(operand, inst.span))
                        .collect();
                    let operands = self.node_list(&operands, inst.span);
                    let (suffix, size) = match &inst.size {
                        None => (String::new(), 0),
                        Some(ast::AsmSize::Suffix(suffix)) => (suffix.name.to_string(), 0),
                        Some(ast::AsmSize::Dynamic(size)) => (String::new(), self.expr(size)),
                    };
                    let mask = inst
                        .evex
                        .mask
                        .as_ref()
                        .map_or(0, |(mask, _)| self.asm_operand_syntax(mask, inst.span));
                    let round = inst.evex.round.map_or(String::new(), |mode| {
                        format!("!{}", mode.map_or(String::new(), |mode| mode.to_string()))
                    });
                    let mut rec = self.node(RecordTag::CodeNode, node::PLACEHOLDER, inst.span);
                    rec.str(SyntaxKey::AsmKind, b"instruction")
                        .str(SyntaxKey::Mnemonic, inst.mnemonic.name.as_str().as_bytes())
                        .ptr(SyntaxKey::Operands, operands)
                        .str(SyntaxKey::Suffix, suffix.as_bytes())
                        .ptr(SyntaxKey::Size, size)
                        .ptr(SyntaxKey::Mask, mask)
                        .int(
                            SyntaxKey::Zeroing,
                            i64::from(inst.evex.mask.as_ref().is_some_and(|(_, zeroing)| *zeroing)),
                        )
                        .str(SyntaxKey::Round, round.as_bytes());
                    self.add(rec)
                }
            })
            .collect();
        let items = self.node_list(&items, span);
        let mut rec = self.node(RecordTag::CodeAsm, node::ASM, span);
        rec.ptr(SyntaxKey::AsmItems, items)
            .int(SyntaxKey::AsmPresent, 1)
            .str(
                SyntaxKey::Features,
                block
                    .features
                    .iter()
                    .map(|feature| feature.name.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
                    .as_bytes(),
            );
        self.add(rec)
    }
}
