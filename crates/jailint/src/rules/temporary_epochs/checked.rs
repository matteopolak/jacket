use super::domain::*;
use super::*;
use jaic::fxhash::HashSet;
use jaic::ide::lifetimes::{LifetimeBody, LifetimeEvent as Event};
use jaic::sema::procs::BodyState;

#[derive(Default)]
pub(super) struct Summary {
    pub exits: Vec<State>,
    pub partial: bool,
    pub guards: Vec<(Site, Predicate)>,
    pub reached: HashSet<usize>,
    pub edges: HashSet<(usize, usize)>,
    pub rejected: bool,
}

pub(super) struct Budget {
    remaining: usize,
}

impl Budget {
    /// Charge before copying; `spend(0)` still polls cancellation.
    pub fn spend(&mut self, n: usize) -> Option<()> {
        if jaic::interrupt::should_stop() {
            return None;
        }
        let remaining = self.remaining.checked_sub(n)?;
        self.remaining = remaining;
        Some(())
    }
}

fn condition_cost(conditions: &[Condition]) -> Option<usize> {
    conditions.iter().try_fold(0usize, |n, c| {
        n.checked_add(predicate_size(&c.predicate)?.0.checked_add(1)?)
    })
}

fn observation_cost(o: &SnapshotObservation) -> Option<usize> {
    condition_cost(&o.conditions)?
        .checked_add(o.fields.len())?
        .checked_add(8)
}

fn protocol_cost(p: &ContiguousBump) -> Option<usize> {
    condition_cost(&p.conditions)?
        .checked_add(operand_size(&p.range.length)?.0.checked_mul(6)?)?
        .checked_add(32)
}

fn state_header(c: &Compiler, i: &Instance) -> ByteSlot {
    ByteSlot {
        offset: 0,
        width: c.ty.types.size_of(i.view_ty),
        ty: i.view_ty,
    }
}

pub(super) fn build(c: &Compiler) -> Analysis {
    build_budgeted(c, STEPS)
}

pub(super) fn build_budgeted(c: &Compiler, limit: usize) -> Analysis {
    extract(
        c,
        &mut Budget {
            remaining: limit,
        },
    )
    .unwrap_or_default()
}

/// Build conditional rows under one output budget; exhaustion publishes `None`.
fn extract(c: &Compiler, budget: &mut Budget) -> Option<Analysis> {
    budget.spend(0)?;
    let mut rows = HashMap::default();
    let mut summaries = HashMap::default();
    let mut scanned = 0usize;
    for (id, b) in c.ed.facts().into_iter().flat_map(|f| &f.lifetime_bodies) {
        budget.spend(1)?;
        if b.procedure != *id
            || c.pr.procs[id.0 as usize].body_state != BodyState::Done
            || !std::sync::Arc::ptr_eq(&c.pr.procs[id.0 as usize].lit, &b.syntax)
        {
            continue;
        }
        scanned = b
            .blocks
            .iter()
            .try_fold(scanned, |n, b| n.checked_add(b.events.len()))?;
        if scanned > 262_144 {
            return None;
        }
        if let Some(summary) = body(c, b) {
            let mut relation = observations(c, *id, &summary, budget)?;
            relation.forwards = forwarded(c, *id, &summary, budget)?;
            relation.loop_bypasses = super::paged::discover(c, b, &summary, budget)?;
            if !relation.observations.is_empty()
                || !relation.forwards.is_empty()
                || !relation.loop_bypasses.is_empty()
            {
                rows.insert(*id, relation);
            }
            summaries.insert(*id, summary);
        }
    }
    let mut protocols = Vec::new();
    for (id, summary) in &summaries {
        budget.spend(1)?;
        let found = bumps(c, *id, summary, budget)?;
        if found.is_empty() {
            continue;
        }
        if protocols.len().checked_add(found.len())? > ITEMS {
            return None;
        }
        for p in &found {
            budget.spend(protocol_cost(p)?)?;
            protocols.push(p.clone());
        }
        let row = rows.entry(*id).or_insert_with(|| Relations {
            incomplete: true,
            ..Relations::default()
        });
        row.protocols = found;
        row.incomplete = true;
    }
    let mut views: HashMap<TypeId, Vec<usize>> = HashMap::default();
    for (index, p) in protocols.iter().enumerate() {
        budget.spend(1)?;
        views.entry(p.instance.view_ty).or_default().push(index);
    }
    let mut schemas = HashSet::default();
    for row in rows.values() {
        for o in &row.observations {
            budget.spend(1)?;
            if let Some(instance) = &o.instance {
                for field in &o.fields {
                    budget.spend(1)?;
                    schemas.insert((
                        instance.view_ty,
                        o.result_ty,
                        field.result_slot,
                        field.source_slot,
                    ));
                }
            }
        }
    }
    for (id, summary) in &summaries {
        budget.spend(1)?;
        let mut row = rows.remove(id).unwrap_or_else(|| Relations {
            incomplete: true,
            ..Relations::default()
        });
        row.incomplete = true;
        materialize_snapshots(c, summary, &mut row, &views, &protocols, budget)?;
        materialize_transitions(
            c,
            *id,
            summary,
            &mut row,
            (&views, &protocols, &schemas),
            budget,
        )?;
        if !row.loop_bypasses.is_empty()
            || !row.forwards.is_empty()
            || !row.observations.is_empty()
            || !row.protocols.is_empty()
            || !row.rewinds.is_empty()
            || !row.resets.is_empty()
        {
            rows.insert(*id, row);
        }
    }
    budget.spend(0)?;
    Some(Analysis {
        relations: rows,
    })
}

