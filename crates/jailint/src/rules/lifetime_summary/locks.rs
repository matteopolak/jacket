//! Ordered, single-receiver recursive lock transfers.
//! Entry requirements distinguish release/acquire from acquire/release even when
//! both have zero net depth. Distinct paths may alias and remain unknown.
use super::{Path, Root};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::rules) struct Balance {
    pub required: u8,
    pub delta: i16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::rules) struct Transfer {
    pub target: Option<std::rc::Rc<Path>>,
    pub(super) object: Option<super::address_objects::Object>,
    pub balance: Balance,
}

impl Transfer {
    pub(super) fn primitive(acquire: bool) -> Self {
        Self {
            target: Some(std::rc::Rc::new(Path {
                root: Root::Parameter(0),
                steps: Vec::new(),
            })),
            object: Some(super::address_objects::Object::input(0)),
            balance: if acquire {
                Balance {
                    required: 0,
                    delta: 1,
                }
            } else {
                Balance {
                    required: 1,
                    delta: -1,
                }
            },
        }
    }
}

impl Transfer {
    fn then(&self, next: &Self) -> Option<Self> {
        let target = match (&self.target, &next.target) {
            (Some(a), Some(b)) if a != b => return None,
            (Some(a), _) | (_, Some(a)) => Some(a.clone()),
            (None, None) => None,
        };
        let object = match (self.object, next.object) {
            (Some(a), Some(b)) if a != b => return None,
            (Some(a), _) | (_, Some(a)) => Some(a),
            (None, None) => None,
        };
        let delta = self.balance.delta.checked_add(next.balance.delta)?;
        let required = i16::from(self.balance.required)
            .max(i16::from(next.balance.required).checked_sub(self.balance.delta)?);
        // Exact recursive depth is deliberately bounded. Exceeding the domain is
        // uncertainty, never a one-bit approximation or a saturating guarantee.
        if delta.abs() > 32 || required > 32 {
            return None;
        }
        Some(Self {
            target,
            object,
            balance: Balance {
                required: required.try_into().ok()?,
                delta,
            },
        })
    }
}

