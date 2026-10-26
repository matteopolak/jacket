//! `unused_private_declaration`: a top-level declaration after `#scope_file` that nothing in its
//! file mentions.
//!
//! ```jai
//! #scope_file
//!
//! clamp_index :: (i: int, count: int) -> int {    // nothing calls it
//!     return max(0, min(i, count - 1));
//! }
//! ```
//!
//! A `#scope_file` declaration cannot be seen from any other file, so a name nothing in its own
//! file uses is dead code: a helper left behind by a refactor, or a constant whose last use was
//! removed. Delete it, or move it under `#scope_export` if another file should use it.
//!
//! Looks at the procedures, structs, enums, constants and variables a file declares between a
//! `#scope_file` and the next `#scope_export` or `#scope_module`. A declaration counts as used
//! when its name appears as an identifier anywhere else in the file, in code the compiler
//! skipped (an `#if` branch left out) included; only mentions outside its own declaration
//! count, so a procedure that only calls itself is unused. The rule stays quiet where names
//! can be built or read without being written out:
//!
//! - files that contain an `#insert` or a `#placeholder`;
//! - declarations with notes, `#c_call`, `#foreign`, `#program_export`, `#compiler`,
//!   `#expand` (a macro may be named by a `for_expansion` convention) or operator forms, which
//!   something outside the file reads by name or signature;
//! - `main`, the entry point, `build` and `get_plugin`, which the compiler and build system
//!   call; `for_expansion`, which `for` finds by name; names starting with `__`, which the
//!   compiler reserves for what it looks up itself (the runtime support hooks); and `_` names;
//! - `using` declarations, declarations that bind several names, and generated files.
//!
//! No fix: deleting is the author's call, and a variable's initializer may have effects.
use super::finding;
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::{DeclKind, ExprKind as E, ScopeKind, StmtKind as S};
use jaic::lexer::Tok;

/// Names the compiler, the build system or `for` look up without the program writing them.
const READ_BY_NAME: &[&str] = &["main", "build", "get_plugin", "for_expansion", "_"];

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "unused_private_declaration",
    category: crate::config::Category::Suspicious,
    summary: "a `#scope_file` declaration that nothing in its file uses",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() || reads_names(cx) {
        return;
    }
    let mut scope = ScopeKind::Export;
    for stmt in cx.stmts {
        let decl = match &stmt.kind {
            S::Scope(kind) => {
                scope = *kind;
                continue;
            }
            S::Decl(decl) => decl,
            _ => continue,
        };
        if scope != ScopeKind::File || decl.using || decl.backtick || !decl.notes.is_empty() {
            continue;
        }
        let [name] = decl.names.as_slice() else {
            continue;
        };
        if READ_BY_NAME.contains(&name.name.as_str())
            || name.name.as_str().starts_with("__")
            || name.name.as_str() == cx.compiler.options.entry_point
        {
            continue;
        }
        if decl.kind == DeclKind::Const
            && let Some(value) = &decl.value
            && let E::Proc(lit) = &value.kind
        {
            let f = &lit.header.flags;
            if f.c_call()
                || f.expand()
                || f.compiler()
                || f.intrinsic()
                || f.elsewhere.is_some()
                || f.program_export.is_some()
                || f.cpp_method()
                || f.runtime_support()
                || lit.header.foreign.is_some()
                || lit.header.operator.is_some()
                || !lit.header.notes.is_empty()
            {
                continue;
            }
        }
        if cx.mentions(name.name, 0, cx.text.len() as u32, stmt.span) {
            continue;
        }
        out.push(finding(
            name.span,
            format!(
                "`{}` is private to this file and nothing in the file uses it",
                name.name.as_str()
            ),
            Some("delete it, or put it under `#scope_export` if another file should use it".into()),
        ));
    }
}

/// The file writes code or names out of strings (`#insert`) or leaves a name to be provided
/// elsewhere (`#placeholder`).
fn reads_names(cx: &Cx) -> bool {
    cx.tokens.iter().any(
        |t| matches!(&t.tok, Tok::Directive(d) if matches!(d.as_str(), "insert" | "placeholder")),
    )
}
