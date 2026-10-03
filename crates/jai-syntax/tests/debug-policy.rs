use jai_source::{SourceMap, Symbols};
use jai_syntax::{FileDeclarationKind, FileItem, parse_file};
use jai_types::DebugPolicy;

#[test]
fn source_bodies_retain_debug_policy_without_changing_expand_or_abi() {
    for (suffix, expanded, debug) in [
        ("#expand #no_debug", true, DebugPolicy::Suppress),
        ("#no_debug #expand", true, DebugPolicy::Suppress),
        ("#no_debug", false, DebugPolicy::Suppress),
        ("", false, DebugPolicy::Emit),
    ] {
        let text = format!("helper :: (value:int) {suffix} {{}}");
        let mut sources = SourceMap::default();
        let source = sources.insert("debug-policy.jai".into(), text);
        let file = parse_file(sources.get(source).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(declaration) = &file.items()[0] else {
            panic!("expected source declaration");
        };
        let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
            panic!("expected source procedure");
        };
        assert_eq!(procedure.expands, expanded);
        assert_eq!(procedure.debug, debug);
        assert_eq!(procedure.convention, jai_types::CallingConvention::Jai);
    }
}

#[test]
fn duplicate_debug_policy_reports_the_repeated_directive() {
    let text = "helper :: () #no_debug #expand #no_debug {}";
    let mut sources = SourceMap::default();
    let source = sources.insert("duplicate-debug-policy.jai".into(), text.into());
    let error = parse_file(sources.get(source).unwrap(), &mut Symbols::default()).unwrap_err();
    assert_eq!(error.location.span.text(text), "#no_debug");
    assert_eq!(
        error.location.span.start as usize,
        text.rfind("#no_debug").unwrap()
    );
    assert!(error.message.contains("duplicate #no_debug"));
}

#[test]
fn standalone_scalar_parser_retains_the_body_policy() {
    let module = jai_syntax::parse("main::()->int #no_debug{return 42;}").unwrap();
    assert_eq!(module.procedures()[0].debug, DebugPolicy::Suppress);
}

#[test]
fn body_policy_cannot_be_added_to_a_prototype_or_callback_type() {
    for text in [
        "external :: () #no_debug #foreign;",
        "Callback :: () #no_debug;",
        "compiler_hook :: () #no_debug #compiler;",
    ] {
        let mut sources = SourceMap::default();
        let source = sources.insert("debug-policy-without-body.jai".into(), text.into());
        let error = parse_file(sources.get(source).unwrap(), &mut Symbols::default()).unwrap_err();
        assert!(error.message.contains("#no_debug"), "{error:?}");
    }
}
