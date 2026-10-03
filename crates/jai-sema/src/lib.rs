//! Resolve names and construct a typed program before code generation.
mod any_values;
mod call_hints;
mod caller_locations;
mod calls;
mod cases;
mod cleanup;
mod context;
mod debug_capture;
mod declarations;
mod deprecation_warnings;
mod discarded_parameters;
mod floats;
mod local_declarations;
mod loops;
mod modules;
mod resolve_options;
mod runtime_defaults;
mod source_locations;
mod source_parameter_defaults;
mod storage_alignment;
mod string_comparison;
pub use resolve_options::ResolveOptions;
mod c_string_literals;
mod compile_time_cases;
mod compile_time_conditionals;
mod compiler_code;
mod constant_limits;
mod constant_queries;
mod enum_operators;
mod enum_values;
mod expression_bindings;
mod field_conversions;
mod inferred_casts;
mod iteration_removal;
pub mod metaprogram;
pub mod modifiers;
mod operator_overloads;
mod pointer_conversions;
mod pointers;
mod procedure_notes;
mod procedure_values;
mod record_default_overrides;
#[cfg(test)]
mod record_placements;
mod reflection;
mod restriction_facts;
mod result_obligations;
mod results;
mod runtime_type_values;
mod safety_checks;
mod sequence_loops;
mod sequences;
mod short_lambdas;
mod simd;
mod storage_casts;
mod type_values;
mod typed_constants;
mod using_bindings;
mod using_declarations;
mod using_directives;
mod value_conditionals;
use jai_eval::operators::Operator;
pub use jai_eval::operators::{Equality, IntOp, Relation};
use jai_eval::{Integer as IntegerValue, Value as ScalarConstant};
use jai_source::{Diagnostic, Span, Symbol, Symbols};
use jai_syntax::{self as syntax, BinaryOp, IntegerType, ReturnType, ScalarType, UnaryOp};
pub use modules::FileAbiBindingContext;
pub use modules::ProcessAbiBindingContext;
pub use modules::compiler_intrinsics::{
    CompilerBindingContext, CompilerModuleOrigin, CompilerModuleOrigins,
};
pub use modules::{
    DiscoveryCaseOutcome, DiscoveryCasePending, DiscoveryConditionOutcome,
    DiscoveryConditionPending, resolve_discovery_cases, resolve_discovery_conditions,
};
pub use modules::{
    DiscoveryInsertionDecision, DiscoveryInsertionOutcome, DiscoveryInsertionPending,
    InsertionAdmissionCallback,
};
pub use modules::{
    DiscoveryParameterOutcome, DiscoveryParameterPending, resolve_discovery_parameters,
};
pub use modules::{
    DiscoveryReadiness, PreparedDiscoveryOutcome, PreparedDiscoveryRequests,
    PreparedDiscoverySession,
};
pub use modules::{DiscoveryUsingOutcome, DiscoveryUsingPending, resolve_discovery_using};
pub use modules::{
    LibraryPending, LibraryReadiness, PreparedLibrarySession, SourcePrefixReadiness,
    SourcePreparationPending,
};
pub use modules::{resolve_graph, resolve_library, select_entry};
pub use modules::{resolve_graph_with_options, resolve_library_with_options};
pub mod compile_time;
pub mod overloads;
pub mod polymorphism;
use std::collections::HashMap;

pub use jai_ir::*;
#[cfg(test)]
mod ir_tests;
use jai_types::{CallingConvention, ContextMode, ProcedureType, TypeId, TypeRegistry};

