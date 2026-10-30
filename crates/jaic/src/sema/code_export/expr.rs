//! Exporting expressions.
use super::constants::node;
use super::{Exporter, bin_op};
use crate::builtin_keys::SyntaxKey;
use crate::records::{Field, FieldRef, Item, Record, RecordTag};
use crate::sema::{Sym, TypeId, ast};

impl Exporter<'_> {
    pub(super) fn expr_inner(&mut self, e: &ast::Expr) -> i64 {
        use ast::ExprKind as E;
        let span = e.span;
        let rec = match &e.kind {
            E::Int(..)
            | E::Char(..)
            | E::Float(..)
            | E::Str(..)
            | E::Bool(..)
            | E::Null
            | E::Uninit
            | E::Context
            | E::PolyVar {
                ..
            }
            | E::PolyRestricted {
                ..
            } => match self.expr_literal_record(e) {
                Ok(rec) => rec,
                Err(id) => return id,
            },
            E::Binary(..)
            | E::Unary(..)
            | E::Member(..)
            | E::InferredMember(..)
            | E::Index(..)
            | E::Call {
                ..
            }
            | E::Cast {
                ..
            }
            | E::Ifx {
                ..
            }
            | E::StructLit {
                ..
            }
            | E::ArrayLit {
                ..
            }
            | E::ArrayType {
                ..
            }
            | E::ProcType(..)
            | E::TypeDirective {
                ..
            }
            | E::Proc(..)
            | E::Lambda {
                ..
            }
            | E::Block(..)
            | E::Struct(..)
            | E::Enum(..) => match self.expr_operator_record(e) {
                Ok(rec) => rec,
                Err(id) => return id,
            },
            E::Run {
                ..
            }
            | E::Code(..)
            | E::Insert(..)
            | E::Location(..)
            | E::CallerLocation
            | E::Backtick(..)
            | E::Exists(..)
            | E::This
            | E::CompileTime
            | E::Bytes(..)
            | E::UnknownDirective(..)
            | E::ProcedureOfCall(..)
            | E::ProcedureName(..)
            | E::Bake {
                ..
            }
            | E::Asm(..)
            | E::CallerCode
            | E::File
            | E::Filepath
            | E::Line => match self.expr_directive_record(e) {
                Ok(rec) => rec,
                Err(id) => return id,
            },
            E::Ident(name) => return self.ident(*name, span),
        };
        let mut rec = rec;
        // Typechecked expressions carry their type (Epic_Fail compares operand types).
        if rec.field("type").is_none()
            && matches!(
                e.kind,
                E::Binary(..)
                    | E::Unary(..)
                    | E::Int(_)
                    | E::Float(_)
                    | E::Str(_)
                    | E::Char(_)
                    | E::Bool(_)
                    | E::Index(..)
                    | E::Member(..)
                    | E::Cast { .. }
            )
            && let Some(ty) = self.expr_type(e)
        {
            let ty = self.ty(ty);
            if ty != 0 {
                rec.ptr("type", ty);
            }
        }
        self.add(rec)
    }

    fn expr_literal_record(&mut self, e: &ast::Expr) -> std::result::Result<Record, i64> {
        use ast::ExprKind as E;
        let span = e.span;
        Ok(match &e.kind {
            E::Int(v) => {
                let mut rec = self.literal(span, 1);
                rec.int("_s64", v.get() as i64)
                    .int("value_flags", self.literal_number_flags(span, None, false));
                rec
            }
            E::Char(v) => {
                let mut rec = self.literal(span, 1);
                rec.int("_s64", *v as i64).int("value_flags", 0x1);
                rec
            }
            E::Float(v) => {
                let mut rec = self.literal(span, 1);
                rec.int("_float64", v.to_bits() as i64).int(
                    "value_flags",
                    self.literal_number_flags(span, Some(*v), false),
                );
                rec
            }
            E::Str(s) => {
                let mut rec = self.literal(span, 2);
                rec.str("_string", s);
                rec
            }
            E::Bool(b) => {
                let mut rec = self.literal(span, 3);
                rec.int("_s64", *b as i64);
                rec
            }
            E::Null => {
                let mut info = Record::new(RecordTag::CodePointerLiteralInfo);
                info.int("pointer_literal_type", 2);
                let info = self.r.add(info);
                let mut rec = self.literal(span, 8);
                rec.ptr("pointer_literal_info", info);
                rec
            }
            E::Uninit => self.literal(span, 0),
            E::Context => self.node(RecordTag::CodeContext, node::CONTEXT, span),
            // `$T` and `$$x`: an identifier that defines a polymorphic variable. `0x80` marks the
            // baked `$$` form, `0x100` an `/interface` restriction (jaic extensions to the flags).
            E::PolyVar {
                name,
                baked,
            } => {
                let mut rec = self.node(RecordTag::CodeIdent, node::IDENT, span);
                rec.str("name", name.as_str().as_bytes()).int(
                    "flags",
                    0x1 | if *baked {
                        0x80
                    } else {
                        0
                    },
                );
                rec.int(SyntaxKey::PolyBaked, i64::from(*baked));
                rec
            }
            E::PolyRestricted {
                name,
                restriction,
                interface,
            } => {
                let restriction = self.expr(restriction);
                let mut rec = self.node(RecordTag::CodeIdent, node::IDENT, span);
                rec.str("name", name.as_str().as_bytes())
                    .int(
                        "flags",
                        0x1 | if *interface {
                            0x100
                        } else {
                            0
                        },
                    )
                    .ptr("polymorph_restriction", restriction)
                    .ptr(SyntaxKey::Restriction, restriction)
                    .int(SyntaxKey::Interface, i64::from(*interface));
                rec
            }
            _ => unreachable!("dispatched only for this group's operations"),
        })
    }

    fn expr_operator_record(&mut self, e: &ast::Expr) -> std::result::Result<Record, i64> {
        use ast::ExprKind as E;
        let span = e.span;
        Ok(match &e.kind {
            E::Binary(op, a, b) => {
                let (a, b) = (self.expr(a), self.expr(b));
                let mut rec = self.node(RecordTag::CodeBinaryOperator, node::BINARY_OPERATOR, span);
                rec.int("operator_type", bin_op(*op))
                    .ptr("left", a)
                    .ptr("right", b);
                rec
            }
            // Jai folds a negated number literal into one literal (`#code case -1` is a
            // Code_Literal whose `_s64` metaprograms overwrite).
            E::Unary(ast::UnOp::Neg, a) if matches!(a.kind, E::Int(_) | E::Float(_)) => {
                match &a.kind {
                    E::Int(v) => {
                        let mut rec = self.literal(span, 1);
                        rec.int("_s64", (v.get() as i64).wrapping_neg())
                            .int("value_flags", self.literal_number_flags(span, None, true));
                        rec
                    }
                    E::Float(v) => {
                        let mut rec = self.literal(span, 1);
                        rec.int("_float64", (-v).to_bits() as i64).int(
                            "value_flags",
                            self.literal_number_flags(span, Some(-v), true),
                        );
                        rec
                    }
                    _ => unreachable!(),
                }
            }
            E::Unary(op, a) => {
                let a = self.expr(a);
                let op = match op {
                    ast::UnOp::Neg => b'-' as i64,
                    ast::UnOp::Plus => b'+' as i64,
                    ast::UnOp::Not => b'!' as i64,
                    ast::UnOp::BitNot => b'~' as i64,
                    ast::UnOp::Star => b'*' as i64,
                    ast::UnOp::Deref => 169,
                };
                let mut rec = self.node(RecordTag::CodeUnaryOperator, node::UNARY_OPERATOR, span);
                rec.int("operator_type", op).ptr("subexpression", a);
                rec
            }
            E::Member(a, name) => {
                let (a, b) = (self.expr(a), self.name_ident(name.name, name.span));
                let mut rec = self.node(RecordTag::CodeBinaryOperator, node::BINARY_OPERATOR, span);
                rec.int("operator_type", b'.' as i64)
                    .ptr("left", a)
                    .ptr("right", b);
                rec
            }
            E::InferredMember(name) => {
                let b = self.name_ident(name.name, name.span);
                let mut rec = self.node(RecordTag::CodeBinaryOperator, node::BINARY_OPERATOR, span);
                rec.int("operator_type", b'.' as i64).ptr("right", b);
                rec
            }
            E::Index(a, b) => {
                let (a, b) = (self.expr(a), self.expr(b));
                let mut rec = self.node(RecordTag::CodeBinaryOperator, node::BINARY_OPERATOR, span);
                rec.int("operator_type", 500).ptr("left", a).ptr("right", b);
                rec
            }
            E::Call {
                callee,
                args,
                hint,
            } => {
                // `f(x ,, allocator = a)`: what follows `,,` changes the context for the call
                // and is no argument of the procedure.
                let (changes, plain): (Vec<_>, Vec<_>) =
                    args.iter().cloned().partition(|arg| arg.context);
                let resolved = self
                    .resolve_call(callee, plain.len())
                    .map(|p| self.instance_called_at(p, span));
                let callee = self.expr(callee);
                let (unsorted, sorted) = self.args(&plain);
                let change_values: Vec<i64> = changes.iter().map(|a| self.expr(&a.value)).collect();
                let change_names = changes
                    .iter()
                    .map(|a| {
                        a.name
                            .map_or(String::new(), |n| n.name.as_str().to_string())
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                let change_list = self.node_list(&change_values, span);
                let mut rec = self.node(RecordTag::CodeProcedureCall, node::PROCEDURE_CALL, span);
                rec.ptr("procedure_expression", callee)
                    .int(
                        SyntaxKey::CallHint,
                        match hint {
                            ast::CallHint::None => 0,
                            ast::CallHint::Inline => 1,
                            ast::CallHint::NoInline => 2,
                        },
                    )
                    .refs("arguments_unsorted", unsorted)
                    .refs("arguments_sorted", sorted);
                if !change_values.is_empty() {
                    rec.ptr(SyntaxKey::ContextArguments, change_list)
                        .str(SyntaxKey::ContextNames, change_names.as_bytes());
                }
                if let Some(p) = resolved {
                    let header = self.resolved_header(p);
                    rec.ptr("resolved_procedure_expression", header);
                    // The call's type: its first return.
                    let first = match self.r.item(header, "returns", Some(0)) {
                        Some(Item::Ref(ret)) => Some(ret),
                        _ => None,
                    };
                    if let Some(FieldRef::Ref(ty)) = first.and_then(|ret| self.r.field(ret, "type"))
                    {
                        rec.ptr("type", ty);
                    }
                }
                rec
            }
            E::Cast {
                ty,
                value,
                flags,
            } => {
                let target = ty.as_ref().map_or(0, |t| self.type_inst(t));
                let value = self.expr(value);
                let mut cast_flags = 0;
                if ty.is_none() {
                    cast_flags |= 0x4;
                }
                if flags.no_check {
                    cast_flags |= 0x8;
                }
                if flags.truncate {
                    cast_flags |= 0x10;
                }
                if flags.force {
                    cast_flags |= 0x80;
                }
                let mut rec = self.node(RecordTag::CodeCast, node::CAST, span);
                rec.ptr("target_type", target)
                    .ptr("expression", value)
                    .int("cast_flags", cast_flags);
                rec
            }
            E::Ifx {
                cond,
                then_value,
                else_value,
                is_static,
            } => {
                let cond = self.expr(cond);
                let then_block = then_value.as_deref().map_or(0, |v| self.expr_block(v));
                let else_block = else_value.as_deref().map_or(0, |v| self.expr_block(v));
                let mut rec = self.node(RecordTag::CodeIf, node::IF, span);
                rec.ptr("condition", cond)
                    .ptr("then_block", then_block)
                    .ptr("else_block", else_block)
                    .int("if_flags", 0x2 | ((*is_static as i64) * 0x4));
                rec
            }
            E::StructLit {
                ty,
                fields,
            } => {
                let ty = ty.as_ref().map_or(0, |t| self.type_inst(t));
                // `name = value` fields are `=` binary operators, as in Jai.
                let mut values = Vec::new();
                for field in fields {
                    let value = self.expr(&field.value);
                    let target = match (&field.name, &field.target) {
                        (Some(n), _) => self.name_ident(n.name, n.span),
                        (None, Some(t)) => self.expr(t),
                        (None, None) => {
                            values.push(value);
                            continue;
                        }
                    };
                    let mut rec = self.node(
                        RecordTag::CodeBinaryOperator,
                        node::BINARY_OPERATOR,
                        field.value.span,
                    );
                    rec.int("operator_type", i64::from(b'='))
                        .ptr("left", target)
                        .ptr("right", value);
                    values.push(self.add(rec));
                }
                let mut info = Record::new(RecordTag::CodeStructLiteralInfo);
                info.ptr("type_expression", ty).refs("arguments", values);
                let info = self.r.add(info);
                let mut rec = self.literal(span, 7);
                rec.ptr("struct_literal_info", info);
                rec
            }
            E::ArrayLit {
                ty,
                elems,
            } => {
                let elem_ty = ty.as_ref().map_or(0, |t| self.type_inst(t));
                let elems: Vec<i64> = elems.iter().map(|e| self.expr(e)).collect();
                let mut info = Record::new(RecordTag::CodeArrayLiteralInfo);
                info.ptr("element_type", elem_ty)
                    .refs("array_members", elems)
                    .int("array_literal_flags", ty.is_none() as i64);
                let info = self.r.add(info);
                let mut rec = self.literal(span, 6);
                rec.ptr("array_literal_info", info);
                rec
            }
            E::ArrayType {
                ..
            }
            | E::ProcType(_)
            | E::TypeDirective {
                ..
            } => return Err(self.type_inst(e)),
            E::Proc(lit) => return Err(self.proc(lit, &[], None, span)),
            E::Lambda {
                header,
                body,
            } => {
                let lit = ast::ProcLit {
                    header: header.clone(),
                    body: None,
                };
                let id = self.proc(&lit, &[], None, span);
                let body = self.expr(body);
                self.r.get_mut(id).unwrap().ptr(SyntaxKey::LambdaBody, body);
                return Err(id);
            }
            E::Block(block) => return Err(self.syntax_block(block)),
            E::Struct(lit) => return Err(self.struct_lit(lit, None, Sym::intern(""))),
            E::Enum(lit) => return Err(self.enum_lit(lit, None, span)),
            _ => unreachable!("dispatched only for this group's operations"),
        })
    }

    fn expr_directive_record(&mut self, e: &ast::Expr) -> std::result::Result<Record, i64> {
        use ast::ExprKind as E;
        let span = e.span;
        Ok(match &e.kind {
            E::Run {
                body,
                flags,
            } => {
                let body = match &**body {
                    ast::RunBody::Expr(expr) => self.expr(expr),
                    ast::RunBody::Block(block) => self.syntax_block(block),
                };
                let mut rec = self.node(RecordTag::CodeDirectiveRun, node::DIRECTIVE_RUN, span);
                rec.ptr(SyntaxKey::RunBody, body).str(
                    SyntaxKey::Flags,
                    flags
                        .iter()
                        .map(|flag| format!(",{}", flag.name))
                        .collect::<String>()
                        .as_bytes(),
                );
                rec
            }
            E::Code(body) => {
                let null =
                    matches!(&**body, ast::CodeBody::Expr(expr) if matches!(expr.kind, E::Null));
                let expr = match &**body {
                    ast::CodeBody::Expr(e) => self.expr(e),
                    ast::CodeBody::Stmt(stmt) => match self.stmt(stmt) {
                        Some(id) => id,
                        None => self.block(std::slice::from_ref(stmt), 1, stmt.span),
                    },
                    ast::CodeBody::Block(b) => self.block(&b.stmts, 1, b.span),
                };
                let mut rec = self.node(RecordTag::CodeDirectiveCode, node::DIRECTIVE_CODE, span);
                rec.ptr("expression", expr).int(
                    "code_flags",
                    if null {
                        2
                    } else {
                        0
                    },
                );
                rec
            }
            E::Insert(boxed) => {
                let crate::ast::InsertExpr {
                    value,
                    scope,
                    replacements,
                    flags,
                } = &**boxed;
                self.insert_syntax(value, scope.as_ref(), replacements, flags, span)
            }
            E::Location(e) => {
                let e = self.opt_expr(e.as_deref());
                let mut rec = self.node(
                    RecordTag::CodeDirectiveLocation,
                    node::DIRECTIVE_LOCATION,
                    span,
                );
                rec.ptr("expression", e);
                rec
            }
            E::CallerLocation => {
                let mut rec = self.node(
                    RecordTag::CodeDirectiveLocation,
                    node::DIRECTIVE_LOCATION,
                    span,
                );
                rec.int("is_caller_location", 1);
                rec
            }
            E::Backtick(e) => {
                let id = self.expr(e);
                if matches!(e.kind, E::Ident(_))
                    && let Some(rec) = self.r.get_mut(id)
                {
                    rec.set("flags", Field::Item(Item::Int(0x40))); // HAS_SCOPE_MODIFIER
                }
                return Err(id);
            }
            E::Exists(query, _) => {
                let query = self.expr(query);
                let mut rec =
                    self.node(RecordTag::CodeDirectiveExists, node::DIRECTIVE_EXISTS, span);
                rec.ptr("query_expression", query);
                rec
            }
            E::This => self.node(RecordTag::CodeNode, node::DIRECTIVE_THIS, span),
            E::CompileTime => self.node(RecordTag::CodeNode, node::DIRECTIVE_COMPILE_TIME, span),
            E::Bytes(e) => {
                let e = self.expr(e);
                let mut rec = self.node(RecordTag::CodeDirectiveBytes, node::DIRECTIVE_BYTES, span);
                rec.ptr("expression", e);
                rec
            }
            E::UnknownDirective(directive) => {
                let operand = self.opt_expr(directive.operand.as_ref());
                let mut rec = self.node(RecordTag::CodeNode, node::PLACEHOLDER, span);
                rec.str(
                    SyntaxKey::DirectiveName,
                    directive.name.name.as_str().as_bytes(),
                )
                .str(
                    SyntaxKey::Flags,
                    directive
                        .flags
                        .iter()
                        .map(|flag| format!(",{}", flag.name))
                        .collect::<String>()
                        .as_bytes(),
                )
                .ptr(SyntaxKey::DirectiveArgs, operand);
                rec
            }
            E::ProcedureOfCall(call) => {
                let call = self.expr(call);
                let mut rec = self.node(RecordTag::CodeNode, node::PLACEHOLDER, span);
                rec.str(SyntaxKey::DirectiveName, b"procedure_of_call")
                    .ptr(SyntaxKey::DirectiveArgs, call);
                rec
            }
            E::ProcedureName(arg) => {
                let arg = self.opt_expr(arg.as_deref());
                let mut rec = self.node(
                    RecordTag::CodeDirectiveProcedureName,
                    node::DIRECTIVE_PROCEDURE_NAME,
                    span,
                );
                rec.ptr("argument", arg);
                rec
            }
            E::Bake {
                callee,
                args,
                constants,
            } => {
                let call = ast::Expr {
                    kind: E::Call {
                        callee: callee.clone(),
                        args: args.clone(),
                        hint: ast::CallHint::None,
                    },
                    span,
                };
                let call = self.expr(&call);
                let mut rec = self.node(RecordTag::CodeDirectiveBake, node::DIRECTIVE_BAKE, span);
                // CONSTANTS_BAKE or PARAMETER_VALUE_BAKE.
                rec.ptr("procedure_call", call)
                    .int("bake_type", i64::from(!*constants));
                rec
            }
            E::Asm(block) => return Err(self.asm_syntax(block, span)),
            E::CallerCode => {
                let mut rec = self.node(
                    RecordTag::CodeDirectiveLocation,
                    node::DIRECTIVE_LOCATION,
                    span,
                );
                rec.int("is_caller_code", 1).int(SyntaxKey::CallerCode, 1);
                rec
            }
            // The location directives are constants of the file they sit in: exported as the
            // literal they evaluate to when the compiler's sources are at hand.
            E::File | E::Filepath | E::Line if self.c.is_some() => {
                let c = self.c.as_deref().unwrap();
                let src = c.src.sources.get(span.file);
                match &e.kind {
                    E::Line => {
                        let (line, _) = src.line_col(span.start);
                        let mut rec = self.literal(span, 1);
                        rec.int("_s64", line as i64).int("value_flags", 0x1);
                        rec
                    }
                    kind => {
                        let text = if matches!(kind, E::File) {
                            crate::paths::program_path(&src.path.to_string())
                        } else {
                            src.path
                                .to_path()
                                .parent()
                                .map(|p| {
                                    format!(
                                        "{}/",
                                        crate::paths::program_path(&p.display().to_string())
                                    )
                                })
                                .unwrap_or_default()
                        };
                        let mut rec = self.literal(span, 2);
                        rec.str("_string", text.as_bytes());
                        rec
                    }
                }
            }
            _ => unreachable!("dispatched only for this group's operations"),
        })
    }

    /// The type `e` checks to in the exported code's scope, when it checks there.
    fn expr_type(&mut self, e: &ast::Expr) -> Option<TypeId> {
        if self.procedure.is_some() {
            return None;
        }
        // Locals are not in that scope: checking would fail after a costly search for the
        // name, or find a global the local shadows.
        if self.reads_local(e) {
            return None;
        }
        let scope = self.scope;
        let c = self.c.as_deref_mut()?;
        let ty = c.check_expr_no_emit(scope, e).ok()?.ty();
        (ty != TypeId::COMPILE_TIME).then_some(ty)
    }

    /// Does `e` name a local of the exported code (looking through operators, members,
    /// indexing, casts and calls)?
    fn reads_local(&self, e: &ast::Expr) -> bool {
        use ast::ExprKind as E;
        match &e.kind {
            E::Ident(name) => self.locals.iter().any(|(n, _)| n == name),
            E::Binary(_, a, b) | E::Index(a, b) => self.reads_local(a) || self.reads_local(b),
            E::Unary(_, a)
            | E::Member(a, _)
            | E::Cast {
                value: a, ..
            } => self.reads_local(a),
            E::Call {
                callee,
                args,
                ..
            } => self.reads_local(callee) || args.iter().any(|a| self.reads_local(&a.value)),
            _ => false,
        }
    }

    /// `Code_Argument` records and the argument values.
    pub(super) fn args(&mut self, args: &[ast::Arg]) -> (Vec<i64>, Vec<i64>) {
        let mut unsorted = Vec::new();
        let mut values = Vec::new();
        for arg in args {
            let value = self.expr(&arg.value);
            if arg.spread {
                let node = self.r.get_mut(value).unwrap();
                let flags = match node.field("node_flags") {
                    Some(FieldRef::Int(flags)) => flags,
                    _ => 0,
                };
                node.int("node_flags", flags | 1);
            }
            let name = arg.name.map_or(0, |n| self.name_ident(n.name, n.span));
            let mut rec = Record::new(RecordTag::CodeArgument);
            rec.ptr("expression", value).ptr("name", name);
            unsorted.push(self.r.add(rec));
            values.push(value);
        }
        (unsorted, values)
    }

    /// A block holding one expression (branches of `ifx`).
    fn expr_block(&mut self, e: &ast::Expr) -> i64 {
        let value = self.expr(e);
        let mut rec = self.node(RecordTag::CodeBlock, node::BLOCK, e.span);
        rec.int("block_type", 1).refs("statements", [value]);
        self.add(rec)
    }
}
