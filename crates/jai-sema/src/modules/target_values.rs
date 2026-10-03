//! Source-visible target values retain the designated source enum identity.
use super::*;

pub(super) fn is_target_path(graph: &ModuleGraph, file: FileInstanceId, path: &NamePath) -> bool {
    path.members.is_empty()
        && graph.target().is_some()
        && graph.lookup(file, path).is_err()
        && matches!(
            graph.symbols().name(path.root),
            "OS" | "BUILD_OS" | "CPU" | "BUILD_CPU"
        )
}

impl FileScope<'_> {
    pub(crate) fn selected_case(
        &self,
        span: Span,
        specialization: Option<&jai_modules::SourceSpecializationKey>,
    ) -> Option<syntax::CompileTimeCaseChoice> {
        self.declarations
            .graph
            .source_cases()
            .iter()
            .find(|selected| {
                selected.file == self.file
                    && selected.location.span == span
                    && selected.origin == jai_modules::SourceConditionOrigin::Semantic
                    && selected.specialization.as_ref() == specialization
            })
            .map(|selected| selected.choice)
    }

    pub(crate) fn selected_condition(
        &self,
        span: Span,
        specialization: Option<&jai_modules::SourceSpecializationKey>,
    ) -> Option<bool> {
        self.declarations
            .graph
            .selected_semantic_condition_for(self.file, span, specialization)
    }

    pub(crate) fn target_value(
        &self,
        path: &NamePath,
        span: Span,
    ) -> Result<Option<Binding>, Diagnostic> {
        let graph = self.declarations.graph;
        if !path.members.is_empty() || graph.lookup(self.file, path).is_ok() {
            return Ok(None);
        }
        let enum_name = match graph.symbols().name(path.root) {
            "OS" | "BUILD_OS" => "Operating_System_Tag",
            "CPU" | "BUILD_CPU" => "CPU_Tag",
            _ => return Ok(None),
        };
        let target = graph.target().ok_or_else(|| {
            Diagnostic::new(
                span,
                "source target constants require an explicitly selected build target",
            )
        })?;
        let tag = if enum_name == "Operating_System_Tag" {
            target.operating_system.source_tag()
        } else {
            target.architecture.source_tag()
        }
        .ok_or_else(|| {
            Diagnostic::new(span, "selected target has no source-defined OS or CPU tag")
        })?;
        let name = graph
            .symbols()
            .find(enum_name)
            .ok_or_else(|| Diagnostic::new(span, "target tag enum must be supplied by source"))?;
        // Preload owns the compiler tag schema. Isolated frontend consumers may
        // supply the enum in their defining scope when bootstrap is disabled.
        let file = graph
            .prelude()
            .map(|module| graph.module(module).unwrap().entry())
            .unwrap_or(self.file);
        let jai_modules::Binding::Declaration(id) =
            graph.lookup(file, &super::path(name)).map_err(|_| {
                Diagnostic::new(
                    span,
                    "target tag enum is absent from the designated source scope",
                )
            })?
        else {
            return Err(Diagnostic::new(
                span,
                "target tag schema must denote a source enum declaration",
            ));
        };
        let ty = self
            .declarations
            .nominals
            .declarations
            .get(&id)
            .copied()
            .ok_or_else(|| {
                Diagnostic::new(span, "target tag enum has no resolved nominal identity")
            })?;
        let enumeration = self
            .declarations
            .nominals
            .enums
            .get(&ty)
            .ok_or_else(|| Diagnostic::new(span, "target tag schema is not an enum"))?;
        let tag = graph.symbols().find(tag).ok_or_else(|| {
            Diagnostic::new(span, "selected target tag is absent from the source enum")
        })?;
        let value = enumeration.members.get(&tag).copied().ok_or_else(|| {
            Diagnostic::new(span, "selected target tag is absent from the source enum")
        })?;
        Ok(Some(Binding::Enum(aggregates::EnumConstant {
            ty,
            value,
        })))
    }
}
