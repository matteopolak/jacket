//! Observable process effects: `@Exits` and `@WritesOutput` on bodiless declarations, and the calls that reach them.
use super::{Event, HashMap, HashSet, LifetimeBody, ProcId, Summary, Val};
use jaic::sema::{Compiler, procs::ProcTarget};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::rules) struct Observable {
    pub output: bool,
    pub exit: bool,
}

impl Observable {
    pub fn include(&mut self, other: Self) {
        self.output |= other.output;
        self.exit |= other.exit;
    }
}

pub(super) fn symbol(c: &Compiler, p: ProcId) -> Option<&str> {
    let ProcTarget::Foreign(id) = c.pr.procs[p.0 as usize].target? else {
        return None;
    };
    Some(&c.out.program.foreigns[id.0 as usize].symbol)
}

pub(super) struct Calls<'a> {
    body: &'a LifetimeBody,
    direct: Observable,
    callees: Vec<ProcId>,
}

impl<'a> Calls<'a> {
    pub fn build(c: &Compiler, body: &'a LifetimeBody, all: &HashMap<ProcId, Summary>) -> Self {
        let mut out = Self {
            body,
            direct: Observable::default(),
            callees: Vec::new(),
        };
        if body.blocks.is_empty() {
            return out;
        }
        let mut queue = vec![0];
        let mut visited = HashSet::default();
        let mut values = None;
        while let Some(i) = queue.pop() {
            if !visited.insert(i) {
                continue;
            }
            let mut stopped = false;
            for event in &body.blocks[i].events {
                if let Event::Call {
                    callee,
                    args,
                    expands: false,
                    ..
                } = event
                {
                    out.callees.push(*callee);
                    if matches!(symbol(c, *callee), Some("write" | "_write" | "WriteFile")) {
                        let domain = values.get_or_insert_with(|| Values::new(body));
                        out.direct.output |= domain.output(c, *callee, args);
                    }
                    if all
                        .get(callee)
                        .is_some_and(|s| s.normal_return == Some(false))
                    {
                        stopped = true;
                        break;
                    }
                }
            }
            if !stopped {
                queue.extend(&body.blocks[i].successors);
            }
        }
        out.callees.sort_unstable_by_key(|p| p.0);
        out.callees.dedup();
        out
    }

    fn always_exits(&self, all: &HashMap<ProcId, Summary>) -> bool {
        let mut proved = vec![false; self.body.blocks.len()];
        let mut remaining = vec![usize::MAX; self.body.blocks.len()];
        let mut predecessors = vec![Vec::new(); self.body.blocks.len()];
        let mut queue = std::collections::VecDeque::new();
        for (i, block) in self.body.blocks.iter().enumerate() {
            let mut stopped = false;
            for event in &block.events {
                if let Event::Call {
                    callee,
                    expands: false,
                    ..
                } = event
                    && let Some(summary) = all.get(callee)
                {
                    if summary.observable.exit {
                        proved[i] = true;
                        queue.push_back(i);
                        break;
                    }
                    if summary.normal_return == Some(false) {
                        stopped = true;
                        break;
                    }
                }
            }
            if !proved[i] && !stopped && !block.normal_return && !block.successors.is_empty() {
                remaining[i] = block.successors.len();
                for &next in &block.successors {
                    predecessors[next].push(i);
                }
            }
        }
        while let Some(i) = queue.pop_front() {
            for &previous in &predecessors[i] {
                if proved[previous] {
                    continue;
                }
                remaining[previous] -= 1;
                if remaining[previous] == 0 {
                    proved[previous] = true;
                    queue.push_back(previous);
                }
            }
        }
        proved.first().copied().unwrap_or(false)
    }

