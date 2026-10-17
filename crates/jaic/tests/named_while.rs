//! General macro condition binding; no library dependencies.
mod common;
use common::compile_source as compile;

#[path = "support/named_while_source.rs"]
mod fixture;


#[test]
fn inserted_condition_binding_preserves_value_identity_and_labelled_control() {
    let (c, result) = compile(&format!("{}\n#assert(check()==0);", fixture::SOURCE));
    if let Err(error) = result {
        panic!("{}", error.render(&c.sources));
    }
}

#[test]
fn private_macro_condition_does_not_leak_to_inserted_caller_code() {
    let (c, result) = compile(
        r#"
        run :: (body:Code) #expand { while hidden := cast(u32)1 { #insert body; break; } }
        main :: () { run(#code { value:=hidden; }); }
    "#,
    );
    let error = result.expect_err("private macro binding must stay private");
    assert!(
        error.message.contains("unknown identifier `hidden`"),
        "{}",
        error.render(&c.sources)
    );
}

#[test]
fn backtick_condition_without_expansion_remains_loop_local() {
    let (c, result) = compile(
        r#"
        main :: () { while `local := cast(u32)1 { break local; } value:=local; }
    "#,
    );
    let error = result.expect_err("ordinary loop binding must not escape");
    assert!(
        error.message.contains("unknown identifier `local`"),
        "{}",
        error.render(&c.sources)
    );
}

#[test]
fn private_macro_condition_preserves_an_outer_caller_binding() {
    let (c, result) = compile(
        r#"
        run :: (body:Code) #expand { while hidden := cast(u32)1 { #insert body; break hidden; } }
        check :: () -> int { hidden:=9; result:int; run(#code { result=hidden; }); return result; }
        #assert(check()==9);
    "#,
    );
    if let Err(error) = result {
        panic!("{}", error.render(&c.sources));
    }
}

#[test]
fn invalid_loop_targets_still_diagnose() {
    let (c, result) = compile(
        r#"
        run :: (body:Code) #expand { while `value := cast(u32)1 { #insert body; } }
        main :: () { run(#code { continue unrelated; }); }
    "#,
    );
    let error = result.expect_err("only an enclosing loop is a control target");
    assert!(
        error
            .message
            .contains("no enclosing loop named `unrelated`"),
        "{}",
        error.render(&c.sources)
    );
}
