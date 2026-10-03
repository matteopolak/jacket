//! Immutable original source case requests and specialization-scoped responses.
use super::*;
use jai_syntax::{CompileTimeCaseChoice, CompileTimeCaseHeader};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CaseRequestId {
    unit: UnitId,
    identity: u64,
    index: usize,
}
impl CaseRequestId {
    pub fn index(self) -> usize {
        self.index
    }
}
fn next_case_identity() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
        value.checked_add(1)
    })
    .expect("source case request identity space exhausted")
}
#[derive(Clone, Debug)]
pub struct DeferredCase {
    pub id: CaseRequestId,
    pub file: FileInstanceId,
    pub module: ModuleId,
    pub location: SourceSpan,
    pub header: CompileTimeCaseHeader,
    pub context: DiscoveryConditionContext,
    pub specialization: Option<SourceSpecializationKey>,
    pub selected: Option<CompileTimeCaseChoice>,
}
#[derive(Clone, Debug)]
pub struct SourceCaseSelection {
    pub file: FileInstanceId,
    pub location: SourceSpan,
    pub specialization: Option<SourceSpecializationKey>,
    pub choice: CompileTimeCaseChoice,
    pub origin: SourceConditionOrigin,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaseSelectionError {
    UnknownRequest,
    InvalidChoice,
    AlreadySelected,
}
impl fmt::Display for CaseSelectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CaseSelectionError {}

impl GraphDiscovery<'_> {
    pub fn cases(&self) -> &[DeferredCase] {
        &self.builder.cases
    }
    pub fn pending_cases(&self) -> impl Iterator<Item = &DeferredCase> {
        self.builder
            .cases
            .iter()
            .filter(|case| case.selected.is_none())
    }
    pub fn select_case(
        &mut self,
        id: CaseRequestId,
        choice: CompileTimeCaseChoice,
    ) -> Result<(), CaseSelectionError> {
        self.invalidate_insertion_admissions();
        if id.unit != self.builder.graph.unit {
            return Err(CaseSelectionError::UnknownRequest);
        }
        let request = self
            .builder
            .cases
            .get_mut(id.index)
            .ok_or(CaseSelectionError::UnknownRequest)?;
        if request.id != id {
            return Err(CaseSelectionError::UnknownRequest);
        }
        match choice {
            CompileTimeCaseChoice::None
                if request.header.has_default || request.header.complete =>
            {
                return Err(CaseSelectionError::InvalidChoice);
            }
            CompileTimeCaseChoice::Arm(index) if index >= request.header.labels.len() => {
                return Err(CaseSelectionError::InvalidChoice);
            }
            CompileTimeCaseChoice::Default if !request.header.has_default => {
                return Err(CaseSelectionError::InvalidChoice);
            }
            _ => {}
        }
        if request.selected.is_some_and(|previous| previous != choice) {
            return Err(CaseSelectionError::AlreadySelected);
        }
        request.selected = Some(choice);
        let (file, location, specialization) = (
            request.file,
            request.location,
            request.specialization.clone(),
        );
        self.builder.record_case_selection(
            file,
            location,
            specialization,
            choice,
            SourceConditionOrigin::Semantic,
        );
        Ok(())
    }
}
impl ModuleGraph {
    pub fn source_cases(&self) -> &[SourceCaseSelection] {
        &self.source_cases
    }
    pub fn selected_case_for(
        &self,
        file: FileInstanceId,
        span: jai_source::Span,
        specialization: Option<&SourceSpecializationKey>,
    ) -> Option<CompileTimeCaseChoice> {
        self.source_cases
            .iter()
            .find(|case| {
                case.file == file
                    && case.location.span == span
                    && case.specialization.as_ref() == specialization
            })
            .map(|case| case.choice)
    }
}
impl Builder<'_> {
    pub(super) fn selected_case(
        &self,
        file: FileInstanceId,
        span: jai_source::Span,
    ) -> Option<CompileTimeCaseChoice> {
        self.graph
            .selected_case_for(file, span, self.specialization_for_span(file, span))
    }
    pub(super) fn record_case_selection(
        &mut self,
        file: FileInstanceId,
        location: SourceSpan,
        specialization: Option<SourceSpecializationKey>,
        choice: CompileTimeCaseChoice,
        origin: SourceConditionOrigin,
    ) {
        for request in &mut self.cases {
            if request.file == file
                && request.location == location
                && request.specialization == specialization
            {
                request.selected.get_or_insert(choice);
            }
        }
        if !self.graph.source_cases.iter().any(|case| {
            case.file == file && case.location == location && case.specialization == specialization
        }) {
            self.graph.source_cases.push(SourceCaseSelection {
                file,
                location,
                specialization,
                choice,
                origin,
            });
        }
    }
    pub(super) fn defer_case(
        &mut self,
        file: FileInstanceId,
        header: &CompileTimeCaseHeader,
        context: DiscoveryConditionContext,
    ) -> GraphError {
        let module = self.graph.files[file.index()].module;
        match self.prepare_callable_aliases(module) {
            Ok(None) => {}
            Ok(Some(error)) | Err(error) => return error,
        }
        let location = SourceSpan {
            source: self.graph.files[file.index()].source,
            span: header.span,
        };
        let specialization = self.specialization_for_span(file, header.span).cloned();
        if !self.cases.iter().any(|case| {
            case.file == file && case.location == location && case.specialization == specialization
        }) {
            self.cases.push(DeferredCase {
                id: CaseRequestId {
                    unit: self.graph.unit,
                    identity: next_case_identity(),
                    index: self.cases.len(),
                },
                file,
                module: self.graph.files[file.index()].module,
                location,
                header: header.clone(),
                context,
                specialization,
                selected: None,
            });
        }
        let diagnostic = self.graph.diagnostic(
            location,
            "source case is awaiting typed compile-time selection",
        );
        let rendered = diagnostic.render(&self.graph.sources);
        GraphError::Pending {
            diagnostic,
            rendered,
        }
    }
    /// Pure graph constants retain their scalar type. Nominal/type labels defer to sema.
    pub(super) fn constant_case(
        &self,
        file: FileInstanceId,
        header: &CompileTimeCaseHeader,
    ) -> Result<CompileTimeCaseChoice, GraphError> {
        if std::iter::once(&header.value)
            .chain(&header.labels)
            .any(|expression| self.case_requires_semantics(file, expression, &mut Vec::new()))
        {
            return Err(self.case_unsupported(
                file,
                header,
                "source case requires canonical typed selection",
            ));
        }
        let subject = self.constant_expression(file, &header.value, &mut Vec::new())?;
        let ty = subject.scalar_type().ok_or_else(|| {
            self.case_unsupported(
                file,
                header,
                "source case requires canonical typed selection",
            )
        })?;
        let subject = subject
            .coerce(ty, header.value.span)
            .map_err(|diagnostic| self.case_diagnostic(file, diagnostic))?;
        let mut labels = Vec::new();
        for label in &header.labels {
            let value = self
                .constant_expression(file, label, &mut Vec::new())?
                .coerce(ty, label.span)
                .map_err(|diagnostic| self.case_diagnostic(file, diagnostic))?;
            if labels.contains(&value) {
                return Err(self.case_diagnostic(
                    file,
                    Diagnostic::new(label.span, "duplicate compile-time case label"),
                ));
            }
            labels.push(value);
        }
        if header.complete
            && !header.has_default
            && !(header.operator == jai_syntax::CaseOperator::Equal
                && labels.contains(&jai_eval::Value::Bool(false))
                && labels.contains(&jai_eval::Value::Bool(true)))
        {
            return Err(self.case_unsupported(
                file,
                header,
                "#complete source cases require canonical typed coverage",
            ));
        }
        Ok(
            match labels.iter().position(|label| match header.operator {
                jai_syntax::CaseOperator::Equal => label == &subject,
                jai_syntax::CaseOperator::NotEqual => label != &subject,
            }) {
                Some(index) => CompileTimeCaseChoice::Arm(index),
                None if header.has_default => CompileTimeCaseChoice::Default,
                None => CompileTimeCaseChoice::None,
            },
        )
    }
    /// Inspect binding provenance before scalar evaluation can erase or reject nominal values.
    /// Unknown names and cycles still go through the ordinary readiness/error evaluator.
    fn case_requires_semantics(
        &self,
        file: FileInstanceId,
        expression: &jai_syntax::Expression,
        active: &mut Vec<DeclarationId>,
    ) -> bool {
        use jai_syntax::ExpressionKind as E;
        let path = match &expression.kind {
            E::Name(name) => Some(NamePath {
                root: *name,
                members: Vec::new(),
            }),
            E::QualifiedName(path) => Some(path.clone()),
            E::Integer(_) | E::Float(_) | E::Character(_) | E::Bool(_) => return false,
            E::Unary(_, value) | E::Cast(_, _, value) => {
                return self.case_requires_semantics(file, value, active);
            }
            E::Binary(_, left, right) => {
                return self.case_requires_semantics(file, left, active)
                    || self.case_requires_semantics(file, right, active);
            }
            // Calls, inferred labels, types and aggregates need the genuine source resolver.
            _ => return true,
        };
        match self
            .graph
            .lookup(file, &path.expect("name expression has a path"))
        {
            Ok(Binding::Parameter(id)) => !matches!(
                self.graph.parameters[id.index()].value,
                ParameterValue::Scalar(_)
            ),
            Ok(Binding::SourceMember { .. }) | Ok(Binding::StorageMember(_)) => true,
            Ok(Binding::Declaration(id)) => {
                if active.contains(&id) {
                    return false;
                }
                let declaration = &self.graph.declarations[id.index()];
                let FileDeclarationKind::Constant(constant) = &declaration.syntax.kind else {
                    return true;
                };
                active.push(id);
                let result =
                    self.case_requires_semantics(declaration.file, &constant.initializer, active);
                active.pop();
                result
            }
            Ok(Binding::OverloadSet(_)) => true,
            Ok(Binding::Module(_)) | Err(_) => false,
        }
    }
    fn case_diagnostic(&self, file: FileInstanceId, diagnostic: Diagnostic) -> GraphError {
        let diagnostic = self.graph.diagnostic(
            SourceSpan {
                source: self.graph.files[file.index()].source,
                span: diagnostic.span,
            },
            diagnostic.message,
        );
        let rendered = diagnostic.render(&self.graph.sources);
        GraphError::Located {
            diagnostic,
            rendered,
        }
    }
    fn case_unsupported(
        &self,
        file: FileInstanceId,
        header: &CompileTimeCaseHeader,
        message: &str,
    ) -> GraphError {
        let diagnostic = self.graph.diagnostic(
            SourceSpan {
                source: self.graph.files[file.index()].source,
                span: header.span,
            },
            message,
        );
        let rendered = diagnostic.render(&self.graph.sources);
        GraphError::Unsupported {
            diagnostic,
            rendered,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    fn sources() -> SourceOverlay {
        let mut sources = SourceOverlay::new();
        sources
            .insert(
                Path::new("/own-case-discovery/main.jai"),
                b"Tag::enum{A;B;} #if Tag.B == {case .A;#load \"absent.jai\";case .B;ANSWER::42;}"
                    .to_vec(),
            )
            .unwrap();
        sources
    }
    #[test]
    fn case_choices_are_immutable_validated_and_bound_to_one_discovery_unit() {
        let sources = sources();
        let mut first = GraphDiscovery::new(
            Path::new("/own-case-discovery/main.jai"),
            GraphOptions::default(),
            &sources,
        )
        .unwrap();
        let mut second = GraphDiscovery::new(
            Path::new("/own-case-discovery/main.jai"),
            GraphOptions::default(),
            &sources,
        )
        .unwrap();
        assert!(!first.advance().unwrap().is_complete());
        assert!(!second.advance().unwrap().is_complete());
        let request = first.pending_cases().next().unwrap().clone();
        assert_eq!(
            second.select_case(request.id, CompileTimeCaseChoice::Arm(1)),
            Err(CaseSelectionError::UnknownRequest)
        );
        assert_eq!(
            first.select_case(request.id, CompileTimeCaseChoice::Default),
            Err(CaseSelectionError::InvalidChoice)
        );
        assert_eq!(
            first.select_case(request.id, CompileTimeCaseChoice::Arm(2)),
            Err(CaseSelectionError::InvalidChoice)
        );
        first
            .select_case(request.id, CompileTimeCaseChoice::Arm(1))
            .unwrap();
        first
            .select_case(request.id, CompileTimeCaseChoice::Arm(1))
            .unwrap();
        assert_eq!(
            first.select_case(request.id, CompileTimeCaseChoice::Arm(0)),
            Err(CaseSelectionError::AlreadySelected)
        );
        assert!(first.advance().unwrap().is_complete());
        let graph = first.into_graph().ok().unwrap();
        assert_eq!(graph.source_cases().len(), 1);
        assert_eq!(
            graph.source_cases()[0].origin,
            SourceConditionOrigin::Semantic
        );
        assert!(
            graph
                .declarations()
                .iter()
                .any(|declaration| graph.symbols().name(declaration.name()) == "ANSWER")
        );
    }

    #[test]
    fn nominal_module_parameter_cases_defer_before_scalar_evaluation() {
        for source in [
            "#module_parameters(K:Kind=.B){Kind::enum{A;B;}}; alias::K; #if alias == {case .A;#load \"absent.jai\";case .B;ANSWER::42;}",
            "#module_parameters(T:Type=u8); alias::T; #if alias == {case s32;#load \"absent.jai\";case u8;ANSWER::42;}",
            "#module_parameters(K:string=\"B\"); alias::K; #if alias == {case \"A\";#load \"absent.jai\";case \"B\";ANSWER::42;}",
        ] {
            let mut sources = SourceOverlay::new();
            sources
                .insert(
                    Path::new("/own-case-discovery/main.jai"),
                    source.as_bytes().to_vec(),
                )
                .unwrap();
            let mut discovery = GraphDiscovery::new(
                Path::new("/own-case-discovery/main.jai"),
                GraphOptions::default(),
                &sources,
            )
            .unwrap();
            assert!(!discovery.advance().unwrap().is_complete());
            let request = discovery
                .pending_cases()
                .next()
                .expect("nominal selector needs typed selection")
                .clone();
            discovery
                .select_case(request.id, CompileTimeCaseChoice::Arm(1))
                .unwrap();
            assert!(discovery.advance().unwrap().is_complete());
        }
    }
}
