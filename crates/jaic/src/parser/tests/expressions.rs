//! Operators, casts, calls and directive primaries.
use super::{body_of, decl, value};
use crate::ast::*;

// -- expressions ----------------------------------------------------------------

/// `x := <expr>;` with every binary, unary and cast node parenthesized.
fn grouped(expr: &str) -> String {
    fn show(e: &Expr) -> String {
        match &e.kind {
            ExprKind::Binary(op, a, b) => {
                let text = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Rem => "%",
                    BinOp::BitAnd => "&",
                    BinOp::BitOr => "|",
                    BinOp::BitXor => "^",
                    BinOp::Shl => "<<",
                    BinOp::Shr => ">>",
                    BinOp::Rotl => "<<<",
                    BinOp::Rotr => ">>>",
                    BinOp::Eq => "==",
                    BinOp::Ne => "!=",
                    BinOp::Lt => "<",
                    BinOp::Le => "<=",
                    BinOp::Gt => ">",
                    BinOp::Ge => ">=",
                    BinOp::And => "&&",
                    BinOp::Or => "||",
                };
                format!("({} {text} {})", show(a), show(b))
            }
            ExprKind::Unary(op, a) => {
                let text = match op {
                    UnOp::Neg => "-",
                    UnOp::Not => "!",
                    UnOp::BitNot => "~",
                    UnOp::Star => "*",
                    _ => "?",
                };
                format!("({text}{})", show(a))
            }
            ExprKind::Cast {
                ty,
                value,
                ..
            } => {
                let ty = ty
                    .as_ref()
                    .map_or("xx".to_string(), |t| format!("cast {}", show(t)));
                format!("({ty} {})", show(value))
            }
            ExprKind::Ident(name) => name.as_str().to_string(),
            ExprKind::Int(v) => v.get().to_string(),
            ExprKind::Member(base, field) => format!("{}.{}", show(base), field.name.as_str()),
            other => format!("{other:?}"),
        }
    }
    show(&value(&format!("x := {expr};")))
}

#[test]
fn binary_precedence() {
    // Loosest to tightest: `||`, `&&`, `== !=`, `< <= > >=`, `+ -`, `%`, `* /`, then the
    // bitwise and shift operators on one level.
    assert_eq!(
        grouped("a || b && c == d < e + f % g * h & i"),
        "(a || (b && (c == (d < (e + (f % (g * (h & i))))))))"
    );
    assert_eq!(
        grouped("a & b * c % d + e < f == g && h || i"),
        "((((((((a & b) * c) % d) + e) < f) == g) && h) || i)"
    );
    // Every level is left associative.
    for (src, want) in [
        ("a - b - c", "((a - b) - c)"),
        ("a / b * c", "((a / b) * c)"),
        ("a << b << c", "((a << b) << c)"),
        ("a || b || c", "((a || b) || c)"),
    ] {
        assert_eq!(grouped(src), want, "{src}");
    }
}

#[test]
fn binary_precedence_matches_recorded_jai_output() {
    // Results recorded from the real compiler (open-jai `utils/stress.jai`, section 1001).
    for (src, want) in [
        ("1 << 2 + 3", "((1 << 2) + 3)"),            // 7
        ("8 >> 1 + 1", "((8 >> 1) + 1)"),            // 5
        ("1 << 2 * 3", "((1 << 2) * 3)"),            // 12
        ("10 % 3 * 2", "(10 % (3 * 2))"),            // 4
        ("1 | 2 & 4", "((1 | 2) & 4)"),              // 0
        ("1 + 2 * 3 << 1", "(1 + (2 * (3 << 1)))"),  // 13
        ("1 << 1 << 2", "((1 << 1) << 2)"),          // 8
        ("0x0F & 0x33 == 0x03", "((15 & 51) == 3)"), // true
    ] {
        assert_eq!(grouped(src), want, "{src}");
    }
}

