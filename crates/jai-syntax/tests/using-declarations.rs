use jai_source::{SourceMap, Symbols};
use jai_syntax::{
    self as syntax, ExpressionKind as E, FileDeclarationKind as D, FileItem as F,
    StatementKind as S, UsingSelection as U,
};

fn parse(source: &str) -> (syntax::ParsedFile, Symbols) {
    let mut sources = SourceMap::default();
    let id = sources.insert("own-using-declaration.jai".into(), source.to_owned());
    let mut symbols = Symbols::default();
    (
        syntax::parse_file(sources.get(id).unwrap(), &mut symbols).unwrap(),
        symbols,
    )
}

#[test]
fn basic_inferred_pointer_declaration_retains_original_initializer_once() {
    let text = "main::(){using buffer := get_current_buffer(builder); using buffer;}";
    let (file, symbols) = parse(text);
    let F::Declaration(main) = &file.items()[0] else {
        panic!()
    };
    let D::Procedure(main) = &main.kind else {
        panic!()
    };
    let [wrapper, standalone] = main.body.as_slice() else {
        panic!()
    };
    let S::UsingDeclaration {
        declaration,
        target_span,
        selection,
    } = &wrapper.kind
    else {
        panic!()
    };
    assert!(matches!(selection, U::All));
    assert_eq!(
        wrapper.span.text(text),
        "using buffer := get_current_buffer(builder);"
    );
    assert_eq!(
        declaration.span.text(text),
        "buffer := get_current_buffer(builder);"
    );
    assert_eq!(target_span.text(text), "buffer");
    let S::Declare(syntax::Declaration::Inferred {
        name, initializer, ..
    }) = &declaration.kind
    else {
        panic!()
    };
    assert_eq!(symbols.name(*name), "buffer");
    assert_eq!(initializer.span.text(text), "get_current_buffer(builder)");
    let target = wrapper.using_declaration_directive().unwrap();
    assert!(matches!(target.target.kind,E::Name(actual) if actual==*name));
    assert_eq!(target.target.span, *target_span);
    assert_eq!(target.span, wrapper.span);
    assert!(matches!(standalone.kind, S::Using(_)));
}

#[test]
fn file_enum_and_global_children_keep_name_spans_and_visibility() {
    let text = "#scope_module using CGWindowLevelKey :: enum s32 { kCGBaseWindowLevelKey :: 0; kCGMinimumWindowLevelKey; } using state:Pair;";
    let (file, symbols) = parse(text);
    let [F::Scope { .. }, wrapper, global] = file.items() else {
        panic!()
    };
    let F::UsingDeclaration {
        declaration,
        target_span,
        location,
        ..
    } = wrapper
    else {
        panic!()
    };
    assert_eq!(declaration.visibility, syntax::Visibility::Module);
    assert_eq!(target_span.text(text), "CGWindowLevelKey");
    assert!(
        declaration
            .location
            .span
            .text(text)
            .starts_with("CGWindowLevelKey :: enum")
    );
    assert!(
        location
            .span
            .text(text)
            .starts_with("using CGWindowLevelKey")
    );
    assert_eq!(
        symbols.name(declaration.declared_name()),
        "CGWindowLevelKey"
    );
    assert!(matches!(declaration.kind, D::Enum(_)));
    let F::UsingDeclaration { declaration, .. } = global else {
        panic!()
    };
    assert_eq!(declaration.visibility, syntax::Visibility::Module);
    assert!(matches!(declaration.kind, D::Global(_)));
    assert_eq!(
        global
            .using_declaration_directive()
            .unwrap()
            .target
            .span
            .text(text),
        "state"
    );
}

