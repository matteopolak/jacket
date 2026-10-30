//! `enum_member_prefix`: every member of an enum starts with the enum's name.
//!
//! ```jai
//! Color :: enum {
//!     COLOR_RED;
//!     COLOR_GREEN;
//!     COLOR_BLUE;
//! }
//! ```
//!
//! Members are used as `Color.COLOR_RED` or `.COLOR_RED`, so the prefix says nothing the
//! enum's name does not: `Color.RED` is shorter and as clear. C needs the prefix because its
//! enum members share one namespace; Jai's do not.
//!
//! Fires on an enum of two or more members when every member, compared without case and
//! without `_`, begins with the enum's name (`Token_Kind` and `TOKEN_KIND_IDENT`, or just
//! `TokenKind` and `TokenKindIdent`) and has something left after it. Enums that insert members
//! with `#insert` or `using`, enums declared `using` (whose members are used without the enum's
//! name, where the prefix is what tells them apart), and enums in generated files, are skipped. No fix is offered:
//! renaming members changes every use of them.
use super::finding;
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::walk::Node;
use jaic::ast::{DeclKind, EnumItem, ExprKind as E, StmtKind as S};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "enum_member_prefix",
    category: crate::config::Category::Style,
    summary: "every member of an enum starts with the enum's name",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    super::walk_code(cx, &mut |n, up| {
        let Node::Expr(e) = n else {
            return;
        };
        let E::Enum(lit) = &e.kind else {
            return;
        };
        // The declaration that names the enum.
        let Some(Node::Stmt(parent)) = up.last() else {
            return;
        };
        let S::Decl(decl) = &parent.kind else {
            return;
        };
        let [name] = decl.names.as_slice() else {
            return;
        };
        // `using` brings the members into scope unqualified, where the prefix tells them apart.
        if decl.kind != DeclKind::Const || decl.using {
            return;
        }
        let mut members = Vec::new();
        if !collect(&lit.items, &mut members) || members.len() < 2 {
            return;
        }
        let stem = normalize(name.name.as_str());
        if stem.is_empty()
            || !members.iter().all(|m| {
                let m = normalize(m);
                m.len() > stem.len() && m.starts_with(&stem)
            })
        {
            return;
        }
        out.push(finding(
            name.span,
            format!(
                "every member of `{}` starts with `{}`",
                name.name.as_str(),
                name.name.as_str()
            ),
            Some(format!(
                "members are used as `{0}.NAME`, so the prefix repeats the enum's name: drop it",
                name.name.as_str()
            )),
        ));
    });
}

/// The member names of `items`; false when some are inserted by code the linter cannot see.
fn collect<'a>(items: &'a [EnumItem], out: &mut Vec<&'a str>) -> bool {
    for item in items {
        match item {
            EnumItem::Member(m) => out.push(m.name.name.as_str()),
            EnumItem::If {
                then_items,
                else_items,
                ..
            } => {
                if !collect(then_items, out) || !collect(else_items, out) {
                    return false;
                }
            }
            EnumItem::Insert(_) | EnumItem::Using(_) => return false,
        }
    }
    true
}

/// A name in lower case without `_`: `Token_Kind`, `TOKEN_KIND` and `TokenKind` are the same.
fn normalize(name: &str) -> String {
    name.chars()
        .filter(|c| *c != '_')
        .flat_map(char::to_lowercase)
        .collect()
}
