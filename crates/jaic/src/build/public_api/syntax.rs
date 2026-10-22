//! Serialize editable public code records, retaining compiler-owned syntax attributes.
mod assembly;
mod forms;
mod numbers;
mod visitor;
use super::{
    Kind,
    abi::view,
    operations::{Call, temporary_string},
    records,
};
use crate::{
    build::MetaOp,
    builtin_keys::SyntaxKey,
    interp::{Interp, Rets, Trap},
    types::TypeId,
};

pub(super) fn get_code(cx: &Call, interp: &mut Interp) -> Result<Rets, Trap> {
    let source = Printer {
        cx,
        interp,
        depth: 0,
    }
    .node(cx.arg(0), true)?;
    let text = temporary_string(interp, source.as_bytes())?;
    cx.invoke(interp, MetaOp::ParseCode, &[text, cx.arg(1)])
}

pub(super) fn modify(cx: &Call, interp: &mut Interp) -> Result<Rets, Trap> {
    let id = records::record_id(cx, interp, cx.arg(1))?;
    let mut printer = Printer {
        cx,
        interp,
        depth: 0,
    };
    let ty = cx.schema().pointed(cx.binding.parameters[1])?;
    let block = printer.u(cx.arg(1), ty, "block")?;
    if block == 0 {
        return Err(Trap::new(
            "compiler_modify_procedure requires a procedure body and block",
        ));
    }
    let block_type = cx.schema().named("Code_Block")?;
    let statements = printer.list(block, block_type, "statements")?;
    let mut source = Vec::new();
    for statement in statements {
        source.push(printer.node(statement, true)?);
    }
    let count = source.len() as u64;
    let ids = printer
        .interp
        .register_data_global(&vec![0; source.len() * 8], 8, false, 1)?;
    let strings = printer
        .interp
        .register_data_global(&vec![0; source.len() * 16], 8, false, 1)?;
    for (index, text) in source.iter().enumerate() {
        super::abi::put_string(printer.interp, strings + index as u64 * 16, text.as_bytes())?;
    }
    cx.invoke(
        printer.interp,
        MetaOp::ModifyProcedure,
        &[cx.workspace(0)? as u64, id as u64, ids, strings, count],
    )
}

struct Printer<'a, 'b> {
    cx: &'a Call<'b>,
    interp: &'a mut Interp,
    depth: usize,
}

impl Printer<'_, '_> {
    fn u(&self, at: u64, ty: TypeId, name: &str) -> Result<u64, Trap> {
        self.cx.number(self.interp, ty, at, name)
    }

    fn text(&self, at: u64, ty: TypeId, name: &str) -> Result<String, Trap> {
        self.cx.text(self.interp, ty, at, name)
    }

    fn list(&self, at: u64, ty: TypeId, name: &str) -> Result<Vec<u64>, Trap> {
        let (address, array) = self.cx.field(ty, at, name)?;
        let Kind::Array {
            element,
            count,
            ..
        } = self.cx.schema().get(array).kind
        else {
            return Err(Trap::new("syntax field must be an array"));
        };
        let (count, data) = if let Some(count) = count {
            (count, address)
        } else {
            view(self.interp, address)?
        };
        let size = self.cx.schema().get(element).size;
        if count
            .checked_mul(size)
            .is_none_or(|bytes| bytes > crate::sema::value::MAX_IMAGE)
        {
            return Err(Trap::new("syntax array size overflow"));
        }
        let mut result = Vec::new();
        for index in 0..count {
            let at = data + index * size;
            result.push(
                if matches!(self.cx.schema().get(element).kind, Kind::Pointer(_)) {
                    self.cx.schema().read_number(self.interp, element, at)?
                } else {
                    at
                },
            );
        }
        Ok(result)
    }

    fn syntax(&mut self, at: u64, key: SyntaxKey) -> Result<crate::records::Item, Trap> {
        let handle = records::builder(self.cx, self.interp)?;
        let id = self.cx.invoke(self.interp, MetaOp::RecOf, &[handle, at])?[0] as i64;
        let reg = self.cx.shared.borrow();
        Ok(reg
            .records
            .syntax_field(id, key)
            .and_then(crate::records::FieldRef::item)
            .unwrap_or(crate::records::Item::Int(0)))
    }

