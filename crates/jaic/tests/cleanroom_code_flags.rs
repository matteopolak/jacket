//! Syntax export flags are the reviewed public numbers, not private AST discriminants.
use jaic::{
    ast::{CodeBody, ExprKind, StmtKind},
    parser,
    records::{FieldRef, Item, Records},
    sema::code_export::export_code,
    source::FileId,
};

fn child(records: &Records, id: i64, field: &str) -> i64 {
    match records.field(id, field) {
        Some(FieldRef::Ref(id)) => id,
        _ => panic!("missing {field}"),
    }
}

fn list(records: &Records, id: i64, field: &str) -> Vec<i64> {
    match records.field(id, field) {
        Some(FieldRef::List(items)) => items
            .iter()
            .map(|item| match item {
                Item::Ref(id) => id,
                _ => panic!("expected node"),
            })
            .collect(),
        _ => panic!("missing {field}"),
    }
}

fn flags(records: &Records, id: i64) -> i64 {
    match records.field(id, "inst_flags") {
        Some(FieldRef::Int(value)) => value,
        None => 0,
        _ => panic!("invalid flags"),
    }
}

#[test]
fn type_instantiation_flags_preserve_array_directive_and_variadic_kinds() {
    let source = "main :: () { grow:[..]u32; fixed:[3]u32; view:[]u32; plain:#type u32; distinct:#type,distinct u32; isa:#type,isa u32; callback:(u32)->u32; consume::(items:..[..]u32){} }";
    let parsed = parser::parse_file(FileId(0), source).unwrap();
    let StmtKind::Decl(decl) = &parsed.stmts[0].kind else {
        panic!("declaration")
    };
    let ExprKind::Proc(proc) = &decl.value.as_ref().unwrap().kind else {
        panic!("procedure")
    };
    let mut records = Records::default();
    let (root, _) = export_code(
        &mut records,
        &CodeBody::Block(proc.body.clone().unwrap()),
        "{",
    );
    let declarations = list(&records, root, "statements");
    for (&decl, expected) in declarations.iter().zip([2, 0, 64, 8, 16, 32, 4]) {
        assert_eq!(
            flags(&records, child(&records, decl, "type_inst")),
            expected
        );
    }
    let header = child(&records, declarations[7], "expression");
    let argument = list(&records, header, "arguments")[0];
    assert_eq!(
        flags(&records, child(&records, argument, "type_inst")),
        3,
        "VARARGS must retain RESIZABLE"
    );
}

#[test]
fn static_using_arguments_are_independent_of_the_public_filter_kind() {
    let source =
        "main :: () { using,only(x,y) point; using,except() point; using,map(x=z) point; }";
    let parsed = parser::parse_file(FileId(0), source).unwrap();
    let StmtKind::Decl(decl) = &parsed.stmts[0].kind else {
        panic!("declaration")
    };
    let ExprKind::Proc(proc) = &decl.value.as_ref().unwrap().kind else {
        panic!("procedure")
    };
    let mut records = Records::default();
    let (root, _) = export_code(
        &mut records,
        &CodeBody::Block(proc.body.clone().unwrap()),
        "{",
    );
    for (&id, (kind, arguments)) in
        list(&records, root, "statements")
            .iter()
            .zip([(1, "x, y"), (2, ""), (3, "x=z")])
    {
        assert!(matches!(records.field(id,"filter_type"),Some(FieldRef::Int(n)) if n==kind));
        assert!(matches!(
            records.field(id, "filter_expression"),
            None | Some(FieldRef::Ref(0))
        ));
        assert!(
            matches!(records.field(id,"__syntax_filter_arguments"),Some(FieldRef::Str(s)) if &**s==arguments.as_bytes())
        );
    }
}

#[test]
fn exported_while_syntax_retains_caller_binding_and_label_modes() {
    let source =
        "main :: () { while `value := next() {} while private := next() {} while :label true {} }";
    let parsed = parser::parse_file(FileId(0), source).unwrap();
    let StmtKind::Decl(decl) = &parsed.stmts[0].kind else {
        panic!("declaration")
    };
    let ExprKind::Proc(proc) = &decl.value.as_ref().unwrap().kind else {
        panic!("procedure")
    };
    let mut records = Records::default();
    let (root, _) = export_code(
        &mut records,
        &CodeBody::Block(proc.body.clone().unwrap()),
        "{",
    );
    for (&id, (bind, tick)) in
        list(&records, root, "statements")
            .iter()
            .zip([(1, 1), (1, 0), (0, 0)])
    {
        assert!(
            matches!(records.field(id, "__syntax_bind_label"), Some(FieldRef::Int(n)) if n==bind)
        );
        assert!(
            matches!(records.field(id, "__syntax_backtick_label"), Some(FieldRef::Int(n)) if n==tick)
        );
    }
}
