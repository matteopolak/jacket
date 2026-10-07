//! Parser unit tests, focused on the ambiguous corners of the grammar.
use super::parse_file;
use crate::ast::*;
use crate::source::FileId;
use std::rc::Rc;

fn parse(src: &str) -> Vec<Stmt> {
    match parse_file(FileId(0), src) {
        Ok(file) => file.stmts,
        Err(d) => panic!(
            "parse error: {} (bytes {}..{})\n{src}",
            d.message, d.span.start, d.span.end
        ),
    }
}

fn error(src: &str) -> String {
    match parse_file(FileId(0), src) {
        Ok(_) => panic!("expected a parse error for:\n{src}"),
        Err(d) => d.message,
    }
}

fn decl(stmt: &Stmt) -> &Rc<Decl> {
    match &stmt.kind {
        StmtKind::Decl(decl) => decl,
        other => panic!("expected a declaration, found {other:?}"),
    }
}

/// The value of the first declaration in `src`.
fn value(src: &str) -> Expr {
    let stmts = parse(src);
    decl(&stmts[0])
        .value
        .clone()
        .expect("declaration has a value")
}

fn proc_of(expr: &Expr) -> &ProcLit {
    match &expr.kind {
        ExprKind::Proc(lit) => lit,
        other => panic!("expected a procedure, found {other:?}"),
    }
}

/// Statements of the body of `f :: () { ... }`.
fn body_of(src: &str) -> Vec<Stmt> {
    let stmts = parse(&format!("f :: () {{\n{src}\n}}"));
    proc_of(decl(&stmts[0]).value.as_ref().unwrap())
        .body
        .clone()
        .unwrap()
        .stmts
}

// -- procedure header versus parenthesized expression -------------------------

#[test]
fn parenthesized_expression_is_not_a_header() {
    assert!(matches!(
        value("x := (a + b) * c;").kind,
        ExprKind::Binary(BinOp::Mul, ..)
    ));
    assert!(matches!(value("x := (a);").kind, ExprKind::Ident(_)));
}