    pub fn infer(&self, all: &HashMap<ProcId, Summary>) -> Observable {
        let mut out = self.direct;
        for p in &self.callees {
            if let Some(s) = all.get(p) {
                out.include(s.observable);
            }
        }
        // An error path that aborts does not make an ordinary printing call an
        // exit operation. Every path must cross an exit, including divergent paths.
        out.exit = out.exit && self.always_exits(all);
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Value {
    Numbers(Vec<u64>),
    OutputHandle,
}

enum Def<'a> {
    Constant(u64),
    Copy(Val),
    Load(Val),
    Call(ProcId, &'a [super::LifetimeArgument]),
    Unknown,
}

struct Values<'a> {
    defs: HashMap<Val, Vec<Def<'a>>>,
    stores: HashMap<Val, Vec<Val>>,
    exposed: HashSet<Val>,
}

impl<'a> Values<'a> {
    fn new(body: &'a LifetimeBody) -> Self {
        let mut out = Self {
            defs: HashMap::default(),
            stores: HashMap::default(),
            exposed: HashSet::default(),
        };
        for e in body.blocks.iter().flat_map(|b| &b.events) {
            match e {
                Event::Parameter {
                    incoming, ..
                } => out.defs.entry(*incoming).or_default().push(Def::Unknown),
                Event::Store {
                    place,
                    value,
                    constant,
                    ..
                } => {
                    out.stores.entry(*place).or_default().push(*value);
                    if let Some(n) = constant {
                        out.defs.entry(*value).or_default().push(Def::Constant(*n));
                    }
                }
                Event::Load {
                    place,
                    value,
                    ..
                } if place != value => out.defs.entry(*value).or_default().push(Def::Load(*place)),
                Event::Copy {
                    value,
                    source,
                } => out.defs.entry(*value).or_default().push(Def::Copy(*source)),
                Event::Address {
                    place, ..
                } => {
                    out.exposed.insert(*place);
                }
                Event::Call {
                    callee,
                    args,
                    results,
                    expands: false,
                    ..
                } => {
                    for &(_, v, _) in results {
                        out.defs
                            .entry(v)
                            .or_default()
                            .push(Def::Call(*callee, args));
                    }
                }
                _ => {}
            }
        }
        out
    }

    fn output(&self, c: &Compiler, callee: ProcId, args: &[super::LifetimeArgument]) -> bool {
        let Some(arg) = args.iter().find(|a| a.formal == 0) else {
            return false;
        };
        let actual = arg
            .constant
            .map(|v| Value::Numbers(vec![v]))
            .or_else(|| super::value(arg).and_then(|v| self.get(c, v, &mut HashSet::default(), 0)));
        match (symbol(c, callee), actual) {
            (Some("write" | "_write"), Some(Value::Numbers(v))) => {
                !v.is_empty() && v.iter().all(|v| matches!(v, 1 | 2))
            }
            (Some("WriteFile"), Some(Value::OutputHandle)) => true,
            _ => false,
        }
    }

    fn get(&self, c: &Compiler, v: Val, seen: &mut HashSet<Val>, depth: usize) -> Option<Value> {
        if depth > 24 || !seen.insert(v) {
            return None;
        }
        let result = (|| {
            let mut result = None;
            for d in self.defs.get(&v)? {
                let selected = match d {
                    Def::Constant(n) => Value::Numbers(vec![*n]),
                    Def::Copy(v) => self.get(c, *v, seen, depth + 1)?,
                    Def::Load(p) => {
                        if self.exposed.contains(p) {
                            return None;
                        }
                        let mut value = None;
                        for v in self.stores.get(p)? {
                            merge(&mut value, self.get(c, *v, seen, depth + 1)?)?;
                        }
                        value?
                    }
                    Def::Call(p, args) if symbol(c, *p) == Some("GetStdHandle") => {
                        let arg = args.iter().find(|a| a.formal == 0)?;
                        let actual =
                            arg.constant.map(|v| Value::Numbers(vec![v])).or_else(|| {
                                super::value(arg).and_then(|v| self.get(c, v, seen, depth + 1))
                            })?;
                        let Value::Numbers(numbers) = actual else {
                            return None;
                        };
                        if numbers.is_empty()
                            || !numbers.iter().all(|v| matches!(*v as i32, -11 | -12))
                        {
                            return None;
                        }
                        Value::OutputHandle
                    }
                    _ => return None,
                };
                merge(&mut result, selected)?;
            }
            result
        })();
        seen.remove(&v);
        result
    }
}

fn merge(into: &mut Option<Value>, value: Value) -> Option<()> {
    match (into.as_mut(), value) {
        (None, v) => *into = Some(v),
        (Some(Value::Numbers(a)), Value::Numbers(b)) => {
            a.extend(b);
            a.sort_unstable();
            a.dedup();
            if a.len() > 8 {
                return None;
            }
        }
        (Some(Value::OutputHandle), Value::OutputHandle) => {}
        _ => return None,
    }
    Some(())
}
