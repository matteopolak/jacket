use super::*;

#[test]
fn v2_policy_and_implicit_remove_retain_actual_loop_targets() {
    let module = parse("main::(){ for #v2 < i: 1..3 { continue i; } }").unwrap();
    let StatementKind::Range(range) = &module.procedures()[0].body[0].kind else {
        panic!()
    };
    assert_eq!(range.policy, IterationPolicy::Version2);
    assert_eq!(range.direction, Direction::Reverse);
    for source in [
        "main::(){for #v2 #v2 1..3 {}}",
        "main::(){for < #v2 1..3 {}}",
    ] {
        assert!(parse(source).is_err());
    }
    let mut sources = jai_source::SourceMap::default();
    let id = sources.insert(
        "own-array-policy.jai".into(),
        "main::(){ for #v2 item: values { remove; } }".into(),
    );
    let file = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
    let FileItem::Declaration(FileDeclaration {
        kind: FileDeclarationKind::Procedure(procedure),
        ..
    }) = &file.items()[0]
    else {
        panic!()
    };
    let StatementKind::ArrayLoop(array) = &procedure.body[0].kind else {
        panic!()
    };
    assert_eq!(array.policy, IterationPolicy::Version2);
    assert!(matches!(
        array.body[0].kind,
        StatementKind::Jump {
            kind: JumpKind::Remove,
            target: LoopTarget::Innermost,
            ..
        }
    ));
}

#[test]
fn runtime_default_retains_middle_position_and_fallthrough() {
    let module =
        parse("main::(){ if 2 == { case 1; #through; case; #through; case 2; return; } }").unwrap();
    let StatementKind::Cases(cases) = &module.procedures()[0].body[0].kind else {
        panic!()
    };
    assert_eq!(cases.default_position, Some(1));
    assert!(cases.default_through);
    assert_eq!(cases.arms.len(), 2);
    assert!(parse("main::(){if 1 == {case; case;}}").is_err());
}

#[test]
fn compile_time_fallthrough_crosses_default_in_both_directions() {
    let mut sources = jai_source::SourceMap::default();
    let id = sources.insert(
        "own-ordered-source-cases.jai".into(),
        "#if 1 == {case 1; A::1; #through; case; B::2; #through; case 2; C::3;}".into(),
    );
    let file = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
    let FileItem::CompileTimeCases {
        cases, ..
    } = &file.items()[0]
    else {
        panic!()
    };
    assert_eq!(cases.default_position, Some(1));
    assert_eq!(
        cases
            .selected_body_refs(CompileTimeCaseChoice::Arm(0))
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        cases
            .selected_body_refs(CompileTimeCaseChoice::Default)
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        cases
            .selected_body_refs(CompileTimeCaseChoice::Arm(1))
            .unwrap()
            .len(),
        1
    );
}
