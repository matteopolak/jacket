//! Specialized baked members enter lexical lookup as ordinary typed bindings.
use crate::polymorphism::BakedValue;
use crate::{Binding, Diagnostic, Resolver, Span, Symbol};
use jai_types::TypeId;

#[derive(Clone, Copy)]
enum MethodPhase<'a> {
    TypesOnly,
    CompleteHeaders,
    Bodies,
    Prerequisites(&'a std::collections::HashSet<jai_ir::ProcedureId>),
}

/// Pure source descriptions keep baked values without allocating pool handles.
pub(crate) enum ReadyInstanceRecordMember {
    Binding(Binding),
    Baked(BakedValue),
}

impl Resolver<'_> {
    pub(crate) fn prepare_record_source_namespace(
        &mut self,
        owner: TypeId,
    ) -> Result<(), Diagnostic> {
        self.bind_specialized_record_method_phase(owner, MethodPhase::TypesOnly)
            .map(|_| ())
    }

    pub(crate) fn specialized_record_member(
        &mut self,
        ty: TypeId,
        name: Symbol,
        _span: Span,
    ) -> Result<Option<Binding>, Diagnostic> {
        if let Some(enumeration) = self.meta.record_specializations.member_enum(ty)
            && let Some((_, value)) = enumeration
                .values
                .iter()
                .find(|(member, _)| *member == name)
        {
            return Ok(Some(Binding::Enum(
                crate::modules::aggregates::EnumConstant {
                    ty,
                    value: *value,
                },
            )));
        }
        if self
            .meta
            .record_specializations
            .methods(ty)
            .is_some_and(|methods| methods.iter().any(|method| method.source.name() == name))
        {
            return Ok(self
                .bind_specialized_record_methods(ty)?
                .get(&name)
                .cloned());
        }
        let Some(scope) = self.meta.record_specializations.member_bindings(ty) else {
            return Ok(None);
        };
        let Some(value) = scope
            .constant(name)
            .cloned()
            .or_else(|| scope.ty(name).map(BakedValue::Type))
        else {
            return Ok(None);
        };
        Ok(Some(self.baked_record_binding(value)))
    }

    /// Instance access exports original members and this template's own baked
    /// formals. The broader lexical overlay remains definition-only metadata.
    pub(crate) fn specialized_instance_record_member(
        &mut self,
        ty: TypeId,
        name: Symbol,
        span: Span,
    ) -> Result<Option<Binding>, Diagnostic> {
        if self.instance_record_member_allowed(ty, name) {
            self.specialized_record_member(ty, name, span)
        } else {
            Ok(None)
        }
    }

    /// Read only published source facts; no method/default/body work is started.
    pub(crate) fn ready_instance_record_member(
        &self,
        ty: TypeId,
        name: Symbol,
        span: Span,
    ) -> Result<Option<ReadyInstanceRecordMember>, Diagnostic> {
        if !self.instance_record_member_allowed(ty, name) {
            return Ok(None);
        }
        let method = self
            .meta
            .record_specializations
            .methods(ty)
            .and_then(|methods| methods.iter().find(|method| method.source.name() == name));
        if let Some(binding) = self.ready_namespace_member(ty, name) {
            if method.is_some()
                && let Binding::Procedure {
                    procedure, ..
                } = binding
                && self.meta.local_declarations.header_readiness(procedure)
                    != Some(crate::local_declarations::HeaderReadiness::Complete)
            {
                if let Some(context) = self.compile_time {
                    context.record_pending(vec![jai_vm::Dependency::Procedure(procedure)]);
                }
                return Err(Diagnostic::new(
                    span,
                    "record member requires its checked header defaults",
                ));
            }
            return Ok(Some(ReadyInstanceRecordMember::Binding(binding)));
        }
        if method.is_some() {
            return Err(Diagnostic::new(
                span,
                "record member header has not been prepared",
            ));
        }
        let Some(bindings) = self.meta.record_specializations.member_bindings(ty) else {
            return Ok(None);
        };
        let value = bindings
            .constant(name)
            .cloned()
            .or_else(|| bindings.ty(name).map(BakedValue::Type));
        Ok(value.map(ReadyInstanceRecordMember::Baked))
    }

    fn instance_record_member_allowed(&self, ty: TypeId, name: Symbol) -> bool {
        let members = self
            .meta
            .record_specializations
            .source_member_names(ty)
            .is_some_and(|members| members.contains(&name));
        let formal = self
            .meta
            .record_specializations
            .record(ty)
            .filter(|record| !record.nested)
            .and_then(|record| record.origin)
            .and_then(|origin| self.graph_scope?.declarations.graph.declaration(origin.0))
            .is_some_and(|declaration| {
                matches!(&declaration.syntax().kind,
                    jai_syntax::FileDeclarationKind::Record(record)
                        if record.parameters.iter().any(|parameter| parameter.name == name))
            });
        members || formal
    }

    /// Every instantiated body is checked, including members never looked up.
    pub(crate) fn bind_all_record_methods_located(
        &mut self,
    ) -> Result<(), jai_source::LocatedDiagnostic> {
        self.record_method_sweep_located(MethodPhase::Bodies)
    }

    /// Publish canonical callable constants without claiming any body is ready.
    pub(crate) fn bind_all_record_method_signatures_located(
        &mut self,
    ) -> Result<(), jai_source::LocatedDiagnostic> {
        self.record_method_sweep_located(MethodPhase::TypesOnly)
    }

    /// Complete real defaults after providers become ready, without checking bodies.
    pub(crate) fn complete_all_record_method_signatures_located(
        &mut self,
    ) -> Result<(), jai_source::LocatedDiagnostic> {
        self.record_method_sweep_located(MethodPhase::CompleteHeaders)
    }

    pub(crate) fn bind_record_method_prerequisites_located(
        &mut self,
        required: &std::collections::HashSet<jai_ir::ProcedureId>,
    ) -> Result<(), jai_source::LocatedDiagnostic> {
        self.record_method_sweep_located(MethodPhase::Prerequisites(required))
    }

    fn record_method_sweep_located(
        &mut self,
        phase: MethodPhase<'_>,
    ) -> Result<(), jai_source::LocatedDiagnostic> {
        let result = self.sweep_record_methods(phase).map_err(|(file, error)| {
            let source = self
                .graph_scope
                .expect("source record methods bind within a file scope")
                .code_file(file)
                .source();
            jai_source::LocatedDiagnostic::new(source, error)
        });
        if matches!(phase, MethodPhase::Bodies | MethodPhase::Prerequisites(_)) {
            let source = self
                .graph_scope
                .expect("source record methods bind within a file scope")
                .source();
            let local = self
                .bind_pending_local_record_methods()
                .map_err(|error| jai_source::LocatedDiagnostic::new(source, error));
            result.and(local)
        } else {
            result
        }
    }

    fn sweep_record_methods(
        &mut self,
        phase: MethodPhase<'_>,
    ) -> Result<(), (jai_modules::FileInstanceId, Diagnostic)> {
        let mut visited = std::collections::HashSet::new();
        let mut first_error = None;
        loop {
            let mut owners: Vec<_> = self
                .meta
                .record_specializations
                .method_owners()
                .filter(|ty| {
                    !visited.contains(ty)
                        && (matches!(phase, MethodPhase::TypesOnly)
                            || self.meta.record_specializations.record(*ty).is_some())
                })
                .collect();
            if owners.is_empty() {
                return first_error.map_or(Ok(()), Err);
            }
            owners.sort_by_key(|ty| ty.index());
            for owner in owners {
                let file = self
                    .meta
                    .record_specializations
                    .method_environment(owner)
                    .expect("record method environment was reserved")
                    .file;
                if visited.len() == 65_536 {
                    return Err((
                        file,
                        Diagnostic::new(
                            self.meta.record_specializations.methods(owner).unwrap()[0]
                                .source
                                .span(),
                            "record method instantiation exceeds declaration budget",
                        ),
                    ));
                }
                if let Err(error) = self.bind_specialized_record_method_phase(owner, phase) {
                    first_error.get_or_insert((file, error));
                }
                visited.insert(owner);
            }
        }
    }

    fn bind_specialized_record_methods(
        &mut self,
        owner: TypeId,
    ) -> Result<std::collections::HashMap<Symbol, Binding>, Diagnostic> {
        self.bind_specialized_record_method_phase(owner, MethodPhase::Bodies)
    }

    fn bind_specialized_record_method_phase(
        &mut self,
        owner: TypeId,
        phase: MethodPhase<'_>,
    ) -> Result<std::collections::HashMap<Symbol, Binding>, Diagnostic> {
        let environment = self
            .meta
            .record_specializations
            .method_environment(owner)
            .ok_or_else(|| Diagnostic::new(self.span, "record method environment is incomplete"))?;
        let methods = self
            .meta
            .record_specializations
            .methods(owner)
            .unwrap_or_default()
            .to_vec();
        let mut bindings = Vec::new();
        for binding in &environment.substitution.types {
            if Some(binding.name) != environment.name
                && !methods
                    .iter()
                    .any(|method| method.source.name() == binding.name)
            {
                bindings.push((binding.name, Binding::Type(binding.ty)));
            }
        }
        for binding in &environment.substitution.constants {
            if Some(binding.name) != environment.name
                && !methods
                    .iter()
                    .any(|method| method.source.name() == binding.name)
            {
                bindings.push((
                    binding.name,
                    self.baked_record_binding(binding.value.clone()),
                ));
            }
        }
        let namespace = match phase {
            MethodPhase::Prerequisites(required) => self.bind_record_method_prerequisites(
                crate::local_declarations::RecordMethodPrerequisites {
                    owner,
                    methods: &methods,
                    bindings,
                    file: environment.file,
                    substitution: &environment.substitution,
                    required,
                },
            )?,
            MethodPhase::Bodies => self.bind_record_methods(
                owner,
                &methods,
                bindings,
                environment.file,
                &environment.substitution,
            )?,
            MethodPhase::TypesOnly => self.bind_record_method_signatures(
                owner,
                &methods,
                bindings,
                environment.file,
                &environment.substitution,
            )?,
            MethodPhase::CompleteHeaders => self.complete_record_method_signatures(
                owner,
                &methods,
                bindings,
                environment.file,
                &environment.substitution,
            )?,
        };
        let mut constants = Vec::with_capacity(methods.len());
        for method in &methods {
            if matches!(&method.source, super::RecordMethodSource::Procedure(source) if source.expands)
                && matches!(
                    namespace.get(&method.source.name()),
                    Some(Binding::Macro(_))
                )
            {
                continue;
            }
            let Some(Binding::Procedure {
                procedure,
                ty,
            }) = namespace.get(&method.source.name())
            else {
                if matches!(&method.source, super::RecordMethodSource::Constant(_)) {
                    // Contextual lambdas retain their actual declaration environment.
                    // The call/value producer selects a real signature before lowering.
                    continue;
                }
                return Err(Diagnostic::new(
                    method.source.span(),
                    "record method header did not publish a callable identity",
                ));
            };
            if !matches!(self.types.kind(*ty), Ok(jai_types::TypeKind::Procedure(_))) {
                return Err(Diagnostic::new(
                    method.source.span(),
                    "record method header does not have a canonical procedure type",
                ));
            }
            constants.push((
                method.source.name(),
                jai_ir::ConstantValue {
                    ty: *ty,
                    kind: jai_ir::ConstantKind::Procedure(*procedure),
                },
            ));
        }
        self.meta
            .record_specializations
            .publish_method_constants(owner, constants);
        Ok(namespace)
    }

    pub(crate) fn baked_record_binding(&mut self, value: BakedValue) -> Binding {
        match value {
            BakedValue::Type(ty) => Binding::Type(ty),
            BakedValue::Code(id) => Binding::Code(id),
            BakedValue::Value(jai_ir::ConstantValue {
                ty,
                kind: jai_ir::ConstantKind::Enum(value),
            }) => Binding::Enum(crate::modules::aggregates::EnumConstant {
                ty,
                value,
            }),
            BakedValue::Value(value) => Binding::TypedConstant(self.meta.intern_constant(value)),
            BakedValue::Float(value) => {
                Binding::TypedConstant(self.meta.intern_constant(jai_ir::ConstantValue {
                    ty: self.types.float(value.ty()),
                    kind: jai_ir::ConstantKind::Float(value),
                }))
            }
            BakedValue::String(bytes) => {
                Binding::TypedConstant(self.meta.intern_constant(jai_ir::ConstantValue {
                    ty: self.types.string(),
                    kind: jai_ir::ConstantKind::StringBytes(bytes.into_vec()),
                }))
            }
        }
    }
}
