//! Expressions, literals and type instantiations.
use super::{Convert, R, quote_bytes};
use crate::{
    ast::{
        self, ArraySize, BinOp, CallHint, CastFlags, CodeBody, DirectiveKind, Expr, ExprKind,
        Int128, TypeModifier, UnOp, UnknownDirective, relocate::PLACEHOLDER_FILE,
    },
    builtin_keys::SyntaxKey,
    intern::Sym,
    interp::Trap,
    lexer::Tok,
    source::Span,
    types::TypeId,
};
use std::rc::Rc;

pub(super) fn mk(kind: ExprKind, span: Span) -> Expr {
    Expr {
        kind,
        span,
        parenthesized: false,
    }
}

impl Convert<'_, '_> {
    pub(super) fn expr(&mut self, at: u64) -> R<Expr> {
        self.node(at, |c, name, ty| c.expr_node(at, name, ty))
    }

    pub(super) fn opt_expr(&mut self, at: u64, ty: TypeId, field: &str) -> R<Option<Expr>> {
        let child = self.ptr(at, ty, field)?;
        if child == 0 {
            return Ok(None);
        }
        self.expr(child).map(Some)
    }

    /// An attribute node as an expression.
    pub(super) fn extra_expr(&mut self, at: u64, key: SyntaxKey) -> R<Option<Expr>> {
        let child = self.extra(at, key)?;
        if child == 0 {
            return Ok(None);
        }
        self.expr(child).map(Some)
    }

