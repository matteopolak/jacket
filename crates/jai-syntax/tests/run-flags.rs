use jai_source::{SourceMap, Symbols};
use jai_syntax::{
    CompileTimeBody, CompileTimeRun, ExpressionKind, FileDeclarationKind, FileItem, RunFlags,
    parse_file,
};

#[test]
fn file_runs_retain_stallable_policy_for_each_body_form() {
    let text = "#run,stallable build(); #run,stallable -> int { return 42; } #run,stallable { configure(); } #run original();";
    let mut sources = SourceMap::default();
    let id = sources.insert("file-run-flags.jai".into(), text.into());
    let parsed = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
    let runs: Vec<_> = parsed
        .items()
        .iter()
        .map(|item| {
            let FileItem::Run(run) = item else {
                panic!("expected a file run")
            };
            run
        })
        .collect();
    assert!(runs[..3].iter().all(|run| run.flags.stallable));
    assert_eq!(runs[3].flags, RunFlags::default());
    assert!(matches!(runs[0].body, CompileTimeBody::Expression(_)));
    assert!(matches!(runs[1].body, CompileTimeBody::Procedure { .. }));
    assert!(matches!(runs[2].body, CompileTimeBody::Block(_)));
    assert_eq!(runs[0].location.span.text(text), "#run,stallable build();");
}

#[test]
fn expression_runs_keep_flags_outside_the_body_and_binary_operator() {
    let text = "VALUE :: #run,stallable build() + 1; DEFAULT :: #run original();";
    let mut sources = SourceMap::default();
    let id = sources.insert("expression-run-flags.jai".into(), text.into());
    let parsed = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
    let FileItem::Declaration(first) = &parsed.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Constant(first) = &first.kind else {
        panic!()
    };
    let ExpressionKind::Binary(_, lhs, _) = &first.initializer.kind else {
        panic!()
    };
    let ExpressionKind::CompileTime(CompileTimeRun {
        flags,
        body,
    }) = &lhs.kind
    else {
        panic!()
    };
    assert!(flags.stallable);
    assert!(matches!(body, CompileTimeBody::Expression(_)));
    assert_eq!(lhs.span.text(text), "#run,stallable build()");
    let FileItem::Declaration(second) = &parsed.items()[1] else {
        panic!()
    };
    let FileDeclarationKind::Constant(second) = &second.kind else {
        panic!()
    };
    let ExpressionKind::CompileTime(run) = &second.initializer.kind else {
        panic!()
    };
    assert_eq!(run.flags, RunFlags::default());
}

#[test]
fn scalar_execution_adapter_rejects_the_resumable_policy_at_its_flag() {
    let text = "VALUE :: #run,stallable build(); main :: () {}";
    let error = jai_syntax::parse(text).unwrap_err();
    assert_eq!(error.span.text(text), "stallable");
    assert_eq!(
        error.message,
        "#run,stallable requires resumable compile-time execution"
    );
}
