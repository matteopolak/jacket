//! Original register operand forwarding and scalar-result regressions.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

#[path = "support/register_source.rs"]
mod source;
use source::SOURCE;

#[test]
fn registers_forward_through_expansion_and_extract_scalar_lanes() {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    if let Err(e) = c.compile_sources(&[ProgramSource::String(format!(
        "{SOURCE}\n#assert(check()==0);"
    ))]) {
        panic!("{}", e.render(&c.sources));
    }
}

#[test]
fn bootstrap_register_identity_and_scalar_misuse_are_checked() {
    let mut fs = VirtualFs::default();
    fs.insert(
        "/lib/prelude/module.jai",
        "__reg :: #type,distinct u16;".as_bytes(),
    );
    let fs = Rc::new(fs);
    let mut options = Options::host();
    options.runtime_support = false;
    options.preload = Some("/lib/prelude/module.jai".into());
    let mut c = Compiler::new(options.clone(), fs.clone());
    if let Err(e) = c.compile_sources(&[ProgramSource::String(format!(
        "{SOURCE}\n#assert(check()==0);"
    ))]) {
        panic!("{}", e.render(&c.sources));
    }
    let ty = c.preload_type("__reg", jaic::source::Span::NONE).unwrap();
    assert_eq!(c.types.name(ty), "__reg");
    for bad in [
        "use :: (v:__reg) #expand {} main :: () { n:u16; use(n); }",
        "use :: (v:__reg) #expand {} main :: () { n:[16]u8; use(n); }",
        "use :: (v:__reg) #expand {} main :: () { #asm { v:gpr; } use(v); }",
        "main :: () { #asm { v:vec; } n:=v; }",
        "main :: () { #asm { v:vec; add v,1; } }",
        "main :: () { value:__reg; }",
        "main :: () { value:[2]__reg; }",
        "Record :: struct { value:__reg; } main :: () { value:Record; }",
        "value:__reg; main :: () { value=cast(__reg)0; }",
        "use :: (v:__reg) {} main :: () { #asm { v:vec; } use(v); }",
    ] {
        let mut c = Compiler::new(options.clone(), fs.clone());
        let error = c
            .compile_sources(&[ProgramSource::String(bad.into())])
            .expect_err("register misuse must be rejected");
        assert!(
            error.message.contains("register"),
            "{}",
            error.render(&c.sources)
        );
    }
}
