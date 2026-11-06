//! Checked identities and original byte bounds, without domain effects or joins.
use jaic::fxhash::HashMap;
use jaic::ide::lifetimes::{LifetimeBody, LifetimeStackRoot};
use jaic::ir::Val;
use jaic::sema::{Compiler, ProcId, procs::BodyState};
use jaic::types::TypeId;

/// A checked instruction template, not a distinct dynamic invocation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct EventSite {
    pub procedure: ProcId,
    pub block: usize,
    pub event: usize,
}

/// A selected typed projection; the enclosing address still needs a certificate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct ByteSlot {
    pub offset: u64,
    pub width: u64,
    pub ty: TypeId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct Address {
    pub root: Val,
    pub offset: u64,
}

impl Address {
    /// Apply an inline byte displacement within an original certified extent.
    /// For example, offset four plus minus one addresses byte three.
    pub fn project(self, delta: i128, width: u64, extent: Option<u64>) -> Option<Self> {
        let offset = (self.offset as i128)
            .checked_add(delta)
            .and_then(|n| u64::try_from(n).ok())?;
        ByteRange {
            offset,
            width,
        }
        .within(extent)
        .then_some(Self {
            root: self.root,
            offset,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ByteRange {
    pub offset: u64,
    pub width: u64,
}

impl ByteRange {
    /// Check the whole range; zero bytes still require a known enclosing extent.
    pub fn within(self, extent: Option<u64>) -> bool {
        self.offset
            .checked_add(self.width)
            .zip(extent)
            .is_some_and(|(end, bytes)| end <= bytes)
    }

    /// Interval separation only: callers must first establish the same physical root.
    pub fn separated(self, other: Self) -> bool {
        let Some(end) = self.offset.checked_add(self.width) else {
            return false;
        };
        let Some(other_end) = other.offset.checked_add(other.width) else {
            return false;
        };
        end <= other.offset || other_end <= self.offset
    }
}

/// Validate retained-body identity before using any of its facts.
/// A bodyless native declaration needs its separate authenticated ABI model.
pub(super) fn current_body(c: &Compiler, key: ProcId, body: &LifetimeBody) -> bool {
    body.procedure == key
        && c.pr.procs.get(key.0 as usize).is_some_and(|p| {
            p.body_state == BodyState::Done && std::sync::Arc::ptr_eq(&p.lit, &body.syntax)
        })
}

/// Immutable access to the retained checked registry of one borrowed compiler.
/// A new compiler/checking mutation requires a new view; this is not a cache.
#[derive(Clone, Copy)]
pub(super) struct CheckedSession<'c> {
    compiler: &'c Compiler,
}

/// A body selected by its actual retained key, bound to the compiler owning its IDs.
#[derive(Clone, Copy)]
pub(super) struct CheckedBody<'c> {
    session: CheckedSession<'c>,
    body: &'c LifetimeBody,
}

impl<'c> CheckedSession<'c> {
    pub fn new(compiler: &'c Compiler) -> Self {
        Self {
            compiler,
        }
    }

    /// Select only an actual current registry entry, never a supplied body copy.
    pub fn body(self, key: ProcId) -> Option<CheckedBody<'c>> {
        let body = self.compiler.ed.facts()?.lifetime_bodies.get(&key)?;
        current_body(self.compiler, key, body).then_some(CheckedBody {
            session: self,
            body,
        })
    }
}
impl<'c> CheckedBody<'c> {
    pub fn compiler(self) -> &'c Compiler {
        self.session.compiler
    }
    pub fn body(self) -> &'c LifetimeBody {
        self.body
    }
}

/// An owned completion identity for conditional summaries used after inference.
#[derive(Clone)]
pub(super) struct BodyIdentity {
    procedure: ProcId,
    function: jaic::ir::FuncId,
    syntax: std::sync::Arc<jaic::ast::ProcLit>,
}

impl std::fmt::Debug for BodyIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BodyIdentity")
            .field("procedure", &self.procedure)
            .field("function", &self.function)
            .finish_non_exhaustive()
    }
}

impl PartialEq for BodyIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.procedure == other.procedure
            && self.function == other.function
            && std::sync::Arc::ptr_eq(&self.syntax, &other.syntax)
    }
}
impl Eq for BodyIdentity {
}

impl BodyIdentity {
    pub fn capture(c: &Compiler, body: &LifetimeBody) -> Option<Self> {
        if !current_body(c, body.procedure, body) {
            return None;
        }
        let jaic::sema::procs::ProcTarget::Func(function) =
            c.pr.procs.get(body.procedure.0 as usize)?.target?
        else {
            return None;
        };
        Some(Self {
            procedure: body.procedure,
            function,
            syntax: body.syntax.clone(),
        })
    }

    /// Summary key for current semantic constraints, independent of body identity.
    pub fn procedure(&self) -> ProcId {
        self.procedure
    }

    pub fn current(&self, c: &Compiler) -> bool {
        c.ed.facts().and_then(|f|f.lifetime_bodies.get(&self.procedure)).is_some_and(|body| {
            current_body(c,self.procedure,body) && std::sync::Arc::ptr_eq(&self.syntax,&body.syntax)
                && matches!(c.pr.procs[self.procedure.0 as usize].target,Some(jaic::sema::procs::ProcTarget::Func(function)) if function==self.function)
        })
    }
}

/// Immutable original allocations and canonical aliases. Borrowed output regions
/// belong to the domain and cannot extend or replace these original bounds.
#[derive(Clone, Default, PartialEq, Eq)]
pub(super) struct StackRoots {
    extents: HashMap<Val, Option<u64>>,
    aliases: HashMap<Val, Val>,
}

