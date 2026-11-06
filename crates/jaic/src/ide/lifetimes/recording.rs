//! Checked lifetime recording and finalization, separate from retained fact types.
use super::{
    Arc, ArrayKind, Compiler, E, FnCtx, HashMap, HashSet, LifetimeBoolean, LifetimeComparison,
    LifetimeEvent, LifetimeOperand, LifetimeRecording, LifetimeStackRoot, Operand, ProcId, Span,
    TypeId, TypeKind, Val, addresses, ast, booleans, controls, ir,
};

impl Compiler {
    /// Freeze only procedure-slot initializer facts, rather than retaining executable data.
    pub(crate) fn lifetime_release_globals(&mut self) {
        use crate::sema::scope::{EntityState, Resolved};
        let Some(facts) = self.ed.facts() else {
            return;
        };
        let ids: HashSet<_> = facts
            .lifetime_bodies
            .values()
            .flat_map(|b| b.blocks.iter().flat_map(|b| &b.events))
            .filter_map(|event| {
                let LifetimeEvent::Expression {
                    ty,
                    entity: Some(entity),
                    ..
                } = event
                else {
                    return None;
                };
                if !matches!(self.ty.types.kind(*ty), TypeKind::Proc(_)) {
                    return None;
                }
                let EntityState::Done(Resolved::Global {
                    storage: ir::Storage::Data(id),
                    ..
                }) = self.entity(*entity).state
                else {
                    return None;
                };
                Some(id)
            })
            .collect();
        let retained = ids
            .into_iter()
            .map(|id| {
                let global = &self.out.program.globals[id.0 as usize];
                (
                    id,
                    (global.init.iter().any(|&b| b != 0), global.relocs.clone()),
                )
            })
            .collect::<Vec<_>>();
        for (id, initial) in retained {
            self.ed
                .facts_mut()
                .unwrap()
                .lifetime_procedure_globals
                .entry(id)
                .or_insert(initial);
        }
    }

    pub(crate) fn lifetime_expression(&self, f: &mut FnCtx, expr: &ast::Expr, op: &Operand) {
        if f.type_only {
            return;
        }
        let Some(record) = &mut f.lifetimes else {
            return;
        };
        if matches!(expr.kind, E::Context) && !record.context_types.contains(&op.ty()) {
            record.context_types.push(op.ty());
        }
        let operand = LifetimeOperand::of(op);
        let entity_span = match &expr.kind {
            E::Member(_, member) => member.span,
            _ => expr.span,
        };
        let entity = self
            .ed
            .facts()
            .and_then(|i| i.last_entity)
            .filter(|(at, _)| *at == entity_span)
            .map(|(_, id)| id);
        let block = f.b.current;
        if matches!(operand, LifetimeOperand::Place(_)) || self.ty.types.is_pointer(op.ty()) {
            record.push(
                block,
                LifetimeEvent::Expression {
                    span: expr.span,
                    ty: op.ty(),
                    operand,
                    entity,
                },
            );
        }
        if let LifetimeOperand::Place(place) = operand {
            if matches!(
                self.ty.types.kind(op.ty()),
                TypeKind::Array {
                    kind: ArrayKind::Resizable,
                    ..
                }
            ) {
                record.resizable.insert(place, op.ty());
            }
            let comparison_reads_storage = f.lifetime_comparing
                && matches!(
                    self.ty.types.kind(op.ty()),
                    TypeKind::String
                        | TypeKind::Array {
                            kind: ArrayKind::View,
                            ..
                        }
                );
            let nonuse = f.lifetime_nonuses.contains(&Some(expr.span)) && !comparison_reads_storage;
            let metadata_only =
                f.lifetime_metadata_base == Some(expr.span) && !self.ty.types.is_pointer(op.ty());
            if matches!(expr.kind, E::Ident(_) | E::Member(..)) && !nonuse && !metadata_only {
                record.push(
                    block,
                    LifetimeEvent::Use {
                        place,
                        ty: op.ty(),
                        span: expr.span,
                    },
                );
            }
        }
        if matches!(expr.kind, E::Asm(_)) {
            record.push(block, LifetimeEvent::Barrier);
        }
    }
}

