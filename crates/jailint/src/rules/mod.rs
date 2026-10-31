//! The rules. Each lives in its own module, documented with why it exists; [`RULES`] lists
//! them with their default levels.
use crate::config::Category;
use crate::syntax::Cx;
use crate::{Edit, Finding, Fix, Level};
use jaic::ast::walk::Node;
use jaic::ast::{BinOp, Expr, ExprKind as E, Stmt, StmtKind as S, UnOp};
use jaic::ide::Found;
use jaic::sema::ProcId;
use jaic::source::Span;

// Helpers shared by several rules; they are not rules themselves.
mod frees;
mod local_scopes;

pub(crate) use unused_parameter::RunState;

pub struct RuleInfo {
    pub name: &'static str,
    /// The rule's group; its level is the rule's default.
    pub category: Category,
    /// One line for `--list`.
    pub summary: &'static str,
    pub(crate) check: fn(&Cx, &mut Vec<Finding>),
}

impl RuleInfo {
    /// The level the rule has when nothing configures it: its category's.
    pub fn default(&self) -> Level {
        self.category.level()
    }
}

macro_rules! rules {
    ($($vis:vis $name:ident),* $(,)?) => {
        $($vis mod $name;)*

        /// The rules, in the order `--list` shows them. A rule is added by writing its module
        /// here; the module exports its own [`RuleInfo`] as `INFO`.
        pub static RULES: &[RuleInfo] = &[$($name::INFO),*];

        /// The module of each rule in [`RULES`], for the test that names agree.
        #[cfg(test)]
        const RULE_MODULES: &[&str] = &[$(stringify!($name)),*];
    };
}

rules! {
    absurd_comparison,
    autocast_in_arg,
    almost_swapped,
    autocast_used,
    approx_constant,
    bad_bit_mask,
    assert_constant,
    banner_comment,
    bitwise_precedence,
    bool_comparison,
    bool_to_int_with_if,
    pub(crate) broken_doc_link,
    cognitive_complexity,
    constant_condition,
    defer_frees_returned,
    defer_in_loop,
    detached_doc_comment,
    double_free,
    dropped_result,
    duplicate_condition,
    element_count_as_bytes,
    enum_member_prefix,
    erasing_op,
    excessive_bools,
    excessive_nesting,
    field_reassign_with_default,
    float_loop_counter,
    exit_in_module,
    format_push_string,
    float_equality,
    identical_branches,
    identical_case_bodies,
    identical_operands,
    identity_op,
    if_not_else,
    incomplete_enum_switch,
    impossible_comparison,
    index_only_loop,
    infinite_loop,
    int_plus_one,
    large_stack_array,
    integer_division_in_float,
    leaked_allocation,
    leaked_temporary,
    lossy_float_literal,
    loop_with_break_condition,
    lossy_xx,
    manual_array_find,
    magic_number,
    manual_assign_op,
    manual_clamp,
    manual_memcpy,
    manual_index_counter,
    min_max,
    misrefactored_assign_op,
    missing_doc,
    mixed_case_hex_literals,
    modified_range_bound,
    needless_bool,
    neg_cmp_float,
    needless_continue,
    neg_multiply,
    no_effect,
    pub(crate) platform_leak,
    pointer_into_resizable,
    print_in_module,
    push_in_static_if,
    range_past_count,
    redundant_alias,
    redundant_deref,
    redundant_cast,
    remove_in_for,
    returned_local_address,
    reversed_range,
    self_assignment,
    shift_overflow,
    shadowed_it,
    shadowed_local,
    single_case_switch,
    string_literal_write,
    temp_storage_escape,
    todo_marker,
    too_many_arguments,
    unchecked_directive,
    unclosed_resource,
    uninit_read,
    unnecessary_force_cast,
    unknown_allow,
    unnecessary_semicolon,
    unpaired_lock,
    unsigned_range_end_minus_one,
    unreadable_literal,
    pub(crate) unused_import,
    unused_parameter,
    unused_private_declaration,
    unused_result,
    unused_suppression,
    unused_variable,
    useless_format,
    useless_resizable,
    use_after_free,
    using_pollution,
    wrapping_constant,
    zero_ptr,
}

/// How an expression that denotes a place is used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlaceUse {
    Read,
    /// Assigned to, or a part of it is.
    Write,
    /// Its address (or the address of a part) is taken: it may be written through.
    Address,
}

