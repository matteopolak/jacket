//! The allocator ownership query observes an address; it does not access that block.
use super::super::{Compiler, LifetimeArgument, ProcId};
use jaic::intern::Sym;
use jaic::sema::{
    Value,
    scope::{EntityState, Resolved},
};

pub(in crate::rules) fn address_query(
    compiler: &Compiler,
    proc: ProcId,
    args: &[LifetimeArgument],
) -> bool {
    // This is the bootstrap Allocator ABI, like ALLOCATE/RESIZE/FREE elsewhere.
    if args.iter().find(|a| a.formal == 0).and_then(|a| a.constant) != Some(9) {
        return false;
    }
    let Some(signature) = compiler.pr.procs[proc.0 as usize].sig.as_ref() else {
        return false;
    };
    let Some(preload) = compiler.ns.preload else {
        return false;
    };
    let scope = compiler.ns.modules[preload.0 as usize].scope;
    compiler
        .scope(scope)
        .names
        .get(&Sym::intern("Allocator"))
        .into_iter()
        .flatten()
        .any(|id| {
            let EntityState::Done(Resolved::Const {
                value: Value::Type(ty),
                ..
            }) = &compiler.entity(*id).state
            else {
                return false;
            };
            compiler.ty.types.as_struct(*ty).is_some_and(|st| {
                compiler
                    .ty
                    .types
                    .struct_info(st)
                    .fields
                    .iter()
                    .any(|field| {
                        field.name == Some(Sym::intern("proc")) && field.ty == signature.ty
                    })
            })
        })
}

// A wrapper preserves query semantics only when the queried formal is never accessed,
// retained, returned, written, or passed to a possibly invalidating operation.
pub(in crate::rules::lifetime_summary) fn solve(
    compiler: &Compiler,
    valid: &super::HashMap<ProcId, &jaic::ide::lifetimes::LifetimeBody>,
    graph: &super::HashMap<ProcId, Vec<ProcId>>,
    all: &mut super::super::Summaries,
) {
    use super::super::{Event, Resolver, Root, acyclic, sccs};
    let mut ids: Vec<_> = valid.keys().copied().collect();
    ids.sort_unstable_by_key(|p| p.0);
    for component in sccs(&ids, graph) {
        let once = acyclic(&component, graph);
        loop {
            let mut changed = false;
            for id in &component {
                let calls: Vec<_> = valid[id]
                    .blocks
                    .iter()
                    .flat_map(|b| &b.events)
                    .filter_map(|event| {
                        let Event::Call {
                            callee,
                            args,
                            expands: false,
                            ..
                        } = event
                        else {
                            return None;
                        };
                        let direct = address_query(compiler, *callee, args);
                        (direct
                            || all
                                .procs
                                .get(callee)
                                .is_some_and(|s| !s.address_queries.is_empty()))
                        .then_some((callee, args, direct))
                    })
                    .collect();
                if calls.is_empty() {
                    continue;
                }
                let resolver = Resolver::new(compiler, valid[id], &all.procs, &all.targets);
                let mut queries = super::HashSet::default();
                for (callee, args, direct) in calls {
                    for arg in args {
                        if !(direct && arg.formal == 3
                            || all
                                .procs
                                .get(callee)
                                .is_some_and(|s| s.address_queries.contains(&arg.formal)))
                        {
                            continue;
                        }
                        if let Some(path) = resolver.argument_trace(
                            arg,
                            &all.procs,
                            &mut super::HashSet::default(),
                            0,
                        ) && let Root::Parameter(formal) = path.root
                            && path.steps.is_empty()
                        {
                            queries.insert(formal);
                        }
                    }
                }
                let summary = all.procs.get_mut(id).unwrap();
                queries.retain(|n| {
                    !summary.may_read_formals.contains(n)
                        && !summary.unknown_write.contains(n)
                        && !summary.escapes.contains(n)
                        && !summary
                            .may
                            .keys()
                            .any(|e| e.target.root == Root::Parameter(*n))
                        && !summary
                            .returns
                            .values()
                            .any(|(p, _)| p.root == Root::Parameter(*n))
                });
                let before = summary.address_queries.len();
                summary.address_queries.extend(queries);
                changed |= before != summary.address_queries.len();
            }
            if !changed || once {
                break;
            }
        }
    }
}