impl FnCtx {
    /// Preserve the checked boolean operation without decoding executable IR.
    pub(crate) fn lifetime_boolean(&mut self, value: Val, operation: LifetimeBoolean) {
        if !self.type_only
            && let Some(record) = &mut self.lifetimes
        {
            record.push(
                self.b.current,
                LifetimeEvent::Boolean {
                    value,
                    operation,
                },
            );
        }
    }

    pub(crate) fn lifetime_branch(
        &mut self,
        condition: Val,
        span: Span,
        then_block: ir::BlockId,
        else_block: ir::BlockId,
    ) {
        if !self.type_only
            && let Some(record) = &mut self.lifetimes
        {
            record.push(
                self.b.current,
                LifetimeEvent::Branch {
                    condition,
                    span,
                    then_block: then_block.0 as usize,
                    else_block: else_block.0 as usize,
                },
            );
        }
    }

    /// The current selected operation may invoke an external effect. For example,
    /// a warning handler can return after changing caller-visible state.
    pub(crate) fn lifetime_barrier(&mut self) {
        if !self.type_only
            && let Some(record) = &mut self.lifetimes
        {
            record.push(self.b.current, LifetimeEvent::Barrier);
        }
    }

    /// Collection-loop metadata at the body entry (`element`) or its step edge (`None`).
    pub(crate) fn lifetime_iteration(
        &mut self,
        region: (Val, TypeId),
        cursor: (Val, crate::intern::Sym, crate::intern::Sym),
        reverse: bool,
        span: Span,
        element: Option<(Val, TypeId, Val)>,
    ) {
        if self.type_only {
            return;
        }
        let Some(record) = &mut self.lifetimes else {
            return;
        };
        if let Some((result, result_ty, subscript)) = element {
            record.push(
                self.b.current,
                LifetimeEvent::Index {
                    base: region.0,
                    base_ty: region.1,
                    result,
                    result_ty,
                    subscript,
                    index: None,
                },
            );
        }
        record.push(
            self.b.current,
            LifetimeEvent::Iteration {
                region: region.0,
                ty: region.1,
                index: cursor.0,
                name: cursor.1,
                index_name: cursor.2,
                reverse,
                span,
                advance: element.is_none(),
            },
        );
    }


    /// Retain a real load at its actual epoch, such as an ifx result-slot read.
    pub(crate) fn lifetime_load(&mut self, place: Val, value: Val, ty: TypeId, span: Span) {
        if !self.type_only
            && let Some(record) = &mut self.lifetimes
        {
            record.push(
                self.b.current,
                LifetimeEvent::Load {
                    place,
                    value,
                    ty,
                    span,
                },
            );
        }
    }

    pub(crate) fn lifetime_store(&mut self, place: Val, value: Val, ty: TypeId, span: Span) {
        if !self.type_only
            && let Some(record) = &mut self.lifetimes
        {
            record.push(
                self.b.current,
                LifetimeEvent::Store {
                    place,
                    value,
                    ty,
                    span,
                    constant: None,
                },
            );
        }
    }

    pub(crate) fn lifetime_view(
        &mut self,
        value: Val,
        source: Val,
        from: TypeId,
        to: TypeId,
        span: Span,
    ) {
        if !self.type_only
            && let Some(record) = &mut self.lifetimes
        {
            record.push(
                self.b.current,
                LifetimeEvent::View {
                    value,
                    source,
                    from,
                    to,
                    span,
                },
            );
        }
    }

    pub(crate) fn lifetime_copy(&mut self, value: Val, source: Val) {
        if value == source {
            return;
        }
        if !self.type_only
            && let Some(record) = &mut self.lifetimes
        {
            record.push(
                self.b.current,
                LifetimeEvent::Copy {
                    value,
                    source,
                },
            );
        }
    }
}

