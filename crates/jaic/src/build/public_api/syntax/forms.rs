//! Syntax flags, declarations and structured control forms.
use super::{Printer, quote};
use crate::{builtin_keys::SyntaxKey, interp::Trap, types::TypeId};

impl Printer<'_, '_> {
    pub(super) fn enum_value(&self, ty: TypeId, name: &str) -> Result<u64, Trap> {
        self.cx
            .schema()
            .enums
            .get(&ty)
            .and_then(|members| members.iter().find(|(member, _)| member == name))
            .map(|(_, value)| *value as u64)
            .ok_or_else(|| {
                Trap::new(format!(
                    "compiler syntax enum {} has no {name}",
                    self.cx.schema().get(ty).name
                ))
            })
    }

    pub(super) fn enum_is(
        &self,
        at: u64,
        ty: TypeId,
        field: &str,
        name: &str,
    ) -> Result<bool, Trap> {
        let (_, kind) = self.cx.schema().field(ty, field)?;
        Ok(self.u(at, ty, field)? == self.enum_value(kind, name)?)
    }

    pub(super) fn has(&self, at: u64, ty: TypeId, field: &str, name: &str) -> Result<bool, Trap> {
        let (_, kind) = self.cx.schema().field(ty, field)?;
        Ok(self.u(at, ty, field)? & self.enum_value(kind, name)? != 0)
    }

    pub(super) fn notes(&mut self, at: u64, ty: TypeId) -> Result<String, Trap> {
        let notes = self.nodes(at, ty, "notes")?;
        Ok(if notes.is_empty() {
            String::new()
        } else {
            format!(" {}", notes.join(" "))
        })
    }

    pub(super) fn extra_text(&mut self, at: u64, key: SyntaxKey) -> Result<String, Trap> {
        let child = self.extra(at, key)?;
        if child == 0 {
            Ok(String::new())
        } else {
            self.node(child, true)
        }
    }

    pub(super) fn using_prefix(&mut self, at: u64, ty: TypeId) -> Result<String, Trap> {
        if self.u(at, ty, "no_parameters")? != 0 {
            return Err(Trap::new("cannot serialize using without parameters"));
        }
        let mut text = String::from("using");
        let kind = self.u(at, ty, "filter_type")?;
        if kind != 0 {
            let filter = if self.enum_is(at, ty, "filter_type", "ONLY")? {
                "only"
            } else if self.enum_is(at, ty, "filter_type", "EXCEPT")? {
                "except"
            } else if self.enum_is(at, ty, "filter_type", "MAP")? {
                "map"
            } else {
                return Err(Trap::new("unknown using filter"));
            };
            let expression = self.child(at, ty, "filter_expression")?;
            let args = if expression.is_empty() {
                let original = self.flag(at, SyntaxKey::StaticFilter)?;
                if original == 0 || (original == 3) != (filter == "map") {
                    return Err(Trap::new("using filter has no matching argument syntax"));
                }
                self.attribute(at, SyntaxKey::FilterArguments)?
            } else {
                expression
            };
            text += &format!(",{filter}({args})");
        }
        text += " ";
        Ok(text)
    }

    pub(super) fn private_using(&mut self, at: u64) -> Result<String, Trap> {
        let filter = self.extra_text(at, SyntaxKey::FilterExpression)?;
        if filter.is_empty() {
            return Ok(format!("using{} ", self.attribute(at, SyntaxKey::Filter)?));
        }
        let kind = match self.flag(at, SyntaxKey::FilterKind)? {
            1 => "only",
            2 => "except",
            3 => "map",
            _ => return Err(Trap::new("unknown private using filter")),
        };
        Ok(format!("using,{kind}({filter}) "))
    }

