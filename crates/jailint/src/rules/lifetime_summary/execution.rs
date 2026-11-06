//! Separate code reached only by compiler runs from code reachable by the runtime.
//! Procedure values and global relocations participate, so freezing a callback does
//! not make its later runtime calls inherit compile-time storage semantics.
use super::{Compiler, HashSet, ProcId};
use jaic::ir::{FuncId, GlobalId};
use jaic::sema::procs::ProcTarget;

use jaic::ide::execution::{Call, Node, View};

fn reachable(graph: &View<'_>, roots: impl IntoIterator<Item = Node>) -> Option<HashSet<Node>> {
    let mut seen = HashSet::default();
    let mut pending: Vec<_> = roots.into_iter().collect();
    while let Some(node) = pending.pop() {
        if !graph.available() || jaic::interrupt::should_stop() {
            return None;
        }
        if seen.insert(node) && !graph.visit_edges(node, |node| pending.push(node)) {
            return None;
        }
    }
    (graph.available() && !jaic::interrupt::should_stop()).then_some(seen)
}

pub(super) fn compile_time_only(
    compiler: &Compiler,
    targets: &crate::rules::lifetime_targets::Universe,
    entry_module: Option<jaic::sema::ModuleId>,
) -> (HashSet<ProcId>, HashSet<ProcId>) {
    let Some(graph) = compiler.lifetime_execution() else {
        return (HashSet::default(), HashSet::default());
    };
    let Some(host) = reachable(
        &graph,
        (0..graph.function_count()).filter_map(|i| {
            graph
                .function(FuncId(i as u32))
                .filter(|f| !f.procedure)
                .map(|_| Node::Function(FuncId(i as u32)))
        }),
    ) else {
        return (HashSet::default(), HashSet::default());
    };
    let mut roots: Vec<_> = (0..graph.function_count())
        .filter_map(|i| {
            let id = FuncId(i as u32);
            graph
                .function(id)
                .filter(|f| f.procedure && (!host.contains(&Node::Function(id)) || f.exported))
                .map(|_| Node::Function(id))
        })
        .collect();
    roots.extend(graph.check_handler().map(Node::Function));
    if let Some(id) = compiler.pr.entry_point {
        let Some(proc) = compiler.pr.procs.get(id.0 as usize) else {
            return (HashSet::default(), HashSet::default());
        };
        match proc.target {
            Some(ProcTarget::Func(id)) => roots.push(Node::Function(id)),
            Some(ProcTarget::Foreign(id))
                if (id.0 as usize) < compiler.out.program.foreigns.len() => {}
            _ => return (HashSet::default(), HashSet::default()),
        }
    }
    // Lint checking does not select `pr.entry_point`, but the configured root
    // procedure still runs at runtime even when a #run also invokes it.
    roots.extend(compiler.pr.procs.iter().filter_map(|proc| {
        (proc.name.as_str() == compiler.options.entry_point
            && [compiler.ns.main_module, entry_module]
                .contains(&Some(compiler.ns.scopes[proc.scope.0 as usize].module)))
        .then_some(proc.target)
        .flatten()
        .and_then(|target| match target {
            ProcTarget::Func(id) => Some(Node::Function(id)),
            ProcTarget::Foreign(_) => None,
        })
    }));
    roots.extend((0..graph.global_count()).filter_map(|i| {
        let id = GlobalId(i as u32);
        graph.global_exported(id).then_some(Node::Global(id))
    }));
    let Some(runtime) = reachable(&graph, roots) else {
        return (HashSet::default(), HashSet::default());
    };
    let only: HashSet<_> = compiler
        .pr
        .procs
        .iter()
        .enumerate()
        .filter_map(|(i, proc)| {
            let Some(ProcTarget::Func(id)) = proc.target else {
                return None;
            };
            (host.contains(&Node::Function(id)) && !runtime.contains(&Node::Function(id)))
                .then_some(ProcId(i as u32))
        })
        .collect();
    let roots: HashSet<_> = (0..graph.function_count())
        .filter_map(|i| {
            graph.function(FuncId(i as u32)).and_then(|f| {
                match (f.procedure, f.generated_single_block_last_call) {
                    (false, Some(Call::Direct(id))) => Some(id),
                    _ => None,
                }
            })
        })
        .collect();
    let mut nested = HashSet::default();
    if let Some(facts) = compiler.ed.facts() {
        for body in facts.lifetime_bodies.values() {
            for event in body.blocks.iter().flat_map(|b| &b.events) {
                match event {
                    super::Event::Call {
                        callee, ..
                    } => {
                        nested.insert(*callee);
                    }
                    super::Event::IndirectCall {
                        callee,
                        allocator: false,
                        ..
                    } => {
                        nested.extend(
                            targets
                                .get(body.procedure, *callee)
                                .procedures
                                .iter()
                                .copied(),
                        );
                    }
                    _ => {}
                }
            }
        }
    }
    let frozen = only.iter().copied().filter(|id| {
        !nested.contains(id) && matches!(compiler.pr.procs[id.0 as usize].target, Some(ProcTarget::Func(f)) if roots.contains(&f))
    }).collect();
    if graph.available() && !jaic::interrupt::should_stop() {
        (only, frozen)
    } else {
        (HashSet::default(), HashSet::default())
    }
}
