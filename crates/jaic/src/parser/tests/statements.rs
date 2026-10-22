//! Declarations, imports, control flow and assignments.
use super::{body_of, decl, error, parse};
use crate::ast::*;

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
    assert_eq!(decl(&stmts[0]).extra_values().len(), 1);
    assert_eq!(decl(&stmts[1]).existing(), [true, false]);
    assert_eq!(decl(&stmts[2]).existing(), [false, true]);
    // `q:` in a declaration is redundant: every name is new.
    assert!(decl(&stmts[3]).existing().iter().all(|&e| !e));
}

// rules: decl.22
#[test]
fn mixed_declarations_assign_to_places() {
    let stmts = parse("ok:, t.str = f();\nt.items[i].*=, n := g();\na.b, c = h();");
    let first = decl(&stmts[0]);
    assert_eq!(first.existing(), [false, true]);
    assert!(first.targets()[0].is_none());
    assert!(matches!(
        first.targets()[1].as_ref().map(|e| &e.kind),
        Some(ExprKind::Member(..))
    ));
    assert_eq!(first.names[1].name.as_str(), "t");
    let second = decl(&stmts[1]);
    assert_eq!(second.existing(), [true, false]);
    assert!(second.targets()[0].is_some());
    // Without a marker a list of places is a plain multiple assignment.
    assert!(matches!(stmts[2].kind, StmtKind::Assign { .. }));
    assert!(error("t.x:, b = f();").contains("only a name can be declared"));
}

#[test]
fn declaration_flags_and_notes() {
    let stmts =
        parse("x: int #align 16; @Note\nF :: () {} @Public @Other\nG :: () {}\n#no_reset y := 1;");
    assert!(decl(&stmts[0]).align().is_some());
    assert_eq!(decl(&stmts[0]).notes.len(), 1);
    assert_eq!(decl(&stmts[1]).notes.len(), 2);
    assert!(decl(&stmts[2]).notes.is_empty());
    assert_eq!(decl(&stmts[3]).flags()[0].name.as_str(), "no_reset");
}

