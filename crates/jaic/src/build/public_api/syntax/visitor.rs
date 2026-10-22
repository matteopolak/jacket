//! Individual syntax forms shared by the node visitor.
use super::Printer;
use crate::{builtin_keys::SyntaxKey, interp::Trap, types::TypeId};

impl Printer<'_, '_> {
    pub(super) fn ident(&mut self, at: u64, ty: TypeId) -> Result<String, Trap> {
        Ok({
            let mut output = self.text(at, ty, "name")?;
            let flags = self.u(at, ty, "flags")?;
            if flags & 1 != 0 {
                output = format!("${output}");
                if self.flag(at, SyntaxKey::PolyBaked)? != 0 {
                    output = format!("${output}");
                }
            }
            if flags & 0x40 != 0 {
                output = format!("`{output}");
            }
            let restriction = self.extra(at, SyntaxKey::Restriction)?;
            if restriction != 0 {
                output += &format!(
                    "/{}{}",
                    if self.flag(at, SyntaxKey::Interface)? != 0 {
                        "interface "
                    } else {
                        ""
                    },
                    self.node(restriction, true)?
                );
            }
            output
        })
    }

    pub(super) fn type_instantiation(&mut self, at: u64, ty: TypeId) -> Result<String, Trap> {
        Ok({
            let flags = self.u(at, ty, "inst_flags")?;
            let prefix = if flags & 1 != 0 {
                ".."
            } else if flags & 32 != 0 {
                "#type,isa "
            } else if flags & 16 != 0 {
                "#type,distinct "
            } else if flags & 8 != 0 {
                "#type "
            } else {
                ""
            };
            let value = if self.u(at, ty, "pointer_to")? != 0 {
                format!("*{}", self.child(at, ty, "pointer_to")?)
            } else if self.u(at, ty, "array_element_type")? != 0 {
                format!(
                    "[{}] {}",
                    if flags & 2 != 0 {
                        "..".into()
                    } else {
                        self.child(at, ty, "array_dimension")?
                    },
                    self.child(at, ty, "array_element_type")?
                )
            } else if self.u(at, ty, "type_directive_target")? != 0 {
                self.child(at, ty, "type_directive_target")?
            } else {
                self.child(at, ty, "type_valued_expression")?
            };
            format!("{prefix}{value}")
        })
    }

    pub(super) fn procedure_call(&mut self, at: u64, ty: TypeId) -> Result<String, Trap> {
        Ok({
            let procedure = self.child(at, ty, "procedure_expression")?;
            let mut arguments = self.args(at, ty, "arguments_unsorted")?;
            let changes = self.metadata_list(at, SyntaxKey::ContextArguments)?;
            let names = self.attribute(at, SyntaxKey::ContextNames)?;
            for (index, change) in changes.into_iter().enumerate() {
                let name = names.split(',').nth(index).unwrap_or("");
                arguments += &format!(
                    " ,, {}{}",
                    if name.is_empty() {
                        String::new()
                    } else {
                        format!("{name} = ")
                    },
                    self.node(change, true)?
                );
            }
            let flags = self.flag(at, SyntaxKey::CallHint)?;
            let hint = if flags == 1 {
                "inline "
            } else if flags == 2 {
                "no_inline "
            } else {
                ""
            };
            format!("{hint}{procedure}({arguments})")
        })
    }

    pub(super) fn cast(&mut self, at: u64, ty: TypeId) -> Result<String, Trap> {
        Ok({
            let flags = self.u(at, ty, "cast_flags")?;
            let value = self.child(at, ty, "expression")?;
            if flags & 4 != 0 {
                format!("xx {value}")
            } else {
                format!(
                    "cast{}{}{}({}) {value}",
                    if flags & 8 != 0 {
                        ",no_check"
                    } else {
                        ""
                    },
                    if flags & 16 != 0 {
                        ",truncate"
                    } else {
                        ""
                    },
                    if flags & 128 != 0 {
                        ",force"
                    } else {
                        ""
                    },
                    self.child(at, ty, "target_type")?
                )
            }
        })
    }

    pub(super) fn conditional(&mut self, at: u64, ty: TypeId) -> Result<String, Trap> {
        Ok({
            let flags = self.u(at, ty, "if_flags")?;
            let condition = self.child(at, ty, "condition")?;
            let then = if flags & 2 != 0 {
                self.value_block(self.u(at, ty, "then_block")?)?
            } else {
                self.child(at, ty, "then_block")?
            };
            let otherwise = if flags & 2 != 0 {
                self.value_block(self.u(at, ty, "else_block")?)?
            } else {
                self.child(at, ty, "else_block")?
            };
            if flags & 2 != 0 {
                format!(
                    "{}ifx {condition} then {then}{}",
                    if flags & 4 != 0 {
                        "#"
                    } else {
                        ""
                    },
                    if otherwise.is_empty() {
                        String::new()
                    } else {
                        format!(" else {otherwise}")
                    }
                )
            } else {
                format!(
                    "{}if {condition}{}{} {then}{}",
                    if flags & 4 != 0 {
                        "#"
                    } else {
                        ""
                    },
                    if flags & 1 != 0 {
                        " =="
                    } else {
                        ""
                    },
                    if flags & 8 != 0 {
                        " #complete"
                    } else {
                        ""
                    },
                    if otherwise.is_empty() {
                        String::new()
                    } else {
                        format!(" else {otherwise}")
                    }
                )
            }
        })
    }

