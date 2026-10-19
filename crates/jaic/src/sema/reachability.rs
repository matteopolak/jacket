//! Which code of a procedure can never run, for the editor to fade (`IdeFacts::dead_runs`).
//!
//! Two parts. While a body is checked, every integer or bool expression of a recorded file gets
//! an `AInfo`: the interval of values its type and operands allow (constants are single values,
//! computed by the checker itself). Afterwards `warn_unreachable` walks the body's statements
//! once and decides which code control cannot reach: after `return`, `break` and `continue`,
//! after an `if` or `switch` whose every path exits, after a loop that only a `break` leaves,
//! in a branch whose condition always has one value, in a `while` that never starts, and in a
//! `case` its value cannot select.
//!
//! Inside a branch the walk narrows the locals its condition constrains (`if x > 10 { }` makes
//! `x` at least 11 there), but only locals that are never given their address, never written by
//! a macro and not assigned in that branch: those cannot change between the test and the code
//! it guards. When unsure the answer is unknown, and unknown fades nothing.
//!
//! A polymorphic procedure is analysed once per instance, so a run of code counts as dead only
//! if every analysis found it dead (`body_evals` against `dead_runs`).
use super::interval::{self, Interval};
use super::lower::{FnCtx, Operand};
use super::scope::EntityKind;
use super::{Compiler, Diagnostic, EntityId, ScopeId, Span, Sym, TypeId, Value, ast};
use crate::ast::{BinOp, ExprKind as E, StmtKind as S, UnOp};
use crate::types::TypeKind;

/// What an integer or bool expression can evaluate to.
#[derive(Clone, Copy, Debug)]
pub(super) struct AInfo {
    /// The values its operands allow, before narrowing.
    pub iv: Interval,
    /// The values its type has.
    pub range: Interval,
    /// Its type is unsigned (a negative operand then does not mean what it says).
    pub unsigned: bool,
    /// The local it names.
    pub local: Option<EntityId>,
}

/// `a op b` for the intervals given, or `None` when it is not known or may wrap in the result
/// type (`result`).
fn fold_binary(
    op: BinOp,
    (x, xu): (Interval, bool),
    (y, yu): (Interval, bool),
    result: &AInfo,
) -> Option<Interval> {
    let logic = interval::is_comparison(op) || matches!(op, BinOp::And | BinOp::Or);
    // A negative value met an unsigned type: the operation sees it as a different number.
    if !matches!(op, BinOp::And | BinOp::Or) && ((x.lo < 0 && yu) || (y.lo < 0 && xu)) {
        return None;
    }
    if !logic && result.unsigned && (x.lo < 0 || y.lo < 0) {
        return None;
    }
    let r = interval::binary(op, x, y)?;
    (logic || r.within(result.range)).then_some(r)
}

