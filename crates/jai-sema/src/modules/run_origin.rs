//! Stable replay metadata never contains semantic arena identities.
use super::*;
use crate::metaprogram::RunLexicalFacts;
use crate::polymorphism::{BakedValue, Substitution};
use jai_ir::{ConstantKind, ConstantValue};
use jai_types::{TypeId, TypeKind, TypeView};
mod lexical;
mod stable_values;

struct RunOriginRequest<'a> {
    workspace: jai_vm::WorkspaceId,
    owner: jai_ir::ProcedureId,
    location: jai_source::SourceSpan,
    lexical: &'a RunLexicalFacts,
}

impl crate::Resolver<'_> {
    pub(crate) fn source_run_origin(
        &self,
        workspace: jai_vm::WorkspaceId,
        owner: jai_ir::ProcedureId,
        source: jai_source::SourceId,
        span: Span,
    ) -> Result<jai_vm::SourceOrigin, Diagnostic> {
        let lexical = self.run_lexical_facts(span)?;
        self.graph_scope
            .ok_or_else(|| Diagnostic::new(span, "#run replay requires a retained source graph"))?
            .run_origin(
                self.types,
                self.meta,
                RunOriginRequest {
                    workspace,
                    owner,
                    location: jai_source::SourceSpan { source, span },
                    lexical: &lexical,
                },
            )
    }
}

