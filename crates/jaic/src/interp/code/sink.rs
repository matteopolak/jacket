//! Initialization of locals that nothing reads, or that only one side of a branch reads.
//!
//! Lowering fills a macro's parameter slots (a default message, the caller's location, an empty
//! argument list) before its body runs, so `assert(ok)` builds all of them, twice over, even
//! though they are only looked at when it fails. This pass works on the IR, before ops are made
//! from it, and
//!
//! - drops the stores into a slot whose contents are never read (or only copied into another
//!   such slot), and
//! - moves the stores into a slot that is only read in one successor of a conditional branch to
//!   the start of that successor.
//!
//! A slot qualifies only when its address is used for nothing but storing into it, copying
//! from it, reading it and passing it on (not stored anywhere or computed with), and the
//! values it is set from are constants, so a pointer kept past its lifetime sees what it
//! would have seen. Code with debug information is left alone.
use crate::ir::{self, Func, GlobalId, Inst, Val, inst_vals, term_uses};

/// Where the instructions of a function run, if not where they are written.
pub(super) struct Plan {
    /// Instructions to leave out: deleted, or moved to `front`.
    pub skip: Vec<Vec<bool>>,
    /// Instructions of another block that a block starts with, in order.
    pub front: Vec<Vec<(u32, u32)>>,
    /// The deleted instructions (the moved ones are in `front`).
    pub deleted: Vec<(u32, u32)>,
}

/// What an instruction does with an operand.
#[derive(Clone, Copy, PartialEq)]
enum Role {
    /// Stored into: a store's address, a copy's or zeroing's destination.
    Write,
    /// The base of a pointer addition.
    Derive,
    /// Read from: a load's address, a copy's source.
    Read,
    /// Stored as a value (the pointer escapes).
    Value,
    Other,
}

/// Marks a use in a block's terminator rather than an instruction.
const TERM: u32 = u32::MAX;

fn roles(inst: &Inst, f: &mut impl FnMut(Val, Role)) {
    match inst {
        Inst::Store {
            addr,
            value,
            ..
        } => {
            f(*addr, Role::Write);
            f(*value, Role::Value);
        }
        Inst::Copy {
            dst,
            src,
            ..
        } => {
            f(*dst, Role::Write);
            f(*src, Role::Read);
        }
        Inst::Zero {
            dst, ..
        } => f(*dst, Role::Write),
        Inst::PtrAdd {
            base,
            offset,
            ..
        } => {
            f(*base, Role::Derive);
            f(*offset, Role::Other);
        }
        Inst::Load {
            addr, ..
        } => f(*addr, Role::Read),
        other => inst_vals(other, &mut |_| {}, &mut |v| f(v, Role::Other)),
    }
}

