//! Procedures that never return, so that a call to one ends its block.
//!
//! A procedure never returns when no path through its body reaches a `return` or the body's end:
//! every path ends in a call to a procedure that never returns. The roots are the C runtime's
//! process and thread exits, found by the symbol a body-less `#foreign` declaration names
//! (`NEVER_RETURNING_SYMBOLS`); the stdlib's own `exit` and the procedures that call it follow.
//!
//! The answer is a fixed point over the procedures whose bodies were checked, computed once
//! after an editor analysis (`ide_check_all`): a procedure joins the set when its body, read
//! with the set so far, cannot complete, and only joins it, so a recursive procedure is never
//! assumed to diverge. The read is syntactic: every branch counts as possible, which can only
//! make a procedure look as if it returns. The calls come from `IdeFacts::callees`, the
//! procedure each call expression of the analysis resolved to (nothing when instances of a
//! polymorphic procedure disagree). A macro can `return` from its caller, so a procedure that
//! expanded such a macro is never in the set, and macro bodies are not read.
//!
//! The set then extends the dead code of `reachability.rs`: the statements after a call that
//! never returns are recorded as dead runs of every body that was analysed, which is what the
//! editor fades and what `jailint` consults.
use super::IdeFacts;
use crate::ast::{self, ExprKind as E, StmtKind as S};
use crate::fxhash::{HashMap, HashSet};
use crate::intern::Sym;
use crate::sema::procs::ProcTarget;
use crate::sema::reachability::runs_in_order;
use crate::sema::{Compiler, ProcId};
use crate::source::Span;

/// The C symbols of the process and thread exits a body-less `#foreign` procedure can name.
const NEVER_RETURNING_SYMBOLS: &[&str] = &[
    "exit",
    "_exit",
    "_Exit",
    "quick_exit",
    "abort",
    "exit_group",
    "pthread_exit",
    "ExitProcess",
    "ExitThread",
];

struct Loop {
    label: Option<Sym>,
    broken: bool,
}

/// A syntactic read of one body against the procedures known not to return.
struct Reading<'a> {
    callees: &'a HashMap<Span, Option<ProcId>>,
    never: &'a HashSet<ProcId>,
    /// The procedures its calls named, in statement position.
    called: Vec<ProcId>,
    /// A `return` can run.
    returned: bool,
    loops: Vec<Loop>,
    /// The runs after a statement control cannot leave.
    dead: Vec<Span>,
}

impl<'a> Reading<'a> {
    fn new(callees: &'a HashMap<Span, Option<ProcId>>, never: &'a HashSet<ProcId>) -> Self {
        Reading {
            callees,
            never,
            called: Vec::new(),
            returned: false,
            loops: Vec::new(),
            dead: Vec::new(),
        }
    }

    /// Whether control can leave the end of `stmts`.
    fn stmts(&mut self, stmts: &[ast::Stmt]) -> bool {
        for (i, stmt) in stmts.iter().enumerate() {
            if self.stmt(stmt) {
                continue;
            }
            if let (Some(next), Some(last)) = (
                stmts[i + 1..].iter().find(|s| runs_in_order(s)),
                stmts.last(),
            ) {
                self.dead.push(next.span.to(last.span));
            }
            return false;
        }
        true
    }

