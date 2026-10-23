//! Declarations, procedures, structs and enums.
use super::{Convert, R, expr::mk};
use crate::{
    ast::{
        self, AstId, CallHintFlag, Decl, DeclKind, DeclRare, EnumItem, EnumLit, EnumMember, Expr,
        ExprKind, Foreign, ForeignName, Int128, Param, ProcBits, ProcFlags, ProcHeader, ProcLit,
        Return, StructBits, StructFlags, StructKind, StructLit, UnionTag, UsingFilter,
    },
    builtin_keys::SyntaxKey,
    interp::Trap,
    types::TypeId,
};
use std::rc::Rc;

/// The procedure directives a header reports as `#name` text.
const REPORTED: [(&str, ProcBits); 5] = [
    ("no_abc", ProcBits::NO_ABC),
    ("no_aoc", ProcBits::NO_AOC),
    ("runtime_support", ProcBits::RUNTIME_SUPPORT),
    ("type_info_none", ProcBits::TYPE_INFO_NONE),
    (
        "cpp_return_type_is_non_pod",
        ProcBits::CPP_RETURN_TYPE_IS_NON_POD,
    ),
];

impl Convert<'_, '_> {
    pub(super) fn decl(&mut self, at: u64, ty: TypeId) -> R<Decl> {
        let start = self.mark();
        let mut rare = DeclRare {
            union_tag: self.extra_expr(at, SyntaxKey::UnionTag)?,
            ..DeclRare::default()
        };
        let as_ = self.has(at, ty, "flags", "IS_MARKED_AS_AS")?;
        let using = self.flag(at, SyntaxKey::Using)?;
        if using {
            rare.using_filter =
                Some(self.private_using(at)?).filter(|filter| !matches!(filter, UsingFilter::None));
        }
        let name = self.text(at, ty, "name")?;
        let backtick = !name.is_empty() && self.has(at, ty, "flags", "HAS_SCOPE_MODIFIER")?;
        let names = if name.is_empty() {
            Vec::new()
        } else {
            vec![self.name(&name)]
        };
        self.word(":");
        let declared_type = self.opt_expr(at, ty, "type_inst")?;
        let mut expression = self.ptr(at, ty, "expression")?;
        let alias = self.extra(at, SyntaxKey::AliasExpression)?;
        if alias != 0 && expression == self.extra(at, SyntaxKey::InitialExpression)? {
            expression = alias;
        }
        let mut notes = Vec::new();
        let mut value = None;
        let mut procedure = false;
        if expression != 0 {
            self.word("=");
            let (kind, _) = self.node_type(expression)?;
            procedure = kind == "PROCEDURE_HEADER";
            let values = if kind == "COMMA_SEPARATED_ARGUMENTS" {
                self.comma_exprs(expression)?
            } else {
                vec![self.expr(expression)?]
            };
            let mut values = values.into_iter();
            value = values.next();
            rare.extra_values = values.collect();
        } else if self.has(at, ty, "flags", "IS_UNINITIALIZED")? {
            value = Some(mk(ExprKind::Uninit, self.word("---")));
        }
        // Notes after a procedure's body are the declaration's.
        if let Some(Expr {
            kind: ExprKind::Proc(lit),
            ..
        }) = &mut value
        {
            let lit = Rc::get_mut(lit).expect("a converted procedure is not shared");
            let header = Rc::get_mut(&mut lit.header).expect("a converted header is not shared");
            notes = std::mem::take(&mut header.notes);
            if let Some(operator) = name.strip_prefix("operator")
                && !operator.is_empty()
                && !operator.starts_with(|c: char| c.is_alphanumeric() || c == '_')
            {
                header.operator = Some(operator.into());
            }
        }
        rare.align = self.opt_expr(at, ty, "alignment_expression")?;
        rare.flags = self.flag_names(at, SyntaxKey::Flags)?;
        if self.flag(at, SyntaxKey::Foreign)? {
            let library = match self.extra(at, SyntaxKey::ForeignLibrary)? {
                0 => None,
                library => Some(self.ident_name(library)?),
            };
            let name = if self.flag(at, SyntaxKey::ForeignNamed)? {
                ForeignName::Named(self.attribute(at, SyntaxKey::ForeignSymbol)?.into())
            } else {
                ForeignName::Default
            };
            rare.foreign = Some(Foreign {
                library,
                name,
            });
        }
        if !procedure {
            notes = self.notes(at, ty)?;
        }
        let kind =
            if !names.is_empty() && value.is_some() && self.has(at, ty, "flags", "IS_CONSTANT")? {
                DeclKind::Const
            } else {
                DeclKind::Var
            };
        let mut decl = Decl {
            id: AstId::fresh(),
            names,
            kind,
            ty: declared_type,
            value,
            using,
            as_,
            backtick,
            notes: notes.into(),
            span: self.since(start),
            rare: None,
        };
        decl.set_rare(rare);
        Ok(decl)
    }

