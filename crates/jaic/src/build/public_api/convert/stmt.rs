//! Statements and control flow.
use super::{Convert, R, expr::mk};
use crate::{
    ast::{
        self, AssignOp, BinOp, Case, ExprKind, For, ForOver, PushContextDeferStmt, PushContextStmt,
        ScopeKind, StaticIfStmt, Stmt, StmtKind, SwitchStmt, UsingStmt, WhileStmt,
    },
    builtin_keys::SyntaxKey,
    interp::Trap,
    types::TypeId,
};
use std::rc::Rc;

/// The assignment an `operator_type` stands for, if it is one.
pub(super) fn assign_op(value: u64) -> Option<AssignOp> {
    Some(match value {
        61 | 159 => AssignOp::Assign,
        145 => AssignOp::Op(BinOp::Add),
        146 => AssignOp::Op(BinOp::Sub),
        147 => AssignOp::Op(BinOp::Mul),
        148 => AssignOp::Op(BinOp::Div),
        149 => AssignOp::Op(BinOp::Rem),
        150 => AssignOp::Op(BinOp::Shl),
        151 => AssignOp::Op(BinOp::Shr),
        152 => AssignOp::Op(BinOp::Rotl),
        153 => AssignOp::Op(BinOp::Rotr),
        154 => AssignOp::Op(BinOp::BitAnd),
        155 => AssignOp::Op(BinOp::BitOr),
        156 => AssignOp::Op(BinOp::BitXor),
        157 => AssignOp::Op(BinOp::And),
        158 => AssignOp::Op(BinOp::Or),
        _ => return None,
    })
}

impl Convert<'_, '_> {
    pub(super) fn stmt(&mut self, at: u64) -> R<Stmt> {
        self.node(at, |c, name, ty| {
            let start = c.mark();
            let kind = c.stmt_kind(at, name, ty)?;
            let notes = c
                .extra_list(at, SyntaxKey::StatementNotes)?
                .into_iter()
                .map(|note| c.note(note))
                .collect::<R<Vec<_>>>()?;
            c.word(";");
            let mut stmt = Stmt {
                kind,
                span: c.since(start),
                notes: ast::Notes::default(),
            };
            // Notes after a declaration are the declaration's.
            match &mut stmt.kind {
                StmtKind::Decl(decl) | StmtKind::AddContext(decl) => Rc::get_mut(decl)
                    .expect("a converted declaration is not shared")
                    .notes
                    .extend(notes),
                _ => stmt.notes.extend(notes),
            }
            Ok(stmt)
        })
    }

    /// A branch or loop body: a block, or the one statement written there.
    fn branch(&mut self, at: u64, ty: TypeId, field: &str) -> R<Stmt> {
        let body = self.ptr(at, ty, field)?;
        self.stmt(body)
    }