fn validity(i: &Instance, lineage: LineageInput) -> ValidityRequirement {
    ValidityRequirement {
        instance: i.clone(),
        reset_generation: GenerationInput::CallerCurrent,
        lineage,
    }
}

/// Interpret checked paths; for example, a repeated block makes coverage partial.
fn body(c: &Compiler, b: &LifetimeBody) -> Option<Summary> {
    body_specialized(c, b, None, None)
}

pub(super) fn body_specialized(
    c: &Compiler,
    b: &LifetimeBody,
    bypass: Option<super::paged::Inputs>,
    mut budget: Option<&mut Budget>,
) -> Option<Summary> {
    let super::entry::Entry {
        initial,
        copies: entry_copies,
        overlays,
    } = super::entry::prepare(c, b, bypass)?;
    let transfer = super::transfer::Transfer {
        c,
        b,
        entry_copies: &entry_copies,
        overlays: &overlays,
    };
    let mut summary = Summary::default();
    let mut work = vec![(0usize, initial)];
    let mut steps = 0usize;
    while let Some((block, mut s)) = work.pop() {
        if jaic::interrupt::should_stop() {
            return None;
        }
        if !s.visited.insert(block) {
            summary.partial = true;
            continue;
        }
        summary.reached.insert(block);
        let bb = b.blocks.get(block)?;
        let mut branch = None;
        for (event, e) in bb.events.iter().enumerate() {
            let charge = 1usize.checked_add(b.comparisons.len())?;
            if let Some(budget) = budget.as_deref_mut() {
                budget.spend(charge)?;
            }
            steps = steps.checked_add(charge)?;
            if steps > STEPS {
                summary.partial = true;
                return Some(summary);
            }
            if jaic::interrupt::should_stop() {
                return None;
            }
            let at = site(b.procedure, block, event);
            refresh(b, &mut s);
            if s.bypass.is_some() {
                if matches!(e, Event::BoxAny { .. } | Event::Grow { .. }) {
                    s.bypass_failed = true;
                }
                if let Event::ReadMemory {
                    value, ..
                } = e
                {
                    if let Some(a) = s.address(*value) {
                        s.bypass_access(c, &a, 0, 0);
                    } else {
                        s.bypass_failed = true;
                    }
                }
                if let Event::Load {
                    place,
                    ty,
                    ..
                }
                | Event::Store {
                    place,
                    ty,
                    ..
                }
                | Event::Initialize {
                    place,
                    ty,
                    ..
                }
                | Event::Indirect {
                    place,
                    ty,
                } = e
                {
                    if let Some(a) = s.address(*place) {
                        s.bypass_access(c, &a, 0, c.ty.types.size_of(*ty));
                    } else {
                        s.bypass_failed = true;
                    }
                }
            }
            if let Event::Branch {
                condition,
                then_block,
                else_block,
                ..
            } = e
            {
                let predicate = match s.value(*condition) {
                    Expr::Predicate(p) => p,
                    _ => Predicate::Unknown,
                };
                if summary.guards.len() < ITEMS {
                    summary.guards.push((at.clone(), predicate));
                }
                branch = Some((*condition, *then_block, *else_block, at.clone()));
            } else {
                transfer.apply(&mut s, e, &at);
            }
            if s.bypass.is_some() && (s.interfered || s.bypass_failed) {
                summary.rejected = true;
            }
            if !s.bounded() {
                summary.partial = true;
                return Some(summary);
            }
        }
        if bb.normal_return {
            summary.exits.push(s);
        } else if let Some((condition, yes, no, at)) = branch {
            let p = s.simplified(&match s.value(condition) {
                Expr::Predicate(p) => p,
                _ => Predicate::Unknown,
            });
            if s.bypass.is_some() && !predicate_supported(&p) {
                summary.rejected = true;
            }
            for (target, truth) in [(yes, true), (no, false)] {
                if matches!(p,Predicate::Constant(v) if v!=truth) {
                    continue;
                }
                let mut fork = s.clone();
                fork.incomplete |= p == Predicate::Unknown;
                if !matches!(p, Predicate::Constant(_)) {
                    fork.conditions.push(Condition {
                        site: at.clone(),
                        predicate: p.clone(),
                        truth,
                    });
                }
                summary.edges.insert((block, target));
                work.push((target, fork));
            }
        } else {
            for next in &bb.successors {
                summary.edges.insert((block, *next));
                work.push((*next, s.clone()));
            }
        }
        if work.len() + summary.exits.len() > ITEMS {
            summary.partial = true;
            return Some(summary);
        }
    }
    Some(summary)
}