impl Compiler {
    /// Freeze the source extent of the selected aggregate constant address.
    /// For example, `Item.{}` can supply an immutable full-width copy source.
    pub(crate) fn lifetime_read_only_region(
        &self,
        f: &mut FnCtx,
        value: Val,
        ty: TypeId,
        source: ir::GlobalId,
    ) {
        if f.type_only || f.lifetimes.is_none() {
            return;
        }
        // Lint recording excludes speculative body execution. Still fail closed
        // for an unresolved LocalGlobal ID instead of indexing executable data.
        let Some(global) = self.out.program.globals.get(source.0 as usize) else {
            return;
        };
        let bytes = self.ty.types.size_of(ty);
        if !global.read_only
            || global.size != bytes
            || u64::try_from(global.init.len()).ok() != Some(bytes)
        {
            return;
        }
        f.lifetimes
            .as_mut()
            .unwrap()
            .read_only_regions
            .push(super::LifetimeReadOnlyRegion {
                value,
                ty,
                bytes,
                leaves: Vec::new(),
                complete: false,
            });
    }

    /// Retain values only from the actual committed image selected for a
    /// memory-valued constant, for example the source of `ctx.* = .{}`.
    pub(crate) fn lifetime_constant_region(
        &self,
        f: &mut FnCtx,
        value: Val,
        ty: TypeId,
        source: ir::GlobalId,
    ) {
        let before = f
            .lifetimes
            .as_ref()
            .map_or(0, |record| record.read_only_regions.len());
        self.lifetime_read_only_region(f, value, ty, source);
        let Some(record) = f.lifetimes.as_mut() else {
            return;
        };
        if record.read_only_regions.len() == before {
            return;
        }
        // The extent recorder authenticated this exact selected global. Unlike
        // runtime configuration storage, const_global's selected image is final.
        let Some(global) = self.out.program.globals.get(source.0 as usize) else {
            return;
        };
        let (leaves, complete) =
            self.lifetime_default_values(ty, Some((&global.init, &global.relocs)));
        let region = record.read_only_regions.last_mut().unwrap();
        region.leaves = leaves;
        region.complete = complete;
    }

    pub(crate) fn lifetime_begin_body(&mut self, f: &mut FnCtx, id: ProcId) {
        use crate::ide::Recorder;
        if self.ed.facts().is_some_and(|i| i.lint) && !self.ed.sketching() {
            self.ed.facts_mut().unwrap().invalidate_execution();
            self.ed.facts_mut().unwrap().lifetime_bodies.remove(&id);
            f.lifetimes = Some(Box::new(LifetimeRecording {
                context_types: self.ty.context_type.into_iter().collect(),
                ..Default::default()
            }));
        }
    }

    pub(crate) fn lifetime_finish_body(
        &mut self,
        f: &mut FnCtx,
        id: ProcId,
        syntax: &Arc<ast::ProcLit>,
    ) {
        if let Some(mut record) = f.lifetimes.take() {
            record
                .blocks
                .resize_with(f.b.func.blocks.len(), Default::default);
            let constants: HashMap<_, _> =
                f.b.func
                    .blocks
                    .iter()
                    .flat_map(|b| &b.insts)
                    .filter_map(|inst| match inst {
                        ir::Inst::IConst {
                            dst,
                            value,
                            ..
                        } => Some((*dst, *value)),
                        _ => None,
                    })
                    .collect();
            for (out, block) in record.blocks.iter_mut().zip(&f.b.func.blocks) {
                out.normal_return = matches!(block.term, ir::Term::Ret(_));
                for event in &mut out.events {
                    event.resolve_constants(&constants);
                }
                if let ir::Term::Branch {
                    cond,
                    then_block,
                    else_block,
                } = block.term
                    && let Some(value) = constants.get(&cond)
                {
                    out.successors.push(
                        if *value == 0 {
                            else_block
                        } else {
                            then_block
                        }
                        .0 as usize,
                    );
                } else {
                    block
                        .term
                        .for_each_successor(|b| out.successors.push(b.0 as usize));
                }
            }
            record.prune_cleanup_edges(&f.b.func, &constants);
            let globals: Vec<_> =
                f.b.func
                    .blocks
                    .iter()
                    .flat_map(|b| &b.insts)
                    .filter_map(|inst| {
                        if let ir::Inst::GlobalAddr {
                            dst,
                            global,
                        } = *inst
                        {
                            Some((dst, global))
                        } else {
                            None
                        }
                    })
                    .collect();
            let boolean_branches =
                booleans::immutable_boolean_branches(&f.b.func, &record.blocks, &constants);
            let facts = self.ed.facts_mut().unwrap();
            for &(_, id) in &globals {
                let global = &self.out.program.globals[id.0 as usize];
                if global.read_only && !global.relocs.is_empty() {
                    facts
                        .lifetime_globals
                        .entry(id)
                        .or_insert_with(|| global.relocs.clone());
                }
            }
            let ids = facts
                .lifetime_files
                .entry(syntax.header.span.file)
                .or_default();
            if !ids.contains(&id) {
                ids.push(id);
            }
            facts.invalidate_execution();
            facts.lifetime_bodies.insert(
                id,
                crate::ide::lifetimes::LifetimeBody {
                    procedure: id,
                    syntax: syntax.clone(),
                    context_types: record.context_types,
                    context_input: record.context_input,
                    call_contexts: record.call_contexts,
                    overlay_accesses: record.overlay_accesses,
                    macro_owners: record.macro_owners,
                    blocks: record.blocks,
                    source_ranges: record.source_ranges,
                    slot_aliases: slot_aliases(&f.b.func),
                    stack_places: stack_places(&f.b.func, record.stack_subplaces),
                    stack_roots: stack_roots(&f.b.func),
                    read_only_regions: record.read_only_regions,
                    globals,
                    address_casts: addresses::capture(&f.b.func),
                    boolean_branches,
                    controls: controls::capture(&f.b.func),
                    comparisons: f
                        .b
                        .func
                        .blocks
                        .iter()
                        .flat_map(|b| &b.insts)
                        .filter_map(|inst| {
                            let ir::Inst::Cmp {
                                dst,
                                op,
                                ty,
                                a,
                                b,
                            } = *inst
                            else {
                                return None;
                            };
                            Some(LifetimeComparison {
                                ty,
                                value: dst,
                                left: a,
                                right: b,
                                op,
                                left_constant: constants.get(&a).copied(),
                                right_constant: constants.get(&b).copied(),
                            })
                        })
                        .collect(),
                },
            );
        }
    }
}


