//! Turn editable public code records back into syntax trees (`compiler_get_code`,
//! `compiler_modify_procedure`).
//!
//! The trees get spans into a text spelled while converting: identifiers, literals as the
//! lexer would read them, and operator symbols, in source order. Diagnostics quote it, and
//! sema reads a literal's spelling for its precision. Every value read is also logged, so a
//! compile-time run started again can tell it made the same code.
mod decl;
mod directive;
mod expr;
mod stmt;

use super::{
    Kind,
    abi::{bytes, view},
    operations::Call,
    records,
};
use crate::{
    ast::{self, CodeBody, Ident, Note, relocate::PLACEHOLDER_FILE},
    builtin_keys::SyntaxKey,
    intern::Sym,
    interp::{Interp, MadeCode, Rets, Trap},
    records::{FieldRef, Item},
    source::Span,
    types::TypeId,
};
use std::rc::Rc;

type R<T> = Result<T, Trap>;

/// The node kinds that are statements, never values.
const STATEMENT_KINDS: &[&str] = &[
    "DECLARATION",
    "COMPOUND_DECLARATION",
    "RETURN",
    "WHILE",
    "FOR",
    "CASE",
    "DEFER",
    "LOOP_CONTROL",
    "USING",
    "PUSH_CONTEXT",
    "DIRECTIVE_IMPORT",
    "DIRECTIVE_LOAD",
    "DIRECTIVE_SCOPE",
    "DIRECTIVE_THROUGH",
    "DIRECTIVE_PLACE",
    "DIRECTIVE_OVERLAY",
    "DIRECTIVE_ADD_CONTEXT",
    "DIRECTIVE_MODULE_PARAMETERS",
    "DIRECTIVE_MODIFY",
    "NOTE",
    "COMMA_SEPARATED_ARGUMENTS",
];

pub(super) fn get_code(cx: &Call, interp: &mut Interp) -> R<Rets> {
    let scope_from = cx.arg(1) as usize;
    if scope_from >= interp.codes.len() {
        return Err(Trap::new(
            "compiler_get_code: no code to take the scope from",
        ));
    }
    let mut c = Convert::new(cx, interp);
    let body = c.code_body(cx.arg(0))?;
    let (source, fingerprint) = c.finish();
    // A compile-time run that asks for typed nodes is run again from the start
    // (`export_request`), and does what the earlier attempt did: its n-th code is the
    // n-th code that attempt made, or it would ask for that new code's nodes again.
    let (made, replayed) = &mut interp.run_codes;
    if let Some(&id) = made.get(*replayed)
        && interp
            .made_codes
            .iter()
            .any(|m| m.id == id && m.scope_from == scope_from && *m.fingerprint == *fingerprint)
    {
        *replayed += 1;
        return Ok(Rets::one(id as u64));
    }
    let id = interp.codes.len();
    interp.codes.push(Rc::new(body));
    interp.made_codes.push(MadeCode {
        id,
        scope_from,
        source,
        fingerprint,
    });
    let (made, replayed) = &mut interp.run_codes;
    made.truncate(*replayed);
    made.push(id);
    *replayed = made.len();
    Ok(Rets::one(id as u64))
}

pub(super) fn modify(cx: &Call, interp: &mut Interp) -> R<Rets> {
    let body = records::record_id(cx, interp, cx.arg(1))?;
    let workspace = cx.workspace(0)?;
    let ty = cx.schema().pointed(cx.binding.parameters[1])?;
    let mut c = Convert::new(cx, interp);
    let block = c.ptr(cx.arg(1), ty, "block")?;
    if block == 0 {
        return Err(Trap::new(
            "compiler_modify_procedure requires a procedure body and block",
        ));
    }
    let stmts = c.statements(block)?;
    let (source, _) = c.finish();
    cx.shared
        .borrow_mut()
        .ws(workspace)
        .map_err(Trap::new)?
        .modifications
        .push(crate::sema::Modification {
            body,
            stmts,
            source,
        });
    Ok(Rets::default())
}

struct Convert<'a, 'b> {
    cx: &'a Call<'b>,
    interp: &'a mut Interp,
    depth: usize,
    /// The spelled source the spans point into.
    text: String,
    /// Every value read, in order.
    print: Vec<u8>,
}

impl<'a, 'b> Convert<'a, 'b> {
    fn new(cx: &'a Call<'b>, interp: &'a mut Interp) -> Self {
        Convert {
            cx,
            interp,
            depth: 0,
            text: String::new(),
            print: Vec::new(),
        }
    }