    /// `a, b := f();` and the mixed lists `a=, b := f();`.
    pub(super) fn compound(&mut self, at: u64, ty: TypeId) -> R<Decl> {
        let start = self.mark();
        let list = self.ptr(at, ty, "comma_separated_assignment")?;
        if list == 0 {
            return Err(Trap::new("a compound declaration needs its names"));
        }
        let lists = self.ty("Code_Comma_Separated_Arguments")?;
        let argument = self.ty("Code_Comma_Separated_Argument")?;
        let declaration = self.ty("Code_Declaration")?;
        let (mut names, mut existing, mut targets, mut backticks) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let mut constant = false;
        for arg in self.list(list, lists, "arguments")? {
            let node = self.ptr(arg, argument, "node")?;
            if self.enum_is(arg, argument, "modifier", "DECLARE")? {
                let name = self.text(node, declaration, "name")?;
                backticks.push(self.has(node, declaration, "flags", "HAS_SCOPE_MODIFIER")?);
                constant |= self.has(node, declaration, "flags", "IS_CONSTANT")?;
                names.push(self.name(&name));
                existing.push(false);
                targets.push(None);
            } else if self.enum_is(arg, argument, "modifier", "ASSIGN")? {
                let place = self.expr(node)?;
                let mut root = &place;
                while let ExprKind::Member(base, _)
                | ExprKind::Index(base, _)
                | ExprKind::Unary(_, base) = &root.kind
                {
                    root = base;
                }
                let ExprKind::Ident(name) = root.kind else {
                    return Err(Trap::new(
                        "a compound declaration assigns only to a variable or its member, index or dereference",
                    ));
                };
                names.push(ast::Ident {
                    name,
                    span: place.span,
                });
                backticks.push(false);
                existing.push(true);
                targets.push((!matches!(place.kind, ExprKind::Ident(_))).then_some(place));
            } else {
                return Err(Trap::new("unknown compound declaration modifier"));
            }
            self.word(",");
        }
        let properties = self.ptr(at, ty, "declaration_properties")?;
        let mut decl = if properties == 0 {
            Decl {
                id: AstId::fresh(),
                names: Vec::new(),
                kind: DeclKind::Var,
                ty: None,
                value: None,
                using: false,
                as_: false,
                backtick: false,
                notes: ast::Notes::default(),
                span: self.since(start),
                rare: None,
            }
        } else {
            self.decl(properties, declaration)?
        };
        let any_existing = existing.iter().any(|&assigned| assigned);
        let any_target = targets.iter().any(Option::is_some);
        let later_backtick = backticks.iter().skip(1).any(|&backtick| backtick);
        decl.names = names;
        decl.kind = if constant && decl.value.is_some() {
            DeclKind::Const
        } else {
            DeclKind::Var
        };
        decl.backtick = !later_backtick && backticks.first().copied().unwrap_or(false);
        let rare = decl.rare_mut();
        rare.existing = if any_existing {
            existing
        } else {
            Vec::new()
        };
        rare.targets = if any_target {
            targets
        } else {
            Vec::new()
        };
        rare.backtick_names = if later_backtick {
            backticks
        } else {
            Vec::new()
        };
        decl.trim_rare();
        decl.span = self.since(start);
        Ok(decl)
    }

