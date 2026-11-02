//! Finite selected procedure values, with explicit open-world sources. Return sets preserve
//! formal procedure parameters so an identity wrapper does not erase a closed caller set.
use super::lifetime_places::Places;
use jaic::fxhash::{HashMap, HashSet};
use jaic::ide::lifetimes::{LifetimeBody, LifetimeEvent as Event, LifetimeOperand};
use jaic::ir::Val;
use jaic::sema::{Compiler, ProcId};
use jaic::types::{TypeId, TypeKind};
use std::collections::VecDeque;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Set {
    pub procedures: HashSet<ProcId>,
    pub unknown: bool,
    pub(super) formals: HashSet<usize>,
}

impl Set {
    fn join(&mut self, other: &Self) -> bool {
        let before = (self.procedures.len(), self.formals.len(), self.unknown);
        self.procedures.extend(&other.procedures);
        self.formals.extend(&other.formals);
        self.unknown |= other.unknown;
        before != (self.procedures.len(), self.formals.len(), self.unknown)
    }

    pub fn open(mut self) -> Self {
        self.unknown |= !self.formals.is_empty() || self.procedures.is_empty();
        self.formals.clear();
        self
    }

    pub fn ordered(&self) -> Vec<ProcId> {
        let mut ids: Vec<_> = self.procedures.iter().copied().collect();
        ids.sort_unstable_by_key(|p| p.0);
        ids
    }
}

#[derive(Default)]
pub(super) struct Universe {
    bodies: HashMap<ProcId, Local>,
}

#[derive(Default)]
struct Local {
    values: HashMap<Val, Set>,
    returns: HashMap<usize, Set>,
}

impl Universe {
    pub fn build(compiler: &Compiler, valid: &HashMap<ProcId, &LifetimeBody>) -> Self {
        let graphs: HashMap<_, _> = valid
            .iter()
            .filter_map(|(&id, body)| Graph::new(compiler, body).map(|g| (id, g)))
            .collect();
        let mut this = Self::default();
        let mut ids: Vec<_> = graphs.keys().copied().collect();
        ids.sort_unstable_by_key(|id| id.0);
        loop {
            let mut changed = false;
            for &id in &ids {
                let next = graphs[&id].solve(&this, valid);
                let old = this.bodies.entry(id).or_default();
                for (v, set) in next.values {
                    changed |= old.values.entry(v).or_default().join(&set);
                }
                for (v, set) in next.returns {
                    changed |= old.returns.entry(v).or_default().join(&set);
                }
            }
            if !changed {
                break;
            }
        }
        this
    }

