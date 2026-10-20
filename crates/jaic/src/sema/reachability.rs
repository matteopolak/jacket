//! Which code of a procedure can never run: the warning at a statement after a `return`, `break`
//! or `continue` of its block, and, for an editor, the code to fade (`IdeFacts::dead_runs`).
//!
//! `warn_unreachable` walks the body's statements once and decides which code control cannot
//! reach: after `return`, `break` and `continue`, after an `if` or `switch` whose every path
//! exits, after a loop that only a `break` leaves, in a branch whose condition always has one
//! value, in a `while` that never starts, and in a `case` its value cannot select. What a
//! condition can be comes from the editor (`ide::values`); a batch compile knows no values, so
//! only the jumps decide there.
use crate::ast::{self, BinOp, StmtKind as S};
use crate::ide::Narrowing;
use crate::intern::Sym;
use crate::sema::Compiler;
use crate::sema::lower::FnCtx;
use crate::source::{Diagnostic, Span};

/// How the walk got to where it is.
#[derive(Default)]
struct Walk {
    loops: Vec<LoopCtx>,
    /// Locals narrowed by the branches being walked.
    env: Narrowing,
    /// The dead runs found, by span.
    dead: Vec<Span>,
    /// For the `if` just walked: whether control can leave its then branch and its else branch
    /// (a missing else always can).
    if_exits: (bool, bool),
}

struct LoopCtx {
    label: Option<Sym>,
    broken: bool,
}

impl Walk {
    /// A `break` (to `label`, or the innermost loop) is reached. A label that names no loop
    /// seen could be any, so every enclosing loop counts.
    fn reached_break(&mut self, label: Option<Sym>) {
        match label {
            None => {
                if let Some(l) = self.loops.last_mut() {
                    l.broken = true;
                }
            }
            Some(name) => {
                if let Some(l) = self.loops.iter_mut().rev().find(|l| l.label == Some(name)) {
                    l.broken = true;
                } else {
                    self.loops.iter_mut().for_each(|l| l.broken = true);
                }
            }
        }
    }
}

/// The statements inside a branch's braces, else the statement itself: what fades with it.
pub(super) fn branch_span(stmt: &ast::Stmt) -> Span {
    match &stmt.kind {
        S::Block(b) => match (b.stmts.first(), b.stmts.last()) {
            (Some(first), Some(last)) => first.span.to(last.span),
            _ => stmt.span,
        },
        _ => stmt.span,
    }
}

fn stmts_span(stmts: &[ast::Stmt]) -> Option<Span> {
    Some(stmts.first()?.span.to(stmts.last()?.span))
}

/// Whether `stmt` is something that would run: constants, and statements that only configure
/// the program, do not count.
pub(crate) fn runs_in_order(stmt: &ast::Stmt) -> bool {
    match &stmt.kind {
        S::Decl(decl) => decl.kind != ast::DeclKind::Const,
        S::Empty
        | S::Scope(_)
        | S::AddContext(_)
        | S::ModuleParameters(_)
        | S::Load {
            ..
        }
        | S::Case(_)
        | S::Overlay(_) => false,
        _ => true,
    }
}

impl Compiler {
    /// Walk `body` for code that cannot run: record it for the editor, and warn at the first
    /// statement after a `return`, `break` or `continue` of its block, as Jai does.
    ///
    /// Returns whether control can reach the end of the body.
    pub(super) fn warn_unreachable(&mut self, f: &FnCtx, body: &ast::Block) -> bool {
        let mut walk = Walk::default();
        let completes = self.walk_stmts(f, &mut walk, &body.stmts, true);
        if self.ed.sketching() || !self.ide_wants_pub(body.span.file) {
            return completes;
        }
        self.ed.body_walked(body.span, walk.dead);
        completes
    }

    /// Whether control can leave the end of `stmts`.
    fn walk_stmts(&mut self, f: &FnCtx, walk: &mut Walk, stmts: &[ast::Stmt], warn: bool) -> bool {
        let mark = walk.env.mark();
        let completes = self.walk_run(f, walk, stmts, warn);
        walk.env.truncate(mark);
        completes
    }

    fn walk_run(&mut self, f: &FnCtx, walk: &mut Walk, stmts: &[ast::Stmt], warn: bool) -> bool {
        for (i, stmt) in stmts.iter().enumerate() {
            if self.walk_stmt(f, walk, stmt, warn) {
                // `if c { exits }` leaves the rest of the block knowing `c` was false (and
                // `if c { } else { exits }` that it was true).
                if let S::If {
                    cond, ..
                } = &stmt.kind
                    && let Some(rest) = stmts_span(&stmts[i + 1..])
                {
                    match walk.if_exits {
                        (false, true) => self.ed.narrow(&mut walk.env, cond, false, rest),
                        (true, false) => self.ed.narrow(&mut walk.env, cond, true, rest),
                        _ => {}
                    }
                }
                continue;
            }
            let Some(next) = stmts[i + 1..].iter().find(|s| runs_in_order(s)) else {
                return false;
            };
            if (next.span.file.0 as usize) >= self.sources.len() {
                return false;
            }
            let word = match &stmt.kind {
                S::Return {
                    ..
                } => Some("return"),
                S::Break(_) => Some("break"),
                S::Continue(_) => Some("continue"),
                _ => None,
            };
            if warn && let Some(word) = word {
                self.warn(
                    Diagnostic::warning(
                        next.span,
                        format!("statements in block after `{word}` are never reached"),
                    )
                    .with_label("this statement can not run")
                    .with_help(format!("remove it, or the `{word}` before it")),
                );
            }
            if let Some(last) = stmts.last() {
                walk.dead.push(next.span.to(last.span));
            }
            return false;
        }
        true
    }