fn refresh(b: &LifetimeBody, s: &mut State) {
    for cmp in &b.comparisons {
        let a = cmp.left_constant.map_or_else(
            || s.value(cmp.left),
            |n| {
                Expr::Scalar(Operand::Constant {
                    ty: TypeId::S64,
                    bits: n,
                })
            },
        );
        let d = cmp.right_constant.map_or_else(
            || s.value(cmp.right),
            |n| {
                Expr::Scalar(Operand::Constant {
                    ty: TypeId::S64,
                    bits: n,
                })
            },
        );
        if !a.size().zip(d.size()).is_some_and(|((n, x), (m, y))| {
            n.checked_add(m)
                .and_then(|n| n.checked_add(1))
                .is_some_and(|n| n <= 128)
                && x.max(y) < 16
        }) {
            continue;
        }
        let left = a.operand();
        let right = d.operand();
        if operand_supported(&left) && operand_supported(&right) {
            s.values.insert(
                cmp.value,
                Expr::Predicate(Predicate::Compare {
                    left,
                    right,
                    op: cmp.op,
                    ty: cmp.ty,
                }),
            );
        }
    }
}

/// Read actual returned field snapshots, such as a copied cursor tuple.
fn observations(
    c: &Compiler,
    _p: ProcId,
    summary: &Summary,
    budget: &mut Budget,
) -> Option<Relations> {
    let mut row = Relations {
        incomplete: true,
        ..Relations::default()
    };
    for s in &summary.exits {
        budget.spend(1)?;
        row.incomplete |= s.incomplete;
        for (result, ty, _, address, site) in &s.returns {
            budget.spend(1)?;
            if scalar(c, *ty) {
                continue;
            }
            let (Some(a), Some(fields)) = (address, leaves(c, *ty)) else {
                continue;
            };

            let mut instance = None;
            let mut captures = Vec::new();
            let mut epoch = None;
            let mut valid = true;
            for slot in fields {
                budget.spend(1)?;
                if let Some(Expr::Read {
                    instance: i,
                    slot: source_slot,
                    site: read_site,
                    epoch: version,
                }) = s.existing(c, a, slot)
                {
                    if instance.as_ref().is_some_and(|j| j != i)
                        || epoch.is_some_and(|e| e != *version)
                    {
                        valid = false;
                        break;
                    }
                    instance = Some(i.clone());
                    epoch = Some(*version);
                    captures.push(FieldCapture {
                        result_slot: slot,
                        source_slot: *source_slot,
                        read_site: read_site.clone(),
                    });
                } else {
                    valid = false;
                    break;
                }
            }
            if valid && !captures.is_empty() {
                if row.observations.len() >= ITEMS {
                    return None;
                }
                budget.spend(
                    captures
                        .len()
                        .checked_add(condition_cost(&s.conditions)?)?
                        .checked_add(8)?,
                )?;
                let o = SnapshotObservation {
                    site: site.clone(),
                    result: *result,
                    result_ty: *ty,
                    instance,
                    fields: captures,
                    conditions: s.conditions.clone(),
                };
                row.observations.push(o);
            }
        }
    }
    Some(row)
}