    /// A parameter, with the `Code_Using` that names it, if any.
    fn param(&mut self, at: u64, ty: TypeId, using: Option<UsingFilter>) -> R<Param> {
        let start = self.mark();
        let discard = self.has(at, ty, "flags", "IS_MARKED_AS_DISCARD")?;
        let auto_bake = self.has(at, ty, "flags", "AUTO_VALUE_BAKE")?;
        let baked = !auto_bake && self.has(at, ty, "flags", "IS_CONSTANT")?;
        let name = self.text(at, ty, "name")?;
        let name = (!name.is_empty()).then(|| self.name(&name));
        let inst = self.ptr(at, ty, "type_inst")?;
        let (variadic, declared_type) = if inst == 0 {
            (false, None)
        } else {
            let inst_ty = self.ty("Code_Type_Instantiation")?;
            let variadic = self.num(inst, inst_ty, "inst_flags")? & 1 != 0;
            (variadic, Some(self.expr(inst)?))
        };
        let default = match self.opt_expr(at, ty, "expression")? {
            Some(value) => Some(value),
            None if self.has(at, ty, "flags", "IS_UNINITIALIZED")? => {
                Some(mk(ExprKind::Uninit, self.word("---")))
            }
            None => None,
        };
        let notes = self.notes(at, ty)?;
        let (using, using_filter) = match using {
            Some(filter) => (
                true,
                Some(filter).filter(|filter| !matches!(filter, UsingFilter::None)),
            ),
            None => (false, None),
        };
        self.word(",");
        Ok(Param {
            name,
            baked,
            auto_bake,
            using,
            using_filter,
            discard,
            variadic,
            ty: declared_type,
            default,
            notes,
            span: self.since(start),
        })
    }

    /// A procedure header's parameters, with the filters of their `using`s.
    fn params(&mut self, at: u64, ty: TypeId) -> R<Vec<Param>> {
        let declaration = self.ty("Code_Declaration")?;
        let using_ty = self.ty("Code_Using")?;
        let ident = self.ty("Code_Ident")?;
        let mut usings = Vec::new();
        for using in self.list(at, ty, "parameter_usings")? {
            let expression = self.ptr(using, using_ty, "expression")?;
            if expression == 0 || self.node_type(expression)?.0 != "IDENT" {
                return Err(Trap::new("parameter using requires an identifier"));
            }
            usings.push((self.text(expression, ident, "name")?, using));
        }
        let mut params = Vec::new();
        for arg in self.list(at, ty, "arguments")? {
            let name = self.text(arg, declaration, "name")?;
            let using = match usings.iter().find(|(used, _)| *used == name) {
                Some(&(_, using)) => Some(self.using_filter(using, using_ty)?),
                None => None,
            };
            params.push(self.param(arg, declaration, using)?);
        }
        Ok(params)
    }

    /// The parameters of a body-less header (`#module_parameters` lists); none for null.
    pub(super) fn header_params(&mut self, at: u64) -> R<Vec<Param>> {
        if at == 0 {
            return Ok(Vec::new());
        }
        self.node(at, |c, name, ty| {
            if name != "PROCEDURE_HEADER" {
                return Err(Trap::new(format!(
                    "a parameter list must be a PROCEDURE_HEADER, got {name}"
                )));
            }
            c.word("(");
            let params = c.params(at, ty)?;
            c.word(")");
            Ok(params)
        })
    }