    pub(super) fn while_loop(&mut self, at: u64, ty: TypeId) -> Result<String, Trap> {
        Ok({
            let label = self.attribute(at, SyntaxKey::Label)?;
            let prefix = if label.is_empty() {
                String::new()
            } else {
                format!(
                    "{}{}{label}{} ",
                    if self.flag(at, SyntaxKey::BindLabel)? == 0 {
                        ":"
                    } else {
                        ""
                    },
                    if self.flag(at, SyntaxKey::BacktickLabel)? != 0 {
                        "`"
                    } else {
                        ""
                    },
                    if self.flag(at, SyntaxKey::BindLabel)? != 0 {
                        " :="
                    } else {
                        ""
                    }
                )
            };
            format!(
                "while {prefix}{} {}",
                self.child(at, ty, "condition")?,
                self.child(at, ty, "block")?
            )
        })
    }

    pub(super) fn insert(&mut self, at: u64, ty: TypeId) -> Result<String, Trap> {
        Ok({
            let mut text = format!("#insert{}", self.attribute(at, SyntaxKey::Flags)?);
            let scope = self.child(at, ty, "scope_redirection")?;
            if !scope.is_empty() {
                text += &format!(",scope({scope})");
            }
            let mut replacements = Vec::new();
            for (keyword, field) in [
                ("break", "break_replacement"),
                ("continue", "continue_replacement"),
                ("remove", "remove_replacement"),
            ] {
                let value = self.child(at, ty, field)?;
                if !value.is_empty() {
                    replacements.push(format!("{keyword} = {value}"));
                }
            }
            if !replacements.is_empty() {
                text += &format!(" ({})", replacements.join(", "));
            }
            format!("{text} {}", self.child(at, ty, "expression")?)
        })
    }

    pub(super) fn push_context(&mut self, at: u64, ty: TypeId) -> Result<String, Trap> {
        Ok({
            let mut text = String::new();
            if self.has(at, ty, "push_context_flags", "IS_BACKTICKED")? {
                text += "`";
            }
            text += "push_context";
            if self.has(at, ty, "push_context_flags", "DEFER_POP")? {
                text += ",defer_pop";
            }
            let original = self.extra(at, SyntaxKey::OriginalContext)?;
            let to_push = self.u(at, ty, "to_push")?;
            if to_push != 0 && !(self.flag(at, SyntaxKey::ContextBare)? != 0 && original == to_push)
            {
                text += &format!(" {}", self.node(to_push, false)?);
            }
            let block = self.child(at, ty, "block")?;
            if !block.is_empty() {
                text += &format!(" {block}");
            }
            text
        })
    }

    pub(super) fn location(&mut self, at: u64, ty: TypeId) -> Result<String, Trap> {
        Ok({
            if self.flag(at, SyntaxKey::CallerCode)? != 0 {
                "#caller_code".into()
            } else if self.u(at, ty, "is_caller_location")? != 0 {
                "#caller_location".into()
            } else {
                let arg = self.child(at, ty, "expression")?;
                if arg.is_empty() {
                    "#location".into()
                } else {
                    format!("#location({arg})")
                }
            }
        })
    }

    pub(super) fn run(&mut self, at: u64, ty: TypeId) -> Result<String, Trap> {
        Ok({
            let body = self.extra(at, SyntaxKey::RunBody)?;
            let value = if body != 0 {
                self.node(body, true)?
            } else {
                format!("({})()", self.child(at, ty, "procedure")?)
            };
            if self.u(at, ty, "flags")? & 1 != 0 {
                let message = self.extra_text(at, SyntaxKey::AssertMessage)?;
                let args = self.extra_text(at, SyntaxKey::AssertArgs)?;
                format!(
                    "#assert({value}{}{})",
                    if message.is_empty() {
                        String::new()
                    } else {
                        format!(", {message}")
                    },
                    if args.is_empty() {
                        String::new()
                    } else {
                        format!(", {args}")
                    }
                )
            } else {
                format!("#run{} {value}", self.attribute(at, SyntaxKey::Flags)?)
            }
        })
    }

    pub(super) fn placeholder(&mut self, at: u64) -> Result<String, Trap> {
        Ok({
            let name = self.attribute(at, SyntaxKey::DirectiveName)?;
            let args = self.extra(at, SyntaxKey::DirectiveArgs)?;
            format!(
                "#{name}{} {}",
                self.attribute(at, SyntaxKey::Flags)?,
                if args == 0 {
                    String::new()
                } else {
                    self.node(args, true)?
                }
            )
        })
    }
}