/// Forward only an existing opaque formal ghost; for example, `return copied_mark`.
fn forwarded(
    c: &Compiler,
    procedure: ProcId,
    summary: &Summary,
    budget: &mut Budget,
) -> Option<Vec<ForwardedMark>> {
    let mut out = Vec::new();
    if summary.partial {
        return Some(out);
    }
    for state in &summary.exits {
        budget.spend(1)?;
        if state.interfered || !state.writes.is_empty() {
            continue;
        }
        for (result, ty, _, address, _) in &state.returns {
            budget.spend(1)?;
            if scalar(c, *ty) || state.output_writes.iter().any(|written| written != result) {
                continue;
            }
            let Some(address) = address else {
                continue;
            };
            budget.spend(4)?;
            let Some(origin) = state.whole(c, address, *ty) else {
                continue;
            };
            if origin.ty != *ty
                || c.pr.procs[procedure.0 as usize]
                    .sig
                    .as_ref()
                    .and_then(|s| s.params.get(origin.formal))
                    .is_none_or(|p| p.ty != origin.ty)
            {
                continue;
            }
            budget.spend(condition_cost(&state.conditions)?)?;
            if state
                .conditions
                .iter()
                .any(|g| !predicate_supported(&g.predicate))
            {
                continue;
            }
            if out.len() >= ITEMS {
                return None;
            }
            budget.spend(
                origin
                    .fields
                    .len()
                    .checked_add(condition_cost(&state.conditions)?)?
                    .checked_add(20)?,
            )?;
            out.push(ForwardedMark {
                result: *result,
                result_ty: *ty,
                source_formal: origin.formal,
                source_ty: origin.ty,
                entry_copy_site: origin.entry_copy,
                return_copy_site: origin.last_copy,
                fields: origin.fields.to_vec(),
                conditions: state.conditions.clone(),
                obligations: vec![
                    Obligation::ExistingMarkOrigin {
                        formal: origin.formal,
                    },
                    Obligation::CallerArgumentSnapshot {
                        formal: origin.formal,
                        ty: origin.ty,
                    },
                    Obligation::ForwardedOutputHeaderSeparation {
                        result: *result,
                        source_formal: origin.formal,
                    },
                ],
            });
        }
    }
    Some(out)
}

fn signed_length(c: &Compiler, value: &Operand) -> bool {
    fn typed(c: &Compiler, value: &Operand) -> bool {
        match value {
            Operand::Constant {
                ty, ..
            } => c.ty.types.int_info(*ty) == Some((64, true)),
            Operand::Captured(capture) => match capture {
                Captured::FormalValue {
                    slot, ..
                }
                | Captured::ReceiverRead {
                    slot, ..
                }
                | Captured::ContextRead {
                    slot, ..
                } => c.ty.types.int_info(slot.ty) == Some((64, true)),
                Captured::Constant {
                    ty, ..
                } => c.ty.types.int_info(*ty) == Some((64, true)),
                _ => false,
            },
            Operand::Add {
                left,
                right,
                ty,
            }
            | Operand::Sub {
                left,
                right,
                ty,
            } => c.ty.types.int_info(*ty) == Some((64, true)) && typed(c, left) && typed(c, right),
            Operand::Unknown => false,
        }
    }
    typed(c, value)
        && !matches!(value,Operand::Constant{bits,..}|Operand::Captured(Captured::Constant{bits,..}) if (*bits as i64)<0)
}