    /// The expression of a node already checked by `node`.
    pub(super) fn expr_node(&mut self, at: u64, name: &str, ty: TypeId) -> R<Expr> {
        let start = self.mark();
        let kind = match name {
            "IDENT" => return self.ident(at, ty),
            "LITERAL" => return self.literal(at, ty),
            "CONTEXT" => {
                self.word("context");
                ExprKind::Context
            }
            "BINARY_OPERATOR" => self.binary(at, ty)?,
            "UNARY_OPERATOR" => self.unary(at, ty)?,
            "PROCEDURE_CALL" => {
                let (callee, args, hint) = self.call(at, ty)?;
                ExprKind::Call {
                    callee: Box::new(callee),
                    args,
                    hint,
                }
            }
            "CAST" => self.cast(at, ty)?,
            "IF" => {
                let flags = self.num(at, ty, "if_flags")?;
                if flags & 2 == 0 {
                    return Err(Trap::new("an if statement cannot be used as an expression"));
                }
                self.word("ifx");
                let cond = self.child(at, ty, "condition")?;
                self.word("then");
                let then_value = self.value_block(at, ty, "then_block")?;
                let else_value = self.value_block(at, ty, "else_block")?;
                ExprKind::Ifx {
                    cond: Box::new(cond),
                    then_value: then_value.map(Box::new),
                    else_value: else_value.map(Box::new),
                    is_static: flags & 4 != 0,
                }
            }
            "TYPE_INSTANTIATION" => return self.type_inst(at, ty),
            "PROCEDURE_HEADER" => return self.procedure(at, ty),
            "PROCEDURE_BODY" => {
                let header = self.ptr(at, ty, "header")?;
                return self.node(header, |c, name, ty| {
                    if name != "PROCEDURE_HEADER" {
                        return Err(Trap::new(format!(
                            "a procedure body's header must be a PROCEDURE_HEADER, got {name}"
                        )));
                    }
                    c.procedure(header, ty)
                });
            }
            "BLOCK" => ExprKind::Block(Box::new(self.block_node(at, ty)?)),
            "STRUCT" => ExprKind::Struct(Rc::new(self.structure(at, ty)?)),
            "ENUM" => ExprKind::Enum(Rc::new(self.enumeration(at, ty)?)),
            "DIRECTIVE_EXISTS" => {
                self.word("#exists");
                ExprKind::Exists(Box::new(self.child(at, ty, "query_expression")?), None)
            }
            "DIRECTIVE_BAKE" => {
                let constants = self.enum_is(at, ty, "bake_type", "CONSTANTS_BAKE")?;
                self.word(if constants {
                    "#bake_constants"
                } else {
                    "#bake_arguments"
                });
                let call = self.child(at, ty, "procedure_call")?;
                let ExprKind::Call {
                    callee,
                    args,
                    ..
                } = call.kind
                else {
                    return Err(Trap::new("#bake_arguments requires a procedure call"));
                };
                ExprKind::Bake {
                    callee,
                    args,
                    constants,
                }
            }
            "DIRECTIVE_INSERT" => {
                let insert = self.insert(at, ty)?;
                ExprKind::Insert(Box::new(ast::InsertExpr {
                    value: insert.value,
                    flags: insert.flags,
                    scope: insert.scope,
                    replacements: insert.replacements,
                }))
            }
            "DIRECTIVE_LOCATION" => {
                if self.flag(at, SyntaxKey::CallerCode)? {
                    self.word("#caller_code");
                    ExprKind::CallerCode
                } else if self.num(at, ty, "is_caller_location")? != 0 {
                    self.word("#caller_location");
                    ExprKind::CallerLocation
                } else {
                    self.word("#location");
                    ExprKind::Location(self.opt_expr(at, ty, "expression")?.map(Box::new))
                }
            }
            "DIRECTIVE_PROCEDURE_NAME" => {
                self.word("#procedure_name");
                ExprKind::ProcedureName(self.opt_expr(at, ty, "argument")?.map(Box::new))
            }
            "DIRECTIVE_BYTES" => {
                self.word("#bytes");
                ExprKind::Bytes(Box::new(self.child(at, ty, "expression")?))
            }
            "DIRECTIVE_CODE" => {
                self.word("#code");
                if self.has(at, ty, "code_flags", "NULL")? {
                    let span = self.word("null");
                    ExprKind::Code(Rc::new(CodeBody::Expr(mk(ExprKind::Null, span))))
                } else {
                    let child = self.ptr(at, ty, "expression")?;
                    ExprKind::Code(Rc::new(self.code_body(child)?))
                }
            }
            "DIRECTIVE_THIS" => {
                self.word("#this");
                ExprKind::This
            }
            "DIRECTIVE_COMPILE_TIME" => {
                self.word("#compile_time");
                ExprKind::CompileTime
            }
            "DIRECTIVE_CONTEXT_TYPE" => {
                let span = self.word("#context_type");
                ExprKind::UnknownDirective(Box::new(UnknownDirective {
                    name: ast::Ident {
                        name: Sym::intern("context_type"),
                        span,
                    },
                    kind: DirectiveKind::of("context_type"),
                    flags: Vec::new(),
                    operand: None,
                }))
            }
            "PLACEHOLDER" => self.placeholder(at)?,
            "ASM" => ExprKind::Asm(Rc::new(self.assembly(at)?)),
            "RESOLVED_OVERLOAD" => return self.child(at, ty, "source_expression"),
            "DIRECTIVE_RUN" => {
                if self.num(at, ty, "flags")? & 1 != 0 {
                    return Err(Trap::new("#assert cannot be used as an expression"));
                }
                let (body, flags) = self.run(at, ty)?;
                ExprKind::Run {
                    body: Rc::new(body),
                    flags,
                }
            }
            "TYPE_DEFINITION" => {
                return Err(Trap::new(
                    "compiler_get_code requires source syntax for a type definition",
                ));
            }
            _ if super::STATEMENT_KINDS.contains(&name) => {
                return Err(Trap::new(format!(
                    "a {name} node cannot be used as an expression"
                )));
            }
            _ => {
                return Err(Trap::new(format!(
                    "compiler code serialization does not support {name}"
                )));
            }
        };
        Ok(mk(kind, self.since(start)))
    }

    fn ident(&mut self, at: u64, ty: TypeId) -> R<Expr> {
        let start = self.mark();
        let name = self.text(at, ty, "name")?;
        if name.is_empty() {
            return Err(Trap::new("an identifier needs a name"));
        }
        let flags = self.num(at, ty, "flags")?;
        let sym = Sym::intern(&name);
        let expr = if flags & 1 != 0 {
            let baked = self.flag(at, SyntaxKey::PolyBaked)?;
            self.word(&format!(
                "{}${name}",
                if baked {
                    "$"
                } else {
                    ""
                }
            ));
            let restriction = self.extra(at, SyntaxKey::Restriction)?;
            let kind = if restriction == 0 {
                ExprKind::PolyVar {
                    name: sym,
                    baked,
                }
            } else {
                self.word("/");
                let interface = self.flag(at, SyntaxKey::Interface)?;
                if interface {
                    self.word("interface");
                }
                ExprKind::PolyRestricted {
                    name: sym,
                    restriction: Box::new(self.expr(restriction)?),
                    interface,
                }
            };
            mk(kind, self.since(start))
        } else {
            mk(ExprKind::Ident(sym), self.word(&name))
        };
        Ok(if flags & 0x40 != 0 {
            mk(ExprKind::Backtick(Box::new(expr)), self.since(start))
        } else {
            expr
        })
    }

