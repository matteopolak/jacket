use divan::{Bencher, counter::ItemsCount};
use jai_modules::{DiscoveryStatus, GraphDiscovery, GraphOptions, SourceOverlay};
use std::{fmt::Write, path::Path};

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::main();
}

fn inputs(files: usize, conditional: bool) -> SourceOverlay {
    let mut overlay = SourceOverlay::default();
    let mut main = String::new();
    if conditional {
        main.push_str("#if #run true {\n");
    }
    for n in 0..files {
        writeln!(main, "#load \"file-{n}.jai\";").unwrap();
        overlay
            .insert(
                Path::new(&format!("/bench/file-{n}.jai")),
                format!("constant_{n} :: {n};").into_bytes(),
            )
            .unwrap();
    }
    if conditional {
        main.push_str("}\n");
    }
    main.push_str("main :: () {}\n");
    overlay
        .insert(Path::new("/bench/main.jai"), main.into_bytes())
        .unwrap();
    overlay
}

fn discover(inputs: &SourceOverlay, conditional: bool) -> jai_modules::ModuleGraph {
    let mut session = GraphDiscovery::new(
        Path::new("/bench/main.jai"),
        GraphOptions::default(),
        inputs,
    )
    .unwrap();
    if conditional {
        let DiscoveryStatus::Awaiting {
            conditions,
            dependencies,
        } = session.advance().unwrap()
        else {
            panic!("expected pending source condition");
        };
        assert!(dependencies.is_empty());
        assert_eq!(conditions.len(), 1);
        session.select_condition(conditions[0], true).unwrap();
    }
    assert!(session.advance().unwrap().is_complete());
    session
        .into_graph()
        .unwrap_or_else(|_| panic!("discovery remained pending"))
}

#[divan::bench(args = [4, 64, 1024])]
fn module_discovery(bencher: Bencher, files: usize) {
    let inputs = inputs(files, false);
    assert_eq!(discover(&inputs, false).files().len(), files + 1);
    bencher
        .counter(ItemsCount::new(files))
        .bench_local(|| discover(divan::black_box(&inputs), false));
}

#[divan::bench(args = [4, 64, 1024])]
fn module_discovery_resume(bencher: Bencher, files: usize) {
    let inputs = inputs(files, true);
    assert_eq!(discover(&inputs, true).files().len(), files + 1);
    bencher
        .counter(ItemsCount::new(files))
        .bench_local(|| discover(divan::black_box(&inputs), true));
}
