//! `missing_doc`: an exported top-level procedure, struct, enum or union with no doc comment.
//!
//! ```jai
//! parse :: (text: string) -> Tree { ... }
//! ```
//!
//! Doc comments are what hover and the generated docs show, so an exported name without one
//! is a gap in the module's interface. Looks at the top level of each file, where a name is
//! exported unless a `#scope_file` or `#scope_module` before it narrows it. A doc is the
//! `//` or `/* */` comment directly above the declaration.
//! `main`, declarations that bind several names, and generated files are skipped. No fix: the
//! text is the author's to write.
use super::finding;
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::{DeclKind, ExprKind as E, ScopeKind, StmtKind as S};
use jaic::docs;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "missing_doc",
    category: crate::config::Category::Restriction,
    summary: "an exported procedure, struct or enum with no doc comment",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() {
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
        if scope != ScopeKind::Export || decl.kind != DeclKind::Const {
            continue;
        }
        let ([name], Some(value)) = (decl.names.as_slice(), &decl.value) else {
            continue;
        };
        let kind = match &value.kind {
            E::Proc(_) => "procedure",
            E::Struct(_) => "struct",
            E::Enum(_) => "enum",
            _ => continue,
        };
        let text = name.name.as_str();
        if text == "main" || docs::declaration(cx.text, name.span.start as usize).is_some() {
            continue;
        }
        out.push(finding(
            name.span,
            format!("the exported {kind} `{text}` has no doc comment"),
            Some("describe it in a `//` comment on the lines above".into()),
        ));
    }
}