impl StackRoots {
    pub fn capture(body: &LifetimeBody, limit: usize) -> Option<Self> {
        let timer = super::lifetime_stats::Timer::start();
        let result = Self::from_parts(&body.stack_roots, &body.slot_aliases, limit)?;
        timer.finish(
            "stack_root_facts",
            Some(body.procedure),
            body.stack_roots.len() + body.slot_aliases.len(),
        );
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-root-facts proc={} roots={} aliases={} retained_bytes={}",
                body.procedure.0,
                result.extents.len(),
                result.aliases.len(),
                result.retained_bytes()
            );
        }
        Some(result)
    }

    fn from_parts(
        roots: &[LifetimeStackRoot],
        aliases: &[(Val, Val)],
        limit: usize,
    ) -> Option<Self> {
        if roots.len() > limit || aliases.len() > limit {
            return None;
        }
        let mut result = Self::default();
        for root in roots {
            if jaic::interrupt::should_stop() {
                return None;
            }
            if result
                .extents
                .insert(root.value, root.bytes)
                .is_some_and(|old| old != root.bytes)
            {
                return None;
            }
        }
        for &(alias, root) in aliases {
            if jaic::interrupt::should_stop() {
                return None;
            }
            if alias == root
                || !result.extents.contains_key(&root)
                || result.extents.contains_key(&alias)
                || result
                    .aliases
                    .insert(alias, root)
                    .is_some_and(|old| old != root)
            {
                return None;
            }
        }
        Some(result)
    }

    /// Keep presence distinct from usable extent: `Some(None)` is an actual
    /// allocation whose size is unknown and must not acquire a borrowed bound.
    pub fn extent(&self, root: Val) -> Option<Option<u64>> {
        self.extents.get(&root).copied()
    }

    pub fn canonical(&self, value: Val) -> Val {
        self.aliases.get(&value).copied().unwrap_or(value)
    }

    pub fn addresses(&self) -> HashMap<Val, Address> {
        self.extents
            .keys()
            .map(|&root| {
                (
                    root,
                    Address {
                        root,
                        offset: 0,
                    },
                )
            })
            .chain(self.aliases.iter().map(|(&alias, &root)| {
                (
                    alias,
                    Address {
                        root,
                        offset: 0,
                    },
                )
            }))
            .collect()
    }

    fn retained_bytes(&self) -> usize {
        self.extents.capacity() * std::mem::size_of::<(Val, Option<u64>)>()
            + self.aliases.capacity() * std::mem::size_of::<(Val, Val)>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_ranges_match_previous_scalar_and_element_checks() {
        for extent in [None, Some(0), Some(8), Some(u64::MAX)] {
            for offset in [0, 1, 8, u64::MAX] {
                for width in [0, 1, 8, u64::MAX] {
                    let old = offset
                        .checked_add(width)
                        .is_some_and(|end| extent.is_some_and(|bytes| end <= bytes));
                    assert_eq!(
                        ByteRange {
                            offset,
                            width
                        }
                        .within(extent),
                        old
                    );
                    for delta in [-9, -1, 0, 1, 9, i128::MAX] {
                        let old = (offset as i128)
                            .checked_add(delta)
                            .and_then(|n| u64::try_from(n).ok())
                            .filter(|&n| {
                                n.checked_add(width)
                                    .is_some_and(|end| extent.is_some_and(|bytes| end <= bytes))
                            });
                        assert_eq!(
                            Address {
                                root: Val(0),
                                offset
                            }
                            .project(delta, width, extent)
                            .map(|a| a.offset),
                            old
                        );
                    }
                }
            }
        }
        assert!(
            ByteRange {
                offset: 0,
                width: 4
            }
            .separated(ByteRange {
                offset: 4,
                width: 4
            })
        );
        assert!(
            !ByteRange {
                offset: 0,
                width: 8
            }
            .separated(ByteRange {
                offset: 4,
                width: 4
            })
        );
    }

    #[test]
    fn separated_ranges_require_both_ends_to_be_representable() {
        let overflow = ByteRange {
            offset: u64::MAX,
            width: 1,
        };
        let byte = ByteRange {
            offset: 0,
            width: 1,
        };
        assert!(!overflow.separated(byte));
        assert!(!byte.separated(overflow));
    }

    #[test]
    fn original_roots_keep_unknown_extent_and_canonical_identity() {
        let roots = [
            LifetimeStackRoot {
                value: Val(1),
                bytes: None,
            },
            LifetimeStackRoot {
                value: Val(2),
                bytes: Some(8),
            },
        ];
        let facts = StackRoots::from_parts(&roots, &[(Val(3), Val(2))], 2).unwrap();
        assert_eq!(facts.extent(Val(1)), Some(None));
        assert_eq!(facts.extent(Val(9)), None);
        assert_eq!(facts.canonical(Val(3)), Val(2));
        assert_eq!(
            facts.addresses()[&Val(3)],
            Address {
                root: Val(2),
                offset: 0
            }
        );
        assert!(StackRoots::from_parts(&roots, &[], 1).is_none());
        assert!(StackRoots::from_parts(&roots, &[(Val(3), Val(9))], 2).is_none());
        assert!(StackRoots::from_parts(&roots, &[(Val(3), Val(1)), (Val(3), Val(2))], 2).is_none());
        assert!(
            StackRoots::from_parts(
                &[
                    roots[0],
                    LifetimeStackRoot {
                        value: Val(1),
                        bytes: Some(8)
                    }
                ],
                &[],
                2
            )
            .is_none()
        );
        assert!(facts.retained_bytes() >= 2 * std::mem::size_of::<(Val, Option<u64>)>());
        let shared = std::rc::Rc::new(facts);
        assert!(std::rc::Rc::ptr_eq(&shared, &shared.clone()));
    }
}