/// How the walk got to where it is.
#[derive(Default)]
struct Walk {
    loops: Vec<LoopCtx>,
    /// Locals narrowed by the branches being walked, innermost last.
    env: Vec<(EntityId, Interval)>,
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
    fn narrowed(&self, local: EntityId) -> Option<Interval> {
        self.env
            .iter()
            .rev()
            .find(|(e, _)| *e == local)
            .map(|(_, i)| *i)
    }

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
pub(super) fn runs_in_order(stmt: &ast::Stmt) -> bool {
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

fn negate(op: BinOp) -> BinOp {
    match op {
        BinOp::Lt => BinOp::Ge,
        BinOp::Le => BinOp::Gt,
        BinOp::Gt => BinOp::Le,
        BinOp::Ge => BinOp::Lt,
        BinOp::Eq => BinOp::Ne,
        BinOp::Ne => BinOp::Eq,
        other => other,
    }
}

/// `a op b` as `b op' a`.
fn flip(op: BinOp) -> BinOp {
    match op {
        BinOp::Lt => BinOp::Gt,
        BinOp::Le => BinOp::Ge,
        BinOp::Gt => BinOp::Lt,
        BinOp::Ge => BinOp::Le,
        other => other,
    }
}

impl Compiler {
    // ----- facts recorded while checking -----

    /// Note what the integer or bool expression `expr`, which checked to `op`, can be.
    pub(super) fn ide_note_ainfo(&mut self, expr: &ast::Expr, op: &Operand) {
        if self.sketching() || !self.ide_wants_pub(expr.span.file) {
            return;
        }
        let info = self.compute_ainfo(expr, op);
        self.ide_note_macro_writes(expr);
        if let Some(ide) = self.ide.as_mut() {
            match info {
                Some(info) => {
                    ide.ainfo.insert(expr.span, info);
                }
                None => {
                    ide.ainfo.remove(&expr.span);
                }
            }
        }
    }

    /// The values of `ty` and whether it is unsigned, if it is a number or bool they are known for.
    fn value_range(&self, ty: TypeId) -> Option<(Interval, bool)> {
        match self.types.kind(self.types.repr(ty)) {
            TypeKind::Bool => Some((Interval::BOOL, true)),
            TypeKind::Int {
                bits,
                signed,
            } => Some((Interval::of_type(*bits, *signed)?, !*signed)),
            _ => None,
        }
    }

    fn compute_ainfo(&self, expr: &ast::Expr, op: &Operand) -> Option<AInfo> {
        let ide = self.ide.as_ref()?;
        if let Operand::Const {
            value,
            ty,
            untyped,
        } = op
        {
            let v = match value {
                Value::Int(i) => *i,
                Value::Bool(b) => *b as i128,
                _ => return None,
            };
            let (range, unsigned) = self
                .value_range(*ty)
                .unwrap_or((Interval::of_type(64, true)?, false));
            return Some(AInfo {
                iv: Interval::point(v),
                range,
                unsigned: unsigned && !*untyped,
                local: None,
            });
        }
        let (range, unsigned) = self.value_range(op.ty())?;
        let mut info = AInfo {
            iv: range,
            range,
            unsigned,
            local: None,
        };
        let child = |e: &ast::Expr| ide.ainfo.get(&e.span).map(|a| (a.iv, a.unsigned));
        match &expr.kind {
            E::Ident(_) => {
                info.local = ide
                    .last_entity
                    .filter(|(span, _)| *span == expr.span)
                    .map(|(_, id)| id)
                    .filter(|id| matches!(self.entity(*id).kind, EntityKind::Local { .. }));
            }
            E::Binary(bin, a, b) => {
                if let (Some(x), Some(y)) = (child(a), child(b))
                    && let Some(r) = fold_binary(*bin, x, y, &info)
                {
                    info.iv = r;
                }
            }
            E::Unary(un, a) => {
                if let Some((x, _)) = child(a)
                    && let Some(r) = interval::unary(*un, x, range)
                    && (matches!(un, UnOp::Not) || r.within(range))
                {
                    info.iv = r;
                }
            }
            E::Cast {
                value, ..
            } => {
                if let Some((x, _)) = child(value)
                    && x.within(range)
                {
                    info.iv = x;
                }
            }
            _ => {}
        }
        Some(info)
    }

    /// Taking a local's address, or naming it from a macro (which may assign it), means calls
    /// and macros can change it behind any test.
    fn ide_note_macro_writes(&mut self, expr: &ast::Expr) {
        let (E::Unary(UnOp::Star, target) | E::Backtick(target)) = &expr.kind else {
            return;
        };
        let id = match (
            &target.kind,
            self.ide.as_ref().and_then(|i| i.ainfo.get(&target.span)),
        ) {
            (E::Ident(_), Some(info)) => info.local,
            (E::Ident(_), None) => self
                .ide
                .as_ref()
                .and_then(|i| i.last_entity)
                .filter(|(span, _)| *span == target.span)
                .map(|(_, id)| id),
            _ => None,
        };
        if let Some(id) = id
            && let Some(ide) = self.ide.as_mut()
        {
            ide.addr_taken.insert(id);
        }
    }

    /// An assignment statement at `span` wrote the variables `lhs` names.
    pub(super) fn ide_note_assign(&mut self, scope: ScopeId, lhs: &[ast::Expr], span: Span) {
        if self.ide.is_none() || self.sketching() {
            return;
        }
        for target in lhs {
            let mut root = target;
            while let E::Backtick(inner) = &root.kind {
                root = inner;
            }
            if let E::Ident(name) = &root.kind
                && let Ok(ids) = self.lookup(scope, *name)
                && let Some(ide) = self.ide.as_mut()
            {
                for id in ids {
                    ide.assigned.entry(id).or_default().push(span);
                }
            }
        }
    }

    /// Whether the condition of an `ifx` of the recorded files always has one truth value.
    pub(super) fn ide_note_ifx(
        &mut self,
        cond: &ast::Expr,
        then_value: Option<&ast::Expr>,
        else_value: Option<&ast::Expr>,
    ) {
        if self.ide.is_none() || self.sketching() {
            return;
        }
        let truth = self
            .ide
            .as_ref()
            .and_then(|i| i.ainfo.get(&cond.span))
            .and_then(|a| a.iv.truth());
        if let Some(then) = then_value {
            self.ide_note_ran(then.span, truth != Some(false));
        }
        if let Some(other) = else_value {
            self.ide_note_ran(other.span, truth != Some(true));
        }
    }

    // ----- the walk -----

    /// Walk `body` for code that cannot run: record it for the editor, and warn at the first
    /// statement after a `return`, `break` or `continue` of its block, as Jai does.
    ///
    /// Returns whether control can reach the end of the body.
    pub(super) fn warn_unreachable(&mut self, f: &FnCtx, body: &ast::Block) -> bool {
        let mut walk = Walk::default();
        let completes = self.walk_stmts(f, &mut walk, &body.stmts, true);
        if self.sketching() || !self.ide_wants_pub(body.span.file) {
            return completes;
        }
        walk.dead.sort_by_key(|s| (s.start, s.end));
        walk.dead.dedup();
        if let Some(ide) = self.ide.as_mut() {
            *ide.body_evals.entry(body.span).or_default() += 1;
            for run in walk.dead {
                *ide.dead_runs.entry((body.span, run)).or_default() += 1;
            }
        }
        completes
    }

    /// Whether control can leave the end of `stmts`.
    fn walk_stmts(&mut self, f: &FnCtx, walk: &mut Walk, stmts: &[ast::Stmt], warn: bool) -> bool {
        let mark = walk.env.len();
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
                        (false, true) => self.narrow(walk, cond, false, rest),
                        (true, false) => self.narrow(walk, cond, true, rest),
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
                let mark = walk.env.len();
                self.narrow(walk, &w.cond, true, w.body.span);
                self.walk_stmt(f, walk, &w.body, warn);
                walk.env.truncate(mark);
                let broken = walk.loops.pop().is_some_and(|l| l.broken);
                truth != Some(true) || broken
            }
            S::For(for_) => {
                if let ast::ForOver::Range(from, to) = &for_.over
                    && self.compare_exprs(walk, BinOp::Gt, from, to) == Some(true)
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
        let mark = walk.env.len();
        let mut then_exits_normally = false;
        if truth == Some(false) {
            walk.dead.push(branch_span(then_branch));
        } else {
            self.narrow(walk, cond, true, then_branch.span);
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
            self.narrow(walk, cond, false, else_branch.span);
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
        let subject = self.eval(walk, value);
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
                    .all(|v| self.compare_exprs(walk, BinOp::Eq, value, v) == Some(false));
            if excluded && !falls_in {
                walk.dead.push(body);
                falls_in = false;
                continue;
            }
            let mark = walk.env.len();
            if !falls_in {
                self.narrow_case(walk, value, case, body);
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

    // ----- values -----

    fn walk_truth(&self, walk: &Walk, cond: &ast::Expr) -> Option<bool> {
        self.eval(walk, cond)?.truth()
    }

    fn compare_exprs(&self, walk: &Walk, op: BinOp, a: &ast::Expr, b: &ast::Expr) -> Option<bool> {
        let (ia, ib) = (self.eval(walk, a)?, self.eval(walk, b)?);
        let ide = self.ide.as_ref()?;
        let (ua, ub) = (
            ide.ainfo.get(&a.span)?.unsigned,
            ide.ainfo.get(&b.span)?.unsigned,
        );
        if (ia.lo < 0 && ub) || (ib.lo < 0 && ua) {
            return None;
        }
        interval::compare(op, ia, ib)
    }

    /// What `expr` can be where the walk is: its checked interval, tightened by the narrowing
    /// of the branches around it.
    fn eval(&self, walk: &Walk, expr: &ast::Expr) -> Option<Interval> {
        let ide = self.ide.as_ref()?;
        let info = *ide.ainfo.get(&expr.span)?;
        let flags = |e: &ast::Expr| ide.ainfo.get(&e.span).map(|a| a.unsigned);
        let tighter = match &expr.kind {
            E::Ident(_) => info.local.and_then(|l| walk.narrowed(l)),
            E::Binary(op, a, b) => {
                let (x, y) = (self.eval(walk, a)?, self.eval(walk, b)?);
                fold_binary(*op, (x, flags(a)?), (y, flags(b)?), &info)
            }
            E::Unary(op, a) => {
                let x = self.eval(walk, a)?;
                interval::unary(*op, x, info.range)
                    .filter(|r| matches!(op, UnOp::Not) || r.within(info.range))
            }
            E::Cast {
                value, ..
            } => self.eval(walk, value).filter(|x| x.within(info.range)),
            _ => None,
        };
        Some(tighter.and_then(|t| t.meet(info.iv)).unwrap_or(info.iv))
    }

    // ----- narrowing -----

    /// Whether `local` keeps the value it has at a test through `region`.
    fn narrowable(&self, local: EntityId, region: Span) -> bool {
        let Some(ide) = self.ide.as_ref() else {
            return false;
        };
        !ide.addr_taken.contains(&local)
            && ide.assigned.get(&local).is_none_or(|writes| {
                writes.iter().all(|w| {
                    w.file == region.file && (w.end <= region.start || w.start >= region.end)
                })
            })
    }

    /// Enter `region`, which runs when `cond` is `holds`: narrow the locals it constrains.
    fn narrow(&self, walk: &mut Walk, cond: &ast::Expr, holds: bool, region: Span) {
        match &cond.kind {
            E::Unary(UnOp::Not, inner) => self.narrow(walk, inner, !holds, region),
            E::Binary(BinOp::And, a, b) if holds => {
                self.narrow(walk, a, true, region);
                self.narrow(walk, b, true, region);
            }
            E::Binary(BinOp::Or, a, b) if !holds => {
                self.narrow(walk, a, false, region);
                self.narrow(walk, b, false, region);
            }
            E::Binary(op, a, b) if interval::is_comparison(*op) => {
                let op = if holds {
                    *op
                } else {
                    negate(*op)
                };
                self.constrain(walk, a, op, b, region);
                self.constrain(walk, b, flip(op), a, region);
            }
            _ => {}
        }
    }

    /// `side op other` holds: `side`, when it names a local, is in the interval that leaves.
    fn constrain(
        &self,
        walk: &mut Walk,
        side: &ast::Expr,
        op: BinOp,
        other: &ast::Expr,
        region: Span,
    ) {
        let E::Ident(_) = side.kind else {
            return;
        };
        let Some(ide) = self.ide.as_ref() else {
            return;
        };
        let (Some(info), Some(other_info)) =
            (ide.ainfo.get(&side.span), ide.ainfo.get(&other.span))
        else {
            return;
        };
        let Some(local) = info.local else {
            return;
        };
        let (Some(cur), Some(o)) = (self.eval(walk, side), self.eval(walk, other)) else {
            return;
        };
        if !self.narrowable(local, region)
            || (o.lo < 0 && info.unsigned)
            || (cur.lo < 0 && other_info.unsigned)
        {
            return;
        }
        let (lo, hi) = match op {
            BinOp::Lt => (cur.lo, cur.hi.min(o.hi.saturating_sub(1))),
            BinOp::Le => (cur.lo, cur.hi.min(o.hi)),
            BinOp::Gt => (cur.lo.max(o.lo.saturating_add(1)), cur.hi),
            BinOp::Ge => (cur.lo.max(o.lo), cur.hi),
            BinOp::Eq => (cur.lo.max(o.lo), cur.hi.min(o.hi)),
            BinOp::Ne => match o.singleton() {
                Some(v) if v == cur.lo => (cur.lo.saturating_add(1), cur.hi),
                Some(v) if v == cur.hi => (cur.lo, cur.hi.saturating_sub(1)),
                _ => return,
            },
            _ => return,
        };
        if let Some(narrower) = Interval::new(lo, hi) {
            walk.env.push((local, narrower));
        }
    }

    /// Inside `case` the switched value is one of the case's values.
    fn narrow_case(&self, walk: &mut Walk, value: &ast::Expr, case: &ast::Case, body: Span) {
        let E::Ident(_) = value.kind else {
            return;
        };
        let Some(local) = self
            .ide
            .as_ref()
            .and_then(|i| i.ainfo.get(&value.span))
            .and_then(|a| a.local)
        else {
            return;
        };
        if case.values.is_empty() || !self.narrowable(local, body) {
            return;
        }
        let mut hull: Option<Interval> = None;
        for v in &case.values {
            let Some(i) = self.eval(walk, v).filter(|i| i.singleton().is_some()) else {
                return;
            };
            hull = Some(match hull {
                Some(h) => Interval {
                    lo: h.lo.min(i.lo),
                    hi: h.hi.max(i.hi),
                },
                None => i,
            });
        }
        if let Some(h) = hull {
            walk.env.push((local, h));
        }
    }
}
