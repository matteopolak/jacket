//! Exporting blocks, statements and switches.
use super::constants::node;
use super::{Exporter, assign_op};
use crate::builtin_keys::SyntaxKey;
use crate::records::RecordTag;
use crate::sema::{Rc, Span, Sym, ast};

impl Exporter<'_> {
    pub(super) fn block(&mut self, stmts: &[ast::Stmt], block_type: i64, span: Span) -> i64 {
        let locals = self.locals.len();
        let id = self.block_inner(stmts, block_type, span);
        self.locals.truncate(locals);
        id
    }

    fn block_inner(&mut self, stmts: &[ast::Stmt], block_type: i64, span: Span) -> i64 {
        let mut statements = Vec::new();
        let mut members = Vec::new();
        for s in stmts {
            let Some(id) = self.stmt(s) else {
                continue;
            };
            if let ast::StmtKind::Decl(d) = &s.kind {
                // A block's members are declarations: each name of `a, b := f()`.
                if d.names.len() > 1 {
                    members.append(&mut self.compound_members);
                } else {
                    members.push(id);
                }
            }
            statements.push(id);
        }
        let mut rec = self.node(RecordTag::CodeBlock, node::BLOCK, span);
        rec.int("block_type", block_type)
            .refs("members", members)
            .refs("statements", statements);
        self.add(rec)
    }

    /// A statement as a block (branches and loop bodies).
    fn stmt_block(&mut self, s: &ast::Stmt) -> i64 {
        match &s.kind {
            ast::StmtKind::Block(b) => self.syntax_block(b),
            _ => self.block(std::slice::from_ref(s), 1, s.span),
        }
    }

    pub(super) fn stmt(&mut self, s: &ast::Stmt) -> Option<i64> {
        use ast::StmtKind as S;
        let span = s.span;
        let id = match &s.kind {
            S::Assign {
                ..
            }
            | S::Block(..)
            | S::If {
                ..
            }
            | S::Switch(..)
            | S::StaticSwitch(..)
            | S::StaticIf(..)
            | S::While(..)
            | S::For(..)
            | S::Break(..)
            | S::Continue(..)
            | S::Remove(..)
            | S::Return {
                ..
            }
            | S::Defer {
                ..
            }
            | S::Case(..) => self.stmt_control(s),
            S::Using(..)
            | S::PushContextDefer(..)
            | S::PushContext(..)
            | S::Insert(..)
            | S::Run(..)
            | S::Assert(..) => self.stmt_context(s),
            S::Import(..)
            | S::Scope(..)
            | S::Through
            | S::Load {
                ..
            }
            | S::Place(..)
            | S::Overlay(..)
            | S::AddContext(..)
            | S::ModuleParameters(..)
            | S::Placeholder(..)
            | S::Directive(..) => self.stmt_module(s),
            S::Decl(d) => self.local_decl(d),
            S::Expr(e) => self.expr(e),
            S::Empty => return None,
        };
        if !s.notes.is_empty() {
            let notes = self.notes(&s.notes.iter().collect::<Vec<_>>());
            let notes = self.node_list(&notes, span);
            self.r
                .get_mut(id)
                .unwrap()
                .ptr(SyntaxKey::StatementNotes, notes);
        }
        Some(id)
    }

    fn stmt_control(&mut self, s: &ast::Stmt) -> i64 {
        use ast::StmtKind as S;
        let span = s.span;
        match &s.kind {
            S::Assign {
                op,
                lhs,
                rhs,
            } => {
                let l = self.comma_values(lhs, span);
                let r = self.comma_values(rhs, span);
                let mut rec = self.node(RecordTag::CodeBinaryOperator, node::BINARY_OPERATOR, span);
                rec.int("operator_type", assign_op(*op))
                    .ptr("left", l)
                    .ptr("right", r);
                self.add(rec)
            }
            S::Block(b) => self.syntax_block(b),
            S::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond = self.expr(cond);
                let then_block = self.stmt_block(then_branch);
                let else_block = else_branch.as_deref().map_or(0, |b| self.stmt_block(b));
                let mut rec = self.node(RecordTag::CodeIf, node::IF, span);
                rec.ptr("condition", cond)
                    .ptr("then_block", then_block)
                    .ptr("else_block", else_block);
                self.add(rec)
            }
            S::Switch(boxed) => {
                let crate::ast::SwitchStmt {
                    value,
                    cases,
                    complete,
                    ..
                } = &**boxed;
                self.switch(value, cases, 0x1 | ((*complete as i64) * 0x8), span)
            }
            S::StaticSwitch(boxed) => {
                let crate::ast::SwitchStmt {
                    value,
                    cases,
                    ..
                } = &**boxed;
                let scope = self.scope;
                let accepted = self.c.as_deref_mut().and_then(|c| {
                    let case = c.static_switch_case(scope, value, cases).ok()??;
                    cases.iter().position(|it| std::ptr::eq(it, case))
                });
                self.switch_accepting(value, cases, 0x1 | 0x4, accepted, span)
            }
            S::StaticIf(boxed) => {
                let crate::ast::StaticIfStmt {
                    cond,
                    then_branch,
                    else_branch,
                    ..
                } = &**boxed;
                // Which branch the compiler took, where the condition is known here (it is in
                // a procedure or a polymorph instance: `#if async` with `async` baked in).
                let scope = self.scope;
                let taken = self
                    .c
                    .as_deref_mut()
                    .and_then(|c| c.eval_static_condition(scope, cond).ok());
                let cond = self.expr(cond);
                let then_block = self.block(then_branch, 1, span);
                let else_block = self.block(else_branch, 1, span);
                let mut rec = self.node(RecordTag::CodeIf, node::IF, span);
                rec.ptr("condition", cond)
                    .ptr("then_block", then_block)
                    .ptr("else_block", else_block)
                    .int("if_flags", 0x4);
                if taken == Some(true) {
                    rec.int("static_if_flags", 0x1);
                }
                self.add(rec)
            }
            S::While(boxed) => {
                let crate::ast::WhileStmt {
                    cond,
                    body,
                    label,
                    bind_label,
                    backtick_label,
                } = &**boxed;
                let cond = self.expr(cond);
                let block = self.stmt_block(body);
                let mut rec = self.node(RecordTag::CodeWhile, node::WHILE, span);
                rec.ptr("condition", cond).ptr("block", block);
                if let Some(label) = label {
                    rec.str(SyntaxKey::Label, label.name.as_str().as_bytes())
                        .int(SyntaxKey::BindLabel, i64::from(*bind_label))
                        .int(SyntaxKey::BacktickLabel, i64::from(*backtick_label));
                }
                self.add(rec)
            }
            S::For(f) => {
                let (a, b) = match &f.over {
                    ast::ForOver::Range(a, b) => (self.expr(a), self.expr(b)),
                    ast::ForOver::Collection(c) => (self.expr(c), 0),
                };
                // The loop's variables are locals of the body, not names of the enclosing
                // scope; they have no declaration record.
                let locals = self.locals.len();
                for (name, default) in [(f.it, "it"), (f.index, "it_index")] {
                    let name = name.map_or_else(|| Sym::intern(default), |i| i.name);
                    self.locals.push((name, 0));
                }
                let it = f.it.map_or(0, |i| self.ident(i.name, i.span));
                let index = f.index.map_or(0, |i| self.ident(i.name, i.span));
                if f.backtick_names {
                    for id in [it, index].into_iter().filter(|&id| id != 0) {
                        self.r.get_mut(id).unwrap().int("flags", 0x40);
                    }
                }
                let pointer = self.opt_expr(f.pointer_if.as_ref());
                let reverse = self.opt_expr(f.reverse_if.as_ref());
                let block = self.stmt_block(&f.body);
                self.locals.truncate(locals);
                let flags = (f.by_pointer as i64) | ((f.reverse as i64) * 0x2);
                let mut rec = self.node(RecordTag::CodeFor, node::FOR, span);
                rec.ptr("iteration_expression", a)
                    .ptr("iteration_expression_right", b)
                    .ptr("ident_it", it)
                    .ptr("ident_it_index", index)
                    .ptr("want_pointer_expression", pointer)
                    .ptr("want_reverse_expression", reverse)
                    .ptr("block", block)
                    .int("for_flags", flags);
                if let Some(path) = f.iterator.as_ref().and_then(ast::Expr::path_string) {
                    rec.str(SyntaxKey::Iterator, path.as_bytes());
                }
                rec.str(
                    SyntaxKey::Flags,
                    f.flags
                        .iter()
                        .map(|flag| format!("#{} ", flag.name))
                        .collect::<String>()
                        .as_bytes(),
                );
                self.add(rec)
            }
            S::Break(label) | S::Continue(label) | S::Remove(label) => {
                let control = match s.kind {
                    S::Break(_) => 0,
                    S::Continue(_) => 1,
                    _ => 2,
                };
                let target = label.map_or(0, |l| self.name_ident(l.name, l.span));
                let mut rec = self.node(RecordTag::CodeLoopControl, node::LOOP_CONTROL, span);
                rec.int("control_type", control).ptr("target_ident", target);
                self.add(rec)
            }
            S::Return {
                values,
                backtick,
            } => {
                let (unsorted, sorted) = self.args(values);
                // Jai leaves a return's `arguments_sorted` empty (yield-jai checks).
                let _ = sorted;
                let mut rec = self.node(RecordTag::CodeReturn, node::RETURN, span);
                rec.refs("arguments_unsorted", unsorted)
                    .int("return_flags", *backtick as i64);
                self.add(rec)
            }
            S::Defer {
                body,
                backtick,
            } => {
                let block = self.stmt_block(body);
                let mut rec = self.node(RecordTag::CodeDefer, node::DEFER, span);
                rec.ptr("block", block)
                    .int("is_backticked", *backtick as i64);
                self.add(rec)
            }
            S::Case(case) => self.case(case),
            _ => unreachable!("dispatched only for this group's operations"),
        }
    }

    fn stmt_context(&mut self, s: &ast::Stmt) -> i64 {
        use ast::StmtKind as S;
        let span = s.span;
        match &s.kind {
            S::Using(boxed) => {
                let crate::ast::UsingStmt {
                    value,
                    filter,
                } = &**boxed;
                let value = self.expr(value);
                let (filter_type, filter_expression) = self.using_filter(filter);
                let mut rec = self.node(RecordTag::CodeUsing, node::USING, span);
                rec.ptr("expression", value)
                    .int("filter_type", filter_type)
                    .ptr("filter_expression", filter_expression)
                    .str(SyntaxKey::Filter, Self::filter_syntax(filter).as_bytes())
                    .int(
                        SyntaxKey::StaticFilter,
                        if filter_expression == 0 {
                            filter_type
                        } else {
                            0
                        },
                    )
                    .str(
                        SyntaxKey::FilterArguments,
                        Self::filter_arguments(filter).as_bytes(),
                    );
                self.add(rec)
            }
            S::PushContextDefer(defer) => {
                let context = self.expr(&defer.context);
                let mut rec = self.node(RecordTag::CodePushContext, node::PUSH_CONTEXT, span);
                rec.ptr("to_push", context)
                    .ptr(SyntaxKey::OriginalContext, context)
                    .int(SyntaxKey::ContextBare, i64::from(defer.bare))
                    .int("push_context_flags", 2 | i64::from(defer.backtick));
                self.add(rec)
            }
            S::PushContext(boxed) => {
                let crate::ast::PushContextStmt {
                    context,
                    body,
                    backtick,
                    bare,
                } = &**boxed;
                let context = self.expr(context);
                let block = self.stmt_block(body);
                let mut rec = self.node(RecordTag::CodePushContext, node::PUSH_CONTEXT, span);
                rec.ptr("to_push", context)
                    .ptr(SyntaxKey::OriginalContext, context)
                    .int(SyntaxKey::ContextBare, i64::from(*bare))
                    .ptr("block", block)
                    .int("push_context_flags", i64::from(*backtick));
                self.add(rec)
            }
            S::Insert(boxed) => {
                let crate::ast::InsertStmt {
                    value,
                    scope,
                    replacements,
                    flags,
                } = &**boxed;
                let rec = self.insert_syntax(value, scope.as_ref(), replacements, flags, span);
                self.add(rec)
            }
            S::Run(e) => {
                let mut rec = self.node(RecordTag::CodeDirectiveRun, node::DIRECTIVE_RUN, span);
                let e = self.expr(e);
                rec.ptr("expression", e).ptr(SyntaxKey::RunBody, e);
                self.add(rec)
            }
            S::Assert(boxed) => {
                let crate::ast::AssertStmt {
                    cond,
                    message,
                    args,
                } = &**boxed;
                let mut rec = self.node(RecordTag::CodeDirectiveRun, node::DIRECTIVE_RUN, span);
                let text = self.c.as_deref().map_or(String::new(), |c| {
                    c.src.sources.snippet(cond.span).to_string()
                });
                let cond = self.expr(cond);
                let message = self.opt_expr(message.as_ref());
                let args = self.comma_values(args, span);
                rec.int("flags", 0x1) // ASSERTION
                    .str("assertion_string", text.as_bytes())
                    .ptr("expression", cond)
                    .ptr("message", message)
                    .ptr(SyntaxKey::RunBody, cond)
                    .ptr(SyntaxKey::AssertMessage, message)
                    .ptr(SyntaxKey::AssertArgs, args);
                self.add(rec)
            }
            _ => unreachable!("dispatched only for this group's operations"),
        }
    }

    fn stmt_module(&mut self, s: &ast::Stmt) -> i64 {
        use ast::StmtKind as S;
        let span = s.span;
        match &s.kind {
            S::Import(import) => {
                let mut rec =
                    self.node(RecordTag::CodeDirectiveImport, node::DIRECTIVE_IMPORT, span);
                let (name, kind) = match &import.source {
                    ast::ImportSource::Module(name) => (name, 0),
                    ast::ImportSource::File(name) => (name, 1),
                    ast::ImportSource::Dir(name) => (name, 2),
                    ast::ImportSource::String(name) => (name, 3),
                };
                rec.str("name", name.as_bytes())
                    .int(SyntaxKey::ImportKind, kind)
                    .str(
                        SyntaxKey::Flags,
                        import
                            .flags
                            .iter()
                            .map(|flag| format!(",{}", flag.name))
                            .collect::<String>()
                            .as_bytes(),
                    )
                    .str(
                        SyntaxKey::Alias,
                        import
                            .name
                            .map_or(String::new(), |name| name.name.to_string())
                            .as_bytes(),
                    )
                    .int(SyntaxKey::Using, i64::from(import.using.is_some()));
                if let Some(filter) = &import.using {
                    rec.str(SyntaxKey::Filter, Self::filter_syntax(filter).as_bytes());
                    let (kind, expression) = self.using_filter(filter);
                    rec.int(SyntaxKey::FilterKind, kind)
                        .ptr(SyntaxKey::FilterExpression, expression);
                }
                let split = import.program_params_start.unwrap_or(import.params.len());
                for (field, args, present) in [
                    (
                        "module_parameters_call",
                        &import.params[..split],
                        import.module_params_present,
                    ),
                    (
                        "program_parameters_call",
                        &import.params[split..],
                        import.program_params_start.is_some(),
                    ),
                ] {
                    if present {
                        let (arguments, _) = self.args(args);
                        let mut call =
                            self.node(RecordTag::CodeProcedureCall, node::PROCEDURE_CALL, span);
                        call.refs("arguments_unsorted", arguments);
                        let call = self.add(call);
                        rec.ptr(field, call);
                    }
                }
                self.add(rec)
            }
            S::Scope(scope) => {
                let mut rec = self.node(RecordTag::CodeDirectiveScope, node::DIRECTIVE_SCOPE, span);
                rec.int(
                    "scope_type",
                    match scope {
                        ast::ScopeKind::Export => 0,
                        ast::ScopeKind::File => 1,
                        ast::ScopeKind::Module => 2,
                    },
                );
                self.add(rec)
            }
            S::Through => {
                let rec = self.node(
                    RecordTag::CodeDirectiveThrough,
                    node::DIRECTIVE_THROUGH,
                    span,
                );
                self.add(rec)
            }
            S::Load {
                path, ..
            } => {
                let mut rec = self.node(RecordTag::CodeDirectiveLoad, node::DIRECTIVE_LOAD, span);
                rec.str("short_name", path.as_bytes());
                if let Some(c) = self.c.as_deref()
                    && (span.file.0 as usize) < c.src.sources.len()
                {
                    let from = c.src.sources.get(span.file).path.to_path();
                    let full = from.parent().unwrap_or(&from).join(&**path);
                    rec.str("fully_pathed_filename", full.to_string_lossy().as_bytes());
                }
                self.add(rec)
            }
            S::Place(e) => {
                let ident = self.expr(e);
                let mut rec = self.node(RecordTag::CodeNode, node::DIRECTIVE_PLACE, span);
                rec.ptr("ident", ident).ptr(SyntaxKey::Operand, ident);
                self.add(rec)
            }
            S::Overlay(e) => {
                let target = self.expr(e);
                let mut rec = self.node(
                    RecordTag::CodeDirectiveOverlay,
                    node::DIRECTIVE_OVERLAY,
                    span,
                );
                rec.ptr("target_expression", target);
                self.add(rec)
            }
            S::AddContext(d) => {
                let name = d.names.first().map_or(Sym::intern(""), |n| n.name);
                let decl = self.decl(d, name, None, None, true, 0);
                let mut rec = self.node(
                    RecordTag::CodeDirectiveAddContext,
                    node::DIRECTIVE_ADD_CONTEXT,
                    span,
                );
                rec.ptr("expression", decl);
                self.add(rec)
            }
            S::ModuleParameters(boxed) => {
                let crate::ast::ModuleParametersStmt {
                    params,
                    runtime_params,
                    body,
                } = &**boxed;
                let module = self.params_header(params, span);
                let program = self.params_header(runtime_params, span);
                let common = match body {
                    Some(b) => self.block(&b.stmts, 0, b.span),
                    None => 0,
                };
                let mut rec = self.node(
                    RecordTag::CodeDirectiveModuleParameters,
                    node::DIRECTIVE_MODULE_PARAMETERS,
                    span,
                );
                rec.ptr("module_parameters", module)
                    .ptr("program_parameters", program)
                    .ptr("common_code", common);
                self.add(rec)
            }
            S::Placeholder(names) => {
                let values = names
                    .iter()
                    .map(|name| self.name_ident(name.name, name.span))
                    .collect::<Vec<_>>();
                let args = self.node_list(&values, span);
                let mut rec = self.node(RecordTag::CodeNode, node::PLACEHOLDER, span);
                rec.str(SyntaxKey::DirectiveName, b"placeholder")
                    .ptr(SyntaxKey::DirectiveArgs, args);
                self.add(rec)
            }
            S::Directive(directive) => {
                let args = self.comma_values(&directive.args, span);
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
                .ptr(SyntaxKey::DirectiveArgs, args);
                self.add(rec)
            }
            _ => unreachable!("dispatched only for this group's operations"),
        }
    }

    /// A parameter list as a body-less procedure header (the form `#module_parameters` takes).
    pub(super) fn params_header(&mut self, params: &[ast::Param], span: Span) -> i64 {
        let header = ast::ProcHeader {
            id: ast::AstId::fresh(),
            params: params.to_vec(),
            returns: Vec::new(),
            flags: ast::ProcFlags::default(),
            foreign: None,
            modify: None,
            notes: Vec::new(),
            span,
            operator: None,
            baked: false,
        };
        let lit = ast::ProcLit {
            header: Rc::new(header),
            body: None,
        };
        self.proc(&lit, &[], None, span)
    }

    fn case(&mut self, case: &ast::Case) -> i64 {
        let cond = self.comma_values(&case.values, case.span);
        let then_block = self.block(&case.body, 1, case.span);
        let mut rec = self.node(RecordTag::CodeCase, node::CASE, case.span);
        rec.ptr("condition", cond)
            .ptr("then_block", then_block)
            .int("marked_as_fallthrough", case.through as i64);
        self.add(rec)
    }

    fn switch(&mut self, value: &ast::Expr, cases: &[ast::Case], flags: i64, span: Span) -> i64 {
        self.switch_accepting(value, cases, flags, None, span)
    }

    /// A switch; `accepted` is the index of the case a static switch (`#if x == {`) took.
    fn switch_accepting(
        &mut self,
        value: &ast::Expr,
        cases: &[ast::Case],
        flags: i64,
        accepted: Option<usize>,
        span: Span,
    ) -> i64 {
        let value = self.expr(value);
        let case_ids = cases.iter().map(|case| self.case(case)).collect::<Vec<_>>();
        let accepted_case = accepted.and_then(|i| case_ids.get(i).copied());
        let mut block = self.node(RecordTag::CodeBlock, node::BLOCK, span);
        block.int("block_type", 1).refs("statements", case_ids);
        let block = self.add(block);
        let mut rec = self.node(RecordTag::CodeIf, node::IF, span);
        rec.ptr("condition", value)
            .ptr("then_block", block)
            .int("if_flags", flags);
        if let Some(case) = accepted_case {
            rec.ptr("static_if_accepted_case", case);
        }
        self.add(rec)
    }
}
