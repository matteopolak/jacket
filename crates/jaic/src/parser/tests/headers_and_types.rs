//! Procedure headers, type forms and struct, enum and union bodies.
use super::{body_of, decl, parse, proc_of, value};
use crate::ast::*;
use crate::parser::parse_file;
use crate::source::FileId;

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
fn unparenthesised_proc_type_return_before_a_body_is_rejected() {
    // `{` after `-> (int, int) -> bool` is ambiguous; the return type must be parenthesised.
    assert_eq!(
        super::error("f :: (n: int) -> (int, int) -> bool { return null; }"),
        "a procedure type in a return type must be parenthesised before the body: write `-> ((A, B) -> C) {`"
    );
    let value = value("f :: (n: int) -> ((int, int) -> bool) { return null; }");
    let lit = proc_of(&value);
    assert!(lit.body.is_some());
    let ret = lit.header.returns[0].ty.as_ref().unwrap();
    assert!(matches!(&ret.kind, ExprKind::ProcType(h) if h.params.len() == 2));
}

#[test]
fn named_returns_and_foreign_flags() {
    let value =
        value("f :: (x: int) -> (a: int, b: float) #c_call #no_context #foreign libc \"fn\";");
    let lit = proc_of(&value);
    assert!(lit.body.is_none());
    assert_eq!(lit.header.returns.len(), 2);
    assert_eq!(lit.header.returns[0].name.unwrap().name.as_str(), "a");
    assert!(lit.header.flags.c_call() && lit.header.flags.no_context());
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
    assert!(lit.flags.has(StructBits::TYPE_INFO_NONE) && lit.flags.has(StructBits::NO_PADDING));
    assert_eq!(lit.body.len(), 9);
    assert!(matches!(&lit.body[0].kind, StmtKind::Decl(d) if d.using));
    assert!(matches!(&lit.body[1].kind, StmtKind::Decl(d) if d.using && d.as_));
    assert!(matches!(&lit.body[3].kind, StmtKind::StaticIf(_)));
    assert!(matches!(&lit.body[4].kind, StmtKind::Place(_)));
    assert!(
        matches!(&lit.body[6].kind, StmtKind::Expr(Expr { kind: ExprKind::Struct(s), .. }) if s.kind == StructKind::Union)
    );
    assert!(matches!(&lit.body[8].kind, StmtKind::Decl(d) if d.align().is_some()));
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
    let Some(ExprKind::InferredMember(tag)) =
        decl(&lit.body[1]).union_tag().cloned().map(|t| t.kind)
    else {
        panic!()
    };
    assert_eq!(tag.name.as_str(), "B");
    let ExprKind::Struct(lit) =
        value("U :: union kind: s8 = 4 { 4,, a: u8; b: u8; -12,, c: s16; }").kind
    else {
        panic!()
    };
    assert!(decl(&lit.body[0]).union_tag().is_some());
    assert!(decl(&lit.body[1]).union_tag().is_none());
    assert!(decl(&lit.body[2]).union_tag().is_some());
}
