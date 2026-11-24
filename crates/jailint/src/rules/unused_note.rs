//! `unused_note`: a note that no compile-time code read during the check.
//!
//! ```jai
//! answer :: () -> int { return 42; } @Exported
//! ```
//!
//! Notes are for metaprograms: a note nothing reads does nothing, which usually means the
//! metaprogram that wanted it was removed or never sees the declaration. A note counts as
//! read when compile-time code of the check (a `#run`, or a build metaprogram intercepting a
//! workspace) loaded or copied its text, or compared it equal to something (see
//! `docs/compiler/note-readers.md`). Notes the compiler reads itself (`@PrintLike`, `@Cold`,
//! `@Section(...)` and the like), the declaration notes the rules read (`LINT_NOTES`), the
//! notes other tools read (`TOOL_NOTES`: `@test`, `@native`) and `@jailint_allow` are not
//! reported. Neither are notes in a file whose check did not finish, or in an untrusted
//! workspace, where compile-time code did not all run.
//!
//! Off by default: notes read by metaprograms the check does not run are reported too. No fix:
//! whether the note or the metaprogram is wrong is the author's call.
use super::finding;
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::walk::{Node, walk_node};
use jaic::ast::{EnumItem, ExprKind, Note, ProcHeader, Stmt, StmtKind};
use jaic::fxhash::HashSet;

/// The [declaration notes](../../../../docs/tools/jailint.md#declaration-notes) the rules read
/// from a procedure's declaration, by name.
pub const LINT_NOTES: [&str; 36] = [
    "AddsToArray",
    "Allocates",
    "AppendsString",
    "AssertsArgument",
    "BorrowsElement",
    "CompletesBorrower",
    "ElementResult",
    "EndsBorrower",
    "Exits",
    "FormatsToString",
    "FreesArgument",
    "GrowsArray",
    "Locks",
    "MaybeFreesArgument",
    "MovesElements",
    "NoLifetimeEffects",
    "Opens",
    "PrintLike",
    "RegistersBorrower",
    "RemovesElement",
    "RemovesFromArray",
    "ResetsElements",
    "ResetsTemporaryStorage",
    "RestoresTemporaryStorage",
    "ReturnsLarger",
    "ReturnsSmaller",
    "ReturnsStandardHandle",
    "StartsBorrower",
    "SwapsArguments",
    "TakesByteCount",
    "TempAllocates",
    "TemporaryStorageMark",
    "Unlocks",
    "UseResult",
    "ValidUntilNextCall",
    "WritesOutput",
];

/// Notes other tools read from the source: `jacket test`'s
/// ([test notes](../../../../docs/tools/jacket-test.md)) and `@native`, which the
/// [native library build](../../../../docs/tools/native-libs.md) reads.
pub const TOOL_NOTES: [&str; 27] = [
    "allow_leaks",
    "args",
    "bench",
    "cases",
    "check_only",
    "compile_fail",
    "compile_time",
    "exit",
    "generate",
    "interp_only",
    "min_time",
    "native",
    "native_only",
    "needs",
    "only",
    "optional",
    "property",
    "retries",
    "samples",
    "serial",
    "should_fail",
    "skip_on",
    "slow",
    "test",
    "timeout",
    "warns",
    "xfail",
];

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "unused_note",
    category: crate::config::Category::Restriction,
    summary: "a note that no compile-time code read",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if !cx.complete || cx.compiler.ed.facts().is_none_or(|facts| facts.untrusted) {
        return;
    }
    let read: HashSet<(u32, u32)> = cx
        .compiler
        .ide_note_readers_in(cx.file)
        .into_iter()
        .map(|(span, _)| (span.start, span.end))
        .collect();
    let mut notes = Vec::new();
    collect(cx.stmts, &mut notes);
    notes.sort_by_key(|n| n.span.start);
    notes.dedup_by_key(|n| n.span.start);
    for note in notes {
        let name = note.text.split('(').next().unwrap_or_default().trim();
        if jaic::sema::COMPILER_NOTES.contains(&name)
            || LINT_NOTES.contains(&name)
            || TOOL_NOTES.contains(&name)
            || name == crate::suppress::NOTE
            || cx.in_code_operand(note.span)
            || read.contains(&(note.span.start, note.span.end))
        {
            continue;
        }
        out.push(finding(
            note.span,
            format!("no compile-time code read the note `@{}`", note.text.trim()),
            Some("remove it, or make the metaprogram that reads it part of the check".into()),
        ));
    }
}

/// Every note written in `stmts` and what they contain.
fn collect<'a>(stmts: &'a [Stmt], out: &mut Vec<&'a Note>) {
    for s in stmts {
        let mut stack = Vec::new();
        walk_node(Node::Stmt(s), &mut stack, &mut |n, _| {
            match n {
                Node::Stmt(s) => {
                    out.extend(s.notes.iter());
                    if let StmtKind::Decl(d) = &s.kind {
                        out.extend(d.notes.iter());
                    }
                }
                Node::Expr(e) => match &e.kind {
                    ExprKind::Proc(lit) => {
                        header(&lit.header, out);
                        if let Some(body) = &lit.body {
                            collect(&body.stmts, out);
                        }
                    }
                    ExprKind::Struct(st) => {
                        out.extend(st.notes.iter());
                        for p in &st.params {
                            out.extend(p.notes.iter());
                        }
                        collect(&st.body, out);
                    }
                    ExprKind::Enum(en) => {
                        out.extend(en.notes.iter());
                        for item in &en.items {
                            if let EnumItem::Member(m) = item {
                                out.extend(m.notes.iter());
                            }
                        }
                    }
                    _ => {}
                },
            }
            true
        });
    }
}

fn header<'a>(h: &'a ProcHeader, out: &mut Vec<&'a Note>) {
    out.extend(h.notes.iter());
    for p in &h.params {
        out.extend(p.notes.iter());
    }
}