/// How the expression `node` (whose ancestors are `chain`, outermost first) is used: the
/// access path it heads (`x`, `x.a`, `x[1].b`) decides.
pub(crate) fn place_use(node: &Expr, chain: &[Node<'_>]) -> PlaceUse {
    let mut cur = node;
    let mut i = chain.len();
    while i > 0 {
        i -= 1;
        match chain[i] {
            Node::Expr(p) => match &p.kind {
                E::Member(base, _) if base.span == cur.span => cur = p,
                E::Index(base, _) if base.span == cur.span => cur = p,
                E::Unary(UnOp::Star, inner) if inner.span == cur.span => return PlaceUse::Address,
                // `x.*`: the pointer's target; writes go elsewhere.
                _ => return PlaceUse::Read,
            },
            Node::Stmt(s) => {
                return match &s.kind {
                    S::Assign {
                        lhs, ..
                    } if lhs.iter().any(|l| l.span == cur.span) => PlaceUse::Write,
                    S::For(f) if f.by_pointer || f.pointer_if.is_some() => match &f.over {
                        jaic::ast::ForOver::Collection(c) if c.span == cur.span => {
                            PlaceUse::Address
                        }
                        _ => PlaceUse::Read,
                    },
                    S::Using(using) if using.value.span == cur.span => PlaceUse::Address,
                    _ => PlaceUse::Read,
                };
            }
        }
    }
    PlaceUse::Read
}

/// `e` is a call argument (directly) in `chain`'s last node.
pub(crate) fn is_call_argument(e: &Expr, chain: &[Node<'_>]) -> bool {
    matches!(chain.last(), Some(Node::Expr(Expr { kind: E::Call { args, .. }, .. }))
        if args.iter().any(|a| a.value.span == e.span))
}

/// Evaluating `e` has no side effects and gives the same value twice in a row: names, member
/// paths, literals, and indexing, operators and casts over them. Calls never qualify.
pub(crate) fn pure(e: &Expr) -> bool {
    match &e.kind {
        E::Ident(_)
        | E::Int(..)
        | E::Float(..)
        | E::Bool(_)
        | E::Null
        | E::Str(_)
        | E::Char(_)
        | E::InferredMember(_) => true,
        E::Member(base, _) => pure(base),
        E::Index(a, b) | E::Binary(_, a, b) => pure(a) && pure(b),
        E::Unary(_, a) => pure(a),
        E::Cast {
            value, ..
        } => pure(value),
        _ => false,
    }
}

/// `a` and `b` are the same pure expression, written the same way.
pub(crate) fn same_pure(cx: &Cx, a: &Expr, b: &Expr) -> bool {
    pure(a) && pure(b) && cx.same_tokens(a, b)
}

/// An integer literal, possibly negated: its value.
pub(crate) fn int_literal(e: &Expr) -> Option<i128> {
    match &e.kind {
        E::Int(v, ..) => i128::try_from(v.get()).ok(),
        E::Unary(UnOp::Neg, inner) => int_literal(inner).map(|v| -v),
        E::Unary(UnOp::Plus, inner) => int_literal(inner),
        _ => None,
    }
}

/// The procedures a call's callee named, when it is a plain name or `Module.name`.
pub(crate) fn callee_procs(cx: &Cx, callee: &Expr) -> Found<ProcId> {
    let span = match &callee.kind {
        E::Ident(_) => callee.span,
        E::Member(_, name) => name.span,
        _ => return Found::default(),
    };
    cx.compiler.ide_procs_named_at(span)
}

/// The procedure is declared with the note `@name`, on its header or after its body.
pub(crate) fn has_note(cx: &Cx, proc: ProcId, name: &str) -> bool {
    cx.compiler.proc_has_note(proc, name)
}

/// Every procedure the callee named carries the note `@name`: the declaration, not the
/// spelling of the name, says what the call means.
pub(crate) fn calls_noted(cx: &Cx, callee: &Expr, name: &str) -> bool {
    let procs = callee_procs(cx, callee);
    let span = match &callee.kind {
        E::Member(_, name) => name.span,
        _ => callee.span,
    };
    // Of an overload set, the one this call chose says what the call means.
    if let Some(chosen) = cx.compiler.ide_resolved_overload(span, &procs) {
        return has_note(cx, chosen, name);
    }
    !procs.is_empty() && procs.iter().all(|&p| has_note(cx, p, name))
}

/// Some statement inside `stmt` (itself included, nested procedures not entered) satisfies `f`.
pub(crate) fn any_stmt(stmt: &Stmt, f: &mut dyn FnMut(&Stmt) -> bool) -> bool {
    let mut found = false;
    let mut stack = Vec::new();
    jaic::ast::walk::walk_node(Node::Stmt(stmt), &mut stack, &mut |n, _| {
        if !found
            && let Node::Stmt(s) = n
            && f(s)
        {
            found = true;
        }
        !found
    });
    found
}

/// Every statement list in a procedure body: the body, nested blocks, `case` bodies and `#if`
/// branches.
pub(crate) fn statement_lists<'a>(body: &'a [Stmt], f: &mut dyn FnMut(&'a [Stmt])) {
    f(body);
    jaic::ast::walk::walk(body, &mut |n, _| {
        if let Some(s) = n.stmt() {
            match &s.kind {
                S::Block(b) => f(&b.stmts),
                S::Switch(boxed) | S::StaticSwitch(boxed) => {
                    let jaic::ast::SwitchStmt {
                        cases, ..
                    } = &**boxed;
                    for c in cases {
                        f(&c.body);
                    }
                }
                S::StaticIf(boxed) => {
                    let jaic::ast::StaticIfStmt {
                        then_branch,
                        else_branch,
                        ..
                    } = &**boxed;
                    f(then_branch);
                    f(else_branch);
                }
                _ => {}
            }
        }
        true
    });
}

/// Visit every statement and expression the file's own code holds: its top-level statements
/// and the bodies of its procedures, macros and generated files left out. For rules that read
/// only syntax, which a macro's expansion site or a generator may write differently.
pub(crate) fn walk_code<'a>(cx: &Cx<'a>, visit: &mut dyn FnMut(Node<'a>, &[Node<'a>])) {
    if cx.generated() {
        return;
    }
    jaic::ast::walk::walk(cx.stmts, &mut |n, up| {
        visit(n, up);
        true
    });
    for p in cx.procs.iter().filter(|p| !p.is_macro) {
        jaic::ast::walk::walk(&p.body.stmts, &mut |n, up| {
            visit(n, up);
            true
        });
    }
}

