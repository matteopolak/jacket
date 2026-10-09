//! Dividing a program among modules before the optimizer runs.
//!
//! Optimizing a large program as one LLVM module keeps the passes on one thread. Instead the
//! functions are grouped by who refers to them, each group is lowered into a module of its own,
//! and every module runs the whole pipeline and writes its own object file in parallel.
//!
//! Two things keep the machine code close to what one module would give:
//!
//! - A function or global that only its own module refers to stays internal there, so the
//!   interprocedural passes (single-caller inlining, argument promotion, constant propagation
//!   through parameters, dead code removal) still see all its uses. Functions are grouped so
//!   that most of them have all their callers in the same module: a function with a single
//!   referrer joins it, and only where a group would outgrow its share is the tree cut.
//! - A small function that a module calls but another module owns is defined in the caller's
//!   module too, as `available_externally`: the optimizer may inline it, and the copy is never
//!   written out. The functions that copy refers to become visible to the linker in turn.
//!
//! What is lost compared to one module: calls to larger functions of other modules are not
//! inlined, nor optimized through their arguments, and constants stored in another module's
//! globals are not folded.

use crate::lower::Plan;
use jaic::ir::{Callee, Func, Inst, Linkage, Program, RelocTarget};
use std::collections::HashMap;

/// IR instructions in a function up to which another module may hold a copy of it for inlining.
const IMPORT_MAX: usize = 200;

/// Instructions of imported copies a module may hold, as a percentage of what it owns.
const IMPORT_BUDGET_PERCENT: usize = 100;

/// Read-only globals up to this size are copied into the modules that read them.
const GLOBAL_IMPORT_MAX: u64 = 16 * 1024;

const NONE: u32 = u32::MAX;

/// Instructions in a function, counting each block's terminator.
pub(crate) fn func_weight(func: &Func) -> usize {
    func.blocks.iter().map(|b| b.insts.len() + 1).sum()
}

fn global_weight(g: &jaic::ir::Global) -> usize {
    g.init.len() / 64 + g.relocs.len() + 1
}

/// Whether each module that refers to the global gets a private copy of it instead of sharing
/// one: a read-only block of zeros with no pointers (the default value of a large type). The
/// optimizer turns a copy from it into a memset, and the copy is then deleted as unused, where
/// one shared definition would stay in the executable.
pub(crate) fn replicated(g: &jaic::ir::Global) -> bool {
    g.read_only && g.export.is_none() && g.relocs.is_empty() && g.init.iter().all(|&b| b == 0)
}

/// A plan that gives every function to the lightest of `units` modules (largest first) and
/// every global to module 0, with every symbol visible to the linker and nothing imported.
pub(crate) fn flat(program: &Program, units: usize) -> Plan {
    let mut order: Vec<usize> = (0..program.funcs.len()).collect();
    let weight = |i: usize| program.funcs[i].as_ref().map_or(0, func_weight);
    order.sort_by_key(|&i| std::cmp::Reverse(weight(i)));
    let mut load = vec![0usize; units];
    load[0] = program.globals.iter().map(global_weight).sum();
    let mut owner = vec![0u32; program.funcs.len()];
    for i in order {
        let unit = (0..units).min_by_key(|&u| load[u]).unwrap_or(0);
        owner[i] = unit as u32;
        load[unit] += weight(i);
    }
    Plan {
        func_owner: owner,
        global_owner: vec![0; program.globals.len()],
        func_shared: vec![true; program.funcs.len()],
        global_shared: vec![true; program.globals.len()],
        imports: vec![Vec::new(); units],
        global_imports: vec![Vec::new(); units],
    }
}

/// Everything a function refers to, as node numbers (functions first, then globals).
fn references(func: &Func, nf: usize, foreign_func: &[Option<u32>], out: &mut Vec<u32>) {
    out.clear();
    let foreign = |id: usize, out: &mut Vec<u32>| {
        if let Some(f) = foreign_func[id] {
            out.push(f);
        }
    };
    for block in &func.blocks {
        for inst in &block.insts {
            match inst {
                Inst::GlobalAddr {
                    global, ..
                } => out.push((nf + global.0 as usize) as u32),
                Inst::FuncAddr {
                    func, ..
                } => out.push(func.0),
                Inst::ForeignAddr {
                    foreign: id, ..
                } => foreign(id.0 as usize, out),
                Inst::Call(call) => match &call.callee {
                    Callee::Func(id) => out.push(id.0),
                    Callee::Foreign(id) => foreign(id.0 as usize, out),
                    Callee::Indirect(..) => {}
                },
                _ => {}
            }
        }
    }
    out.sort_unstable();
    out.dedup();
}