    fn literal(&mut self, at: u64, ty: TypeId) -> R<Expr> {
        let start = self.mark();
        let kind = match self.num(at, ty, "value_type")? {
            1 => return self.number(at, ty),
            2 => {
                let raw = self.raw(at, ty, "_string")?;
                self.word(&quote_bytes(&raw));
                ExprKind::Str(raw.into())
            }
            3 => {
                let value = self.num(at, ty, "_s64")? != 0;
                self.word(if value {
                    "true"
                } else {
                    "false"
                });
                ExprKind::Bool(value)
            }
            7 => {
                let info = self.ptr(at, ty, "struct_literal_info")?;
                if info == 0 {
                    return Err(Trap::new("a struct literal needs its struct_literal_info"));
                }
                let literal = self.ty("Code_Struct_Literal_Info")?;
                let target = self.opt_expr(info, literal, "type_expression")?;
                self.word(".{");
                let mut fields = Vec::new();
                for argument in self.list(info, literal, "arguments")? {
                    fields.push(self.struct_field(argument)?);
                }
                self.word("}");
                ExprKind::StructLit {
                    ty: target.map(Box::new),
                    fields,
                }
            }
            6 => {
                let info = self.ptr(at, ty, "array_literal_info")?;
                if info == 0 {
                    return Err(Trap::new("an array literal needs its array_literal_info"));
                }
                let literal = self.ty("Code_Array_Literal_Info")?;
                let element = self.opt_expr(info, literal, "element_type")?;
                self.word(".[");
                let elems = self
                    .list(info, literal, "array_members")?
                    .into_iter()
                    .map(|member| self.expr(member))
                    .collect::<R<Vec<_>>>()?;
                self.word("]");
                ExprKind::ArrayLit {
                    ty: element.map(Box::new),
                    elems,
                }
            }
            8 => {
                let info = self.ptr(at, ty, "pointer_literal_info")?;
                if info != 0 {
                    let literal = self.ty("Code_Pointer_Literal_Info")?;
                    if self.num(info, literal, "data_pointer")? != 0 {
                        return Err(Trap::new("cannot serialize an opaque pointer literal"));
                    }
                }
                self.word("null");
                ExprKind::Null
            }
            0 => {
                self.word("---");
                ExprKind::Uninit
            }
            value => return Err(Trap::new(format!("unsupported literal value kind {value}"))),
        };
        Ok(mk(kind, self.since(start)))
    }

    /// `name = value` and `target = value` fields are `=` operators.
    fn struct_field(&mut self, at: u64) -> R<ast::Arg> {
        let (name, ty) = self.node_type(at)?;
        if name == "BINARY_OPERATOR" && self.num(at, ty, "operator_type")? == u64::from(b'=') {
            let left = self.ptr(at, ty, "left")?;
            let (left_name, _) = self.node_type(left)?;
            let (name, target) = if left_name == "IDENT" {
                (Some(self.ident_name(left)?), None)
            } else {
                (None, Some(Box::new(self.expr(left)?)))
            };
            self.word("=");
            return Ok(ast::Arg {
                name,
                target,
                context: false,
                spread: false,
                value: self.child(at, ty, "right")?,
            });
        }
        Ok(ast::Arg {
            name: None,
            target: None,
            context: false,
            spread: false,
            value: self.expr(at)?,
        })
    }