#[test]
fn if_condition_in_parentheses_is_not_a_header() {
    let stmts = body_of("if (x) { y(); }");
    match &stmts[0].kind {
        StmtKind::If {
            cond,
            then_branch,
            ..
        } => {
            assert!(matches!(cond.kind, ExprKind::Ident(_)));
            assert!(matches!(then_branch.kind, StmtKind::Block(_)));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn header_with_body_is_a_procedure() {
    let value = value(
        "f :: (a: int, b := 2, $T: Type, using c: *X, args: ..Any, $$k: int) -> int, bool #must { return a, true; }",
    );
    let lit = proc_of(&value);
    let h = &lit.header;
    assert_eq!(h.params.len(), 6);
    assert!(h.params[1].default.is_some() && h.params[1].ty.is_none());
    assert!(h.params[2].baked);
    assert!(h.params[3].using);
    assert!(h.params[4].variadic);
    assert!(!h.params[5].baked); // `$$k` bakes per call, not in the header.
    assert!(h.params[5].auto_bake);
    assert_eq!(h.returns.len(), 2);
    assert!(h.returns[1].must);
    assert!(lit.body.is_some());
}

#[test]
fn header_without_body_is_a_type() {
    let stmts = parse("cb: (x: int) -> int;\ncb2: (int, *float) -> void;\ncb3: () -> bool = null;");
    for stmt in &stmts {
        assert!(matches!(
            decl(stmt).ty.as_ref().unwrap().kind,
            ExprKind::ProcType(_)
        ));
    }
    let ExprKind::ProcType(header) = &decl(&stmts[1]).ty.as_ref().unwrap().kind else {
        unreachable!()
    };
    assert!(header.params.iter().all(|p| p.name.is_none()));
}

#[test]
fn proc_type_return_does_not_swallow_the_next_parameter() {
    let value = value("f :: (cb: (int) -> int, n: int) {}");
    let h = &proc_of(&value).header;
    assert_eq!(h.params.len(), 2);
    assert_eq!(h.params[1].name.unwrap().name.as_str(), "n");
}

#[test]
fn named_returns_and_foreign_flags() {
    let value =
        value("f :: (x: int) -> (a: int, b: float) #c_call #no_context #foreign libc \"fn\";");
    let lit = proc_of(&value);
    assert!(lit.body.is_none());
    assert_eq!(lit.header.returns.len(), 2);
    assert_eq!(lit.header.returns[0].name.unwrap().name.as_str(), "a");
    assert!(lit.header.flags.c_call && lit.header.flags.no_context);
    let foreign = lit.header.foreign.as_ref().unwrap();
    assert_eq!(foreign.library.unwrap().name.as_str(), "libc");
    assert!(matches!(&foreign.name, ForeignName::Named(n) if &**n == "fn"));
}

#[test]
fn modify_block_and_deprecation() {
    let value = value("f :: (a: $T) #modify { T = int; } #deprecated \"old\" {}");
    let header = &proc_of(&value).header;
    assert_eq!(header.modify.as_ref().unwrap().stmts.len(), 1);
    assert!(matches!(&header.flags.deprecated, Some(Some(msg)) if &**msg == b"old"));
}

#[test]
fn operator_declarations() {
    let stmts = parse(
        "operator + :: (a: V, b: V) -> V { return a; }\noperator [] :: (a: V, i: int) -> int { return 0; }\noperator *= :: (a: *V, b: V) {}",
    );
    let ops: Vec<_> = stmts
        .iter()
        .map(|s| {
            proc_of(decl(s).value.as_ref().unwrap())
                .header
                .operator
                .clone()
                .unwrap()
        })
        .collect();
    assert_eq!(
        ops.iter().map(|o| &**o).collect::<Vec<_>>(),
        ["+", "[]", "*="]
    );
}

#[test]
fn lambdas() {
    assert!(matches!(
        value("f :: x => x * 2;").kind,
        ExprKind::Lambda { .. }
    ));
    let ExprKind::Lambda {
        header, ..
    } = value("f :: (a, b) => a + b;").kind
    else {
        panic!()
    };
    assert_eq!(header.params.len(), 2);
}

// -- types as expressions -----------------------------------------------------

#[test]
fn pointer_type_and_address_of_share_the_unary_star() {
    let stmts = parse("p: *int = *x;\nq := a * b;\nr := *a.b;");
    assert!(matches!(
        decl(&stmts[0]).ty.as_ref().unwrap().kind,
        ExprKind::Unary(UnOp::Star, _)
    ));
    assert!(matches!(
        decl(&stmts[0]).value.as_ref().unwrap().kind,
        ExprKind::Unary(UnOp::Star, _)
    ));
    assert!(matches!(
        decl(&stmts[1]).value.as_ref().unwrap().kind,
        ExprKind::Binary(BinOp::Mul, ..)
    ));
    // `*a.b` takes the address of the member.
    let ExprKind::Unary(UnOp::Star, inner) = &decl(&stmts[2]).value.as_ref().unwrap().kind else {
        panic!()
    };
    assert!(matches!(inner.kind, ExprKind::Member(..)));
}

#[test]
fn postfix_dereference_binds_tighter_than_member_access() {
    let ExprKind::Member(base, field) = value("x := p.*.next;").kind else {
        panic!()
    };
    assert_eq!(field.name.as_str(), "next");
    assert!(matches!(base.kind, ExprKind::Unary(UnOp::Deref, _)));
    assert!(matches!(
        value("x := << p;").kind,
        ExprKind::Unary(UnOp::Deref, _)
    ));
    assert!(matches!(
        value("x := (.*) p;").kind,
        ExprKind::Unary(UnOp::Deref, _)
    ));
}

#[test]
fn array_types() {
    let ExprKind::ArrayType {
        size,
        elem,
    } = value("T :: [4] *int;").kind
    else {
        panic!()
    };
    assert!(matches!(size, ArraySize::Fixed(_)));
    assert!(matches!(elem.kind, ExprKind::Unary(UnOp::Star, _)));
    assert!(matches!(
        value("T :: [] u8;").kind,
        ExprKind::ArrayType {
            size: ArraySize::View,
            ..
        }
    ));
    assert!(matches!(
        value("T :: [..] [..] u8;").kind,
        ExprKind::ArrayType {
            size: ArraySize::Resizable,
            ..
        }
    ));
}

#[test]
fn array_literal_with_array_type_prefix() {
    let ExprKind::ArrayLit {
        ty: Some(ty),
        elems,
    } = value("x := [2]int.[1, 2];").kind
    else {
        panic!()
    };
    assert!(matches!(ty.kind, ExprKind::ArrayType { .. }));
    assert_eq!(elems.len(), 2);
    assert!(matches!(
        value("x := .[1, 2, 3];").kind,
        ExprKind::ArrayLit {
            ty: None,
            ..
        }
    ));
}

#[test]
fn polymorphic_names_and_restrictions() {
    let value = value("f :: (a: $T/Number, b: $U/interface Cmp, c: [$N] $E) {}");
    let h = &proc_of(&value).header;
    assert!(matches!(
        h.params[0].ty.as_ref().unwrap().kind,
        ExprKind::PolyRestricted {
            interface: false,
            ..
        }
    ));
    assert!(matches!(
        h.params[1].ty.as_ref().unwrap().kind,
        ExprKind::PolyRestricted {
            interface: true,
            ..
        }
    ));
    let ExprKind::ArrayType {
        size: ArraySize::Fixed(n),
        elem,
    } = &h.params[2].ty.as_ref().unwrap().kind
    else {
        panic!()
    };
    assert!(
        matches!(n.kind, ExprKind::PolyVar { .. }) && matches!(elem.kind, ExprKind::PolyVar { .. })
    );
}

#[test]
fn type_directive_modifiers() {
    assert!(matches!(
        value("U :: #type,distinct u32;").kind,
        ExprKind::TypeDirective {
            modifier: TypeModifier::Distinct,
            ..
        }
    ));
    assert!(matches!(
        value("U :: #type, distinct u32;").kind,
        ExprKind::TypeDirective {
            modifier: TypeModifier::Distinct,
            ..
        }
    ));
    assert!(
        matches!(value("F :: #type (int) -> int;").kind, ExprKind::TypeDirective { ty, .. } if matches!(ty.kind, ExprKind::ProcType(_)))
    );
}

// -- aggregates -----------------------------------------------------------------

#[test]
fn struct_literal_versus_block() {
    let ExprKind::StructLit {
        ty,
        fields,
    } = value("v := Foo.{a = 1, 2};").kind
    else {
        panic!()
    };
    assert!(ty.is_some());
    assert_eq!(fields[0].name.unwrap().name.as_str(), "a");
    assert!(fields[1].name.is_none());
    assert!(matches!(
        value("v := .{1, 2};").kind,
        ExprKind::StructLit {
            ty: None,
            ..
        }
    ));
    // A brace after an `if` condition is a block, never a literal.
    let stmts = body_of("if x { y = 1; }");
    assert!(
        matches!(&stmts[0].kind, StmtKind::If { then_branch, .. } if matches!(then_branch.kind, StmtKind::Block(_)))
    );
}

// rules: struct.17
#[test]
fn struct_literal_without_dot() {
    let is_literal = |e: &Expr| {
        matches!(
            e.kind,
            ExprKind::StructLit {
                ty: None,
                ..
            }
        )
    };
    assert!(is_literal(&value("v: Desc = {.GENERAL, 1};")));
    assert!(is_literal(&value("v: E = {1, {2, 3}};")));
    let ExprKind::Call {
        args, ..
    } = value("v := f({.A, 1}, {x = 2});").kind
    else {
        panic!()
    };
    assert!(args.iter().all(|a| is_literal(&a.value)));
    // Statements, declarations and lambda bodies keep their braces as blocks.
    for src in [
        "v := f({ if x return 1; });",
        "v := f({ y := 2; });",
        "v := g(x => { return x; });",
    ] {
        let ExprKind::Call {
            args, ..
        } = value(src).kind
        else {
            panic!("{src}")
        };
        assert!(!is_literal(&args[0].value), "{src}");
    }
    // Fields named with `:` read as declarations; only then does the error explain `=`.
    let help = |src: &str| match parse_file(FileId(0), src) {
        Ok(_) => panic!("expected a parse error for:\n{src}"),
        Err(d) => d.help,
    };
    assert!(help("v := f({x: 1, y: 2});")[0].contains("names its fields with `=`"));
    assert!(help("v := f({ y := 2 z := 3; });").is_empty());
}

#[test]
fn struct_body_forms() {
    let value = value(
        "S :: struct (T: Type, N := 4) #type_info_none {
            using base: Base;
            #as using other: Other;
            x, y: T;
            #if N > 2 { z: int; } else { w: int; }
            #place x;
            a: float;
            union { i: s32; f: float; }
            struct { p: int; }
            tag: u8 #align 4;
        } #no_padding",
    );
    let ExprKind::Struct(lit) = value.kind else {
        panic!()
    };
    assert_eq!(lit.params.len(), 2);
    assert!(lit.flags.type_info_none && lit.flags.no_padding);
    assert_eq!(lit.body.len(), 9);
    assert!(matches!(&lit.body[0].kind, StmtKind::Decl(d) if d.using));
    assert!(matches!(&lit.body[1].kind, StmtKind::Decl(d) if d.using && d.as_));
    assert!(matches!(&lit.body[3].kind, StmtKind::StaticIf { .. }));
    assert!(matches!(&lit.body[4].kind, StmtKind::Place(_)));
    assert!(
        matches!(&lit.body[6].kind, StmtKind::Expr(Expr { kind: ExprKind::Struct(s), .. }) if s.kind == StructKind::Union)
    );
    assert!(matches!(&lit.body[8].kind, StmtKind::Decl(d) if d.align.is_some()));
}

// rules: enum.17
#[test]
fn enum_forms() {
    let ExprKind::Enum(lit) =
        value("E :: enum_flags u32 #specified { A; B :: 4; C :: 1 << 3; }").kind
    else {
        panic!()
    };
    assert!(lit.flags_enum && lit.specified && lit.base.is_some());
    assert_eq!(lit.items.len(), 3);
    let ExprKind::Enum(lit) = value("E :: enum { A; #if X { B; } else { C; } }").kind else {
        panic!()
    };
    assert!(lit.base.is_none());
    assert!(matches!(lit.items[1], EnumItem::If { .. }));
    // `A : : 5` spells out the constant form's empty type.
    let ExprKind::Enum(lit) = value("E :: enum { A :: 4; B : :5; C; }").kind else {
        panic!()
    };
    assert!(matches!(&lit.items[1], EnumItem::Member(m) if m.value.is_some()));
}

#[test]
fn tagged_unions() {
    let ExprKind::Struct(lit) =
        value("U :: union kind : Kind { .A ,, a: int; .B ,, b: float; }").kind
    else {
        panic!()
    };
    assert!(lit.tag.is_some());
    let Some(ExprKind::InferredMember(tag)) = decl(&lit.body[1]).union_tag.clone().map(|t| t.kind)
    else {
        panic!()
    };
    assert_eq!(tag.name.as_str(), "B");
    let ExprKind::Struct(lit) =
        value("U :: union kind: s8 = 4 { 4,, a: u8; b: u8; -12,, c: s16; }").kind
    else {
        panic!()
    };
    assert!(decl(&lit.body[0]).union_tag.is_some());
    assert!(decl(&lit.body[1]).union_tag.is_none());
    assert!(decl(&lit.body[2]).union_tag.is_some());
}

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
            ExprKind::Int(v) => v.to_string(),
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
        ExprKind::Exists(_)
    ));
    assert!(matches!(
        value("x := #asm { mov.q rax, 1; };").kind,
        ExprKind::Asm(_)
    ));
    assert!(matches!(
        value("L :: #system_library \"libc\";").kind,
        ExprKind::UnknownDirective {
            operand: Some(_),
            ..
        }
    ));
    // Flags before the operand may be spaced out (Pixel_Maker's Metal bindings).
    let ExprKind::UnknownDirective {
        flags, ..
    } = value("Metal :: #library, system, link_always \"Metal\";").kind
    else {
        panic!("expected #library");
    };
    assert_eq!(
        flags.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(),
        ["system", "link_always"]
    );
    assert!(matches!(value("x := `y;").kind, ExprKind::Backtick(_)));
    let ExprKind::Insert {
        scope, ..
    } = value("x := #insert,scope(s) code;").kind
    else {
        panic!()
    };
    assert!(scope.is_some());
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

