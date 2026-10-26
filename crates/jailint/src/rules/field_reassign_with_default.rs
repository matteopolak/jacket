//! `field_reassign_with_default`: `p: Point; p.x = 1; p.y = 2;`, fields set right after the
//! default value is made.
//!
//! ```jai
//! config: Config;
//! config.width = 80;
//! config.height = 24;
//! ```
//!
//! is a struct literal: `config := Config.{ width = 80, height = 24 };`. The fields not named
//! keep their defaults, as in the declaration, and the values are set where the variable is
//! made instead of in statements that a later edit can separate from it.
//!
//! Fires on a local `name: T;` of a struct type with no value, directly followed by one or
//! more plain assignments to distinct fields of `name` (`name.field = value;`) whose values do
//! not mention `name`. The editor offers the literal when no comment sits among the statements.
use super::{statement_lists, statement_range, suggesting};
use crate::syntax::Cx;
use crate::{Edit, Finding};
use jaic::ast::{AssignOp, DeclKind, ExprKind as E, Stmt, StmtKind as S};
use jaic::sema::scope::EntityKind;
use jaic::types::TypeKind;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "field_reassign_with_default",
    category: crate::config::Category::Pedantic,
    summary: "`x: T; x.a = 1; x.b = 2;` where a struct literal `x := T.{ a = 1, b = 2 }` sets the fields at the declaration",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() {
        return;
    }
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        statement_lists(&p.body.stmts, &mut |list| {
            for (i, s) in list.iter().enumerate() {
                if let Some(found) = examine(cx, list, i, s) {
                    out.push(found);
                }
            }
        });
    }
}

fn examine(cx: &Cx, list: &[Stmt], at: usize, s: &Stmt) -> Option<Finding> {
    let types = &cx.compiler.ty.types;
    let S::Decl(d) = &s.kind else {
        return None;
    };
    let [name] = d.names.as_slice() else {
        return None;
    };
    let ty = d.ty.as_ref()?;
    if d.kind != DeclKind::Var
        || d.value.is_some()
        || d.using
        || d.backtick
        || d.rare.is_some()
        || !d.notes.is_empty()
    {
        return None;
    }
    // A struct (not a union, whose literal sets one member).
    let entities = cx.facts.locals.get(&name.span)?;
    let &first = entities.first()?;
    let EntityKind::Local {
        ty: local, ..
    } = cx.compiler.entity(first).kind
    else {
        return None;
    };
    let TypeKind::Struct(id) = types.kind(local) else {
        return None;
    };
    let info = types.struct_info(*id);
    if info.is_union {
        return None;
    }
    let mut fields: Vec<(String, &jaic::ast::Expr)> = Vec::new();
    let mut last = None;
    for next in &list[at + 1..] {
        let S::Assign {
            op: AssignOp::Assign,
            lhs,
            rhs,
        } = &next.kind
        else {
            break;
        };
        let ([target], [value]) = (lhs.as_slice(), rhs.as_slice()) else {
            break;
        };
        let E::Member(base, field) = &target.kind else {
            break;
        };
        if !matches!(base.kind, E::Ident(n) if n == name.name)
            || !info.fields.iter().any(|f| f.name == Some(field.name))
            || fields.iter().any(|(f, _)| f == field.name.as_str())
            || cx
                .name_tokens(name.name, value.span.start, value.span.end)
                .next()
                .is_some()
        {
            break;
        }
        fields.push((field.name.as_str().to_string(), value));
        last = Some(next);
    }
    let last = last?;
    let shown = name.name.as_str();
    let literal = fields
        .iter()
        .map(|(f, v)| format!("{f} = {}", cx.whole_src(v.span).trim()))
        .collect::<Vec<_>>()
        .join(", ");
    let replacement = format!("{shown} := {}.{{ {literal} }};", cx.src(ty.span).trim());
    let start = s.span.start as usize;
    let (_, end) = statement_range(cx, last.span);
    let message = format!(
        "`{shown}` is made with its defaults and then {} field{} set",
        if fields.len() == 1 {
            "one"
        } else {
            "its"
        },
        if fields.len() == 1 {
            ""
        } else {
            "s"
        }
    );
    let help = "set the fields in a struct literal where the variable is declared".to_string();
    if cx.has_comment_between(start, end) {
        return Some(super::finding(name.span, message, Some(help)));
    }
    Some(suggesting(
        name.span,
        message,
        help,
        format!("write `{replacement}`"),
        vec![Edit {
            start,
            end,
            text: replacement,
        }],
    ))
}
