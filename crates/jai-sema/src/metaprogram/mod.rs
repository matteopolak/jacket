//! Captured syntax values and structural insertion preserve lexical identities.
use crate::{Binding, Diagnostic, Expr, Resolver, Span, Statement, syntax};
use jai_modules::FileInstanceId;
use jai_source::{ScopeId, SourceSpan, Symbol};
use jai_types::{CodeValueId, CodeValueIds};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
mod caller_references;
mod expansion;
mod expansion_candidates;
mod expansion_scope;
mod exports;
mod formal_matching;
pub(crate) use formal_matching::ExpandedSourceError;
mod iteration;
mod iteration_protocol;
mod lexical_keys;
mod local_macros;
mod run_facts;
mod storage_keys;
pub(crate) use lexical_keys::RunLexicalKey;
pub(crate) use run_facts::{RunBindingFact, RunCaptureFacts, RunLexicalFacts, RunMacroFact};
mod loop_replacements;
pub(crate) use local_macros::{ExpandedTarget, LocalMacroId, MacroId};
mod compiler_quote_budget;
mod compiler_quote_capture;
mod compiler_quote_finish;
mod compiler_quotes;
mod declaration_capture;
pub(crate) use compiler_quote_budget::CompilerQuoteBudget;
pub(crate) use compiler_quote_capture::PublishedCompilerQuote;
pub(crate) use compiler_quote_finish::CompilerQuoteFinishError;
pub(crate) use compiler_quotes::{
    CompilerQuoteBinding, CompilerQuoteSource, CompilerQuoteTemplate,
};
mod declaration_members;
mod discarded;
pub(crate) mod record_members;
mod returns;
pub(crate) use record_members::literal_record_members;