    /// An integer or float literal, negated when its sign is set.
    fn number(&mut self, at: u64, ty: TypeId) -> R<Expr> {
        let start = self.mark();
        let value = self.num(at, ty, "_u64")?;
        let negative = self.has(at, ty, "value_flags", "MINUS_SIGN")?;
        let (negate, kind) = if self.has(at, ty, "value_flags", "FLOAT")? {
            let spelling = self.float_spelling(at, ty, value)?;
            let (negate, digits) = match spelling.strip_prefix('-') {
                Some(digits) => (true, digits.to_owned()),
                None => (false, spelling),
            };
            if negate {
                self.word("-");
            }
            let span = self.word(&digits);
            let Some(Tok::Float(value)) = crate::lexer::lex(PLACEHOLDER_FILE, &digits)
                .ok()
                .and_then(|tokens| tokens.into_iter().next())
                .map(|token| token.tok)
            else {
                return Err(Trap::new("edited float literal does not spell a float"));
            };
            (negate, mk(ExprKind::Float(value), span))
        } else if self.has(at, ty, "value_flags", "HEX")?
            || self.has(at, ty, "value_flags", "BINARY")?
        {
            let magnitude = if negative {
                value.wrapping_neg()
            } else {
                value
            };
            if negative {
                self.word("-");
            }
            let spelling = if self.has(at, ty, "value_flags", "BINARY")? {
                format!("0b{magnitude:b}")
            } else {
                format!("0x{magnitude:x}")
            };
            let span = self.word(&spelling);
            (
                negative,
                mk(ExprKind::Int(Int128::new(magnitude.into())), span),
            )
        } else {
            let negate = negative && (value as i64) < 0;
            let magnitude = if negate {
                (value as i64).unsigned_abs()
            } else {
                value
            };
            if negate {
                self.word("-");
            }
            let span = self.word(&magnitude.to_string());
            (
                negate,
                mk(ExprKind::Int(Int128::new(magnitude.into())), span),
            )
        };
        Ok(if negate {
            mk(
                ExprKind::Unary(UnOp::Neg, Box::new(kind)),
                self.since(start),
            )
        } else {
            kind
        })
    }

    /// The spelling of a float literal that keeps both its bits and the lexer's precision
    /// category (whether it defaults to or requires `float64`).
    fn float_spelling(&mut self, at: u64, ty: TypeId, bits: u64) -> R<String> {
        let value = f64::from_bits(bits);
        if self.has(at, ty, "value_flags", "HEX")? {
            let negative = self.has(at, ty, "value_flags", "MINUS_SIGN")?;
            let magnitude = if negative {
                -value
            } else {
                value
            };
            let text = if self.has(at, ty, "value_flags", "DEFAULTS_TO_FLOAT64")? {
                format!("0h{:016x}", magnitude.to_bits())
            } else {
                let narrow = magnitude as f32;
                if f64::from(narrow).to_bits() != magnitude.to_bits() {
                    return Err(Trap::new(
                        "edited float literal is not representable at its declared precision",
                    ));
                }
                format!("0h{:08x}", narrow.to_bits())
            };
            return Ok(format!(
                "{}{text}",
                if negative {
                    "-"
                } else {
                    ""
                }
            ));
        }
        if value.is_nan() {
            return Err(Trap::new("cannot serialize a decimal NaN literal"));
        }
        let wanted = if self.has(at, ty, "value_flags", "REQUIRES_FLOAT64")? {
            9
        } else if self.has(at, ty, "value_flags", "DEFAULTS_TO_FLOAT64")? {
            8
        } else {
            0
        };
        let sign = if value.is_sign_negative() {
            "-"
        } else {
            ""
        };
        if value.is_infinite() {
            let digits = if wanted == 9 {
                "1.000000001"
            } else {
                "1.0"
            };
            return Ok(format!("{sign}{digits}e999"));
        }
        if value == 0.0 && wanted > 0 {
            return Ok(format!("{sign}0.{}1e-999", "0".repeat(wanted - 1)));
        }
        let scientific = format!("{:.16e}", value.abs());
        let (coefficient, exponent) = scientific
            .split_once('e')
            .expect("scientific formatting has an exponent");
        let fraction = coefficient
            .split_once('.')
            .map_or(0, |(_, part)| part.len());
        let mut exponent = exponent
            .parse::<i32>()
            .expect("scientific formatting has a numeric exponent")
            - fraction as i32;
        let mut digits = coefficient.replace('.', "");
        while digits.len() > 1 && digits.ends_with('0') {
            digits.pop();
            exponent += 1;
        }
        let text = if wanted == 0 {
            format!("{digits}.0")
        } else if digits.len() <= wanted {
            format!("0.{}{digits}", "0".repeat(wanted - digits.len()))
        } else {
            let split = digits.len() - wanted;
            format!("{}.{}", &digits[..split], &digits[split..])
        };
        Ok(format!("{sign}{text}e{}", exponent + wanted as i32))
    }