    fn flag(&mut self, at: u64, key: SyntaxKey) -> Result<u64, Trap> {
        Ok(match self.syntax(at, key)? {
            crate::records::Item::Int(value) => value as u64,
            _ => 0,
        })
    }

    fn attribute(&mut self, at: u64, key: SyntaxKey) -> Result<String, Trap> {
        Ok(match self.syntax(at, key)? {
            crate::records::Item::Str(text) => String::from_utf8_lossy(&text).into_owned(),
            _ => String::new(),
        })
    }

    fn extra(&mut self, at: u64, key: SyntaxKey) -> Result<u64, Trap> {
        let id = match self.syntax(at, key)? {
            crate::records::Item::Ref(id) => id,
            _ => 0,
        };
        if id == 0 {
            return Ok(0);
        }
        records::build(
            self.cx,
            self.interp,
            id,
            self.cx.schema().named("Code_Node")?,
        )
    }

    fn child(&mut self, at: u64, ty: TypeId, name: &str) -> Result<String, Trap> {
        let pointer = self.u(at, ty, name)?;
        if pointer == 0 {
            Ok(String::new())
        } else {
            self.node(pointer, false)
        }
    }

    fn nodes(&mut self, at: u64, ty: TypeId, name: &str) -> Result<Vec<String>, Trap> {
        let pointers = self.list(at, ty, name)?;
        pointers
            .into_iter()
            .map(|pointer| self.node(pointer, true))
            .collect()
    }

    fn args(&mut self, at: u64, ty: TypeId, name: &str) -> Result<String, Trap> {
        let arguments = self.list(at, ty, name)?;
        let argument_type = self.cx.schema().named("Code_Argument")?;
        let mut output = Vec::new();
        for argument in arguments {
            let name = self.child(argument, argument_type, "name")?;
            let pointer = self.u(argument, argument_type, "expression")?;
            let spread = pointer != 0
                && self.has(
                    pointer,
                    self.cx.schema().named("Code_Node")?,
                    "node_flags",
                    "EXPRESSION_IS_SPREAD",
                )?;
            let expression = format!(
                "{}{}",
                if spread {
                    ".."
                } else {
                    ""
                },
                self.child(argument, argument_type, "expression")?
            );
            output.push(if name.is_empty() {
                expression
            } else {
                format!("{name} = {expression}")
            });
        }
        Ok(output.join(", "))
    }

    fn node(&mut self, at: u64, skip: bool) -> Result<String, Trap> {
        if at == 0 {
            return Err(Trap::new("cannot serialize a null syntax node"));
        }
        if self.depth >= 512 {
            return Err(Trap::new("syntax serialization nesting limit exceeded"));
        }
        self.depth += 1;
        let result = self.inner(at, skip).and_then(|mut text| {
            let notes = self.extra(at, SyntaxKey::StatementNotes)?;
            if notes != 0 {
                text += " ";
                text += &self.node(notes, true)?;
            }
            Ok(text)
        });
        self.depth -= 1;
        result
    }

    fn node_type(&self, at: u64) -> Result<(String, TypeId), Trap> {
        let base = self.cx.schema().named("Code_Node")?;
        let kind = self.u(at, base, "kind")?;
        if kind == 0 {
            return Err(Trap::new("cannot serialize an uninitialized syntax node"));
        }
        let (_, kind_type) = self.cx.schema().field(base, "kind")?;
        let name = self
            .cx
            .schema()
            .enums
            .get(&kind_type)
            .and_then(|members| members.iter().find(|(_, value)| *value == kind as i64))
            .map(|(name, _)| name.clone())
            .ok_or_else(|| Trap::new(format!("unknown syntax node kind {kind}")))?;
        let class = format!(
            "Code_{}",
            name.split('_')
                .map(|word| {
                    let mut chars = word.chars();
                    chars.next().map_or(String::new(), |first| {
                        format!("{first}{}", chars.as_str().to_ascii_lowercase())
                    })
                })
                .collect::<Vec<_>>()
                .join("_")
        );
        let ty = self.cx.schema().named(&class)?;
        Ok((name, ty))
    }