impl crate::modules::FileScope<'_> {
    pub(crate) fn module_environment_origin(
        &self,
        file: FileInstanceId,
        span: Span,
    ) -> Result<Vec<u8>, Diagnostic> {
        self.declarations
            .graph
            .module_environment_origin(file)
            .map_err(|error| {
                let source = self
                    .declarations
                    .graph
                    .file(file)
                    .map_or(self.source(), |file| file.source());
                Diagnostic::at_source(jai_source::SourceSpan { source, span }, error.to_string())
            })
    }
    pub(crate) fn is_isolated_procedure(&self, procedure: jai_ir::ProcedureId) -> bool {
        self.declarations
            .generics
            .borrow()
            .is_isolated_procedure(procedure)
    }
    fn nominal_origin(&self, ty: TypeId) -> Result<Option<Vec<u8>>, jai_vm::Error> {
        let graph = self.declarations.graph;
        let mut identities = vec![];
        for (&id, &candidate) in &self.declarations.nominals.declarations {
            if candidate != ty {
                continue;
            }
            let Some(declaration) = graph.declaration(id) else {
                continue;
            };
            let defines_nominal = match &declaration.syntax().kind {
                syntax::FileDeclarationKind::Record(_) | syntax::FileDeclarationKind::Enum(_) => {
                    true
                }
                syntax::FileDeclarationKind::TypeAlias(alias) => matches!(
                    alias.ty,
                    syntax::TypeSyntax::Variant { .. }
                        | syntax::TypeSyntax::InlineRecord(_)
                        | syntax::TypeSyntax::InlineEnum(_)
                ),
                _ => false,
            };
            if !defines_nominal {
                continue;
            }
            let source = graph
                .sources()
                .get(declaration.location().source)
                .ok_or(jai_vm::Error::InvalidIr("nominal replay source is absent"))?;
            let environment = graph
                .module_environment_origin(declaration.file())
                .map_err(|_| jai_vm::Error::InvalidIr("nominal source environment is not ready"))?;
            let mut identity = vec![];
            for token in [
                source.path().as_os_str().as_encoded_bytes(),
                &(declaration.location().span.start as u64).to_le_bytes(),
                &(declaration.location().span.end as u64).to_le_bytes(),
                graph.symbols().name(declaration.name()).as_bytes(),
                &environment,
            ] {
                identity.extend_from_slice(&(token.len() as u64).to_le_bytes());
                identity.extend_from_slice(token);
            }
            identities.push(identity);
        }
        identities.sort();
        Ok(identities.into_iter().next())
    }
    fn run_origin(
        &self,
        types: &dyn TypeView,
        meta: &crate::reflection::MetaContext,
        request: RunOriginRequest<'_>,
    ) -> Result<jai_vm::SourceOrigin, jai_source::Diagnostic> {
        let RunOriginRequest {
            workspace,
            owner,
            location:
                jai_source::SourceSpan {
                    source: source_id,
                    span,
                },
            lexical,
        } = request;
        let source = self.declarations.graph.sources().get(source_id).unwrap();
        let body = source
            .text()
            .as_bytes()
            .get(span.start..span.end)
            .ok_or_else(|| jai_source::Diagnostic::new(span, "invalid source run span"))?
            .to_vec();
        let body_hash = body.iter().fold(0xcbf29ce484222325u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        });
        let mut encoder = Encoder {
            types,
            meta,
            lexical,
            local: &meta.local_declarations,
            scope: self,
            seen: HashMap::new(),
            bytes: vec![],
            depth: 0,
        };
        encoder.token(b"module-environment");
        encoder.token(&self.module_environment_origin(self.file, span)?);
        if let Some(origin) = meta.local_declarations.stable_procedure_origin(owner) {
            encoder.token(b"run-owner");
            encoder.token(origin);
            encoder
                .origin_environments(meta.local_declarations.procedure_origin_files(owner))
                .map_err(|error| {
                    Diagnostic::at_source(
                        jai_source::SourceSpan {
                            source: source_id,
                            span,
                        },
                        error.to_string(),
                    )
                })?;
        } else if let Some(declaration) = self
            .declarations
            .signatures
            .iter()
            .find_map(|(id, signature)| (signature.id == owner).then_some(*id))
            .and_then(|id| self.declarations.graph.declaration(id))
        {
            let source = self
                .declarations
                .graph
                .sources()
                .get(declaration.location().source)
                .unwrap();
            encoder.token(b"run-owner");
            encoder.token(&self.module_environment_origin(declaration.file(), span)?);
            encoder.token(source.path().as_os_str().as_encoded_bytes());
            encoder.text(declaration.location().span.start);
            encoder.text(declaration.location().span.end);
            encoder.token(
                self.declarations
                    .graph
                    .symbols()
                    .name(declaration.name())
                    .as_bytes(),
            );
        }
        if let Some(substitution) = self.substitution {
            encoder
                .substitution(substitution)
                .map_err(|error| jai_source::Diagnostic::new(span, error.to_string()))?;
        }
        encoder.lexical_facts().map_err(|error| {
            Diagnostic::at_source(
                jai_source::SourceSpan {
                    source: source_id,
                    span,
                },
                error.to_string(),
            )
        })?;
        Ok(jai_vm::SourceOrigin {
            workspace,
            path: source.path().to_path_buf(),
            start: span.start,
            end: span.end,
            body_hash,
            body,
            specialization: encoder.bytes,
        })
    }
}
struct Encoder<'a, 'b> {
    types: &'a dyn TypeView,
    meta: &'a crate::reflection::MetaContext,
    lexical: &'a RunLexicalFacts,
    local: &'a crate::local_declarations::LocalDeclarationRegistry,
    scope: &'b crate::modules::FileScope<'b>,
    seen: HashMap<TypeId, usize>,
    depth: usize,
    bytes: Vec<u8>,
}
impl Encoder<'_, '_> {
    fn token(&mut self, bytes: &[u8]) {
        self.bytes
            .extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        self.bytes.extend_from_slice(bytes);
    }
    fn text(&mut self, value: impl std::fmt::Debug) {
        self.token(format!("{value:?}").as_bytes());
    }
    fn origin_environments(
        &mut self,
        files: Option<&[FileInstanceId]>,
    ) -> Result<(), jai_vm::Error> {
        let files = files.ok_or(jai_vm::Error::InvalidIr(
            "local replay origin has no source environments",
        ))?;
        self.token(b"local-source-environments");
        self.text(files.len());
        for &file in files {
            let environment = self
                .scope
                .declarations
                .graph
                .module_environment_origin(file)
                .map_err(|_| jai_vm::Error::InvalidIr("local source environment is not ready"))?;
            self.token(&environment);
        }
        Ok(())
    }
    fn ty(&mut self, ty: TypeId) -> Result<(), jai_vm::Error> {
        self.token(b"canonical-type-v2");
        if let Some(&index) = self.seen.get(&ty) {
            self.token(b"reference");
            self.token(&(index as u64).to_le_bytes());
            return Ok(());
        }
        if self.seen.len() >= 4096 {
            return Err(jai_vm::Error::Limit(jai_vm::LimitKind::ValueCells));
        }
        if self.depth >= 256 {
            return Err(jai_vm::Error::Limit(jai_vm::LimitKind::EvaluationDepth));
        }
        self.depth += 1;
        self.seen.insert(ty, self.seen.len());
        let nominal = matches!(
            self.types.kind(ty)?,
            TypeKind::Record(_) | TypeKind::Enum(_) | TypeKind::Distinct(_) | TypeKind::Any(_)
        );
        if nominal && self.specialized_type(ty)? {
            // The source template and its normalized arguments carry nominal identity.
        } else if nominal && let Some(origin) = self.scope.nominal_origin(ty)? {
            self.token(b"nominal");
            self.token(&origin);
        } else if nominal
            && let Some(origin) = self
                .local
                .stable_type_origin(ty, self.scope.declarations.graph.symbols())
        {
            self.token(b"local-nominal");
            self.token(&origin);
            self.origin_environments(self.local.type_origin_files(ty))?;
            if let Some(substitution) = self.local.type_origin_substitution(ty).cloned() {
                self.substitution(&substitution)?;
            }
        }
        match self.types.kind(ty)? {
            TypeKind::Void => self.token(b"void"),
            TypeKind::Type => self.token(b"type"),
            TypeKind::Code => self.token(b"code"),
            TypeKind::Bool => self.token(b"bool"),
            TypeKind::String => self.token(b"string"),
            TypeKind::Integer(value) => {
                self.token(b"integer");
                self.token(&stable_values::integer_type_key(*value));
            }
            TypeKind::Float(value) => {
                self.token(b"float");
                self.token(&value.bits().to_le_bytes());
            }
            TypeKind::Pointer(element) => {
                self.token(b"pointer");
                self.ty(*element)?;
            }
            TypeKind::Slice(element) => {
                self.token(b"slice");
                self.ty(*element)?;
            }
            TypeKind::DynamicArray(element) => {
                self.token(b"dynamic-array");
                self.ty(*element)?;
            }
            TypeKind::FixedArray { element, count } => {
                self.token(b"fixed-array");
                self.token(&count.to_le_bytes());
                self.ty(*element)?;
            }
            TypeKind::Any(id) => {
                self.token(b"any");
                let record = self.types.record(*id)?;
                self.token(&stable_values::record_layout_key(
                    *id,
                    record.fields.len(),
                    &record.layout,
                )?);
                self.token(&(record.fields.len() as u64).to_le_bytes());
                for &field in &record.fields {
                    self.ty(field)?;
                }
            }
            TypeKind::Record(id) => {
                let record = self.types.record(*id)?;
                self.token(b"record");
                self.token(match record.kind {
                    jai_types::RecordKind::Struct => b"struct",
                    jai_types::RecordKind::Union => b"union",
                });
                self.token(&stable_values::record_layout_key(
                    *id,
                    record.fields.len(),
                    &record.layout,
                )?);
                self.token(&(record.fields.len() as u64).to_le_bytes());
                for &field in &record.fields {
                    self.ty(field)?;
                }
            }
            TypeKind::Enum(id) => {
                let enumeration = self.types.enumeration(*id)?;
                self.token(b"enum");
                self.token(&stable_values::integer_type_key(enumeration.representation));
                self.token(&(enumeration.values.len() as u64).to_le_bytes());
                for value in &enumeration.values {
                    self.token(&value.bits().to_le_bytes());
                }
            }
            TypeKind::Distinct(id) => {
                let distinct = self.types.distinct(*id)?;
                self.token(b"distinct");
                self.token(match distinct.kind {
                    jai_types::DistinctKind::Distinct => b"distinct",
                    jai_types::DistinctKind::IsA => b"isa",
                });
                self.ty(distinct.representation)?;
            }
            TypeKind::Procedure(id) => {
                let procedure = self.types.procedure_type(*id)?;
                self.token(b"procedure");
                self.token(match procedure.convention {
                    jai_types::CallingConvention::Jai => b"jai",
                    jai_types::CallingConvention::C => b"c",
                    jai_types::CallingConvention::Stdcall => b"stdcall",
                    jai_types::CallingConvention::CppMethod => b"cpp-method",
                });
                self.token(match procedure.context {
                    jai_types::ContextMode::Implicit => b"implicit-context",
                    jai_types::ContextMode::None => b"no-context",
                });
                match procedure.variadic {
                    jai_types::Variadic::None => self.token(b"variadic-none"),
                    jai_types::Variadic::C { fixed_parameters } => {
                        self.token(b"variadic-c");
                        self.token(&(fixed_parameters as u64).to_le_bytes());
                    }
                    jai_types::Variadic::Jai { parameter, element } => {
                        self.token(b"variadic-jai");
                        self.token(&(parameter as u64).to_le_bytes());
                        self.ty(element)?;
                    }
                }
                self.token(&(procedure.parameters.len() as u64).to_le_bytes());
                for &ty in &procedure.parameters {
                    self.ty(ty)?;
                }
                self.token(&(procedure.results.len() as u64).to_le_bytes());
                for &ty in &procedure.results {
                    self.ty(ty)?;
                }
            }
        }
        self.depth -= 1;
        Ok(())
    }
    fn constant(&mut self, value: &ConstantValue) -> Result<(), jai_vm::Error> {
        self.token(b"checked-constant-v2");
        self.ty(value.ty)?;
        match &value.kind {
            ConstantKind::NativePointer(pointer) => {
                if pointer.type_id() != value.ty || pointer.validate(self.types).is_err() {
                    return Err(jai_vm::Error::InvalidIr(
                        "native address constant has an invalid source recipe",
                    ));
                }
                self.token(b"native-address-recipe");
                match pointer.source() {
                    jai_ir::NativePointerSource::Weak(value) => {
                        self.token(b"weak");
                        self.token(&value.to_le_bytes());
                    }
                    jai_ir::NativePointerSource::Strong(value) => {
                        self.token(b"strong");
                        self.token(&stable_values::integer_type_key(value.ty()));
                        self.token(&value.bits().to_le_bytes());
                    }
                }
                self.token(match pointer.mode() {
                    jai_types::CastMode::Checked => b"checked",
                    jai_types::CastMode::Unchecked => b"unchecked",
                    jai_types::CastMode::Truncate => b"truncate",
                    jai_types::CastMode::Force(_) => {
                        return Err(jai_vm::Error::InvalidIr(
                            "native address constant cannot contain a storage cast",
                        ));
                    }
                });
            }
            ConstantKind::RuntimeType(value) => {
                self.token(b"runtime-type");
                self.ty(value.identity().ty())?;
                self.token(b"layout-policy");
                self.token(&layout_policy_key(value.identity().policy()));
            }
            ConstantKind::Int(value) => {
                self.token(b"int");
                self.token(&value.bits().to_le_bytes());
            }
            ConstantKind::Float(value) => {
                self.token(b"float");
                self.token(&value.bits().to_le_bytes());
            }
            ConstantKind::Bool(value) => {
                self.token(b"bool");
                self.token(&[u8::from(*value)]);
            }
            ConstantKind::Enum(value) => {
                self.token(b"enum");
                self.token(&value.bits().to_le_bytes());
            }
            ConstantKind::Procedure(procedure) => {
                self.token(b"procedure");
                self.procedure_source(*procedure)?;
            }
            ConstantKind::Zero => self.token(b"zero"),
            ConstantKind::StringBytes(value) => {
                self.token(b"bytes");
                self.token(value);
            }
            ConstantKind::Record(values) | ConstantKind::Array(values) => {
                self.token(b"elements");
                self.token(&(values.len() as u64).to_le_bytes());
                for value in values {
                    self.constant(value)?;
                }
            }
            ConstantKind::Union { field, value } => {
                self.token(b"union");
                self.ty(self.types.record_type(field.record())?)?;
                self.token(&(field.index() as u64).to_le_bytes());
                self.constant(value)?;
            }
            ConstantKind::Distinct(value) => {
                self.token(b"distinct-value");
                self.constant(value)?;
            }
        }
        Ok(())
    }
    fn substitution(&mut self, substitution: &Substitution) -> Result<(), jai_vm::Error> {
        for binding in &substitution.types {
            self.token(
                self.scope
                    .declarations
                    .graph
                    .symbols()
                    .name(binding.name)
                    .as_bytes(),
            );
            self.ty(binding.ty)?;
        }
        for binding in &substitution.constants {
            self.token(
                self.scope
                    .declarations
                    .graph
                    .symbols()
                    .name(binding.name)
                    .as_bytes(),
            );
            match &binding.value {
                BakedValue::Type(ty) => self.ty(*ty)?,
                BakedValue::Value(value) => self.constant(value)?,
                BakedValue::Float(value) => {
                    self.token(b"baked-float");
                    self.ty(self.types.float(value.ty()))?;
                    self.token(&value.bits().to_le_bytes());
                }
                BakedValue::Code(id) => {
                    self.token(b"baked-code");
                    let fact = self
                        .lexical
                        .capture_for_code(*id)
                        .ok_or(jai_vm::Error::InvalidIr(
                            "code specialization has no retained source capture",
                        ))?
                        .clone();
                    self.binding_fact(&fact)?;
                }
                BakedValue::String(value) => {
                    self.token(b"baked-string");
                    self.token(value);
                }
            }
        }
        if !substitution.callables.is_empty() {
            self.token(b"callable-policies-v1");
            self.token(&(substitution.callables.len() as u64).to_le_bytes());
            for binding in &substitution.callables {
                <Self as crate::procedure_values::contracts::CallablePolicyEncoder>::symbol(
                    self,
                    binding.name,
                );
                self.token(&(binding.occurrence as u64).to_le_bytes());
                binding.policy.encode(self)?;
            }
        }
        Ok(())
    }
}

