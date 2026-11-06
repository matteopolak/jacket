//! Preserve allocation provenance through a checked integer round-up of an address.
//! Ordinary integer results do not become pointers merely because they are cast.
use super::{Def, Event, HashMap, HashSet, ProcId, Resolver, Summary, Val, value};
use jaic::ast::{BinOp, Expr, ExprKind as E, StmtKind as S};
use jaic::intern::Sym;
use jaic::types::TypeKind;

fn identifier(e: &Expr, name: Sym) -> bool {
    matches!(e.kind,E::Ident(n) if n==name)
}

fn binary(e: &Expr, op: BinOp) -> Option<(&Expr, &Expr)> {
    let E::Binary(actual, a, b) = &e.kind else {
        return None;
    };
    (*actual == op).then_some((a, b))
}

fn round_up(e: &Expr, address: Sym, alignment: Sym) -> bool {
    let Some((division, last_alignment)) = binary(e, BinOp::Mul) else {
        return false;
    };
    if !identifier(last_alignment, alignment) {
        return false;
    }
    let Some((numerator, divisor)) = binary(division, BinOp::Div) else {
        return false;
    };
    if !identifier(divisor, alignment) {
        return false;
    }
    let Some((sum, one)) = binary(numerator, BinOp::Sub) else {
        return false;
    };
    if !matches!(&one.kind,E::Int(n,..) if n.get()==1) {
        return false;
    }
    let Some((a, b)) = binary(sum, BinOp::Add) else {
        return false;
    };
    identifier(a, address) && identifier(b, alignment)
}

impl Resolver<'_> {
    pub(super) fn rounded_address(&self, all: &HashMap<ProcId, Summary>) -> Option<usize> {
        let sig = self.compiler.pr.procs[self.body.procedure.0 as usize]
            .sig
            .as_ref()?;
        if sig.params.len() != 2 || sig.returns.len() != 1 {
            return None;
        }
        let integer = |ty| {
            matches!(
                self.compiler.ty.types.kind(ty),
                TypeKind::Int {
                    bits: 64,
                    ..
                }
            )
        };
        if !sig.params.iter().all(|p| integer(p.ty)) || !integer(sig.returns[0]) {
            return None;
        }
        let params = &self.body.syntax.header.params;
        if params.len() != 2 {
            return None;
        }
        let a = params[0].name?.name;
        let b = params[1].name?.name;
        let statements = &self.body.syntax.body.as_ref()?.stmts;
        let [prefix, returned] = statements.as_slice() else {
            return None;
        };
        if !matches!(prefix.kind, S::Expr(_)) {
            return None;
        }
        let S::Return {
            values,
            backtick: false,
        } = &returned.kind
        else {
            return None;
        };
        let [result] = values.as_slice() else {
            return None;
        };
        if !round_up(&result.value, a, b) {
            return None;
        }
        if self
            .params
            .keys()
            .any(|p| self.exposed.contains(p) || self.writes.get(p).is_none_or(|v| v.len() != 1))
        {
            return None;
        }
        // A source-level operator overload must not be mistaken for primitive arithmetic.
        if self
            .body
            .blocks
            .iter()
            .flat_map(|b| &b.events)
            .any(|e| match e {
                Event::Call {
                    callee,
                    span,
                    expands: false,
                    ..
                } => {
                    span.file == result.value.span.file
                        && result.value.span.start <= span.start
                        && span.end <= result.value.span.end
                        && all
                            .get(callee)
                            .is_none_or(|s| s.normal_return != Some(false))
                }
                Event::IndirectCall {
                    span, ..
                } => {
                    span.file == result.value.span.file
                        && result.value.span.start <= span.start
                        && span.end <= result.value.span.end
                }
                _ => false,
            })
        {
            return None;
        }
        Some(0)
    }

    pub(super) fn integer_address_source(
        &self,
        integer: Val,
        all: &HashMap<ProcId, Summary>,
        seen: &mut HashSet<Val>,
        depth: usize,
    ) -> Option<Val> {
        if depth > 24 || !seen.insert(integer) {
            return None;
        }
        let result = if let Some(cast) = self.address_casts.get(&integer).filter(|c| !c.to_pointer)
        {
            Some(cast.source)
        } else {
            let [definition] = self.values.get(&integer)?.as_slice() else {
                return None;
            };
            match definition {
                Def::Copy(source) => self.integer_address_source(*source, all, seen, depth + 1),
                Def::Load(place, _) => self
                    .return_source(*place)
                    .and_then(|v| self.integer_address_source(v, all, seen, depth + 1)),
                Def::Call(callee, args, 0) => all
                    .get(callee)?
                    .rounded_address
                    .and_then(|formal| args.iter().find(|a| a.formal == formal))
                    .and_then(value)
                    .and_then(|v| self.integer_address_source(v, all, seen, depth + 1)),
                _ => None,
            }
        };
        seen.remove(&integer);
        result
    }
}

impl Resolver<'_> {
    pub(super) fn adjusted_pointer(
        &self,
        v: Val,
        all: &HashMap<ProcId, Summary>,
        seen: &mut HashSet<Val>,
        depth: usize,
    ) -> bool {
        if depth > 24 || !seen.insert(v) {
            return false;
        }
        if let Some(cast) = self.address_casts.get(&v) {
            return self.adjusted_pointer(cast.source, all, seen, depth + 1);
        }
        self.values.get(&v).into_iter().flatten().any(|d| match d {
            Def::Copy(source) => self.adjusted_pointer(*source, all, seen, depth + 1),
            Def::Load(place, _) => self
                .return_source(*place)
                .is_some_and(|v| self.adjusted_pointer(v, all, seen, depth + 1)),
            Def::Call(callee, _, index) => all
                .get(callee)
                .is_some_and(|s| s.rounded_address.is_some() || s.adjusted_returns.contains(index)),
            _ => false,
        })
    }

    pub(in crate::rules) fn adjusted_result(
        &self,
        event: &Event,
        index: usize,
        all: &HashMap<ProcId, Summary>,
    ) -> bool {
        self.call_targets(event).procedures.iter().any(|p| {
            all.get(p)
                .is_some_and(|s| s.adjusted_returns.contains(&index))
        })
    }
}


pub(super) mod bit_returns;
pub(super) use bit_returns::BitReturn;

pub(super) use super::captured_values as producers;