    fn inner(&mut self, at: u64, skip: bool) -> Result<String, Trap> {
        let (name, ty) = self.node_type(at)?;
        let original = self.flag(at, SyntaxKey::OriginalKind)?;
        let kind = self.u(at, self.cx.schema().named("Code_Node")?, "kind")?;
        if original != 0 && original != kind {
            return Err(Trap::new(
                "cannot serialize a node whose kind was changed without rebuilding its record",
            ));
        }
        if self.flag(at, SyntaxKey::Incomplete)? != 0 {
            return Err(Trap::new("cannot serialize incomplete syntax"));
        }
        let text = match name.as_str() {
            "IDENT" => self.ident(at, ty)?,
            "LITERAL" => self.literal(at, ty)?,
            "CONTEXT" => "context".into(),
            "BINARY_OPERATOR" => {
                let op = self.u(at, ty, "operator_type")? as i32;
                let left = self.child(at, ty, "left")?;
                let right = self.child(at, ty, "right")?;
                if op == 46 {
                    format!("{left}.{right}")
                } else if op == 500 {
                    format!("{left}[{right}]")
                } else {
                    let expression = format!("{left} {} {right}", operator(op)?);
                    if skip {
                        expression
                    } else {
                        format!("({expression})")
                    }
                }
            }
            "UNARY_OPERATOR" => {
                let op = self.u(at, ty, "operator_type")? as i32;
                let value = self.child(at, ty, "subexpression")?;
                let result = if op == 169 {
                    format!("{value}.*")
                } else {
                    format!("{}{value}", operator(op)?)
                };
                if skip {
                    result
                } else {
                    format!("({result})")
                }
            }
            "PROCEDURE_CALL" => self.procedure_call(at, ty)?,
            "BLOCK" => format!(
                "{}{}{{\n{}\n}}",
                if self.has(at, ty, "block_flags", "NO_ARRAY_BOUNDS_CHECK")? {
                    "#no_abc "
                } else {
                    ""
                },
                if self.has(at, ty, "block_flags", "NO_ARITHMETIC_OVERFLOW_CHECK")? {
                    "#no_aoc "
                } else {
                    ""
                },
                self.statements(at)?
            ),
            "RETURN" => format!(
                "{}return {}",
                if self.u(at, ty, "return_flags")? & 1 != 0 {
                    "`"
                } else {
                    ""
                },
                self.args(at, ty, "arguments_unsorted")?
            ),
            "DECLARATION" => self.declaration(at, ty)?,
            "TYPE_INSTANTIATION" => self.type_instantiation(at, ty)?,
            "PROCEDURE_HEADER" => self.procedure(at, ty)?,
            "PROCEDURE_BODY" => {
                let header = self.u(at, ty, "header")?;
                self.procedure(header, self.cx.schema().named("Code_Procedure_Header")?)?
            }
            "CAST" => {
                let expression = self.cast(at, ty)?;
                if skip {
                    expression
                } else {
                    format!("({expression})")
                }
            }
            "COMMA_SEPARATED_ARGUMENTS" => {
                let arguments = self.list(at, ty, "arguments")?;
                let arg_ty = self.cx.schema().named("Code_Comma_Separated_Argument")?;
                let mut parts = Vec::new();
                for arg in arguments {
                    parts.push(self.child(arg, arg_ty, "node")?);
                }
                parts.join(", ")
            }
            "IF" => self.conditional(at, ty)?,
            "CASE" => {
                let block = self.u(at, ty, "then_block")?;
                format!(
                    "case {};\n{}{}",
                    self.child(at, ty, "condition")?,
                    if block == 0 {
                        String::new()
                    } else {
                        self.statements(block)?
                    },
                    if self.u(at, ty, "marked_as_fallthrough")? != 0 {
                        "#through;\n"
                    } else {
                        ""
                    }
                )
            }
            "WHILE" => self.while_loop(at, ty)?,
            "DEFER" => format!(
                "{}defer {}",
                if self.u(at, ty, "is_backticked")? != 0 {
                    "`"
                } else {
                    ""
                },
                self.child(at, ty, "block")?
            ),
            "LOOP_CONTROL" => {
                let control = self.u(at, ty, "control_type")?;
                format!(
                    "{} {}",
                    match control {
                        0 => "break",
                        1 => "continue",
                        _ => "remove",
                    },
                    self.child(at, ty, "target_ident")?
                )
            }
            "USING" => format!(
                "{}{}",
                self.using_prefix(at, ty)?,
                self.child(at, ty, "expression")?
            ),
            "FOR" => self.for_loop(at, ty)?,
            "STRUCT" => self.structure(at, ty)?,
            "ENUM" => {
                let mut text = if self.u(at, ty, "is_flags")? != 0 {
                    "enum_flags ".to_string()
                } else {
                    "enum ".to_string()
                };
                text += &self.child(at, ty, "internal_type_inst")?;
                if self.u(at, ty, "marked_as_complete")? != 0 {
                    text += " #complete ";
                }
                if self.u(at, ty, "marked_as_specified")? != 0 {
                    text += " #specified ";
                }
                text += &self.enum_block(self.u(at, ty, "block")?)?;
                text += &self.notes(at, ty)?;
                text
            }
            "DIRECTIVE_EXISTS" => format!("#exists({})", self.child(at, ty, "query_expression")?),
            "DIRECTIVE_BAKE" => format!(
                "{} {}",
                if self.enum_is(at, ty, "bake_type", "CONSTANTS_BAKE")? {
                    "#bake_constants"
                } else {
                    "#bake_arguments"
                },
                self.child(at, ty, "procedure_call")?
            ),
            "DIRECTIVE_MODULE_PARAMETERS" => format!(
                "#module_parameters {} {} {}",
                self.child(at, ty, "module_parameters")?,
                self.child(at, ty, "program_parameters")?,
                self.child(at, ty, "common_code")?
            ),
            "DIRECTIVE_INSERT" => self.insert(at, ty)?,
            "DIRECTIVE_IMPORT" => self.import(at, ty)?,
            "PUSH_CONTEXT" => self.push_context(at, ty)?,
            "ASM" => self.assembly(at)?,
            "COMPOUND_DECLARATION" => self.compound(at, ty)?,
            "RESOLVED_OVERLOAD" => self.child(at, ty, "source_expression")?,
            "NOTE" => format!("@{}", self.text(at, ty, "text")?),
            "TYPE_DEFINITION" => {
                return Err(Trap::new(
                    "compiler_get_code requires source syntax for a type definition",
                ));
            }
            "DIRECTIVE_THIS" => "#this".into(),
            "DIRECTIVE_COMPILE_TIME" => "#compile_time".into(),
            "DIRECTIVE_THROUGH" => "#through".into(),
            "DIRECTIVE_LOCATION" => self.location(at, ty)?,
            "DIRECTIVE_CONTEXT_TYPE" => "#context_type".into(),
            "DIRECTIVE_PROCEDURE_NAME" => {
                format!("#procedure_name({})", self.child(at, ty, "argument")?)
            }
            "DIRECTIVE_BYTES" => format!("#bytes {}", self.child(at, ty, "expression")?),
            "DIRECTIVE_ADD_CONTEXT" => {
                format!("#add_context {}", self.child(at, ty, "expression")?)
            }
            "DIRECTIVE_MODIFY" => format!("#modify {}", self.child(at, ty, "block")?),
            "DIRECTIVE_OVERLAY" => {
                format!("#overlay({})", self.child(at, ty, "target_expression")?)
            }
            "DIRECTIVE_PLACE" => {
                let operand = self.extra(at, SyntaxKey::Operand)?;
                format!("#place {}", self.node(operand, true)?)
            }
            "DIRECTIVE_CODE" if self.has(at, ty, "code_flags", "NULL")? => "#code,null".into(),
            "DIRECTIVE_CODE" => format!(
                "#code{} {}",
                self.attribute(at, SyntaxKey::Flags)?,
                self.child(at, ty, "expression")?
            ),
            "DIRECTIVE_RUN" => self.run(at, ty)?,
            "DIRECTIVE_LOAD" => format!("#load {}", quote(&self.text(at, ty, "short_name")?)),
            "DIRECTIVE_SCOPE" => match self.u(at, ty, "scope_type")? {
                0 => "#scope_export".into(),
                1 => "#scope_file".into(),
                2 => "#scope_module".into(),
                _ => return Err(Trap::new("unknown scope directive")),
            },
            "PLACEHOLDER" => self.placeholder(at)?,
            _ => {
                return Err(Trap::new(format!(
                    "compiler code serialization does not support {name}"
                )));
            }
        };
        Ok(text)
    }