    fn finish(self) -> (Rc<str>, Box<[u8]>) {
        (self.text.into(), self.print.into())
    }

    fn ty(&self, name: &str) -> R<TypeId> {
        self.cx.schema().named(name)
    }

    // Reading records. Pointers are logged by presence only: a rerun places records elsewhere.

    fn num(&mut self, at: u64, ty: TypeId, field: &str) -> R<u64> {
        let value = self.cx.number(self.interp, ty, at, field)?;
        self.print.extend_from_slice(&value.to_le_bytes());
        Ok(value)
    }

    fn ptr(&mut self, at: u64, ty: TypeId, field: &str) -> R<u64> {
        let value = self.cx.number(self.interp, ty, at, field)?;
        self.print.push(u8::from(value != 0));
        Ok(value)
    }

    fn raw(&mut self, at: u64, ty: TypeId, field: &str) -> R<Vec<u8>> {
        let (count, data) = view(self.interp, self.cx.field(ty, at, field)?.0)?;
        let size = usize::try_from(count).map_err(|_| Trap::new("invalid syntax string size"))?;
        let raw = bytes(self.interp, data, size)?;
        self.print.extend_from_slice(&count.to_le_bytes());
        self.print.extend_from_slice(&raw);
        Ok(raw)
    }

    fn text(&mut self, at: u64, ty: TypeId, field: &str) -> R<String> {
        Ok(String::from_utf8_lossy(&self.raw(at, ty, field)?).into_owned())
    }

    fn list(&mut self, at: u64, ty: TypeId, field: &str) -> R<Vec<u64>> {
        let (address, array) = self.cx.field(ty, at, field)?;
        let Kind::Array {
            element,
            count,
            ..
        } = self.cx.schema().get(array)?.kind
        else {
            return Err(Trap::new("syntax field must be an array"));
        };
        let (count, data) = match count {
            Some(count) => (count, address),
            None => view(self.interp, address)?,
        };
        let size = self.cx.schema().get(element)?.size;
        if count
            .checked_mul(size)
            .is_none_or(|bytes| bytes > crate::sema::value::MAX_IMAGE)
        {
            return Err(Trap::new("syntax array size overflow"));
        }
        self.print.extend_from_slice(&count.to_le_bytes());
        let pointers = matches!(self.cx.schema().get(element)?.kind, Kind::Pointer(_));
        (0..count)
            .map(|index| {
                let at = data + index * size;
                if pointers {
                    self.cx.schema().read_number(self.interp, element, at)
                } else {
                    Ok(at)
                }
            })
            .collect()
    }

    fn enum_value(&self, ty: TypeId, name: &str) -> R<u64> {
        self.cx
            .schema()
            .enums
            .get(&ty)
            .and_then(|members| members.iter().find(|(member, _)| member == name))
            .map(|(_, value)| *value as u64)
            .ok_or_else(|| Trap::new(format!("compiler syntax enum has no member {name}")))
    }

    fn has(&mut self, at: u64, ty: TypeId, field: &str, name: &str) -> R<bool> {
        let (_, kind) = self.cx.schema().field(ty, field)?;
        Ok(self.num(at, ty, field)? & self.enum_value(kind, name)? != 0)
    }

    fn enum_is(&mut self, at: u64, ty: TypeId, field: &str, name: &str) -> R<bool> {
        let (_, kind) = self.cx.schema().field(ty, field)?;
        Ok(self.num(at, ty, field)? == self.enum_value(kind, name)?)
    }

    /// A syntax attribute: what the exporter kept of the source beside the public fields.
    fn syntax(&mut self, at: u64, key: SyntaxKey) -> R<Item> {
        let id = records::record_of(self.cx, self.interp, at)?;
        let item = self
            .cx
            .shared
            .borrow()
            .records
            .syntax_field(id, key)
            .and_then(FieldRef::item)
            .unwrap_or(Item::Int(0));
        match &item {
            Item::Int(value) => self.print.extend_from_slice(&value.to_le_bytes()),
            Item::Str(text) => {
                self.print.extend_from_slice(&text.len().to_le_bytes());
                self.print.extend_from_slice(text);
            }
            Item::Ref(id) => self.print.push(u8::from(*id != 0)),
        }
        Ok(item)
    }