#[test]
fn filters_are_retained_for_declaration_and_standalone_forms() {
    let text = "main::(){using,except(skip) value := producer(); using,only .[\"x\"] named:Pair; using,map(rename) Alias::Pair; using,except(skip) value;}";
    let (file, _) = parse(text);
    let F::Declaration(declaration) = &file.items()[0] else {
        panic!()
    };
    let D::Procedure(main) = &declaration.kind else {
        panic!()
    };
    let directives = main
        .body
        .iter()
        .map(|statement| statement.using_declaration_directive())
        .collect::<Vec<_>>();
    assert!(matches!(
        directives[0].as_ref().unwrap().selection,
        U::Except(_)
    ));
    assert!(matches!(
        directives[1].as_ref().unwrap().selection,
        U::Only(_)
    ));
    assert!(matches!(
        directives[2].as_ref().unwrap().selection,
        U::Map(_)
    ));
    assert!(directives[3].is_none());
    assert!(
        matches!(main.body[3].kind,S::Using(ref directive) if matches!(directive.selection,U::Except(_)))
    );
}

#[test]
fn imports_remain_actual_import_nodes_and_conditional_wrappers_do_not_flatten() {
    let (file, _) = parse(
        "using Basic::#import \"Basic\"; #if true using Answer::enum {value::42;} else BAD::Missing;",
    );
    assert!(matches!(file.items()[0], F::Import(_)));
    let F::Conditional {
        then_items,
        else_items,
        ..
    } = &file.items()[1]
    else {
        panic!()
    };
    assert!(matches!(
        then_items.as_slice(),
        [F::UsingDeclaration { .. }]
    ));
    assert!(matches!(else_items.as_slice(), [F::Declaration(_)]));
}

#[test]
fn malformed_declarations_preserve_located_parser_errors() {
    for text in [
        "using value := ;",
        "using value: ;",
        "main::(){using value:=42}",
        "main::(){using,unknown(x) value:=42;}",
    ] {
        let mut sources = SourceMap::default();
        let id = sources.insert("bad-using.jai".into(), text.to_owned());
        let error =
            syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
        assert_eq!(error.location.source, id);
        assert!(error.location.span.end <= text.len());
    }
}

#[test]
#[ignore = "requires original source reference and pinned Focus checkout"]
fn actual_string_builder_and_focus_core_graphics_parse_unchanged() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (relative, name) in [
        (
            "simple_memcpy",
        ),
        (
            "corpus/upstream/focus-editor--focus/modules/Objective_C/CoreGraphics.jai",
            "CGWindowLevelKey",
        ),
    ] {
        let source = std::fs::read_to_string(root.join(relative)).unwrap();
        let (file, symbols) = parse(&source);
        let named = file
            .items()
            .iter()
            .find(|item| match item {
                F::Declaration(declaration) | F::UsingDeclaration { declaration, .. } => {
                    symbols.name(declaration.declared_name()) == name
                }
                _ => false,
            })
            .unwrap();
        if name == "simple_memcpy" {
            let F::Declaration(declaration) = named else {
                panic!()
            };
            let D::Procedure(procedure) = &declaration.kind else {
                panic!()
            };
            let wrapper = procedure
                .body
                .iter()
                .find(|statement| matches!(statement.kind, S::UsingDeclaration { .. }))
                .unwrap();
            let directive = wrapper.using_declaration_directive().unwrap();
            assert_eq!(
                symbols.name(match directive.target.kind {
                    E::Name(name) => name,
                    _ => panic!(),
                }),
                "buffer"
            );
            assert!(matches!(directive.selection, U::All));
            let S::UsingDeclaration { declaration, .. } = &wrapper.kind else {
                panic!()
            };
            let S::Declare(syntax::Declaration::Inferred { initializer, .. }) = &declaration.kind
            else {
                panic!()
            };
            assert_eq!(
                initializer.span.text(&source),
                "get_current_buffer(builder)"
            );
        } else {
            let F::UsingDeclaration { declaration, .. } = named else {
                panic!()
            };
            assert!(matches!(declaration.kind, D::Enum(_)));
            assert_eq!(
                named
                    .using_declaration_directive()
                    .unwrap()
                    .target
                    .span
                    .text(&source),
                name
            );
        }
        println!(
            "{relative}: parsed {} complete original items",
            file.items().len()
        );
    }
}