    fn literal(&mut self, at: u64, ty: TypeId) -> Result<String, Trap> {
        match self.u(at, ty, "value_type")? {
            1 => {
                let value = self.u(at, ty, "_u64")?;
                if self.has(at, ty, "value_flags", "FLOAT")? {
                    self.floating(at, ty, value)
                } else if self.has(at, ty, "value_flags", "HEX")?
                    || self.has(at, ty, "value_flags", "BINARY")?
                {
                    let negative = self.has(at, ty, "value_flags", "MINUS_SIGN")?;
                    let magnitude = if negative {
                        value.wrapping_neg()
                    } else {
                        value
                    };
                    Ok(format!(
                        "{}{}",
                        if negative {
                            "-"
                        } else {
                            ""
                        },
                        if self.has(at, ty, "value_flags", "BINARY")? {
                            format!("0b{magnitude:b}")
                        } else {
                            format!("0x{magnitude:x}")
                        }
                    ))
                } else if self.has(at, ty, "value_flags", "MINUS_SIGN")? {
                    Ok((value as i64).to_string())
                } else {
                    Ok(value.to_string())
                }
            }
            2 => {
                let address = self.cx.field(ty, at, "_string")?.0;
                let (count, data) = view(self.interp, address)?;
                let raw = super::abi::bytes(
                    self.interp,
                    data,
                    usize::try_from(count).map_err(|_| Trap::new("invalid syntax string size"))?,
                )?;
                Ok(quote_bytes(&raw))
            }
            3 => Ok(if self.u(at, ty, "_s64")? == 0 {
                "false"
            } else {
                "true"
            }
            .into()),
            7 => {
                let pointer = self.u(at, ty, "struct_literal_info")?;
                let literal = self.cx.schema().named("Code_Struct_Literal_Info")?;
                let target = self.child(pointer, literal, "type_expression")?;
                let arguments = self.nodes(pointer, literal, "arguments")?;
                Ok(format!("{target}.{{{}}}", arguments.join(", ")))
            }
            6 => {
                let pointer = self.u(at, ty, "array_literal_info")?;
                let literal = self.cx.schema().named("Code_Array_Literal_Info")?;
                Ok(format!(
                    "{}.[{}]",
                    self.child(pointer, literal, "element_type")?,
                    self.nodes(pointer, literal, "array_members")?.join(", ")
                ))
            }
            8 => {
                let pointer = self.u(at, ty, "pointer_literal_info")?;
                if pointer == 0 {
                    Ok("null".into())
                } else {
                    let literal = self.cx.schema().named("Code_Pointer_Literal_Info")?;
                    if self.u(pointer, literal, "data_pointer")? == 0 {
                        Ok("null".into())
                    } else {
                        Err(Trap::new("cannot serialize an opaque pointer literal"))
                    }
                }
            }
            0 => Ok("---".into()),
            value => Err(Trap::new(format!("unsupported literal value kind {value}"))),
        }
    }