impl crate::procedure_values::contracts::CallablePolicyEncoder for Encoder<'_, '_> {
    type Error = jai_vm::Error;

    fn token(&mut self, bytes: &[u8]) {
        Encoder::token(self, bytes);
    }
    fn ty(&mut self, ty: TypeId) -> Result<(), Self::Error> {
        Encoder::ty(self, ty)
    }
    fn symbol(&mut self, symbol: jai_source::Symbol) {
        self.token(
            self.scope
                .declarations
                .graph
                .symbols()
                .name(symbol)
                .as_bytes(),
        );
    }
    fn field(&mut self, field: jai_types::FieldId) -> Result<(), Self::Error> {
        self.ty(self.types.record_type(field.record())?)?;
        self.token(&(field.index() as u64).to_le_bytes());
        Ok(())
    }
    fn constant(&mut self, value: &ConstantValue) -> Result<(), Self::Error> {
        Encoder::constant(self, value)
    }
    fn runtime_default(
        &mut self,
        read: &crate::runtime_defaults::RuntimeDefaultRead,
    ) -> Result<(), Self::Error> {
        use crate::runtime_defaults::{DefaultReadRoot, DefaultReadStep};
        match read.root() {
            DefaultReadRoot::Global { declaration, ty } => {
                self.token(b"global-default-read");
                self.declaration(declaration)?;
                self.ty(ty)?;
            }
            DefaultReadRoot::Context { ty } => {
                self.token(b"context-default-read");
                self.ty(ty)?;
            }
        }
        self.ty(read.ty())?;
        self.token(&(read.steps().len() as u64).to_le_bytes());
        for step in read.steps() {
            match *step {
                DefaultReadStep::Dereference(ty) => {
                    self.token(b"dereference");
                    self.ty(ty)?;
                }
                DefaultReadStep::Field(field) => {
                    self.token(b"field");
                    <Self as crate::procedure_values::contracts::CallablePolicyEncoder>::field(
                        self, field,
                    )?;
                }
            }
        }
        Ok(())
    }
    fn symbol_sort_key(&self, symbol: jai_source::Symbol) -> Vec<u8> {
        self.scope
            .declarations
            .graph
            .symbols()
            .name(symbol)
            .as_bytes()
            .to_vec()
    }
}

