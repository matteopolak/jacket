//! Graph insertion receipts contain source bindings, never semantic arena handles.
use super::*;
use jai_modules::{DeclarationInsertionCode, SourceCaptureValue};
use jai_types::TypeId;

pub(super) enum SourceBinding {
    Graph(jai_modules::Binding),
    Value(SourceCaptureValue),
}

impl Resolver<'_> {
    pub(crate) fn declaration_insertion_code(
        &self,
        id: CodeValueId,
        visibility: syntax::Visibility,
        mode: syntax::InsertScope,
        span: Span,
    ) -> Result<DeclarationInsertionCode, Diagnostic> {
        let code = self.meta.codes.get(id).ok_or_else(|| {
            Diagnostic::new(span, "declaration Code belongs to another semantic context")
        })?;
        let capture = code
            .scope
            .as_ref()
            .ok_or_else(|| Diagnostic::new(span, "#code,null has no captured declarations"))?;
        let items = super::declaration_members::literal_file_items(
            &code.body,
            capture.location.source,
            visibility,
        )
        .map_err(|error| error.with_fallback_source(capture.location.source))?;
        let mut graph_bindings = Vec::new();
        let mut values = Vec::new();
        if mode == syntax::InsertScope::Captured {
            let scope = self.graph_scope.ok_or_else(|| {
                Diagnostic::at_source(
                    capture.location,
                    "declaration capture requires its original source graph",
                )
            })?;
            let effective = capture.local_scopes.insertion_capture_bindings(
                &capture.frames,
                scope,
                self.symbols,
                capture.location.span,
            )?;
            let mut ordered = effective.into_iter().collect::<Vec<_>>();
            ordered.sort_by_key(|(name, _)| self.symbols.name(*name));
            let mut nodes = 0usize;
            let mut bytes = 0usize;
            for (name, binding) in ordered {
                lexical_keys::charge(&mut nodes, 1, lexical_keys::MAX_LEXICAL_NODES, span)?;
                match self.source_insertion_binding(
                    binding,
                    capture.location,
                    &mut nodes,
                    &mut bytes,
                )? {
                    SourceBinding::Graph(binding) => graph_bindings.push((name, binding)),
                    SourceBinding::Value(value) => values.push((name, value)),
                }
            }
        }
        let policy = |mode: jai_ir::CheckMode| {
            if mode.enabled() {
                syntax::CheckPolicy::Inherited
            } else {
                syntax::CheckPolicy::Disabled
            }
        };
        Ok(DeclarationInsertionCode {
            file: capture.file,
            source_file: capture.source_file,
            location: capture.location,
            items,
            bindings: graph_bindings,
            values,
            checks: syntax::SafetyChecks {
                array_bounds: policy(capture.checks.array_bounds),
                arithmetic_overflow: policy(capture.checks.arithmetic_overflow),
            },
            debug: capture.debug,
            origins: capture.expansion_origins.clone(),
        })
    }

    pub(super) fn source_insertion_binding(
        &self,
        binding: Binding,
        location: SourceSpan,
        nodes: &mut usize,
        bytes: &mut usize,
    ) -> Result<SourceBinding, Diagnostic> {
        let scope = self.graph_scope.ok_or_else(|| {
            Diagnostic::at_source(
                location,
                "declaration capture requires its original source graph",
            )
        })?;
        self.source_insertion_binding_in_scope(binding, location, nodes, bytes, scope)
    }

    pub(super) fn source_insertion_binding_in_scope(
        &self,
        binding: Binding,
        location: SourceSpan,
        nodes: &mut usize,
        bytes: &mut usize,
        scope: crate::modules::FileScope<'_>,
    ) -> Result<SourceBinding, Diagnostic> {
        let value = match binding {
            Binding::CompilerInput {
                ..
            } => {
                return Err(Diagnostic::at_source(
                    location,
                    "compiler input cannot enter declaration insertion capture",
                ));
            }
            Binding::Namespace(module) => {
                return Ok(SourceBinding::Graph(jai_modules::Binding::Module(module)));
            }
            Binding::Imported(binding) => return Ok(SourceBinding::Graph(binding)),
            Binding::Type(ty) => SourceCaptureValue::Type(scope.insertion_capture_type(
                self.types,
                &self.meta.record_specializations,
                ty,
                location,
            )?),
            Binding::Enum(value) => {
                return self.source_insertion_enum_in_scope(value.ty, value.value, location, scope);
            }
            Binding::Procedure {
                procedure,
                ty,
            } => {
                return scope
                    .insertion_capture_procedure(procedure, ty, location)
                    .map(SourceBinding::Graph);
            }
            Binding::Library(crate::ForeignLibraryId::File(declaration)) => {
                return Ok(SourceBinding::Graph(jai_modules::Binding::Declaration(
                    declaration,
                )));
            }
            Binding::Constant(value) => SourceCaptureValue::Scalar(match value {
                crate::ScalarConstant::Literal(value) => jai_eval::Value::Literal(value),
                crate::ScalarConstant::Int(value) => jai_eval::Value::Int(value),
                crate::ScalarConstant::Bool(value) => jai_eval::Value::Bool(value),
                crate::ScalarConstant::Float(value) => jai_eval::Value::Float(value),
                crate::ScalarConstant::WeakFloat(value) => {
                    let encoding = value
                        .request_key()
                        .canonical_bytes(
                            lexical_keys::MAX_LEXICAL_NODES,
                            lexical_keys::MAX_LEXICAL_BYTES,
                        )
                        .map_err(|_| {
                            Diagnostic::at_source(
                                location,
                                "weak float declaration capture exceeds its encoding limit",
                            )
                        })?;
                    lexical_keys::charge(
                        bytes,
                        encoding.len(),
                        lexical_keys::MAX_LEXICAL_BYTES,
                        location.span,
                    )?;
                    jai_eval::Value::WeakFloat(value)
                }
            }),
            Binding::TypedConstant(id) => {
                let value = self.meta.constant(id).ok_or_else(|| {
                    Diagnostic::at_source(
                        location,
                        "captured constant belongs to another semantic context",
                    )
                })?;
                lexical_keys::charge_constant(value, nodes, bytes, location.span)?;
                return self.source_insertion_constant_in_scope(value, location, scope);
            }
            Binding::Storage(_) => {
                return Err(Diagnostic::at_source(
                    location,
                    "runtime storage cannot be transported into a file declaration capture",
                ));
            }
            Binding::Library(_) => {
                return Err(Diagnostic::at_source(
                    location,
                    "local library capture requires a retained graph library declaration",
                ));
            }
            Binding::Code(code) => {
                return scope
                    .insertion_capture_code(code, location)
                    .map(SourceBinding::Graph);
            }
            Binding::Macro(_) => {
                return Err(Diagnostic::at_source(
                    location,
                    "local macro capture requires a graph source declaration receipt",
                ));
            }
            Binding::Discarded(_) => {
                return Err(Diagnostic::at_source(
                    location,
                    "#discard bindings cannot be transported into inserted declarations",
                ));
            }
            Binding::LambdaPreview(_) => {
                return Err(Diagnostic::at_source(
                    location,
                    "pure lambda preview facts cannot be transported as declaration values",
                ));
            }
        };
        Ok(SourceBinding::Value(value))
    }

    fn source_insertion_enum_in_scope(
        &self,
        ty: TypeId,
        value: jai_types::Integer,
        location: SourceSpan,
        scope: crate::modules::FileScope<'_>,
    ) -> Result<SourceBinding, Diagnostic> {
        let jai_modules::ModuleType::Declaration(declaration) = scope.insertion_capture_type(
            self.types,
            &self.meta.record_specializations,
            ty,
            location,
        )?
        else {
            return Err(Diagnostic::at_source(
                location,
                "enum capture requires its original source enum declaration",
            ));
        };
        Ok(SourceBinding::Value(SourceCaptureValue::Enumeration(
            jai_modules::EnumParameter {
                declaration,
                value,
            },
        )))
    }

    pub(super) fn source_insertion_constant_in_scope(
        &self,
        value: &jai_ir::ConstantValue,
        location: SourceSpan,
        scope: crate::modules::FileScope<'_>,
    ) -> Result<SourceBinding, Diagnostic> {
        use jai_ir::ConstantKind as C;
        let value = match &value.kind {
            C::Int(value) => SourceCaptureValue::Scalar(jai_eval::Value::Int(*value)),
            C::Bool(value) => SourceCaptureValue::Scalar(jai_eval::Value::Bool(*value)),
            C::Float(value) => SourceCaptureValue::Scalar(jai_eval::Value::Float(*value)),
            C::StringBytes(bytes) => SourceCaptureValue::String(bytes.clone().into_boxed_slice()),
            C::Enum(integer) => {
                return self.source_insertion_enum_in_scope(value.ty, *integer, location, scope);
            }
            C::RuntimeType(value) => SourceCaptureValue::Type(scope.insertion_capture_type(
                self.types,
                &self.meta.record_specializations,
                value.identity().ty(),
                location,
            )?),
            C::Procedure(procedure) => {
                return scope
                    .insertion_capture_procedure(*procedure, value.ty, location)
                    .map(SourceBinding::Graph);
            }
            _ => {
                return Err(Diagnostic::at_source(
                    location,
                    "aggregate, zero, or runtime pointer declaration capture needs a canonical source constant transport",
                ));
            }
        };
        Ok(SourceBinding::Value(value))
    }
}