    fn stmt_kind(&mut self, at: u64, name: &str, ty: TypeId) -> R<StmtKind> {
        Ok(match name {
            "DECLARATION" => StmtKind::Decl(Rc::new(self.decl(at, ty)?)),
            "COMPOUND_DECLARATION" => StmtKind::Decl(Rc::new(self.compound(at, ty)?)),
            "BINARY_OPERATOR" => {
                let Some(op) = assign_op(self.num(at, ty, "operator_type")?) else {
                    return Ok(StmtKind::Expr(self.expr_node(at, name, ty)?));
                };
                let child = self.ptr(at, ty, "left")?;
                let lhs = self.comma_exprs(child)?;
                self.word("=");
                let child = self.ptr(at, ty, "right")?;
                let rhs = self.comma_exprs(child)?;
                StmtKind::Assign {
                    op,
                    lhs,
                    rhs,
                }
            }
            "BLOCK" => StmtKind::Block(self.block_node(at, ty)?),
            "IF" => return self.conditional(at, name, ty),
            "CASE" => StmtKind::Case(Rc::new(self.case(at, ty)?)),
            "WHILE" => {
                self.word("while");
                let label = self.attribute(at, SyntaxKey::Label)?;
                let label = (!label.is_empty()).then(|| self.name(&label));
                let bind_label = self.flag(at, SyntaxKey::BindLabel)?;
                let backtick_label = self.flag(at, SyntaxKey::BacktickLabel)?;
                let cond = self.child(at, ty, "condition")?;
                StmtKind::While(Box::new(WhileStmt {
                    label,
                    bind_label,
                    backtick_label,
                    cond,
                    body: Box::new(self.branch(at, ty, "block")?),
                }))
            }
            "FOR" => StmtKind::For(Box::new(self.for_loop(at, ty)?)),
            "LOOP_CONTROL" => {
                let control = self.num(at, ty, "control_type")?;
                self.word(match control {
                    0 => "break",
                    1 => "continue",
                    _ => "remove",
                });
                let target = self.opt_name(at, ty, "target_ident")?;
                match control {
                    0 => StmtKind::Break(target),
                    1 => StmtKind::Continue(target),
                    _ => StmtKind::Remove(target),
                }
            }
            "RETURN" => {
                let backtick = self.num(at, ty, "return_flags")? & 1 != 0;
                self.word("return");
                StmtKind::Return {
                    values: self.arguments(at, ty, "arguments_unsorted")?,
                    backtick,
                }
            }
            "DEFER" => {
                let backtick = self.num(at, ty, "is_backticked")? != 0;
                self.word("defer");
                StmtKind::Defer {
                    body: Box::new(self.branch(at, ty, "block")?),
                    backtick,
                }
            }
            "USING" => {
                let filter = self.using_filter(at, ty)?;
                StmtKind::Using(Box::new(UsingStmt {
                    value: self.child(at, ty, "expression")?,
                    filter,
                }))
            }
            "PUSH_CONTEXT" => self.push_context(at, ty)?,
            "DIRECTIVE_INSERT" => StmtKind::Insert(Box::new(self.insert(at, ty)?)),
            "DIRECTIVE_RUN" => self.run_stmt(at, ty)?,
            "DIRECTIVE_IMPORT" => StmtKind::Import(Rc::new(self.import(at, ty)?)),
            "DIRECTIVE_LOAD" => {
                self.word("#load");
                let path = self.text(at, ty, "short_name")?;
                let span = self.word(&super::quote_bytes(path.as_bytes()));
                StmtKind::Load {
                    path: path.into(),
                    span,
                }
            }
            "DIRECTIVE_SCOPE" => StmtKind::Scope(match self.num(at, ty, "scope_type")? {
                0 => {
                    self.word("#scope_export");
                    ScopeKind::Export
                }
                1 => {
                    self.word("#scope_file");
                    ScopeKind::File
                }
                2 => {
                    self.word("#scope_module");
                    ScopeKind::Module
                }
                _ => return Err(Trap::new("unknown scope directive")),
            }),
            "DIRECTIVE_THROUGH" => {
                self.word("#through");
                StmtKind::Through
            }
            "DIRECTIVE_PLACE" => {
                self.word("#place");
                let operand = self.extra(at, SyntaxKey::Operand)?;
                StmtKind::Place(Box::new(self.expr(operand)?))
            }
            "DIRECTIVE_OVERLAY" => {
                self.word("#overlay");
                StmtKind::Overlay(Box::new(self.child(at, ty, "target_expression")?))
            }
            "DIRECTIVE_ADD_CONTEXT" => {
                self.word("#add_context");
                let decl = self.ptr(at, ty, "expression")?;
                StmtKind::AddContext(Rc::new(self.node(decl, |c, name, ty| {
                    if name != "DECLARATION" {
                        return Err(Trap::new(format!(
                            "#add_context requires a declaration, got {name}"
                        )));
                    }
                    c.decl(decl, ty)
                })?))
            }
            "DIRECTIVE_MODULE_PARAMETERS" => {
                self.word("#module_parameters");
                let child = self.ptr(at, ty, "module_parameters")?;
                let params = self.header_params(child)?;
                let child = self.ptr(at, ty, "program_parameters")?;
                let runtime_params = self.header_params(child)?;
                let common = self.ptr(at, ty, "common_code")?;
                let body = if common == 0 {
                    None
                } else {
                    Some(self.block(common)?)
                };
                StmtKind::ModuleParameters(Box::new(ast::ModuleParametersStmt {
                    params,
                    runtime_params,
                    body,
                }))
            }
            "PLACEHOLDER" => self.directive_stmt(at, name, ty)?,
            _ => StmtKind::Expr(self.expr_node(at, name, ty)?),
        })
    }