#[derive(Clone, Debug)]
enum Binding {
    CompilerInput {
        binding: jai_ir::ExpressionBindingId,
        ty: TypeId,
    },
    Discarded(TypeId),
    LambdaPreview(short_lambdas::PreviewBinding),
    Macro(metaprogram::LocalMacroId),
    Namespace(jai_source::ModuleId),
    Imported(jai_modules::Binding),
    Library(ForeignLibraryId),
    Procedure {
        procedure: ProcedureId,
        ty: TypeId,
    },
    Code(jai_types::CodeValueId),
    TypedConstant(typed_constants::ConstantId),
    Type(TypeId),
    Storage(Storage),
    Constant(ScalarConstant),
    Enum(modules::aggregates::EnumConstant),
}
#[derive(Clone)]
struct ParameterSignature {
    name: Symbol,
    ty: TypeId,
    default: Option<ParameterDefault>,
    evaluation: syntax::ParameterEvaluation,
}
/// Source-owned defaults that are evaluated when an argument is omitted.
#[derive(Clone)]
enum ParameterDefault {
    Source(std::rc::Rc<source_parameter_defaults::SourceParameterDefault>),
    Discarded,
    Constant(jai_ir::ConstantValue),
    RuntimeRead(runtime_defaults::RuntimeDefaultRead),
    CallerLocation,
    CodeNull { ty: TypeId },
}
#[derive(Clone)]
struct Signature {
    ty: TypeId,
    id: ProcedureId,
    parameters: Vec<ParameterSignature>,
    source_variadic: overloads::CandidateVariadic,
    results: Vec<ResultSignature>,
}
#[derive(Clone)]
struct ResultSignature {
    usage: syntax::ResultUsage,
    name: Option<Symbol>,
    ty: TypeId,
    default: Option<jai_ir::ConstantValue>,
}
enum Expr {
    Code(jai_types::CodeValueId),
    Type(TypeId),
    Null,
    Pointer {
        ty: TypeId,
        value: ValueExpr,
    },
    Literal(i128),
    WeakFloat(floats::WeakFloat),
    Float(FloatExpr),
    WeakConditional(Box<Conditional<Expr>>),
    Int(IntExpr),
    Bool(BoolExpr),
    Void(Call),
    IndirectVoid {
        inline_hint: jai_types::InlineHint,
        callee: Box<ValueExpr>,
        arguments: Vec<(ParameterId, ValueExpr)>,
    },
    Typed {
        ty: TypeId,
        value: ValueExpr,
    },
    Enum {
        ty: TypeId,
        flags: bool,
        representation: IntegerType,
        value: ValueExpr,
    },
}
impl Expr {
    fn int(self, span: Span) -> Result<IntExpr, Diagnostic> {
        match self {
            Self::Int(e) => Ok(e),
            Self::Literal(n) => Self::Literal(n).int_as(IntegerType::S64, span),
            Self::WeakConditional(e) => Self::WeakConditional(e).int_as(IntegerType::S64, span),
            _ => Err(Diagnostic::new(span, "expected int value")),
        }
    }
    fn int_as(self, ty: IntegerType, span: Span) -> Result<IntExpr, Diagnostic> {
        match self {
            Self::WeakConditional(e) => Ok(IntExpr::new(
                ty,
                IntExprKind::Conditional(Box::new(Conditional {
                    condition: e.condition,
                    then_value: e.then_value.int_as(ty, span)?,
                    else_value: e.else_value.int_as(ty, span)?,
                })),
            )),
            Self::Literal(n) => IntegerValue::checked(ty, n)
                .map(IntExpr::constant)
                .ok_or_else(|| {
                    Diagnostic::new(span, "integer constant is out of range for its target type")
                }),
            Self::Int(e) if e.ty() == ty => Ok(e),
            Self::Int(e) if ty.contains(e.ty()) => Ok(IntExpr::new(
                ty,
                IntExprKind::Cast(CastMode::Unchecked, Box::new(e)),
            )),
            _ => Err(Diagnostic::new(
                span,
                "implicit integer conversion does not preserve the source type's entire range",
            )),
        }
    }
    fn bool(self, span: Span) -> Result<BoolExpr, Diagnostic> {
        match self {
            Self::Bool(e) => Ok(e),
            _ => Err(Diagnostic::new(span, "expected bool value")),
        }
    }
    fn value(self, span: Span) -> Result<ValueExpr, Diagnostic> {
        match self {
            Self::Type(_) | Self::Code(_) => Err(Diagnostic::new(
                span,
                "type values require compile-time binding and cannot occupy runtime storage",
            )),
            Self::Null => Err(Diagnostic::new(
                span,
                "null requires a pointer type context",
            )),
            Self::Pointer { value, .. } => Ok(value),
            Self::Float(e) => Ok(ValueExpr::Float(e)),
            value @ Self::WeakFloat(_) => value.float(span).map(ValueExpr::Float),
            value @ Self::WeakConditional(_) if value.has_float() => {
                value.float(span).map(ValueExpr::Float)
            }
            Self::Int(e) => Ok(ValueExpr::Int(e)),
            Self::Literal(n) => Self::Literal(n).int(span).map(ValueExpr::Int),
            Self::WeakConditional(e) => Self::WeakConditional(e).int(span).map(ValueExpr::Int),
            Self::Bool(e) => Ok(ValueExpr::Bool(e)),
            Self::Typed { value, .. } | Self::Enum { value, .. } => Ok(value),
            Self::Void(_) | Self::IndirectVoid { .. } => {
                Err(Diagnostic::new(span, "void call cannot supply a value"))
            }
        }
    }
    fn condition(
        self,
        span: Span,
        types: &dyn jai_types::TypeView,
    ) -> Result<BoolExpr, Diagnostic> {
        match self {
            Self::Type(_) | Self::Code(_) => Err(Diagnostic::new(
                span,
                "type values cannot supply a runtime condition",
            )),
            Self::Null => Ok(BoolExpr::Constant(false)),
            Self::Pointer { value, .. } => Ok(BoolExpr::FromPointer(Box::new(value))),
            Self::Bool(e) => Ok(e),
            value @ (Self::Float(_) | Self::WeakFloat(_)) => {
                let value = value.float(span)?;
                let zero = FloatExpr::constant(match value.ty() {
                    jai_types::FloatType::F32 => jai_types::FloatValue::F32(0),
                    jai_types::FloatType::F64 => jai_types::FloatValue::F64(0),
                });
                Ok(BoolExpr::CompareFloats(
                    Relation::NotEqual,
                    Box::new(value),
                    Box::new(zero),
                ))
            }
            Self::Int(e) => Ok(BoolExpr::FromInt(Box::new(e))),
            Self::Literal(n) => Ok(BoolExpr::Constant(n != 0)),
            Self::WeakConditional(e) => Ok(BoolExpr::Conditional(Box::new(Conditional {
                condition: e.condition,
                then_value: e.then_value.condition(span, types)?,
                else_value: e.else_value.condition(span, types)?,
            }))),
            Self::Enum {
                flags: true,
                representation,
                value,
                ..
            } => Ok(BoolExpr::FromInt(Box::new(IntExpr::new(
                representation,
                IntExprKind::EnumValue(Box::new(value)),
            )))),
            Self::Typed { ty, value }
                if matches!(types.kind(ty), Ok(jai_types::TypeKind::Procedure(_))) =>
            {
                Ok(BoolExpr::FromPointer(Box::new(value)))
            }
            Self::Typed { .. } | Self::Enum { .. } => Err(Diagnostic::new(
                span,
                "nominal value cannot implicitly supply a condition",
            )),
            Self::Void(_) | Self::IndirectVoid { .. } => {
                Err(Diagnostic::new(span, "void call cannot supply a condition"))
            }
        }
    }
    fn weak_integer(&self) -> bool {
        matches!(self, Self::Literal(_) | Self::WeakConditional(_)) && !self.has_float()
    }
    fn cast_integer(
        self,
        ty: IntegerType,
        mode: CastMode,
        span: Span,
    ) -> Result<IntExpr, Diagnostic> {
        if matches!(mode, CastMode::Force(_)) {
            return Err(Diagnostic::new(
                span,
                "force requires a checked storage cast, not numeric conversion",
            ));
        }
        Ok(match self {
            Self::Type(_) | Self::Code(_) => {
                return Err(Diagnostic::new(
                    span,
                    "type values cannot be cast to an integer",
                ));
            }
            Self::Pointer { value, .. } => IntExpr::new(
                ty,
                IntExprKind::FromPointer {
                    value: Box::new(value),
                    mode,
                },
            ),
            Self::Null => IntExpr::constant(IntegerValue::wrapping(ty, 0)),
            Self::Literal(n) => match mode {
                CastMode::Force(_) => unreachable!("force was rejected before numeric conversion"),
                CastMode::Unchecked | CastMode::Truncate => {
                    IntExpr::constant(IntegerValue::wrapping(ty, n))
                }
                CastMode::Checked => IntegerValue::checked(ty, n)
                    .map(IntExpr::constant)
                    .unwrap_or_else(|| IntExpr::new(ty, IntExprKind::InvalidCheckedCast)),
            },
            Self::WeakConditional(e) => IntExpr::new(
                ty,
                IntExprKind::Conditional(Box::new(Conditional {
                    condition: e.condition,
                    then_value: e.then_value.cast_integer(ty, mode, span)?,
                    else_value: e.else_value.cast_integer(ty, mode, span)?,
                })),
            ),
            value @ (Self::Float(_) | Self::WeakFloat(_)) => {
                if mode != CastMode::Checked {
                    return Err(Diagnostic::new(
                        span,
                        if mode == CastMode::Truncate {
                            "trunc float-to-integer conversion has no established source policy"
                        } else {
                            "unchecked float-to-integer conversion has no established source policy"
                        },
                    ));
                }
                IntExpr::new(
                    ty,
                    IntExprKind::FromFloat(mode, Box::new(value.float(span)?)),
                )
            }
            Self::Int(e) => IntExpr::new(ty, IntExprKind::Cast(mode, Box::new(e))),
            Self::Bool(e) => IntExpr::new(ty, IntExprKind::FromBool(Box::new(e))),
            Self::Enum {
                representation,
                value,
                ..
            } => IntExpr::new(
                ty,
                IntExprKind::Cast(
                    mode,
                    Box::new(IntExpr::new(
                        representation,
                        IntExprKind::EnumValue(Box::new(value)),
                    )),
                ),
            ),
            Self::Typed { .. } => {
                return Err(Diagnostic::new(
                    span,
                    "aggregate cannot be cast to an integer",
                ));
            }
            Self::Void(_) | Self::IndirectVoid { .. } => {
                return Err(Diagnostic::new(
                    span,
                    "void call cannot be cast to an integer",
                ));
            }
        })
    }
    fn integer_type(&self, span: Span) -> Result<Option<IntegerType>, Diagnostic> {
        match self {
            Self::Int(e) => Ok(Some(e.ty())),
            Self::Literal(_) | Self::WeakConditional(_) => Ok(None),
            _ => Err(Diagnostic::new(span, "expected integer operands")),
        }
    }
}