#[test]
fn declaration_forms() {
    let stmts = parse(
        "a: int;\nb: int = 1;\nc := 2;\nd :: 3;\ne: int : 4;\nf, g := h();\ni, j: float;\nk := ---;",
    );
    let kinds: Vec<_> = stmts
        .iter()
        .map(|s| (decl(s).kind, decl(s).ty.is_some(), decl(s).value.is_some()))
        .collect();
    assert_eq!(
        kinds,
        [
            (DeclKind::Var, true, false),
            (DeclKind::Var, true, true),
            (DeclKind::Var, false, true),
            (DeclKind::Const, false, true),
            (DeclKind::Const, true, true),
            (DeclKind::Var, false, true),
            (DeclKind::Var, true, false),
            (DeclKind::Var, false, true),
        ]
    );
    assert_eq!(decl(&stmts[5]).names.len(), 2);
    assert!(matches!(
        decl(&stmts[7]).value.as_ref().unwrap().kind,
        ExprKind::Uninit
    ));
}

#[test]
fn multiple_values_and_mixed_declarations() {
    let stmts = parse("a, b := 1, 2;\nx=, y := f();\nm:, n = g();\np, q:, r := h();");
    assert_eq!(decl(&stmts[0]).extra_values.len(), 1);
    assert_eq!(decl(&stmts[1]).existing, [true, false]);
    assert_eq!(decl(&stmts[2]).existing, [false, true]);
    // `q:` in a declaration is redundant: every name is new.
    assert!(decl(&stmts[3]).existing.iter().all(|&e| !e));
}

