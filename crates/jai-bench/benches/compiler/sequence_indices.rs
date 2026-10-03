//! Resolve real unsigned index expressions with the compiler allocator profiler.
use divan::{Bencher, counter::ItemsCount};

#[divan::bench(args = [64, 1024])]
fn unsigned_string_index_resolve(bencher: Bencher, count: usize) {
    let mut source = String::from("main::()->int{text:=\"A\";index:u64=0;sum:int=0;");
    for _ in 0..count {
        source.push_str("sum+=cast(int)text[index];");
    }
    source.push_str("return sum;}");
    let path = std::path::Path::new("/bench/sequence-index.jai");
    let mut inputs = jai_modules::SourceOverlay::default();
    inputs.insert(path, source.into_bytes()).unwrap();
    let graph = jai_modules::ModuleGraph::load_with_provider(
        path,
        jai_modules::GraphOptions::default(),
        &inputs,
    )
    .unwrap();
    let options = jai_sema::ResolveOptions {
        layout: Some(jai_types::LayoutPolicy::lp64()),
        ..Default::default()
    };
    let resolve =
        || jai_sema::resolve_graph_with_options(&graph, &options, &mut jai_vm::NoEffects).unwrap();
    let preflight = resolve();
    let execution = jai_vm::execute(
        &preflight,
        jai_vm::Limits {
            fuel: 4_000_000,
            ..Default::default()
        },
    );
    let jai_vm::Outcome::Complete(values) = execution.outcome else {
        panic!("index preflight failed: {:?}", execution.outcome)
    };
    assert_eq!(values[0].integer().unwrap().value(), 65 * count as i128);
    bencher.counter(ItemsCount::new(count)).bench_local(resolve);
}