    pub(super) fn for_loop(&mut self, at: u64, ty: TypeId) -> Result<String, Trap> {
        if self.u(at, ty, "macro_expansion_procedure_call")? != 0
            || self.u(at, ty, "want_replacement_for_expansion")? != 0
        {
            return Err(Trap::new("cannot serialize expanded iterator"));
        }
        let mut text = format!("for {}", self.attribute(at, SyntaxKey::Flags)?);
        let iterator = self.attribute(at, SyntaxKey::Iterator)?;
        if !iterator.is_empty() {
            text += &format!(":{iterator} ");
        }
        for (field, flag, conditional, plain) in [
            ("want_pointer_expression", "POINTER", "*=", "* "),
            ("want_reverse_expression", "REVERSE", "<=", "< "),
        ] {
            let expression = self.child(at, ty, field)?;
            if !expression.is_empty() {
                text += &format!("{conditional}{expression} ");
            } else if self.has(at, ty, "for_flags", flag)? {
                text += plain;
            }
        }
        let ident = self.child(at, ty, "ident_it")?;
        let index = self.child(at, ty, "ident_it_index")?;
        if !ident.is_empty() {
            text += &ident;
            if !index.is_empty() {
                text += &format!(", {index}");
            }
            text += ": ";
        } else if !index.is_empty() {
            return Err(Trap::new("for index requires an iterator identifier"));
        }
        text += &self.child(at, ty, "iteration_expression")?;
        let right = self.child(at, ty, "iteration_expression_right")?;
        if !right.is_empty() {
            text += &format!(" .. {right}");
        }
        text += &format!(" {}", self.child(at, ty, "block")?);
        Ok(text)
    }

    pub(super) fn structure(&mut self, at: u64, ty: TypeId) -> Result<String, Trap> {
        let mut text = if self.has(at, ty, "textual_flags", "UNION")? {
            "union ".to_string()
        } else {
            "struct ".to_string()
        };
        if self.has(at, ty, "textual_flags", "UNION_IS_TAGGED")? {
            text += &format!("{} ", self.extra_text(at, SyntaxKey::UnionTag)?);
        }
        let arguments = self.u(at, ty, "arguments_block")?;
        if arguments != 0 {
            text += &format!(
                "({}) ",
                self.nodes(
                    arguments,
                    self.cx.schema().named("Code_Block")?,
                    "statements"
                )?
                .join(", ")
            );
        }
        for (flag, directive) in [
            ("NO_PADDING", "no_padding"),
            ("TYPE_INFO_NONE", "type_info_none"),
            ("TYPE_INFO_NO_SIZE_COMPLAINT", "type_info_no_size_complaint"),
            (
                "TYPE_INFO_PROCEDURES_ARE_VOID_POINTERS",
                "type_info_procedures_are_void_pointers",
            ),
        ] {
            if self.has(at, ty, "textual_flags", flag)? {
                text += &format!("#{directive} ");
            }
        }
        let align = self.u(at, ty, "alignment")?;
        if align > 0 {
            text += &format!("#align {align} ");
        } else {
            let expr = self.extra_text(at, SyntaxKey::Alignment)?;
            if !expr.is_empty() {
                text += &format!("#align {expr} ");
            }
        }
        for modify in self.nodes(at, ty, "modify_directives")? {
            text += &format!("{modify} ");
        }
        text += &self.child(at, ty, "block")?;
        text += &self.notes(at, ty)?;
        Ok(text)
    }

    pub(super) fn enum_block(&mut self, at: u64) -> Result<String, Trap> {
        let block = self.cx.schema().named("Code_Block")?;
        let mut text = String::from("{\n");
        for member in self.list(at, block, "statements")? {
            let (name, ty) = self.node_type(member)?;
            if name == "DECLARATION" {
                text += &self.text(member, ty, "name")?;
                let value = self.child(member, ty, "expression")?;
                if !value.is_empty() {
                    text += &format!(" :: {value}");
                }
                text += &self.notes(member, ty)?;
            } else if name == "IF" {
                text += &format!(
                    "#if {} {}",
                    self.child(member, ty, "condition")?,
                    self.enum_block(self.u(member, ty, "then_block")?)?
                );
                let other = self.u(member, ty, "else_block")?;
                if other != 0 {
                    text += &format!(" else {}", self.enum_block(other)?);
                }
            } else {
                text += &self.node(member, true)?;
            }
            text += ";\n";
        }
        text += "}";
        Ok(text)
    }