    pub(super) fn procedure(&mut self, at: u64, ty: TypeId) -> R<Expr> {
        let start = self.mark();
        self.word("(");
        let params = self.params(at, ty)?;
        self.word(")");
        let lambda = self.extra(at, SyntaxKey::LambdaBody)?;
        let mut header = ProcHeader {
            id: AstId::fresh(),
            params,
            returns: Vec::new(),
            flags: ProcFlags::default(),
            foreign: None,
            modify: None,
            notes: Vec::new(),
            span: self.since(start),
            operator: None,
            baked: false,
        };
        if lambda != 0 {
            self.word("=>");
            let body = self.expr(lambda)?;
            let span = self.since(start);
            header.span = span;
            return Ok(mk(
                ExprKind::Lambda {
                    header: Rc::new(header),
                    body: Box::new(body),
                },
                span,
            ));
        }
        let declaration = self.ty("Code_Declaration")?;
        let returns = self.list(at, ty, "returns")?;
        if !returns.is_empty() {
            self.word("->");
        }
        for ret in returns {
            let ret_start = self.mark();
            let name = self.text(ret, declaration, "name")?;
            let name = (!name.is_empty()).then(|| self.name(&name));
            let declared_type = self.opt_expr(ret, declaration, "type_inst")?;
            let default = self.opt_expr(ret, declaration, "expression")?;
            let must = self.flag(ret, SyntaxKey::ReturnMust)?;
            header.returns.push(Return {
                name,
                ty: declared_type,
                default,
                must,
                span: self.since(ret_start),
            });
        }
        let flags = &mut header.flags;
        for (name, bit) in [
            ("C_CALL", ProcBits::C_CALL),
            ("SYNTACTICALLY_MARKED_AS_NO_CONTEXT", ProcBits::NO_CONTEXT),
            ("MACRO", ProcBits::EXPAND),
            ("NO_DEBUG", ProcBits::NO_DEBUG),
            ("SYMMETRIC", ProcBits::SYMMETRIC),
            ("CPP_METHOD", ProcBits::CPP_METHOD),
            ("NO_CALL", ProcBits::NO_CALL),
        ] {
            if self.has(at, ty, "procedure_flags", name)? {
                flags.set(bit);
            }
        }
        if self.has(
            at,
            ty,
            "procedure_flags",
            "SYNTACTICALLY_MARKED_AS_INLINE_YES",
        )? {
            flags.inline = CallHintFlag::Inline;
        } else if self.has(
            at,
            ty,
            "procedure_flags",
            "SYNTACTICALLY_MARKED_AS_INLINE_NO",
        )? {
            flags.inline = CallHintFlag::NoInline;
        }
        let intrinsic = self.has(at, ty, "procedure_flags", "INTRINSIC")?;
        if intrinsic
            || self.has(
                at,
                ty,
                "procedure_flags",
                "SYNTACTICALLY_MARKED_AS_COMPILER",
            )?
        {
            flags.set(if intrinsic {
                ProcBits::INTRINSIC
            } else {
                ProcBits::COMPILER
            });
            let builtin = self.attribute(at, SyntaxKey::BuiltinName)?;
            flags.builtin_name = (!builtin.is_empty()).then(|| builtin.into());
        }
        if self.has(at, ty, "procedure_flags", "DEPRECATED")? {
            let message = self.raw(at, ty, "deprecation_string")?;
            flags.deprecated = Some(
                (!message.is_empty() || self.flag(at, SyntaxKey::DeprecationNamed)?)
                    .then(|| message.into()),
            );
        }
        let elsewhere = self.has(at, ty, "procedure_flags", "ELSEWHERE")?;
        if elsewhere || self.flag(at, SyntaxKey::Foreign)? {
            let library = self.opt_name(at, ty, "library_identifier")?;
            let symbol = self.text(at, ty, "foreign_function_name")?;
            let name = if !symbol.is_empty() || self.flag(at, SyntaxKey::ForeignNamed)? {
                ForeignName::Named(symbol.into())
            } else {
                ForeignName::Default
            };
            if elsewhere {
                header.flags.elsewhere = Some(library);
                header.foreign = library.map(|library| Foreign {
                    library: Some(library),
                    name,
                });
            } else {
                header.foreign = Some(Foreign {
                    library,
                    name,
                });
            }
        }
        for flag in self.flag_names(at, SyntaxKey::Flags)? {
            match REPORTED
                .iter()
                .find(|(name, _)| *name == flag.name.as_str())
            {
                Some(&(_, bit)) => header.flags.set(bit),
                None => header.flags.other.push(flag),
            }
        }
        if self.flag(at, SyntaxKey::ProgramExport)? {
            header.flags.program_export = Some(if self.flag(at, SyntaxKey::ExportNamed)? {
                Some(
                    self.attribute(at, SyntaxKey::ExportName)?
                        .into_bytes()
                        .into(),
                )
            } else {
                None
            });
        }
        header.modify = self.modify(at, ty)?;
        header.span = self.since(start);
        let body = self.ptr(at, ty, "body_or_null")?;
        let body = if body == 0 {
            None
        } else {
            let block = self.ptr(body, self.ty("Code_Procedure_Body")?, "block")?;
            if block == 0 {
                return Err(Trap::new("a procedure body needs a block"));
            }
            Some(self.block(block)?)
        };
        header.notes = self.notes(at, ty)?;
        let external = header.foreign.is_some()
            || header.flags.compiler()
            || header.flags.intrinsic()
            || header.flags.elsewhere.is_some();
        let header = Rc::new(header);
        let kind = if body.is_some() || external {
            ExprKind::Proc(Rc::new(ProcLit {
                header,
                body,
            }))
        } else {
            ExprKind::ProcType(header)
        };
        Ok(mk(kind, self.since(start)))
    }