    /// The expressions of a node that may be a `COMMA_SEPARATED_ARGUMENTS`.
    pub(super) fn comma_exprs(&mut self, at: u64) -> R<Vec<ast::Expr>> {
        if at == 0 {
            return Ok(Vec::new());
        }
        let nodes = self.comma_nodes(at)?;
        let mut values = Vec::new();
        for (index, node) in nodes.into_iter().enumerate() {
            if index > 0 {
                self.word(",");
            }
            values.push(self.expr(node)?);
        }
        Ok(values)
    }

    fn conditional(&mut self, at: u64, name: &str, ty: TypeId) -> R<StmtKind> {
        let flags = self.num(at, ty, "if_flags")?;
        if flags & 2 != 0 {
            return Ok(StmtKind::Expr(self.expr_node(at, name, ty)?));
        }
        let start = self.mark();
        self.word(if flags & 4 != 0 {
            "#if"
        } else {
            "if"
        });
        let cond = self.child(at, ty, "condition")?;
        if flags & 1 != 0 {
            self.word("==");
            let header = self.since(start);
            let block = self.ptr(at, ty, "then_block")?;
            let mut cases = Vec::new();
            if block != 0 {
                for case in self.list(block, self.ty("Code_Block")?, "statements")? {
                    cases.push(self.node(case, |c, name, ty| {
                        if name != "CASE" {
                            return Err(Trap::new(format!(
                                "a switch holds only CASE nodes, got {name}"
                            )));
                        }
                        c.case(case, ty)
                    })?);
                }
            }
            let switch = Box::new(SwitchStmt {
                value: cond,
                cases,
                complete: flags & 4 == 0 && flags & 8 != 0,
                header,
            });
            return Ok(if flags & 4 != 0 {
                StmtKind::StaticSwitch(switch)
            } else {
                StmtKind::Switch(switch)
            });
        }
        if flags & 4 != 0 {
            let then_block = self.ptr(at, ty, "then_block")?;
            let then_branch = if then_block == 0 {
                Vec::new()
            } else {
                self.block(then_block)?.stmts
            };
            let then_arm = self.since(start);
            let else_block = self.ptr(at, ty, "else_block")?;
            let (else_branch, else_arm) = if else_block == 0 {
                (Vec::new(), None)
            } else {
                let start = self.mark();
                self.word("else");
                let stmts = self.block(else_block)?.stmts;
                (stmts, Some(self.since(start)))
            };
            return Ok(StmtKind::StaticIf(Box::new(StaticIfStmt {
                cond,
                then_branch,
                else_branch,
                then_arm,
                else_arm,
            })));
        }
        let then_branch = Box::new(self.branch(at, ty, "then_block")?);
        let else_block = self.ptr(at, ty, "else_block")?;
        let else_branch = if else_block == 0 {
            None
        } else {
            self.word("else");
            Some(Box::new(self.stmt(else_block)?))
        };
        Ok(StmtKind::If {
            cond: Box::new(cond),
            then_branch,
            else_branch,
        })
    }

    fn case(&mut self, at: u64, ty: TypeId) -> R<Case> {
        let start = self.mark();
        self.word("case");
        let child = self.ptr(at, ty, "condition")?;
        let values = self.comma_exprs(child)?;
        let block = self.ptr(at, ty, "then_block")?;
        let body = if block == 0 {
            Vec::new()
        } else {
            self.statements(block)?
        };
        let through = self.num(at, ty, "marked_as_fallthrough")? != 0;
        Ok(Case {
            values,
            body,
            through,
            span: self.since(start),
        })
    }