/// Match a returned base-plus-cursor range with its single cursor advance.
fn bumps(
    c: &Compiler,
    p: ProcId,
    summary: &Summary,
    budget: &mut Budget,
) -> Option<Vec<ContiguousBump>> {
    let mut out = Vec::new();
    if summary.partial {
        return Some(out);
    }
    for s in &summary.exits {
        budget.spend(1)?;
        if s.interfered || s.writes.len() != 1 || !s.output_writes.is_empty() {
            continue;
        }
        for (result, ty, value, _, at) in &s.returns {
            budget.spend(1)?;
            if c.ty.types.pointee(*ty).is_none() {
                continue;
            }
            let Expr::Offset {
                base,
                index,
                stride: 1,
            } = value
            else {
                continue;
            };
            let Expr::Read {
                instance,
                slot: data,
                ..
            } = &**base
            else {
                continue;
            };
            let Expr::Read {
                instance: cursor_instance,
                slot: cursor,
                ..
            } = &**index
            else {
                continue;
            };
            if instance != cursor_instance
                || data.width != 8
                || cursor.width != 8
                || c.ty.types.pointee(data.ty).is_none()
                || c.ty.types.int_info(cursor.ty) != Some((64, true))
            {
                continue;
            }
            let separate = data
                .offset
                .checked_add(data.width)
                .is_some_and(|end| end <= cursor.offset)
                || cursor
                    .offset
                    .checked_add(cursor.width)
                    .is_some_and(|end| end <= data.offset);
            if !separate {
                continue;
            }

            budget.spend(condition_cost(&s.conditions)?)?;
            if s.conditions
                .iter()
                .any(|g| !predicate_supported(&g.predicate))
            {
                continue;
            }

            let mut found = None;
            for write in &s.writes {
                budget.spend(write.value.size()?.0.checked_add(1)?)?;
                if write.instance == *instance
                    && write.slot == *cursor
                    && let Expr::Add {
                        left,
                        right,
                        ..
                    } = &write.value
                    && **left == **index
                {
                    found = Some(right.operand());
                }
            }
            let Some(length) = found else {
                continue;
            };
            if s.writes.len() != 1 {
                continue;
            }

            if !signed_length(c, &length)
                || s.writes
                    .iter()
                    .any(|w| w.instance == *instance && w.slot != *cursor)
            {
                continue;
            }
            if out.len() >= ITEMS {
                return None;
            }
            budget.spend(
                condition_cost(&s.conditions)?
                    .checked_add(value.size()?.0)?
                    .checked_add(32)?,
            )?;
            let base = base.captured();
            let cursor_capture = index.captured();
            if base == Captured::Unknown || cursor_capture == Captured::Unknown {
                continue;
            }
            let range = AllocationRange {
                base: base.clone(),
                cursor: cursor_capture,
                length,
            };
            budget.spend(operand_size(&range.length)?.0.checked_mul(6)?)?;
            let written = vec![*cursor];
            let protocol = ProtocolKey {
                procedure: p,
                data: *data,
                cursor: *cursor,
                capacity: None,
            };
            let obligations = vec![
                Obligation::ReadableWritableState {
                    instance: instance.clone(),
                    bytes: c.ty.types.size_of(instance.view_ty),
                },
                Obligation::NonNegativeCursorAndLength {
                    cursor: range.cursor.clone(),
                    length: range.length.clone(),
                },
                Obligation::CheckedArithmetic {
                    expressions: vec![
                        s.writes
                            .iter()
                            .find(|w| w.instance == *instance && w.slot == *cursor)
                            .map_or(Operand::Unknown, |w| w.value.operand()),
                    ],
                },
                Obligation::ReadableWritableBacking {
                    instance: instance.clone(),
                    range: range.clone(),
                },
                Obligation::NonNullBacking {
                    base,
                },
                Obligation::CheckedAlignmentAndAddition {
                    range: range.clone(),
                },
                Obligation::HeaderDataSeparation {
                    instance: instance.clone(),
                    state: state_header(c, instance),
                    range: range.clone(),
                },
            ];
            out.push(ContiguousBump {
                site: at.clone(),
                instance: instance.clone(),
                protocol,
                result: *result,
                range,
                written,
                obligations,
                conditions: s.conditions.clone(),
            });
        }
    }
    Some(out)
}

type Views = HashMap<TypeId, Vec<usize>>;
type Schemas = HashSet<(TypeId, TypeId, ByteSlot, ByteSlot)>;
type MaterializationIndex<'a> = (&'a Views, &'a [ContiguousBump], &'a Schemas);

