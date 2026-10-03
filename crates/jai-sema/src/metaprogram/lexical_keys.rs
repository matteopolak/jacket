//! Local run-cache keys retain checked lexical facts and active expansion identity.
use super::*;

const MAX_LEXICAL_BINDINGS: usize = 65_536;
pub(super) const MAX_LEXICAL_NODES: usize = 1_048_576;
pub(super) const MAX_LEXICAL_BYTES: usize = 1_048_576;

/// These arena identities are confined to one semantic session. Stable effect
/// receipts use source provenance and checked values, never this cache key.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RunLexicalKey {
    capture: CaptureKey,
    active_codes: Vec<CodeValueId>,
    active_macros: Vec<MacroId>,
    callers: Vec<Arc<CaptureKey>>,
}

impl Resolver<'_> {
    pub(crate) fn run_lexical_key(&self, span: Span) -> Result<RunLexicalKey, Diagnostic> {
        Ok(RunLexicalKey {
            capture: self.capture_key(span)?,
            active_codes: self.meta.codes.active.clone(),
            active_macros: self.meta.codes.active_macros.clone(),
            callers: self
                .meta
                .codes
                .exports
                .iter()
                .filter_map(|frame| frame.caller_key.clone())
                .collect(),
        })
    }

    pub(super) fn capture_key(&self, span: Span) -> Result<CaptureKey, Diagnostic> {
        let scope = self
            .graph_scope
            .ok_or_else(|| Diagnostic::new(span, "code capture requires a defining file scope"))?;
        let (file, _, scope_source) = scope.code_origin();
        let source = self.debug.source().unwrap_or(scope_source);
        let substitution = scope.substitution.cloned();
        if self.scopes.iter().map(HashMap::len).sum::<usize>() > MAX_LEXICAL_BINDINGS {
            return Err(Diagnostic::new(
                span,
                "lexical capture exceeds its binding limit",
            ));
        }
        let mut nodes = self.scopes.len();
        let mut bytes = 0usize;
        let mut bindings = Vec::with_capacity(self.scopes.len());
        for frame in &self.scopes {
            let mut values = Vec::with_capacity(frame.len());
            for (&name, binding) in frame {
                let binding = match binding.clone() {
                    Binding::CompilerInput {
                        ..
                    } => {
                        return Err(Diagnostic::new(
                            span,
                            "compiler input cannot enter a retained source capture",
                        ));
                    }
                    Binding::Discarded(ty) => CaptureBinding::Discarded(ty),
                    Binding::LambdaPreview(_) => {
                        return Err(Diagnostic::new(
                            span,
                            "lambda candidate parameters cannot be captured as Code",
                        ));
                    }
                    Binding::Macro(id) => CaptureBinding::Macro(id),
                    Binding::Namespace(module) => CaptureBinding::Namespace(module),
                    Binding::Imported(binding) => match binding {
                        jai_modules::Binding::Declaration(id) => {
                            CaptureBinding::ImportedDeclaration(id)
                        }
                        jai_modules::Binding::OverloadSet(id) => {
                            CaptureBinding::ImportedOverloadSet(id)
                        }
                        jai_modules::Binding::Module(id) => CaptureBinding::ImportedModule(id),
                        jai_modules::Binding::Parameter(id) => {
                            CaptureBinding::ImportedParameter(id)
                        }
                        jai_modules::Binding::SourceMember {
                            declaration,
                            member,
                        } => CaptureBinding::ImportedSourceMember(declaration, member),
                        jai_modules::Binding::StorageMember(id) => {
                            CaptureBinding::ImportedStorageMember(id)
                        }
                    },
                    Binding::Library(id) => CaptureBinding::Library(id),
                    Binding::Code(id) => CaptureBinding::Code(id),
                    Binding::TypedConstant(id) => {
                        let value = self.meta.constant(id).ok_or_else(|| {
                            Diagnostic::new(span, "captured constant belongs to another context")
                        })?;
                        charge_constant(value, &mut nodes, &mut bytes, span)?;
                        CaptureBinding::Constant(value.clone())
                    }
                    Binding::Type(ty) => CaptureBinding::Type(ty),
                    Binding::Storage(storage) => {
                        let key =
                            storage_keys::StorageKey::new(self.places, storage.place(), span)?;
                        charge(&mut nodes, key.nodes(), MAX_LEXICAL_NODES, span)?;
                        charge(&mut bytes, key.bytes(), MAX_LEXICAL_BYTES, span)?;
                        CaptureBinding::Storage(key)
                    }
                    Binding::Procedure {
                        procedure,
                        ty,
                    } => CaptureBinding::Procedure(procedure, ty),
                    Binding::Constant(crate::ScalarConstant::Literal(value)) => {
                        CaptureBinding::WeakInteger(value)
                    }
                    Binding::Constant(crate::ScalarConstant::Int(value)) => CaptureBinding::Scalar(
                        self.types.scalar(crate::ScalarType::Int(value.ty())),
                        value.bits(),
                    ),
                    Binding::Constant(crate::ScalarConstant::Bool(value)) => {
                        CaptureBinding::Scalar(
                            self.types.scalar(crate::ScalarType::Bool),
                            u64::from(value),
                        )
                    }
                    Binding::Constant(crate::ScalarConstant::Float(value)) => {
                        CaptureBinding::Constant(jai_ir::ConstantValue {
                            ty: self.types.float(value.ty()),
                            kind: jai_ir::ConstantKind::Float(value),
                        })
                    }
                    Binding::Constant(crate::ScalarConstant::WeakFloat(value)) => {
                        CaptureBinding::WeakFloat(value.request_key().clone())
                    }
                    Binding::Enum(value) => CaptureBinding::Enum(value.ty, value.value),
                };
                charge(&mut nodes, 1, MAX_LEXICAL_NODES, span)?;
                values.push((name, binding));
            }
            values.sort_by_key(|(name, _)| self.symbols.name(*name));
            bindings.push(values);
        }
        Ok(CaptureKey {
            checks: self.checks,
            debug: self.debug.policy(),
            file,
            source_file: self.capture_source_file(source, span)?,
            procedure: self.procedure,
            source,
            start: span.start,
            end: span.end,
            bindings,
            substitution: substitution.clone(),
            lexical_scopes: self.local_scopes.capture_identity(),
            expansion_origins: self
                .debug
                .caller_origins()
                .iter()
                .map(|origin| (origin.source, origin.span.start, origin.span.end))
                .collect(),
        })
    }
}