    fn broke(&mut self, label: Option<Sym>) {
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

    /// Whether control can leave the end of `stmt`.
    fn stmt(&mut self, stmt: &ast::Stmt) -> bool {
        match &stmt.kind {
            S::Return {
                ..
            } => {
                self.returned = true;
                false
            }
            S::Continue(_) => false,
            S::Break(label) => {
                self.broke(label.as_ref().map(|l| l.name));
                false
            }
            S::Block(b) => self.stmts(&b.stmts),
            S::If {
                then_branch,
                else_branch,
                ..
            } => {
                let then = self.stmt(then_branch);
                let other = else_branch.as_deref().is_none_or(|e| self.stmt(e));
                then || other
            }
            S::While(w) => {
                self.loops.push(Loop {
                    label: w.label.as_ref().map(|l| l.name),
                    broken: false,
                });
                self.stmt(&w.body);
                let broken = self.loops.pop().is_some_and(|l| l.broken);
                !matches!(w.cond.kind, E::Bool(true)) || broken
            }
            S::For(for_) => {
                self.loops.push(Loop {
                    label: None,
                    broken: false,
                });
                self.stmt(&for_.body);
                self.loops.pop();
                true
            }
            S::Defer {
                body, ..
            } => {
                self.stmt(body);
                true
            }
            S::PushContext(p) => self.stmt(&p.body),
            S::Switch(switch) => {
                let exhaustive =
                    switch.complete || switch.cases.iter().any(|c| c.values.is_empty());
                let mut completes = !exhaustive;
                for (i, case) in switch.cases.iter().enumerate() {
                    let ends = self.stmts(&case.body);
                    completes |= ends && (!case.through || i + 1 == switch.cases.len());
                }
                completes
            }
            S::StaticIf(boxed) => {
                let then = self.stmts(&boxed.then_branch);
                let other = self.stmts(&boxed.else_branch);
                then || other
            }
            S::Expr(expr) if matches!(expr.kind, E::Call { .. }) => {
                let Some(Some(callee)) = self.callees.get(&expr.span) else {
                    return true;
                };
                self.called.push(*callee);
                !self.never.contains(callee)
            }
            _ => true,
        }
    }
}

impl Compiler {
    /// Whether a procedure is a body-less declaration of a process or thread exit.
    fn is_exit_primitive(&self, id: ProcId) -> bool {
        let proc = self.proc(id);
        let Some(ProcTarget::Foreign(foreign)) = proc.target else {
            return false;
        };
        proc.lit.body.is_none()
            && proc.sig.as_ref().is_some_and(|sig| sig.returns.is_empty())
            && NEVER_RETURNING_SYMBOLS.contains(
                &self.out.program.foreigns[foreign.0 as usize]
                    .symbol
                    .as_str(),
            )
    }

    /// Compute the procedures that never return from what the analysis checked, and record the
    /// code after a call to one as dead in the bodies analysed. See the module documentation.
    pub(crate) fn ide_note_never_returns(&mut self) {
        let Some(ide) = self.ed.stop() else {
            return;
        };
        let mut ide = ide;
        let mut never: HashSet<ProcId> = (0..self.procs.len())
            .map(|i| ProcId(i as u32))
            .filter(|&id| self.is_exit_primitive(id))
            .collect();
        let candidates: Vec<ProcId> = (0..self.procs.len())
            .map(|i| ProcId(i as u32))
            .filter(|&id| {
                let proc = self.proc(id);
                proc.lit.body.is_some()
                    && !proc.is_macro
                    && !proc.is_poly
                    && !never.contains(&id)
                    && !ide.macro_returners.contains(&id)
            })
            .collect();
        // Who to read again when a procedure joins the set.
        let mut callers: HashMap<ProcId, Vec<ProcId>> = HashMap::default();
        let mut work = Vec::new();
        for &id in &candidates {
            let body = self
                .proc(id)
                .lit
                .body
                .as_ref()
                .expect("candidates have bodies");
            let mut reading = Reading::new(&ide.callees, &never);
            let completes = reading.stmts(&body.stmts);
            for callee in reading.called {
                callers.entry(callee).or_default().push(id);
            }
            if !completes && !reading.returned {
                never.insert(id);
                work.push(id);
            }
        }
        while let Some(done) = work.pop() {
            for &id in callers.get(&done).into_iter().flatten() {
                if never.contains(&id) {
                    continue;
                }
                let body = self
                    .proc(id)
                    .lit
                    .body
                    .as_ref()
                    .expect("candidates have bodies");
                let mut reading = Reading::new(&ide.callees, &never);
                let completes = reading.stmts(&body.stmts);
                if !completes && !reading.returned {
                    never.insert(id);
                    work.push(id);
                }
            }
        }
        self.record_dead_after_exits(&mut ide, &candidates, &never);
        self.ed.resume(ide);
    }

    fn record_dead_after_exits(
        &self,
        ide: &mut IdeFacts,
        candidates: &[ProcId],
        never: &HashSet<ProcId>,
    ) {
        // Procedures that expand a macro that returns are read like the others: the macro's
        // `return` is not a statement of the body, so they are skipped for the dead runs too.
        let mut seen = HashSet::default();
        for &id in candidates {
            if ide.macro_returners.contains(&id) {
                continue;
            }
            let body = self
                .proc(id)
                .lit
                .body
                .as_ref()
                .expect("candidates have bodies");
            let Some(&evals) = ide.body_evals.get(&body.span) else {
                continue;
            };
            if !seen.insert(body.span) {
                continue;
            }
            let mut reading = Reading::new(&ide.callees, never);
            reading.stmts(&body.stmts);
            for run in reading.dead {
                let count = ide.dead_runs.entry((body.span, run)).or_default();
                *count = (*count).max(evals);
            }
        }
    }
}
