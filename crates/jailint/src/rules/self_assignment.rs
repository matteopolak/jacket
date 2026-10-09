//! `self_assignment`: `x = x;`, `p.pos = p.pos;`.
//!
//! Assigning a value to itself does nothing. It is almost always a slip: a local copied to a
//! field of the same name after `using info;` (where both sides now name the field), or one
//! side of a copy pasted wrong.
//!
//! Fires on a plain `=` with one target whose two sides are the same side-effect-free
//! expression (names, member paths, indexing; no calls), spelled the same. Macros and
//! polymorphic bodies are checked too: the statement does nothing in every instance. The
//! editor offers to remove the statement; `--fix` does not, since the slip may be the other
//! side.
use super::{finding, same_pure};
use crate::syntax::{Cx, walk};
use crate::{Edit, Finding, Fix};
use jaic::ast::{AssignOp, StmtKind as S};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        walk(&p.body.stmts, &mut |n, _| {
            let Some(s) = n.stmt() else {
                return true;
            };
            if let S::Assign {
                op: AssignOp::Assign,
                lhs,
                rhs,
            } = &s.kind
                && let ([l], [r]) = (lhs.as_slice(), rhs.as_slice())
                && same_pure(cx, l, r)
            {
                let target = cx.whole_src(l.span).trim();
                let mut found = finding(
                    s.span,
                    format!("`{target}` is assigned to itself"),
                    Some("this does nothing; was another value or target meant? (after `using`, both sides name the same field)".into()),
                );
                // The statement's own `;` goes with it.
                let start = s.span.start as usize;
                let mut end = s.span.end as usize;
                let rest = cx.text.get(end..).unwrap_or("");
                let skipped = rest.len() - rest.trim_start().len();
                if rest[skipped..].starts_with(';') {
                    end += skipped + 1;
                }
                found.fix = Some(Fix {
                    title: "remove the statement".into(),
                    edits: vec![Edit {
                        start,
                        end,
                        text: String::new(),
                    }],
                    machine_applicable: false,
                });
                out.push(found);
            }
            true
        });
    }
}