    fn for_loop(&mut self, at: u64, ty: TypeId) -> R<For> {
        if self.ptr(at, ty, "macro_expansion_procedure_call")? != 0
            || self.num(at, ty, "want_replacement_for_expansion")? != 0
        {
            return Err(Trap::new("cannot serialize expanded iterator"));
        }
        self.word("for");
        let flags = self.flag_names(at, SyntaxKey::Flags)?;
        let iterator = self.attribute(at, SyntaxKey::Iterator)?;
        let iterator = if iterator.is_empty() {
            None
        } else {
            let start = self.mark();
            let mut path: Option<ast::Expr> = None;
            for part in iterator.split('.') {
                let name = self.name(part);
                path = Some(match path {
                    None => mk(ExprKind::Ident(name.name), name.span),
                    Some(base) => mk(ExprKind::Member(Box::new(base), name), self.since(start)),
                });
            }
            path
        };
        let pointer_if = self.opt_expr(at, ty, "want_pointer_expression")?;
        let reverse_if = self.opt_expr(at, ty, "want_reverse_expression")?;
        let by_pointer = pointer_if.is_none() && self.has(at, ty, "for_flags", "POINTER")?;
        let reverse = reverse_if.is_none() && self.has(at, ty, "for_flags", "REVERSE")?;
        let ident = self.ty("Code_Ident")?;
        let mut backtick_names = false;
        let mut names = [None, None];
        for (slot, field) in names.iter_mut().zip(["ident_it", "ident_it_index"]) {
            let name = self.ptr(at, ty, field)?;
            if name != 0 {
                *slot = Some(self.ident_name(name)?);
                backtick_names |= self.num(name, ident, "flags")? & 0x40 != 0;
            }
        }
        let [it, index] = names;
        if it.is_none() && index.is_some() {
            return Err(Trap::new("for index requires an iterator identifier"));
        }
        let first = self.child(at, ty, "iteration_expression")?;
        let over = match self.opt_expr(at, ty, "iteration_expression_right")? {
            Some(last) => ForOver::Range(first, last),
            None => ForOver::Collection(first),
        };
        Ok(For {
            it,
            index,
            by_pointer,
            reverse,
            pointer_if,
            reverse_if,
            iterator,
            backtick_names,
            over,
            body: Box::new(self.branch(at, ty, "block")?),
            flags,
        })
    }

    fn push_context(&mut self, at: u64, ty: TypeId) -> R<StmtKind> {
        let start = self.mark();
        let backtick = self.has(at, ty, "push_context_flags", "IS_BACKTICKED")?;
        self.word("push_context");
        let original = self.extra(at, SyntaxKey::OriginalContext)?;
        let to_push = self.ptr(at, ty, "to_push")?;
        let bare = to_push == 0 || (self.flag(at, SyntaxKey::ContextBare)? && original == to_push);
        let context = if bare {
            mk(ExprKind::Context, self.since(start))
        } else {
            self.expr(to_push)?
        };
        Ok(if self.has(at, ty, "push_context_flags", "DEFER_POP")? {
            StmtKind::PushContextDefer(Box::new(PushContextDeferStmt {
                context,
                bare,
                backtick,
            }))
        } else {
            StmtKind::PushContext(Box::new(PushContextStmt {
                backtick,
                context,
                body: Box::new(self.branch(at, ty, "block")?),
                bare,
            }))
        })
    }

    /// `#run` and `#assert` statements.
    fn run_stmt(&mut self, at: u64, ty: TypeId) -> R<StmtKind> {
        if self.num(at, ty, "flags")? & 1 != 0 {
            self.word("#assert");
            let cond = match self.run(at, ty)?.0 {
                ast::RunBody::Expr(cond) => cond,
                ast::RunBody::Block(_) => {
                    return Err(Trap::new("an assertion's condition must be an expression"));
                }
            };
            let message = self.extra_expr(at, SyntaxKey::AssertMessage)?;
            let args = self
                .extra_list(at, SyntaxKey::AssertArgs)?
                .into_iter()
                .map(|arg| self.expr(arg))
                .collect::<R<Vec<_>>>()?;
            return Ok(StmtKind::Assert(Box::new(ast::AssertStmt {
                cond,
                message,
                args,
            })));
        }
        // `#run f();` at statement level keeps the `#run` expression it runs.
        let body = self.extra(at, SyntaxKey::RunBody)?;
        if body != 0 {
            let (name, ty) = self.node_type(body)?;
            if name == "DIRECTIVE_RUN" && self.num(body, ty, "flags")? & 1 == 0 {
                return Ok(StmtKind::Run(Box::new(self.expr(body)?)));
            }
        }
        let start = self.mark();
        let (body, flags) = self.run(at, ty)?;
        Ok(StmtKind::Run(Box::new(mk(
            ExprKind::Run {
                body: Rc::new(body),
                flags,
            },
            self.since(start),
        ))))
    }
}