pub(super) fn charge(
    used: &mut usize,
    count: usize,
    limit: usize,
    span: Span,
) -> Result<(), Diagnostic> {
    let next = used
        .checked_add(count)
        .filter(|next| *next <= limit)
        .ok_or_else(|| Diagnostic::new(span, "lexical capture exceeds its value budget"))?;
    *used = next;
    Ok(())
}

pub(super) fn charge_constant(
    value: &jai_ir::ConstantValue,
    nodes: &mut usize,
    bytes: &mut usize,
    span: Span,
) -> Result<(), Diagnostic> {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        charge(nodes, 1, MAX_LEXICAL_NODES, span)?;
        match &value.kind {
            jai_ir::ConstantKind::Record(values) | jai_ir::ConstantKind::Array(values) => {
                if pending.len().saturating_add(values.len()) > MAX_LEXICAL_NODES {
                    return Err(Diagnostic::new(
                        span,
                        "lexical capture exceeds its value budget",
                    ));
                }
                pending.extend(values);
            }
            jai_ir::ConstantKind::Distinct(value)
            | jai_ir::ConstantKind::Union {
                value, ..
            } => pending.push(value),
            jai_ir::ConstantKind::StringBytes(value) => {
                charge(bytes, value.len(), MAX_LEXICAL_BYTES, span)?
            }
            _ => {}
        }
    }
    Ok(())
}