/// Divide `program` among `units` modules.
pub(crate) fn plan(program: &Program, units: usize) -> Plan {
    let nf = program.funcs.len();
    let ng = program.globals.len();
    let nodes = nf + ng;

    // A foreign symbol named like a program export is that function (`Backend::declare_foreign`).
    let mut exports: HashMap<&str, u32> = HashMap::new();
    for (i, func) in program.funcs.iter().enumerate() {
        if let Some(func) = func
            && let Linkage::Export(name) = &func.linkage
        {
            exports.entry(name.as_str()).or_insert(i as u32);
        }
    }
    let foreign_func: Vec<Option<u32>> = program
        .foreigns
        .iter()
        .map(|f| {
            if f.is_data {
                None
            } else {
                exports.get(f.symbol.as_str()).copied()
            }
        })
        .collect();

    let mut weight = vec![0usize; nodes];
    let mut refs: Vec<Vec<u32>> = vec![Vec::new(); nodes];
    let mut scratch = Vec::new();
    for (i, func) in program.funcs.iter().enumerate() {
        if let Some(func) = func {
            weight[i] = func_weight(func);
            references(func, nf, &foreign_func, &mut scratch);
            refs[i] = scratch.clone();
        }
    }
    for (j, g) in program.globals.iter().enumerate() {
        weight[nf + j] = g.init.len() / 4096 + g.relocs.len() / 8 + 1;
        let mut out: Vec<u32> = g
            .relocs
            .iter()
            .filter_map(|r| match r.target {
                RelocTarget::Global(id) => Some((nf + id.0 as usize) as u32),
                RelocTarget::Func(id) => Some(id.0),
                RelocTarget::Foreign(id) => foreign_func[id.0 as usize],
            })
            .collect();
        out.sort_unstable();
        out.dedup();
        refs[nf + j] = out;
    }

    // Private copies are no one's business but the module that makes them.
    let copy: Vec<bool> = program.globals.iter().map(replicated).collect();
    for targets in &mut refs {
        targets.retain(|&t| (t as usize) < nf || !copy[t as usize - nf]);
    }
    for (j, &is_copy) in copy.iter().enumerate() {
        if is_copy {
            weight[nf + j] = 0;
        }
    }

    // Who refers to each node: none, one node, or several.
    const MANY: u32 = u32::MAX - 1;
    let mut sole = vec![NONE; nodes];
    for (n, targets) in refs.iter().enumerate() {
        for &t in targets {
            if t as usize == n {
                continue;
            }
            let slot = &mut sole[t as usize];
            *slot = if *slot == NONE || *slot == n as u32 {
                n as u32
            } else {
                MANY
            };
        }
    }

    // A node with one referrer hangs under it. Following those links gives a forest (plus cycles
    // of nodes nothing else refers to, which the loop below starts at an arbitrary member).
    let mut children: Vec<Vec<u32>> = vec![Vec::new(); nodes];
    let mut roots: Vec<u32> = Vec::new();
    for (n, &referrer) in sole.iter().enumerate() {
        match referrer {
            NONE | MANY => roots.push(n as u32),
            parent => children[parent as usize].push(n as u32),
        }
    }
    let mut parent = vec![NONE; nodes];
    let mut order: Vec<u32> = Vec::with_capacity(nodes);
    let mut seen = vec![false; nodes];
    let visit = |root: u32, order: &mut Vec<u32>, seen: &mut Vec<bool>, parent: &mut Vec<u32>| {
        let mut stack = vec![root];
        seen[root as usize] = true;
        while let Some(n) = stack.pop() {
            order.push(n);
            for &c in &children[n as usize] {
                if !seen[c as usize] {
                    seen[c as usize] = true;
                    parent[c as usize] = n;
                    stack.push(c);
                }
            }
        }
    };
    for &r in &roots {
        visit(r, &mut order, &mut seen, &mut parent);
    }
    for n in 0..nodes as u32 {
        if !seen[n as usize] {
            visit(n, &mut order, &mut seen, &mut parent);
        }
    }

    // Cut subtrees where a group would outgrow its share.
    let total: usize = weight.iter().sum();
    let cap = (total / (units * 2)).max(1);
    let mut acc = weight.clone();
    let mut group_root = vec![false; nodes];
    let mut kids: Vec<Vec<u32>> = vec![Vec::new(); nodes];
    for &n in &order {
        let p = parent[n as usize];
        if p != NONE {
            kids[p as usize].push(n);
        }
    }
    for &n in order.iter().rev() {
        let mut list = std::mem::take(&mut kids[n as usize]);
        list.sort_by_key(|&k| std::cmp::Reverse(acc[k as usize]));
        let mut sum = weight[n as usize] + list.iter().map(|&k| acc[k as usize]).sum::<usize>();
        for &k in &list {
            if sum <= cap {
                break;
            }
            group_root[k as usize] = true;
            sum -= acc[k as usize];
        }
        acc[n as usize] = sum;
        if parent[n as usize] == NONE {
            group_root[n as usize] = true;
        }
    }
    let mut group = vec![0u32; nodes];
    let mut group_weight: Vec<usize> = Vec::new();
    for &n in &order {
        let p = parent[n as usize];
        group[n as usize] = if group_root[n as usize] || p == NONE {
            group_weight.push(acc[n as usize]);
            (group_weight.len() - 1) as u32
        } else {
            group[p as usize]
        };
    }

    // Largest group first, each to the lightest module.
    let mut by_weight: Vec<u32> = (0..group_weight.len() as u32).collect();
    by_weight.sort_by_key(|&g| (std::cmp::Reverse(group_weight[g as usize]), g));
    let mut load = vec![0usize; units];
    let mut bin = vec![0u32; group_weight.len()];
    for g in by_weight {
        let unit = (0..units).min_by_key(|&u| load[u]).unwrap_or(0);
        bin[g as usize] = unit as u32;
        load[unit] += group_weight[g as usize];
    }
    let owner: Vec<u32> = group.iter().map(|&g| bin[g as usize]).collect();

    // What the linker has to see: whatever another module refers to, and what the outside does.
    let mut shared = vec![false; nodes];
    for (i, func) in program.funcs.iter().enumerate() {
        if let Some(func) = func
            && matches!(func.linkage, Linkage::Export(_))
        {
            shared[i] = true;
        }
    }
    if let Some(id) = program.check_failed {
        shared[id.0 as usize] = true;
    }
    for (j, g) in program.globals.iter().enumerate() {
        if g.export.is_some() {
            shared[nf + j] = true;
        }
    }
    let mut need: Vec<Vec<bool>> = vec![vec![false; nodes]; units];
    for n in 0..nodes {
        let k = owner[n] as usize;
        for &r in &refs[n] {
            if owner[r as usize] as usize != k {
                shared[r as usize] = true;
                need[k][r as usize] = true;
            }
        }
    }

    // Small functions another module calls are copied into the caller's module.
    let importable = |n: usize| -> bool {
        if n >= nf {
            // Constant data folds into the code that reads it; a zero default value turns a copy
            // into a memset.
            let g = &program.globals[n - nf];
            return g.read_only
                && g.export.is_none()
                && g.size <= GLOBAL_IMPORT_MAX
                && !replicated(g);
        }
        weight[n] <= IMPORT_MAX
            && program.funcs[n]
                .as_ref()
                .is_some_and(|f| !matches!(f.linkage, Linkage::Export(_)))
    };
    let mut owned_weight = vec![0usize; units];
    for n in 0..nodes {
        owned_weight[owner[n] as usize] += weight[n];
    }
    let mut imports: Vec<Vec<u32>> = vec![Vec::new(); units];
    for k in 0..units {
        let budget = owned_weight[k] * IMPORT_BUDGET_PERCENT / 100;
        let mut used = 0usize;
        let mut queue: Vec<u32> = (0..nodes as u32)
            .filter(|&n| need[k][n as usize] && importable(n as usize))
            .collect();
        let mut imported = vec![false; nodes];
        while let Some(n) = queue.pop() {
            let n = n as usize;
            if imported[n] || (n < nf && used + weight[n] > budget) {
                continue;
            }
            imported[n] = true;
            if n < nf {
                used += weight[n];
            }
            imports[k].push(n as u32);
            for &r in &refs[n] {
                let r = r as usize;
                if owner[r] as usize != k {
                    shared[r] = true;
                    if !need[k][r] {
                        need[k][r] = true;
                    }
                    if importable(r) && !imported[r] {
                        queue.push(r as u32);
                    }
                }
            }
        }
        imports[k].sort_unstable();
    }

    let global_imports: Vec<Vec<u32>> = imports
        .iter()
        .map(|v| {
            v.iter()
                .filter(|&&n| n as usize >= nf)
                .map(|&n| n - nf as u32)
                .collect()
        })
        .collect();
    for v in &mut imports {
        v.retain(|&n| (n as usize) < nf);
    }
    Plan {
        global_imports,
        func_owner: owner[..nf].to_vec(),
        global_owner: owner[nf..].to_vec(),
        func_shared: shared[..nf].to_vec(),
        global_shared: shared[nf..].to_vec(),
        imports,
    }
}