#[derive(Clone)]
pub struct CapturedScope {
    checks: crate::safety_checks::ActiveChecks,
    debug: jai_types::DebugPolicy,
    pub file: FileInstanceId,
    pub source_file: FileInstanceId,
    pub file_scope: ScopeId,
    pub procedure: jai_ir::ProcedureId,
    pub location: SourceSpan,
    pub substitution: Option<crate::polymorphism::Substitution>,
    expansion_origins: Vec<SourceSpan>,
    frames: Vec<HashMap<Symbol, Binding>>,
    local_scopes: crate::local_declarations::LocalScopes,
}
impl std::fmt::Debug for CapturedScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapturedScope")
            .field("file", &self.file)
            .field("source_file", &self.source_file)
            .field("file_scope", &self.file_scope)
            .field("procedure", &self.procedure)
            .field("location", &self.location)
            .field("substitution", &self.substitution)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum CaptureBinding {
    Discarded(jai_types::TypeId),
    Macro(LocalMacroId),
    Namespace(jai_source::ModuleId),
    ImportedDeclaration(jai_source::DeclarationId),
    ImportedOverloadSet(jai_modules::OverloadSetId),
    ImportedModule(jai_source::ModuleId),
    ImportedParameter(jai_modules::ParameterId),
    ImportedSourceMember(jai_source::DeclarationId, Symbol),
    ImportedStorageMember(jai_modules::SourceStorageMemberId),
    Library(crate::ForeignLibraryId),
    Code(CodeValueId),
    Constant(jai_ir::ConstantValue),
    Type(jai_types::TypeId),
    Storage(storage_keys::StorageKey),
    Procedure(jai_ir::ProcedureId, jai_types::TypeId),
    Scalar(jai_types::TypeId, u64),
    WeakInteger(i128),
    WeakFloat(jai_eval::floats::WeakFloatKey),
    Enum(jai_types::TypeId, jai_types::Integer),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct CaptureKey {
    checks: crate::safety_checks::ActiveChecks,
    debug: jai_types::DebugPolicy,
    file: FileInstanceId,
    source_file: FileInstanceId,
    procedure: jai_ir::ProcedureId,
    source: jai_source::SourceId,
    start: usize,
    end: usize,
    bindings: Vec<Vec<(Symbol, CaptureBinding)>>,
    substitution: Option<crate::polymorphism::Substitution>,
    lexical_scopes: Vec<crate::local_declarations::LexicalScopeId>,
    expansion_origins: Vec<(jai_source::SourceId, usize, usize)>,
}

/// AST export is read-only. Transformations allocate a fresh code value and
/// retain its original lexical capture unless insertion explicitly rebinds it.
#[derive(Clone, Debug)]
pub struct CapturedCode {
    id: CodeValueId,
    body: syntax::CodeBody,
    scope: Option<CapturedScope>,
}
impl CapturedCode {
    pub fn id(&self) -> CodeValueId {
        self.id
    }
    pub fn body(&self) -> &syntax::CodeBody {
        &self.body
    }
    pub fn scope(&self) -> Option<&CapturedScope> {
        self.scope.as_ref()
    }
}

#[derive(Default)]
pub(crate) struct CodeRegistry {
    ids: CodeValueIds,
    values: Vec<Arc<CapturedCode>>,
    active: Vec<CodeValueId>,
    captures: HashMap<CaptureKey, CodeValueId>,
    active_macros: Vec<MacroId>,
    local_macros: local_macros::LocalMacroRegistry,
    exports: Vec<ExportFrame>,
    null: Option<CodeValueId>,
    return_regions: Vec<(jai_ir::ProcedureId, bool)>,
    replacements: Vec<loop_replacements::ReplacementFrame>,
    source_files: Vec<FileInstanceId>,
}
#[derive(Default)]
struct ExportFrame {
    remap: HashMap<Symbol, Symbol>,
    names: HashSet<Symbol>,
    bindings: HashMap<Symbol, Binding>,
    cleanup_target: Option<exports::CallerCleanupTarget>,
    return_target: Option<exports::CallerReturnTarget>,
    body_scope: Option<usize>,
    caller_scope: Option<Arc<CapturedScope>>,
    caller_key: Option<Arc<CaptureKey>>,
}
impl CodeRegistry {
    pub(crate) fn null(&mut self) -> CodeValueId {
        if let Some(id) = self.null {
            return id;
        }
        let id = self.ids.allocate();
        self.values.push(Arc::new(CapturedCode {
            id,
            body: syntax::CodeBody::Null,
            scope: None,
        }));
        self.null = Some(id);
        id
    }
    fn enter_macro(
        &mut self,
        declaration: impl Into<MacroId>,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let declaration = declaration.into();
        if self.active_macros.len() + self.active.len() >= 128 {
            return Err(Diagnostic::new(span, "macro expansion exceeds depth limit"));
        }
        if self.active_macros.contains(&declaration) {
            return Err(Diagnostic::new(span, "cyclic #expand procedure expansion"));
        }
        self.active_macros.push(declaration);
        Ok(())
    }
    fn leave_macro(&mut self, declaration: impl Into<MacroId>) {
        debug_assert_eq!(self.active_macros.pop(), Some(declaration.into()));
    }
    fn capture(
        &mut self,
        key: CaptureKey,
        body: syntax::CodeBody,
        scope: CapturedScope,
    ) -> CodeValueId {
        if let Some(&id) = self.captures.get(&key) {
            return id;
        }
        let id = self.ids.allocate();
        self.values.push(Arc::new(CapturedCode {
            id,
            body,
            scope: Some(scope),
        }));
        self.captures.insert(key, id);
        id
    }
    pub(crate) fn get(&self, id: CodeValueId) -> Option<&Arc<CapturedCode>> {
        self.ids
            .owns(id)
            .then(|| self.values.get(id.index()))
            .flatten()
    }
    fn enter(&mut self, id: CodeValueId, span: Span) -> Result<(), Diagnostic> {
        if self.active.len() + self.active_macros.len() >= 128 {
            return Err(Diagnostic::new(
                span,
                "code insertion exceeds expansion depth limit",
            ));
        }
        if self.active.contains(&id) {
            return Err(Diagnostic::new(span, "cyclic captured code insertion"));
        }
        self.active.push(id);
        Ok(())
    }
    fn leave(&mut self, id: CodeValueId) {
        debug_assert_eq!(self.active.pop(), Some(id));
    }
}

impl Resolver<'_> {
    pub(super) fn capture_source_file(
        &self,
        source: jai_source::SourceId,
        span: Span,
    ) -> Result<FileInstanceId, Diagnostic> {
        let scope = self
            .graph_scope
            .ok_or_else(|| Diagnostic::new(span, "source capture requires a defining file"))?;
        let file = self
            .meta
            .codes
            .source_files
            .last()
            .copied()
            .unwrap_or_else(|| scope.code_origin().0);
        if scope.code_file(file).source() != source {
            return Err(Diagnostic::new(
                span,
                "source capture has no retained original file instance for this quotation",
            ));
        }
        Ok(file)
    }

    pub(crate) fn capture_code(
        &mut self,
        body: &syntax::CodeBody,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        if matches!(body, syntax::CodeBody::Null) {
            return Ok(Expr::Code(self.meta.codes.null()));
        }
        let key = self.capture_key(span)?;
        let capture = self.capture_scope(span)?;
        Ok(Expr::Code(self.meta.codes.capture(
            key,
            body.clone(),
            capture,
        )))
    }

    fn capture_scope(&mut self, span: Span) -> Result<CapturedScope, Diagnostic> {
        let scope = self
            .graph_scope
            .ok_or_else(|| Diagnostic::new(span, "code capture requires a defining file scope"))?;
        let (file, file_scope, scope_source) = scope.code_origin();
        let source = self.debug.source().unwrap_or(scope_source);
        let source_file = self.capture_source_file(source, span)?;
        let substitution = scope.substitution.cloned();
        let mut frames = self.scopes.clone();
        if let Some(substitution) = &substitution {
            let mut overlay = HashMap::new();
            for binding in &substitution.types {
                overlay.insert(binding.name, Binding::Type(binding.ty));
            }
            for binding in &substitution.constants {
                use crate::polymorphism::BakedValue;
                let value = match &binding.value {
                    BakedValue::Type(ty) => Binding::Type(*ty),
                    BakedValue::Code(id) => Binding::Code(*id),
                    BakedValue::Value(value) => {
                        Binding::TypedConstant(self.meta.intern_constant(value.clone()))
                    }
                    BakedValue::Float(value) => {
                        Binding::TypedConstant(self.meta.intern_constant(jai_ir::ConstantValue {
                            ty: self.types.float(value.ty()),
                            kind: jai_ir::ConstantKind::Float(*value),
                        }))
                    }
                    BakedValue::String(bytes) => {
                        Binding::TypedConstant(self.meta.intern_constant(jai_ir::ConstantValue {
                            ty: self.types.string(),
                            kind: jai_ir::ConstantKind::StringBytes(bytes.to_vec()),
                        }))
                    }
                };
                overlay.insert(binding.name, value);
            }
            // Keep bindings and lexical declaration frames at the same depth.
            // The specialization overlay supplies missing outer names, while
            // actual lexical bindings retain their usual shadowing priority.
            let outer = frames.first_mut().ok_or_else(|| {
                Diagnostic::new(span, "code capture requires a lexical binding frame")
            })?;
            for (name, binding) in overlay {
                outer.entry(name).or_insert(binding);
            }
        }
        Ok(CapturedScope {
            checks: self.checks,
            debug: self.debug.policy(),
            file,
            source_file,
            file_scope,
            procedure: self.procedure,
            location: SourceSpan { source, span },
            frames,
            local_scopes: self.local_scopes.clone(),
            expansion_origins: self.debug.caller_origins().to_vec(),
            substitution,
        })
    }

    fn inserted_code(
        &mut self,
        directive: &syntax::InsertDirective,
    ) -> Result<(CodeValueId, Arc<CapturedCode>), Diagnostic> {
        let Expr::Code(id) = self.expr(&directive.value)? else {
            return Err(Diagnostic::new(
                directive.span,
                "#insert requires a captured Code value; generated string insertion awaits structural parser rebinding",
            ));
        };
        let code = Arc::clone(self.meta.codes.get(id).ok_or_else(|| {
            Diagnostic::new(
                directive.span,
                "code value belongs to another semantic context",
            )
        })?);
        let capture = code.scope.as_ref().ok_or_else(|| {
            Diagnostic::new(
                directive.span,
                "#insert requires captured syntax; #code,null has no body or scope",
            )
        })?;
        if capture.procedure != self.procedure && capture.frames.iter().flat_map(|frame| frame.values()).any(|binding| matches!(binding, Binding::Storage(storage) if matches!(storage.place().kind(), jai_ir::PlaceKind::Local(_)))) {
            return Err(Diagnostic::new(directive.span,"captured local storage cannot escape its defining procedure"));
        }
        self.meta.codes.enter(id, directive.span)?;
        Ok((id, code))
    }

    pub(crate) fn insert_expression(
        &mut self,
        directive: &syntax::InsertDirective,
    ) -> Result<Expr, Diagnostic> {
        self.reject_nonstatement_replacements(directive)?;
        let (id, code) = self.inserted_code(directive)?;
        let capture = code.scope.as_ref().expect("inserted code has a capture");
        self.meta.codes.source_files.push(capture.source_file);
        let original_source = self.debug.replace_source(Some(capture.location.source));
        let original_callers = self.debug.replace_caller_origins(Vec::new());
        let original_debug_policy = self.debug.enter_policy(capture.debug);
        let original_span = std::mem::replace(&mut self.span, capture.location.span);
        let original_checks = self.checks;
        if directive.scope == syntax::InsertScope::Captured {
            self.checks = capture.checks;
        }
        let original_scopes = if directive.scope == syntax::InsertScope::Captured {
            Some(std::mem::replace(&mut self.scopes, capture.frames.clone()))
        } else {
            None
        };
        let original_file = self.graph_scope;
        let original_depth = self.scopes.len();
        self.meta.codes.overlay_exports(&mut self.scopes);
        let original_locals = if directive.scope == syntax::InsertScope::Captured {
            Some(self.install_captured_local_scopes(&code))
        } else {
            Some(self.local_scopes.clone())
        };
        if directive.scope == syntax::InsertScope::Captured {
            self.graph_scope = self.graph_scope.map(|scope| scope.code_file(capture.file));
        }
        let result = match &code.body {
            syntax::CodeBody::Expression(expression) => self
                .expr(expression)
                .map_err(|error| error.with_fallback_source(capture.location.source)),
            _ => Err(Diagnostic::new(
                directive.span,
                "expression insertion requires exactly one quoted expression",
            )),
        };
        if let Some(scopes) = original_scopes {
            self.scopes = scopes;
        } else {
            self.scopes.truncate(original_depth);
        }
        self.graph_scope = original_file;
        if let Some(mut locals) = original_locals {
            locals.resume_after_expansion(&self.local_scopes);
            self.local_scopes = locals;
        }
        self.checks = original_checks;
        self.debug.replace_source(original_source);
        self.debug.replace_caller_origins(original_callers);
        self.debug.restore_policy(original_debug_policy);
        self.span = original_span;
        self.meta.codes.leave(id);
        self.meta.codes.source_files.pop();
        result
    }

    pub(crate) fn insert_statement(
        &mut self,
        directive: &syntax::InsertDirective,
    ) -> Result<Statement, Diagnostic> {
        let replacements = self.capture_loop_replacements(directive)?;
        let (id, code) = self.inserted_code(directive)?;
        self.meta.codes.replacements.push(replacements);
        let capture = code.scope.as_ref().expect("inserted code has a capture");
        self.meta.codes.source_files.push(capture.source_file);
        let original_source = self.debug.replace_source(Some(capture.location.source));
        let original_callers = self.debug.replace_caller_origins(Vec::new());
        let original_debug_policy = self.debug.enter_policy(capture.debug);
        let original_span = std::mem::replace(&mut self.span, capture.location.span);
        let original_checks = self.checks;
        if directive.scope == syntax::InsertScope::Captured {
            self.checks = capture.checks;
        }
        let original_scopes = if directive.scope == syntax::InsertScope::Captured {
            Some(std::mem::replace(&mut self.scopes, capture.frames.clone()))
        } else {
            None
        };
        let original_file = self.graph_scope;
        let original_depth = self.scopes.len();
        self.meta.codes.overlay_exports(&mut self.scopes);
        let original_locals = if directive.scope == syntax::InsertScope::Captured {
            Some(self.install_captured_local_scopes(&code))
        } else {
            Some(self.local_scopes.clone())
        };
        if directive.scope == syntax::InsertScope::Captured {
            self.graph_scope = self.graph_scope.map(|scope| scope.code_file(capture.file));
        }
        self.meta.codes.return_regions.push((self.procedure, true));
        let result = match &code.body {
            syntax::CodeBody::Null => unreachable!("null code rejected before insertion"),
            syntax::CodeBody::Block(statements) => self.block(statements, true).map(|block| {
                self.debug
                    .attach_block(&[jai_ir::DebugPathStep::Child(jai_ir::DebugBranch::Block)]);
                Statement::Block(block)
            }),
            syntax::CodeBody::Statement(statement) => self
                .block(std::slice::from_ref(statement), true)
                .map(|block| {
                    self.debug
                        .attach_block(&[jai_ir::DebugPathStep::Child(jai_ir::DebugBranch::Block)]);
                    Statement::Block(block)
                }),
            syntax::CodeBody::Expression(expression) => self
                .statement(&syntax::Statement::new(
                    expression.span,
                    syntax::StatementKind::Expression((**expression).clone()),
                ))
                .inspect(|_| {
                    self.debug.forward_statement();
                }),
        };
        let result = result.map_err(|error| error.with_fallback_source(capture.location.source));
        self.meta.codes.return_regions.pop();
        if let Some(scopes) = original_scopes {
            self.scopes = scopes;
        } else {
            self.scopes.truncate(original_depth);
        }
        self.graph_scope = original_file;
        if let Some(mut locals) = original_locals {
            locals.resume_after_expansion(&self.local_scopes);
            self.local_scopes = locals;
        }
        self.checks = original_checks;
        self.debug.replace_source(original_source);
        self.debug.replace_caller_origins(original_callers);
        self.debug.restore_policy(original_debug_policy);
        self.span = original_span;
        self.meta.codes.leave(id);
        self.meta.codes.source_files.pop();
        self.meta
            .codes
            .replacements
            .pop()
            .expect("active insertion replacement scope");
        result
    }

    pub(crate) fn insert_place(
        &mut self,
        directive: &syntax::InsertDirective,
    ) -> Result<jai_ir::Place, Diagnostic> {
        self.reject_nonstatement_replacements(directive)?;
        let (id, code) = self.inserted_code(directive)?;
        let capture = code.scope.as_ref().expect("inserted code has a capture");
        self.meta.codes.source_files.push(capture.source_file);
        let original_source = self.debug.replace_source(Some(capture.location.source));
        let original_callers = self.debug.replace_caller_origins(Vec::new());
        let original_debug_policy = self.debug.enter_policy(capture.debug);
        let original_span = std::mem::replace(&mut self.span, capture.location.span);
        let original_checks = self.checks;
        if directive.scope == syntax::InsertScope::Captured {
            self.checks = capture.checks;
        }
        let original_scopes = if directive.scope == syntax::InsertScope::Captured {
            Some(std::mem::replace(&mut self.scopes, capture.frames.clone()))
        } else {
            None
        };
        let original_locals = if directive.scope == syntax::InsertScope::Captured {
            Some(self.install_captured_local_scopes(&code))
        } else {
            Some(self.local_scopes.clone())
        };
        let original_file = self.graph_scope;
        let original_depth = self.scopes.len();
        self.meta.codes.overlay_exports(&mut self.scopes);
        if directive.scope == syntax::InsertScope::Captured {
            self.graph_scope = self.graph_scope.map(|scope| scope.code_file(capture.file));
        }
        let result = match &code.body {
            syntax::CodeBody::Expression(expression) => {
                syntax::PlaceSyntax::try_from((**expression).clone())
                    .and_then(|place| self.resolve_place(&place))
                    .map_err(|error| error.with_fallback_source(capture.location.source))
            }
            _ => Err(Diagnostic::new(
                directive.span,
                "assignment insertion requires one quoted storage expression",
            )),
        };
        if let Some(scopes) = original_scopes {
            self.scopes = scopes;
        } else {
            self.scopes.truncate(original_depth);
        }
        if let Some(mut locals) = original_locals {
            locals.resume_after_expansion(&self.local_scopes);
            self.local_scopes = locals;
        }
        self.graph_scope = original_file;
        self.checks = original_checks;
        self.debug.replace_source(original_source);
        self.debug.replace_caller_origins(original_callers);
        self.debug.restore_policy(original_debug_policy);
        self.span = original_span;
        self.meta.codes.leave(id);
        self.meta.codes.source_files.pop();
        result
    }
    fn install_captured_local_scopes(
        &mut self,
        code: &CapturedCode,
    ) -> crate::local_declarations::LocalScopes {
        let mut locals = code
            .scope
            .as_ref()
            .expect("inserted code has a capture")
            .local_scopes
            .clone();
        locals.resume_after_expansion(&self.local_scopes);
        std::mem::replace(&mut self.local_scopes, locals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_code_has_one_owned_identity_and_no_fabricated_capture() {
        let mut registry = CodeRegistry::default();
        let id = registry.null();
        assert_eq!(registry.null(), id);
        let value = registry.get(id).unwrap();
        assert!(matches!(value.body(), syntax::CodeBody::Null));
        assert!(value.scope().is_none());
        assert!(CodeRegistry::default().get(id).is_none());
    }
}
