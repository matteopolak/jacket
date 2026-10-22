//! Assembly syntax metadata and editable operands.
use super::Printer;
use crate::{builtin_keys::SyntaxKey, interp::Trap};

impl Printer<'_, '_> {
    pub(super) fn metadata_list(&mut self, at: u64, key: SyntaxKey) -> Result<Vec<u64>, Trap> {
        let child = self.extra(at, key)?;
        if child == 0 {
            return Ok(Vec::new());
        }
        let (name, ty) = self.node_type(child)?;
        if name == "COMMA_SEPARATED_ARGUMENTS" {
            let args = self.list(child, ty, "arguments")?;
            let aty = self.cx.schema().named("Code_Comma_Separated_Argument")?;
            args.into_iter()
                .map(|arg| self.u(arg, aty, "node"))
                .collect()
        } else {
            Ok(vec![child])
        }
    }

    pub(super) fn assembly_operand(&mut self, at: u64) -> Result<String, Trap> {
        let kind = self.attribute(at, SyntaxKey::AsmKind)?;
        if kind == "declaration" {
            let mut text = self.attribute(at, SyntaxKey::Name)?;
            if self.flag(at, SyntaxKey::Colon)? != 0 {
                text += &format!(": {}", self.attribute(at, SyntaxKey::Class)?);
            }
            let pin = self.attribute(at, SyntaxKey::Pin)?;
            if !pin.is_empty() {
                text += &format!(" === {pin}");
            }
            return Ok(text);
        }
        if kind == "memory" {
            let mut terms = String::new();
            for term in self.metadata_list(at, SyntaxKey::Terms)? {
                if self.flag(term, SyntaxKey::Negate)? != 0 {
                    terms += " - ";
                } else if !terms.is_empty() {
                    terms += " + ";
                }
                terms += &self.extra_text(term, SyntaxKey::Value)?;
                let scale = self.extra_text(term, SyntaxKey::Scale)?;
                if !scale.is_empty() {
                    terms += &format!(" * {scale}");
                }
            }
            return Ok(format!(
                "[{terms}]{}",
                if self.flag(at, SyntaxKey::Broadcast)? != 0 {
                    "!"
                } else {
                    ""
                }
            ));
        }
        self.node(at, false)
    }

    pub(super) fn assembly(&mut self, at: u64) -> Result<String, Trap> {
        if self.flag(at, SyntaxKey::AsmPresent)? == 0 {
            return Err(Trap::new("assembly node has no syntax metadata"));
        }
        let mut text = format!("#asm {} {{\n", self.attribute(at, SyntaxKey::Features)?);
        for item in self.metadata_list(at, SyntaxKey::AsmItems)? {
            if self.attribute(item, SyntaxKey::AsmKind)? == "declaration" {
                text += &self.assembly_operand(item)?;
            } else {
                text += &self.attribute(item, SyntaxKey::Mnemonic)?;
                let suffix = self.attribute(item, SyntaxKey::Suffix)?;
                if !suffix.is_empty() {
                    text += &format!(".{suffix}");
                }
                let size = self.extra_text(item, SyntaxKey::Size)?;
                if !size.is_empty() {
                    text += &format!("?{size}");
                }
                let round = self.attribute(item, SyntaxKey::Round)?;
                let mut rounded = false;
                let mut operands = Vec::new();
                for operand in self.metadata_list(item, SyntaxKey::Operands)? {
                    let mut value = self.assembly_operand(operand)?;
                    if !round.is_empty()
                        && !rounded
                        && self.attribute(operand, SyntaxKey::AsmKind)? != "memory"
                    {
                        value += &round;
                        rounded = true;
                    }
                    operands.push(value);
                }
                if !round.is_empty() && !rounded {
                    return Err(Trap::new(
                        "assembly rounding modifier requires a register operand",
                    ));
                }
                if !operands.is_empty() {
                    text += &format!(" {}", operands.join(", "));
                }
                let mask = self.extra(item, SyntaxKey::Mask)?;
                if mask != 0 {
                    text += if self.flag(item, SyntaxKey::Zeroing)? != 0 {
                        " &* "
                    } else {
                        " & "
                    };
                    text += &self.assembly_operand(mask)?;
                }
            }
            text += ";\n";
        }
        text += "}";
        Ok(text)
    }
}