impl super::Resolver<'_> {
    fn lock_call(
        &self,
        event: &super::Event,
        site: crate::rules::lifetime_movement::facts::Site,
        all: &super::HashMap<super::ProcId, super::Summary>,
    ) -> Option<Transfer> {
        let (super::Event::Call {
            args, ..
        }
        | super::Event::IndirectCall {
            args, ..
        }) = event
        else {
            return Some(Transfer::default());
        };
        let _storage = self.storage_query();
        let targets = self.call_targets(event);
        if targets.unknown || targets.procedures.is_empty() {
            return None;
        }
        let mut chosen = None;
        for id in targets.ordered() {
            let mut transfer = all.get(&id)?.locks.clone()?;
            if let Some(object) = transfer.object {
                transfer.object = Some(self.instantiate_lock_object(object, args, site, all)?);
            }
            if let Some(path) = &transfer.target {
                transfer.target = Some(std::rc::Rc::new(self.substitute(
                    path,
                    args,
                    all,
                    &mut super::HashSet::default(),
                    0,
                )?));
            }
            if chosen.as_ref().is_some_and(|prior| prior != &transfer) {
                return None;
            }
            chosen = Some(transfer);
        }
        chosen
    }

    pub(super) fn infer_locks(
        &self,
        all: &super::HashMap<super::ProcId, super::Summary>,
    ) -> Option<Transfer> {
        use super::{Event, HashMap};
        use std::collections::VecDeque;
        let live = super::returning_blocks(self.body, all, self.targets);
        if live.is_empty() {
            return None;
        }
        // An absent block state is unreachable; a present None is reachable but
        // unknown. Exact joins preserve both required entry depth and net delta.
        let mut states: HashMap<usize, Option<Transfer>> = HashMap::default();
        states.insert(0, Some(Transfer::default()));
        let mut queue = VecDeque::from([0]);
        let mut result: Option<Option<Transfer>> = None;
        while let Some(index) = queue.pop_front() {
            if jaic::interrupt::should_stop() {
                return None;
            }
            if !live.contains(index) {
                continue;
            }
            let mut state = states[&index].clone();
            for (ordinal, event) in self.body.blocks[index].events.iter().enumerate() {
                match event {
                    Event::Barrier
                    | Event::Grow {
                        ..
                    } => state = None,
                    Event::Call {
                        expands: false, ..
                    }
                    | Event::IndirectCall {
                        ..
                    } => {
                        state = state.and_then(|prior| {
                            prior.then(&self.lock_call(
                                event,
                                crate::rules::lifetime_movement::facts::Site {
                                    block: index,
                                    event: ordinal,
                                },
                                all,
                            )?)
                        });
                    }
                    Event::Store {
                        place,
                        ty,
                        ..
                    }
                    | Event::Initialize {
                        place,
                        ty,
                        ..
                    } if self
                        .places
                        .existing(*place, *ty)
                        .is_none_or(|p| self.places.unresolved(p)) =>
                    {
                        state = None;
                    }
                    _ => {}
                }
            }
            let block = &self.body.blocks[index];
            if block.normal_return {
                match &mut result {
                    Some(prior) if *prior != state => *prior = None,
                    None => result = Some(state.clone()),
                    _ => {}
                }
            }
            for &next in &block.successors {
                if !live.contains(next) {
                    continue;
                }
                let changed = if let Some(prior) = states.get_mut(&next) {
                    if prior.is_some() && *prior != state {
                        *prior = None;
                        true
                    } else {
                        false
                    }
                } else {
                    states.insert(next, state.clone());
                    true
                };
                if changed {
                    queue.push_back(next);
                }
            }
        }
        let mut result = result??;
        if result.balance == Balance::default() {
            result.target = None;
            result.object = None;
        }
        if result
            .target
            .as_ref()
            .is_some_and(|p| !matches!(p.root, Root::Parameter(_)))
        {
            return None;
        }
        Some(result)
    }
}

/// Authenticate the foreign ABI boundary before assigning protocol semantics.
pub(super) fn native(compiler: &jaic::sema::Compiler, id: super::ProcId) -> Option<Transfer> {
    if compiler.options.os != jaic::target::TargetOs::Windows {
        return None;
    }
    let signature = compiler.pr.procs[id.0 as usize].sig.as_ref()?;
    if !signature.c_call
        || signature.c_varargs
        || signature.params.iter().any(|p| p.variadic)
        || signature.params.len() != 1
        || !compiler.ty.types.is_pointer(signature.params[0].ty)
        || !signature.returns.is_empty()
    {
        return None;
    }
    let jaic::sema::procs::ProcTarget::Foreign(foreign) = compiler.proc(id).target? else {
        return None;
    };
    let foreign = &compiler.out.program.foreigns[foreign.0 as usize];
    let library = &compiler.out.program.libraries[foreign.library?];
    if foreign.is_data || !library.system || !library.name.eq_ignore_ascii_case("kernel32") {
        return None;
    }
    match foreign.symbol.as_str() {
        "EnterCriticalSection" => Some(Transfer::primitive(true)),
        "LeaveCriticalSection" => Some(Transfer::primitive(false)),
        _ => None,
    }
}

/// An explicit indexed obligation also supplies intentional wrapper exemption.
pub(super) fn declared(
    compiler: &jaic::sema::Compiler,
    id: super::ProcId,
    name: &str,
) -> Option<usize> {
    let signature = compiler.proc(id).sig.as_ref()?;
    super::super::lifetime_movement::notes(compiler, id).find_map(|note| {
        let parameters = super::super::lifetime_movement::parameters(&note.text, name)?;
        let formal = match parameters.as_slice() {
            [] => 0,
            [number] => number.parse().ok()?,
            _ => return None,
        };
        signature
            .params
            .get(formal)
            .filter(|p| !p.discard)
            .map(|_| formal)
    })
}