#[test]
fn bitwise_and_shift_operators_share_a_level() {
    for (src, want) in [
        ("a << b | c", "((a << b) | c)"),
        ("a | b << c", "((a | b) << c)"),
        ("a & b | c", "((a & b) | c)"),
        ("a | b & c", "((a | b) & c)"),
        ("a ^ b & c", "((a ^ b) & c)"),
        ("a >> b & c", "((a >> b) & c)"),
        ("a <<< b | c", "((a <<< b) | c)"),
        ("a & b + c", "((a & b) + c)"),
        ("a + b & c", "(a + (b & c))"),
        ("a * b >> c", "(a * (b >> c))"),
    ] {
        assert_eq!(grouped(src), want, "{src}");
    }
}

#[test]
fn remainder_sits_between_multiplication_and_addition() {
    for (src, want) in [
        ("t - c % t", "(t - (c % t))"),
        ("c % t + 1", "((c % t) + 1)"),
        ("a % b * c", "(a % (b * c))"),
        ("a * b % c", "((a * b) % c)"),
        ("a % b / c", "(a % (b / c))"),
        ("a / b % c", "((a / b) % c)"),
        ("i % 2 == 0", "((i % 2) == 0)"),
    ] {
        assert_eq!(grouped(src), want, "{src}");
    }
}

#[test]
fn prefix_operators_bind_tighter_than_binary_ones() {
    for (src, want) in [
        ("-a & b", "((-a) & b)"),
        ("~a & b", "((~a) & b)"),
        ("-s >> 63", "((-s) >> 63)"),
        ("!a && b", "((!a) && b)"),
        ("-a.b * c", "((-a.b) * c)"),
        ("*a.b", "(*a.b)"),
        ("cast(u8) a & b + c", "((cast u8 (a & b)) + c)"),
        ("xx a | b == c", "((xx (a | b)) == c)"),
        ("a * cast(u8) b << c", "(a * (cast u8 (b << c)))"),
        ("cast(u32) b << 4 | 1", "(cast u32 ((b << 4) | 1))"),
    ] {
        assert_eq!(grouped(src), want, "{src}");
    }
}

/// The operator at the top of `x := <expr>;`, and whether its left operand is a cast.
fn top_op(src: &str) -> (BinOp, bool) {
    let ExprKind::Binary(op, lhs, _) = value(src).kind else {
        panic!("{src}: not a binary expression")
    };
    (op, matches!(lhs.kind, ExprKind::Cast { .. }))
}

/// The binary operator directly inside the cast of `x := <cast>;`.
fn cast_value_op(src: &str) -> Option<BinOp> {
    let ExprKind::Cast {
        value: inner, ..
    } = value(src).kind
    else {
        panic!("{src}: not a cast")
    };
    match inner.kind {
        ExprKind::Binary(op, ..) => Some(op),
        _ => None,
    }
}

