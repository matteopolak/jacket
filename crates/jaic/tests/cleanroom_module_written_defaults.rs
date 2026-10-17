//! Independent ordinary-module argument identity controls, without an SDK.
use jaic::sema::{Compiler, Options, VirtualFs};
use std::path::Path;
use std::rc::Rc;

const LEAF: &str =
    include_str!("../../../tests/fixtures/module-written-defaults/modules/Leaf/module.jai");

const BRIDGE: &str =
    include_str!("../../../tests/fixtures/module-written-defaults/modules/Bridge/module.jai");

fn compiler(main: &str) -> Compiler {
    let mut fs = VirtualFs::default();
    fs.insert("/lib/Leaf/module.jai", LEAF.as_bytes());
    fs.insert("/lib/Bridge/module.jai", BRIDGE.as_bytes());
    fs.insert("/app/main.jai", main.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    options.import_paths = vec!["/lib".into()];
    Compiler::new(options, Rc::new(fs))
}

#[test]
fn written_defaults_forwarding_and_storage_follow_instance_identity() {
    let mut c = compiler(include_str!(
        "../../../tests/fixtures/module-written-defaults/main.jai"
    ));
    if let Err(error) = c.compile_program(Path::new("/app/main.jai")) {
        panic!("{}", error.render(&c.sources));
    }
}

#[test]
fn omitted_default_equivalence_assertion_remains_an_error() {
    let mut c = compiler(
        r#"// Diagnostic control for the original omitted-default equivalence hypothesis.
Omitted :: #import "Leaf"(FLAG = true);
Bridge :: #import "Bridge";
#assert type_of(Bridge.native) == *Omitted.Token;
main :: () {}
"#,
    );
    let error = c.compile_program(Path::new("/app/main.jai")).unwrap_err();
    assert!(error.message.contains("#assert failed"), "{error:?}");
}

#[test]
fn nominal_argument_mismatch_remains_an_error() {
    let mut c = compiler(
        r#"Omitted :: #import "Leaf"(FLAG = true);
Explicit :: #import "Leaf"(FLAG = true, SECOND = true);
accept :: (value: *Explicit.Token) #no_context {}
main :: () { accept(*Omitted.token); }
"#,
    );
    let error = c.compile_program(Path::new("/app/main.jai")).unwrap_err();
    assert!(
        error.message.contains("does not match parameter"),
        "{error:?}"
    );
}
