//! Reached layout queries complete their original reserved source record only.
use super::*;
use crate::{Binding, Resolver};
use std::cell::Cell;

impl Resolver<'_> {
    pub(crate) fn prepare_queried_source_layout(
        &mut self,
        ty: TypeId,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if !matches!(self.types.kind(ty), Ok(jai_types::TypeKind::Record(_)))
            || self.types.record_definition(ty).is_ok()
        {
            return Ok(());
        }
        let Some(scope) = self.graph_scope else {
            return Ok(());
        };
        let graph = scope.declarations.graph;
        let pending = Cell::new(None);
        let context = self.compile_time;
        let mut evaluate = |file, expression: &syntax::Expression| {
            jai_eval::evaluate_paths(expression, |path, span| {
                let defining = crate::modules::FileScope {
                    file,
                    substitution: None,
                    ..scope
                };
                match defining.value(path, span) {
                    Ok(Binding::Constant(value)) => Ok(value),
                    Ok(Binding::Enum(value)) => Ok(ScalarConstant::Int(value.value)),
                    Ok(_) => Err(Diagnostic::new(
                        span,
                        "record layout count requires a scalar constant",
                    )),
                    Err(error) => {
                        if context.is_some()
                            && let Ok(jai_modules::Binding::Declaration(id)) =
                                graph.lookup(file, path)
                            && let Some(source) = graph.declaration(id)
                            && matches!(
                                source.syntax().kind,
                                syntax::FileDeclarationKind::Constant(_)
                            )
                            && !scope.declarations.nominals.is_type_alias(graph, id)
                            && !scope.declarations.values.contains_key(&id)
                        {
                            let cause = PendingType::Constant {
                                declaration: id,
                                location: jai_source::SourceSpan {
                                    source: graph
                                        .file(file)
                                        .expect("original record source file")
                                        .source(),
                                    span,
                                },
                            };
                            pending.set(Some(cause));
                            let diagnostic = cause.diagnostic(graph);
                            return Err(Diagnostic::at_source(
                                diagnostic.location,
                                diagnostic.message,
                            ));
                        }
                        Err(error)
                    }
                }
            })
            .map_err(|error| crate::modules::located(graph, file, error))
        };
        let result = TypeResolver {
            graph,
            types: self.types,
            nominals: &scope.declarations.nominals,
            records: &mut self.meta.record_specializations,
            evaluate: &mut evaluate,
            scalar_pending: Some(&pending),
            aliases: HashSet::new(),
            lexical: None,
            lexical_active: false,
            nominal_context: NominalAnnotationContext::None,
        }
        .prepare_reserved_source_record(ty, scope.file, span);
        let result = match pending.get() {
            Some(cause) => Err(TypeFailure::Pending(cause)),
            None => result,
        };
        match result {
            Ok(()) => Ok(()),
            Err(TypeFailure::Pending(cause)) => {
                if let PendingType::Constant {
                    declaration, ..
                } = cause
                    && let Some(context) = context
                {
                    let mut required = context.pending_constants.borrow_mut();
                    if !required.contains(&declaration) {
                        required.push(declaration);
                    }
                }
                let diagnostic = cause.diagnostic(graph);
                Err(Diagnostic::at_source(
                    diagnostic.location,
                    diagnostic.message,
                ))
            }
            Err(TypeFailure::Diagnostic(error)) => {
                Err(Diagnostic::at_source(error.location, error.message))
            }
        }
    }
}

impl<F> TypeResolver<'_, '_, F>
where
    F: FnMut(FileInstanceId, &syntax::Expression) -> Result<ScalarConstant, LocatedDiagnostic>,
{
    fn prepare_reserved_source_record(
        &mut self,
        ty: TypeId,
        file: FileInstanceId,
        span: Span,
    ) -> TypeResult<()> {
        let mut owner = ty;
        let mut path = Vec::new();
        while let Some((parent, index)) = self.records.nested_parent(owner) {
            if path.len() >= crate::constant_limits::MAX_CONSTANT_DEPTH {
                return Err(failure(
                    self.graph,
                    file,
                    Diagnostic::new(
                        span,
                        "nested source layout exceeds declaration depth budget",
                    ),
                ));
            }
            path.push((parent, index, owner));
            owner = parent;
        }
        let Some(source) = self
            .nominals
            .declarations
            .iter()
            .find_map(|(&id, &reserved)| {
                (reserved == owner)
                    .then(|| self.graph.declaration(id))
                    .flatten()
            })
        else {
            // Specialized and lexical records have their own genuine body jobs.
            return Ok(());
        };
        let syntax::FileDeclarationKind::Record(record) = &source.syntax().kind else {
            return Ok(());
        };
        if !record.parameters.is_empty() {
            return Ok(());
        }
        path.reverse();
        if path.is_empty() {
            return member_enums::prepare_static_record(
                self.graph,
                source.id(),
                self.types,
                self.nominals,
                self.records,
                self.evaluate,
            );
        }
        self.prepare_nested_layout_path(
            source.id(),
            source.file(),
            record,
            &Substitution::default(),
            &path,
        )
    }

    fn prepare_nested_layout_path(
        &mut self,
        origin: DeclarationId,
        file: FileInstanceId,
        record: &syntax::RecordDeclaration,
        outer: &Substitution,
        path: &[(TypeId, usize, TypeId)],
    ) -> TypeResult<()> {
        let &(owner, index, expected) = path.first().expect("a genuine nested source path");
        let scope = self.reserve_member_names(owner, file, record, outer)?;
        let selected = self.checked_members(file, &record.members, outer)?;
        let Some(syntax::RecordMember::Record(nested)) = selected.get(index) else {
            return Err(failure(
                self.graph,
                file,
                Diagnostic::new(
                    record.span,
                    "reserved nested layout no longer matches its original selected member",
                ),
            ));
        };
        if self
            .records
            .reserve_nested(owner, index, nested.kind, self.types)
            != expected
        {
            return Err(failure(
                self.graph,
                file,
                Diagnostic::new(
                    nested.span,
                    "nested source layout changed its canonical type identity",
                ),
            ));
        }
        if path.len() == 1 {
            self.bind_nested_record(
                owner,
                Some(RecordTemplateId(origin)),
                index,
                file,
                nested,
                &scope,
            )?;
            return Ok(());
        }
        self.prepare_nested_layout_path(origin, file, nested, &scope, &path[1..])
    }
}