/// Report each call, in a file that belongs to a module, to a procedure with the note `note`.
/// For rules about what a library should leave to the program that uses it.
pub(crate) fn calls_in_module(
    cx: &Cx,
    out: &mut Vec<Finding>,
    note: &str,
    message: &str,
    help: &str,
) {
    if cx.generated() || !cx.in_module() {
        return;
    }
    walk_code(cx, &mut |n, _| {
        let Node::Expr(e) = n else {
            return;
        };
        if let E::Call {
            callee, ..
        } = &e.kind
            && calls_noted(cx, callee, note)
        {
            out.push(finding(e.span, message.into(), Some(help.into())));
        }
    });
}

/// Where a finding about a procedure goes, and how its message names it: the declared name,
/// else the start of the header.
pub(crate) fn proc_label(p: &crate::syntax::ProcSite) -> (Span, String) {
    match p.decl.and_then(|d| d.names.first()) {
        Some(name) => (name.span, format!("`{}`", name.name.as_str())),
        None => (
            Span {
                end: p.header.span.start + 1,
                ..p.header.span
            },
            "this procedure".to_string(),
        ),
    }
}

/// A finding without a fix.
pub(crate) fn finding(span: Span, message: String, help: Option<String>) -> Finding {
    Finding {
        start: span.start as usize,
        end: span.end as usize,
        message,
        label: None,
        help,
        fix: None,
    }
}

/// A statement's bytes with the `;` the span may stop before.
pub(crate) fn statement_range(cx: &Cx, span: Span) -> (usize, usize) {
    let start = span.start as usize;
    let mut end = span.end as usize;
    let rest = &cx.text[end..];
    let trimmed = rest.trim_start_matches([' ', '\t']);
    if trimmed.starts_with(';') && !cx.text[..end].ends_with(';') {
        end += rest.len() - trimmed.len() + 1;
    }
    (start, end)
}

/// A finding with a fix the editor offers and `--fix` leaves alone: the change may not be what
/// was meant, or it changes what the program does. `title` names the change.
pub(crate) fn suggesting(
    span: Span,
    message: String,
    help: String,
    title: impl Into<String>,
    edits: Vec<Edit>,
) -> Finding {
    Finding {
        start: span.start as usize,
        end: span.end as usize,
        message,
        label: None,
        help: Some(help),
        fix: Some(Fix {
            title: title.into(),
            edits,
            machine_applicable: false,
        }),
    }
}

/// A finding whose fix replaces bytes `start..end` with `text`.
pub(crate) fn replacing(
    span: Span,
    message: String,
    help: String,
    (start, end): (usize, usize),
    text: String,
    machine_applicable: bool,
) -> Finding {
    Finding {
        start: span.start as usize,
        end: span.end as usize,
        message,
        label: None,
        help: Some(help.clone()),
        fix: Some(Fix {
            title: help,
            edits: vec![Edit {
                start,
                end,
                text,
            }],
            machine_applicable,
        }),
    }
}

/// How the operator is written.
pub(crate) fn op_text(op: BinOp) -> &'static str {
    match op {
        BinOp::Eq => "==",
        BinOp::Ne => "!=",
        BinOp::Lt => "<",
        BinOp::Le => "<=",
        BinOp::Gt => ">",
        BinOp::Ge => ">=",
        BinOp::Sub => "-",
        BinOp::Div => "/",
        BinOp::Rem => "%",
        BinOp::BitAnd => "&",
        BinOp::BitOr => "|",
        BinOp::BitXor => "^",
        BinOp::And => "&&",
        BinOp::Or => "||",
        BinOp::Add => "+",
        BinOp::Mul => "*",
        BinOp::Shl => "<<",
        BinOp::Shr => ">>",
        BinOp::Rotl => "<<<",
        BinOp::Rotr => ">>>",
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn each_rule_is_named_after_its_module() {
        let names: Vec<_> = super::RULES.iter().map(|r| r.name).collect();
        assert_eq!(names, super::RULE_MODULES);
    }
}