#[test]
fn no_reset_may_follow_using() {
    let stmts = parse("using #no_reset m: int #align 64;\n#no_reset using n: int;");
    assert_eq!(decl(&stmts[0]).flags()[0].name.as_str(), "no_reset");
    assert!(decl(&stmts[0]).align().is_some());
    assert_eq!(decl(&stmts[1]).flags()[0].name.as_str(), "no_reset");
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
fn import_argument_lists_retain_the_program_boundary() {
    let stmts = parse(
        "#import \"M\"(a=1)(b=2); #import \"M\"()(b=3); #import \"M\"(a=4); #import \"M\"()();",
    );
    for (stmt, expected) in stmts.iter().zip([Some(1), Some(0), None, Some(0)]) {
        let StmtKind::Import(import) = &stmt.kind else {
            panic!("expected import");
        };
        assert_eq!(import.program_params_start, expected);
    }
}

#[test]
fn file_level_directives() {
    let stmts = parse(
        "#scope_file\n#scope_export\n#scope_module\n#add_context ctx: int = 1;\n#module_parameters (A := 1) (B := 2);\n#placeholder X;\n#assert A == 1 \"msg\";\n#run main();\n#insert \"x :: 1;\";\n#poke_name Basic operator==;\n#library \"x\";",
    );
    assert!(matches!(stmts[0].kind, StmtKind::Scope(ScopeKind::File)));
    assert!(matches!(stmts[3].kind, StmtKind::AddContext(_)));
    assert!(
        matches!(&stmts[4].kind, StmtKind::ModuleParameters(m) if m.params.len() == 1 && m.runtime_params.len() == 1)
    );
    assert!(matches!(stmts[5].kind, StmtKind::Placeholder(_)));
    assert!(matches!(
        &stmts[6].kind,
        StmtKind::Assert(a) if a.message.is_some()
    ));
    assert!(matches!(stmts[7].kind, StmtKind::Run(_)));
    assert!(matches!(stmts[8].kind, StmtKind::Insert(_)));
    assert!(matches!(&stmts[9].kind, StmtKind::Directive(d) if d.args.len() == 2));
    assert!(matches!(stmts[10].kind, StmtKind::Directive(_)));
}

#[test]
fn assert_takes_directive_flags() {
    let stmts = parse("#assert,stallable(A == 1);\n#assert,stallable A == 1 \"msg\";");
    assert!(matches!(&stmts[0].kind, StmtKind::Assert(a) if a.message.is_none()));
    assert!(matches!(&stmts[1].kind, StmtKind::Assert(a) if a.message.is_some()));
}

#[test]
fn static_if_at_file_scope() {
    let stmts = parse(
        "#if A { x :: 1; y :: 2; } else #if B z :: 3; else { w :: 4; }\n#if C print(\"hi\");\n#if OS == { case .A; #load \"a.jai\"; case; #load \"b.jai\"; }\n#if #complete N == { case 1; a :: 1; case 2; a :: 2; }",
    );
    let StmtKind::StaticIf(first) = &stmts[0].kind else {
        panic!()
    };
    let (then_branch, else_branch) = (&first.then_branch, &first.else_branch);
    assert_eq!(then_branch.len(), 2);
    assert!(
        matches!(&else_branch[0].kind, StmtKind::StaticIf(i) if i.then_branch.len() == 1 && i.else_branch.len() == 1)
    );
    assert!(matches!(&stmts[1].kind, StmtKind::StaticIf(i) if i.then_branch.len() == 1));
    assert!(matches!(&stmts[2].kind, StmtKind::StaticSwitch(s) if s.cases.len() == 2));
    // `#if #complete x == {}` (KodaJai) parses as a plain static switch.
    assert!(matches!(&stmts[3].kind, StmtKind::StaticSwitch(s) if s.cases.len() == 2));
}

// -- statements -----------------------------------------------------------------

#[test]
fn switch_statements() {
    let stmts = body_of("if x == {\n case 1; a();\n case 2, 3; b(); #through;\n case; c();\n}");
    let StmtKind::Switch(switch) = &stmts[0].kind else {
        panic!()
    };
    let (cases, complete) = (&switch.cases, switch.complete);
    assert!(!complete);
    assert_eq!(cases.len(), 3);
    assert_eq!(cases[1].values.len(), 2);
    assert!(cases[1].through && !cases[0].through);
    assert!(cases[2].values.is_empty());
    let stmts =
        body_of("if #complete x == { case .A; a(); }\nif x == #complete { case .A: { a(); } }");
    assert!(matches!(
        &stmts[0].kind,
        StmtKind::Switch(s) if s.complete
    ));
    assert!(matches!(
        &stmts[1].kind,
        StmtKind::Switch(s) if s.complete
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
fn for_expansion_takes_a_name_path() {
    let stmts = body_of("for: tree.Walk root.range {}\nfor: Mod.Type.Walk xs {}\nfor: Walk xs {}");
    let paths: Vec<Option<String>> = stmts
        .iter()
        .map(|s| match &s.kind {
            StmtKind::For(f) => f.iterator.as_ref().and_then(crate::ast::Expr::path_string),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        paths,
        [
            Some("tree.Walk".to_string()),
            Some("Mod.Type.Walk".to_string()),
            Some("Walk".to_string())
        ]
    );
    let StmtKind::For(f) = &stmts[0].kind else {
        unreachable!()
    };
    assert!(
        matches!(&f.over, ForOver::Collection(e) if e.path_string().as_deref() == Some("root.range"))
    );
}

#[test]
fn for_loop_forms() {
    let stmts = body_of(
        "for x, i: arr {}\nfor arr {}\nfor 0..n-1 {}\nfor < arr {}\nfor * p: arr {}\nfor :it_iter v: arr {}\nfor #v2 < a..b {}\nfor i: 1..10 print(i);\nfor #char \"a\"..#char \"e\" {}\nfor #v2 #run f() {}",
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
    // A directive that starts an expression is the range or collection, not a flag.
    assert!(fors[8].flags.is_empty() && matches!(fors[8].over, ForOver::Range(..)));
    assert!(fors[9].flags.len() == 1 && matches!(fors[9].over, ForOver::Collection(_)));
}

#[test]
fn while_break_continue_remove() {
    let stmts = body_of(
        "while outer := i < 3 { break outer; }\nwhile x continue;\nfor a { remove; }\nfor a { remove it; }",
    );
    assert!(matches!(
        &stmts[0].kind,
        StmtKind::While(w) if w.label.is_some()
    ));
    assert!(
        matches!(&stmts[1].kind, StmtKind::While(w) if w.label.is_none() && matches!(w.body.kind, StmtKind::Continue(None)))
    );
}

#[test]
fn while_condition_backtick_is_retained_without_affecting_other_labels() {
    let stmts = body_of(
        "while `value := next() { break value; } while private := next() {} while :named true {}",
    );
    for (stmt, expected) in stmts
        .iter()
        .zip([(true, true), (true, false), (false, false)])
    {
        let StmtKind::While(w) = &stmt.kind else {
            panic!("while statement")
        };
        assert_eq!((w.bind_label, w.backtick_label), expected);
        assert!(w.label.is_some());
    }
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
        StmtKind::Using(u) if matches!(u.filter, UsingFilter::None)
    ));
    assert!(
        matches!(&stmts[6].kind, StmtKind::Using(u) if matches!(&u.filter, UsingFilter::Only(names) if names.len() == 2))
    );
    assert!(matches!(
        &stmts[7].kind,
        StmtKind::Using(u) if matches!(u.filter, UsingFilter::Except(_))
    ));
    assert!(matches!(&stmts[8].kind, StmtKind::PushContext(_)));
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