    fn declaration(&mut self, at: u64, ty: TypeId) -> Result<String, Trap> {
        self.declaration_inner(at, ty, true, false)
    }

    fn declaration_inner(
        &mut self,
        at: u64,
        ty: TypeId,
        print_using: bool,
        parameter: bool,
    ) -> Result<String, Trap> {
        let name = self.text(at, ty, "name")?;
        let mut text = String::new();
        let tag = self.extra_text(at, SyntaxKey::UnionTag)?;
        if !tag.is_empty() {
            text += &format!("{tag} ,, ");
        }
        if self.has(at, ty, "flags", "IS_MARKED_AS_AS")? {
            text += "#as ";
        }
        if print_using && self.flag(at, SyntaxKey::Using)? != 0 {
            text += &self.private_using(at)?;
        }
        let target = self.child(at, ty, "type_inst")?;
        if !name.is_empty() {
            if self.has(at, ty, "flags", "HAS_SCOPE_MODIFIER")? {
                text += "`";
            }
            text += &name;
            text += ":";
        }
        text += &format!(" {target}");
        let mut value = self.u(at, ty, "expression")?;
        let alias = self.extra(at, SyntaxKey::AliasExpression)?;
        if alias != 0 && value == self.extra(at, SyntaxKey::InitialExpression)? {
            value = alias;
        }
        if value != 0 {
            text += if !name.is_empty() && !parameter && self.has(at, ty, "flags", "IS_CONSTANT")? {
                ": "
            } else {
                " = "
            };
            text += &self.node(value, true)?;
        } else if self.has(at, ty, "flags", "IS_UNINITIALIZED")? {
            text += " = ---";
        }
        let align = self.child(at, ty, "alignment_expression")?;
        if !align.is_empty() {
            text += &format!(" #align {align}");
        }
        text += &self.attribute(at, SyntaxKey::Flags)?;
        if self.flag(at, SyntaxKey::Foreign)? != 0 {
            text += " #foreign";
            let library = self.extra_text(at, SyntaxKey::ForeignLibrary)?;
            if !library.is_empty() {
                text += &format!(" {library}");
            }
            if self.flag(at, SyntaxKey::ForeignNamed)? != 0 {
                text += &format!(" {}", quote(&self.attribute(at, SyntaxKey::ForeignSymbol)?));
            }
        }
        if value == 0 || self.node_type(value)?.0 != "PROCEDURE_HEADER" {
            text += &self.notes(at, ty)?;
        }
        Ok(text)
    }

