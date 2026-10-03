//! Lexical macro definitions retain their actual declaration and capture identities.
use super::*;
use crate::local_declarations::LocalDeclarationId;
use jai_types::{Integer, TypeId};

/// A registry-owned handle cannot be substituted for a graph declaration ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct LocalMacroId(CodeValueId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum MacroId {
    Module(jai_source::DeclarationId),
    Local(LocalMacroId),
}
impl From<jai_source::DeclarationId> for MacroId {
    fn from(value: jai_source::DeclarationId) -> Self {
        Self::Module(value)
    }
}

#[derive(Clone)]
pub(crate) struct ExpandedTarget {
    pub(crate) id: MacroId,
    pub(crate) file: FileInstanceId,
    pub(crate) procedure: Arc<syntax::Procedure>,
    pub(super) capture: Option<Arc<CapturedScope>>,
}
impl ExpandedTarget {
    pub(crate) fn captured_source(&self) -> Option<jai_source::SourceId> {
        self.capture.as_ref().map(|capture| capture.location.source)
    }

    pub(super) fn module(
        (id, file, procedure): (jai_source::DeclarationId, FileInstanceId, syntax::Procedure),
    ) -> Self {
        Self {
            id: MacroId::Module(id),
            file,
            procedure: Arc::new(procedure),
            capture: None,
        }
    }
}

#[derive(Default)]
pub(super) struct LocalMacroRegistry {
    ids: CodeValueIds,
    definitions: HashMap<LocalDeclarationId, LocalMacroId>,
    values: Vec<ExpandedTarget>,
}
impl LocalMacroRegistry {
    pub(super) fn get(&self, id: LocalMacroId) -> Option<&ExpandedTarget> {
        self.ids
            .owns(id.0)
            .then(|| self.values.get(id.0.index()))
            .flatten()
    }

    fn define(
        &mut self,
        declaration: LocalDeclarationId,
        procedure: &syntax::Procedure,
        mut capture: CapturedScope,
    ) -> LocalMacroId {
        if let Some(&id) = self.definitions.get(&declaration) {
            return id;
        }
        let id = LocalMacroId(self.ids.allocate());
        // Self lookup uses this semantic macro binding; recursion never needs a
        // runtime procedure allocation or a fabricated module declaration.
        capture
            .frames
            .last_mut()
            .expect("a lexical declaration has a frame")
            .insert(procedure.name, Binding::Macro(id));
        self.values.push(ExpandedTarget {
            id: MacroId::Local(id),
            file: capture.file,
            procedure: Arc::new(procedure.clone()),
            capture: Some(Arc::new(capture)),
        });
        self.definitions.insert(declaration, id);
        id
    }
}

impl Resolver<'_> {
    pub(crate) fn define_local_macro(
        &mut self,
        id: LocalDeclarationId,
        procedure: &syntax::Procedure,
    ) -> Result<Binding, Diagnostic> {
        if procedure.modify.is_some() || procedure.compiler.is_some() {
            return Err(Diagnostic::new(
                procedure.span,
                "local #expand procedures cannot combine compiler or #modify execution",
            ));
        }
        let capture = self.capture_scope(procedure.span)?;
        Ok(Binding::Macro(
            self.meta.codes.local_macros.define(id, procedure, capture),
        ))
    }

    pub(super) fn local_expanded_target(
        &self,
        id: LocalMacroId,
        span: Span,
    ) -> Result<ExpandedTarget, Diagnostic> {
        self.meta
            .codes
            .local_macros
            .get(id)
            .cloned()
            .ok_or_else(|| Diagnostic::new(span, "local macro belongs to another semantic context"))
    }

    pub(crate) fn expanded_annotation(
        &mut self,
        target: &ExpandedTarget,
        syntax: &syntax::TypeSyntax,
        span: Span,
    ) -> Result<TypeId, Diagnostic> {
        let Some(capture) = &target.capture else {
            let definition = self
                .graph_scope
                .expect("expanded source has a module graph")
                .code_file(target.file);
            return definition
                .annotation(syntax, self.types, span)
                .map_err(|error| error.with_fallback_source(definition.source()));
        };
        let original_file = self.graph_scope;
        let original_frames = std::mem::replace(&mut self.scopes, capture.frames.clone());
        let mut locals = capture.local_scopes.clone();
        locals.resume_after_expansion(&self.local_scopes);
        let mut original_locals = std::mem::replace(&mut self.local_scopes, locals);
        self.graph_scope = original_file.map(|scope| scope.code_file(capture.file));
        let original_source = self.debug.replace_source(Some(capture.location.source));
        self.meta.codes.source_files.push(capture.source_file);
        let result = self
            .lexical_annotation(syntax, span)
            .map_err(|error| error.with_fallback_source(capture.location.source));
        self.debug.replace_source(original_source);
        self.meta.codes.source_files.pop();
        self.graph_scope = original_file;
        self.scopes = original_frames;
        original_locals.resume_after_expansion(&self.local_scopes);
        self.local_scopes = original_locals;
        result
    }

    pub(crate) fn expanded_enum_flags(&self, target: &ExpandedTarget, ty: TypeId) -> bool {
        self.meta
            .local_declarations
            .enum_is_flags(ty)
            .unwrap_or_else(|| {
                self.graph_scope
                    .is_some_and(|scope| scope.code_file(target.file).enum_flags(ty))
            })
    }

    pub(crate) fn expanded_enum_member(
        &self,
        target: &ExpandedTarget,
        ty: TypeId,
        name: Symbol,
    ) -> Option<Integer> {
        self.meta
            .local_declarations
            .enum_member_value(ty, name)
            .or_else(|| {
                self.graph_scope
                    .and_then(|scope| scope.code_file(target.file).enum_member_value(ty, name))
            })
    }
}