impl LifetimeRecording {
    /// Propagate activation flags and unaliased byte locals through cleanup branches.
    /// A slot whose address escapes, is aliased, or is accessed at another width is excluded.
    fn prune_cleanup_edges(&mut self, func: &ir::Func, constants: &HashMap<Val, u64>) {
        if !func
            .blocks
            .iter()
            .any(|b| matches!(b.term, ir::Term::Branch { .. }))
        {
            return;
        }
        let mut flags: Vec<_> = self.cleanup_flags.iter().copied().collect();
        if !self.bool_slots.is_empty() {
            flags.extend(
                private_byte_flags(func)
                    .into_iter()
                    .filter(|v| self.bool_slots.contains(v)),
            );
        }
        flags.sort_unstable_by_key(|v| v.0);
        flags.dedup();
        if flags.is_empty() {
            return;
        }
        let slots: HashMap<_, _> = flags.into_iter().enumerate().map(|(i, v)| (v, i)).collect();
        let mut states = vec![None; self.blocks.len()];
        states[0] = Some(vec![1u8; slots.len()]);
        let mut queue = std::collections::VecDeque::from([0]);
        while let Some(i) = queue.pop_front() {
            let mut state = states[i].clone().unwrap();
            let loads = cleanup_loads(&func.blocks[i].insts, &mut state, &slots, constants);
            let successors = match func.blocks[i].term {
                ir::Term::Branch {
                    cond,
                    then_block,
                    else_block,
                } => match loads.get(&cond) {
                    Some(2) => vec![else_block.0 as usize],
                    Some(4) => vec![then_block.0 as usize],
                    _ => self.blocks[i].successors.clone(),
                },
                _ => self.blocks[i].successors.clone(),
            };
            for next in successors {
                let changed = if let Some(previous) = &mut states[next] {
                    let mut changed = false;
                    for (old, &new) in previous.iter_mut().zip(&state) {
                        let joined = *old | new;
                        changed |= joined != *old;
                        *old = joined;
                    }
                    changed
                } else {
                    states[next] = Some(state.clone());
                    true
                };
                if changed {
                    queue.push_back(next);
                }
            }
        }
        for (i, state) in states.into_iter().enumerate() {
            let Some(mut state) = state else {
                continue;
            };
            let loads = cleanup_loads(&func.blocks[i].insts, &mut state, &slots, constants);
            if let ir::Term::Branch {
                cond,
                then_block,
                else_block,
            } = func.blocks[i].term
            {
                match loads.get(&cond) {
                    Some(2) => self.blocks[i].successors = vec![else_block.0 as usize],
                    Some(4) => self.blocks[i].successors = vec![then_block.0 as usize],
                    _ => {}
                }
            }
        }
    }
}