    fn flag(&mut self, at: u64, key: SyntaxKey) -> R<bool> {
        Ok(matches!(self.syntax(at, key)?, Item::Int(value) if value != 0))
    }

    fn int_attribute(&mut self, at: u64, key: SyntaxKey) -> R<i64> {
        Ok(match self.syntax(at, key)? {
            Item::Int(value) => value,
            _ => 0,
        })
    }

    fn attribute(&mut self, at: u64, key: SyntaxKey) -> R<String> {
        Ok(match self.syntax(at, key)? {
            Item::Str(text) => String::from_utf8_lossy(&text).into_owned(),
            _ => String::new(),
        })
    }

    /// A node kept as an attribute, materialized for the caller's layouts (0 when absent).
    fn extra(&mut self, at: u64, key: SyntaxKey) -> R<u64> {
        let id = match self.syntax(at, key)? {
            Item::Ref(id) => id,
            _ => 0,
        };
        if id == 0 {
            return Ok(0);
        }
        let node = self.ty("Code_Node")?;
        records::build(self.cx, self.interp, id, node)
    }

    /// The nodes of an attribute holding one node or a `COMMA_SEPARATED_ARGUMENTS` of them.
    fn extra_list(&mut self, at: u64, key: SyntaxKey) -> R<Vec<u64>> {
        let child = self.extra(at, key)?;
        if child == 0 {
            return Ok(Vec::new());
        }
        self.comma_nodes(child)
    }

    /// `at`'s nodes when it is a `COMMA_SEPARATED_ARGUMENTS`, else `at` alone.
    fn comma_nodes(&mut self, at: u64) -> R<Vec<u64>> {
        let (name, ty) = self.node_type(at)?;
        if name != "COMMA_SEPARATED_ARGUMENTS" {
            return Ok(vec![at]);
        }
        let argument = self.ty("Code_Comma_Separated_Argument")?;
        self.list(at, ty, "arguments")?
            .into_iter()
            .map(|arg| self.ptr(arg, argument, "node"))
            .collect()
    }

    // Spelling.

    fn mark(&self) -> usize {
        self.text.len()
    }

    /// The span from `start` to the last word spelled.
    fn since(&self, start: usize) -> Span {
        let end = self.text.trim_end().len().max(start);
        Span::new(PLACEHOLDER_FILE, start, end)
    }

    fn word(&mut self, word: &str) -> Span {
        let start = self.mark();
        self.text.push_str(word);
        let span = self.since(start);
        self.text.push(' ');
        span
    }

    fn name(&mut self, name: &str) -> Ident {
        Ident {
            name: Sym::intern(name),
            span: self.word(name),
        }
    }

    fn idents(&mut self, names: impl IntoIterator<Item = impl AsRef<str>>) -> Vec<Ident> {
        names
            .into_iter()
            .map(|name| self.name(name.as_ref()))
            .collect()
    }