// rules: decl.22
#[test]
fn mixed_declarations_assign_to_places() {
    let stmts = parse("ok:, t.str = f();\nt.items[i].*=, n := g();\na.b, c = h();");
    let first = decl(&stmts[0]);
    assert_eq!(first.existing, [false, true]);
    assert!(first.targets[0].is_none());
    assert!(matches!(
        first.targets[1].as_ref().map(|e| &e.kind),
        Some(ExprKind::Member(..))
    ));
    assert_eq!(first.names[1].name.as_str(), "t");
    let second = decl(&stmts[1]);
    assert_eq!(second.existing, [true, false]);
    assert!(second.targets[0].is_some());
    // Without a marker a list of places is a plain multiple assignment.
    assert!(matches!(stmts[2].kind, StmtKind::Assign { .. }));
    assert!(error("t.x:, b = f();").contains("only a name can be declared"));
}

#[test]
fn declaration_flags_and_notes() {
    let stmts =
        parse("x: int #align 16 @Note;\nF :: () {} @Public @Other\nG :: () {}\n#no_reset y := 1;");
    assert!(decl(&stmts[0]).align.is_some());
    assert_eq!(decl(&stmts[0]).notes.len(), 1);
    assert_eq!(decl(&stmts[1]).notes.len(), 2);
    assert!(decl(&stmts[2]).notes.is_empty());
    assert_eq!(decl(&stmts[3]).flags[0].name.as_str(), "no_reset");
}

