//! Follow selected cursor edges. A break/return cannot reach the step, while an explicit
//! user index write repairs (or makes unknown) the cursor before its next advance.
use super::{
    BodyState, Cells, Cx, Finding, HashMap, HashSet, Op, Places, Span, Summaries, VecDeque, build,
    finding,
};
use crate::rules::lifetime_movement::Action;

pub(in crate::rules) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    let Some(facts) = cx.compiler.ed.facts() else {
        return;
    };
    let summaries = cx
        .run
        .lifetimes
        .get_or_init(|| Summaries::build_for(cx.compiler, &cx.run.lifetime_files));
    let mut bodies: Vec<_> = facts.lifetime_bodies_in(cx.file).collect();
    bodies.sort_by_key(|b| b.procedure.0);
    for body in bodies {
        let proc = &cx.compiler.pr.procs[body.procedure.0 as usize];
        if proc.body_state != BodyState::Done || !std::sync::Arc::ptr_eq(&proc.lit, &body.syntax) {
            continue;
        }
        let mut places = Places::new(cx.compiler, body);
        let blocks = build(
            cx,
            body,
            &mut places,
            &mut Cells::default(),
            summaries,
            &mut HashMap::default(),
        );
        for (start, b) in blocks.iter().enumerate() {
            let Op::Iteration {
                region,
                index,
                reverse: false,
                name,
                index_name,
                span,
                advance: false,
            } = b.op
            else {
                continue;
            };
            let mut seen = HashSet::default();
            let mut queue = VecDeque::from([(start, None)]);
            let mut bad = HashSet::default();
            while let Some((at, removed)) = queue.pop_front() {
                if !seen.insert((at, removed)) {
                    continue;
                }
                let mut removed = removed;
                match blocks[at].op {
                    Op::IndexWrite(p) if p == index => {
                        removed = None;
                    }
                    Op::Iteration {
                        index: p,
                        advance: true,
                        ..
                    } if p == index => {
                        if let Some(call) = removed {
                            bad.insert(call);
                        }
                        continue;
                    }
                    Op::IterationMutation(r, m, call)
                        if r == region
                            && matches!(
                                m.action,
                                Action::Ordered | Action::Unordered | Action::UnknownRemoval
                            ) =>
                    {
                        removed = Some(call);
                    }
                    _ => {}
                }
                queue.extend(blocks[at].next.iter().map(|&next| (next, removed)));
            }
            let mut bad: Vec<Span> = bad.into_iter().collect();
            bad.sort_unstable_by_key(|s| (s.file.0, s.start, s.end));
            for call in bad {
                let at = if call.file == cx.file {
                    call
                } else {
                    continue;
                };
                let source = cx.src(call);
                let procedure = source.split('(').next().unwrap_or("the call");
                let array = cx.src(span);
                let mut report = finding(
                    at,
                    format!("`{procedure}` changes `{array}` while this `for` walks it"),
                    Some(format!(
                        "the element moved into the freed slot is skipped; use `remove {name};` (or loop backwards with `for < {array}`)"
                    )),
                );
                report.secondary.push((
                    span,
                    "this collection's cursor advances after removal".into(),
                ));
                if standalone_current(cx, call, name, index_name) {
                    report.fix = Some(crate::Fix {
                        title: format!("use `remove {name}`"),
                        edits: vec![crate::Edit {
                            start: at.start as usize,
                            end: at.end as usize,
                            text: format!("remove {name}"),
                        }],
                        machine_applicable: false,
                    });
                }
                if !out
                    .iter()
                    .any(|f| f.start == report.start && f.end == report.end)
                {
                    out.push(report);
                }
            }
        }
    }
}

// Syntax supplies an edit range only; loop membership and cursor reachability come from CFG.
fn standalone_current(
    cx: &Cx,
    call: Span,
    name: jaic::intern::Sym,
    index: jaic::intern::Sym,
) -> bool {
    use jaic::ast::{
        ExprKind as E, StmtKind as S,
        walk::{Node, walk},
    };
    let mut found = false;
    for proc in &cx.procs {
        walk(&proc.body.stmts, &mut |node, chain| {
            if let Node::Expr(expr) = node
                && expr.span == call
                && let E::Call {
                    args, ..
                } = &expr.kind
                && args.len() == 2
                && matches!(args[1].value.kind, E::Ident(n) if n == name || n == index)
                && matches!(chain.last(), Some(Node::Stmt(s)) if matches!(&s.kind, S::Expr(e) if e.span == call))
            {
                found = true;
            }
            !found
        });
    }
    found
}
