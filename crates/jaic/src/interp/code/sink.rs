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
    let mut sinker = Sinker::new(func, defs, read_only);
    sinker.collect_sites();
    sinker.assign_owners();
    sinker.find_live_slots();
    sinker.delete_dead();
    sinker.find_homes();
    for b in 0..func.blocks.len() {
        sinker.sink_out_of(b);
    }
    sinker.into_plan()
}

/// The analysis of one function: each pass fills in fields from the ones before it.
struct Sinker<'a> {
    func: &'a Func,
    defs: &'a [u32],
    read_only: &'a dyn Fn(GlobalId) -> bool,
    nslots: usize,
    /// Where each singly-defined value is defined.
    def_site: Vec<Option<(u32, u32)>>,
    /// Every use of every value.
    sites: Vec<Vec<(u32, u32, Role)>>,
    /// The slot each address value points into: the result of `SlotAddr`, or of a pointer
    /// addition from one.
    owner: Vec<Option<u32>>,
    tracked: Vec<bool>,
    mentioned: Vec<bool>,
    /// The address values of each slot.
    members: Vec<Vec<Val>>,
    /// Slots something reads (or that escape): the rest are only written.
    live: Vec<bool>,
    skip: Vec<Vec<bool>>,
    deleted: Vec<(u32, u32)>,
    /// The number of predecessors of each block.
    preds: Vec<u32>,
    /// The block all of a slot's address values are defined in.
    home: Vec<Option<u32>>,
    split: Vec<bool>,
    front: Vec<Vec<(u32, u32)>>,
}

impl<'a> Sinker<'a> {
    fn new(func: &'a Func, defs: &'a [u32], read_only: &'a dyn Fn(GlobalId) -> bool) -> Self {
        let n = func.vals.len();
        let nslots = func.slots.len();
        let nblocks = func.blocks.len();
        Self {
            func,
            defs,
            read_only,
            nslots,
            def_site: vec![None; n],
            sites: vec![Vec::new(); n],
            owner: vec![None; n],
            tracked: vec![true; nslots],
            mentioned: vec![false; nslots],
            members: vec![Vec::new(); nslots],
            live: vec![false; nslots],
            skip: func
                .blocks
                .iter()
                .map(|b| vec![false; b.insts.len()])
                .collect(),
            deleted: Vec::new(),
            preds: vec![0u32; nblocks],
            home: vec![None; nslots],
            split: vec![false; nslots],
            front: vec![Vec::new(); nblocks],
        }
    }

    fn single(&self, v: Val) -> bool {
        self.defs[v.0 as usize] == 1
    }