    fn procedure(&mut self, at: u64, ty: TypeId) -> Result<String, Trap> {
        let dtype = self.cx.schema().named("Code_Declaration")?;
        let utype = self.cx.schema().named("Code_Using")?;
        let lambda = self.extra(at, SyntaxKey::LambdaBody)?;
        let mut parameters = Vec::new();
        for arg in self.list(at, ty, "arguments")? {
            let mut text = String::new();
            let name = self.text(arg, dtype, "name")?;
            for using in self.list(at, ty, "parameter_usings")? {
                let expression = self.u(using, utype, "expression")?;
                if expression == 0 || self.node_type(expression)?.0 != "IDENT" {
                    return Err(Trap::new("parameter using requires an identifier"));
                }
                if self.text(expression, self.cx.schema().named("Code_Ident")?, "name")? == name {
                    text += &self.using_prefix(using, utype)?;
                }
            }
            if self.has(arg, dtype, "flags", "IS_MARKED_AS_DISCARD")? {
                text += "#discard ";
            }
            if self.has(arg, dtype, "flags", "AUTO_VALUE_BAKE")? {
                text += "$$";
            } else if self.has(arg, dtype, "flags", "IS_CONSTANT")? {
                text += "$";
            }
            if lambda != 0 && self.u(arg, dtype, "type_inst")? == 0 {
                text += &name;
            } else {
                text += &self.declaration_inner(arg, dtype, false, true)?;
            }
            parameters.push(text);
        }
        let mut text = format!("({})", parameters.join(", "));
        if lambda != 0 {
            return Ok(format!("{text} => {}", self.node(lambda, true)?));
        }
        let mut returns = Vec::new();
        for arg in self.list(at, ty, "returns")? {
            let mut result = self.declaration_inner(arg, dtype, false, false)?;
            if self.flag(arg, SyntaxKey::ReturnMust)? != 0 {
                result += " #must";
            }
            returns.push(result);
        }
        if !returns.is_empty() {
            text += &format!(" -> ({})", returns.join(", "));
        }
        for (flag, directive) in [
            ("C_CALL", " #c_call"),
            ("SYNTACTICALLY_MARKED_AS_NO_CONTEXT", " #no_context"),
            ("MACRO", " #expand"),
            ("NO_DEBUG", " #no_debug"),
            ("SYMMETRIC", " #symmetric"),
            ("CPP_METHOD", " #cpp_method"),
            ("NO_CALL", " #no_call"),
            ("SYNTACTICALLY_MARKED_AS_INLINE_YES", " inline"),
            ("SYNTACTICALLY_MARKED_AS_INLINE_NO", " no_inline"),
        ] {
            if self.has(at, ty, "procedure_flags", flag)? {
                text += directive;
            }
        }
        if self.has(
            at,
            ty,
            "procedure_flags",
            "SYNTACTICALLY_MARKED_AS_COMPILER",
        )? || self.has(at, ty, "procedure_flags", "INTRINSIC")?
        {
            text += if self.has(at, ty, "procedure_flags", "INTRINSIC")? {
                " #intrinsic"
            } else {
                " #compiler"
            };
            let builtin = self.attribute(at, SyntaxKey::BuiltinName)?;
            if !builtin.is_empty() {
                text += &format!(" {}", quote(&builtin));
            }
        }
        if self.has(at, ty, "procedure_flags", "DEPRECATED")? {
            text += " #deprecated";
            let message = self.text(at, ty, "deprecation_string")?;
            if !message.is_empty() || self.flag(at, SyntaxKey::DeprecationNamed)? != 0 {
                text += &format!(" {}", quote(&message));
            }
        }
        if self.flag(at, SyntaxKey::Foreign)? != 0
            || self.has(at, ty, "procedure_flags", "ELSEWHERE")?
        {
            text += if self.has(at, ty, "procedure_flags", "ELSEWHERE")? {
                " #elsewhere"
            } else {
                " #foreign"
            };
            let library = self.child(at, ty, "library_identifier")?;
            if !library.is_empty() {
                text += &format!(" {library}");
            }
            let symbol = self.text(at, ty, "foreign_function_name")?;
            if !symbol.is_empty() || self.flag(at, SyntaxKey::ForeignNamed)? != 0 {
                text += &format!(" {}", quote(&symbol));
            }
        }
        text += &self.attribute(at, SyntaxKey::Flags)?;
        if self.flag(at, SyntaxKey::ProgramExport)? != 0 {
            text += " #program_export";
            if self.flag(at, SyntaxKey::ExportNamed)? != 0 {
                text += &format!(" {}", quote(&self.attribute(at, SyntaxKey::ExportName)?));
            }
        }
        for modify in self.nodes(at, ty, "modify_directives")? {
            text += &format!(" {modify}");
        }
        let body = self.u(at, ty, "body_or_null")?;
        if body != 0 {
            text += &format!(
                " {}",
                self.child(
                    body,
                    self.cx.schema().named("Code_Procedure_Body")?,
                    "block"
                )?
            );
        }
        text += &self.notes(at, ty)?;
        Ok(text)
    }
}