    /// The block of the last `#modify` directive.
    fn modify(&mut self, at: u64, ty: TypeId) -> R<Option<ast::Block>> {
        let directives = self.list(at, ty, "modify_directives")?;
        let Some(&last) = directives.last() else {
            return Ok(None);
        };
        self.node(last, |c, name, ty| {
            if name != "DIRECTIVE_MODIFY" {
                return Err(Trap::new(format!(
                    "modify_directives holds only DIRECTIVE_MODIFY nodes, got {name}"
                )));
            }
            c.word("#modify");
            let block = c.ptr(last, ty, "block")?;
            c.block(block).map(Some)
        })
    }

    pub(super) fn structure(&mut self, at: u64, ty: TypeId) -> R<StructLit> {
        let start = self.mark();
        let kind = if self.has(at, ty, "textual_flags", "UNION")? {
            self.word("union");
            StructKind::Union
        } else {
            self.word("struct");
            StructKind::Struct
        };
        let tag = if self.has(at, ty, "textual_flags", "UNION_IS_TAGGED")? {
            let tag = self.extra(at, SyntaxKey::UnionTag)?;
            Some(self.node(tag, |c, name, ty| {
                if name != "DECLARATION" {
                    return Err(Trap::new(format!(
                        "a union tag must be a DECLARATION, got {name}"
                    )));
                }
                let tag_name = c.text(tag, ty, "name")?;
                Ok(UnionTag {
                    name: c.name(&tag_name),
                    ty: c.opt_expr(tag, ty, "type_inst")?.map(Box::new),
                    value: c.opt_expr(tag, ty, "expression")?.map(Box::new),
                })
            })?)
        } else {
            None
        };
        let mut params = Vec::new();
        let arguments = self.ptr(at, ty, "arguments_block")?;
        if arguments != 0 {
            self.word("(");
            let declaration = self.ty("Code_Declaration")?;
            for param in self.list(arguments, self.ty("Code_Block")?, "statements")? {
                let using = if self.flag(param, SyntaxKey::Using)? {
                    Some(self.private_using(param)?)
                } else {
                    None
                };
                params.push(self.param(param, declaration, using)?);
            }
            self.word(")");
        }
        let mut flags = StructFlags::default();
        for (name, bit) in [
            ("NO_PADDING", StructBits::NO_PADDING),
            ("TYPE_INFO_NONE", StructBits::TYPE_INFO_NONE),
            (
                "TYPE_INFO_NO_SIZE_COMPLAINT",
                StructBits::TYPE_INFO_NO_SIZE_COMPLAINT,
            ),
            (
                "TYPE_INFO_PROCEDURES_ARE_VOID_POINTERS",
                StructBits::TYPE_INFO_PROCEDURES_ARE_VOID_POINTERS,
            ),
        ] {
            if self.has(at, ty, "textual_flags", name)? {
                flags.set(bit);
            }
        }
        let alignment = self.num(at, ty, "alignment")?;
        flags.align = if alignment > 0 {
            self.word("#align");
            let span = self.word(&alignment.to_string());
            Some(mk(ExprKind::Int(Int128::new(alignment.into())), span))
        } else {
            self.extra_expr(at, SyntaxKey::Alignment)?
        };
        let modify = self.modify(at, ty)?;
        let body = self.ptr(at, ty, "block")?;
        let body = if body == 0 {
            return Err(Trap::new("a struct needs its block"));
        } else {
            self.block(body)?.stmts
        };
        let notes = self.notes(at, ty)?;
        Ok(StructLit {
            id: AstId::fresh(),
            kind,
            tag,
            params,
            body,
            flags,
            modify,
            notes,
            span: self.since(start),
        })
    }