impl FnCtx {
    pub(crate) fn lifetime_scope_end(&mut self, scope: crate::sema::ScopeId, span: Span) {
        if !self.type_only
            && let Some(record) = &mut self.lifetimes
        {
            record.push(
                self.b.current,
                LifetimeEvent::ScopeEnd {
                    scope,
                    span,
                },
            );
        }
    }
}

// Only direct, uniquely addressed one-byte slots can be propagated without alias analysis.
pub(super) fn private_byte_flags(func: &ir::Func) -> Vec<Val> {
    let mut addresses = HashMap::<ir::SlotId, Vec<Val>>::default();
    for inst in func.blocks.iter().flat_map(|b| &b.insts) {
        if let ir::Inst::SlotAddr {
            dst,
            slot,
        } = inst
            && func.slots[slot.0 as usize].size == 1
        {
            addresses.entry(*slot).or_default().push(*dst);
        }
    }
    let mut flags: HashSet<_> = addresses
        .values()
        .filter_map(|v| match v.as_slice() {
            [v] => Some(*v),
            _ => None,
        })
        .collect();
    for block in &func.blocks {
        for inst in &block.insts {
            match inst {
                ir::Inst::Load {
                    ty: ir::Ty::I8, ..
                } => {}
                ir::Inst::Store {
                    ty: ir::Ty::I8,
                    value,
                    ..
                } => {
                    flags.remove(value);
                }
                _ => ir::inst_vals(inst, &mut |_| {}, &mut |v| {
                    flags.remove(&v);
                }),
            }
        }
        ir::term_uses(&block.term, &mut |v| {
            flags.remove(&v);
        });
    }
    flags.into_iter().collect()
}

fn cleanup_loads(
    insts: &[ir::Inst],
    state: &mut [u8],
    slots: &HashMap<Val, usize>,
    constants: &HashMap<Val, u64>,
) -> HashMap<Val, u8> {
    let mut loads = HashMap::default();
    for inst in insts {
        match inst {
            ir::Inst::Store {
                addr,
                value,
                ..
            } => {
                if let Some(&slot) = slots.get(addr) {
                    state[slot] = constants
                        .get(value)
                        .map_or(1, |v| 2 << usize::from(*v != 0));
                }
            }
            ir::Inst::Load {
                addr,
                dst,
                ..
            } => {
                if let Some(&slot) = slots.get(addr) {
                    loads.insert(*dst, state[slot]);
                }
            }
            ir::Inst::Bin {
                dst,
                op: ir::BinOp::Xor,
                ty: ir::Ty::I8,
                a,
                b,
            } => {
                let input = if constants.get(a) == Some(&1) {
                    loads.get(b)
                } else if constants.get(b) == Some(&1) {
                    loads.get(a)
                } else {
                    None
                };
                if let Some(&v) = input {
                    loads.insert(*dst, (v & 1) | ((v & 2) << 1) | ((v & 4) >> 1));
                }
            }
            ir::Inst::Cmp {
                dst,
                op: ir::CmpOp::Eq | ir::CmpOp::Ne,
                a,
                b,
                ..
            } => {
                let input = if constants.get(a) == Some(&0) {
                    loads.get(b)
                } else if constants.get(b) == Some(&0) {
                    loads.get(a)
                } else {
                    None
                };
                if let Some(&value) = input {
                    let inverted = matches!(
                        inst,
                        ir::Inst::Cmp {
                            op: ir::CmpOp::Eq,
                            ..
                        }
                    );
                    loads.insert(
                        *dst,
                        if inverted {
                            (value & 1) | ((value & 2) << 1) | ((value & 4) >> 1)
                        } else {
                            value
                        },
                    );
                }
            }
            _ => {}
        }
    }
    loads
}