#[test]
fn procedures_and_structs_need_no_semicolon() {
    let stmts = parse("A :: () {}\nB :: struct { x: int; }\nC :: enum { X; }\nD :: 1;");
    assert_eq!(stmts.len(), 4);
}

#[test]
fn statement_starting_with_prefix_operator_after_a_block() {
    // `*p = 1;` on the line after a brace-terminated declaration is a new statement.
    let stmts = parse("f :: () {}\n*p = 1;");
    assert_eq!(stmts.len(), 2);
    assert!(matches!(&stmts[1].kind, StmtKind::Assign { .. }));
}

#[test]
fn imports_and_loads() {
    let stmts = parse(
        "#import \"Basic\";\nM :: #import \"Math\"(A = 1);\n#import,file \"f.jai\";\n#import,dir \"d\";\n#import,string \"x :: 1;\";\n#load \"g.jai\";",
    );
    let StmtKind::Import(i0) = &stmts[0].kind else {
        panic!()
    };
    assert!(matches!(&i0.source, ImportSource::Module(m) if &**m == "Basic") && i0.name.is_none());
    let StmtKind::Import(i1) = &stmts[1].kind else {
        panic!()
    };
    assert_eq!(i1.name.unwrap().name.as_str(), "M");
    assert_eq!(i1.params.len(), 1);
    assert!(
        matches!(&stmts[2].kind, StmtKind::Import(i) if matches!(i.source, ImportSource::File(_)))
    );
    assert!(
        matches!(&stmts[3].kind, StmtKind::Import(i) if matches!(i.source, ImportSource::Dir(_)))
    );
    assert!(
        matches!(&stmts[4].kind, StmtKind::Import(i) if matches!(i.source, ImportSource::String(_)))
    );
    assert!(matches!(&stmts[5].kind, StmtKind::Load { .. }));
}

#[test]
fn file_level_directives() {
    let stmts = parse(
        "#scope_file\n#scope_export\n#scope_module\n#add_context ctx: int = 1;\n#module_parameters (A := 1) (B := 2);\n#placeholder X;\n#assert A == 1 \"msg\";\n#run main();\n#insert \"x :: 1;\";\n#poke_name Basic operator==;\n#library \"x\";",
    );
    assert!(matches!(stmts[0].kind, StmtKind::Scope(ScopeKind::File)));
    assert!(matches!(stmts[3].kind, StmtKind::AddContext(_)));
    assert!(
        matches!(&stmts[4].kind, StmtKind::ModuleParameters { params, runtime_params, .. } if params.len() == 1 && runtime_params.len() == 1)
    );
    assert!(matches!(stmts[5].kind, StmtKind::Placeholder(_)));
    assert!(matches!(
        &stmts[6].kind,
        StmtKind::Assert {
            message: Some(_),
            ..
        }
    ));
    assert!(matches!(stmts[7].kind, StmtKind::Run(_)));
    assert!(matches!(stmts[8].kind, StmtKind::Insert { .. }));
    assert!(matches!(&stmts[9].kind, StmtKind::Directive { args, .. } if args.len() == 2));
    assert!(matches!(stmts[10].kind, StmtKind::Directive { .. }));
}