    fn binary(&mut self, at: u64, ty: TypeId) -> R<ExprKind> {
        let op = self.num(at, ty, "operator_type")?;
        let left = self.ptr(at, ty, "left")?;
        if op == u64::from(b'.') {
            if left == 0 {
                self.word(".");
                let child = self.ptr(at, ty, "right")?;
                return Ok(ExprKind::InferredMember(self.ident_name(child)?));
            }
            let base = self.expr(left)?;
            self.word(".");
            let child = self.ptr(at, ty, "right")?;
            let member = self.ident_name(child)?;
            return Ok(ExprKind::Member(Box::new(base), member));
        }
        let left = self.expr(left)?;
        if op == 500 {
            self.word("[");
            let index = self.child(at, ty, "right")?;
            self.word("]");
            return Ok(ExprKind::Index(Box::new(left), Box::new(index)));
        }
        if super::stmt::assign_op(op).is_some() {
            return Err(Trap::new("an assignment cannot be used as an expression"));
        }
        let (op, symbol) = bin_op(op)?;
        self.word(symbol);
        let right = self.child(at, ty, "right")?;
        Ok(ExprKind::Binary(op, Box::new(left), Box::new(right)))
    }

    fn unary(&mut self, at: u64, ty: TypeId) -> R<ExprKind> {
        let op = match self.num(at, ty, "operator_type")? {
            45 => UnOp::Neg,
            43 => UnOp::Plus,
            33 => UnOp::Not,
            126 => UnOp::BitNot,
            42 => UnOp::Star,
            137 | 168 | 169 => UnOp::Deref,
            value => return Err(Trap::new(format!("unsupported operator {value}"))),
        };
        if op != UnOp::Deref {
            self.word(match op {
                UnOp::Neg => "-",
                UnOp::Plus => "+",
                UnOp::Not => "!",
                UnOp::BitNot => "~",
                _ => "*",
            });
        }
        let value = self.child(at, ty, "subexpression")?;
        if op == UnOp::Deref {
            self.word(".*");
        }
        Ok(ExprKind::Unary(op, Box::new(value)))
    }

    /// A call's callee, arguments (with the `,,` context changes last) and inline hint.
    pub(super) fn call(&mut self, at: u64, ty: TypeId) -> R<(Expr, Vec<ast::Arg>, CallHint)> {
        let hint = match self.int_attribute(at, SyntaxKey::CallHint)? {
            1 => {
                self.word("inline");
                CallHint::Inline
            }
            2 => {
                self.word("no_inline");
                CallHint::NoInline
            }
            _ => CallHint::None,
        };
        let callee = self.child(at, ty, "procedure_expression")?;
        self.word("(");
        let mut args = self.arguments(at, ty, "arguments_unsorted")?;
        let changes = self.extra_list(at, SyntaxKey::ContextArguments)?;
        let names = self.attribute(at, SyntaxKey::ContextNames)?;
        let names = names.split(',').map(str::to_owned).collect::<Vec<_>>();
        for (index, change) in changes.into_iter().enumerate() {
            self.word(",,");
            let name = match names.get(index) {
                Some(name) if !name.is_empty() => {
                    let name = self.name(name);
                    self.word("=");
                    Some(name)
                }
                _ => None,
            };
            args.push(ast::Arg {
                name,
                target: None,
                context: true,
                spread: false,
                value: self.expr(change)?,
            });
        }
        self.word(")");
        Ok((callee, args, hint))
    }

    fn cast(&mut self, at: u64, ty: TypeId) -> R<ExprKind> {
        let flags = self.num(at, ty, "cast_flags")?;
        let target = if flags & 4 != 0 {
            self.word("xx");
            None
        } else {
            self.word("cast");
            Some(Box::new(self.child(at, ty, "target_type")?))
        };
        let value = self.child(at, ty, "expression")?;
        Ok(ExprKind::Cast {
            ty: target,
            value: Box::new(value),
            flags: CastFlags {
                no_check: flags & 8 != 0,
                truncate: flags & 16 != 0,
                force: flags & 128 != 0,
            },
        })
    }

    /// An `ifx` branch: a block of one value is that value.
    fn value_block(&mut self, at: u64, ty: TypeId, field: &str) -> R<Option<Expr>> {
        let block = self.ptr(at, ty, field)?;
        if block == 0 {
            return Ok(None);
        }
        let statements = self.list(block, self.ty("Code_Block")?, "statements")?;
        if let [value] = statements[..] {
            return self.expr(value).map(Some);
        }
        self.expr(block).map(Some)
    }