impl FnCtx {
    pub(crate) fn lifetime_enter_site(&mut self, site: Option<Span>) -> Option<Span> {
        let record = self.lifetimes.as_mut()?;
        let previous = record.source_site;
        record.source_site = previous.or(site);
        previous
    }

    pub(crate) fn lifetime_restore_site(&mut self, site: Option<Span>) {
        if let Some(record) = &mut self.lifetimes {
            record.source_site = site;
        }
    }
}

/// Anonymous literal/argument stores use raw address arithmetic instead of member events.
/// Only addresses derived from this activation's allocations qualify, never loaded pointers.
fn stack_places(func: &ir::Func, subplaces: Vec<Val>) -> Vec<Val> {
    let mut stack: HashSet<Val> = subplaces.into_iter().collect();
    let mut derived: HashMap<Val, Vec<Val>> = HashMap::default();
    for instruction in func.blocks.iter().flat_map(|b| &b.insts) {
        match *instruction {
            ir::Inst::SlotAddr {
                dst, ..
            }
            | ir::Inst::StackAlloc {
                dst, ..
            } => {
                stack.insert(dst);
            }
            ir::Inst::PtrAdd {
                dst,
                base,
                ..
            } => derived.entry(base).or_default().push(dst),
            _ => {}
        }
    }
    let mut pending: Vec<_> = stack.iter().copied().collect();
    while let Some(base) = pending.pop() {
        for &dst in derived.get(&base).into_iter().flatten() {
            if stack.insert(dst) {
                pending.push(dst);
            }
        }
    }
    let mut stack: Vec<_> = stack.into_iter().collect();
    stack.sort_unstable_by_key(|v| v.0);
    stack
}

/// Retain original allocations, never the transitive derived-address closure.
fn stack_roots(func: &ir::Func) -> Vec<LifetimeStackRoot> {
    let constants: HashMap<_, _> = func
        .blocks
        .iter()
        .flat_map(|b| &b.insts)
        .filter_map(|inst| match inst {
            ir::Inst::IConst {
                dst,
                ty: ir::Ty::I64,
                value,
            } if i64::try_from(*value).is_ok() => Some((*dst, *value)),
            _ => None,
        })
        .collect();
    let mut slots = HashSet::default();
    let mut roots = Vec::new();
    for inst in func.blocks.iter().flat_map(|b| &b.insts) {
        match *inst {
            ir::Inst::SlotAddr {
                dst,
                slot,
            } if slots.insert(slot) => roots.push(LifetimeStackRoot {
                value: dst,
                bytes: Some(func.slots[slot.0 as usize].size),
            }),
            ir::Inst::StackAlloc {
                dst,
                size,
                ..
            } => roots.push(LifetimeStackRoot {
                value: dst,
                bytes: constants.get(&size).copied(),
            }),
            _ => {}
        }
    }
    roots.sort_unstable_by_key(|root| root.value.0);
    roots
}

fn slot_aliases(func: &ir::Func) -> Vec<(Val, Val)> {
    let mut slots = HashMap::default();
    let mut aliases = Vec::new();
    for inst in func.blocks.iter().flat_map(|b| &b.insts) {
        if let ir::Inst::SlotAddr {
            dst,
            slot,
        } = *inst
        {
            let canonical = *slots.entry(slot).or_insert(dst);
            if dst != canonical {
                aliases.push((dst, canonical));
            }
        }
    }
    aliases
}

#[cfg(test)]
mod stack_root_tests {
    use super::*;