#[test]
fn static_if_at_file_scope() {
    let stmts = parse(
        "#if A { x :: 1; y :: 2; } else #if B z :: 3; else { w :: 4; }\n#if C print(\"hi\");\n#if OS == { case .A; #load \"a.jai\"; case; #load \"b.jai\"; }\n#if #complete N == { case 1; a :: 1; case 2; a :: 2; }",
    );
    let StmtKind::StaticIf {
        then_branch,
        else_branch,
        ..
    } = &stmts[0].kind
    else {
        panic!()
    };
    assert_eq!(then_branch.len(), 2);
    assert!(
        matches!(&else_branch[0].kind, StmtKind::StaticIf { then_branch, else_branch, .. } if then_branch.len() == 1 && else_branch.len() == 1)
    );
    assert!(
        matches!(&stmts[1].kind, StmtKind::StaticIf { then_branch, .. } if then_branch.len() == 1)
    );
    assert!(matches!(&stmts[2].kind, StmtKind::StaticSwitch { cases, .. } if cases.len() == 2));
    // `#if #complete x == {}` (KodaJai) parses as a plain static switch.
    assert!(matches!(&stmts[3].kind, StmtKind::StaticSwitch { cases, .. } if cases.len() == 2));
}

// -- statements -----------------------------------------------------------------

#[test]
fn switch_statements() {
    let stmts = body_of("if x == {\n case 1; a();\n case 2, 3; b(); #through;\n case; c();\n}");
    let StmtKind::Switch {
        cases,
        complete,
        ..
    } = &stmts[0].kind
    else {
        panic!()
    };
    assert!(!complete);
    assert_eq!(cases.len(), 3);
    assert_eq!(cases[1].values.len(), 2);
    assert!(cases[1].through && !cases[0].through);
    assert!(cases[2].values.is_empty());
    let stmts =
        body_of("if #complete x == { case .A; a(); }\nif x == #complete { case .A: { a(); } }");
    assert!(matches!(
        stmts[0].kind,
        StmtKind::Switch {
            complete: true,
            ..
        }
    ));
    assert!(matches!(
        stmts[1].kind,
        StmtKind::Switch {
            complete: true,
            ..
        }
    ));
}

#[test]
fn equality_comparison_is_not_a_switch() {
    let stmts = body_of("if x == y { a(); } else if x == z then b(); else c();");
    let StmtKind::If {
        cond,
        else_branch,
        ..
    } = &stmts[0].kind
    else {
        panic!()
    };
    assert!(matches!(cond.kind, ExprKind::Binary(BinOp::Eq, ..)));
    assert!(matches!(
        &else_branch.as_ref().unwrap().kind,
        StmtKind::If {
            else_branch: Some(_),
            ..
        }
    ));
}

