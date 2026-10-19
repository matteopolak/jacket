//! Exporting declarations, enums and struct literals.
use super::Exporter;
use super::constants::node;
use crate::builtin_keys::SyntaxKey;
use crate::records::{FieldRef, Item, Record, RecordTag};
use crate::sema::scope::Resolved;
use crate::sema::*;
use crate::types::TypeKind;

impl Exporter<'_> {
    /// A declaration inside a body or struct, with its type once the body was lowered.
    pub(super) fn local_decl(&mut self, d: &ast::Decl) -> i64 {
        if d.names.len() > 1 {
            return self.compound_decl(d);
        }
        let name = d.names.first().map_or(Sym::intern(""), |n| n.name);
        let ty = self
            .c
            .as_deref()
            .and_then(|c| c.local_decl_types.get(&(d.id, 0)).copied());
        let rec = self.decl(d, name, ty, None, true, 0);
        // `x := value`: the value has the declared type (its own typing may not see locals).
        if d.ty.is_none()
            && let Some(FieldRef::Ref(ty)) = self.r.field(rec, "type")
            && let Some(FieldRef::Ref(value)) = self.r.field(rec, "expression")
            && value != 0
            && let Some(value) = self.r.get_mut(value)
            && value
                .field("type")
                .is_none_or(|t| matches!(t, FieldRef::Ref(0)))
        {
            value.ptr("type", ty);
        }
        for n in &d.names {
            self.locals.push((n.name, rec));
        }
        rec
    }

    /// `a, b := f();` / `a, b: int = 1, 2;` / `a=, b := f();`: the names (declarations, or
    /// identifiers for `=` targets) in `comma_separated_assignment`, the shared type and
    /// values in a nameless `declaration_properties`.
    fn compound_decl(&mut self, d: &ast::Decl) -> i64 {
        let values = if d.extra_values().is_empty() {
            None
        } else {
            let mut ids = Vec::new();
            for v in d.value.iter().chain(d.extra_values()) {
                let node = self.expr(v);
                let mut a = Record::new(RecordTag::CodeCommaSeparatedArgument);
                a.ptr("node", node);
                ids.push(self.r.add(a));
            }
            let mut rec = self.node(
                RecordTag::CodeCommaSeparatedArguments,
                node::COMMA_SEPARATED_ARGUMENTS,
                d.span,
            );
            rec.refs("arguments", ids);
            Some(self.add(rec))
        };
        // Before the names are in scope: the values may mention variables they shadow.
        let properties = self.decl(d, Sym::intern(""), None, values, false, 0);
        let mut arguments = Vec::new();
        let mut declared = Vec::new();
        for (i, n) in d.names.iter().enumerate() {
            let assign = d.existing().get(i).copied().unwrap_or(false);
            let target = if let Some(Some(place)) = d.targets().get(i) {
                self.expr(place)
            } else if assign {
                self.ident(n.name, n.span)
            } else {
                let ty = self
                    .c
                    .as_deref()
                    .and_then(|c| c.local_decl_types.get(&(d.id, i)).copied())
                    .map_or(0, |t| self.ty(t));
                let mut rec = self.node(RecordTag::CodeDeclaration, node::DECLARATION, n.span);
                let backticked = d.backtick || d.backtick_names().get(i).copied().unwrap_or(false);
                rec.str("name", n.name.as_str().as_bytes())
                    .ptr("type", ty)
                    .int(
                        "flags",
                        i64::from(d.kind == ast::DeclKind::Const)
                            | if backticked {
                                0x20_0000
                            } else {
                                0
                            },
                    );
                let id = self.add(rec);
                declared.push((n.name, id));
                id
            };
            let mut a = Record::new(RecordTag::CodeCommaSeparatedArgument);
            a.ptr("node", target).int(
                "modifier",
                if assign {
                    2
                } else {
                    1
                },
            );
            arguments.push(self.r.add(a));
        }
        self.compound_members = declared.iter().map(|&(_, id)| id).collect();
        self.locals.extend(declared);
        let mut names = self.node(
            RecordTag::CodeCommaSeparatedArguments,
            node::COMMA_SEPARATED_ARGUMENTS,
            d.span,
        );
        names.refs("arguments", arguments);
        let names = self.add(names);
        let mut rec = self.node(
            RecordTag::CodeCompoundDeclaration,
            node::COMPOUND_DECLARATION,
            d.span,
        );
        rec.ptr("comma_separated_assignment", names)
            .ptr("declaration_properties", properties)
            .int("node_flags", 0x4);
        self.add(rec)
    }

    pub(super) fn decl(
        &mut self,
        d: &ast::Decl,
        name: Sym,
        ty: Option<TypeId>,
        expression: Option<i64>,
        statement: bool,
        extra_flags: i64,
    ) -> i64 {
        let type_inst = d.ty.as_ref().map_or(0, |t| self.type_inst(t));
        let expression = match expression {
            Some(e) => e,
            None => match &d.value {
                Some(v) => match &v.kind {
                    ast::ExprKind::Proc(lit) => self.proc(lit, &d.notes, None, v.span),
                    _ => self.expr(v),
                },
                None => 0,
            },
        };
        let notes: Vec<&ast::Note> = d.notes.iter().collect();
        let notes = self.notes(&notes);
        let mut flags = extra_flags;
        if d.kind == ast::DeclKind::Const {
            flags |= 0x1;
        }
        if d.as_ {
            flags |= 0x2;
        }
        if matches!(&d.value, Some(v) if matches!(v.kind, ast::ExprKind::Uninit)) {
            flags |= 0x80;
        }
        let backticked = d.backtick
            || d.names
                .iter()
                .zip(d.backtick_names())
                .any(|(n, &b)| b && n.name == name);
        if backticked {
            flags |= 0x0020_0000; // HAS_SCOPE_MODIFIER
        }
        let ty = ty.map_or(0, |t| self.ty(t));
        let alignment = self.opt_expr(d.align());
        let mut rec = self.node(RecordTag::CodeDeclaration, node::DECLARATION, d.span);
        rec.str("name", name.as_str().as_bytes())
            .ptr("type", ty)
            .ptr("type_inst", type_inst)
            .ptr("expression", expression)
            .int("flags", flags)
            .refs("notes", notes);
        rec.ptr("alignment_expression", alignment)
            .int(SyntaxKey::Using, i64::from(d.using))
            .str(
                SyntaxKey::Flags,
                d.flags()
                    .iter()
                    .map(|flag| format!(" #{}", flag.name))
                    .collect::<String>()
                    .as_bytes(),
            );
        if let Some(filter) = d.using_filter() {
            rec.str(SyntaxKey::Filter, Self::filter_syntax(filter).as_bytes());
            let (kind, expression) = self.using_filter(filter);
            rec.int(SyntaxKey::FilterKind, kind)
                .ptr(SyntaxKey::FilterExpression, expression);
        }
        if let Some(tag) = d.union_tag() {
            let tag = self.expr(tag);
            rec.ptr(SyntaxKey::UnionTag, tag);
        }
        if let Some(foreign) = d.foreign() {
            let library = foreign
                .library
                .map_or(0, |name| self.name_ident(name.name, name.span));
            rec.int(SyntaxKey::Foreign, 1)
                .ptr(SyntaxKey::ForeignLibrary, library);
            if let ast::ForeignName::Named(symbol) = &foreign.name {
                rec.str(SyntaxKey::ForeignSymbol, symbol.as_bytes())
                    .int(SyntaxKey::ForeignNamed, 1);
            }
        }
        if statement {
            rec.int("node_flags", 0x4); // ALLOWED_BY_CONTEXT
        }
        self.add(rec)
    }

    /// A top-level declaration with its resolved type.
    pub(super) fn global_decl(
        &mut self,
        d: &ast::Decl,
        index: usize,
        resolved: &Resolved,
        file_private: bool,
    ) -> i64 {
        let name = d.names[index].name;
        let span = d.span;
        let mut alias_expression = None;
        let (ty, expression) = match resolved {
            Resolved::Proc(p)
            | Resolved::Const {
                value: Value::Proc(p),
                ..
            } => {
                let p = *p;
                let c = self.c.as_deref_mut().unwrap();
                let sig = c.signature(p, span).ok();
                let lit = c.proc(p).lit.clone();
                let header = self.proc(&lit, &d.notes, Some((p, sig.clone())), span);
                if let Some(value) = &d.value
                    && !matches!(value.kind, ast::ExprKind::Proc(_))
                {
                    alias_expression = Some((header, self.expr(value)));
                }
                let ty = match sig {
                    Some(s) => Some(s.ty),
                    // A polymorphic procedure's type shows `$` where a type variable decides.
                    None if self.c.as_deref().is_some_and(|c| c.proc(p).is_poly) => {
                        self.c.as_deref_mut().map(|c| c.poly_proc_type(p))
                    }
                    None => None,
                };
                (ty, Some(header))
            }
            Resolved::Const {
                value: Value::Type(t),
                ty,
            } => {
                let expression = match d.value.as_ref().map(|v| &v.kind) {
                    Some(ast::ExprKind::Struct(lit)) => Some(self.struct_lit(lit, Some(*t), name)),
                    Some(ast::ExprKind::Enum(lit)) => Some(self.enum_lit(lit, Some(*t), span)),
                    _ => None,
                };
                (Some(*ty), expression)
            }
            Resolved::PolyStruct(_) => {
                let expression = match d.value.as_ref().map(|v| &v.kind) {
                    Some(ast::ExprKind::Struct(lit)) => Some(self.struct_lit(lit, None, name)),
                    _ => None,
                };
                (Some(TypeId::TYPE), expression)
            }
            Resolved::Const {
                ty, ..
            }
            | Resolved::Global {
                ty, ..
            } => (Some(*ty), None),
            _ => (None, None),
        };
        // IS_GLOBAL, and SCOPE_FILE below a `#scope_file`.
        let scope = 0x0010_0000
            | if file_private {
                0x80000
            } else {
                0
            };
        let declaration = self.decl(d, name, ty, expression, false, scope);
        if let Some((initial, alias)) = alias_expression {
            self.r
                .get_mut(declaration)
                .unwrap()
                .ptr(SyntaxKey::InitialExpression, initial)
                .ptr(SyntaxKey::AliasExpression, alias);
        }
        declaration
    }

    /// An enum literal; `external_type` is its type when the declaration resolved to one.
    pub(super) fn enum_lit(
        &mut self,
        lit: &ast::EnumLit,
        defined: Option<TypeId>,
        span: Span,
    ) -> i64 {
        let notes: Vec<&ast::Note> = lit.notes.iter().collect();
        let notes = self.notes(&notes);
        let external = defined
            .filter(|&t| {
                self.c
                    .as_deref()
                    .is_some_and(|c| matches!(c.types.kind(t), TypeKind::Enum(_)))
            })
            .map_or(0, |t| self.ty(t));
        let block = self.enum_block(&lit.items, span);
        let base = lit.base.as_ref().map_or(0, |base| self.type_inst(base));
        let mut rec = self.node(RecordTag::CodeEnum, node::ENUM, span);
        rec.refs("notes", notes)
            .ptr("block", block)
            .ptr("internal_type_inst", base)
            .ptr("external_type", external)
            .int("marked_as_complete", lit.complete as i64)
            .int("marked_as_specified", lit.specified as i64)
            .int("is_flags", lit.flags_enum as i64);
        self.add(rec)
    }

    fn enum_block(&mut self, items: &[ast::EnumItem], span: Span) -> i64 {
        let statements: Vec<_> = items
            .iter()
            .map(|item| match item {
                ast::EnumItem::Member(member) => {
                    let value = self.opt_expr(member.value.as_ref());
                    let notes = self.notes(&member.notes.iter().collect::<Vec<_>>());
                    let mut rec = self.node(
                        RecordTag::CodeDeclaration,
                        node::DECLARATION,
                        member.name.span,
                    );
                    rec.str("name", member.name.name.as_str().as_bytes())
                        .ptr("expression", value)
                        .refs("notes", notes)
                        .int("flags", 1);
                    self.add(rec)
                }
                ast::EnumItem::Insert(value) => {
                    let value = self.expr(value);
                    let mut rec =
                        self.node(RecordTag::CodeDirectiveInsert, node::DIRECTIVE_INSERT, span);
                    rec.ptr("expression", value);
                    self.add(rec)
                }
                ast::EnumItem::If {
                    cond,
                    then_items,
                    else_items,
                } => {
                    let condition = self.expr(cond);
                    let then_block = self.enum_block(then_items, span);
                    let else_block = self.enum_block(else_items, span);
                    let mut rec = self.node(RecordTag::CodeIf, node::IF, span);
                    rec.ptr("condition", condition)
                        .ptr("then_block", then_block)
                        .ptr("else_block", else_block)
                        .int("if_flags", 4);
                    self.add(rec)
                }
            })
            .collect();
        let mut rec = self.node(RecordTag::CodeBlock, node::BLOCK, span);
        rec.refs("statements", statements).int("block_type", 2);
        self.add(rec)
    }

    /// A struct literal. Without a concrete type (polymorphic structs, anonymous
    /// structs in bodies) `defined_type` is a bare descriptor named `name`.
    pub(super) fn struct_lit(
        &mut self,
        lit: &ast::StructLit,
        defined: Option<TypeId>,
        name: Sym,
    ) -> i64 {
        let start = self.sub.len();
        let block = self.block(&lit.body, 2, lit.span);
        let arguments = if lit.params.is_empty() {
            0
        } else {
            let header = self.params_header(&lit.params, lit.span);
            let args: Vec<_> = match self.r.field(header, "arguments") {
                Some(FieldRef::List(list)) => list
                    .iter()
                    .filter_map(|item| match item {
                        Item::Ref(id) => Some(id),
                        _ => None,
                    })
                    .collect(),
                _ => Vec::new(),
            };
            let mut rec = self.node(RecordTag::CodeBlock, node::BLOCK, lit.span);
            rec.refs("statements", args).int("block_type", 5);
            self.add(rec)
        };
        let flags = (i64::from(lit.kind == ast::StructKind::Union) * 2)
            | i64::from(lit.flags.bits().bits())
            | (i64::from(lit.tag.is_some()) * 0x40);
        let alignment = self.opt_expr(lit.flags.align.as_ref());
        let tag = lit.tag.as_ref().map_or(0, |tag| {
            let ty = tag.ty.as_deref().map_or(0, |ty| self.type_inst(ty));
            let value = tag.value.as_deref().map_or(0, |value| self.expr(value));
            let mut rec = self.node(RecordTag::CodeDeclaration, node::DECLARATION, tag.name.span);
            rec.str("name", tag.name.name.as_str().as_bytes())
                .ptr("type_inst", ty)
                .ptr("expression", value);
            self.add(rec)
        });
        let modify = self.modify_syntax(lit.modify.as_ref());
        let notes: Vec<&ast::Note> = lit.notes.iter().collect();
        let notes = self.notes(&notes);
        let defined = defined
            .filter(|&t| {
                self.c
                    .as_deref()
                    .is_some_and(|c| matches!(c.types.kind(t), TypeKind::Struct(_)))
            })
            .map(|t| self.ty(t))
            .filter(|&t| t != 0);
        let defined = defined.unwrap_or_else(|| {
            let mut generic = Record::new(RecordTag::TypeInfoStruct);
            generic
                .int("type", 7)
                .str("name", name.as_str().as_bytes())
                .int("nontextual_flags", (!lit.params.is_empty()) as i64 * 0x100);
            self.r.add(generic)
        });
        let mut rec = self.node(RecordTag::CodeStruct, node::STRUCT, lit.span);
        rec.ptr("block", block)
            .ptr("arguments_block", arguments)
            .ptr(SyntaxKey::Alignment, alignment)
            .ptr(SyntaxKey::UnionTag, tag)
            .refs("modify_directives", modify)
            .refs("notes", notes)
            .ptr("defined_type", defined)
            .int("textual_flags", flags);
        let id = self.add(rec);
        let sub = self.sub[start..].to_vec();
        self.out.structs.push((id, sub));
        id
    }
}
