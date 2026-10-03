//! Numeric pointer declarations use the selected target without minting VM provenance.
use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_types::{LayoutPolicy, ScalarLayout};
use std::{
    fs,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(std::path::PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-native-pointer-constant-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join("main.jai"), source).unwrap();
        Self(path)
    }
    fn graph(&self) -> ModuleGraph {
        ModuleGraph::load(&self.0.join("main.jai"), GraphOptions::default()).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn pointer32() -> LayoutPolicy {
    LayoutPolicy::new(
        ScalarLayout::new(4, 4),
        [
            ScalarLayout::new(1, 1),
            ScalarLayout::new(2, 2),
            ScalarLayout::new(4, 4),
            ScalarLayout::new(8, 8),
        ],
        [ScalarLayout::new(4, 4), ScalarLayout::new(8, 8)],
        ScalarLayout::new(1, 1),
    )
    .unwrap()
}

#[test]
fn checked_pointer_defaults_use_the_explicit_selected_width() {
    let fixture =
        Fixture::new("address:*void=cast(*void) cast(u64)0x100000000; main::()->int{return 42;}");
    let graph = fixture.graph();
    for layout in [None, Some(LayoutPolicy::lp64())] {
        resolve_graph_with_options(
            &graph,
            &ResolveOptions {
                layout,
                ..ResolveOptions::default()
            },
            &mut jai_vm::NoEffects,
        )
        .expect("unknown target defers normalization; 64-bit target admits the address");
    }
    let error = resolve_graph_with_options(
        &graph,
        &ResolveOptions {
            layout: Some(pointer32()),
            ..ResolveOptions::default()
        },
        &mut jai_vm::NoEffects,
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("out of range for the selected target")
    );
}

#[test]
fn a_selected_width_zero_is_null_without_granting_address_provenance() {
    let fixture = Fixture::new(
        "address:*void=cast,trunc(*void) cast(u64)0x100000000; main::()->int{if address==null return 42;return 1;}",
    );
    let graph = fixture.graph();
    let program = resolve_graph_with_options(
        &graph,
        &ResolveOptions {
            layout: Some(pointer32()),
            ..ResolveOptions::default()
        },
        &mut jai_vm::NoEffects,
    )
    .unwrap();
    let mut vm = jai_vm::Vm::new_with_target(
        &program,
        jai_vm::NoEffects,
        jai_vm::Limits::default(),
        jai_vm::ByteTarget {
            policy: pointer32(),
            endian: jai_vm::Endian::Little,
        },
    )
    .unwrap();
    let entry = match program.entry() {
        jai_ir::EntryPoint::Void(entry) | jai_ir::EntryPoint::Int(entry) => entry,
    };
    let result = vm.execute(entry, vec![]);
    let jai_vm::Outcome::Complete(values) = result.outcome else {
        panic!("selected-width null constant must execute: {result:?}");
    };
    let [jai_vm::Value::Int(value)] = values.as_slice() else {
        panic!("expected one integer result");
    };
    assert_eq!(value.value(), 42);
}

#[test]
fn weak_literals_keep_target_dependent_range_and_runtime_normalization() {
    let fixture =
        Fixture::new("address:*void=cast(*void)0xffffffffffffffff; main::()->int{return 42;}");
    let graph = fixture.graph();
    for layout in [None, Some(LayoutPolicy::lp64())] {
        resolve_graph_with_options(
            &graph,
            &ResolveOptions {
                layout,
                ..Default::default()
            },
            &mut jai_vm::NoEffects,
        )
        .unwrap();
    }
    assert!(
        resolve_graph_with_options(
            &graph,
            &ResolveOptions {
                layout: Some(pointer32()),
                ..Default::default()
            },
            &mut jai_vm::NoEffects
        )
        .is_err()
    );
    let fixture = Fixture::new(
        "main::()->int{p:=cast,trunc(*void)0x100000000;if p==null return 42;return 1;}",
    );
    let graph = fixture.graph();
    let program = resolve_graph_with_options(
        &graph,
        &ResolveOptions {
            layout: Some(pointer32()),
            ..Default::default()
        },
        &mut jai_vm::NoEffects,
    )
    .unwrap();
    let mut vm = jai_vm::Vm::new_with_target(
        &program,
        jai_vm::NoEffects,
        jai_vm::Limits::default(),
        jai_vm::ByteTarget {
            policy: pointer32(),
            endian: jai_vm::Endian::Little,
        },
    )
    .unwrap();
    let entry = match program.entry() {
        jai_ir::EntryPoint::Void(entry) | jai_ir::EntryPoint::Int(entry) => entry,
    };
    let result = vm.execute(entry, vec![]);
    assert!(
        matches!(result.outcome, jai_vm::Outcome::Complete(values) if values[0].integer().unwrap().value() == 42)
    );
}