    pub fn get(&self, body: ProcId, value: Val) -> Set {
        self.bodies
            .get(&body)
            .and_then(|b| b.values.get(&value))
            .cloned()
            .unwrap_or_default()
            .open()
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum Cell {
    Value(Val),
    Place(usize),
}

struct Call {
    destination: usize,
    callee: Callee,
    arguments: HashMap<usize, usize>,
    result: usize,
}

enum Callee {
    Direct(ProcId),
    Indirect(usize),
}

#[derive(Default)]
struct Graph {
    ids: HashMap<Cell, usize>,
    seeds: Vec<Set>,
    defined: Vec<bool>,
    forward: Vec<Vec<usize>>,
    calls: Vec<Call>,
    returns: Vec<(usize, usize)>,
}

fn procedure(compiler: &Compiler, ty: TypeId) -> bool {
    matches!(compiler.ty.types.kind(ty), TypeKind::Proc(_))
}

impl Graph {
    fn id(&mut self, cell: Cell) -> usize {
        if let Some(&id) = self.ids.get(&cell) {
            return id;
        }
        let id = self.seeds.len();
        self.ids.insert(cell, id);
        self.seeds.push(Set::default());
        self.defined.push(false);
        self.forward.push(Vec::new());
        id
    }

    fn edge(&mut self, destination: Cell, source: Cell) {
        let a = self.id(destination);
        let b = self.id(source);
        self.forward[b].push(a);
        self.defined[a] = true;
    }

    fn new(compiler: &Compiler, body: &LifetimeBody) -> Option<Self> {
        let events: Vec<_> = body.blocks.iter().flat_map(|b| &b.events).collect();
        if !events.iter().any(|e| match e {
            Event::Procedure {
                ..
            }
            | Event::IndirectCall {
                allocator: false, ..
            } => true,
            Event::Parameter {
                ty, ..
            }
            | Event::Load {
                ty, ..
            }
            | Event::Store {
                ty, ..
            }
            | Event::Return {
                ty, ..
            } => procedure(compiler, *ty),
            Event::Call {
                results, ..
            } => results.iter().any(|(_, _, ty)| procedure(compiler, *ty)),
            _ => false,
        }) {
            return None;
        }
        let mut places = Places::new(compiler, body);
        let owned: HashSet<_> = body.stack_places.iter().copied().collect();
        let exposed: HashSet<_> = events
            .iter()
            .filter_map(|e| match e {
                Event::Address {
                    place,
                    ty,
                    ..
                } => Some(places.place(*place, *ty)),
                _ => None,
            })
            .collect();
        let mut graph = Self::default();
        for e in &events {
            match *e {
                Event::Procedure {
                    value,
                    callee,
                    ..
                } => {
                    let n = graph.id(Cell::Value(*value));
                    graph.seeds[n].procedures.insert(*callee);
                    graph.defined[n] = true;
                }
                Event::Parameter {
                    incoming,
                    formal,
                    ty,
                    ..
                } if procedure(compiler, *ty) => {
                    let n = graph.id(Cell::Value(*incoming));
                    graph.seeds[n].formals.insert(*formal);
                    graph.defined[n] = true;
                }
                Event::Store {
                    place,
                    value,
                    ty,
                    ..
                } if procedure(compiler, *ty) => {
                    graph.edge(Cell::Place(places.place(*place, *ty)), Cell::Value(*value));
                }
                Event::Load {
                    place,
                    value,
                    ty,
                    ..
                } if procedure(compiler, *ty) => {
                    let p = places.place(*place, *ty);
                    graph.edge(Cell::Value(*value), Cell::Place(p));
                    if !owned.contains(&places.paths[p].0.0)
                        || places.unresolved(p)
                        || exposed.contains(&p)
                        || places.ancestors(p).any(|a| exposed.contains(&a))
                    {
                        let n = graph.id(Cell::Place(p));
                        graph.seeds[n].unknown = true;
                    }
                }
                Event::Call {
                    callee,
                    args,
                    results,
                    expands: false,
                    ..
                } => graph.call(compiler, Callee::Direct(*callee), args, results),
                Event::IndirectCall {
                    callee,
                    args,
                    results,
                    allocator: false,
                    ..
                } => {
                    let n = graph.id(Cell::Value(*callee));
                    graph.call(compiler, Callee::Indirect(n), args, results);
                }
                Event::Return {
                    value,
                    index,
                    ty,
                    ..
                } if procedure(compiler, *ty) => {
                    let n = graph.id(Cell::Value(*value));
                    graph.returns.push((*index, n));
                }
                _ => {}
            }
        }
        // Copy edges may erase a procedure type (e.g. through *void) and recover it later.
        // Follow only the backwards closure of values needed by a typed operation.
        let copies: HashMap<_, Vec<_>> = events
            .iter()
            .filter_map(|e| match e {
                Event::Copy {
                    value,
                    source,
                } => Some((*value, *source)),
                _ => None,
            })
            .fold(HashMap::default(), |mut map, (v, source)| {
                map.entry(v).or_default().push(source);
                map
            });
        let mut queue: VecDeque<_> = graph
            .ids
            .keys()
            .filter_map(|k| match k {
                Cell::Value(v) => Some(*v),
                Cell::Place(_) => None,
            })
            .collect();
        let mut seen = HashSet::default();
        while let Some(v) = queue.pop_front() {
            if !seen.insert(v) {
                continue;
            }
            for &source in copies.get(&v).into_iter().flatten() {
                graph.edge(Cell::Value(v), Cell::Value(source));
                queue.push_back(source);
            }
        }
        for (n, defined) in graph.defined.iter().enumerate() {
            graph.seeds[n].unknown |= !defined;
        }
        Some(graph)
    }

    fn call(
        &mut self,
        compiler: &Compiler,
        callee: Callee,
        args: &[jaic::ide::lifetimes::LifetimeArgument],
        results: &[(usize, Val, TypeId)],
    ) {
        let args: HashMap<_, _> = args
            .iter()
            .filter(|a| procedure(compiler, a.ty))
            .filter_map(|a| match a.operand {
                LifetimeOperand::Value(v) | LifetimeOperand::Place(v) => {
                    Some((a.formal, self.id(Cell::Value(v))))
                }
                LifetimeOperand::Other => None,
            })
            .collect();
        for &(result, value, ty) in results {
            if !procedure(compiler, ty) {
                continue;
            }
            let destination = self.id(Cell::Value(value));
            self.defined[destination] = true;
            self.calls.push(Call {
                destination,
                callee: match callee {
                    Callee::Direct(p) => Callee::Direct(p),
                    Callee::Indirect(n) => Callee::Indirect(n),
                },
                arguments: args.clone(),
                result,
            });
        }
    }

    fn solve(&self, all: &Universe, valid: &HashMap<ProcId, &LifetimeBody>) -> Local {
        let mut values = self.seeds.clone();
        loop {
            let mut changed = false;
            for (from, tos) in self.forward.iter().enumerate() {
                let set = values[from].clone();
                for &to in tos {
                    changed |= values[to].join(&set);
                }
            }
            for call in &self.calls {
                let targets = match call.callee {
                    Callee::Direct(p) => Set {
                        procedures: HashSet::from_iter([p]),
                        ..Set::default()
                    },
                    Callee::Indirect(n) => values[n].clone(),
                };
                let mut result = Set {
                    unknown: targets.unknown || !targets.formals.is_empty(),
                    ..Set::default()
                };
                for proc in targets.procedures {
                    if !valid.contains_key(&proc) {
                        result.unknown = true;
                        continue;
                    }
                    let Some(set) = all
                        .bodies
                        .get(&proc)
                        .and_then(|b| b.returns.get(&call.result))
                    else {
                        continue;
                    };
                    result.procedures.extend(&set.procedures);
                    result.unknown |= set.unknown;
                    for formal in &set.formals {
                        if let Some(&n) = call.arguments.get(formal) {
                            result.join(&values[n]);
                        } else {
                            result.unknown = true;
                        }
                    }
                }
                changed |= values[call.destination].join(&result);
            }
            if !changed {
                break;
            }
        }
        let mut result = Local::default();
        for (cell, n) in &self.ids {
            if let Cell::Value(v) = cell {
                result.values.insert(*v, values[*n].clone());
            }
        }
        for &(index, n) in &self.returns {
            result.returns.entry(index).or_default().join(&values[n]);
        }
        result
    }
}
