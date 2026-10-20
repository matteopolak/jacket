//! What an editor analysis knows about the values of integer and bool expressions, for the
//! walk that finds code which cannot run (`sema::reachability`).
//!
//! While a body is checked, every integer or bool expression of a recorded file gets an `AInfo`:
//! the interval of values its type and operands allow (constants are single values, computed by
//! the checker itself). The walk then asks what a condition can be where it is (`Editor::value`)
//! and narrows the locals a branch's condition constrains (`if x > 10 { }` makes `x` at least 11
//! there), but only locals that are never given their address, never written by a macro and not
//! assigned in that branch: those cannot change between the test and the code it guards. When
//! unsure the answer is unknown, and unknown fades nothing. A batch compile records nothing, so
//! every answer is unknown there.
use super::interval::{self, Interval};
use super::{Editor, IdeFacts};
use crate::ast::{self, BinOp, ExprKind as E, UnOp};
use crate::sema::lower::Operand;
use crate::sema::scope::EntityKind;
use crate::sema::{Compiler, EntityId, ScopeId, Value};
use crate::source::Span;
use crate::types::{TypeId, TypeKind};

/// What an integer or bool expression can evaluate to.
#[derive(Clone, Copy, Debug)]
pub(crate) struct AInfo {
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

/// Locals narrowed by the branches being walked, innermost last.
#[derive(Default)]
pub(crate) struct Narrowing(Vec<(EntityId, Interval)>);

impl Narrowing {
    /// Where to `truncate` to when the branch being entered is left.
    pub(crate) fn mark(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn truncate(&mut self, mark: usize) {
        self.0.truncate(mark);
    }

    fn narrowed(&self, local: EntityId) -> Option<Interval> {
        self.0
            .iter()
            .rev()
            .find(|(e, _)| *e == local)
            .map(|(_, i)| *i)
    }
}

impl IdeFacts {
    fn compare(&self, n: &Narrowing, op: BinOp, a: &ast::Expr, b: &ast::Expr) -> Option<bool> {
        let (ia, ib) = (self.eval(n, a)?, self.eval(n, b)?);
        let ide = self;
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
    fn eval(&self, n: &Narrowing, expr: &ast::Expr) -> Option<Interval> {
        let ide = self;
        let info = *ide.ainfo.get(&expr.span)?;
        let flags = |e: &ast::Expr| ide.ainfo.get(&e.span).map(|a| a.unsigned);
        let tighter = match &expr.kind {
            E::Ident(_) => info.local.and_then(|l| n.narrowed(l)),
            E::Binary(op, a, b) => {
                let (x, y) = (self.eval(n, a)?, self.eval(n, b)?);
                fold_binary(*op, (x, flags(a)?), (y, flags(b)?), &info)
            }
            E::Unary(op, a) => {
                let x = self.eval(n, a)?;
                interval::unary(*op, x, info.range)
                    .filter(|r| matches!(op, UnOp::Not) || r.within(info.range))
            }
            E::Cast {
                value, ..
            } => self.eval(n, value).filter(|x| x.within(info.range)),
            _ => None,
        };
        Some(tighter.and_then(|t| t.meet(info.iv)).unwrap_or(info.iv))
    }

    /// Whether `local` keeps the value it has at a test through `region`.
    fn narrowable(&self, local: EntityId, region: Span) -> bool {
        let ide = self;
        !ide.addr_taken.contains(&local)
            && ide.assigned.get(&local).is_none_or(|writes| {
                writes.iter().all(|w| {
                    w.file == region.file && (w.end <= region.start || w.start >= region.end)
                })
            })
    }

    /// Enter `region`, which runs when `cond` is `holds`: narrow the locals it constrains.
    fn narrow(&self, n: &mut Narrowing, cond: &ast::Expr, holds: bool, region: Span) {
        match &cond.kind {
            E::Unary(UnOp::Not, inner) => self.narrow(n, inner, !holds, region),
            E::Binary(BinOp::And, a, b) if holds => {
                self.narrow(n, a, true, region);
                self.narrow(n, b, true, region);
            }
            E::Binary(BinOp::Or, a, b) if !holds => {
                self.narrow(n, a, false, region);
                self.narrow(n, b, false, region);
            }
            E::Binary(op, a, b) if interval::is_comparison(*op) => {
                let op = if holds {
                    *op
                } else {
                    negate(*op)
                };
                self.constrain(n, a, op, b, region);
                self.constrain(n, b, flip(op), a, region);
            }
            _ => {}
        }
    }

    /// `side op other` holds: `side`, when it names a local, is in the interval that leaves.
    fn constrain(
        &self,
        n: &mut Narrowing,
        side: &ast::Expr,
        op: BinOp,
        other: &ast::Expr,
        region: Span,
    ) {
        let E::Ident(_) = side.kind else {
            return;
        };
        let ide = self;
        let (Some(info), Some(other_info)) =
            (ide.ainfo.get(&side.span), ide.ainfo.get(&other.span))
        else {
            return;
        };
        let Some(local) = info.local else {
            return;
        };
        let (Some(cur), Some(o)) = (self.eval(n, side), self.eval(n, other)) else {
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
            n.0.push((local, narrower));
        }
    }

    /// Inside `case` the switched value is one of the case's values.
    fn narrow_case(&self, n: &mut Narrowing, value: &ast::Expr, case: &ast::Case, body: Span) {
        let E::Ident(_) = value.kind else {
            return;
        };
        let Some(local) = self.ainfo.get(&value.span).and_then(|a| a.local) else {
            return;
        };
        if case.values.is_empty() || !self.narrowable(local, body) {
            return;
        }
        let mut hull: Option<Interval> = None;
        for v in &case.values {
            let Some(i) = self.eval(n, v).filter(|i| i.singleton().is_some()) else {
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
            n.0.push((local, h));
        }
    }
}

impl Editor {
    /// What `expr` can be where the walk is: its checked interval, tightened by the narrowing
    /// `n` of the branches around it.
    pub(crate) fn value(&self, n: &Narrowing, expr: &ast::Expr) -> Option<Interval> {
        self.facts()?.eval(n, expr)
    }

    /// Whether `a op b` always (or never) holds where the walk is.
    pub(crate) fn compare(
        &self,
        n: &Narrowing,
        op: BinOp,
        a: &ast::Expr,
        b: &ast::Expr,
    ) -> Option<bool> {
        self.facts()?.compare(n, op, a, b)
    }

    /// Enter `region`, which runs when `cond` is `holds`: narrow the locals it constrains.
    pub(crate) fn narrow(&self, n: &mut Narrowing, cond: &ast::Expr, holds: bool, region: Span) {
        if let Some(ide) = self.facts() {
            ide.narrow(n, cond, holds, region);
        }
    }

    /// Enter the `body` of `case`, where the switched `value` is one of the case's values.
    pub(crate) fn narrow_case(
        &self,
        n: &mut Narrowing,
        value: &ast::Expr,
        case: &ast::Case,
        body: Span,
    ) {
        if let Some(ide) = self.facts() {
            ide.narrow_case(n, value, case, body);
        }
    }

    /// A walk of the body at `body` found the runs `dead` that cannot run. A polymorphic
    /// procedure is walked once per instance, so a run counts as dead only if every walk found
    /// it dead (`body_evals` against `dead_runs`).
    pub(crate) fn body_walked(&mut self, body: Span, mut dead: Vec<Span>) {
        let Some(ide) = self.facts_mut() else {
            return;
        };
        dead.sort_by_key(|s| (s.start, s.end));
        dead.dedup();
        *ide.body_evals.entry(body).or_default() += 1;
        for run in dead {
            *ide.dead_runs.entry((body, run)).or_default() += 1;
        }
    }
}

impl Compiler {
    /// Note what the integer or bool expression `expr`, which checked to `op`, can be.
    pub(crate) fn ide_note_ainfo(&mut self, expr: &ast::Expr, op: &Operand) {
        if self.ed.sketching() || !self.ide_wants_pub(expr.span.file) {
            return;
        }
        let info = self.compute_ainfo(expr, op);
        self.ide_note_macro_writes(expr);
        if let Some(ide) = self.ed.facts_mut() {
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
        match self.ty.types.kind(self.ty.types.repr(ty)) {
            TypeKind::Bool => Some((Interval::BOOL, true)),
            TypeKind::Int {
                bits,
                signed,
            } => Some((Interval::of_type(*bits, *signed)?, !*signed)),
            _ => None,
        }
    }

    fn compute_ainfo(&self, expr: &ast::Expr, op: &Operand) -> Option<AInfo> {
        let ide = self.ed.facts()?;
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
            self.ed.facts().and_then(|i| i.ainfo.get(&target.span)),
        ) {
            (E::Ident(_), Some(info)) => info.local,
            (E::Ident(_), None) => self
                .ed
                .facts()
                .and_then(|i| i.last_entity)
                .filter(|(span, _)| *span == target.span)
                .map(|(_, id)| id),
            _ => None,
        };
        if let Some(id) = id
            && let Some(ide) = self.ed.facts_mut()
        {
            ide.addr_taken.insert(id);
        }
    }

    /// An assignment statement at `span` wrote the variables `lhs` names.
    pub(crate) fn ide_note_assign(&mut self, scope: ScopeId, lhs: &[ast::Expr], span: Span) {
        if !self.ed.is_live() || self.ed.sketching() {
            return;
        }
        for target in lhs {
            let mut root = target;
            while let E::Backtick(inner) = &root.kind {
                root = inner;
            }
            if let E::Ident(name) = &root.kind
                && let Ok(ids) = self.lookup(scope, *name)
                && let Some(ide) = self.ed.facts_mut()
            {
                for id in ids {
                    ide.assigned.entry(id).or_default().push(span);
                }
            }
        }
    }

    /// Whether the condition of an `ifx` of the recorded files always has one truth value.
    pub(crate) fn ide_note_ifx(
        &mut self,
        cond: &ast::Expr,
        then_value: Option<&ast::Expr>,
        else_value: Option<&ast::Expr>,
    ) {
        if !self.ed.is_live() || self.ed.sketching() {
            return;
        }
        let truth = self
            .ed
            .facts()
            .and_then(|i| i.ainfo.get(&cond.span))
            .and_then(|a| a.iv.truth());
        if let Some(then) = then_value {
            self.ide_note_ran(then.span, truth != Some(false));
        }
        if let Some(other) = else_value {
            self.ide_note_ran(other.span, truth != Some(true));
        }
    }
}