fn layout_policy_key(policy: jai_types::LayoutPolicy) -> Vec<u8> {
    use jai_types::{FloatType, IntegerType};
    let mut key = Vec::with_capacity(96);
    for layout in [
        policy.pointer(),
        policy.integer(IntegerType::U8),
        policy.integer(IntegerType::U16),
        policy.integer(IntegerType::U32),
        policy.integer(IntegerType::U64),
        policy.float(FloatType::F32),
        policy.float(FloatType::F64),
        policy.boolean(),
    ] {
        key.extend_from_slice(&layout.size.to_le_bytes());
        key.extend_from_slice(&layout.alignment.to_le_bytes());
    }
    key
}

#[cfg(test)]
mod tests {
    use super::layout_policy_key;
    use jai_types::{FloatType, IntegerType, LayoutPolicy, ScalarLayout};

    #[test]
    fn runtime_type_replay_layout_keys_preserve_pointer_width_and_scalar_alignment() {
        let base = LayoutPolicy::lp64();
        let policy = |pointer, integer64| {
            LayoutPolicy::new(
                pointer,
                [
                    base.integer(IntegerType::U8),
                    base.integer(IntegerType::U16),
                    base.integer(IntegerType::U32),
                    integer64,
                ],
                [base.float(FloatType::F32), base.float(FloatType::F64)],
                base.boolean(),
            )
            .unwrap()
        };
        assert_eq!(layout_policy_key(base).len(), 96);
        assert_eq!(
            layout_policy_key(base),
            layout_policy_key(policy(base.pointer(), base.integer(IntegerType::U64)))
        );
        assert_ne!(
            layout_policy_key(base),
            layout_policy_key(policy(
                ScalarLayout::new(4, 4),
                base.integer(IntegerType::U64)
            ))
        );
        assert_ne!(
            layout_policy_key(base),
            layout_policy_key(policy(base.pointer(), ScalarLayout::new(8, 4)))
        );
    }
}