#[test]
fn prefix_cast_takes_bitwise_and_shift_operators() {
    // Bitwise and shift operators continue the cast's value, for `cast(T)`, its flagged
    // form and `xx`.
    for (src, op) in [
        ("x := cast(float) (h >> 16) & 0xFF;", BinOp::BitAnd),
        ("x := cast(u64) p | TAG;", BinOp::BitOr),
        ("x := cast(int) turn ^ 1;", BinOp::BitXor),
        ("x := cast(u32) a << 16;", BinOp::Shl),
        ("x := cast(s32) v >> 6;", BinOp::Shr),
        ("x := cast(u32) v <<< 3;", BinOp::Rotl),
        ("x := cast,no_check(u16) a << 4;", BinOp::Shl),
        ("x := xx (ch | MARK) & MASK;", BinOp::BitAnd),
        ("x := xx a1 | h1;", BinOp::BitOr),
    ] {
        assert_eq!(cast_value_op(src), Some(op), "{src}");
    }
    // They share one level, left associative: `a << 16 | b` is `(a << 16) | b`.
    assert_eq!(
        cast_value_op("x := cast(u32) a << 16 | b;"),
        Some(BinOp::BitOr)
    );
    // Arithmetic, comparison and logical operators apply to the cast's result.
    for (src, op) in [
        ("x := cast(*u8) p + offset;", BinOp::Add),
        ("x := cast(s64) p - cast(s64) q;", BinOp::Sub),
        ("x := cast(float) w * scale;", BinOp::Mul),
        ("x := cast(float) a / b;", BinOp::Div),
        ("x := cast(int) a % b;", BinOp::Rem),
        ("x := cast(u8) a == b;", BinOp::Eq),
        ("x := cast(int) a < b;", BinOp::Lt),
        ("x := cast(bool) a && b;", BinOp::And),
        ("x := cast(bool) a || b;", BinOp::Or),
        ("x := xx a + b;", BinOp::Add),
    ] {
        assert_eq!(top_op(src), (op, true), "{src}");
    }
    // A bitwise operator after the cast's value still stops at the next arithmetic one:
    // `cast(T) a & b + c` adds `c` to the cast.
    let ExprKind::Binary(BinOp::Add, lhs, _) = value("x := cast(u8) a & b + c;").kind else {
        panic!("expected an addition")
    };
    assert!(matches!(lhs.kind, ExprKind::Cast { ref value, .. }
        if matches!(value.kind, ExprKind::Binary(BinOp::BitAnd, ..))));
    // Postfix operators bind before the cast; a parenthesized cast or the call form
    // ends it.
    assert_eq!(cast_value_op("x := cast(float) s.width;"), None);
    assert_eq!(
        cast_value_op("x := cast(s32) xs[2] & 0xffff;"),
        Some(BinOp::BitAnd)
    );
    assert_eq!(
        cast_value_op("x := cast(float) f(a) << 1;"),
        Some(BinOp::Shl)
    );
    assert_eq!(top_op("x := (cast(u32) a) << 16;"), (BinOp::Shl, true));
    assert_eq!(top_op("x := cast(u32, a) << 16;"), (BinOp::Shl, true));
}

#[test]
fn unary_and_cast_forms() {
    assert!(matches!(
        value("x := -a.b;").kind,
        ExprKind::Unary(UnOp::Neg, _)
    ));
    let ExprKind::Cast {
        ty,
        flags,
        ..
    } = value("x := cast,no_check(s8) y;").kind
    else {
        panic!()
    };
    assert!(ty.is_some() && flags.no_check);
    assert!(matches!(
        value("x := xx y;").kind,
        ExprKind::Cast {
            ty: None,
            ..
        }
    ));
    assert!(
        matches!(value("x := xx,trunc y;").kind, ExprKind::Cast { ty: None, flags, .. } if flags.truncate)
    );
    assert!(matches!(
        value("x := y.(u64);").kind,
        ExprKind::Cast {
            ty: Some(_),
            ..
        }
    ));
    assert!(matches!(
        value("x := cast(u8, y).z;").kind,
        ExprKind::Member(..)
    ));
    // `cast` binds tighter than a following binary operator.
    assert!(
        matches!(value("x := cast(float) a + b;").kind, ExprKind::Binary(BinOp::Add, lhs, _) if matches!(lhs.kind, ExprKind::Cast { .. }))
    );
}

#[test]
fn ifx_forms() {
    let ExprKind::Ifx {
        then_value,
        else_value,
        is_static,
        ..
    } = value("x := ifx c then a else b;").kind
    else {
        panic!()
    };
    assert!(then_value.is_some() && else_value.is_some() && !is_static);
    assert!(matches!(
        value("x := ifx c a else b;").kind,
        ExprKind::Ifx {
            then_value: Some(_),
            else_value: Some(_),
            ..
        }
    ));
    assert!(matches!(
        value("x := ifx c else b;").kind,
        ExprKind::Ifx {
            then_value: None,
            else_value: Some(_),
            ..
        }
    ));
    assert!(matches!(
        value("x := ifx c then a;").kind,
        ExprKind::Ifx {
            then_value: Some(_),
            else_value: None,
            ..
        }
    ));
    assert!(matches!(
        value("x := #ifx c then a else b;").kind,
        ExprKind::Ifx {
            is_static: true,
            ..
        }
    ));
    // Inside an argument list the ifx ends at the comma.
    let ExprKind::Call {
        args, ..
    } = value("x := f(ifx c then 1 else 2, 3);").kind
    else {
        panic!()
    };
    assert_eq!(args.len(), 2);
}

