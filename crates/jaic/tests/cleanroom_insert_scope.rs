//! Human-authored Code insertion reduction and independent lexical-scope controls.
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource, VirtualFs};
use std::{path::PathBuf, rc::Rc};

fn compile(source: &str) -> Result<(), String> {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    compiler
        .compile_sources(&[ProgramSource::String(source.into())])
        .map_err(|error| error.render(&compiler.sources))
}

#[test]
fn exact_human_declaration_insertion_is_visible_to_the_following_statement() {
    let root = std::env::var_os("JAIC_STDLIB")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"));
    let mut options = Options::host();
    options.import_paths = vec![root.clone()];
    options.preload = Some(root.join("prelude/module.jai"));
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    let result = compiler.compile_sources(&[ProgramSource::String(
        include_str!("../../../tests/producer-requests/human-insert-declaration-scope/main.jai")
            .into(),
    )]);
    result
        .map_err(|error| error.render(&compiler.sources))
        .unwrap();
}

#[test]
fn unbraced_declarations_publish_but_explicit_blocks_do_not() {
    compile(
        "verify::()->int {code::#code value:=2; #insert code; return value;} #assert verify()==2;",
    )
    .unwrap();
    for body in [
        "a::#code {b:=2;}; #insert a; x:=b;",
        "a::#code {{b:=2;}}; #insert a; x:=b;",
        "a::#code N::2; x:=N; #insert a;",
    ] {
        let error =
            compile(&format!("main::(){{{body}}}")).expect_err("official lexical scope negative");
        assert!(error.contains("unknown identifier"), "{body}: {error}");
    }
}

#[test]
fn explicit_blocks_keep_locals_and_defers_inside_the_block() {
    compile(
        r#"
        verify::()->int {
            result:=0;
            code::#code {local:=2; defer result+=local; {local:=9;} result=1;}
            #insert code;
            return result;
        }
        #assert verify()==3;
    "#,
    )
    .unwrap();
    let error = compile("main::(){code::#code {local:=2;} #insert code; out:=local;}")
        .expect_err("explicit Code block locals stay private");
    assert!(error.contains("unknown identifier"), "{error}");
}

#[test]
fn capture_and_explicit_scope_override_remain_distinct() {
    for (modifier, expected) in [("", 7), (",scope()", 11)] {
        compile(&format!(
            r#"
            verify :: ()->int {{
                source:=7;
                code :: #code value:=source;
                {{
                    source:=11;
                    #insert{modifier} code;
                    return value;
                }}
            }}
            #assert verify()=={expected};
        "#
        ))
        .unwrap();
    }
}

#[test]
fn duplicate_inserted_and_receiving_declarations_are_rejected() {
    for body in [
        "value:=1; code::#code value:=2; #insert code;",
        "code::#code value:=2; #insert code; value:=3;",
        "code::#code value:=2; #insert code; #insert code;",
        "value::1; code::#code value::2; #insert code;",
        "code::#code value::2; #insert code; value::3;",
        "code::#code value::2; #insert code; #insert code;",
        "source:=1; code::#code value:=2; {value:=3; #insert code;}",
    ] {
        let error = compile(&format!("main :: () {{ {body} }}"))
            .expect_err("same-block declaration insertion must not silently shadow");
        assert!(error.contains("already declared"), "{body}: {error}");
    }
}

#[test]
fn forward_constants_overloads_and_aliases_survive_insertion() {
    compile(
        r#"
        verify :: ()->int {
            result:=0;
            code :: #code {
                value:=twice(3)+twice(true);
                twice :: (value:int)->int {return value*2;}
                twice :: (value:bool)->int {return ifx value then 4 else 0;}
                Record :: struct {value:int;}
                Alias :: Record;
                record:Alias=.{value=2};
                #assert Alias==Record;
                #assert type_of(record)==Record;
                result=value+record.value+twice(2);
            }
            #insert code;
            return result;
        }
        #assert verify()==16;
    "#,
    )
    .unwrap();
}

#[test]
fn inserted_using_declaration_exposes_only_its_receiving_block() {
    compile(
        r#"
        Record :: struct {value:int;}
        verify :: ()->int {
            code :: #code using record:=Record.{value=7};
            #insert code;
            return value;
        }
        #assert verify()==7;
    "#,
    )
    .unwrap();
    let error = compile(
        r#"
        Record :: struct {value:int;}
        main :: () {
            code :: #code { {using record:=Record.{value=7};} }
            #insert code;
            out:=value;
        }
    "#,
    )
    .expect_err("nested using must not leak");
    assert!(error.contains("unknown identifier"), "{error}");
}

#[test]
fn inserted_constants_remain_immutable() {
    let error = compile("main :: () { code::#code value::2; #insert code; value=3; }")
        .expect_err("insertion does not change :: storage semantics");
    assert!(
        error.contains("constant") || error.contains("assign"),
        "{error}"
    );
}