fn materialize_snapshots(
    c: &Compiler,
    summary: &Summary,
    row: &mut Relations,
    views: &Views,
    protocols: &[ContiguousBump],
    budget: &mut Budget,
) -> Option<()> {
    for o in &row.observations {
        budget.spend(1)?;
        let Some(instance) = &o.instance else {
            continue;
        };
        let Some(candidates) = views.get(&instance.view_ty) else {
            continue;
        };
        budget.spend(condition_cost(&o.conditions)?)?;
        if summary.partial
            || o.conditions
                .iter()
                .any(|g| !predicate_supported(&g.predicate))
        {
            continue;
        }
        let mut eligible = false;
        for state in &summary.exits {
            budget.spend(1)?;
            if state.interfered
                || !state.writes.is_empty()
                || state.output_writes.iter().any(|result| *result != o.result)
            {
                continue;
            }
            budget.spend(
                condition_cost(&state.conditions)?
                    .checked_add(condition_cost(&o.conditions)?)?
                    .checked_add(state.returns.len())?,
            )?;
            if state.conditions == o.conditions
                && state.returns.iter().any(|(_, _, _, _, at)| at == &o.site)
            {
                eligible = true;
                break;
            }
        }
        if !eligible {
            continue;
        }
        for index in candidates {
            let p = &protocols[*index];
            budget.spend(o.fields.len().checked_add(1)?)?;
            if !o.fields.iter().any(|f| f.source_slot == p.protocol.cursor) {
                continue;
            }
            if row.snapshots.len() >= ITEMS {
                return None;
            }
            budget.spend(observation_cost(o)?.checked_add(16)?)?;
            row.snapshots.push(SnapshotTemplate {
                observation: o.clone(),
                protocol: p.protocol.clone(),
                validity: validity(instance, LineageInput::CallerCurrentFrontier),
                obligations: vec![
                    Obligation::ActiveProtocol {
                        instance: instance.clone(),
                        protocol: p.protocol.clone(),
                    },
                    Obligation::OutputHeaderSeparation {
                        result: o.result,
                        instance: instance.clone(),
                        state: state_header(c, instance),
                    },
                ],
            });
        }
    }
    Some(())
}

fn materialize_transitions(
    c: &Compiler,
    id: ProcId,
    summary: &Summary,
    row: &mut Relations,
    index: MaterializationIndex<'_>,
    budget: &mut Budget,
) -> Option<()> {
    let (views, protocols, schemas) = index;
    for state in &summary.exits {
        budget.spend(1)?;
        if summary.partial
            || state.interfered
            || state.writes.len() != 1
            || !state.output_writes.is_empty()
        {
            continue;
        }
        let write = &state.writes[0];
        let Some(candidates) = views.get(&write.instance.view_ty) else {
            continue;
        };
        budget.spend(condition_cost(&state.conditions)?)?;
        if state
            .conditions
            .iter()
            .any(|g| !predicate_supported(&g.predicate))
        {
            continue;
        }
        for index in candidates {
            let p = &protocols[*index];
            budget.spend(1)?;
            if write.slot != p.protocol.cursor {
                continue;
            }
            if write.value.integer() == Some(0) {
                if row.resets.len() >= ITEMS {
                    return None;
                }
                budget.spend(condition_cost(&state.conditions)?.checked_add(16)?)?;
                row.resets.push(ResetTemplate {
                    site: write.site.clone(),
                    instance: write.instance.clone(),
                    protocol: p.protocol.clone(),
                    obligations: vec![Obligation::ActiveProtocol {
                        instance: write.instance.clone(),
                        protocol: p.protocol.clone(),
                    }],
                    retire_all_prior_leases: true,
                    conditions: state.conditions.clone(),
                });
            }
            let Expr::Scalar(Operand::Captured(Captured::FormalValue {
                formal,
                slot,
            })) = &write.value
            else {
                continue;
            };
            let Some(ty) = c.pr.procs[id.0 as usize]
                .sig
                .as_ref()
                .and_then(|s| s.params.get(*formal))
                .map(|p| p.ty)
            else {
                continue;
            };
            if scalar(c, ty) || !schemas.contains(&(write.instance.view_ty, ty, *slot, write.slot))
            {
                continue;
            }
            if row.rewinds.len() >= ITEMS {
                return None;
            }
            budget.spend(condition_cost(&state.conditions)?.checked_add(24)?)?;
            row.rewinds.push(RewindTemplate {
                site: write.site.clone(),
                instance: write.instance.clone(),
                protocol: p.protocol.clone(),
                mark: MarkInput {
                    formal: *formal,
                    ty,
                    cursor_slot: *slot,
                },
                obligations: vec![
                    Obligation::ActiveProtocol {
                        instance: write.instance.clone(),
                        protocol: p.protocol.clone(),
                    },
                    Obligation::SameInstanceAndValidLineage {
                        mark_formal: *formal,
                        validity: validity(&write.instance, LineageInput::CallerReachableMark),
                    },
                    Obligation::FrontierBounds {
                        instance: write.instance.clone(),
                        cursor: write.slot,
                        mark_slot: *slot,
                    },
                ],
                discard_descendants: true,
                conditions: state.conditions.clone(),
            });
        }
    }
    Some(())
}