    /// Whether control can leave the end of `stmt`.
    fn walk_stmt(&mut self, f: &FnCtx, walk: &mut Walk, stmt: &ast::Stmt, warn: bool) -> bool {
        match &stmt.kind {
            S::Return {
                ..
            }
            | S::Continue(_) => false,
            S::Break(label) => {
                walk.reached_break(label.as_ref().map(|l| l.name));
                false
            }
            S::Block(b) => self.walk_stmts(f, walk, &b.stmts, warn),
            S::If {
                cond,
                then_branch,
                else_branch,
            } => self.walk_if(f, walk, cond, then_branch, else_branch.as_deref(), warn),
            S::While(w) => {
                let truth = self.walk_truth(walk, &w.cond);
                if truth == Some(false) {
                    walk.dead.push(branch_span(&w.body));
                    return true;
                }
                walk.loops.push(LoopCtx {
                    label: w.label.as_ref().map(|l| l.name),
                    broken: false,
                });
                let mark = walk.env.mark();
                self.ed.narrow(&mut walk.env, &w.cond, true, w.body.span);
                self.walk_stmt(f, walk, &w.body, warn);
                walk.env.truncate(mark);
                let broken = walk.loops.pop().is_some_and(|l| l.broken);
                truth != Some(true) || broken
            }
            S::For(for_) => {
                if let ast::ForOver::Range(from, to) = &for_.over
                    && self.ed.compare(&walk.env, BinOp::Gt, from, to) == Some(true)
                {
                    walk.dead.push(branch_span(&for_.body));
                    return true;
                }
                walk.loops.push(LoopCtx {
                    label: None,
                    broken: false,
                });
                self.walk_stmt(f, walk, &for_.body, warn);
                walk.loops.pop();
                true
            }
            S::Defer {
                body, ..
            } => {
                // It runs when the scope ends, not here; what is inside is still read.
                self.walk_stmt(f, walk, body, warn);
                true
            }
            S::PushContext(p) => self.walk_stmt(f, walk, &p.body, warn),
            S::Switch(boxed) => self.walk_switch(f, walk, boxed),
            S::StaticIf(boxed) => {
                let crate::ast::StaticIfStmt {
                    then_branch,
                    else_branch,
                    ..
                } = &**boxed;
                match f.static_taken.get(&stmt.span) {
                    Some(true) => self.walk_stmts(f, walk, then_branch, warn),
                    Some(false) => self.walk_stmts(f, walk, else_branch, warn),
                    None => true,
                }
            }
            _ => true,
        }
    }

    fn walk_if(
        &mut self,
        f: &FnCtx,
        walk: &mut Walk,
        cond: &ast::Expr,
        then_branch: &ast::Stmt,
        else_branch: Option<&ast::Stmt>,
        warn: bool,
    ) -> bool {
        let truth = self.walk_truth(walk, cond);
        let mark = walk.env.mark();
        let mut then_exits_normally = false;
        if truth == Some(false) {
            walk.dead.push(branch_span(then_branch));
        } else {
            self.ed.narrow(&mut walk.env, cond, true, then_branch.span);
            then_exits_normally = self.walk_stmt(f, walk, then_branch, warn);
            walk.env.truncate(mark);
        }
        let Some(else_branch) = else_branch else {
            walk.if_exits = (then_exits_normally, true);
            return then_exits_normally || truth != Some(true);
        };
        let mut else_exits_normally = false;
        if truth == Some(true) {
            walk.dead.push(branch_span(else_branch));
        } else {
            self.ed.narrow(&mut walk.env, cond, false, else_branch.span);
            else_exits_normally = self.walk_stmt(f, walk, else_branch, warn);
            walk.env.truncate(mark);
        }
        walk.if_exits = (then_exits_normally, else_exits_normally);
        then_exits_normally || else_exits_normally
    }

    fn walk_switch(&mut self, f: &FnCtx, walk: &mut Walk, switch: &ast::SwitchStmt) -> bool {
        let ast::SwitchStmt {
            value,
            cases,
            complete,
            ..
        } = switch;
        let subject = self.ed.value(&walk.env, value);
        let exhaustive = *complete || cases.iter().any(|c| c.values.is_empty());
        let mut completes = !exhaustive;
        let mut falls_in = false;
        for case in cases {
            let Some(body) = stmts_span(&case.body) else {
                falls_in = case.through && falls_in;
                continue;
            };
            let excluded = !case.values.is_empty()
                && subject.is_some()
                && case
                    .values
                    .iter()
                    .all(|v| self.ed.compare(&walk.env, BinOp::Eq, value, v) == Some(false));
            if excluded && !falls_in {
                walk.dead.push(body);
                falls_in = false;
                continue;
            }
            let mark = walk.env.mark();
            if !falls_in {
                self.ed.narrow_case(&mut walk.env, value, case, body);
            }
            let ends_normally = self.walk_stmts(f, walk, &case.body, false);
            walk.env.truncate(mark);
            falls_in = case.through && ends_normally;
            if !case.through && ends_normally {
                completes = true;
            }
        }
        completes
    }


    /// Whether `cond` always has one truth value where the walk is.
    fn walk_truth(&self, walk: &Walk, cond: &ast::Expr) -> Option<bool> {
        self.ed.value(&walk.env, cond)?.truth()
    }
}