    /// A type instantiation's type expression.
    pub(super) fn type_inst(&mut self, at: u64, ty: TypeId) -> R<Expr> {
        let start = self.mark();
        let flags = self.num(at, ty, "inst_flags")?;
        let modifier = if flags & 32 != 0 {
            Some(TypeModifier::Isa)
        } else if flags & 16 != 0 {
            Some(TypeModifier::Distinct)
        } else if flags & 8 != 0 {
            Some(TypeModifier::Plain)
        } else {
            None
        };
        if modifier.is_some() {
            self.word("#type");
        }
        let pointer = self.ptr(at, ty, "pointer_to")?;
        let element = self.ptr(at, ty, "array_element_type")?;
        let target = self.ptr(at, ty, "type_directive_target")?;
        let value = if pointer != 0 {
            self.word("*");
            let pointee = self.expr(pointer)?;
            mk(
                ExprKind::Unary(UnOp::Star, Box::new(pointee)),
                self.since(start),
            )
        } else if element != 0 {
            self.word("[");
            let size = if flags & 2 != 0 {
                self.word("..");
                ArraySize::Resizable
            } else {
                match self.opt_expr(at, ty, "array_dimension")? {
                    Some(dimension) => ArraySize::Fixed(Box::new(dimension)),
                    None => ArraySize::View,
                }
            };
            self.word("]");
            let elem = self.expr(element)?;
            mk(
                ExprKind::ArrayType {
                    size,
                    elem: Box::new(elem),
                },
                self.since(start),
            )
        } else if target != 0 {
            self.expr(target)?
        } else {
            let value = self.ptr(at, ty, "type_valued_expression")?;
            if value == 0 {
                return Err(Trap::new("a type instantiation needs a type"));
            }
            self.expr(value)?
        };
        Ok(match modifier {
            Some(modifier) => mk(
                ExprKind::TypeDirective {
                    modifier,
                    ty: Box::new(value),
                },
                self.since(start),
            ),
            None => value,
        })
    }

    /// A `PLACEHOLDER` node in value position: `#procedure_of_call` or an unknown directive.
    fn placeholder(&mut self, at: u64) -> R<ExprKind> {
        if !self.attribute(at, SyntaxKey::AsmKind)?.is_empty() {
            return Err(Trap::new("an assembly operand cannot be used outside #asm"));
        }
        let name = self.attribute(at, SyntaxKey::DirectiveName)?;
        if name.is_empty() {
            return Err(Trap::new("a placeholder node needs a directive name"));
        }
        if name == "procedure_of_call" {
            self.word("#procedure_of_call");
            let call = self.extra(at, SyntaxKey::DirectiveArgs)?;
            return Ok(ExprKind::ProcedureOfCall(Box::new(self.expr(call)?)));
        }
        let span = self.word(&format!("#{name}"));
        let flags = self.flag_names(at, SyntaxKey::Flags)?;
        let operand = self.extra_expr(at, SyntaxKey::DirectiveArgs)?;
        Ok(ExprKind::UnknownDirective(Box::new(UnknownDirective {
            name: ast::Ident {
                name: Sym::intern(&name),
                span,
            },
            kind: DirectiveKind::of(&name),
            flags,
            operand,
        })))
    }
}

/// The operator of a `BINARY_OPERATOR` that is not an assignment, member or index.
fn bin_op(value: u64) -> R<(BinOp, &'static str)> {
    Ok(match value {
        43 => (BinOp::Add, "+"),
        45 => (BinOp::Sub, "-"),
        42 => (BinOp::Mul, "*"),
        47 => (BinOp::Div, "/"),
        37 => (BinOp::Rem, "%"),
        38 => (BinOp::BitAnd, "&"),
        124 => (BinOp::BitOr, "|"),
        94 => (BinOp::BitXor, "^"),
        60 => (BinOp::Lt, "<"),
        62 => (BinOp::Gt, ">"),
        131 => (BinOp::Eq, "=="),
        132 => (BinOp::Ne, "!="),
        133 => (BinOp::And, "&&"),
        134 => (BinOp::Or, "||"),
        135 => (BinOp::Le, "<="),
        136 => (BinOp::Ge, ">="),
        137 | 168 => (BinOp::Shl, "<<"),
        138 => (BinOp::Shr, ">>"),
        139 => (BinOp::Rotl, "<<<"),
        140 => (BinOp::Rotr, ">>>"),
        _ => return Err(Trap::new(format!("unsupported operator {value}"))),
    })
}