pub(super) fn plan(
    func: &Func,
    defs: &[u32],
    read_only: &dyn Fn(GlobalId) -> bool,
) -> Option<Plan> {
    if func.debug.is_some() {
        return None;
    }
    let n = func.vals.len();
    let nblocks = func.blocks.len();
    let single = |v: Val| defs[v.0 as usize] == 1;

    // Where each singly-defined value is defined, and every use of every value.
    let mut def_site: Vec<Option<(u32, u32)>> = vec![None; n];
    let mut sites: Vec<Vec<(u32, u32, Role)>> = vec![Vec::new(); n];
    for (b, block) in func.blocks.iter().enumerate() {
        for (i, inst) in block.insts.iter().enumerate() {
            inst_vals(
                inst,
                &mut |v| {
                    if single(v) {
                        def_site[v.0 as usize] = Some((b as u32, i as u32));
                    }
                },
                &mut |_| {},
            );
            roles(inst, &mut |v, role| {
                sites[v.0 as usize].push((b as u32, i as u32, role));
            });
        }
        term_uses(&block.term, &mut |v| {
            sites[v.0 as usize].push((b as u32, TERM, Role::Other));
        });
    }

    // The slot each address value points into: the result of `SlotAddr`, or of a pointer
    // addition from one.
    let nslots = func.slots.len();
    let mut owner: Vec<Option<u32>> = vec![None; n];
    let mut tracked = vec![true; nslots];
    let mut mentioned = vec![false; nslots];
    for block in &func.blocks {
        for inst in &block.insts {
            if let Inst::SlotAddr {
                dst,
                slot,
            } = inst
            {
                mentioned[slot.0 as usize] = true;
                if single(*dst) {
                    owner[dst.0 as usize] = Some(slot.0);
                } else {
                    tracked[slot.0 as usize] = false;
                }
            }
        }
    }
    for _ in 0..4 {
        let mut changed = false;
        for block in &func.blocks {
            for inst in &block.insts {
                if let Inst::PtrAdd {
                    dst,
                    base,
                    ..
                } = inst
                    && single(*dst)
                    && owner[dst.0 as usize].is_none()
                    && let Some(s) = owner[base.0 as usize]
                {
                    owner[dst.0 as usize] = Some(s);
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    let mut members: Vec<Vec<Val>> = vec![Vec::new(); nslots];
    for (v, o) in owner.iter().enumerate() {
        if let Some(s) = *o {
            members[s as usize].push(Val(v as u32));
        }
    }
    let inst_at = |b: u32, i: u32| &func.blocks[b as usize].insts[i as usize];

    // Slots something reads (or that escape): the rest are only written.
    let mut live = vec![false; nslots];
    for s in 0..nslots {
        live[s] = !tracked[s] || !mentioned[s];
    }
    loop {
        let mut changed = false;
        for s in 0..nslots {
            if live[s] {
                continue;
            }
            let reads = members[s].iter().any(|&v| {
                sites[v.0 as usize].iter().any(|&(b, i, role)| match role {
                    Role::Write => false,
                    Role::Derive => match inst_at(b, i) {
                        Inst::PtrAdd {
                            dst, ..
                        } => owner[dst.0 as usize] != Some(s as u32),
                        _ => true,
                    },
                    Role::Read => match (i != TERM).then(|| inst_at(b, i)) {
                        Some(Inst::Copy {
                            dst, ..
                        }) => match owner[dst.0 as usize] {
                            Some(t) => t as usize == s || live[t as usize],
                            None => true,
                        },
                        _ => true,
                    },
                    Role::Value | Role::Other => true,
                })
            });
            if reads {
                live[s] = true;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let mut skip: Vec<Vec<bool>> = func
        .blocks
        .iter()
        .map(|b| vec![false; b.insts.len()])
        .collect();
    let mut deleted = Vec::new();
    let is_dead = |v: Val| owner[v.0 as usize].is_some_and(|s| !live[s as usize]);
    for (b, block) in func.blocks.iter().enumerate() {
        for (i, inst) in block.insts.iter().enumerate() {
            let dead = match inst {
                Inst::SlotAddr {
                    dst, ..
                }
                | Inst::PtrAdd {
                    dst, ..
                } => is_dead(*dst),
                Inst::Store {
                    addr, ..
                } => is_dead(*addr),
                Inst::Copy {
                    dst, ..
                }
                | Inst::Zero {
                    dst, ..
                } => is_dead(*dst),
                _ => false,
            };
            if dead {
                skip[b][i] = true;
                deleted.push((b as u32, i as u32));
            }
        }
    }

    // Moving into a successor.
    let mut preds = vec![0u32; nblocks];
    if let Some(entry) = preds.first_mut() {
        *entry = 1;
    }
    for block in &func.blocks {
        for next in block.term.successors() {
            preds[next.0 as usize] += 1;
        }
    }
    // The block all of a slot's address values are defined in.
    let mut home: Vec<Option<u32>> = vec![None; nslots];
    let mut split = vec![false; nslots];
    for (s, list) in members.iter().enumerate() {
        for v in list {
            let Some((b, _)) = def_site[v.0 as usize] else {
                split[s] = true;
                continue;
            };
            match home[s] {
                None => home[s] = Some(b),
                Some(h) if h != b => split[s] = true,
                _ => {}
            }
        }
    }
    let mut front: Vec<Vec<(u32, u32)>> = vec![Vec::new(); nblocks];
    for b in 0..nblocks {
        let ir::Term::Branch {
            then_block,
            else_block,
            ..
        } = func.blocks[b].term
        else {
            continue;
        };
        if then_block == else_block {
            continue;
        }
        for x in [then_block.0 as usize, else_block.0 as usize] {
            if x == b || preds[x] != 1 {
                continue;
            }
            let mut set: Vec<bool> = (0..nslots)
                .map(|s| live[s] && !split[s] && home[s] == Some(b as u32))
                .collect();
            let in_set =
                |set: &[bool], v: Val| owner[v.0 as usize].is_some_and(|s| set[s as usize]);
            // Whether the use of a slot's value at `(sb, i)` fits moving the slots in `set`.
            let site_agrees = |set: &[bool], sb: u32, i: u32, role: Role| -> bool {
                if sb as usize == x {
                    return role != Role::Value;
                }
                if sb as usize != b || i == TERM || skip[sb as usize][i as usize] {
                    return sb as usize == b && i != TERM;
                }
                match (role, inst_at(sb, i)) {
                    (Role::Derive, _) => true,
                    (
                        Role::Read,
                        Inst::Copy {
                            dst, ..
                        },
                    ) => in_set(set, *dst),
                    (
                        Role::Write,
                        Inst::Store {
                            value, ..
                        },
                    ) => {
                        matches!(
                            def_site[value.0 as usize].map(|(db, di)| inst_at(db, di)),
                            Some(Inst::IConst { .. } | Inst::FConst { .. })
                        ) && def_site[value.0 as usize].is_some_and(|(db, _)| db as usize == b)
                    }
                    (
                        Role::Write,
                        Inst::Zero {
                            ..
                        },
                    ) => true,
                    (
                        Role::Write,
                        Inst::Copy {
                            src, ..
                        },
                    ) => {
                        in_set(set, *src)
                            || def_site[src.0 as usize].is_some_and(|(db, di)| {
                                db as usize == b
                                    && matches!(
                                        inst_at(db, di),
                                        Inst::GlobalAddr { global, .. }
                                            if read_only(*global)
                                    )
                            })
                    }
                    _ => false,
                }
            };
            // Drop slots that something else reads or sets, until the rest agree.
            loop {
                let mut changed = false;
                for s in 0..nslots {
                    if !set[s] {
                        continue;
                    }
                    let ok = members[s].iter().all(|&v| {
                        sites[v.0 as usize]
                            .iter()
                            .all(|&(sb, i, role)| site_agrees(&set, sb, i, role))
                    });
                    if !ok {
                        set[s] = false;
                        changed = true;
                    }
                }
                if !changed {
                    break;
                }
            }
            if !set.iter().any(|&s| s) {
                continue;
            }
            // The instructions of `b` that fill those slots, and the constants only they use.
            let mut moved: Vec<u32> = Vec::new();
            for (i, inst) in func.blocks[b].insts.iter().enumerate() {
                if skip[b][i] {
                    continue;
                }
                let takes = match inst {
                    Inst::SlotAddr {
                        dst, ..
                    }
                    | Inst::PtrAdd {
                        dst, ..
                    } => in_set(&set, *dst),
                    Inst::Store {
                        addr, ..
                    } => in_set(&set, *addr),
                    Inst::Copy {
                        dst, ..
                    }
                    | Inst::Zero {
                        dst, ..
                    } => in_set(&set, *dst),
                    _ => false,
                };
                if takes {
                    moved.push(i as u32);
                }
            }
            let mut helpers: Vec<u32> = Vec::new();
            let mut sound = true;
            for &i in &moved {
                inst_vals(inst_at(b as u32, i), &mut |_| {}, &mut |v| {
                    if in_set(&set, v) {
                        return;
                    }
                    if !single(v) {
                        sound = false;
                        return;
                    }
                    let Some((db, di)) = def_site[v.0 as usize] else {
                        return;
                    };
                    if db as usize != b {
                        return;
                    }
                    if moved.contains(&di) || helpers.contains(&di) {
                        return;
                    }
                    let leaf = matches!(
                        inst_at(db, di),
                        Inst::IConst { .. } | Inst::FConst { .. } | Inst::GlobalAddr { .. }
                    );
                    let only_here = sites[v.0 as usize].iter().all(|&(ub, ui, _)| {
                        (ub as usize == x)
                            || (ub as usize == b && ui != TERM && moved.contains(&ui))
                    });
                    if leaf && only_here && di < i {
                        helpers.push(di);
                    }
                });
            }
            if !sound {
                continue;
            }
            let mut all: Vec<u32> = moved.into_iter().chain(helpers).collect();
            all.sort_unstable();
            all.dedup();
            for &i in &all {
                skip[b][i as usize] = true;
                front[x].push((b as u32, i));
            }
        }
    }
    let any = !deleted.is_empty() || front.iter().any(|f| !f.is_empty());
    any.then_some(Plan {
        skip,
        front,
        deleted,
    })
}