    fn inst_at(&self, b: u32, i: u32) -> &'a Inst {
        &self.func.blocks[b as usize].insts[i as usize]
    }

    fn in_set(&self, set: &[bool], v: Val) -> bool {
        self.owner[v.0 as usize].is_some_and(|s| set[s as usize])
    }

    fn is_dead(&self, v: Val) -> bool {
        self.owner[v.0 as usize].is_some_and(|s| !self.live[s as usize])
    }

    /// Where each singly-defined value is defined, and every use of every value.
    fn collect_sites(&mut self) {
        let func = self.func;
        for (b, block) in func.blocks.iter().enumerate() {
            for (i, inst) in block.insts.iter().enumerate() {
                inst_vals(
                    inst,
                    &mut |v| {
                        if self.defs[v.0 as usize] == 1 {
                            self.def_site[v.0 as usize] = Some((b as u32, i as u32));
                        }
                    },
                    &mut |_| {},
                );
                roles(inst, &mut |v, role| {
                    self.sites[v.0 as usize].push((b as u32, i as u32, role));
                });
            }
            term_uses(&block.term, &mut |v| {
                self.sites[v.0 as usize].push((b as u32, TERM, Role::Other));
            });
        }
    }

    /// Finds the slot each address value points into.
    fn assign_owners(&mut self) {
        let func = self.func;
        for block in &func.blocks {
            for inst in &block.insts {
                if let Inst::SlotAddr {
                    dst,
                    slot,
                } = inst
                {
                    self.mentioned[slot.0 as usize] = true;
                    if self.single(*dst) {
                        self.owner[dst.0 as usize] = Some(slot.0);
                    } else {
                        self.tracked[slot.0 as usize] = false;
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
                        && self.single(*dst)
                        && self.owner[dst.0 as usize].is_none()
                        && let Some(s) = self.owner[base.0 as usize]
                    {
                        self.owner[dst.0 as usize] = Some(s);
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        for (v, o) in self.owner.iter().enumerate() {
            if let Some(s) = *o {
                self.members[s as usize].push(Val(v as u32));
            }
        }
    }

    /// Whether something reads slot `s`, given the slots found live so far.
    fn reads_slot(&self, s: usize) -> bool {
        self.members[s].iter().any(|&v| {
            self.sites[v.0 as usize]
                .iter()
                .any(|&(b, i, role)| match role {
                    Role::Write => false,
                    Role::Derive => match self.inst_at(b, i) {
                        Inst::PtrAdd {
                            dst, ..
                        } => self.owner[dst.0 as usize] != Some(s as u32),
                        _ => true,
                    },
                    Role::Read => match (i != TERM).then(|| self.inst_at(b, i)) {
                        Some(Inst::Copy {
                            dst, ..
                        }) => match self.owner[dst.0 as usize] {
                            Some(t) => t as usize == s || self.live[t as usize],
                            None => true,
                        },
                        _ => true,
                    },
                    Role::Value | Role::Other => true,
                })
        })
    }

    /// Slots something reads (or that escape): the rest are only written.
    fn find_live_slots(&mut self) {
        for s in 0..self.nslots {
            self.live[s] = !self.tracked[s] || !self.mentioned[s];
        }
        loop {
            let mut changed = false;
            for s in 0..self.nslots {
                if !self.live[s] && self.reads_slot(s) {
                    self.live[s] = true;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }

    /// Deletes the instructions that only fill slots nothing reads.
    fn delete_dead(&mut self) {
        let func = self.func;
        for (b, block) in func.blocks.iter().enumerate() {
            for (i, inst) in block.insts.iter().enumerate() {
                let dead = match inst {
                    Inst::SlotAddr {
                        dst, ..
                    }
                    | Inst::PtrAdd {
                        dst, ..
                    } => self.is_dead(*dst),
                    Inst::Store {
                        addr, ..
                    } => self.is_dead(*addr),
                    Inst::Copy {
                        dst, ..
                    }
                    | Inst::Zero {
                        dst, ..
                    } => self.is_dead(*dst),
                    _ => false,
                };
                if dead {
                    self.skip[b][i] = true;
                    self.deleted.push((b as u32, i as u32));
                }
            }
        }
    }

    /// The predecessor counts, and the block each slot's address values are defined in.
    fn find_homes(&mut self) {
        let func = self.func;
        if let Some(entry) = self.preds.first_mut() {
            *entry = 1;
        }
        for block in &func.blocks {
            for next in block.term.successors() {
                self.preds[next.0 as usize] += 1;
            }
        }
        for (s, list) in self.members.iter().enumerate() {
            for v in list {
                let Some((b, _)) = self.def_site[v.0 as usize] else {
                    self.split[s] = true;
                    continue;
                };
                match self.home[s] {
                    None => self.home[s] = Some(b),
                    Some(h) if h != b => self.split[s] = true,
                    _ => {}
                }
            }
        }
    }

    /// Moves what block `b` does for a slot that only one successor reads into that successor.
    fn sink_out_of(&mut self, b: usize) {
        let ir::Term::Branch {
            then_block,
            else_block,
            ..
        } = self.func.blocks[b].term
        else {
            return;
        };
        if then_block == else_block {
            return;
        }
        for x in [then_block.0 as usize, else_block.0 as usize] {
            if x == b || self.preds[x] != 1 {
                continue;
            }
            let set = self.agreeing_slots(b, x);
            if !set.iter().any(|&s| s) {
                continue;
            }
            let Some(all) = self.movable(b, x, &set) else {
                continue;
            };
            for &i in &all {
                self.skip[b][i as usize] = true;
                self.front[x].push((b as u32, i));
            }
        }
    }

    /// Whether the use of a slot's value at `(sb, i)` fits moving the slots in `set` from `b`
    /// into `x`.
    fn site_agrees(
        &self,
        set: &[bool],
        (b, x): (usize, usize),
        (sb, i): (u32, u32),
        role: Role,
    ) -> bool {
        if sb as usize == x {
            return role != Role::Value;
        }
        if sb as usize != b || i == TERM || self.skip[sb as usize][i as usize] {
            return sb as usize == b && i != TERM;
        }
        match (role, self.inst_at(sb, i)) {
            (Role::Derive, _) => true,
            (
                Role::Read,
                Inst::Copy {
                    dst, ..
                },
            ) => self.in_set(set, *dst),
            (
                Role::Write,
                Inst::Store {
                    value, ..
                },
            ) => {
                matches!(
                    self.def_site[value.0 as usize].map(|(db, di)| self.inst_at(db, di)),
                    Some(Inst::IConst { .. } | Inst::FConst { .. })
                ) && self.def_site[value.0 as usize].is_some_and(|(db, _)| db as usize == b)
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
                self.in_set(set, *src)
                    || self.def_site[src.0 as usize].is_some_and(|(db, di)| {
                        db as usize == b
                            && matches!(
                                self.inst_at(db, di),
                                Inst::GlobalAddr { global, .. }
                                    if (self.read_only)(*global)
                            )
                    })
            }
            _ => false,
        }
    }

    /// The slots of block `b` that can move into `x`: live, whole in `b`, and used the way
    /// moving allows.
    fn agreeing_slots(&self, b: usize, x: usize) -> Vec<bool> {
        let mut set: Vec<bool> = (0..self.nslots)
            .map(|s| self.live[s] && !self.split[s] && self.home[s] == Some(b as u32))
            .collect();
        // Drop slots that something else reads or sets, until the rest agree.
        loop {
            let mut changed = false;
            for s in 0..self.nslots {
                if !set[s] {
                    continue;
                }
                let ok = self.members[s].iter().all(|&v| {
                    self.sites[v.0 as usize]
                        .iter()
                        .all(|&(sb, i, role)| self.site_agrees(&set, (b, x), (sb, i), role))
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
        set
    }

    /// The instructions of `b` that fill the slots in `set`, and the constants only they use;
    /// none if one of them takes a value that is not safe to recompute in `x`.
    fn movable(&self, b: usize, x: usize, set: &[bool]) -> Option<Vec<u32>> {
        let mut moved: Vec<u32> = Vec::new();
        for (i, inst) in self.func.blocks[b].insts.iter().enumerate() {
            if self.skip[b][i] {
                continue;
            }
            let takes = match inst {
                Inst::SlotAddr {
                    dst, ..
                }
                | Inst::PtrAdd {
                    dst, ..
                } => self.in_set(set, *dst),
                Inst::Store {
                    addr, ..
                } => self.in_set(set, *addr),
                Inst::Copy {
                    dst, ..
                }
                | Inst::Zero {
                    dst, ..
                } => self.in_set(set, *dst),
                _ => false,
            };
            if takes {
                moved.push(i as u32);
            }
        }
        let mut helpers: Vec<u32> = Vec::new();
        let mut sound = true;
        for &i in &moved {
            inst_vals(self.inst_at(b as u32, i), &mut |_| {}, &mut |v| {
                if self.in_set(set, v) {
                    return;
                }
                if !self.single(v) {
                    sound = false;
                    return;
                }
                let Some((db, di)) = self.def_site[v.0 as usize] else {
                    return;
                };
                if db as usize != b {
                    return;
                }
                if moved.contains(&di) || helpers.contains(&di) {
                    return;
                }
                let leaf = matches!(
                    self.inst_at(db, di),
                    Inst::IConst { .. } | Inst::FConst { .. } | Inst::GlobalAddr { .. }
                );
                let only_here = self.sites[v.0 as usize].iter().all(|&(ub, ui, _)| {
                    (ub as usize == x) || (ub as usize == b && ui != TERM && moved.contains(&ui))
                });
                if leaf && only_here && di < i {
                    helpers.push(di);
                }
            });
        }
        if !sound {
            return None;
        }
        let mut all: Vec<u32> = moved.into_iter().chain(helpers).collect();
        all.sort_unstable();
        all.dedup();
        Some(all)
    }

    fn into_plan(self) -> Option<Plan> {
        let any = !self.deleted.is_empty() || self.front.iter().any(|f| !f.is_empty());
        any.then_some(Plan {
            skip: self.skip,
            front: self.front,
            deleted: self.deleted,
        })
    }
}
