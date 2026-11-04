//! Separate code reached only by compiler runs from code reachable by the runtime.
//! Procedure values and global relocations participate, so freezing a callback does
//! not make its later runtime calls inherit compile-time storage semantics.
use super::{Compiler, HashSet, ProcId};
use jaic::ir::{self, Callee, FuncId, GlobalId, Inst};
use jaic::sema::procs::ProcTarget;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Node {
    Function(FuncId),
    Global(GlobalId),
}

fn reachable(program: &ir::Program, roots: impl IntoIterator<Item = Node>) -> HashSet<Node> {
    let mut seen = HashSet::default();
    let mut pending: Vec<_> = roots.into_iter().collect();
    while let Some(node) = pending.pop() {
        if !seen.insert(node) {
            continue;
        }
        match node {
            Node::Function(id) => {
                let Some(function) = program.funcs.get(id.0 as usize).and_then(Option::as_ref)
                else {
                    continue;
                };
                for inst in function.blocks.iter().flat_map(|b| &b.insts) {
                    match inst {
                        Inst::Call(call) => {
                            if let Callee::Func(id) = call.callee {
                                pending.push(Node::Function(id));
                            }
                        }
                        Inst::FuncAddr {
                            func, ..
                        } => pending.push(Node::Function(*func)),
                        Inst::GlobalAddr {
                            global, ..
                        } => pending.push(Node::Global(*global)),
                        _ => {}
                    }
                }
            }
            Node::Global(id) => {
                for reloc in &program.globals[id.0 as usize].relocs {
                    match reloc.target {
                        ir::RelocTarget::Func(id) => pending.push(Node::Function(id)),
                        ir::RelocTarget::Global(id) => pending.push(Node::Global(id)),
                        ir::RelocTarget::Foreign(_) => {}
                    }
                }
            }
        }
    }
    seen
}

pub(super) fn compile_time_only(
    compiler: &Compiler,
    targets: &crate::rules::lifetime_targets::Universe,
    entry_module: Option<jaic::sema::ModuleId>,
) -> (HashSet<ProcId>, HashSet<ProcId>) {
    let program = &compiler.out.program;
    let host = reachable(
        program,
        program.funcs.iter().enumerate().filter_map(|(i, f)| {
            f.as_ref()
                .filter(|f| f.origin != ir::FuncOrigin::Procedure)
                .map(|_| Node::Function(FuncId(i as u32)))
        }),
    );
    let mut roots: Vec<_> = program
        .funcs
        .iter()
        .enumerate()
        .filter_map(|(i, f)| {
            let id = FuncId(i as u32);
            f.as_ref()
                .filter(|f| {
                    f.origin == ir::FuncOrigin::Procedure
                        && (!host.contains(&Node::Function(id))
                            || matches!(f.linkage, ir::Linkage::Export(_)))
                })
                .map(|_| Node::Function(id))
        })
        .collect();
    if let Some(id) = compiler.pr.entry_point
        && let Some(ProcTarget::Func(id)) = compiler.pr.procs[id.0 as usize].target
    {
        roots.push(Node::Function(id));
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
    roots.extend(
        program
            .globals
            .iter()
            .enumerate()
            .filter_map(|(i, g)| g.export.as_ref().map(|_| Node::Global(GlobalId(i as u32)))),
    );
    let runtime = reachable(program, roots);
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
    let roots: HashSet<_> = program
        .funcs
        .iter()
        .filter_map(Option::as_ref)
        .filter(|f| f.origin != ir::FuncOrigin::Procedure)
        .filter(|f| f.blocks.len() == 1)
        .filter_map(|f| {
            // Argument expressions can call procedures before the outer call.
            // Only the final activation can be frozen before another call reuses
            // its stack. Branching thunks need a separate path proof.
            f.blocks[0]
                .insts
                .iter()
                .rev()
                .find_map(|inst| {
                    if let Inst::Call(call) = inst {
                        Some(match call.callee {
                            Callee::Func(id) => Some(id),
                            _ => None,
                        })
                    } else {
                        None
                    }
                })
                .flatten()
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
    (only, frozen)
}
