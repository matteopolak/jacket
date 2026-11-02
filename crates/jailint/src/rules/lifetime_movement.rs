//! Element identity contracts. Only primitives and explicit semantic notes seed effects;
//! ordinary wrappers receive substituted parameter-relative summaries.
pub(super) mod facts;

use jaic::ast::Note;
use jaic::ir::Val;
use jaic::sema::{Compiler, ProcId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Selector {
    Formal(usize),
    Constant(i128),
    Local(Val),
    Result(usize),
    Last,
    CountDelta(i128),
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Action {
    Ordered,
    Unordered,
    Insert,
    Pop,
    Reset,
    Swap,
    Reverse,
    Sort,
    Key,
    UnknownRemoval,
    Count,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct Mutation {
    pub action: Action,
    pub first: Selector,
    pub second: Selector,
}

impl Mutation {
    pub fn new(action: Action, first: Selector) -> Self {
        Self {
            action,
            first,
            second: Selector::Unknown,
        }
    }

    pub fn guaranteed(self) -> bool {
        !matches!(self.action, Action::Sort | Action::UnknownRemoval)
    }
}

pub(super) fn notes(compiler: &Compiler, proc: ProcId) -> impl Iterator<Item = &Note> {
    compiler.pr.procs[proc.0 as usize]
        .lit
        .header
        .notes
        .iter()
        .chain(compiler.pr.proc_decl_notes.get(&proc).into_iter().flatten())
}

pub(super) fn parameters<'a>(text: &'a str, name: &str) -> Option<Vec<&'a str>> {
    let text = text.trim();
    if text == name {
        return Some(Vec::new());
    }
    let args = text
        .strip_prefix(name)?
        .strip_prefix('(')?
        .strip_suffix(')')?;
    Some(args.split(',').map(str::trim).collect())
}

pub(super) fn released_formal(compiler: &Compiler, proc: ProcId) -> Option<usize> {
    notes(compiler, proc).find_map(|n| {
        let args =
            parameters(&n.text, "FreesArgument").or_else(|| parameters(&n.text, "Closes"))?;
        if args.is_empty() {
            Some(0)
        } else {
            args[0].parse().ok()
        }
    })
}

pub(super) fn contracts(compiler: &Compiler, proc: ProcId) -> Vec<(usize, Mutation)> {
    notes(compiler, proc)
        .filter_map(|n| {
            if let Some(args) = parameters(&n.text, "RemovesFromArray") {
                let action = match args.first().copied() {
                    Some("ordered") => Action::Ordered,
                    Some("unordered") => Action::Unordered,
                    Some("last") => Action::Pop,
                    _ => Action::UnknownRemoval,
                };
                let first = args
                    .get(1)
                    .and_then(|s| s.parse().ok())
                    .map_or(Selector::Unknown, Selector::Formal);
                return Some((0, Mutation::new(action, first)));
            }
            if let Some(args) = parameters(&n.text, "ResetsElements") {
                return Some((
                    args.first().and_then(|s| s.parse().ok()).unwrap_or(0),
                    Mutation::new(Action::Reset, Selector::Unknown),
                ));
            }
            if let Some(args) = parameters(&n.text, "RemovesElement") {
                return Some((
                    0,
                    Mutation::new(
                        Action::Key,
                        Selector::Formal(args.first().and_then(|s| s.parse().ok()).unwrap_or(1)),
                    ),
                ));
            }
            let args = parameters(&n.text, "MovesElements")?;
            let action = match *args.first()? {
                "insert" => Action::Insert,
                "swap" => Action::Swap,
                "reverse" => Action::Reverse,
                "sort" => Action::Sort,
                _ => return None,
            };
            Some((
                0,
                Mutation {
                    action,
                    first: args
                        .get(1)
                        .and_then(|s| s.parse().ok())
                        .map_or(Selector::Unknown, Selector::Formal),
                    second: args
                        .get(2)
                        .and_then(|s| s.parse().ok())
                        .map_or(Selector::Unknown, Selector::Formal),
                },
            ))
        })
        .collect()
}

/// A must fact about logical identity, independent of physical allocation validity.
pub(super) fn invalidates(m: Mutation, index: Selector, length: Option<i128>) -> bool {
    use Selector::{Constant as C, Last, Unknown};
    match (m.action, index, m.first) {
        (Action::Reset, _, _) => true,
        (Action::Count, _, C(0)) => true,
        (Action::Count, C(i), C(n)) => i >= n,
        (Action::Count, Last, Last) => true,
        (Action::Count, Last, Selector::CountDelta(n)) => n <= -1,
        (Action::Count, Selector::CountDelta(i), Selector::CountDelta(n)) => i >= n && n < 0,
        (Action::Pop, Last, _) => true,
        (Action::Ordered | Action::Insert, C(i), C(n)) => i >= n,
        (Action::Ordered | Action::Unordered, Last, _) => true,
        (Action::Unordered | Action::Key, i, n) => i != Unknown && i == n,
        (Action::Swap, i, a) => {
            i != Unknown
                && matches!((a, m.second), (C(x), C(y)) if x != y)
                && (i == a || i == m.second)
        }
        (Action::Reverse, C(i), _) => {
            length.is_some_and(|n| n > 1 && i >= 0 && i < n && i != n - 1 - i)
        }
        _ => false,
    }
}

/// Container formal, key selector, pointer result. A logical-entry contract deliberately
/// says nothing about the physical storage layout of the container.
pub(super) fn borrowed(compiler: &Compiler, proc: ProcId) -> Vec<(usize, Selector, usize)> {
    notes(compiler, proc)
        .filter_map(|n| {
            let (args, returned) = if let Some(a) = parameters(&n.text, "BorrowsElement") {
                (a, false)
            } else {
                (parameters(&n.text, "ElementResult")?, true)
            };
            let container = args.first()?.parse().ok()?;
            let key = args.get(1)?.parse().ok();
            let pointer = args.get(2)?.parse().ok()?;
            Some((
                container,
                key.map_or(Selector::Unknown, |key| {
                    if returned {
                        Selector::Result(key)
                    } else {
                        Selector::Formal(key)
                    }
                }),
                pointer,
            ))
        })
        .collect()
}

/// Possible logical invalidation. Exact negative facts remain useful in strict mode.
pub(super) fn possibly_invalidates(m: Mutation, index: Selector, length: Option<i128>) -> bool {
    use Selector::{Constant as C, CountDelta, Last};
    if invalidates(m, index, length) {
        return true;
    }
    match (m.action, index, m.first) {
        (Action::Count, C(i), C(n)) => i >= n,
        (Action::Count, _, CountDelta(n)) if n >= 0 => false,
        (Action::Ordered | Action::Insert, C(i), C(n)) => i >= n,
        (Action::Unordered, C(i), C(n)) => i == n || length.is_none_or(|len| i == len - 1),
        (Action::Key, C(i), C(n)) => i == n,
        (Action::Swap, _, a) if a != Selector::Unknown && a == m.second => false,
        (Action::Swap, C(i), C(n)) => match m.second {
            C(j) => i == n || i == j,
            _ => true,
        },
        (Action::Reverse, C(i), _) => length.is_none_or(|n| i != n - 1 - i),
        (Action::Pop, C(i), _) => length.is_none_or(|n| i == n - 1),
        (Action::Sort, _, _) => length.is_none_or(|n| n > 1),
        (Action::Count, Last, Last) => true,
        _ => true,
    }
}
