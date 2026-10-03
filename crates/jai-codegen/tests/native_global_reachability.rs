//! Link demand follows checked global places, including their projected roots.
use jai_codegen::native_reachability::Reachable;
use jai_ir::{ConstantKind, GlobalId, GlobalInitializer};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn program(source: &str) -> jai_ir::Program {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let fixture = Fixture(std::env::temp_dir().join(format!(
        "jai-global-demand-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir(&fixture.0).unwrap();
    let input = fixture.0.join("main.jai");
    fs::write(&input, source).unwrap();
    let graph = jai_modules::ModuleGraph::load(&input, Default::default()).unwrap();
    jai_sema::resolve_graph(&graph)
        .unwrap_or_else(|error| panic!("{}", error.render(graph.sources())))
}

fn integer_global(program: &jai_ir::Program, expected: i128) -> GlobalId {
    program
        .globals()
        .iter()
        .find_map(|global| {
            let value = match global.initializer() {
                GlobalInitializer::Int(value) => Some(*value),
                GlobalInitializer::Value(value) => match value.kind {
                    ConstantKind::Int(value) => Some(value),
                    _ => None,
                },
                _ => None,
            };
            value
                .filter(|value| value.value() == expected)
                .map(|_| global.id())
        })
        .unwrap()
}

#[test]
fn global_demand_tracks_calls_but_skips_inactive_and_terminated_bodies() {
    let program = program(
        r#"
active:int=21;
inactive:int=3;
after_return:int=5;
dead_body:int=7;
callee_global:int=9;
dead :: () { dead_body+=1; }
helper :: () { callee_global+=1; }
main :: ()->int {
    if false inactive+=1;
    helper();
    return active+active;
    after_return+=1;
}
"#,
    );
    let reachable = Reachable::executable(program.library(), program.entry()).unwrap();
    for live in [21, 9] {
        assert!(reachable.contains_global(integer_global(&program, live)));
    }
    for dead in [3, 5, 7] {
        assert!(!reachable.contains_global(integer_global(&program, dead)));
    }
}

#[test]
fn projected_record_and_array_places_demand_their_actual_global_root() {
    let program = program(
        r#"
Pair :: struct { first:int; second:int; }
pair:Pair=Pair.{first=20,second=22};
numbers:[2]int=.[20,22];
unused:int=3;
main :: ()->int {
    address:=*pair.second;
    address.*+=0;
    numbers[0]+=0;
    return pair.first+numbers[1];
}
"#,
    );
    let reachable = Reachable::executable(program.library(), program.entry()).unwrap();
    let unused = integer_global(&program, 3);
    assert!(!reachable.contains_global(unused));
    for global in program
        .globals()
        .iter()
        .filter(|global| global.id() != unused)
    {
        assert!(reachable.contains_global(global.id()));
    }
}
