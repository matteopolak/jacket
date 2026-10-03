//! Stable replay receives original source facts rather than session identities.
use super::*;
use jai_types::TypeId;

const MAX_CAPTURE_FACTS: usize = 4096;

#[derive(Clone, Debug)]
pub(crate) struct RunLexicalFacts {
    pub(crate) current: RunCaptureFacts,
    pub(crate) captures: Vec<RunCaptureFacts>,
    pub(crate) active_codes: Vec<usize>,
    pub(crate) active_macros: Vec<RunMacroFact>,
    pub(crate) active_callers: Vec<usize>,
    codes: HashMap<CodeValueId, RunBindingFact>,
}
impl RunLexicalFacts {
    /// A handle selects checked metadata; its arena identity is never encoded.
    pub(crate) fn capture_for_code(&self, id: CodeValueId) -> Option<&RunBindingFact> {
        self.codes.get(&id)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RunCaptureFacts {
    pub(crate) file: FileInstanceId,
    pub(crate) source_file: FileInstanceId,
    pub(crate) location: SourceSpan,
    pub(crate) substitution: Option<crate::polymorphism::Substitution>,
    pub(crate) bindings: Vec<Vec<(Symbol, RunBindingFact)>>,
    pub(crate) creation_origins: Vec<SourceSpan>,
    pub(crate) checks: crate::safety_checks::ActiveChecks,
    pub(crate) debug: jai_types::DebugPolicy,
}

#[derive(Clone, Debug)]
pub(crate) enum RunBindingFact {
    Discarded(TypeId),
    Type(TypeId),
    Constant(jai_ir::ConstantValue),
    WeakInteger(i128),
    WeakFloat(jai_eval::floats::WeakFloatKey),
    Code(usize),
    NullCode,
    Macro(RunMacroFact),
    Procedure {
        procedure: jai_ir::ProcedureId,
        ty: TypeId,
    },
    Namespace(jai_source::ModuleId),
    Imported(jai_modules::Binding),
    Library(crate::ForeignLibraryId),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum RunMacroFact {
    Module(jai_source::DeclarationId),
    /// Index into the same `captures` pool used by Code.
    Local(usize),
}

struct FactsBuilder<'a, 'b> {
    resolver: &'a Resolver<'b>,
    codes: HashMap<CodeValueId, RunBindingFact>,
    macros: HashMap<LocalMacroId, usize>,
    pending: Vec<&'a CapturedScope>,
    nodes: usize,
    bytes: usize,
    span: Span,
}

impl Resolver<'_> {
    pub(crate) fn run_lexical_facts(&self, span: Span) -> Result<RunLexicalFacts, Diagnostic> {
        let scope = self.graph_scope.ok_or_else(|| {
            Diagnostic::new(
                span,
                "run lexical facts require a defining source environment",
            )
        })?;
        let (file, _, source) = scope.code_origin();
        let mut builder = FactsBuilder {
            resolver: self,
            codes: HashMap::new(),
            macros: HashMap::new(),
            pending: Vec::new(),
            nodes: 0,
            bytes: 0,
            span,
        };
        let substitution = scope.substitution.cloned();
        builder.substitution(substitution.as_ref())?;
        let current = RunCaptureFacts {
            file,
            source_file: self.capture_source_file(self.debug.source().unwrap_or(source), span)?,
            location: SourceSpan {
                source: self.debug.source().unwrap_or(source),
                span,
            },
            substitution,
            bindings: builder.bindings(&self.scopes)?,
            creation_origins: self.debug.caller_origins().to_vec(),
            checks: self.checks,
            debug: self.debug.policy(),
        };
        let mut active_codes = Vec::new();
        for &id in &self.meta.codes.active {
            let RunBindingFact::Code(index) = builder.code(id)? else {
                return Err(Diagnostic::new(
                    span,
                    "active code insertion has no capture",
                ));
            };
            active_codes.push(index);
        }
        let mut active_macros = Vec::new();
        for &id in &self.meta.codes.active_macros {
            active_macros.push(builder.macro_fact(id)?);
        }
        let mut active_callers = Vec::new();
        for frame in &self.meta.codes.exports {
            if let Some(capture) = frame.caller_scope.as_deref() {
                active_callers.push(builder.reserve(capture)?);
            }
        }
        let mut captures = Vec::new();
        // Reserve a pool index before examining self bindings. Iterative
        // discovery preserves aliases and cycles without recursive copies.
        while let Some(&capture) = builder.pending.get(captures.len()) {
            builder.substitution(capture.substitution.as_ref())?;
            captures.push(RunCaptureFacts {
                file: capture.file,
                source_file: capture.source_file,
                location: capture.location,
                substitution: capture.substitution.clone(),
                bindings: builder.bindings(&capture.frames)?,
                creation_origins: capture.expansion_origins.clone(),
                checks: capture.checks,
                debug: capture.debug,
            });
        }
        Ok(RunLexicalFacts {
            current,
            captures,
            active_codes,
            active_macros,
            active_callers,
            codes: builder.codes,
        })
    }
}

impl<'a> FactsBuilder<'a, '_> {
    fn charge(&mut self, count: usize) -> Result<(), Diagnostic> {
        lexical_keys::charge(
            &mut self.nodes,
            count,
            lexical_keys::MAX_LEXICAL_NODES,
            self.span,
        )
    }
    fn constant(
        &mut self,
        value: &jai_ir::ConstantValue,
    ) -> Result<jai_ir::ConstantValue, Diagnostic> {
        lexical_keys::charge_constant(value, &mut self.nodes, &mut self.bytes, self.span)?;
        Ok(value.clone())
    }
    fn reserve(&mut self, capture: &'a CapturedScope) -> Result<usize, Diagnostic> {
        if self.pending.len() >= MAX_CAPTURE_FACTS {
            return Err(Diagnostic::new(
                self.span,
                "run lexical facts exceed their capture limit",
            ));
        }
        self.charge(1)?;
        let index = self.pending.len();
        self.pending.push(capture);
        Ok(index)
    }
    fn code(&mut self, id: CodeValueId) -> Result<RunBindingFact, Diagnostic> {
        if let Some(fact) = self.codes.get(&id) {
            return Ok(fact.clone());
        }
        let resolver = self.resolver;
        let code = resolver.meta.codes.get(id).ok_or_else(|| {
            Diagnostic::new(
                self.span,
                "run code fact belongs to another semantic context",
            )
        })?;
        let fact = match &code.scope {
            Some(capture) => RunBindingFact::Code(self.reserve(capture)?),
            None if matches!(code.body, syntax::CodeBody::Null) => RunBindingFact::NullCode,
            None => {
                return Err(Diagnostic::new(
                    self.span,
                    "run code fact has no source capture",
                ));
            }
        };
        self.codes.insert(id, fact.clone());
        Ok(fact)
    }
    fn macro_fact(&mut self, id: MacroId) -> Result<RunMacroFact, Diagnostic> {
        match id {
            MacroId::Module(id) => Ok(RunMacroFact::Module(id)),
            MacroId::Local(id) => {
                if let Some(&index) = self.macros.get(&id) {
                    return Ok(RunMacroFact::Local(index));
                }
                let resolver = self.resolver;
                let target = resolver.meta.codes.local_macros.get(id).ok_or_else(|| {
                    Diagnostic::new(
                        self.span,
                        "run macro fact belongs to another semantic context",
                    )
                })?;
                let capture = target.capture.as_deref().ok_or_else(|| {
                    Diagnostic::new(self.span, "local run macro fact has no source capture")
                })?;
                let index = self.reserve(capture)?;
                self.macros.insert(id, index);
                Ok(RunMacroFact::Local(index))
            }
        }
    }
    fn substitution(
        &mut self,
        substitution: Option<&crate::polymorphism::Substitution>,
    ) -> Result<(), Diagnostic> {
        let Some(substitution) = substitution else {
            return Ok(());
        };
        self.charge(
            substitution.types.len() + substitution.constants.len() + substitution.callables.len(),
        )?;
        for binding in &substitution.constants {
            match &binding.value {
                crate::polymorphism::BakedValue::Code(id) => {
                    self.code(*id)?;
                }
                crate::polymorphism::BakedValue::Value(value) => {
                    lexical_keys::charge_constant(
                        value,
                        &mut self.nodes,
                        &mut self.bytes,
                        self.span,
                    )?;
                }
                crate::polymorphism::BakedValue::String(bytes) => lexical_keys::charge(
                    &mut self.bytes,
                    bytes.len(),
                    lexical_keys::MAX_LEXICAL_BYTES,
                    self.span,
                )?,
                _ => {}
            }
        }
        Ok(())
    }
    fn bindings(
        &mut self,
        frames: &[HashMap<Symbol, Binding>],
    ) -> Result<Vec<Vec<(Symbol, RunBindingFact)>>, Diagnostic> {
        self.charge(frames.len())?;
        let mut result = Vec::new();
        for frame in frames {
            self.charge(frame.len())?;
            let mut frame: Vec<_> = frame.iter().collect();
            frame.sort_by_key(|(name, _)| self.resolver.symbols.name(**name));
            let mut facts = Vec::new();
            for (&name, binding) in frame {
                let fact = match binding {
                    Binding::CompilerInput {
                        ..
                    } => {
                        return Err(Diagnostic::new(
                            self.span,
                            "compiler input cannot enter a native #run recipe",
                        ));
                    }
                    Binding::Storage(_) => continue,
                    Binding::LambdaPreview(_) => {
                        return Err(Diagnostic::new(
                            self.span,
                            "lambda candidate facts cannot enter run replay metadata",
                        ));
                    }
                    Binding::Discarded(ty) => RunBindingFact::Discarded(*ty),
                    Binding::Type(ty) => RunBindingFact::Type(*ty),
                    Binding::TypedConstant(id) => {
                        let value = self.resolver.meta.constant(*id).ok_or_else(|| {
                            Diagnostic::new(
                                self.span,
                                "run constant fact belongs to another context",
                            )
                        })?;
                        RunBindingFact::Constant(self.constant(value)?)
                    }
                    Binding::Code(id) => self.code(*id)?,
                    Binding::Macro(id) => {
                        RunBindingFact::Macro(self.macro_fact(MacroId::Local(*id))?)
                    }
                    Binding::Namespace(module) => RunBindingFact::Namespace(*module),
                    Binding::Imported(binding) => RunBindingFact::Imported(*binding),
                    Binding::Library(id) => RunBindingFact::Library(*id),
                    Binding::Procedure {
                        procedure,
                        ty,
                    } => RunBindingFact::Procedure {
                        procedure: *procedure,
                        ty: *ty,
                    },
                    Binding::Constant(crate::ScalarConstant::Literal(value)) => {
                        RunBindingFact::WeakInteger(*value)
                    }
                    Binding::Constant(crate::ScalarConstant::WeakFloat(value)) => {
                        RunBindingFact::WeakFloat(value.request_key().clone())
                    }
                    Binding::Constant(crate::ScalarConstant::Int(value)) => {
                        RunBindingFact::Constant(jai_ir::ConstantValue {
                            ty: self
                                .resolver
                                .types
                                .scalar(crate::ScalarType::Int(value.ty())),
                            kind: jai_ir::ConstantKind::Int(*value),
                        })
                    }
                    Binding::Constant(crate::ScalarConstant::Bool(value)) => {
                        RunBindingFact::Constant(jai_ir::ConstantValue {
                            ty: self.resolver.types.scalar(crate::ScalarType::Bool),
                            kind: jai_ir::ConstantKind::Bool(*value),
                        })
                    }
                    Binding::Constant(crate::ScalarConstant::Float(value)) => {
                        RunBindingFact::Constant(jai_ir::ConstantValue {
                            ty: self.resolver.types.float(value.ty()),
                            kind: jai_ir::ConstantKind::Float(*value),
                        })
                    }
                    Binding::Enum(value) => RunBindingFact::Constant(jai_ir::ConstantValue {
                        ty: value.ty,
                        kind: jai_ir::ConstantKind::Enum(value.value),
                    }),
                };
                facts.push((name, fact));
            }
            result.push(facts);
        }
        Ok(result)
    }
}
