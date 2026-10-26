//! `#insert`: running the operand and turning its result into statements or an expression.
use super::lower::Operand;
use super::{Compiler, Result, ScopeId, Span, ast, err};
use super::{Rc, Value};

impl Compiler {
    /// Evaluate the operand of `#insert`. `#insert -> string { ... }` (or `-> Code`) is a
    /// procedure without parameters that runs at compile time; its result is inserted.
    pub fn eval_insert_operand(&mut self, scope: ScopeId, value: &ast::Expr) -> Result<Operand> {
        if let ast::ExprKind::Proc(lit) = &value.kind
            && lit.header.params.is_empty()
            && !lit.header.returns.is_empty()
        {
            let call = ast::Expr {
                kind: ast::ExprKind::Call {
                    callee: Box::new(value.clone()),
                    args: Vec::new(),
                    hint: ast::CallHint::None,
                },
                span: value.span,
            };
            let op = self.check_run_inner(scope, &ast::RunBody::Expr(call), None, value.span)?;
            if self.ed.is_live() {
                self.ide_note_insert(value, &op);
            }
            return Ok(op);
        }
        let op = self.eval_const(scope, value, None)?;
        if self.ed.is_live() {
            self.ide_note_insert(value, &op);
        }
        Ok(op)
    }

    pub fn insert_stmts_from(&mut self, op: Operand, span: Span) -> Result<Vec<ast::Stmt>> {
        match op {
            Operand::Const {
                value: Value::Code(code),
                ..
            } => Ok(match &*self.pr.codes[code.0 as usize] {
                ast::CodeBody::Block(b) => b.stmts.clone(),
                ast::CodeBody::Stmt(s) => vec![s.clone()],
                ast::CodeBody::Expr(e) => vec![ast::Stmt {
                    kind: ast::StmtKind::Expr(e.clone()),
                    span: e.span,
                    notes: crate::ast::Notes::default(),
                }],
            }),
            Operand::Const {
                value: Value::String(s),
                ..
            } => {
                let text: Rc<str> = String::from_utf8_lossy(&s).into();
                let file = self.src.insert_source(span, text.clone())?;
                let ast = crate::parser::parse_file(file, &text).map_err(Box::new)?;
                Ok(ast.stmts)
            }
            Operand::Void => Ok(Vec::new()),
            other => err(
                span,
                format!(
                    "#insert needs a string or Code, found {}",
                    self.describe(&other)
                ),
            ),
        }
    }

    pub fn eval_insert_stmts(
        &mut self,
        scope: ScopeId,
        value: &ast::Expr,
    ) -> Result<Vec<ast::Stmt>> {
        let op = self.eval_insert_operand(scope, value)?;
        self.insert_stmts_from(op, value.span)
    }

    /// The expression an expression-position `#insert` produces, and the scope it is checked
    /// in: a `Code` value's own scope (where its names were written), else `scope`.
    pub fn eval_insert_expr(
        &mut self,
        scope: ScopeId,
        value: &ast::Expr,
    ) -> Result<(ast::Expr, ScopeId)> {
        let op = self.eval_insert_operand(scope, value)?;
        if let Operand::Const {
            value: Value::Code(code),
            ..
        } = &op
        {
            let code_scope = self.code_scope_at(*code, scope);
            return Ok((self.insert_expr_of(op, value)?, code_scope));
        }
        Ok((self.insert_expr_of(op, value)?, scope))
    }

    fn insert_expr_of(&mut self, op: Operand, value: &ast::Expr) -> Result<ast::Expr> {
        match op {
            Operand::Const {
                value: Value::Code(code),
                ..
            } => match &*self.pr.codes[code.0 as usize] {
                ast::CodeBody::Expr(e) => Ok(e.clone()),
                ast::CodeBody::Stmt(_) => {
                    err(value.span, "#insert of a statement in expression position")
                }
                ast::CodeBody::Block(b) => match b.stmts.as_slice() {
                    [
                        ast::Stmt {
                            kind: ast::StmtKind::Expr(e),
                            ..
                        },
                    ] => Ok(e.clone()),
                    _ => err(
                        value.span,
                        "#insert of a statement block in expression position",
                    ),
                },
            },
            Operand::Const {
                value: Value::String(s),
                ..
            } => {
                // A trailing `;` (`#insert "x.y;"` as an expression) ends the statement only.
                let source = String::from_utf8_lossy(&s);
                let source = source.trim_end().trim_end_matches(';');
                let text = format!("__jaic_insert :: ({source});");
                let file = self.src.insert_source(value.span, text.clone().into())?;
                let ast = crate::parser::parse_file(file, &text).map_err(Box::new)?;
                match ast.stmts.first().map(|s| &s.kind) {
                    Some(ast::StmtKind::Decl(d)) if d.value.is_some() => {
                        Ok(d.value.clone().unwrap())
                    }
                    _ => err(value.span, "could not parse inserted expression"),
                }
            }
            other => err(
                value.span,
                format!(
                    "#insert needs a string or Code, found {}",
                    self.describe(&other)
                ),
            ),
        }
    }
}
