//! Record self annotations retain their actual enclosing nominal identity.
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "jai-record-self-types-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("main.jai"), source).unwrap();
        Self(root)
    }
    fn program(&self) -> Result<jai_ir::Program, String> {
        let graph = jai_modules::ModuleGraph::load(&self.0.join("main.jai"), Default::default())
            .map_err(|error| error.to_string())?;
        jai_sema::resolve_graph_with_options(
            &graph,
            &jai_sema::ResolveOptions {
                layout: Some(jai_types::LayoutPolicy::lp64()),
                ..Default::default()
            },
            &mut jai_vm::NoEffects,
        )
        .map_err(|error| error.to_string())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn check(source: &str) {
    let program = Fixture::new(source).program().unwrap();
    let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
    let jai_vm::Outcome::Complete(values) = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
}

#[test]
fn ordinary_self_pointer_uses_the_actual_record() {
    check(
        "Node::struct{next:*#this;value:int;} main::()->int{a:Node;b:Node;a.next=*b;a.next.value=42;return b.value;}",
    );
}

#[test]
fn generic_self_pointer_uses_the_actual_instantiation() {
    check(
        "Node::struct(T:Type){next:*#this;value:T;} main::()->int{a:Node(int);b:Node(int);a.next=*b;a.next.value=42;return b.value;}",
    );
}

#[test]
fn procedure_types_can_reference_separately_defined_self_records() {
    check(
        "Node::struct(T:Type){next:*#this;value:T;} Callback::(node:*Node(int))->int; read::(node:*Node(int))->int{return node.value;} main::()->int{node:Node(int);node.value=42;callback:Callback=read;return callback(*node);}",
    );
}

#[test]
fn local_self_pointer_retains_local_identity() {
    check(
        "main::()->int{Node::struct{next:*#this;value:int;} a:Node;b:Node;a.next=*b;a.next.value=42;return b.value;}",
    );
}

#[test]
fn anonymous_nested_self_type_is_the_inner_record() {
    check(
        "Outer::struct{inner:struct{next:*#this;value:int;};} main::()->int{a:Outer;b:Outer;a.inner.next=*b.inner;a.inner.next.value=42;return b.inner.value;}",
    );
}

#[test]
fn self_pointer_does_not_cross_generic_instantiations() {
    let error=Fixture::new("Node::struct(T:Type){next:*#this;value:T;} main::()->int{a:Node(int);b:Node(u8);a.next=*b;return 0;}").program().unwrap_err();
    assert!(error.contains("type"), "{error}");
}

#[test]
fn self_type_requires_an_enclosing_record() {
    let error = Fixture::new("bad:*#this;main::()->int{return 0;}")
        .program()
        .unwrap_err();
    assert!(error.contains("enclosing record"), "{error}");
}

#[test]
fn by_value_self_cycles_are_rejected() {
    let error = Fixture::new("Node::struct{next:#this;} main::()->int{return 0;}")
        .program()
        .unwrap_err();
    assert!(error.contains("type contains itself by value"), "{error}");
}

#[test]
fn procedure_type_headers_do_not_inherit_record_self() {
    let error = Fixture::new("Node::struct{callback:(next:*#this)->int;} main::()->int{return 0;}")
        .program()
        .unwrap_err();
    assert!(error.contains("procedure headers"), "{error}");
}
