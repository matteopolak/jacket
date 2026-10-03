//! Compiler-only control receipts. Code never enters runtime values or signatures.
use crate::{Error, LimitKind};
use jai_ir::{
    Call, ContextDefinition, ExpressionBindingId, Global, Places, ProcedureId, ValueExpr,
};
use jai_types::{TypeId, TypeKind, TypeView};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_PLAN: AtomicU64 = AtomicU64::new(1);

/// Identity of one compiler plan, independent of native procedure identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CompilerCodePlanId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CompilerControlId {
    plan: CompilerCodePlanId,
    index: usize,
}

impl CompilerControlId {
    pub fn plan(self) -> CompilerCodePlanId {
        self.plan
    }
    pub fn index(self) -> usize {
        self.index
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CompilerSlotId {
    plan: CompilerCodePlanId,
    index: usize,
}
impl CompilerSlotId {
    pub fn plan(self) -> CompilerCodePlanId {
        self.plan
    }
    pub fn index(self) -> usize {
        self.index
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CompilerSlotSchema {
    pub ty: TypeId,
}
#[derive(Clone, Copy, Debug)]
pub struct CompilerRuntimeInput {
    pub binding: ExpressionBindingId,
    pub slot: CompilerSlotId,
    pub ty: TypeId,
}

/// Selects source metadata retained by the compiler, never a VM `Value`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CompilerReturnSiteId {
    plan: CompilerCodePlanId,
    index: usize,
}
impl CompilerReturnSiteId {
    pub fn plan(self) -> CompilerCodePlanId {
        self.plan
    }
    pub fn index(self) -> usize {
        self.index
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CompilerCodePlanLimits {
    pub controls: usize,
    pub runtime_leaves: usize,
    pub return_sites: usize,
    pub edges: usize,
    pub slots: usize,
    pub inputs: usize,
    pub proof_facts: usize,
}
impl Default for CompilerCodePlanLimits {
    fn default() -> Self {
        Self {
            controls: 4096,
            runtime_leaves: 4096,
            return_sites: 4096,
            edges: 8192,
            slots: 4096,
            inputs: 8192,
            proof_facts: 65536,
        }
    }
}

#[derive(Debug)]
pub enum CompilerRuntimeLeafKind {
    Expression(ValueExpr),
    Call(Call),
}
/// Even a rejected or abandoned leaf retires an arbitrary staging AST iteratively.
#[derive(Debug)]
pub struct CompilerRuntimeLeaf {
    kind: Option<CompilerRuntimeLeafKind>,
    inputs: Box<[CompilerRuntimeInput]>,
}
impl CompilerRuntimeLeaf {
    pub fn expression(value: ValueExpr) -> Self {
        Self {
            kind: Some(CompilerRuntimeLeafKind::Expression(value)),
            inputs: Box::new([]),
        }
    }
    pub fn call(value: Call) -> Self {
        Self {
            kind: Some(CompilerRuntimeLeafKind::Call(value)),
            inputs: Box::new([]),
        }
    }
    pub fn with_inputs(mut self, inputs: Vec<CompilerRuntimeInput>) -> Self {
        self.inputs = inputs.into_boxed_slice();
        self
    }
    pub fn inputs(&self) -> &[CompilerRuntimeInput] {
        &self.inputs
    }
    pub fn kind(&self) -> &CompilerRuntimeLeafKind {
        self.kind.as_ref().expect("live compiler runtime leaf")
    }
}
impl Drop for CompilerRuntimeLeaf {
    fn drop(&mut self) {
        match self.kind.take() {
            Some(CompilerRuntimeLeafKind::Expression(value)) => {
                jai_ir::discard_value_expression(value)
            }
            Some(CompilerRuntimeLeafKind::Call(value)) => jai_ir::discard_call(value),
            None => {}
        }
    }
}
#[derive(Debug)]
pub enum CompilerControl {
    Evaluate(usize),
    Assign {
        slot: CompilerSlotId,
        leaf: usize,
    },
    Block {
        children: Box<[CompilerControlId]>,
        locals: Box<[CompilerSlotId]>,
    },
    If {
        condition: usize,
        then_control: CompilerControlId,
        else_control: CompilerControlId,
    },
    Return(CompilerReturnSiteId),
}

/// Staging is bounded and append-only; children must already belong to this plan.
pub struct CompilerCodePlanBuilder {
    id: CompilerCodePlanId,
    limits: CompilerCodePlanLimits,
    controls: Vec<CompilerControl>,
    runtime: Vec<CompilerRuntimeLeaf>,
    sites: usize,
    captures: Vec<Box<[CompilerSlotId]>>,
    edges: usize,
    slots: Vec<CompilerSlotSchema>,
    inputs: usize,
}
impl CompilerCodePlanBuilder {
    pub fn new(limits: CompilerCodePlanLimits) -> Result<Self, Error> {
        let id = NEXT_PLAN
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| Error::InvalidIr("compiler Code plan identity space exhausted"))?;
        Ok(Self {
            id: CompilerCodePlanId(id),
            limits,
            controls: vec![],
            runtime: vec![],
            sites: 0,
            captures: vec![],
            edges: 0,
            slots: vec![],
            inputs: 0,
        })
    }
    pub fn id(&self) -> CompilerCodePlanId {
        self.id
    }
    pub fn slot(&mut self, ty: TypeId) -> Result<CompilerSlotId, Error> {
        if self.slots.len() >= self.limits.slots {
            return Err(node_limit());
        }
        let id = CompilerSlotId {
            plan: self.id,
            index: self.slots.len(),
        };
        self.slots.push(CompilerSlotSchema {
            ty,
        });
        Ok(id)
    }
    fn owns_slot(&self, slot: CompilerSlotId) -> Result<(), Error> {
        if slot.plan != self.id || slot.index >= self.slots.len() {
            return Err(Error::InvalidIr("compiler slot belongs to another plan"));
        }
        Ok(())
    }
    pub fn reserve_return_site(&mut self) -> Result<CompilerReturnSiteId, Error> {
        self.reserve_return_site_with_captures(vec![])
    }
    pub fn reserve_return_site_with_captures(
        &mut self,
        captures: Vec<CompilerSlotId>,
    ) -> Result<CompilerReturnSiteId, Error> {
        for &slot in &captures {
            self.owns_slot(slot)?;
        }
        let mut unique = std::collections::HashSet::new();
        if captures.iter().any(|slot| !unique.insert(*slot)) {
            return Err(Error::InvalidIr("duplicate compiler return capture"));
        }
        let inputs = self
            .inputs
            .checked_add(captures.len())
            .filter(|&n| n <= self.limits.inputs)
            .ok_or_else(node_limit)?;
        if self.sites >= self.limits.return_sites {
            return Err(node_limit());
        }
        self.inputs = inputs;
        let site = CompilerReturnSiteId {
            plan: self.id,
            index: self.sites,
        };
        self.sites += 1;
        self.captures.push(captures.into_boxed_slice());
        Ok(site)
    }
    fn owns_control(&self, control: CompilerControlId) -> Result<(), Error> {
        if control.plan != self.id || control.index >= self.controls.len() {
            return Err(Error::InvalidIr("compiler control belongs to another plan"));
        }
        Ok(())
    }
    fn control(&mut self, control: CompilerControl) -> Result<CompilerControlId, Error> {
        if self.controls.len() >= self.limits.controls {
            return Err(node_limit());
        }
        let id = CompilerControlId {
            plan: self.id,
            index: self.controls.len(),
        };
        self.controls.push(control);
        Ok(id)
    }
    fn runtime(&mut self, leaf: CompilerRuntimeLeaf) -> Result<usize, Error> {
        if self.runtime.len() >= self.limits.runtime_leaves {
            return Err(node_limit());
        }
        let inputs = self
            .inputs
            .checked_add(leaf.inputs.len())
            .filter(|&n| n <= self.limits.inputs)
            .ok_or_else(node_limit)?;
        for input in leaf.inputs() {
            self.owns_slot(input.slot)?;
            if self.slots[input.slot.index].ty != input.ty {
                return Err(Error::InvalidIr("compiler input slot type mismatch"));
            }
        }
        let index = self.runtime.len();
        self.runtime.push(leaf);
        self.inputs = inputs;
        Ok(index)
    }
    pub fn evaluate(&mut self, leaf: CompilerRuntimeLeaf) -> Result<CompilerControlId, Error> {
        if self.controls.len() >= self.limits.controls {
            return Err(node_limit());
        }
        let leaf = self.runtime(leaf)?;
        self.control(CompilerControl::Evaluate(leaf))
    }
    pub fn assign(
        &mut self,
        slot: CompilerSlotId,
        leaf: CompilerRuntimeLeaf,
    ) -> Result<CompilerControlId, Error> {
        self.owns_slot(slot)?;
        if self.controls.len() >= self.limits.controls {
            return Err(node_limit());
        }
        let leaf = self.runtime(leaf)?;
        self.control(CompilerControl::Assign {
            slot,
            leaf,
        })
    }
    pub fn return_code(&mut self, site: CompilerReturnSiteId) -> Result<CompilerControlId, Error> {
        if site.plan != self.id || site.index >= self.sites {
            return Err(Error::InvalidIr(
                "compiler Code return site belongs to another plan",
            ));
        }
        self.control(CompilerControl::Return(site))
    }
    pub fn block(&mut self, children: Vec<CompilerControlId>) -> Result<CompilerControlId, Error> {
        self.block_with_locals(children, vec![])
    }
    pub fn block_with_locals(
        &mut self,
        children: Vec<CompilerControlId>,
        locals: Vec<CompilerSlotId>,
    ) -> Result<CompilerControlId, Error> {
        for &slot in &locals {
            self.owns_slot(slot)?;
        }
        let mut unique = std::collections::HashSet::new();
        if locals.iter().any(|slot| !unique.insert(*slot)) {
            return Err(Error::InvalidIr("duplicate compiler block local"));
        }
        for &child in &children {
            self.owns_control(child)?;
        }
        let edges = self
            .edges
            .checked_add(children.len())
            .and_then(|n| n.checked_add(locals.len()))
            .filter(|&n| n <= self.limits.edges)
            .ok_or_else(node_limit)?;
        let id = self.control(CompilerControl::Block {
            children: children.into_boxed_slice(),
            locals: locals.into_boxed_slice(),
        })?;
        self.edges = edges;
        Ok(id)
    }
    pub fn branch(
        &mut self,
        condition: ValueExpr,
        then_control: CompilerControlId,
        else_control: CompilerControlId,
    ) -> Result<CompilerControlId, Error> {
        self.branch_leaf(
            CompilerRuntimeLeaf::expression(condition),
            then_control,
            else_control,
        )
    }
    pub fn branch_leaf(
        &mut self,
        condition: CompilerRuntimeLeaf,
        then_control: CompilerControlId,
        else_control: CompilerControlId,
    ) -> Result<CompilerControlId, Error> {
        self.owns_control(then_control)?;
        self.owns_control(else_control)?;
        if self.controls.len() >= self.limits.controls {
            return Err(node_limit());
        }
        let edges = self
            .edges
            .checked_add(2)
            .filter(|&n| n <= self.limits.edges)
            .ok_or_else(node_limit)?;
        let condition = self.runtime(condition)?;
        let id = self.control(CompilerControl::If {
            condition,
            then_control,
            else_control,
        })?;
        self.edges = edges;
        Ok(id)
    }
    pub fn finish(self, root: CompilerControlId) -> Result<CompilerCodePlan, Error> {
        self.owns_control(root)?;
        let plan = CompilerCodePlan {
            id: self.id,
            controls: self.controls,
            runtime: self.runtime,
            sites: self.sites,
            slots: self.slots,
            captures: self.captures,
            proof_facts: self.limits.proof_facts,
            root,
        };
        plan.validate_control()?;
        Ok(plan)
    }
}
fn node_limit() -> Error {
    Error::Limit(LimitKind::ValueCells)
}
fn native_proof_error(error: jai_ir::IrError) -> Error {
    match error {
        jai_ir::IrError::Type(error) => Error::Type(error),
        error => Error::IrValidation(error.to_string()),
    }
}

/// No procedure signature, local ABI, pointer, or runtime result encodes Code.
pub struct CompilerCodePlan {
    id: CompilerCodePlanId,
    controls: Vec<CompilerControl>,
    runtime: Vec<CompilerRuntimeLeaf>,
    sites: usize,
    slots: Vec<CompilerSlotSchema>,
    captures: Vec<Box<[CompilerSlotId]>>,
    proof_facts: usize,
    root: CompilerControlId,
}
impl CompilerCodePlan {
    pub fn id(&self) -> CompilerCodePlanId {
        self.id
    }
    pub fn root(&self) -> CompilerControlId {
        self.root
    }
    pub fn return_site_count(&self) -> usize {
        self.sites
    }
    pub fn return_site_captures(&self, site: CompilerReturnSiteId) -> Option<&[CompilerSlotId]> {
        self.owns_site(site)
            .then(|| self.captures[site.index].as_ref())
    }
    pub fn control(&self, id: CompilerControlId) -> Option<&CompilerControl> {
        (id.plan == self.id)
            .then(|| self.controls.get(id.index))
            .flatten()
    }
    pub fn controls(&self) -> &[CompilerControl] {
        &self.controls
    }
    pub fn slots(&self) -> &[CompilerSlotSchema] {
        &self.slots
    }
    pub fn slot_schema(&self, slot: CompilerSlotId) -> Option<&CompilerSlotSchema> {
        (slot.plan == self.id)
            .then(|| self.slots.get(slot.index))
            .flatten()
    }
    pub fn metadata_size(&self) -> Result<usize, Error> {
        let mut size = self
            .controls
            .len()
            .checked_add(self.slots.len())
            .ok_or_else(node_limit)?;
        for leaf in &self.runtime {
            size = size
                .checked_add(1 + leaf.inputs.len())
                .ok_or_else(node_limit)?;
        }
        for control in &self.controls {
            let count = match control {
                CompilerControl::Block {
                    children,
                    locals,
                } => children
                    .len()
                    .checked_add(locals.len())
                    .ok_or_else(node_limit)?,
                CompilerControl::If {
                    ..
                } => 2,
                _ => 0,
            };
            size = size.checked_add(count).ok_or_else(node_limit)?;
        }
        for captures in &self.captures {
            size = size.checked_add(captures.len()).ok_or_else(node_limit)?;
        }
        size.checked_add(self.sites).ok_or_else(node_limit)
    }
    pub fn runtime_leaves(&self) -> &[CompilerRuntimeLeaf] {
        &self.runtime
    }
    pub fn owns_site(&self, site: CompilerReturnSiteId) -> bool {
        site.plan == self.id && site.index < self.sites
    }
    fn validate_control(&self) -> Result<(), Error> {
        let mut returns = Vec::with_capacity(self.controls.len());
        for (index, control) in self.controls.iter().enumerate() {
            let child = |id: CompilerControlId| -> Result<bool, Error> {
                if id.plan != self.id || id.index >= index {
                    return Err(Error::InvalidIr(
                        "compiler control graph is not an owned acyclic graph",
                    ));
                }
                Ok(returns[id.index])
            };
            let returns_code = match control {
                CompilerControl::Evaluate(leaf) => {
                    if *leaf >= self.runtime.len() {
                        return Err(Error::InvalidIr("compiler runtime leaf is unknown"));
                    }
                    false
                }
                CompilerControl::Assign {
                    slot,
                    leaf,
                } => {
                    if self.slot_schema(*slot).is_none() || *leaf >= self.runtime.len() {
                        return Err(Error::InvalidIr("compiler assignment identity is unknown"));
                    }
                    false
                }
                CompilerControl::Return(site) => {
                    if !self.owns_site(*site) {
                        return Err(Error::InvalidIr("compiler return site is unknown"));
                    }
                    true
                }
                CompilerControl::Block {
                    children,
                    locals,
                } => {
                    for slot in locals {
                        if self.slot_schema(*slot).is_none() {
                            return Err(Error::InvalidIr("compiler block slot is unknown"));
                        }
                    }
                    let mut terminates = false;
                    for &id in children {
                        terminates |= child(id)?;
                    }
                    terminates
                }
                CompilerControl::If {
                    condition,
                    then_control,
                    else_control,
                } => {
                    if !matches!(
                        self.runtime.get(*condition).map(CompilerRuntimeLeaf::kind),
                        Some(CompilerRuntimeLeafKind::Expression(_))
                    ) {
                        return Err(Error::InvalidIr("compiler condition is not an expression"));
                    }
                    let then_returns = child(*then_control)?;
                    let else_returns = child(*else_control)?;
                    then_returns && else_returns
                }
            };
            returns.push(returns_code);
        }
        if self.root.plan != self.id || !returns.get(self.root.index).copied().unwrap_or(false) {
            return Err(Error::InvalidIr(
                "compiler Code plan may finish without returning Code",
            ));
        }
        self.validate_initialization()
    }
    fn validate_initialization(&self) -> Result<(), Error> {
        use std::collections::BTreeSet;
        #[derive(Default)]
        struct Facts {
            needs: BTreeSet<usize>,
            writes: BTreeSet<usize>,
            kills: BTreeSet<usize>,
            returns: bool,
        }
        let inputs = |index: usize| -> BTreeSet<usize> {
            self.runtime[index]
                .inputs()
                .iter()
                .map(|input| input.slot.index)
                .collect()
        };
        let mut facts: Vec<Facts> = Vec::with_capacity(self.controls.len());
        let mut retained = 0usize;
        for control in &self.controls {
            let next = match control {
                CompilerControl::Evaluate(leaf) => Facts {
                    needs: inputs(*leaf),
                    ..Facts::default()
                },
                CompilerControl::Assign {
                    slot,
                    leaf,
                } => Facts {
                    needs: inputs(*leaf),
                    writes: [slot.index].into(),
                    ..Facts::default()
                },
                CompilerControl::Return(site) => Facts {
                    needs: self.captures[site.index]
                        .iter()
                        .map(|slot| slot.index)
                        .collect(),
                    returns: true,
                    ..Facts::default()
                },
                CompilerControl::Block {
                    children,
                    locals,
                } => {
                    let mut next = Facts::default();
                    for child in children {
                        if next.returns {
                            break;
                        }
                        let child = &facts[child.index];
                        next.needs
                            .extend(child.needs.difference(&next.writes).copied());
                        for killed in &child.kills {
                            next.writes.remove(killed);
                        }
                        next.kills.extend(&child.kills);
                        next.writes.extend(&child.writes);
                        next.returns = child.returns;
                    }
                    if !next.returns {
                        for local in locals {
                            next.writes.remove(&local.index);
                            next.kills.insert(local.index);
                        }
                    }
                    next
                }
                CompilerControl::If {
                    condition,
                    then_control,
                    else_control,
                } => {
                    let a = &facts[then_control.index];
                    let b = &facts[else_control.index];
                    let mut next = Facts {
                        needs: inputs(*condition),
                        returns: a.returns && b.returns,
                        ..Facts::default()
                    };
                    next.needs.extend(&a.needs);
                    next.needs.extend(&b.needs);
                    match (a.returns, b.returns) {
                        (false, false) => {
                            next.writes
                                .extend(a.writes.intersection(&b.writes).copied());
                            next.kills.extend(a.kills.union(&b.kills).copied());
                        }
                        (true, false) => {
                            next.writes.extend(&b.writes);
                            next.kills.extend(&b.kills);
                        }
                        (false, true) => {
                            next.writes.extend(&a.writes);
                            next.kills.extend(&a.kills);
                        }
                        (true, true) => {}
                    }
                    next
                }
            };
            retained = retained
                .checked_add(next.needs.len())
                .and_then(|n| n.checked_add(next.writes.len()))
                .and_then(|n| n.checked_add(next.kills.len()))
                .filter(|&n| n <= self.proof_facts)
                .ok_or_else(node_limit)?;
            facts.push(next);
        }
        if !facts[self.root.index].needs.is_empty() {
            return Err(Error::InvalidIr(
                "compiler slot may be read before initialization",
            ));
        }
        Ok(())
    }
    /// The ambient owner belongs to an existing source expression context. It is
    /// not the compiler Code function and does not give that function a native ABI.
    #[allow(clippy::too_many_arguments)]
    pub fn verify<'a>(
        &'a self,
        types: &'a dyn TypeView,
        signatures: &'a HashMap<ProcedureId, TypeId>,
        globals: &'a [Global],
        places: &'a Places,
        ambient_owner: ProcedureId,
        context: Option<&'a ContextDefinition>,
    ) -> Result<CheckedCompilerCodePlan<'a>, Error> {
        self.validate_control()?;
        for slot in &self.slots {
            jai_ir::verify_compiler_slot_type(types, slot.ty).map_err(native_proof_error)?;
        }
        let mut runtime = Vec::with_capacity(self.runtime.len());
        for leaf in &self.runtime {
            let bindings: Vec<_> = leaf
                .inputs()
                .iter()
                .map(|input| (input.binding, input.ty))
                .collect();
            let proof = match leaf.kind() {
                CompilerRuntimeLeafKind::Expression(expression) => {
                    CheckedCompilerRuntimeLeaf::Expression(
                        jai_ir::verify_compiler_bound_expression_with_context(
                            types,
                            expression,
                            signatures,
                            globals,
                            places,
                            ambient_owner,
                            context,
                            &bindings,
                        )
                        .map_err(native_proof_error)?,
                    )
                }
                CompilerRuntimeLeafKind::Call(call) => CheckedCompilerRuntimeLeaf::Call(
                    jai_ir::verify_compiler_bound_call_with_context(
                        types,
                        call,
                        signatures,
                        globals,
                        places,
                        ambient_owner,
                        context,
                        &bindings,
                    )
                    .map_err(native_proof_error)?,
                ),
            };
            runtime.push(proof);
        }
        for control in &self.controls {
            if let CompilerControl::Assign {
                slot,
                leaf,
            } = control
            {
                let results = match &runtime[*leaf] {
                    CheckedCompilerRuntimeLeaf::Expression(proof) => {
                        vec![proof.expression().type_id(types)]
                    }
                    CheckedCompilerRuntimeLeaf::Call(proof) => proof.results().to_vec(),
                };
                if results.as_slice() != [self.slots[slot.index].ty] {
                    return Err(Error::InvalidIr(
                        "compiler assignment result type or count mismatch",
                    ));
                }
            }
            if let CompilerControl::If {
                condition, ..
            } = control
            {
                let CompilerRuntimeLeafKind::Expression(value) = self.runtime[*condition].kind()
                else {
                    unreachable!()
                };
                if !matches!(types.kind(value.type_id(types))?, TypeKind::Bool) {
                    return Err(Error::InvalidIr(
                        "compiler Code branch condition is not Boolean",
                    ));
                }
            }
        }
        Ok(CheckedCompilerCodePlan {
            plan: self,
            runtime,
            ambient_owner,
        })
    }
}

pub enum CheckedCompilerRuntimeLeaf<'a> {
    Expression(jai_ir::CheckedExpression<'a>),
    Call(jai_ir::CheckedCall<'a>),
}
/// Verification precedes beginning the compiler effect transaction.
pub struct CheckedCompilerCodePlan<'a> {
    plan: &'a CompilerCodePlan,
    runtime: Vec<CheckedCompilerRuntimeLeaf<'a>>,
    ambient_owner: ProcedureId,
}
impl<'a> CheckedCompilerCodePlan<'a> {
    pub fn plan(&self) -> &'a CompilerCodePlan {
        self.plan
    }
    pub fn runtime_leaves(&self) -> &[CheckedCompilerRuntimeLeaf<'a>] {
        &self.runtime
    }
    pub(crate) fn into_parts(
        self,
    ) -> (
        &'a CompilerCodePlan,
        Vec<CheckedCompilerRuntimeLeaf<'a>>,
        ProcedureId,
    ) {
        (self.plan, self.runtime, self.ambient_owner)
    }
    pub fn ambient_owner(&self) -> ProcedureId {
        self.ambient_owner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::{IntegerType, ScalarType, TypeRegistry};
    fn value(ty: TypeId) -> CompilerRuntimeLeaf {
        CompilerRuntimeLeaf::expression(ValueExpr::Zero(ty))
    }
    fn deep(ty: TypeId) -> ValueExpr {
        let mut expression = ValueExpr::Zero(ty);
        for _ in 0..20_000 {
            expression = ValueExpr::PointerCast {
                value: Box::new(expression),
                ty,
                mode: jai_types::CastMode::Unchecked,
            };
        }
        expression
    }
    #[test]
    fn unused_rejected_and_invalid_deep_leaves_retire_iteratively() {
        let types = TypeRegistry::new();
        let ty = types.scalar(ScalarType::Int(IntegerType::S64));
        drop(CompilerRuntimeLeaf::expression(deep(ty)));
        let mut builder = CompilerCodePlanBuilder::new(CompilerCodePlanLimits {
            controls: 0,
            ..Default::default()
        })
        .unwrap();
        assert!(
            builder
                .evaluate(CompilerRuntimeLeaf::expression(deep(ty)))
                .is_err()
        );
        let mut builder = CompilerCodePlanBuilder::new(Default::default()).unwrap();
        let site = builder.reserve_return_site().unwrap();
        let returned = builder.return_code(site).unwrap();
        builder
            .evaluate(CompilerRuntimeLeaf::expression(deep(ty)))
            .unwrap();
        let plan = builder.finish(returned).unwrap();
        assert!(
            plan.verify(
                &types,
                &HashMap::new(),
                &[],
                &Places::default(),
                ProcedureId::new(42),
                None
            )
            .is_err()
        );
        drop(plan);
    }
    #[test]
    fn native_slots_are_owned_and_initialized_before_return_capture() {
        let types = TypeRegistry::new();
        let ty = types.scalar(ScalarType::Int(IntegerType::S64));
        let mut builder = CompilerCodePlanBuilder::new(Default::default()).unwrap();
        let slot = builder.slot(ty).unwrap();
        let site = builder
            .reserve_return_site_with_captures(vec![slot])
            .unwrap();
        let returned = builder.return_code(site).unwrap();
        assert!(builder.finish(returned).is_err());
        let mut builder = CompilerCodePlanBuilder::new(Default::default()).unwrap();
        let slot = builder.slot(ty).unwrap();
        let write = builder.assign(slot, value(ty)).unwrap();
        let site = builder
            .reserve_return_site_with_captures(vec![slot])
            .unwrap();
        let returned = builder.return_code(site).unwrap();
        let root = builder.block(vec![write, returned]).unwrap();
        let plan = builder.finish(root).unwrap();
        assert!(
            plan.verify(
                &types,
                &HashMap::new(),
                &[],
                &Places::default(),
                ProcedureId::new(42),
                None
            )
            .is_ok()
        );
        let mut other = CompilerCodePlanBuilder::new(Default::default()).unwrap();
        assert!(other.assign(slot, value(ty)).is_err());
        assert!(other.return_code(site).is_err());
    }
    #[test]
    fn branch_merge_and_lexical_exit_do_not_initialize_a_capture() {
        let types = TypeRegistry::new();
        let ty = types.scalar(ScalarType::Int(IntegerType::S64));
        for lexical_exit in [false, true] {
            let mut builder = CompilerCodePlanBuilder::new(Default::default()).unwrap();
            let slot = builder.slot(ty).unwrap();
            let write = builder.assign(slot, value(ty)).unwrap();
            let prelude = if lexical_exit {
                builder.block_with_locals(vec![write], vec![slot]).unwrap()
            } else {
                let empty = builder.block(vec![]).unwrap();
                builder
                    .branch(
                        ValueExpr::Bool(jai_ir::BoolExpr::Constant(true)),
                        write,
                        empty,
                    )
                    .unwrap()
            };
            let site = builder
                .reserve_return_site_with_captures(vec![slot])
                .unwrap();
            let returned = builder.return_code(site).unwrap();
            let root = builder.block(vec![prelude, returned]).unwrap();
            assert!(builder.finish(root).is_err());
        }
    }
}
