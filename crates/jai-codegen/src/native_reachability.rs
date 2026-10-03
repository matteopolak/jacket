//! Runtime demand over checked procedure identities, independent of #run execution.
use jai_ir::*;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt,
};

/// Published libraries choose their roots explicitly; executable roots are entry points.
#[derive(Clone, Debug)]
pub enum Publication {
    AllBodies,
    Selected(Vec<ProcedureId>),
}
#[derive(Debug)]
pub enum Error {
    CompilerRequest { chain: Vec<ProcedureId> },
    CompileTimeOnly { chain: Vec<ProcedureId> },
    SourceContract { chain: Vec<ProcedureId> },
    MissingProcedure(ProcedureId),
    InvalidCheckedIr(String),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CompilerRequest {
                chain,
            } => {
                f.write_str(
                    "native runtime reaches compiler-only request through procedure chain ",
                )?;
                for (index, id) in chain.iter().enumerate() {
                    if index != 0 {
                        f.write_str(" -> ")?;
                    }
                    write!(f, "{}", id.index())?;
                }
                Ok(())
            }
            Self::CompileTimeOnly {
                chain,
            } => {
                f.write_str(
                    "native runtime reaches #compile_time procedure through procedure chain ",
                )?;
                for (index, id) in chain.iter().enumerate() {
                    if index != 0 {
                        f.write_str(" -> ")?;
                    }
                    write!(f, "{}", id.index())?;
                }
                Ok(())
            }
            Self::SourceContract {
                chain,
            } => {
                f.write_str(
                    "native executable reaches a bodyless source contract without a checked provider through procedure chain ",
                )?;
                for (index, id) in chain.iter().enumerate() {
                    if index != 0 {
                        f.write_str(" -> ")?;
                    }
                    write!(f, "{}", id.index())?;
                }
                Ok(())
            }
            Self::MissingProcedure(id) => write!(
                f,
                "native root or dependency procedure {} is absent",
                id.index()
            ),
            Self::InvalidCheckedIr(error) => write!(
                f,
                "native reachability encountered invalid checked IR: {error}"
            ),
        }
    }
}
impl std::error::Error for Error {
}
#[derive(Debug)]
pub struct Reachable {
    procedures: HashSet<ProcedureId>,
    globals: HashSet<GlobalId>,
}
#[derive(Clone, Copy)]
enum Purpose {
    Executable,
    LibraryObject,
}
impl Reachable {
    pub fn contains(&self, id: ProcedureId) -> bool {
        self.procedures.contains(&id)
    }
    pub fn contains_global(&self, id: GlobalId) -> bool {
        self.globals.contains(&id)
    }
    pub fn executable(library: &Library, entry: EntryPoint) -> Result<Self, Error> {
        Self::program(library, entry, Purpose::Executable)
    }
    /// Entry-root object publication preserves unresolved typed source contracts.
    /// A successful object publication does not prove an executable provider exists.
    pub fn object(library: &Library, entry: EntryPoint) -> Result<Self, Error> {
        Self::program(library, entry, Purpose::LibraryObject)
    }
    fn program(library: &Library, entry: EntryPoint, purpose: Purpose) -> Result<Self, Error> {
        Self::from_roots(
            library,
            purpose,
            [match entry {
                EntryPoint::Int(id) | EntryPoint::Void(id) => id,
            }]
            .into_iter()
            .chain(
                library
                    .program_exports()
                    .iter()
                    .filter_map(|export| match export.target {
                        ExportTarget::Procedure(id) => Some(id),
                        ExportTarget::Global(_) => None,
                    }),
            ),
        )
    }
    pub fn library(library: &Library, publication: &Publication) -> Result<Self, Error> {
        let mut roots = match publication {
            Publication::AllBodies => library
                .procedures()
                .iter()
                .filter(|procedure| {
                    library.procedure_phase(procedure.id)
                        == jai_types::ProcedureExecution::RuntimeAndCompileTime
                })
                .map(|procedure| procedure.id)
                .collect(),
            Publication::Selected(roots) => roots.clone(),
        };
        roots.extend(
            library
                .program_exports()
                .iter()
                .filter_map(|export| match export.target {
                    ExportTarget::Procedure(id) => Some(id),
                    ExportTarget::Global(_) => None,
                }),
        );
        roots.sort_by_key(|id| id.index());
        roots.dedup();
        Self::from_roots(library, Purpose::LibraryObject, roots)
    }
    fn from_roots(
        library: &Library,
        purpose: Purpose,
        roots: impl IntoIterator<Item = ProcedureId>,
    ) -> Result<Self, Error> {
        // These are all emitted regardless of body demand. Exhaustive scans keep
        // future static/global procedure-address variants from silently disappearing.
        let mut root_edges = HashSet::new();
        let mut globals = library
            .program_exports()
            .iter()
            .filter_map(|export| match export.target {
                ExportTarget::Global(id) => Some(id),
                ExportTarget::Procedure(_) => None,
            })
            .collect::<HashSet<_>>();
        let mut constants = Vec::new();
        for global in library.globals() {
            match global.initializer() {
                GlobalInitializer::Int(_)
                | GlobalInitializer::Bool(_)
                | GlobalInitializer::External(_) => {}
                GlobalInitializer::Value(value) => constants.push(Node::Constant(value)),
            }
        }
        if let Some(context) = library.context() {
            constants.push(Node::Constant(&context.default));
        }
        scan(library, None, constants, &mut root_edges, &mut globals)?;
        let mut roots: Vec<_> = roots.into_iter().chain(root_edges).collect();
        roots.sort_by_key(|id| id.index());
        roots.dedup();
        let mut pending = VecDeque::new();
        let mut parents = HashMap::new();
        let compiler_requests: HashSet<_> = library
            .prototypes()
            .iter()
            .filter_map(|prototype| match prototype.origin {
                PrototypeOrigin::Compiler => Some(prototype.id),
                PrototypeOrigin::Foreign {
                    ..
                }
                | PrototypeOrigin::SourceContract {
                    ..
                }
                | PrototypeOrigin::Intrinsic(_) => None,
            })
            .collect();
        let source_contracts: HashSet<_> = library
            .prototypes()
            .iter()
            .filter_map(|prototype| {
                matches!(prototype.origin, PrototypeOrigin::SourceContract { .. })
                    .then_some(prototype.id)
            })
            .collect();
        for root in roots {
            if parents.insert(root, None).is_none() {
                pending.push_back(root);
            }
        }
        while let Some(id) = pending.pop_front() {
            if !library.signatures().contains_key(&id) {
                return Err(Error::MissingProcedure(id));
            }
            let compile_time_only =
                library.procedure_phase(id) == jai_types::ProcedureExecution::CompileTimeOnly;
            let source_contract =
                matches!(purpose, Purpose::Executable) && source_contracts.contains(&id);
            if compiler_requests.contains(&id) || compile_time_only || source_contract {
                let mut chain = vec![id];
                let mut at = id;
                while let Some(Some(parent)) = parents.get(&at) {
                    chain.push(*parent);
                    at = *parent;
                }
                chain.reverse();
                return Err(if source_contract {
                    Error::SourceContract {
                        chain,
                    }
                } else if compile_time_only {
                    Error::CompileTimeOnly {
                        chain,
                    }
                } else {
                    Error::CompilerRequest {
                        chain,
                    }
                });
            }
            if let Some(procedure) = library.procedure_by_id(id) {
                let mut edges = HashSet::new();
                scan(
                    library,
                    Some(procedure),
                    vec![Node::Block(&procedure.body)],
                    &mut edges,
                    &mut globals,
                )?;
                let mut edges: Vec<_> = edges.into_iter().collect();
                edges.sort_by_key(|id| id.index());
                for edge in edges {
                    if let std::collections::hash_map::Entry::Vacant(entry) = parents.entry(edge) {
                        entry.insert(Some(id));
                        pending.push_back(edge);
                    }
                }
            }
        }
        Ok(Self {
            procedures: parents.into_keys().collect(),
            globals,
        })
    }
}

