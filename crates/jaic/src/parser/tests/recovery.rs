//! Diagnostics, inline assembly, nesting limits and dead platform branches.
use super::{error, parse, proc_of, value};
use crate::ast::*;
use crate::source::FileId;
use std::rc::Rc;

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
    let n = crate::parser::MAX_NESTING + 10;
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
    // Forty levels of debug frames come close to the 2 MiB of a test thread (a macOS CI run
    // died of it), so the nested parses get a thread of their own.
    std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(move || {
            parse(&format!("#assert {nested};"));
            parse(&format!("#assert({nested}, \"message\");"));
            assert!(!error(&format!("#assert {};", "#assert(".repeat(n))).is_empty());
        })
        .unwrap()
        .join()
        .unwrap();
    // Both forms still parse.
    parse("#assert(size_of(int) == 8, \"words\");");
    parse("#assert(f(a, b));");
    parse("#assert (a) == b, \"m\";");
}

#[test]
fn nested_ifx_statements_are_parsed_once() {
    // Found by the `parser` fuzz target: a statement starting with `ifx` was parsed as an
    // assignment `if`, then rewound and parsed again as an expression, so each `{ifx` nested in
    // the condition doubled the work (2.5 KB timed out).
    let n = 40;
    // Debug frames are large; a test thread has 2 MiB.
    std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(move || {
            assert!(!error(&format!("main :: () {{ {} }}", "{ifx ".repeat(n))).is_empty());
            for branch in [
                "ifx c then { ",
                "ifx c then x = { ",
                "ifx c then f() else { ",
            ] {
                assert!(!error(&format!("main :: () {{ {} }}", branch.repeat(n))).is_empty());
            }
        })
        .unwrap()
        .join()
        .unwrap();
    // Both forms still parse: a branch that assigns makes an `if`, anything else an expression.
    let body = |src: &str| {
        let value = value(&format!("main :: () {{ {src} }}"));
        proc_of(&value).body.clone().unwrap().stmts
    };
    let stmts = body(
        "ifx c then a = 1 else a = 2; ifx c then f() else b = 3; ifx c then f() else g(); ifx c f(); ifx c then { f(); } else { g(); }",
    );
    assert!(
        matches!(stmts[0].kind, StmtKind::If { .. }),
        "{:?}",
        stmts[0].kind
    );
    assert!(
        matches!(stmts[1].kind, StmtKind::If { .. }),
        "{:?}",
        stmts[1].kind
    );
    for stmt in &stmts[2..] {
        let StmtKind::Expr(e) = &stmt.kind else {
            panic!("expected an expression statement, found {:?}", stmt.kind);
        };
        assert!(matches!(e.kind, ExprKind::Ifx { .. }), "{:?}", e.kind);
    }
    // `ifx` as a name.
    assert!(matches!(body("ifx := 1;")[0].kind, StmtKind::Decl(_)));
}

#[test]
fn dead_platform_branches_are_skipped_for_a_library_target() {
    let target = crate::parser::PlatformTarget {
        os: "MACOS",
        cpu: "ARM64",
    };
    let src = r#"
#if OS == .LINUX || (OS == .WINDOWS && CPU == .X64) {
    // a brace } in a comment, "a } string", @note}
    gone :: .{ 1, 2 };
    /* nested /* } */ } */
    bad syntax here ((
}
#if !(OS == .MACOS) { ignored :: 1; } else { kept :: 2; }
#if OS == .MACOS && CPU == .ARM64 { here :: .{ 3 }; }
#if OS == .MACOS { #if CPU == .X64 { nested_gone :: 1; } else { nested_kept :: 2; } }
"#;
    let file = crate::parser::parse_file_for_target(FileId(0), src, target).expect("parses");
    let mut names = Vec::new();
    fn collect(stmts: &[Stmt], names: &mut Vec<String>, taken: bool) {
        for s in stmts {
            match &s.kind {
                StmtKind::StaticIf(branch) => {
                    let ExprKind::Bool(value) = branch.cond.kind else {
                        panic!("condition was not decided");
                    };
                    collect(&branch.then_branch, names, taken && value);
                    collect(&branch.else_branch, names, taken && !value);
                }
                StmtKind::Decl(d) if taken => {
                    names.extend(d.names.iter().map(|n| n.name.as_str().to_string()))
                }
                _ => {}
            }
        }
    }
    collect(&file.stmts, &mut names, true);
    assert_eq!(names, ["kept", "here", "nested_kept"]);
}