pub fn resolve(module: &syntax::Module) -> Result<Program, Diagnostic> {
    declarations::check_top_level_names(module)?;
    let mut types = TypeRegistry::new();
    let mut meta = reflection::MetaContext::default();
    let (globals, global_bindings) =
        declarations::resolve_globals(module, &types, &mut meta.storage_alignments)?;
    let mut signatures = HashMap::new();
    for procedure in module.procedures() {
        let result = procedure.scalar_return_type().ok_or_else(|| {
            Diagnostic::new(
                procedure.span,
                "single-file scalar resolver requires a scalar signature",
            )
        })?;
        let parameters = calls::parameters(procedure, &global_bindings, &types)?;
        let results = match result {
            ReturnType::Void => vec![],
            ReturnType::Value(ty) => vec![ResultSignature {
                usage: procedure.results[0].usage,
                name: None,
                ty: types.scalar(ty),
                default: None,
            }],
        };
        let ty = types
            .procedure(ProcedureType {
                parameters: parameters
                    .iter()
                    .filter(|parameter| {
                        parameter.evaluation == syntax::ParameterEvaluation::Evaluate
                    })
                    .map(|parameter| parameter.ty)
                    .collect(),
                results: results.iter().map(|result| result.ty).collect(),
                convention: CallingConvention::Jai,
                context: ContextMode::None,
                variadic: jai_types::Variadic::None,
            })
            .map_err(|error| Diagnostic::new(procedure.span, error.to_string()))?;
        signatures.insert(
            procedure.name,
            Signature {
                id: ProcedureId::new(signatures.len()),
                ty,
                parameters,
                source_variadic: overloads::CandidateVariadic::None,
                results,
            },
        );
    }
    let main = module
        .symbols()
        .find("main")
        .and_then(|name| signatures.get(&name))
        .ok_or_else(|| Diagnostic::new(Span::default(), "no main procedure"))?;
    if !main.parameters.is_empty() {
        return Err(Diagnostic::new(
            Span::default(),
            "main cannot take parameters",
        ));
    }
    let entry = match main.results.as_slice() {
        [] => EntryPoint::Void(main.id),
        [result] if result.ty == types.scalar(ScalarType::Int(IntegerType::S64)) => {
            EntryPoint::Int(main.id)
        }
        _ => {
            return Err(Diagnostic::new(
                Span::default(),
                "main must return int or void",
            ));
        }
    };
    let mut places = PlaceRegistry::new();
    let mut procedures = Vec::new();
    for procedure in module.procedures() {
        meta.remember_inline_hint(signatures[&procedure.name].id, procedure.inline_hint);
        meta.remember_execution(signatures[&procedure.name].id, procedure.execution);
    }
    for procedure in module.procedures() {
        let signature = &signatures[&procedure.name];
        meta.remember_inline_hint(signature.id, procedure.inline_hint);
        let mut resolver = Resolver {
            expression_owner: Some(signature.id),
            debug: debug_capture::Capture::default(),
            checks: safety_checks::ActiveChecks::default().overridden(procedure.checks),
            context: None,
            context_available: true,
            meta: &mut meta,
            procedure: signature.id,
            types: &mut types,
            places: &mut places,
            signatures: &signatures,
            symbols: module.symbols(),
            globals: &global_bindings,
            graph_scope: None,
            compile_time: None,
            target_layout: None,
            scopes: vec![HashMap::new()],
            local_scopes: local_declarations::LocalScopes::default(),
            locals: Vec::new(),
            span: procedure.span,
            results: &signature.results,
            loops: Vec::new(),
            next_loop: 0,
            cleanups: Vec::new(),
            active_push: None,
            next_push: 0,
            deferred_scopes: Vec::new(),
            cleanup_context: None,
        };
        resolver.debug.enter_policy(procedure.debug);
        let mut parameters = Vec::new();
        for parameter in &signature.parameters {
            if parameter.evaluation == syntax::ParameterEvaluation::Discard {
                resolver.bind_discarded_parameter(parameter.name, parameter.ty)?;
                continue;
            }
            parameters.push(resolver.declare_typed(parameter.name, parameter.ty)?);
        }
        let body = resolver.block(&procedure.body, false)?;
        if !signature.results.is_empty() && body.flow != Flow::Terminates {
            return Err(Diagnostic::new(
                procedure.span,
                "value-returning procedure may reach its end",
            ));
        }
        resolver.remember_procedure_notes(signature.id, &procedure.notes)?;
        procedures.push(Procedure {
            id: signature.id,
            signature: signature.ty,
            parameters,
            locals: resolver.locals,
            cleanups: resolver.cleanups,
            body,
        });
    }
    procedures.extend(meta.local_declarations.ready_snapshot().into_values());
    procedures.sort_by_key(|procedure| procedure.id.index());
    let types = types
        .freeze()
        .map_err(|error| Diagnostic::new(Span::default(), error.to_string()))?;
    ProgramBuilder::new(types)
        .storage_alignments(meta.storage_alignments)
        .procedure_hints(meta.procedure_hints)
        .procedure_phases(meta.procedure_phases)
        .procedures(procedures)
        .prototypes(meta.local_declarations.prototypes())
        .foreign_libraries(meta.local_declarations.foreign_libraries())
        .globals(globals)
        .places(places.freeze())
        .finish_library()
        .and_then(|library| library.into_program(entry))
        .map_err(|error| Diagnostic::new(Span::default(), error.to_string()))
}
struct LoopBinding {
    id: LoopId,
    name: Option<Symbol>,
    cleanup_depth: usize,
    iteration: Option<iteration_removal::Iteration>,
}
#[derive(Clone, Copy)]
struct CleanupControlContext {
    loop_depth: usize,
}
struct Resolver<'a> {
    expression_owner: Option<ProcedureId>,
    debug: debug_capture::Capture,
    checks: safety_checks::ActiveChecks,
    context: Option<&'a context::Schema>,
    context_available: bool,
    meta: &'a mut reflection::MetaContext,
    graph_scope: Option<modules::FileScope<'a>>,
    compile_time: Option<&'a compile_time::Context<'a>>,
    target_layout: Option<jai_types::LayoutPolicy>,
    procedure: ProcedureId,
    types: &'a mut TypeRegistry,
    places: &'a mut PlaceRegistry,
    signatures: &'a HashMap<Symbol, Signature>,
    symbols: &'a Symbols,
    scopes: Vec<HashMap<Symbol, Binding>>,
    local_scopes: local_declarations::LocalScopes,
    globals: &'a HashMap<Symbol, Binding>,
    locals: Vec<Local>,
    span: Span,
    results: &'a [ResultSignature],
    loops: Vec<LoopBinding>,
    next_loop: usize,
    cleanups: Vec<jai_ir::Cleanup>,
    active_push: Option<PushContextId>,
    next_push: usize,
    deferred_scopes: Vec<Vec<CleanupId>>,
    cleanup_context: Option<CleanupControlContext>,
}
impl Resolver<'_> {
    fn error(&self, text: impl Into<String>) -> Diagnostic {
        Diagnostic::new(self.span, text)
    }
    fn lookup(&self, name: Symbol) -> Result<Binding, Diagnostic> {
        self.lookup_path(
            &syntax::NamePath {
                root: name,
                members: Vec::new(),
            },
            self.span,
        )
    }
    fn lookup_path(&self, path: &syntax::NamePath, span: Span) -> Result<Binding, Diagnostic> {
        if let Some(binding) = self.lexical_graph_binding_ready(path, span)? {
            return self.imported_binding_value_ready(binding, span);
        }
        if let Some(value) = self.lexical_using_binding_ready(path.root, span)? {
            if matches!(value, Binding::Discarded(_)) {
                return Err(Diagnostic::new(span, "#discard parameter cannot be read"));
            }
            if let Binding::Storage(storage) = value {
                self.check_local_storage_capture(storage, span)?;
            }
            if path.members.is_empty() {
                return Ok(value);
            }
            return Err(Diagnostic::new(span, "scalar value is not a namespace"));
        }
        if let Some(value) = self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(&path.root))
            .cloned()
        {
            if matches!(value, Binding::Discarded(_)) {
                return Err(Diagnostic::new(span, "#discard parameter cannot be read"));
            }
            if let Binding::Storage(storage) = value {
                self.check_local_storage_capture(storage, span)?;
            }
            if path.members.is_empty() {
                return Ok(value);
            }
            return Err(Diagnostic::new(span, "scalar value is not a namespace"));
        }
        if let Some(scope) = self.graph_scope {
            if let Some(context) = self.compile_time
                && let Some(id) = scope.pending_constant(path, context.deferred)
            {
                context.pending_constants.borrow_mut().push(id);
                return Err(Diagnostic::new(
                    span,
                    "constant declaration is pending typed compile-time evaluation",
                ));
            }
            return scope.value(path, span);
        }
        if !path.members.is_empty() {
            return Err(Diagnostic::new(
                span,
                "qualified name requires a module scope",
            ));
        }
        self.globals.get(&path.root).cloned().ok_or_else(|| {
            Diagnostic::new(
                span,
                format!("unknown variable '{}'", self.symbols.name(path.root)),
            )
        })
    }
    fn storage(&mut self, name: Symbol) -> Result<Storage, Diagnostic> {
        let path = syntax::NamePath {
            root: name,
            members: Vec::new(),
        };
        let binding = if let Some(jai_modules::Binding::StorageMember(id)) =
            self.lexical_graph_binding_ready(&path, self.span)?
        {
            self.imported_storage_member_value(id, self.span)?
        } else {
            match self.lookup(name)? {
                Binding::Imported(jai_modules::Binding::StorageMember(id)) => {
                    self.imported_storage_member_value(id, self.span)?
                }
                binding => binding,
            }
        };
        match binding {
            Binding::CompilerInput { .. } => {
                Err(self.error("compiler input is read-only and has no native place"))
            }
            Binding::Discarded(_) => Err(self.error("#discard parameter cannot be assigned")),
            Binding::Storage(storage) => Ok(storage),
            Binding::Constant(_)
            | Binding::Namespace(_)
            | Binding::Imported(_)
            | Binding::Enum(_)
            | Binding::Type(_)
            | Binding::Procedure { .. }
            | Binding::Library(_)
            | Binding::Macro(_)
            | Binding::Code(_)
            | Binding::LambdaPreview(_)
            | Binding::TypedConstant(_) => Err(self.error("cannot assign to a constant")),
        }
    }
    fn bind_name(&mut self, name: Symbol, binding: Binding) -> Result<(), Diagnostic> {
        if self.local_import_alias_reserved(name) {
            return Err(self.error(format!(
                "duplicate local import alias '{}'",
                self.symbols.name(name)
            )));
        }
        if self.local_using_name_reserved(name) {
            return Err(self.error(format!(
                "duplicate local using member '{}'",
                self.symbols.name(name)
            )));
        }
        if self.local_name_reserved(name) {
            return Err(self.error(format!(
                "duplicate local declaration '{}'",
                self.symbols.name(name)
            )));
        }
        let scope = self.scopes.last_mut().expect("resolver always has a scope");
        if scope.contains_key(&name) {
            return Err(self.error(format!("duplicate variable '{}'", self.symbols.name(name))));
        }
        scope.insert(name, binding);
        Ok(())
    }
    fn bind(&mut self, name: Symbol, local: Local) -> Result<(), Diagnostic> {
        self.bind_name(name, Binding::Storage(Storage::local(local, self.types)))?;
        self.locals.push(local);
        self.debug_local(local, name, self.span)?;
        Ok(())
    }
    fn allocate_typed(&mut self, ty: TypeId) -> Result<Local, Diagnostic> {
        self.ensure_runtime_type_storage(ty, self.span)?;
        let local = Local::new_typed(self.procedure, self.locals.len(), ty, self.types)
            .map_err(|error| self.error(error.to_string()))?;
        self.locals.push(local);
        Ok(local)
    }
    fn allocate(&mut self, ty: ScalarType) -> Local {
        let local = Local::new(self.procedure, self.locals.len(), ty, self.types);
        self.locals.push(local);
        local
    }
    fn declare_int(&mut self, name: Symbol, ty: IntegerType) -> Result<IntLocal, Diagnostic> {
        let local = self.declare(name, ScalarType::Int(ty))?;
        Ok(local.integer(self.types).expect("integer declaration"))
    }
    fn declare_bool(&mut self, name: Symbol) -> Result<BoolLocal, Diagnostic> {
        let local = self.declare(name, ScalarType::Bool)?;
        Ok(local.boolean(self.types).expect("boolean declaration"))
    }
    fn declare(&mut self, name: Symbol, ty: ScalarType) -> Result<Local, Diagnostic> {
        self.declare_typed(name, self.types.scalar(ty))
    }
    fn declare_typed(&mut self, name: Symbol, ty: TypeId) -> Result<Local, Diagnostic> {
        self.ensure_runtime_type_storage(ty, self.span)?;
        let local = Local::new_typed(self.procedure, self.locals.len(), ty, self.types)
            .map_err(|error| self.error(error.to_string()))?;
        self.bind(name, local)?;
        Ok(local)
    }
    fn store(&mut self, local: Storage, value: Expr) -> Result<Statement, Diagnostic> {
        self.reject_iteration_write(local.place(), self.span)?;
        let value = if local.place().ty() == self.types.meta_type() {
            self.runtime_type_expression(value, self.span)?
        } else {
            value
        };
        Ok(match local {
            Storage::Int(id) => Statement::StoreInt(id, value.int_as(id.ty(), self.span)?),
            Storage::Bool(id) => Statement::StoreBool(id, value.bool(self.span)?),
            Storage::Value(place) => {
                Statement::Store(place, self.coerce_value(value, place.ty(), self.span)?)
            }
        })
    }
    fn block(
        &mut self,
        statements: &[syntax::Statement],
        scoped: bool,
    ) -> Result<Block, Diagnostic> {
        let debug_location = self.debug_location(self.span)?;
        self.debug.begin_block(debug_location);
        if scoped {
            self.scopes.push(HashMap::new());
        }
        self.register_local_declarations(statements)?;
        let statements = self.select_compile_time_statements(statements)?;
        self.refresh_local_import_environments(&statements)?;
        self.resolve_registered_local_declarations()?;
        self.deferred_scopes.push(Vec::new());
        let mut out = Vec::new();
        let mut flow = Flow::FallsThrough;
        for statement in &statements {
            if let syntax::StatementKind::Import(import) = &statement.kind {
                self.bind_scoped_import(import)?;
                continue;
            }
            if let syntax::StatementKind::ConstantResults(group) = &statement.kind {
                if group
                    .names
                    .iter()
                    .all(|&(name, _)| self.symbols.name(name) == "_")
                {
                    self.constant_result_values(group)?;
                }
                for &(name, span) in &group.names {
                    if self.symbols.name(name) != "_" {
                        self.resolve_local_name(name, span)?;
                    }
                }
                continue;
            }
            if let syntax::StatementKind::Constant(constant) = &statement.kind {
                if matches!(
                    constant.initializer.kind,
                    syntax::ExpressionKind::ShortLambda(_)
                ) {
                    self.remember_local_constant_source(constant.name);
                    continue;
                }
                self.resolve_local_name(constant.name, constant.span)?;
                continue;
            }
            if let syntax::StatementKind::Procedure(procedure) = &statement.kind
                && procedure.expands
            {
                self.resolve_local_name(procedure.name, procedure.span)?;
                continue;
            }
            if matches!(
                &statement.kind,
                syntax::StatementKind::Library(_)
                    | syntax::StatementKind::Record(_)
                    | syntax::StatementKind::Enum(_)
                    | syntax::StatementKind::TypeAlias(_)
                    | syntax::StatementKind::Procedure(_)
                    | syntax::StatementKind::ProcedurePrototype(_)
            ) {
                continue;
            }
            if flow == Flow::Terminates {
                return Err(Diagnostic::new(statement.span, "unreachable statement"));
            }
            if let syntax::StatementKind::Defer(body) = &statement.kind {
                let outer_span = std::mem::replace(&mut self.span, statement.span);
                let result = self.register_defer(body);
                self.span = outer_span;
                result?;
                continue;
            }
            let s = self.statement(statement)?;
            flow = match &s {
                Statement::Exit(_) => Flow::Terminates,
                Statement::If(_, yes, no)
                    if yes.flow == Flow::Terminates && no.flow == Flow::Terminates =>
                {
                    Flow::Terminates
                }
                Statement::Cases(c) => c.flow,
                Statement::Block(b) | Statement::PushContext { body: b, .. } => b.flow,
                _ => Flow::FallsThrough,
            };
            self.debug.emit_statement(out.len());
            out.push(s);
        }
        let pending = self
            .deferred_scopes
            .pop()
            .expect("each block has a cleanup scope");
        if flow == Flow::FallsThrough {
            for cleanup in pending.into_iter().rev() {
                self.debug.emit_cleanup(cleanup, out.len());
                out.push(Statement::Cleanup(cleanup));
            }
        }
        if scoped {
            self.scopes.pop();
            self.pop_local_scope();
        }
        self.debug.finish_block();
        Ok(Block {
            statements: out,
            flow,
        })
    }
    fn statement(&mut self, statement: &syntax::Statement) -> Result<Statement, Diagnostic> {
        let debug_location = self.debug_location(statement.span)?;
        let outer_span = std::mem::replace(&mut self.span, statement.span);
        self.debug.begin_statement(debug_location);
        let result = self.statement_kind(statement).map_err(|diagnostic| {
            match self
                .debug
                .source()
                .or_else(|| self.graph_scope.map(|scope| scope.source()))
            {
                Some(source) => diagnostic.with_fallback_source(source),
                None => diagnostic,
            }
        });
        self.debug.finish_statement(result.as_ref().ok());
        self.span = outer_span;
        result
    }
    fn statement_kind(&mut self, statement: &syntax::Statement) -> Result<Statement, Diagnostic> {
        Ok(match &statement.kind {
            syntax::StatementKind::Simd(block) => Statement::Simd(self.simd_block(block)?),
            syntax::StatementKind::InstructionBytes(instruction) => {
                Statement::Simd(self.machine_bytes(instruction)?)
            }
            syntax::StatementKind::Import(import) => {
                return Err(Diagnostic::new(
                    import.span,
                    "scoped imports require a source statement sequence",
                ));
            }
            syntax::StatementKind::Using(directive) => self.using_directive(directive)?,
            syntax::StatementKind::UsingDeclaration { .. } => self.using_declaration(statement)?,
            syntax::StatementKind::CallerExport(inner) => self.caller_export(inner)?,
            syntax::StatementKind::ContextField(_) => {
                return Err(Diagnostic::new(
                    statement.span,
                    "#add_context is only valid in compiler-managed Preload context registration",
                ));
            }
            syntax::StatementKind::Procedure(_)
            | syntax::StatementKind::ProcedurePrototype(_)
            | syntax::StatementKind::Library(_)
            | syntax::StatementKind::Record(_)
            | syntax::StatementKind::Enum(_)
            | syntax::StatementKind::TypeAlias(_) => {
                unreachable!("local declarations are registered before block statements")
            }
            syntax::StatementKind::PushContext { value, body } => {
                self.push_context(value.as_ref(), body, self.span)?
            }
            syntax::StatementKind::Declare(declaration) => {
                if matches!(declaration, syntax::Declaration::External { .. }) {
                    return self.bind_local_external_statement(declaration);
                }
                let alignment = self.declaration_alignment(declaration)?;
                if alignment.is_some() && self.symbols.name(declaration.name()) == "_" {
                    return Err(Diagnostic::new(
                        self.span,
                        "storage alignment requires a stored declaration",
                    ));
                }
                if let Some(statement) = self.discard_result_declaration(declaration)? {
                    return Ok(statement);
                }
                let (name, ty, expression) = match declaration {
                    syntax::Declaration::External { .. } => {
                        unreachable!("external storage handled before local allocation")
                    }
                    syntax::Declaration::Inferred {
                        name, initializer, ..
                    } => {
                        let expression = self.expr(initializer)?;
                        let ty = self.expression_type(&expression, initializer.span)?;
                        (*name, ty, Some(expression))
                    }
                    syntax::Declaration::Explicit {
                        name,
                        ty,
                        initializer,
                        ..
                    } => {
                        let ty = self.types.scalar(*ty);
                        let expression = match initializer {
                            Some(expression)
                                if matches!(
                                    expression.kind,
                                    syntax::ExpressionKind::Uninitialized
                                ) =>
                            {
                                None
                            }
                            Some(expression) => Some(self.expr_expected(expression, ty)?),
                            None => Some(match self.types.kind(ty).unwrap() {
                                jai_types::TypeKind::Integer(integer) => Expr::Int(
                                    IntExpr::constant(IntegerValue::wrapping(*integer, 0)),
                                ),
                                jai_types::TypeKind::Bool => Expr::Bool(BoolExpr::Constant(false)),
                                _ => unreachable!(),
                            }),
                        };
                        (*name, ty, expression)
                    }
                    syntax::Declaration::UnresolvedExplicit {
                        name,
                        ty,
                        initializer,
                        ..
                    } => {
                        let ty = self.lexical_annotation(ty, self.span)?;
                        let expression = match initializer {
                            Some(expression)
                                if matches!(
                                    expression.kind,
                                    syntax::ExpressionKind::Uninitialized
                                ) =>
                            {
                                None
                            }
                            Some(expression) => Some(self.expr_expected(expression, ty)?),
                            None if self.types.record_definition(ty).is_ok_and(|record| {
                                record.kind == jai_types::RecordKind::Union
                            }) =>
                            {
                                None
                            }
                            None => Some(self.typed_value(
                                self.default_value(ty, self.span)?.into_expression(),
                                ty,
                                self.span,
                            )?),
                        };
                        (*name, ty, expression)
                    }
                };
                let local = self.declare_typed(name, ty)?;
                if let Some(alignment) = alignment {
                    self.meta
                        .storage_alignments
                        .set_local(local.id(), alignment)
                        .map_err(|error| Diagnostic::new(self.span, error.to_string()))?;
                }
                self.bind_callback_declaration(
                    local.place(),
                    declaration,
                    expression.as_ref(),
                    self.span,
                )?;
                match expression {
                    Some(expression) => {
                        self.store(Storage::local(local, self.types), expression)?
                    }
                    None => Statement::Block(Block {
                        statements: Vec::new(),
                        flow: Flow::FallsThrough,
                    }),
                }
            }
            syntax::StatementKind::Assign(name, expression) => {
                if self.symbols.name(*name) == "_" {
                    let target = syntax::PlaceSyntax {
                        kind: syntax::PlaceKind::Name(*name),
                        span: self.span,
                    };
                    return self.assign_results(&[target], std::slice::from_ref(expression), None);
                }
                let storage = self.storage(*name)?;
                let value = self.expr_expected(expression, storage.place().ty())?;
                self.store(storage, value)?
            }
            syntax::StatementKind::Update(name, operator, expression) => {
                let target = syntax::PlaceSyntax {
                    kind: syntax::PlaceKind::Name(*name),
                    span: self.span,
                };
                self.update_place(&target, *operator, expression)?
            }
            syntax::StatementKind::AssignPlace { target, value } => {
                if let Some(result) = self.overloaded_index_assignment(target, value) {
                    return result;
                }
                if self.is_result_discard_target(target) {
                    return self.assign_results(
                        std::slice::from_ref(target),
                        std::slice::from_ref(value),
                        None,
                    );
                }
                let place = self.resolve_place(target)?;
                self.reject_iteration_write(place, target.span)?;
                let expression = self.expr_expected(value, place.ty())?;
                Statement::Store(
                    place,
                    self.coerce_value(expression, place.ty(), value.span)?,
                )
            }
            syntax::StatementKind::UpdatePlace {
                target,
                operation,
                value,
            } => self.update_place(target, *operation, value)?,
            syntax::StatementKind::Return(expression) => {
                self.reject_expanded_return(statement.span)?;
                self.resolve_return(expression.as_ref())?
            }
            syntax::StatementKind::ReturnValues(values) => {
                self.reject_expanded_return(statement.span)?;
                self.resolve_return_values(values)?
            }
            syntax::StatementKind::DeclareResults { names, ty, values } => {
                self.declare_results(names, ty.as_ref(), values)?
            }
            syntax::StatementKind::MixedResults {
                bindings,
                ty,
                values,
            } => self.mixed_results(bindings, ty.as_ref(), values)?,
            syntax::StatementKind::AssignResults {
                targets,
                values,
                operation,
            } => self.assign_results(targets, values, *operation)?,
            syntax::StatementKind::Defer(_)
            | syntax::StatementKind::Constant(_)
            | syntax::StatementKind::ConstantResults(_) => {
                unreachable!("declarations are handled by block resolution")
            }
            syntax::StatementKind::Expression(e)
                if matches!(e.kind, syntax::ExpressionKind::CompileTime(_)) =>
            {
                let syntax::ExpressionKind::CompileTime(body) = &e.kind else {
                    unreachable!()
                };
                self.resolve_compile_time_statement(body, e.span)?
            }
            syntax::StatementKind::Expression(e) => {
                if let Some(block) = self.expand_statement(e)? {
                    return Ok(Statement::Block(block));
                }
                if let Some(statement) = self.discard_call_results(e)? {
                    statement
                } else {
                    let value = self.expr(e)?;
                    self.check_discarded_bound_operator(e, &value)?;
                    match value {
                        Expr::Type(_) | Expr::Code(_) => {
                            return Err(Diagnostic::new(
                                e.span,
                                "type values must be used in a type query or compile-time binding",
                            ));
                        }
                        Expr::Null => {
                            return Err(Diagnostic::new(
                                e.span,
                                "null requires a pointer type context",
                            ));
                        }
                        Expr::Pointer { value, .. } => Statement::DiscardValue(value),
                        value @ (Expr::Float(_) | Expr::WeakFloat(_)) => {
                            Statement::DiscardValue(value.value(e.span)?)
                        }
                        value @ Expr::WeakConditional(_) if value.has_float() => {
                            Statement::DiscardValue(value.value(e.span)?)
                        }
                        Expr::Literal(n) => Statement::DiscardInt(Expr::Literal(n).int(e.span)?),
                        Expr::WeakConditional(value) => {
                            Statement::DiscardInt(Expr::WeakConditional(value).int(e.span)?)
                        }
                        Expr::Int(e) => Statement::DiscardInt(e),
                        Expr::Bool(e) => Statement::DiscardBool(e),
                        Expr::Void(c) => Statement::CallVoid(c),
                        Expr::IndirectVoid {
                            callee,
                            arguments,
                            inline_hint,
                        } => Statement::IndirectCallResults {
                            inline_hint,
                            callee,
                            arguments,
                            destinations: vec![],
                        },
                        Expr::Typed { value, .. } | Expr::Enum { value, .. } => {
                            Statement::DiscardValue(value)
                        }
                    }
                }
            }
            syntax::StatementKind::CompileTimeCases(_)
            | syntax::StatementKind::CompileTimeAssert { .. }
            | syntax::StatementKind::CompileTimeIf { .. } => {
                unreachable!("static conditionals are selected before block lowering")
            }
            syntax::StatementKind::If(cond, yes, no) => {
                let condition = self.condition_expression(cond)?;
                let yes = self.block(yes, true)?;
                self.debug
                    .attach_block(&[DebugPathStep::Child(DebugBranch::IfThen)]);
                let no = self.block(no, true)?;
                self.debug
                    .attach_block(&[DebugPathStep::Child(DebugBranch::IfElse)]);
                Statement::If(condition, yes, no)
            }
            syntax::StatementKind::Cases(c) => self.resolve_cases(c)?,
            syntax::StatementKind::While(condition, body) => self.resolve_while(condition, body)?,
            syntax::StatementKind::Range(range) => self.resolve_range(range)?,
            syntax::StatementKind::ArrayLoop(loop_) => self.resolve_array_loop(loop_)?,
            syntax::StatementKind::Jump { kind, target, span } => {
                self.resolve_jump(*kind, *target, *span)?
            }
            syntax::StatementKind::Block(body) => {
                let body = self.block(body, true)?;
                self.debug
                    .attach_block(&[DebugPathStep::Child(DebugBranch::Block)]);
                Statement::Block(body)
            }
            syntax::StatementKind::CheckScope { checks, body } => {
                let body = self.checked_block(*checks, body, true)?;
                self.debug
                    .attach_block(&[DebugPathStep::Child(DebugBranch::Block)]);
                Statement::Block(body)
            }
            syntax::StatementKind::Insert(directive) => self.insert_statement(directive)?,
        })
    }
    fn expr(&mut self, expr: &syntax::Expression) -> Result<Expr, Diagnostic> {
        let span = expr.span;
        Ok(match &expr.kind {
            syntax::ExpressionKind::AnonymousProcedure(source) => {
                self.anonymous_procedure_value(source, span, None)?
            }
            syntax::ExpressionKind::ShortLambda(source) => {
                self.short_lambda_value(source, span, None, None)?
            }
            syntax::ExpressionKind::CallerLocation => {
                return Err(Diagnostic::new(
                    span,
                    "#caller_location requires a procedure parameter default",
                ));
            }
            syntax::ExpressionKind::SourceLocation => self.source_location_expression(span)?,
            syntax::ExpressionKind::SourceFile => self.source_file_expression(span)?,
            syntax::ExpressionKind::SourceFilepath => self.source_filepath_expression(span)?,
            syntax::ExpressionKind::SourceLine => self.source_line_expression(span)?,
            syntax::ExpressionKind::Code(body) => self.capture_code(body, span)?,
            syntax::ExpressionKind::Insert(directive) => self.insert_expression(directive)?,
            syntax::ExpressionKind::Type(ty) => Expr::Type(self.reflected_type_syntax(ty, span)?),
            syntax::ExpressionKind::TypeQuery { query, value } => {
                self.type_query(*query, value, span)?
            }
            syntax::ExpressionKind::Null => Expr::Null,
            syntax::ExpressionKind::AddressOf(source) => self.address_expression(source, span)?,
            syntax::ExpressionKind::Dereference(source) => {
                self.dereference_expression(source, span)?
            }
            syntax::ExpressionKind::Index { base, index } => {
                self.index_expression(base, index, span)?
            }
            syntax::ExpressionKind::Context => self.context_expression(span)?,
            syntax::ExpressionKind::InferredMember(_) => {
                return Err(Diagnostic::new(
                    span,
                    "leading-dot member requires a contextual enum or type",
                ));
            }
            syntax::ExpressionKind::HereString(literal) => {
                self.string_literal(&literal.bytes, span)?
            }
            syntax::ExpressionKind::String(bytes) => self.string_literal(bytes, span)?,
            syntax::ExpressionKind::ArrayLiteral(literal) => {
                self.array_literal(literal, None, span)?
            }
            syntax::ExpressionKind::Integer(n) => Expr::Literal(*n),
            syntax::ExpressionKind::Character(value) => Expr::Int(IntExpr::constant(
                IntegerValue::wrapping(IntegerType::U8, i128::from(*value)),
            )),
            syntax::ExpressionKind::Float(value) => Expr::float_literal(value),
            syntax::ExpressionKind::Bool(b) => Expr::Bool(BoolExpr::Constant(*b)),
            syntax::ExpressionKind::CompileTimePredicate => Expr::Bool(BoolExpr::CompileTime),
            syntax::ExpressionKind::Name(name) => self.path_expression(
                &syntax::NamePath {
                    root: *name,
                    members: Vec::new(),
                },
                span,
            )?,
            syntax::ExpressionKind::QualifiedName(path) => self.path_expression(path, span)?,
            syntax::ExpressionKind::Call(name, arguments) => {
                let path = syntax::NamePath {
                    root: *name,
                    members: Vec::new(),
                };
                if let Some(ty) = self.try_record_application(&path, arguments, span)? {
                    return Ok(ty);
                }
                self.resolve_call(*name, arguments, span)?
            }
            syntax::ExpressionKind::QualifiedCall(path, arguments) => {
                if let Some(ty) = self.try_record_application(path, arguments, span)? {
                    return Ok(ty);
                }
                self.resolve_call_path(path, arguments, span)?
            }
            syntax::ExpressionKind::CallHint { hint, call } => {
                self.hinted_call_expression(*hint, call, span)?
            }
            syntax::ExpressionKind::IndirectCall { callee, args } => {
                if let syntax::ExpressionKind::ShortLambda(source) = &callee.kind {
                    return self.short_lambda_call(source, callee.span, args, span);
                }
                let target = self.expr(callee)?;
                self.indirect_call_from_source(target, args, span, Some(callee))?
            }
            syntax::ExpressionKind::ContextCall {
                callee,
                args,
                overrides,
            } => self.context_call(callee, args, overrides, span)?,
            syntax::ExpressionKind::StructLiteral(literal) => {
                self.record_literal(literal, None, span)?
            }
            syntax::ExpressionKind::PositionalStructLiteral(literal) => {
                self.positional_record_literal(literal, None, span)?
            }
            syntax::ExpressionKind::Member { base, member } => {
                if matches!(base.kind, syntax::ExpressionKind::Context) {
                    return self.context_member(*member, span);
                }
                let base = self.expr(base)?;
                self.member_value(base, *member, span)?
            }
            syntax::ExpressionKind::CompileTime(body) => self.resolve_compile_time(body, span)?,
            syntax::ExpressionKind::InferredCast { .. } => {
                return Err(Diagnostic::new(
                    span,
                    "xx cast requires a destination type from its context",
                ));
            }
            syntax::ExpressionKind::TypeCast { mode, ty, value } => {
                let target = self.lexical_annotation(ty, span)?;
                self.cast_expression(value, target, *mode, span)?
            }
            syntax::ExpressionKind::Conditional(e) => {
                let condition = self.condition_expression(&e.condition)?;
                let yes = self.expr(&e.then_value)?;
                let no = e.else_value.as_ref().map(|e| self.expr(e)).transpose()?;
                if matches!(yes, Expr::Pointer { .. } | Expr::Null)
                    || no
                        .as_ref()
                        .is_some_and(|value| matches!(value, Expr::Pointer { .. } | Expr::Null))
                {
                    return self.pointer_conditional_pair(
                        condition,
                        yes,
                        no.unwrap_or(Expr::Null),
                        None,
                        span,
                    );
                }
                if yes.has_float() || no.as_ref().is_some_and(Expr::has_float) {
                    return Ok(Expr::WeakConditional(Box::new(Conditional {
                        condition,
                        then_value: yes,
                        else_value: no.unwrap_or(Expr::Literal(0)),
                    })));
                }
                if matches!(yes, Expr::Type(_)) {
                    let ty = self.types.meta_type();
                    let yes = self.runtime_type_expression(yes, span)?;
                    let no = no
                        .map(|value| self.runtime_type_expression(value, span))
                        .transpose()?;
                    return self.value_conditional_pair(condition, yes, no, ty, span);
                }
                if matches!(yes, Expr::Typed { .. } | Expr::Enum { .. }) {
                    let ty = self.expression_type(&yes, span)?;
                    let no = if ty == self.types.meta_type() {
                        no.map(|value| self.runtime_type_expression(value, span))
                            .transpose()?
                    } else {
                        no
                    };
                    return self.value_conditional_pair(condition, yes, no, ty, span);
                }
                match yes {
                    Expr::Bool(yes) => Expr::Bool(BoolExpr::Conditional(Box::new(Conditional {
                        condition,
                        then_value: yes,
                        else_value: match no {
                            Some(e) => e.bool(span)?,
                            None => BoolExpr::Constant(false),
                        },
                    }))),
                    Expr::Type(_)
                    | Expr::Code(_)
                    | Expr::Typed { .. }
                    | Expr::Enum { .. }
                    | Expr::Pointer { .. }
                    | Expr::Null => {
                        return Err(Diagnostic::new(
                            span,
                            "nominal conditional values are not implemented",
                        ));
                    }
                    Expr::Void(_) | Expr::IndirectVoid { .. } => {
                        return Err(Diagnostic::new(
                            span,
                            "void call cannot supply an ifx result",
                        ));
                    }
                    yes => {
                        let no = no.unwrap_or(Expr::Literal(0));
                        if yes.weak_integer() && no.weak_integer() {
                            return Ok(Expr::WeakConditional(Box::new(Conditional {
                                condition,
                                then_value: yes,
                                else_value: no,
                            })));
                        }
                        let (yes, no) = Self::integer_pair(yes, no, span)?;
                        Expr::Int(IntExpr::new(
                            yes.ty(),
                            IntExprKind::Conditional(Box::new(Conditional {
                                condition,
                                then_value: yes,
                                else_value: no,
                            })),
                        ))
                    }
                }
            }
            syntax::ExpressionKind::Unary(op, e) => {
                if let Some(result) =
                    self.overloaded_operator(syntax::OperatorKind::Unary(*op), &[e], span)
                {
                    return result;
                }
                let value = if *op == UnaryOp::LogicalNot && inferred_casts::needs_cast_context(e) {
                    Expr::Bool(self.condition_expression(e)?)
                } else {
                    self.expr(e)?
                };
                if *op == UnaryOp::Complement && matches!(value, Expr::Enum { .. }) {
                    return self.enum_unary(*op, value, span);
                }
                if self.is_variant_expression(&value) {
                    return self.variant_unary(*op, value, span);
                }
                if value.has_float() {
                    match op {
                        UnaryOp::Positive => return Ok(value),
                        UnaryOp::Negate => return value.negate_float(span),
                        UnaryOp::Complement => {
                            return Err(Diagnostic::new(
                                span,
                                "operator is not defined for floating-point values",
                            ));
                        }
                        _ => {}
                    }
                }
                match (op, value) {
                    (UnaryOp::LogicalNot, e) => {
                        Expr::Bool(BoolExpr::Not(Box::new(e.condition(span, self.types)?)))
                    }
                    (UnaryOp::Positive, Expr::Literal(n)) => Expr::Literal(n),
                    (UnaryOp::Positive, e) => Expr::Int(e.int(span)?),
                    (UnaryOp::Negate, Expr::Literal(n)) => Expr::Literal(
                        n.checked_neg()
                            .ok_or_else(|| Diagnostic::new(span, "integer literal overflow"))?,
                    ),
                    (UnaryOp::Complement, Expr::Literal(n)) => Expr::Literal(!n),
                    (op, e) => {
                        let e = e.int(span)?;
                        let ty = e.ty();
                        Expr::Int(
                            IntExpr::new(
                                ty,
                                match op {
                                    UnaryOp::Negate => IntExprKind::Negate(Box::new(e)),
                                    UnaryOp::Complement => IntExprKind::Complement(Box::new(e)),
                                    _ => unreachable!(),
                                },
                            )
                            .with_overflow_check(self.checks.arithmetic_overflow),
                        )
                    }
                }
            }
            syntax::ExpressionKind::Cast(mode, ty, e) => {
                self.cast_expression(e, self.types.scalar(*ty), *mode, span)?
            }
            syntax::ExpressionKind::Binary(op, a, b) => {
                if let Some(result) =
                    self.overloaded_operator(syntax::OperatorKind::Binary(*op), &[a, b], span)
                {
                    return result;
                }
                if matches!(a.kind, syntax::ExpressionKind::InferredCast { .. })
                    || matches!(b.kind, syntax::ExpressionKind::InferredCast { .. })
                {
                    return self.inferred_binary(*op, a, b, span);
                }
                let (a, b) = if Self::contextual_enum_operand(a) {
                    let b = self.expr(b)?;
                    let ty = self.expression_type(&b, span)?;
                    (self.resolve_contextual_enum_operand(a, ty)?, b)
                } else {
                    let a = self.expr(a)?;
                    let b = if Self::contextual_enum_operand(b) {
                        let ty = self.expression_type(&a, span)?;
                        self.resolve_contextual_enum_operand(b, ty)?
                    } else {
                        self.expr(b)?
                    };
                    (a, b)
                };
                self.binary(*op, a, b, span)?
            }
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "expression lowering is not implemented for this syntax",
                ));
            }
        })
    }
    fn constant(value: ScalarConstant) -> Expr {
        match value {
            ScalarConstant::Literal(n) => Expr::Literal(n),
            ScalarConstant::Int(n) => Expr::Int(IntExpr::constant(n)),
            ScalarConstant::Bool(b) => Expr::Bool(BoolExpr::Constant(b)),
            ScalarConstant::Float(value) => Expr::Float(FloatExpr::constant(value)),
            ScalarConstant::WeakFloat(value) => Expr::WeakFloat(floats::WeakFloat::Bound(value)),
        }
    }
    fn integer_pair(a: Expr, b: Expr, span: Span) -> Result<(IntExpr, IntExpr), Diagnostic> {
        let ty = match (a.integer_type(span)?, b.integer_type(span)?) {
            (None, None) => IntegerType::S64,
            (Some(ty), None) | (None, Some(ty)) => ty,
            (Some(a), Some(b)) => a.common(b).ok_or_else(|| {
                Diagnostic::new(span, "integer operands have incompatible ranges")
            })?,
        };
        Ok((a.int_as(ty, span)?, b.int_as(ty, span)?))
    }
    fn binary(&mut self, op: BinaryOp, a: Expr, b: Expr, span: Span) -> Result<Expr, Diagnostic> {
        if self.is_string_expression(&a) || self.is_string_expression(&b) {
            return self.string_binary(op, a, b, span);
        }
        if let (Expr::Type(left), Expr::Type(right)) = (&a, &b) {
            return match op {
                BinaryOp::Equal => Ok(Expr::Bool(BoolExpr::Constant(left == right))),
                BinaryOp::NotEqual => Ok(Expr::Bool(BoolExpr::Constant(left != right))),
                _ => Err(Diagnostic::new(
                    span,
                    "type values only support nominal equality comparisons",
                )),
            };
        }
        let (a, b) = self.runtime_type_binary_operands(a, b, span)?;
        if self.is_runtime_type_expression(&a) || self.is_runtime_type_expression(&b) {
            return self.runtime_type_binary(op, a, b, span);
        }
        if self.is_variant_expression(&a) || self.is_variant_expression(&b) {
            return self.variant_binary(op, a, b, span);
        }
        if a.procedure_type(self.types).is_some() || b.procedure_type(self.types).is_some() {
            return self.procedure_binary(op, a, b, span);
        }
        if matches!(a, Expr::Pointer { .. } | Expr::Null)
            || matches!(b, Expr::Pointer { .. } | Expr::Null)
        {
            return self.pointer_binary(op, a, b, span);
        }
        if (a.has_float() || b.has_float())
            && !matches!(Operator::from(op), Operator::And | Operator::Or)
        {
            return self.float_binary(op, a, b, span);
        }
        if matches!(a, Expr::Enum { .. }) || matches!(b, Expr::Enum { .. }) {
            return self.enum_binary(op, a, b, span);
        }
        if let (Expr::Literal(a), Expr::Literal(b)) = (&a, &b)
            && let Ok(value) = jai_eval::binary_literals(op, *a, *b, span)
        {
            return Ok(Self::constant(value));
        }
        Ok(match Operator::from(op) {
            Operator::Integer(op) => {
                let (a, b) = Self::integer_pair(a, b, span)?;
                Expr::Int(
                    IntExpr::new(a.ty(), IntExprKind::Binary(op, Box::new(a), Box::new(b)))
                        .with_overflow_check(self.checks.arithmetic_overflow),
                )
            }
            Operator::Relation(op) => {
                let (a, b) = Self::integer_pair(a, b, span)?;
                Expr::Bool(BoolExpr::CompareInts(op, Box::new(a), Box::new(b)))
            }
            Operator::Equality(op) => match (a, b) {
                (Expr::Bool(a), Expr::Bool(b)) => {
                    Expr::Bool(BoolExpr::CompareBools(op, Box::new(a), Box::new(b)))
                }
                (a, b) => {
                    let (a, b) = Self::integer_pair(a, b, span)?;
                    Expr::Bool(BoolExpr::CompareInts(
                        match op {
                            Equality::Equal => Relation::Equal,
                            Equality::NotEqual => Relation::NotEqual,
                        },
                        Box::new(a),
                        Box::new(b),
                    ))
                }
            },
            Operator::And => Expr::Bool(BoolExpr::And(
                Box::new(a.condition(span, self.types)?),
                Box::new(b.condition(span, self.types)?),
            )),
            Operator::Or => Expr::Bool(BoolExpr::Or(
                Box::new(a.condition(span, self.types)?),
                Box::new(b.condition(span, self.types)?),
            )),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn check(source: &str) -> Result<Program, Diagnostic> {
        resolve(&syntax::parse(source).unwrap())
    }
    #[test]
    fn conditional_results_require_matching_non_void_types() {
        for source in [
            "main :: ()->int { return ifx true then 1 else false; }",
            "main :: ()->bool { return ifx false then false else 1; }",
            "f :: () {} main :: ()->int { return ifx true then f() else 1; }",
            "f :: () {} main :: ()->int { return ifx false then 1 else f(); }",
            "f :: () {} main :: ()->int { return ifx f() then 1 else 2; }",
            "main :: ()->int { return ifx true then 1 else missing; }",
            "N :: ifx true then 1 else M; M :: N; main :: ()->int { return N; }",
        ] {
            assert!(check(source).is_err(), "{source}");
        }
        assert!(
            check("N :: ifx true then M else 1 / 0; M :: 42; main :: ()->int { return N; }")
                .is_ok()
        );
    }
    #[test]
    fn integer_conversions_reject_range_loss_and_oversized_literals() {
        for source in [
            "main :: () { n:u8 = 256; }",
            "main :: () { n:s8 = 128; }",
            "main :: () { n:u64 = -1; }",
            "main :: () { n := 18446744073709551615; }",
            "main :: () { a:u16=42; b:u8=a; }",
            "main :: () { a:s8=42; b:u64=a; }",
            "main :: () { a:u64=42; b:s64=a; }",
            "main :: () { a:s8=1; b:u8=1; c:=a+b; }",
            "f :: (n:u8) {} main :: () { f(256); }",
            "f :: ()->u8 { n:u16=1; return n; } main :: () {}",
            "N : u8 : 256; main :: () {}",
            "n:u8 = 256; main :: () {}",
            "main :: () { n := +true; }",
        ] {
            assert!(check(source).is_err(), "{source}");
        }
        assert!(check("N :: 255; main :: () { n:u8=N; large:u64=18446744073709551615; }").is_ok());
    }
    #[test]
    fn reject_unresolved_names_and_arity() {
        for src in [
            "main :: () { x = 1; }",
            "main :: () { missing(); }",
            "f :: (x:int) {} main :: () { f(); }",
        ] {
            assert!(check(src).is_err(), "{src}");
        }
    }
    #[test]
    fn preserve_scalar_types() {
        for src in [
            "main :: () { x: int = true; }",
            "main :: () { x: bool = 1; }",
            "main :: ()->int { return true; }",
            "f :: (b:bool) {} main :: () { f(1); }",
            "main :: () { x := true + false; }",
        ] {
            assert!(check(src).is_err(), "{src}");
        }
    }
    #[test]
    fn void_values_cannot_escape() {
        assert!(check("f :: () {} main :: () { f(); }").is_ok());
        assert!(check("f :: () {} main :: () { x := f(); }").is_err());
    }
    #[test]
    fn return_flow_and_scope() {
        for src in [
            "main :: ()->int {}",
            "main :: () { return; x := 1; }",
            "main :: () { { x := 1; } x = 2; }",
            "main :: () { x := 1; x := 2; }",
        ] {
            assert!(check(src).is_err(), "{src}");
        }
        assert!(check("main :: ()->int { if true return 1; else return 2; }").is_ok());
    }
    #[test]
    fn jumps_require_active_loop_targets_and_reachable_code() {
        for source in [
            "main :: () { break; }",
            "main :: () { continue; }",
            "main :: () { x := 1; while true { break x; } }",
            "main :: () { for i: 1..3 {} break i; }",
            "main :: () { while true { break; x := 1; } }",
            "main :: () { while true { if true continue; else break; x := 1; } }",
            "main :: ()->int { while true { break; } }",
        ] {
            assert!(check(source).is_err(), "{source}");
        }
    }
    #[test]
    fn iterator_and_while_bindings_are_scoped_and_typed() {
        for source in [
            "main :: () { for true..4 {} }",
            "main :: () { for 1..false {} }",
            "main :: () { for i: 1..3 {} i = 4; }",
            "main :: () { while value := true {} value = false; }",
            "main :: () { while value := true { value = 1; } }",
            "f :: () {} main :: () { while value := f() {} }",
        ] {
            assert!(check(source).is_err(), "{source}");
        }
        assert!(check("main :: () { for i: 1..3 { for i: 1..3 break i; continue i; } }").is_ok());
    }
    #[test]
    fn cleanup_cannot_escape_to_its_enclosing_procedure_or_loop() {
        for source in [
            "main :: () { defer return; }",
            "main :: () { for i: 1..3 { defer break i; } }",
            "main :: () { while true { defer continue; } }",
            "main :: () { defer x = 1; x := 0; }",
            "main :: () { defer { x := 1; } x = 2; }",
            "main :: () { return; defer {} }",
        ] {
            assert!(check(source).is_err(), "{source}");
        }
        assert!(check("main :: () { defer { for i: 1..3 { if i == 2 break i; } } }").is_ok());
    }
    #[test]
    fn constants_are_immutable_and_dependencies_must_be_acyclic() {
        for source in [
            "A :: B; B :: A; main :: () {}",
            "A :: A; main :: () {}",
            "A :: 1; main :: () { A = 2; }",
            "main :: () { A :: 1; A += 2; }",
            "main :: (x:int) { A :: x; }",
            "main :: () { x := 1; { A :: x; } }",
            "A :: missing; main :: () {}",
            "A :: 1 / 0; main :: () {}",
            "A : bool : 1; main :: () {}",
            "A :: 1; A := 2; main :: () {}",
            "main :: () { A :: 1; A :: 2; }",
        ] {
            assert!(check(source).is_err(), "{source}");
        }
    }
    #[test]
    fn global_initializers_and_scalar_calls_are_checked() {
        for source in [
            "x : bool = 1; main :: () {}",
            "x : int = true; main :: () {}",
            "x := y; y := 3; main :: () {}",
            "f :: ()->int { return 1; } x := f(); main :: () {}",
            "main :: () {} main := 0;",
            "f :: () {} main :: () { f := 2; f(); }",
        ] {
            assert!(check(source).is_err(), "{source}");
        }
    }
    #[test]
    fn deep_constant_dependency_chains_use_a_worklist() {
        use std::fmt::Write;
        let mut source = String::new();
        for n in 0..20_000 {
            writeln!(source, "C{n} :: C{} + 1;", n + 1).unwrap();
        }
        source.push_str("C20000 :: 0; main :: ()->int { return C0; }");
        assert!(check(&source).is_ok());
    }
    #[test]
    fn entry_point_and_parameter_invariants() {
        for src in [
            "f :: () {}",
            "main :: (x:int) {}",
            "main :: ()->bool { return true; }",
            "f :: (x:int,x:bool) {} main :: () {}",
        ] {
            assert!(check(src).is_err(), "{src}");
        }
    }
}