fn quote(value: &str) -> String {
    quote_bytes(value.as_bytes())
}

fn quote_bytes(value: &[u8]) -> String {
    let mut result = String::from("\"");
    for &byte in value {
        match byte {
            b'"' => result += "\\\"",
            b'\\' => result += "\\\\",
            b'\n' => result += "\\n",
            b'\r' => result += "\\r",
            b'\t' => result += "\\t",
            32..=126 => result.push(byte as char),
            _ => result += &format!("\\x{byte:02x}"),
        }
    }
    result.push('"');
    result
}

fn operator(value: i32) -> Result<String, Trap> {
    Ok(match value {
        131 => "==",
        132 => "!=",
        133 => "&&",
        134 => "||",
        135 => "<=",
        136 => ">=",
        137 | 168 => "<<",
        139 => "<<<",
        138 => ">>",
        140 => ">>>",
        141 => "---",
        145 => "+=",
        146 => "-=",
        147 => "*=",
        148 => "/=",
        149 => "%=",
        150 => "<<=",
        151 => ">>=",
        152 => "<<<=",
        153 => ">>>=",
        154 => "&=",
        155 => "|=",
        156 => "^=",
        157 => "&&=",
        158 => "||=",
        159 => "=",
        _ if (1..=127).contains(&value) => return Ok((value as u8 as char).to_string()),
        _ => return Err(Trap::new(format!("unsupported operator {value}"))),
    }
    .into())
}

impl Printer<'_, '_> {
    fn value_block(&mut self, at: u64) -> Result<String, Trap> {
        if at == 0 {
            return Ok(String::new());
        }
        let statements = self.list(at, self.cx.schema().named("Code_Block")?, "statements")?;
        if statements.len() == 1 {
            self.node(statements[0], false)
        } else {
            self.node(at, true)
        }
    }
}

impl Printer<'_, '_> {
    fn statements(&mut self, at: u64) -> Result<String, Trap> {
        let mut text = String::new();
        for statement in self.list(at, self.cx.schema().named("Code_Block")?, "statements")? {
            let case = self.node_type(statement)?.0 == "CASE";
            text += &self.node(statement, true)?;
            text += if case {
                "\n"
            } else {
                ";\n"
            };
        }
        Ok(text)
    }
}
