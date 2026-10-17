//! Public Code import calls retain both syntax groups, including empty groups.
use jaic::{
    ast::{CodeBody, ExprKind, StmtKind},
    parser,
    records::{FieldRef, Records},
    sema::code_export::export_code,
    source::FileId,
};

#[test]
fn exported_calls_preserve_presence_and_separate_argument_counts() {
    for (source, expected) in [
        ("#import \"M\";", [None, None]),
        ("#import \"M\"();", [Some(0), None]),
        ("#import \"M\"()();", [Some(0), Some(0)]),
        ("#import \"M\"()(enabled=false);", [Some(0), Some(1)]),
        ("#import \"M\"(local=9)();", [Some(1), Some(0)]),
        ("#import \"M\"(local=9)(enabled=false);", [Some(1), Some(1)]),
    ] {
        let text = format!("probe::() {{ {source} }}");
        let parsed = parser::parse_file(FileId(0), &text).unwrap();
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
        let Some(FieldRef::List(stmts)) = records.field(root, "statements") else {
            panic!("statements")
        };
        let jaic::records::Item::Ref(import) = stmts.iter().next().unwrap() else {
            panic!("import")
        };
        for (field, expected) in ["module_parameters_call", "program_parameters_call"]
            .into_iter()
            .zip(expected)
        {
            let actual = match records.field(import, field) {
                None => None,
                Some(FieldRef::Ref(id)) => match records.field(id, "arguments_unsorted") {
                    Some(FieldRef::List(args)) => Some(args.len()),
                    _ => panic!("arguments"),
                },
                _ => panic!("call"),
            };
            assert_eq!(actual, expected, "{source}: {field}");
        }
    }
}