enum Node<'a> {
    Block(&'a Block),
    Statement(&'a Statement),
    Value(&'a ValueExpr),
    InstallBinding(ExpressionBindingId, &'a ValueExpr),
    RemoveBindings(&'a [(ExpressionBindingId, ValueExpr)]),
    Int(&'a IntExpr),
    Bool(&'a BoolExpr),
    Float(&'a FloatExpr),
    Place(Place),
    Call(&'a Call),
    Constant(&'a ConstantValue),
    Static(&'a StaticValue),
    Cleanup(CleanupId),
}
fn scan<'a>(
    library: &'a Library,
    procedure: Option<&'a Procedure>,
    mut pending: Vec<Node<'a>>,
    edges: &mut HashSet<ProcedureId>,
    globals: &mut HashSet<GlobalId>,
) -> Result<(), Error> {
    let mut places = HashSet::new();
    let mut phase_bindings = crate::execution_phase::PhaseBindings::new();
    let mut cleanups = HashSet::new();
    let mut statics = HashSet::new();
    while let Some(node) = pending.pop() {
        match node {
            Node::InstallBinding(binding, producer) => {
                let fact =
                    crate::execution_phase::native_value_condition(producer, &phase_bindings);
                phase_bindings.insert(binding, fact);
                places.clear();
            }
            Node::RemoveBindings(bindings) => {
                for (binding, _) in bindings.iter().rev() {
                    phase_bindings.remove(binding);
                }
                places.clear();
            }
            Node::Block(block) => {
                let count = block
                    .statements
                    .iter()
                    .position(|statement| {
                        crate::execution_phase::native_statement_terminates_with_bindings(
                            statement,
                            &phase_bindings,
                        )
                    })
                    .map_or(block.statements.len(), |index| index + 1);
                pending.extend(block.statements[..count].iter().rev().map(Node::Statement))
            }
            Node::Statement(statement) => match statement {
                Statement::Simd(block) => {
                    for instruction in block.instructions().iter().rev() {
                        match instruction {
                            jai_ir::SimdInstruction::Load {
                                address, ..
                            }
                            | jai_ir::SimdInstruction::Store {
                                address, ..
                            } => {
                                pending.push(Node::Value(address));
                            }
                            jai_ir::SimdInstruction::Add {
                                ..
                            }
                            | jai_ir::SimdInstruction::DebugTrap
                            | jai_ir::SimdInstruction::Arm64DebugTrap => {}
                        }
                    }
                }
                Statement::PushContext {
                    value,
                    body,
                    ..
                } => {
                    pending.push(Node::Block(body));
                    pending.push(Node::Value(value));
                }
                Statement::IndirectCallResults {
                    callee,
                    arguments,
                    destinations,
                    ..
                } => {
                    pending.push(Node::Value(callee));
                    pending.extend(arguments.iter().map(|(_, value)| Node::Value(value)));
                    pending.extend(destinations.iter().flatten().copied().map(Node::Place));
                }
                Statement::Store(place, value) => {
                    pending.push(Node::Place(*place));
                    pending.push(Node::Value(value));
                }
                Statement::DiscardValue(value) => pending.push(Node::Value(value)),
                Statement::CallResults {
                    call,
                    destinations,
                } => {
                    pending.push(Node::Call(call));
                    pending.extend(destinations.iter().flatten().copied().map(Node::Place));
                }
                Statement::StoreInt(place, value) => {
                    pending.push(Node::Place(place.place()));
                    pending.push(Node::Int(value));
                }
                Statement::StoreBool(place, value) => {
                    pending.push(Node::Place(place.place()));
                    pending.push(Node::Bool(value));
                }
                Statement::Exit(exit) => {
                    pending.extend(exit.cleanups.iter().copied().map(Node::Cleanup));
                    match &exit.transfer {
                        Transfer::ReturnValues(values) => {
                            pending.extend(values.iter().map(Node::Value))
                        }
                        Transfer::ReturnInt(value) => pending.push(Node::Int(value)),
                        Transfer::ReturnBool(value) => pending.push(Node::Bool(value)),
                        Transfer::ReturnVoid | Transfer::Break(_) | Transfer::Continue(_) => {}
                    }
                }
                Statement::Cleanup(id) => pending.push(Node::Cleanup(*id)),
                Statement::DiscardInt(value) => pending.push(Node::Int(value)),
                Statement::DiscardBool(value) => pending.push(Node::Bool(value)),
                Statement::CallVoid(call) => pending.push(Node::Call(call)),
                Statement::If(condition, yes, no) => {
                    pending.push(Node::Bool(condition));
                    match crate::execution_phase::native_condition_with_bindings(
                        condition,
                        &phase_bindings,
                    ) {
                        Some(true) => pending.push(Node::Block(yes)),
                        Some(false) => pending.push(Node::Block(no)),
                        None => {
                            pending.push(Node::Block(yes));
                            pending.push(Node::Block(no));
                        }
                    }
                }
                Statement::Cases(cases) => {
                    pending.push(Node::Statement(&cases.subject));
                    for arm in &cases.arms {
                        pending.push(Node::Bool(&arm.condition));
                        pending.push(Node::Block(&arm.body));
                    }
                    if let Some(block) = &cases.default {
                        pending.push(Node::Block(block));
                    }
                }
                Statement::While {
                    condition,
                    body,
                    ..
                } => {
                    let skip = match condition {
                        LoopCondition::Value(value) | LoopCondition::BoundBool(_, value) => {
                            crate::execution_phase::native_condition_with_bindings(
                                value,
                                &phase_bindings,
                            ) == Some(false)
                        }
                        LoopCondition::BoundInt(_, _) => false,
                    };
                    if !skip {
                        pending.push(Node::Block(body));
                    }
                    match condition {
                        LoopCondition::Value(value) | LoopCondition::BoundBool(_, value) => {
                            pending.push(Node::Bool(value))
                        }
                        LoopCondition::BoundInt(_, value) => pending.push(Node::Int(value)),
                    }
                }
                Statement::Range(range) => {
                    pending.push(Node::Int(&range.start));
                    pending.push(Node::Int(&range.end));
                    pending.push(Node::Block(&range.body));
                }
                Statement::Block(block) => pending.push(Node::Block(block)),
            },
            Node::Call(call) => {
                edges.insert(call.procedure);
                pending.extend(call.arguments.iter().map(|(_, value)| Node::Value(value)));
            }
            Node::Cleanup(id) => {
                if cleanups.insert(id) {
                    pending.push(Node::Block(
                        &procedure
                            .and_then(|procedure| procedure.cleanups.get(id.index()))
                            .ok_or_else(|| Error::InvalidCheckedIr("cleanup owner/index".into()))?
                            .body,
                    ));
                }
            }
            Node::Place(place) => {
                if places.insert(place) {
                    let arena = library.places();
                    match place.kind() {
                        PlaceKind::Global(id) => {
                            globals.insert(id);
                        }
                        PlaceKind::Local(_) | PlaceKind::Context(_) => {}
                        PlaceKind::Field(id) => {
                            let projection = arena.projection(id).map_err(invalid)?;
                            pending.push(Node::Place(projection.base));
                        }
                        PlaceKind::Dereference(id) => pending.push(Node::Value(
                            &arena.dereference(id).map_err(invalid)?.pointer,
                        )),
                        PlaceKind::Index(id) => {
                            let projection = arena.index(id).map_err(invalid)?;
                            pending.push(Node::Place(projection.base));
                            pending.push(Node::Int(&projection.index));
                        }
                        PlaceKind::SequenceField(id) => pending
                            .push(Node::Place(arena.sequence_field(id).map_err(invalid)?.base)),
                    }
                }
            }
            Node::Value(value) => match value {
                ValueExpr::StorageBitcast {
                    source, ..
                } => match source {
                    StorageBitcastSource::Place(place) => pending.push(Node::Place(*place)),
                    StorageBitcastSource::Value(value) => pending.push(Node::Value(value)),
                },
                ValueExpr::Bind {
                    bindings,
                    body,
                    ..
                } => {
                    pending.push(Node::RemoveBindings(bindings));
                    pending.push(Node::Value(body));
                    for (binding, producer) in bindings.iter().rev() {
                        pending.push(Node::InstallBinding(*binding, producer));
                        pending.push(Node::Value(producer));
                    }
                }
                ValueExpr::Bound {
                    ..
                } => {}
                ValueExpr::ProcedureValue {
                    procedure, ..
                } => {
                    edges.insert(*procedure);
                }
                ValueExpr::IndirectCall {
                    callee,
                    arguments,
                    ..
                } => {
                    pending.push(Node::Value(callee));
                    pending.extend(arguments.iter().map(|(_, value)| Node::Value(value)));
                }
                ValueExpr::Call {
                    call, ..
                } => pending.push(Node::Call(call)),
                ValueExpr::Int(value) => pending.push(Node::Int(value)),
                ValueExpr::Float(value) => pending.push(Node::Float(value)),
                ValueExpr::Bool(value) => pending.push(Node::Bool(value)),
                ValueExpr::Load(place)
                | ValueExpr::AddressOf {
                    place, ..
                }
                | ValueExpr::ArrayToSlice {
                    array: place, ..
                } => pending.push(Node::Place(*place)),
                ValueExpr::Conditional {
                    expression, ..
                } => {
                    pending.push(Node::Bool(&expression.condition));
                    match crate::execution_phase::native_condition_with_bindings(
                        &expression.condition,
                        &phase_bindings,
                    ) {
                        Some(true) => pending.push(Node::Value(&expression.then_value)),
                        Some(false) => pending.push(Node::Value(&expression.else_value)),
                        None => {
                            pending.push(Node::Value(&expression.then_value));
                            pending.push(Node::Value(&expression.else_value));
                        }
                    }
                }
                ValueExpr::Array {
                    elements, ..
                }
                | ValueExpr::Record {
                    fields: elements, ..
                } => pending.extend(elements.iter().map(Node::Value)),
                ValueExpr::OrderedRecord {
                    initializers, ..
                } => pending.extend(initializers.iter().map(|(_, value)| Node::Value(value))),
                ValueExpr::RecordBuild {
                    initializers, ..
                } => pending.extend(initializers.iter().map(|(_, value)| Node::Value(value))),
                ValueExpr::SequenceBuild {
                    initializers, ..
                } => pending.extend(initializers.iter().map(|(_, value)| Node::Value(value))),
                ValueExpr::SequenceConcat {
                    parts, ..
                } => {
                    for part in parts {
                        match part {
                            SequencePackPart::Element(value) | SequencePackPart::Spread(value) => {
                                pending.push(Node::Value(value));
                            }
                        }
                    }
                }
                ValueExpr::SequenceField {
                    base, ..
                }
                | ValueExpr::ArrayView {
                    array: base, ..
                }
                | ValueExpr::SequenceView {
                    sequence: base, ..
                }
                | ValueExpr::PointerCast {
                    value: base, ..
                }
                | ValueExpr::AddressOfValue {
                    value: base, ..
                }
                | ValueExpr::Distinct {
                    value: base, ..
                }
                | ValueExpr::UnwrapDistinct {
                    value: base, ..
                }
                | ValueExpr::Union {
                    value: base, ..
                }
                | ValueExpr::Field {
                    base, ..
                }
                | ValueExpr::TypeDescriptor {
                    value: base, ..
                } => pending.push(Node::Value(base)),
                ValueExpr::Index {
                    base,
                    index,
                    ..
                } => {
                    pending.push(Node::Value(base));
                    pending.push(Node::Int(index));
                }
                ValueExpr::PointerOffset {
                    pointer,
                    offset,
                    ..
                }
                | ValueExpr::PointerOffsetLeft {
                    pointer,
                    offset,
                    ..
                } => {
                    pending.push(Node::Value(pointer));
                    pending.push(Node::Int(offset));
                }
                ValueExpr::EnumFromInt {
                    value, ..
                }
                | ValueExpr::PointerFromInteger {
                    value, ..
                } => pending.push(Node::Int(value)),
                ValueExpr::StaticAddress {
                    data, ..
                } => {
                    if statics.insert(data.identity()) {
                        pending.extend(
                            data.objects()
                                .iter()
                                .map(|object| Node::Static(object.value())),
                        );
                    }
                }
                ValueExpr::RuntimeType(value) => {
                    let data = value.data();
                    if statics.insert(data.identity()) {
                        pending.extend(
                            data.objects()
                                .iter()
                                .map(|object| Node::Static(object.value())),
                        );
                    }
                }
                ValueExpr::NativePointer(_)
                | ValueExpr::Context {
                    ..
                }
                | ValueExpr::StringBytes {
                    ..
                }
                | ValueExpr::Zero(_)
                | ValueExpr::Enum {
                    ..
                } => {}
            },
            Node::Int(value) => match value.kind() {
                IntExprKind::Value(value)
                | IntExprKind::EnumValue(value)
                | IntExprKind::FromPointer {
                    value, ..
                } => pending.push(Node::Value(value)),
                IntExprKind::PointerDifference {
                    left,
                    right,
                } => {
                    pending.push(Node::Value(left));
                    pending.push(Node::Value(right));
                }
                IntExprKind::FromFloat(_, value) => pending.push(Node::Float(value)),
                IntExprKind::FromBool(value) => pending.push(Node::Bool(value)),
                IntExprKind::Cast(_, value)
                | IntExprKind::Negate(value)
                | IntExprKind::Complement(value) => pending.push(Node::Int(value)),
                IntExprKind::Binary(_, left, right) => {
                    pending.push(Node::Int(left));
                    pending.push(Node::Int(right));
                }
                IntExprKind::Conditional(value) => {
                    pending.push(Node::Bool(&value.condition));
                    match crate::execution_phase::native_condition_with_bindings(
                        &value.condition,
                        &phase_bindings,
                    ) {
                        Some(true) => pending.push(Node::Int(&value.then_value)),
                        Some(false) => pending.push(Node::Int(&value.else_value)),
                        None => {
                            pending.push(Node::Int(&value.then_value));
                            pending.push(Node::Int(&value.else_value));
                        }
                    }
                }
                IntExprKind::Load(place) => pending.push(Node::Place(place.place())),
                IntExprKind::Call(call) => pending.push(Node::Call(call)),
                IntExprKind::Constant(_) | IntExprKind::InvalidCheckedCast => {}
            },
            Node::Bool(value)
                if crate::execution_phase::native_boolean_with_bindings(value, &phase_bindings)
                    .is_some() => {}
            Node::Bool(value) => match value {
                BoolExpr::Value(value) | BoolExpr::FromPointer(value) => {
                    pending.push(Node::Value(value))
                }
                BoolExpr::FromInt(value) => pending.push(Node::Int(value)),
                BoolExpr::Load(place) => pending.push(Node::Place(place.place())),
                BoolExpr::Call(call) => pending.push(Node::Call(call)),
                BoolExpr::Not(value) => pending.push(Node::Bool(value)),
                BoolExpr::CompareInts(_, left, right) => {
                    pending.push(Node::Int(left));
                    pending.push(Node::Int(right));
                }
                BoolExpr::CompareFloats(_, left, right) => {
                    pending.push(Node::Float(left));
                    pending.push(Node::Float(right));
                }
                BoolExpr::ComparePointers(_, left, right)
                | BoolExpr::CompareStrings(_, left, right) => {
                    pending.push(Node::Value(left));
                    pending.push(Node::Value(right));
                }
                BoolExpr::CompareBools(_, left, right) => {
                    pending.push(Node::Bool(left));
                    pending.push(Node::Bool(right));
                }
                BoolExpr::And(left, right) | BoolExpr::Or(left, right) => {
                    pending.push(Node::Bool(left));
                    let bypass = matches!(
                        (
                            value,
                            crate::execution_phase::native_condition_with_bindings(
                                left,
                                &phase_bindings
                            )
                        ),
                        (BoolExpr::And(..), Some(false)) | (BoolExpr::Or(..), Some(true))
                    );
                    if !bypass {
                        pending.push(Node::Bool(right));
                    }
                }
                BoolExpr::Conditional(value) => {
                    pending.push(Node::Bool(&value.condition));
                    match crate::execution_phase::native_condition_with_bindings(
                        &value.condition,
                        &phase_bindings,
                    ) {
                        Some(true) => pending.push(Node::Bool(&value.then_value)),
                        Some(false) => pending.push(Node::Bool(&value.else_value)),
                        None => {
                            pending.push(Node::Bool(&value.then_value));
                            pending.push(Node::Bool(&value.else_value));
                        }
                    }
                }
                BoolExpr::Constant(_) | BoolExpr::CompileTime => {}
            },
            Node::Float(value) => match value.kind() {
                FloatExprKind::Value(value) => pending.push(Node::Value(value)),
                FloatExprKind::Load(place) => pending.push(Node::Place(*place)),
                FloatExprKind::Call(call) => pending.push(Node::Call(call)),
                FloatExprKind::Negate(value) | FloatExprKind::Cast(value) => {
                    pending.push(Node::Float(value))
                }
                FloatExprKind::Binary(_, left, right) => {
                    pending.push(Node::Float(left));
                    pending.push(Node::Float(right));
                }
                FloatExprKind::FromInt(value) => pending.push(Node::Int(value)),
                FloatExprKind::Conditional(value) => {
                    pending.push(Node::Bool(&value.condition));
                    match crate::execution_phase::native_condition_with_bindings(
                        &value.condition,
                        &phase_bindings,
                    ) {
                        Some(true) => pending.push(Node::Float(&value.then_value)),
                        Some(false) => pending.push(Node::Float(&value.else_value)),
                        None => {
                            pending.push(Node::Float(&value.then_value));
                            pending.push(Node::Float(&value.else_value));
                        }
                    }
                }
                FloatExprKind::Constant(_) => {}
            },
            Node::Constant(value) => match &value.kind {
                ConstantKind::RuntimeType(value) => {
                    let data = value.data();
                    if statics.insert(data.identity()) {
                        pending.extend(
                            data.objects()
                                .iter()
                                .map(|object| Node::Static(object.value())),
                        );
                    }
                }
                ConstantKind::Procedure(procedure) => {
                    edges.insert(*procedure);
                }
                ConstantKind::Record(values) | ConstantKind::Array(values) => {
                    pending.extend(values.iter().map(Node::Constant))
                }
                ConstantKind::Union {
                    value, ..
                }
                | ConstantKind::Distinct(value) => pending.push(Node::Constant(value)),
                ConstantKind::Int(_)
                | ConstantKind::NativePointer(_)
                | ConstantKind::Float(_)
                | ConstantKind::Bool(_)
                | ConstantKind::StringBytes(_)
                | ConstantKind::Enum(_)
                | ConstantKind::Zero => {}
            },
            Node::Static(value) => match &value.kind {
                StaticValueKind::Constant(value) => pending.push(Node::Constant(value)),
                StaticValueKind::Record(values) | StaticValueKind::Array(values) => {
                    pending.extend(values.iter().map(Node::Static))
                }
                StaticValueKind::Address(_)
                | StaticValueKind::Slice {
                    ..
                } => {}
            },
        }
    }
    Ok(())
}
fn invalid(error: IrError) -> Error {
    Error::InvalidCheckedIr(error.to_string())
}
