//! Observable process effects: `@Exits` and `@WritesOutput` on bodiless declarations, and the calls that reach them.
use jaic::sema::Compiler;

use super::{Event, HashMap, HashSet, LifetimeBody, ProcId, Summary, Val};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::rules) struct Observable {
    pub output: bool,
    pub exit: bool,
    /// The formals that reach a `@WritesOutput` argument, as bits (bit `k` is formal `k`): the
    /// procedure writes to the standard output or error when a caller passes one there, as a
    /// wrapper around `write` does.
    pub output_through: u64,
}

impl Observable {
    /// Adds a callee's output and exit; its `output_through` names the callee's formals, so a
    /// caller reads it per call instead.
    pub fn include(&mut self, other: Self) {
        self.output |= other.output;
        self.exit |= other.exit;
    }

    /// The formals in `output_through`.
    fn output_formals(self) -> impl Iterator<Item = usize> {
        (0..64).filter(move |k| self.output_through & (1 << k) != 0)
    }
}

/// The argument a procedure writes through when it is the standard output or error: `@WritesOutput(0)`.
fn output_formal(c: &Compiler, p: ProcId) -> Option<usize> {
    super::lifetime_movement::notes(c, p).find_map(|n| {
        super::lifetime_movement::parameters(&n.text, "WritesOutput")?
            .first()?
            .parse()
            .ok()
    })
}

pub(super) struct Calls<'a> {
    compiler: &'a Compiler,
    body: &'a LifetimeBody,
    /// The calls reached before a call that does not return, with their arguments.
    sites: Vec<(ProcId, &'a [super::LifetimeArgument])>,
    callees: Vec<ProcId>,
    values: std::cell::OnceCell<Values<'a>>,
}

impl<'a> Calls<'a> {
    pub fn build(c: &'a Compiler, body: &'a LifetimeBody, all: &HashMap<ProcId, Summary>) -> Self {
        let mut out = Self {
            compiler: c,
            body,
            sites: Vec::new(),
            callees: Vec::new(),
            values: std::cell::OnceCell::new(),
        };

        if body.blocks.is_empty() {
            return out;
        }

        let mut queue = vec![0];
        let mut visited = HashSet::default();

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
                    out.sites.push((*callee, args));

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
        let mut out = Observable::default();

        for &(callee, args) in &self.sites {
            let through = all.get(&callee).map(|s| s.observable).unwrap_or_default();

            for formal in output_formal(self.compiler, callee)
                .into_iter()
                .chain(through.output_formals())
            {
                let values = self.values.get_or_init(|| Values::new(self.body));

                match values.output(self.compiler, formal, args) {
                    Reaches::Output => out.output = true,
                    Reaches::Formal(k) if k < 64 => out.output_through |= 1 << k,
                    Reaches::Formal(_) | Reaches::Neither => {}
                }
            }
        }

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
    /// The value the caller passed as this formal.
    Formal(usize),
}

/// Where an argument passed to a `@WritesOutput` formal comes from.
enum Reaches {
    /// The standard output or error.
    Output,
    /// This procedure's own formal, so it writes output when its caller passes one.
    Formal(usize),
    Neither,
}

enum Def<'a> {
    Constant(u64),
    Copy(Val),
    Load(Val),
    Call(ProcId, &'a [super::LifetimeArgument]),
    Formal(usize),
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
                    incoming,
                    formal,
                    ..
                } => out
                    .defs
                    .entry(*incoming)
                    .or_default()
                    .push(Def::Formal(*formal)),
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

    fn output(&self, c: &Compiler, formal: usize, args: &[super::LifetimeArgument]) -> Reaches {
        let Some(arg) = args.iter().find(|a| a.formal == formal) else {
            return Reaches::Neither;
        };
        let actual = arg
            .constant
            .map(|v| Value::Numbers(vec![v]))
            .or_else(|| super::value(arg).and_then(|v| self.get(c, v, &mut HashSet::default(), 0)));

        match actual {
            Some(Value::Numbers(v)) if !v.is_empty() && v.iter().all(|v| matches!(v, 1 | 2)) => {
                Reaches::Output
            }
            Some(Value::OutputHandle) => Reaches::Output,
            Some(Value::Formal(k)) => Reaches::Formal(k),
            Some(Value::Numbers(_)) | None => Reaches::Neither,
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
                    Def::Formal(k) => Value::Formal(*k),
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
                    Def::Call(p, args)
                        if super::lifetime_movement::notes(c, *p)
                            .any(|n| n.is("ReturnsStandardHandle")) =>
                    {
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
                    Def::Call(..) => return None,
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
        (Some(Value::Formal(a)), Value::Formal(b)) if *a == b => {}
        _ => return None,
    }

    Some(())
}