    /// `" #a #b"` or `",a,b"`: the flag names of an attribute.
    fn flag_names(&mut self, at: u64, key: SyntaxKey) -> R<Vec<Ident>> {
        let text = self.attribute(at, key)?;
        let names = text
            .split([' ', ',', '#'])
            .filter(|name| !name.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>();
        Ok(self.idents(names))
    }

    // Nodes.

    fn node_type(&mut self, at: u64) -> R<(String, TypeId)> {
        let base = self.ty("Code_Node")?;
        let kind = self.num(at, base, "kind")?;
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
        Ok((name, self.ty(&class)?))
    }

    /// Converts the node at `at` with `f`, after the checks every node gets.
    fn node<T>(&mut self, at: u64, f: impl FnOnce(&mut Self, &str, TypeId) -> R<T>) -> R<T> {
        if at == 0 {
            return Err(Trap::new("cannot serialize a null syntax node"));
        }
        if self.depth >= 512 {
            return Err(Trap::new("syntax serialization nesting limit exceeded"));
        }
        let (name, ty) = self.node_type(at)?;
        let original = self.int_attribute(at, SyntaxKey::OriginalKind)?;
        let kind = self.num(at, self.ty("Code_Node")?, "kind")?;
        if original != 0 && original as u64 != kind {
            return Err(Trap::new(
                "cannot serialize a node whose kind was changed without rebuilding its record",
            ));
        }
        if self.flag(at, SyntaxKey::Incomplete)? {
            return Err(Trap::new("cannot serialize incomplete syntax"));
        }
        self.depth += 1;
        let result = f(self, &name, ty);
        self.depth -= 1;
        result
    }

    /// The root of a `Code` value: a block, an expression or a statement.
    fn code_body(&mut self, at: u64) -> R<CodeBody> {
        if at != 0 && self.node_type(at)?.0 == "BLOCK" {
            return Ok(CodeBody::Block(self.block(at)?));
        }
        let stmt = self.stmt(at)?;
        Ok(match stmt.kind {
            ast::StmtKind::Expr(e) if stmt.notes.is_empty() => CodeBody::Expr(e),
            _ => CodeBody::Stmt(stmt),
        })
    }

    fn block(&mut self, at: u64) -> R<ast::Block> {
        self.node(at, |c, name, ty| {
            if name != "BLOCK" {
                return Err(Trap::new(format!("expected a block, got {name}")));
            }
            c.block_node(at, ty)
        })
    }

    /// A `BLOCK` node already checked by `node`.
    fn block_node(&mut self, at: u64, ty: TypeId) -> R<ast::Block> {
        let start = self.mark();
        self.word("{");
        let mut checks_off = ast::CheckBits::NONE;
        if self.has(at, ty, "block_flags", "NO_ARRAY_BOUNDS_CHECK")? {
            checks_off = checks_off.with(ast::CheckBits::NO_ABC);
        }
        if self.has(at, ty, "block_flags", "NO_ARITHMETIC_OVERFLOW_CHECK")? {
            checks_off = checks_off.with(ast::CheckBits::NO_AOC);
        }
        let stmts = self.statements(at)?;
        self.word("}");
        Ok(ast::Block {
            stmts,
            span: self.since(start),
            checks_off,
        })
    }

    fn statements(&mut self, block: u64) -> R<Vec<ast::Stmt>> {
        let ty = self.ty("Code_Block")?;
        self.list(block, ty, "statements")?
            .into_iter()
            .map(|stmt| self.stmt(stmt))
            .collect()
    }

    fn notes(&mut self, at: u64, ty: TypeId) -> R<Vec<Note>> {
        let notes = self.list(at, ty, "notes")?;
        notes.into_iter().map(|note| self.note(note)).collect()
    }

    fn note(&mut self, at: u64) -> R<Note> {
        self.node(at, |c, name, ty| {
            if name != "NOTE" {
                return Err(Trap::new(format!("expected a note, got {name}")));
            }
            let text = c.text(at, ty, "text")?;
            let span = c.word(&format!("@{text}"));
            Ok(Note {
                text: text.into(),
                span,
            })
        })
    }

    /// An identifier node's name.
    fn ident_name(&mut self, at: u64) -> R<Ident> {
        self.node(at, |c, name, ty| {
            if name != "IDENT" {
                return Err(Trap::new(format!("expected an identifier, got {name}")));
            }
            let text = c.text(at, ty, "name")?;
            Ok(c.name(&text))
        })
    }

    /// The expression a node field points to.
    fn child(&mut self, at: u64, ty: TypeId, field: &str) -> R<ast::Expr> {
        let child = self.ptr(at, ty, field)?;
        self.expr(child)
    }

    fn opt_name(&mut self, at: u64, ty: TypeId, field: &str) -> R<Option<Ident>> {
        let child = self.ptr(at, ty, field)?;
        if child == 0 {
            return Ok(None);
        }
        self.ident_name(child).map(Some)
    }

    /// `Code_Argument`s: calls, returns and import parameters.
    fn arguments(&mut self, at: u64, ty: TypeId, field: &str) -> R<Vec<ast::Arg>> {
        let argument = self.ty("Code_Argument")?;
        let node = self.ty("Code_Node")?;
        let arguments = self.list(at, ty, field)?;
        let mut result = Vec::new();
        for (index, arg) in arguments.into_iter().enumerate() {
            if index > 0 {
                self.word(",");
            }
            let name = self.opt_name(arg, argument, "name")?;
            if name.is_some() {
                self.word("=");
            }
            let value = self.ptr(arg, argument, "expression")?;
            let spread =
                value != 0 && self.has(value, node, "node_flags", "EXPRESSION_IS_SPREAD")?;
            if spread {
                self.word("..");
            }
            result.push(ast::Arg {
                name,
                target: None,
                context: false,
                spread,
                value: self.expr(value)?,
            });
        }
        Ok(result)
    }
}

/// A string literal's spelling.
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