    #[test]
    fn root_extents_require_original_allocations_and_known_nonnegative_sizes() {
        let mut b = ir::Builder::new(
            "stack_roots".into(),
            ir::Sig {
                params: vec![ir::Ty::I64],
                returns: Vec::new(),
                conv: ir::Conv::Jai,
                c_varargs: false,
                c_fixed: 0,
                c_abi: None,
            },
        );
        let slot = b.slot(8, 8);
        let root = b.slot_addr(slot);
        let alias = b.slot_addr(slot);
        let dynamic = b.ptr_add(root, b.param(0));
        let zero = b.iconst(ir::Ty::I64, 0);
        let negative = b.iconst(ir::Ty::I64, u64::MAX);
        let narrow = b.iconst(ir::Ty::I8, 8);
        let mut allocations = Vec::new();
        for size in [zero, negative, narrow, b.param(0)] {
            let dst = b.new_val(ir::Ty::Ptr);
            b.push(ir::Inst::StackAlloc {
                dst,
                size,
                align: 8,
            });
            allocations.push(dst);
        }
        let roots: HashMap<_, _> = stack_roots(&b.func)
            .into_iter()
            .map(|r| (r.value, r.bytes))
            .collect();
        assert_eq!(roots.len(), 5);
        assert_eq!(roots[&root], Some(8));
        assert!(!roots.contains_key(&alias));
        assert!(!roots.contains_key(&dynamic));
        assert_eq!(roots[&allocations[0]], Some(0));
        for allocation in &allocations[1..] {
            assert_eq!(roots[allocation], None);
        }
        assert_eq!(slot_aliases(&b.func), vec![(alias, root)]);
    }
}

#[cfg(test)]
mod immutable_region_tests {
    use super::*;

    #[test]
    fn selected_images_require_committed_read_only_exact_byte_storage() {
        let mut c = Compiler::new(
            crate::sema::Options::host(),
            std::rc::Rc::new(crate::sema::NativeFs),
        );
        let mut f = FnCtx::new(
            "image".into(),
            ir::Sig {
                params: Vec::new(),
                returns: Vec::new(),
                conv: ir::Conv::Jai,
                c_varargs: false,
                c_fixed: 0,
                c_abi: None,
            },
            crate::source::FileId(0),
        );
        f.lifetimes = Some(Box::default());
        for (read_only, size, len, want) in [
            (true, 8, 8, true),
            (false, 8, 8, false),
            (true, 4, 4, false),
            (true, 8, 7, false),
        ] {
            let source = c.out.program.add_global(ir::Global {
                name: "image".into(),
                size,
                align: 8,
                init: vec![0; len],
                relocs: Vec::new(),
                read_only,
                export: None,
            });
            let value = f.b.global_addr(source);
            let before = f.lifetimes.as_ref().unwrap().read_only_regions.len();
            c.lifetime_read_only_region(&mut f, value, TypeId::S64, source);
            assert_eq!(
                f.lifetimes.as_ref().unwrap().read_only_regions.len(),
                before + usize::from(want)
            );
        }
        assert!(
            f.lifetimes
                .as_ref()
                .unwrap()
                .read_only_regions
                .iter()
                .all(|region| region.leaves.is_empty() && !region.complete),
            "runtime immutability alone must not freeze compiler-finalized numeric values"
        );
        let before = f.lifetimes.as_ref().unwrap().read_only_regions.clone();
        c.lifetime_read_only_region(&mut f, Val(0), TypeId::S64, ir::GlobalId(u32::MAX));
        assert_eq!(before, f.lifetimes.as_ref().unwrap().read_only_regions);
        f.type_only = true;
        c.lifetime_read_only_region(&mut f, Val(0), TypeId::S64, ir::GlobalId(0));
        assert_eq!(before, f.lifetimes.as_ref().unwrap().read_only_regions);
    }
}

#[cfg(test)]
mod report_tests {
    use super::*;

    #[test]
    fn runtime_report_barriers_obey_recording_and_type_only() {
        let mut f = FnCtx::new(
            "report_barrier".into(),
            ir::Sig {
                params: Vec::new(),
                returns: Vec::new(),
                conv: ir::Conv::Jai,
                c_varargs: false,
                c_fixed: 0,
                c_abi: None,
            },
            crate::source::FileId(0),
        );
        f.lifetimes = Some(Box::default());
        f.type_only = true;
        f.lifetime_barrier();
        assert!(f.lifetimes.as_ref().unwrap().blocks.is_empty());
        f.type_only = false;
        let block = f.b.new_block();
        f.b.switch_to(block);
        f.lifetime_barrier();
        let record = f.lifetimes.as_ref().unwrap();
        assert!(record.blocks[0].events.is_empty());
        assert!(matches!(
            record.blocks[block.0 as usize].events.as_slice(),
            [LifetimeEvent::Barrier]
        ));
        f.lifetimes = None;
        f.lifetime_barrier();
        assert!(f.lifetimes.is_none());
    }
}