#[test]
fn for_loop_forms() {
    let stmts = body_of(
        "for x, i: arr {}\nfor arr {}\nfor 0..n-1 {}\nfor < arr {}\nfor * p: arr {}\nfor :it_iter v: arr {}\nfor #v2 < a..b {}\nfor i: 1..10 print(i);",
    );
    let fors: Vec<&For> = stmts
        .iter()
        .map(|s| match &s.kind {
            StmtKind::For(f) => &**f,
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        (fors[0].it.is_some(), fors[0].index.is_some()),
        (true, true)
    );
    assert!(fors[1].it.is_none() && matches!(fors[1].over, ForOver::Collection(_)));
    assert!(matches!(fors[2].over, ForOver::Range(..)));
    assert!(fors[3].reverse);
    assert!(fors[4].by_pointer && fors[4].it.is_some());
    assert!(fors[5].iterator.is_some());
    assert!(
        fors[6].reverse && fors[6].flags.len() == 1 && matches!(fors[6].over, ForOver::Range(..))
    );
    assert!(matches!(fors[7].body.kind, StmtKind::Expr(_)));
}

#[test]
fn while_break_continue_remove() {
    let stmts = body_of(
        "while outer := i < 3 { break outer; }\nwhile x continue;\nfor a { remove; }\nfor a { remove it; }",
    );
    assert!(matches!(
        &stmts[0].kind,
        StmtKind::While {
            label: Some(_),
            ..
        }
    ));
    assert!(
        matches!(&stmts[1].kind, StmtKind::While { label: None, body, .. } if matches!(body.kind, StmtKind::Continue(None)))
    );
}

#[test]
fn return_defer_using_push_context() {
    let stmts = body_of(
        "return a, b;\nreturn x = 1;\n`return 1;\ndefer free(p);\n`defer { a(); }\nusing x;\nusing,only(a, b) y;\nusing,except(c) z;\npush_context ctx { f(); }",
    );
    assert!(
        matches!(&stmts[0].kind, StmtKind::Return { values, backtick: false } if values.len() == 2)
    );
    assert!(matches!(&stmts[1].kind, StmtKind::Return { values, .. } if values[0].name.is_some()));
    assert!(matches!(
        &stmts[2].kind,
        StmtKind::Return {
            backtick: true,
            ..
        }
    ));
    assert!(matches!(
        &stmts[3].kind,
        StmtKind::Defer {
            backtick: false,
            ..
        }
    ));
    assert!(matches!(
        &stmts[4].kind,
        StmtKind::Defer {
            backtick: true,
            ..
        }
    ));
    assert!(matches!(
        &stmts[5].kind,
        StmtKind::Using {
            filter: UsingFilter::None,
            ..
        }
    ));
    assert!(
        matches!(&stmts[6].kind, StmtKind::Using { filter: UsingFilter::Only(names), .. } if names.len() == 2)
    );
    assert!(matches!(
        &stmts[7].kind,
        StmtKind::Using {
            filter: UsingFilter::Except(_),
            ..
        }
    ));
    assert!(matches!(&stmts[8].kind, StmtKind::PushContext { .. }));
}

#[test]
fn assignments() {
    let stmts = body_of(
        "a = b;\na, b = b, a;\na += 1; a -= 1; a *= 2; a /= 2; a %= 2;\na &= 1; a |= 1; a ^= 1; a <<= 1; a >>= 1; a <<<= 1; a >>>= 1; a &&= b; a ||= b;",
    );
    assert!(
        matches!(&stmts[1].kind, StmtKind::Assign { lhs, rhs, .. } if lhs.len() == 2 && rhs.len() == 2)
    );
    let ops: Vec<_> = stmts[2..]
        .iter()
        .map(|s| match &s.kind {
            StmtKind::Assign {
                op, ..
            } => *op,
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(ops.len(), 14);
    assert!(ops.iter().all(|op| matches!(op, AssignOp::Op(_))));
    assert_eq!(ops[12], AssignOp::Op(BinOp::And));
    assert_eq!(ops[13], AssignOp::Op(BinOp::Or));
}

// -- diagnostics ----------------------------------------------------------------

#[test]
fn diagnostics_name_the_expected_and_found_tokens() {
    assert_eq!(
        error("x := 1\ny := 2;"),
        "expected `;` after statement, found `y`"
    );
    assert_eq!(
        error("f :: () { return 1 }"),
        "expected `;` after `return`, found `}`"
    );
    assert_eq!(error("x := ;"), "expected expression, found `;`");
    assert_eq!(
        error("x := f(1, 2;"),
        "expected `)` in argument list, found `;`"
    );
    assert_eq!(
        error("S :: struct { x: int;"),
        "expected `}` to close the block, found end of file"
    );
    assert_eq!(
        error("#import 5;"),
        "expected string literal after `#import`, found number"
    );
}

#[test]
fn parenthesized_cast_with_flags_is_not_a_header() {
    let stmts = parse("x := (cast,no_check(u64) a & m);");
    let StmtKind::Decl(decl) = &stmts[0].kind else {
        panic!("expected a declaration")
    };
    // An expression, not a procedure header; the cast takes `a & m` as its value.
    let ExprKind::Cast {
        value: inner, ..
    } = &decl.value.as_ref().unwrap().kind
    else {
        panic!("expected a cast")
    };
    assert!(matches!(inner.kind, ExprKind::Binary(BinOp::BitAnd, ..)));
}

fn asm_block(src: &str) -> Rc<AsmBlock> {
    match value(src).kind {
        ExprKind::Asm(block) => block,
        other => panic!("expected an #asm block, found {other:?}"),
    }
}

#[test]
fn asm_instruction_forms() {
    let block = asm_block(
        "x := #asm AVX, AVX2 { mov.q a, 1; lock_btr.d [p + i*8 - 16], n; popcnt?T r, v; mov.64 q:, 7; pause; };",
    );
    assert_eq!(block.features.len(), 2);
    let AsmItem::Inst(mov) = &block.items[0] else {
        panic!("expected an instruction");
    };
    assert_eq!(mov.mnemonic.name.as_str(), "mov");
    assert!(matches!(&mov.size, Some(AsmSize::Suffix(s)) if s.name.as_str() == "q"));
    let AsmItem::Inst(btr) = &block.items[1] else {
        panic!("expected an instruction");
    };
    assert_eq!(btr.mnemonic.name.as_str(), "lock_btr");
    let AsmOperand::Mem(mem) = &btr.operands[0] else {
        panic!("expected a memory operand");
    };
    assert_eq!(mem.terms.len(), 3);
    assert!(mem.terms[1].scale.is_some() && !mem.terms[1].negate);
    assert!(mem.terms[2].negate);
    let AsmItem::Inst(popcnt) = &block.items[2] else {
        panic!("expected an instruction");
    };
    assert!(matches!(popcnt.size, Some(AsmSize::Dynamic(_))));
    let AsmItem::Inst(wide) = &block.items[3] else {
        panic!("expected an instruction");
    };
    assert!(matches!(&wide.size, Some(AsmSize::Suffix(s)) if s.name.as_str() == "64"));
    assert!(matches!(&wide.operands[0], AsmOperand::Decl(d) if d.colon && d.class.is_none()));
    assert_eq!(block.items.len(), 5);
}

#[test]
fn asm_declarations_and_pins() {
    let block = asm_block("x := #asm { t: gpr === a; u === c; mov w: gpr === 15, 10; v: vec; };");
    let AsmItem::Decl(t) = &block.items[0] else {
        panic!("expected a declaration");
    };
    assert!(t.colon && t.class.is_some() && matches!(t.pin, Some(AsmPin::Name(_))));
    let AsmItem::Decl(u) = &block.items[1] else {
        panic!("expected a pin");
    };
    assert!(!u.colon && u.pin.is_some());
    let AsmItem::Inst(mov) = &block.items[2] else {
        panic!("expected an instruction");
    };
    assert!(matches!(
        &mov.operands[0],
        AsmOperand::Decl(d) if matches!(d.pin, Some(AsmPin::Index(15)))
    ));
}

#[test]
fn asm_errors() {
    assert!(error("x := #asm { mov a, 1 mov b, 2; };").contains("expected `;`"));
    assert!(error("x := #asm { mov a, [b + ; };").contains("expected"));
    assert!(error("x := #asm { mov a, 1;").contains("unterminated"));
}

#[test]
fn deep_nesting_is_an_error_not_a_stack_overflow() {
    // Found by the `parser` fuzz target: a few kilobytes of `(` or `{` overflowed the stack.
    let n = super::MAX_NESTING + 10;
    let cases = [
        format!("x := {}1{};", "(".repeat(n), ")".repeat(n)),
        format!("main :: () {{{}{}}}", "{".repeat(n), "}".repeat(n)),
        format!("x: {}int;", "*".repeat(n)),
        format!("x: {}int;", "[1]".repeat(n)),
        format!("A :: {}int;{}", "struct { a: ".repeat(n), "}".repeat(n)),
    ];
    // Reaching the limit itself takes more stack than a test thread has in a debug build.
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            for src in &cases {
                assert!(error(src).contains("nested too deeply"), "{src:.40}");
            }
            let fine = 200;
            parse(&format!("x := {}1{};", "(".repeat(fine), ")".repeat(fine)));
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn nested_assert_is_parsed_once() {
    // Found by the `parser` fuzz target: `#assert(` tried the `(cond, message)` form, rewound and
    // parsed again, so every nested `#assert(` doubled the work (15 KB took minutes).
    let n = 40;
    let nested = format!("{}x{}", "#assert(".repeat(n), ")".repeat(n));
    parse(&format!("#assert {nested};"));
    parse(&format!("#assert({nested}, \"message\");"));
    assert!(!error(&format!("#assert {};", "#assert(".repeat(n))).is_empty());
    // Both forms still parse.
    parse("#assert(size_of(int) == 8, \"words\");");
    parse("#assert(f(a, b));");
    parse("#assert (a) == b, \"m\";");
}