    pub(super) fn import(&mut self, at: u64, ty: TypeId) -> Result<String, Trap> {
        let mut text = String::new();
        if self.flag(at, SyntaxKey::Using)? != 0 {
            text += &self.private_using(at)?;
        }
        let alias = self.attribute(at, SyntaxKey::Alias)?;
        if !alias.is_empty() {
            text += &format!("{alias} :: ");
        }
        text += "#import";
        text += match self.flag(at, SyntaxKey::ImportKind)? {
            0 => "",
            1 => ",file",
            2 => ",dir",
            3 => ",string",
            _ => return Err(Trap::new("unknown import kind")),
        };
        text += &self.attribute(at, SyntaxKey::Flags)?;
        text += &format!(" {}", quote(&self.text(at, ty, "name")?));
        let module = self.u(at, ty, "module_parameters_call")?;
        let program = self.u(at, ty, "program_parameters_call")?;
        if program != 0 && module == 0 {
            text += "()";
        }
        for call in [module, program] {
            if call != 0 {
                text += &format!(
                    "({})",
                    self.args(
                        call,
                        self.cx.schema().named("Code_Procedure_Call")?,
                        "arguments_unsorted"
                    )?
                );
            }
        }
        Ok(text)
    }

    pub(super) fn compound(&mut self, at: u64, ty: TypeId) -> Result<String, Trap> {
        let decl = self.u(at, ty, "declaration_properties")?;
        let dtype = self.cx.schema().named("Code_Declaration")?;
        let args = self.u(at, ty, "comma_separated_assignment")?;
        let aty = self.cx.schema().named("Code_Comma_Separated_Argument")?;
        let args = self.list(
            args,
            self.cx.schema().named("Code_Comma_Separated_Arguments")?,
            "arguments",
        )?;
        let assigns = args.iter().try_fold(false, |found, &arg| {
            Ok::<_, Trap>(found || self.enum_is(arg, aty, "modifier", "ASSIGN")?)
        })?;
        let mut parts = Vec::new();
        for (index, &arg) in args.iter().enumerate() {
            let node = self.u(arg, aty, "node")?;
            let last = index + 1 == args.len();
            let part = if self.enum_is(arg, aty, "modifier", "DECLARE")? {
                let mut part = self.text(node, dtype, "name")?;
                if self.has(node, dtype, "flags", "HAS_SCOPE_MODIFIER")? {
                    part = format!("`{part}");
                }
                if assigns {
                    part += ":";
                    let mut inst = self.child(node, dtype, "type_inst")?;
                    if inst.is_empty() && last {
                        inst = self.child(decl, dtype, "type_inst")?;
                    }
                    if !inst.is_empty() {
                        part += &format!(" {inst}");
                    }
                }
                part
            } else if self.enum_is(arg, aty, "modifier", "ASSIGN")? {
                let mut part = self.node(node, true)?;
                if assigns && !last {
                    part += " =";
                }
                part
            } else {
                return Err(Trap::new("unknown compound declaration modifier"));
            };
            parts.push(part);
        }
        let mut text = parts.join(", ");
        if assigns {
            text += " = ";
        } else {
            let inst = self.child(decl, dtype, "type_inst")?;
            text += ":";
            if !inst.is_empty() {
                text += &format!(" {inst}");
            }
        }
        let value = self.child(decl, dtype, "expression")?;
        if !value.is_empty() {
            if !assigns {
                text += " = ";
            }
            text += &value;
        } else if self.has(decl, dtype, "flags", "IS_UNINITIALIZED")? {
            text += " = ---";
        }
        Ok(text)
    }
}