#[test]
fn call_arguments() {
    let ExprKind::Call {
        args,
        hint,
        ..
    } = value("x := inline f(a, n = 1, ..rest,, allocator = temp);").kind
    else {
        panic!()
    };
    assert_eq!(hint, CallHint::Inline);
    assert_eq!(args.len(), 4);
    assert!(args[1].name.is_some() && !args[1].context);
    assert!(args[2].spread);
    assert!(args[3].context);
}

#[test]
fn here_string_argument_in_call() {
    let stmts = body_of("print(#string END\nhello\nEND, 7);\ns := #string END\nraw\nEND\ny := 1;");
    assert_eq!(stmts.len(), 3);
    let StmtKind::Expr(Expr {
        kind: ExprKind::Call {
            args, ..
        },
        ..
    }) = &stmts[0].kind
    else {
        panic!()
    };
    assert!(matches!(&args[0].value.kind, ExprKind::Str(s) if &**s == b"hello\n"));
    // A here-string ends its statement without a semicolon.
    assert!(matches!(
        decl(&stmts[1]).value.as_ref().unwrap().kind,
        ExprKind::Str(_)
    ));
}

#[test]
fn inferred_members_and_directive_primaries() {
    assert!(matches!(
        value("x := .FOO;").kind,
        ExprKind::InferredMember(_)
    ));
    assert!(matches!(
        value("x := #char \"a\";").kind,
        ExprKind::Char(97)
    ));
    assert!(matches!(
        value("x := #run,stallable f();").kind,
        ExprKind::Run { .. }
    ));
    assert!(matches!(
        value("x := #run { a(); };").kind,
        ExprKind::Run { .. }
    ));
    assert!(matches!(value("x := #code a + b;").kind, ExprKind::Code(_)));
    assert!(matches!(
        value("x := #location(y);").kind,
        ExprKind::Location(Some(_))
    ));
    assert!(matches!(
        value("x := #bake_arguments f(a = 1);").kind,
        ExprKind::Bake {
            constants: false,
            ..
        }
    ));
    assert!(matches!(
        value("x := #exists(foo);").kind,
        ExprKind::Exists(_, None)
    ));
    assert!(matches!(
        value("x := #asm { mov.q rax, 1; };").kind,
        ExprKind::Asm(_)
    ));
    assert!(matches!(
        value("L :: #system_library \"libc\";").kind,
        ExprKind::UnknownDirective(d) if d.operand.is_some()
    ));
    // Flags before the operand may be spaced out (Pixel_Maker's Metal bindings).
    let ExprKind::UnknownDirective(directive) =
        value("Metal :: #library, system, link_always \"Metal\";").kind
    else {
        panic!("expected #library");
    };
    assert_eq!(
        directive
            .flags
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>(),
        ["system", "link_always"]
    );
    assert!(matches!(value("x := `y;").kind, ExprKind::Backtick(_)));
    let ExprKind::Insert(insert) = value("x := #insert,scope(s) code;").kind else {
        panic!()
    };
    assert!(insert.scope.is_some());
}

#[test]
fn directive_comma_is_a_flag_only_when_it_hugs_the_directive() {
    // `#file, x` has two arguments; `#run,stallable` has a flag.
    let ExprKind::Call {
        args, ..
    } = value("x := f(#file, y);").kind
    else {
        panic!()
    };
    assert_eq!(args.len(), 2);
    let ExprKind::Call {
        args, ..
    } = value("x := f(cast(*#Context, y), z);").kind
    else {
        panic!()
    };
    assert_eq!(args.len(), 2);
}

#[test]
fn run_operand_is_a_full_expression() {
    assert!(
        matches!(value("N :: #run f() + 1;").kind, ExprKind::Run { body, .. } if matches!(&*body, RunBody::Expr(Expr { kind: ExprKind::Binary(..), .. })))
    );
}

// -- declarations ---------------------------------------------------------------