    pub(super) fn enumeration(&mut self, at: u64, ty: TypeId) -> R<EnumLit> {
        let start = self.mark();
        let flags_enum = self.num(at, ty, "is_flags")? != 0;
        self.word(if flags_enum {
            "enum_flags"
        } else {
            "enum"
        });
        let base = self.opt_expr(at, ty, "internal_type_inst")?;
        let complete = self.num(at, ty, "marked_as_complete")? != 0;
        let specified = self.num(at, ty, "marked_as_specified")? != 0;
        let child = self.ptr(at, ty, "block")?;
        let items = self.enum_items(child)?;
        let notes = self.notes(at, ty)?;
        Ok(EnumLit {
            id: AstId::fresh(),
            flags_enum,
            base,
            items,
            specified,
            complete,
            notes,
            span: self.since(start),
        })
    }

    fn enum_items(&mut self, block: u64) -> R<Vec<EnumItem>> {
        if block == 0 {
            return Ok(Vec::new());
        }
        self.word("{");
        let mut items = Vec::new();
        for member in self.list(block, self.ty("Code_Block")?, "statements")? {
            items.push(self.node(member, |c, name, ty| {
                Ok(match name {
                    "DECLARATION" => {
                        let member_name = c.text(member, ty, "name")?;
                        let name = c.name(&member_name);
                        let value = c.opt_expr(member, ty, "expression")?;
                        EnumItem::Member(EnumMember {
                            name,
                            value,
                            notes: c.notes(member, ty)?,
                        })
                    }
                    "IF" => {
                        c.word("#if");
                        let cond = c.child(member, ty, "condition")?;
                        let child = c.ptr(member, ty, "then_block")?;
                        let then_items = c.enum_items(child)?;
                        let child = c.ptr(member, ty, "else_block")?;
                        let else_items = c.enum_items(child)?;
                        EnumItem::If {
                            cond,
                            then_items,
                            else_items,
                        }
                    }
                    "DIRECTIVE_INSERT" => {
                        c.word("#insert");
                        EnumItem::Insert(c.child(member, ty, "expression")?)
                    }
                    "USING" => {
                        c.word("using");
                        EnumItem::Using(c.child(member, ty, "expression")?)
                    }
                    _ => {
                        return Err(Trap::new(format!(
                            "an enum body holds only members, using, #if and #insert, got {name}"
                        )));
                    }
                })
            })?);
            self.word(";");
        }
        self.word("}");
        Ok(items)
    }
}