#[test]
fn inserting_inside_a_nested_receiving_block_does_not_leak_out() {
    let error = compile(
        r#"
        main :: () {
            code::#code value:=2;
            {#insert code; value+=1;}
            out:=value;
        }
    "#,
    )
    .expect_err("the receiving block still bounds visibility");
    assert!(error.contains("unknown identifier"), "{error}");
}

#[test]
fn a_use_before_an_inserted_mutable_declaration_is_not_a_forward_local() {
    let error = compile("main :: () { code::#code value:=2; out:=value; #insert code; }")
        .expect_err("ordinary runtime locals are not forward declarations");
    assert!(error.contains("unknown identifier"), "{error}");
}

#[test]
fn insertion_does_not_create_a_new_overload_declaration_scope() {
    compile(
        r#"
        verify :: ()->int {
            select :: (value:int)->int {return value+1;}
            code::#code select :: (value:bool)->int {return ifx value then 7 else 0;}
            #insert code;
            return select(2)+select(true);
        }
        #assert verify()==10;
    "#,
    )
    .unwrap();
    let error = compile(
        r#"
        main :: () {
            select :: (value:int)->int {return value+1;}
            code::#code select :: (value:int)->int {return value+2;}
            #insert code;
        }
    "#,
    )
    .expect_err("a synthetic insertion lookup scope must not hide duplicate signatures");
    assert!(error.contains("identical parameter types"), "{error}");
}

#[test]
fn forward_code_value_and_sibling_receiving_blocks_are_supported() {
    compile(
        r#"
        verify :: ()->int {
            #insert code;
            code::#code value:=2;
            reusable::#code item:=3;
            sum:=value;
            {#insert reusable; sum+=item;}
            {#insert reusable; sum+=item;}
            return sum;
        }
        #assert verify()==8;
    "#,
    )
    .unwrap();
}

#[test]
fn captured_private_module_names_do_not_escape_with_new_bindings() {
    let module = r#"
        #scope_file
        private_value :: 7;
        #scope_export
        declaration :: #code generated:=private_value;
    "#;
    for tail in ["return generated;", "return generated+private_value;"] {
        let source = format!(
            r#"
            M :: #import "Maker";
            verify :: ()->int {{ #insert M.declaration; {tail} }}
            #assert verify()==7;
        "#
        );
        let mut fs = VirtualFs::default();
        fs.insert("/lib/Maker/module.jai", module.as_bytes());
        fs.insert("/main.jai", source.as_bytes());
        let mut options = Options::host();
        options.runtime_support = false;
        options.import_paths = vec!["/lib".into()];
        let mut compiler = Compiler::new(options, Rc::new(fs));
        let result = compiler.compile_program(std::path::Path::new("/main.jai"));
        if tail == "return generated;" {
            result
                .map_err(|error| error.render(&compiler.sources))
                .unwrap();
        } else {
            let error = result.expect_err("captured module privacy is not a namespace reexport");
            assert!(
                error.message.contains("unknown identifier"),
                "{}",
                error.render(&compiler.sources)
            );
        }
    }
}

#[test]
fn callable_aliases_keep_their_selected_header_after_publication() {
    compile(
        r#"
        verify :: ()->int {
            operation :: (amount:int=3)->int {return amount*2;}
            code::#code callback:=operation;
            alias_code::#code alias::operation;
            #insert code;
            #insert alias_code;
            return callback()+callback(amount=5)+alias(amount=2);
        }
        #assert verify()==20;
    "#,
    )
    .unwrap();
}

#[test]
fn macro_private_declarations_do_not_leak_and_backtick_bindings_still_work() {
    compile(
        r#"
        introduce :: () #expand { `value:=7; }
        verify :: ()->int {
            code::#code introduce();
            #insert code;
            return value;
        }
        #assert verify()==7;
    "#,
    )
    .unwrap();
    let error = compile(
        r#"
        hidden :: () #expand {private_value:=7;}
        main :: () {
            code::#code hidden();
            #insert code;
            out:=private_value;
        }
    "#,
    )
    .expect_err("macro body privacy survives insertion");
    assert!(error.contains("unknown identifier"), "{error}");
}

#[test]
fn inserted_local_storage_and_defers_live_until_the_receiving_block_exits() {
    compile(
        r#"
        verify :: ()->int {
            result:=1;
            {
                code::#code local:=2;
                cleanup::#code defer result=local;
                #insert code;
                #insert cleanup;
                local=3;
                if result!=1 return 0;
            }
            return result;
        }
        #assert verify()==3;
    "#,
    )
    .unwrap();
}

#[test]
fn code_export_and_text_parse_preserve_payload_shape_without_source_heuristics() {
    use jaic::{
        ast::CodeBody, build::parse_code_text, records::Records, sema::code_export::export_code,
        source::FileId,
    };
    for (text, statement) in [
        ("value:=2", true),
        ("{value:=2;}", false),
        ("{{value:=2;}}", false),
    ] {
        let body = parse_code_text(FileId(0), text).unwrap();
        assert_eq!(matches!(&*body, CodeBody::Stmt(_)), statement);
        let mut records = Records::default();
        // An unavailable/replaced source spelling cannot change AST lexical shape.
        let (root, _) = export_code(&mut records, &body, "");
        assert_eq!(
            records.get(root).unwrap().tag,
            if statement {
                "Code_Declaration"
            } else {
                "Code_Block"
            }
        );
    }
}
