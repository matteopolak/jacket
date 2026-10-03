//! Discover body source dependencies without publishing lexical names globally.
use super::*;
use jai_syntax::{Expression, ExpressionKind, RecordMember, Statement, StatementKind};

#[derive(Clone)]
enum LocalValue {
    Constant(jai_syntax::ConstantDeclaration),
    Semantic,
}
#[derive(Clone, Default)]
struct LocalScope {
    values: HashMap<Symbol, LocalValue>,
    statements: Vec<Statement>,
    parameters: Vec<jai_syntax::Parameter>,
    runtime_names: Vec<Symbol>,
    field_names: Vec<Symbol>,
    record_members: Vec<RecordMember>,
    pending_exports: bool,
}
impl LocalScope {
    fn parameters(parameters: &[jai_syntax::Parameter]) -> Self {
        Self {
            values: parameters
                .iter()
                .map(|parameter| (parameter.name, LocalValue::Semantic))
                .collect(),
            parameters: parameters.to_vec(),
            statements: vec![],
            runtime_names: vec![],
            field_names: vec![],
            record_members: vec![],
            pending_exports: false,
        }
    }
}
type LocalScopes = Vec<LocalScope>;

struct RecordSelection {
    members: Vec<RecordMember>,
    pending: Option<GraphError>,
}

impl Builder<'_> {
    pub(super) fn scoped_declaration(
        &mut self,
        file: FileInstanceId,
        declaration: &FileDeclaration,
    ) -> Result<(), GraphError> {
        // A marker has no declaration identity or source body job.
        if matches!(&declaration.kind, FileDeclarationKind::Placeholder(_)) {
            return Ok(());
        }
        let owner = self
            .graph
            .declarations
            .iter()
            .find(|candidate| {
                candidate.file() == file && candidate.location() == declaration.location
            })
            .expect("discovery declaration retains graph identity")
            .id();
        match &declaration.kind {
            FileDeclarationKind::Procedure(procedure) => {
                if !contains_import(&procedure.body)
                    || !self.scan_source_procedure(owner, file, procedure)
                {
                    return Ok(());
                }
                let mut scopes = vec![LocalScope::parameters(&procedure.parameters)];
                self.scoped_statements(owner, file, &procedure.body, &mut scopes, true)
            }
            FileDeclarationKind::Record(record) => {
                self.scoped_record(owner, file, &record.members, &mut vec![])
            }
            _ => Ok(()),
        }
    }

    fn scoped_record(
        &mut self,
        owner: DeclarationId,
        file: FileInstanceId,
        members: &[RecordMember],
        scopes: &mut LocalScopes,
    ) -> Result<(), GraphError> {
        let mut record_scope = LocalScope {
            record_members: members.to_vec(),
            ..Default::default()
        };
        retain_record_bindings(&mut record_scope, members);
        scopes.push(record_scope);
        let result = (|| {
            let fields_affect_dependencies = record_contains_import(members);
            let mut pending = self.promote_anonymous_fields(
                owner,
                file,
                members,
                scopes,
                fields_affect_dependencies,
            )?;
            let selected = self.select_record_import_members(
                owner,
                file,
                members,
                scopes,
                fields_affect_dependencies,
            )?;
            retain_pending(&mut pending, selected.pending);
            retain_pending(
                &mut pending,
                self.promote_anonymous_fields(
                    owner,
                    file,
                    &new_anonymous_members(&selected.members, members),
                    scopes,
                    fields_affect_dependencies,
                )?,
            );
            for member in &selected.members {
                let result = match member {
                    RecordMember::Procedure(procedure) if contains_import(&procedure.body) => {
                        if !self.scan_source_procedure(owner, file, procedure) {
                            continue;
                        }
                        let mut inherited = scopes.clone();
                        inherited.push(LocalScope::parameters(&procedure.parameters));
                        self.scoped_statements(owner, file, &procedure.body, &mut inherited, true)
                    }
                    RecordMember::Record(record) => {
                        self.scoped_record(owner, file, &record.members, scopes)
                    }
                    RecordMember::AnonymousRecord(record) => {
                        self.scoped_record(owner, file, &record.members, scopes)
                    }
                    _ => Ok(()),
                };
                match result {
                    Ok(()) => {}
                    Err(
                        error @ GraphError::Pending {
                            ..
                        },
                    ) => {
                        pending.get_or_insert(error);
                    }
                    Err(error) => return Err(error),
                }
            }
            pending.map_or(Ok(()), Err)
        })();
        scopes.pop();
        result
    }

    /// Select an unnamed child's physical fields in its actual lexical frame
    /// before guards in the parent can substitute a same-spelled file constant.
    fn promote_anonymous_fields(
        &mut self,
        owner: DeclarationId,
        file: FileInstanceId,
        members: &[RecordMember],
        scopes: &mut LocalScopes,
        fields_affect_dependencies: bool,
    ) -> Result<Option<GraphError>, GraphError> {
        let mut pending = None;
        for member in members {
            let RecordMember::AnonymousRecord(record) = member else {
                continue;
            };
            let mut child_scope = LocalScope {
                record_members: record.members.clone(),
                ..Default::default()
            };
            retain_record_bindings(&mut child_scope, &record.members);
            scopes.push(child_scope);
            let result = (|| {
                let mut pending = self.promote_anonymous_fields(
                    owner,
                    file,
                    &record.members,
                    scopes,
                    fields_affect_dependencies,
                )?;
                let selected = self.select_record_import_members(
                    owner,
                    file,
                    &record.members,
                    scopes,
                    fields_affect_dependencies,
                )?;
                retain_pending(&mut pending, selected.pending);
                retain_pending(
                    &mut pending,
                    self.promote_anonymous_fields(
                        owner,
                        file,
                        &new_anonymous_members(&selected.members, &record.members),
                        scopes,
                        fields_affect_dependencies,
                    )?,
                );
                // Nested children have already promoted their selected fields
                // into this actual child frame. Unselected alternatives reserve
                // names only; they are not asserted to be physical storage.
                let mut fields = LocalScope::default();
                let child = scopes.last().unwrap();
                for &name in &child.field_names {
                    fields.values.entry(name).or_insert(LocalValue::Semantic);
                    fields.field_names.push(name);
                    if child.runtime_names.contains(&name) {
                        fields.runtime_names.push(name);
                    }
                }
                Ok((fields, pending))
            })();
            scopes.pop();
            let (fields, child_pending) = result?;
            let parent = scopes.last_mut().unwrap();
            for name in fields.values.into_keys() {
                parent.values.entry(name).or_insert(LocalValue::Semantic);
                if !parent.field_names.contains(&name) {
                    parent.field_names.push(name);
                }
            }
            for name in fields.runtime_names {
                if !parent.runtime_names.contains(&name) {
                    parent.runtime_names.push(name);
                }
            }
            retain_pending(&mut pending, child_pending);
        }
        Ok(pending)
    }

    fn scoped_statements(
        &mut self,
        owner: DeclarationId,
        file: FileInstanceId,
        statements: &[Statement],
        scopes: &mut LocalScopes,
        block: bool,
    ) -> Result<(), GraphError> {
        if block {
            scopes.push(LocalScope {
                statements: statements.to_vec(),
                ..Default::default()
            });
        } else {
            scopes
                .last_mut()
                .unwrap()
                .statements
                .extend_from_slice(statements);
        }
        // Lexical declarations have whole-block visibility. Retain initializers
        // and their defining depth rather than adding fake file declarations.
        for wrapper in statements {
            let statement = match &wrapper.kind {
                StatementKind::UsingDeclaration {
                    declaration, ..
                } => declaration.as_ref(),
                _ => wrapper,
            };
            let value = match &statement.kind {
                StatementKind::Constant(constant) => {
                    Some((constant.name, LocalValue::Constant(constant.clone())))
                }
                StatementKind::ConstantResults(declaration) => {
                    for &(name, _) in &declaration.names {
                        if self.graph.symbols.name(name) != "_" {
                            scopes
                                .last_mut()
                                .unwrap()
                                .values
                                .insert(name, LocalValue::Semantic);
                        }
                    }
                    None
                }
                StatementKind::Declare(declaration) => {
                    scopes
                        .last_mut()
                        .unwrap()
                        .runtime_names
                        .push(declaration.name());
                    Some((declaration.name(), LocalValue::Semantic))
                }
                StatementKind::Procedure(procedure) => Some((procedure.name, LocalValue::Semantic)),
                StatementKind::ProcedurePrototype(prototype) => {
                    Some((prototype.name, LocalValue::Semantic))
                }
                StatementKind::Record(record) => Some((record.name, LocalValue::Semantic)),
                StatementKind::Enum(enumeration) => Some((enumeration.name, LocalValue::Semantic)),
                StatementKind::TypeAlias(alias) => Some((alias.name, LocalValue::Semantic)),
                StatementKind::Import(import) => {
                    import.namespace.map(|name| (name, LocalValue::Semantic))
                }
                _ => None,
            };
            if let Some((name, value)) = value {
                scopes.last_mut().unwrap().values.insert(name, value);
            }
        }
        let result = (|| {
            let mut pending = None;
            for statement in statements {
                let depth = scopes.len();
                let result = (|| {
                    match &statement.kind {
                        StatementKind::UsingDeclaration {
                            declaration, ..
                        } => {
                            let original_count = scopes.last().unwrap().statements.len();
                            let child_result = self.scoped_statements(
                                owner,
                                file,
                                std::slice::from_ref(declaration.as_ref()),
                                scopes,
                                false,
                            );
                            scopes
                                .last_mut()
                                .unwrap()
                                .statements
                                .truncate(original_count);
                            child_result?;
                            let directive =
                                statement.using_declaration_directive().ok_or_else(|| {
                                    self.located(
                                        SourceSpan {
                                            source: self.graph.files[file.index()].source,
                                            span: statement.span,
                                        },
                                        "using requires one named declaration",
                                    )
                                })?;
                            let location = SourceSpan {
                                source: self.graph.files[file.index()].source,
                                span: directive.span,
                            };
                            let result = self.defer_using_declaration(
                                file,
                                &directive,
                                Visibility::File,
                                location,
                                lexical_context(owner, scopes),
                                UsingDeclarationSource::Statement(declaration.clone()),
                            );
                            if let Some(publication) = self.graph.using_publication(
                                file,
                                directive.span,
                                self.specialization_for_span(file, directive.span),
                            ) {
                                let scope = scopes.last_mut().unwrap();
                                for &(name, _) in &publication.bindings {
                                    scope.values.insert(name, LocalValue::Semantic);
                                }
                                for &(_, name) in &publication.aliases {
                                    scope.values.insert(name, LocalValue::Semantic);
                                    scope.runtime_names.push(name);
                                }
                            } else {
                                scopes.last_mut().unwrap().pending_exports = true;
                            }
                            result?;
                        }
                        StatementKind::Using(directive) => {
                            let location = SourceSpan {
                                source: self.graph.files[file.index()].source,
                                span: directive.span,
                            };
                            let result = self.defer_using(
                                file,
                                directive,
                                Visibility::File,
                                location,
                                lexical_context(owner, scopes),
                            );
                            if let Some(publication) = self.graph.using_publication(
                                file,
                                directive.span,
                                self.specialization_for_span(file, directive.span),
                            ) {
                                let scope = scopes.last_mut().unwrap();
                                for &(name, _) in &publication.bindings {
                                    scope.values.insert(name, LocalValue::Semantic);
                                }
                                for &(_, name) in &publication.aliases {
                                    scope.values.insert(name, LocalValue::Semantic);
                                    scope.runtime_names.push(name);
                                }
                            } else {
                                scopes.last_mut().unwrap().pending_exports = true;
                            }
                            result?;
                        }
                        StatementKind::Import(import) => {
                            let mut import = jai_syntax::ImportDeclaration {
                                namespace: import.namespace,
                                using: import.using,
                                mode: import.mode,
                                target: import.target.clone(),
                                arguments: import.arguments.clone(),
                                visibility: Visibility::File,
                                location: SourceSpan {
                                    source: self.graph.files[file.index()].source,
                                    span: statement.span,
                                },
                            };
                            for arguments in [
                                &mut import.arguments.instance,
                                &mut import.arguments.program,
                            ]
                            .into_iter()
                            .flatten()
                            {
                                for argument in arguments {
                                    if let jai_syntax::ModuleArgumentValue::Expression(expression) =
                                        &mut argument.value
                                    {
                                        *expression = self.expand_local(
                                            file,
                                            expression,
                                            scopes,
                                            &mut vec![],
                                        )?;
                                    }
                                }
                            }
                            let module = if let Some(module) = self.graph.scoped_import_for(
                                file,
                                statement.span,
                                self.specialization_for_span(file, statement.span),
                            ) {
                                module
                            } else {
                                let module = self
                                    .import_module(file, &import)
                                    .map_err(|error| self.scoped_pending(error))?;
                                self.graph.scoped_imports.push(ScopedImportEdge {
                                    file,
                                    module,
                                    location: import.location,
                                    specialization: self
                                        .specialization_for_span(file, statement.span)
                                        .cloned(),
                                });
                                module
                            };
                            if import.namespace.is_none() || import.using {
                                for &name in self.graph.modules[module.index()].exports.keys() {
                                    scopes
                                        .last_mut()
                                        .unwrap()
                                        .values
                                        .entry(name)
                                        .or_insert(LocalValue::Semantic);
                                }
                            }
                        }
                        StatementKind::CompileTimeIf {
                            condition,
                            then_body,
                            else_body,
                        } => {
                            let needs_dependency =
                                contains_import(then_body) || contains_import(else_body);
                            let selected = if let Some(selected) =
                                self.selected_condition(file, condition.span)
                            {
                                selected
                            } else if needs_dependency
                                && self.specialization_for_span(file, condition.span).is_some()
                            {
                                reserve_conditional_names(
                                    scopes.last_mut().unwrap(),
                                    then_body,
                                    self.graph.symbols.find("_"),
                                );
                                reserve_conditional_names(
                                    scopes.last_mut().unwrap(),
                                    else_body,
                                    self.graph.symbols.find("_"),
                                );
                                return Err(self.defer_condition(
                                    file,
                                    condition,
                                    lexical_context(owner, scopes),
                                ));
                            } else {
                                let evaluated = self
                                    .expand_local(file, condition, scopes, &mut vec![])
                                    .and_then(|condition| {
                                        self.constant_expression(file, &condition, &mut vec![])
                                    });
                                match evaluated {
                                    Ok(value) => {
                                        let selected =
                                            truth(&value, condition.span).map_err(|error| {
                                                self.located(
                                                    SourceSpan {
                                                        source: self.graph.files[file.index()]
                                                            .source,
                                                        span: error.span,
                                                    },
                                                    error.message,
                                                )
                                            })?;
                                        self.record_selection(file, condition.span, selected);
                                        selected
                                    }
                                    Err(
                                        GraphError::Pending {
                                            ..
                                        }
                                        | GraphError::Unsupported {
                                            ..
                                        },
                                    ) => {
                                        if !needs_dependency {
                                            reserve_conditional_names(
                                                scopes.last_mut().unwrap(),
                                                then_body,
                                                self.graph.symbols.find("_"),
                                            );
                                            reserve_conditional_names(
                                                scopes.last_mut().unwrap(),
                                                else_body,
                                                self.graph.symbols.find("_"),
                                            );
                                            return Ok(());
                                        }
                                        reserve_conditional_names(
                                            scopes.last_mut().unwrap(),
                                            then_body,
                                            self.graph.symbols.find("_"),
                                        );
                                        reserve_conditional_names(
                                            scopes.last_mut().unwrap(),
                                            else_body,
                                            self.graph.symbols.find("_"),
                                        );
                                        return Err(self.defer_condition(
                                            file,
                                            condition,
                                            DiscoveryConditionContext::Lexical {
                                                declaration: owner,
                                                scopes: scopes
                                                    .iter()
                                                    .map(|scope| DiscoveryLexicalScope {
                                                        parameters: scope.parameters.clone(),
                                                        statements: scope.statements.clone(),
                                                        runtime_names: scope.runtime_names.clone(),
                                                        record_members: scope
                                                            .record_members
                                                            .clone(),
                                                    })
                                                    .collect(),
                                            },
                                        ));
                                    }
                                    Err(error) => return Err(error),
                                }
                            };
                            self.scoped_statements(
                                owner,
                                file,
                                if selected {
                                    then_body
                                } else {
                                    else_body
                                },
                                scopes,
                                false,
                            )?;
                        }
                        StatementKind::CompileTimeCases(cases) => {
                            let needs_dependency = case_bodies(cases).any(contains_import);
                            let selected =
                                self.scoped_case_body(owner, file, cases, scopes, needs_dependency);
                            if !matches!(&selected, Ok(Some(_))) {
                                for body in case_bodies(cases) {
                                    reserve_conditional_names(
                                        scopes.last_mut().unwrap(),
                                        body,
                                        self.graph.symbols.find("_"),
                                    );
                                }
                            }
                            if let Some(body) = selected? {
                                self.scoped_statements(owner, file, &body, scopes, false)?;
                            }
                        }
                        StatementKind::Block(body)
                        | StatementKind::Defer(body)
                        | StatementKind::CheckScope {
                            body, ..
                        }
                        | StatementKind::PushContext {
                            body, ..
                        } => self.scoped_statements(owner, file, body, scopes, true)?,
                        StatementKind::While(condition, body) => {
                            if let jai_syntax::WhileCondition::Binding {
                                name, ..
                            } = condition
                            {
                                scopes.push(LocalScope {
                                    values: HashMap::from([(*name, LocalValue::Semantic)]),
                                    runtime_names: vec![*name],
                                    ..Default::default()
                                });
                                self.scoped_statements(owner, file, body, scopes, true)?;
                                scopes.pop();
                            } else {
                                self.scoped_statements(owner, file, body, scopes, true)?;
                            }
                        }
                        StatementKind::If(_, yes, no) => {
                            self.scoped_statements(owner, file, yes, scopes, true)?;
                            self.scoped_statements(owner, file, no, scopes, true)?;
                        }
                        StatementKind::Range(range) => {
                            scopes.push(LocalScope {
                                values: HashMap::from([(range.iterator, LocalValue::Semantic)]),
                                runtime_names: vec![range.iterator],
                                ..Default::default()
                            });
                            self.scoped_statements(owner, file, &range.body, scopes, true)?;
                            scopes.pop();
                        }
                        StatementKind::ArrayLoop(loop_) => {
                            let names: Vec<_> =
                                std::iter::once(loop_.iterator).chain(loop_.index).collect();
                            scopes.push(LocalScope {
                                values: names
                                    .iter()
                                    .map(|&name| (name, LocalValue::Semantic))
                                    .collect(),
                                runtime_names: names,
                                ..Default::default()
                            });
                            self.scoped_statements(owner, file, &loop_.body, scopes, true)?;
                            scopes.pop();
                        }
                        StatementKind::Cases(cases) => {
                            for (_, body, _) in &cases.arms {
                                self.scoped_statements(owner, file, body, scopes, true)?;
                            }
                            if let Some(body) = &cases.default {
                                self.scoped_statements(owner, file, body, scopes, true)?;
                            }
                        }
                        StatementKind::Procedure(procedure) if contains_import(&procedure.body) => {
                            if !self.scan_source_procedure(owner, file, procedure) {
                                return Ok(());
                            }
                            let mut inherited = scopes.clone();
                            inherited.push(LocalScope::parameters(&procedure.parameters));
                            self.scoped_statements(
                                owner,
                                file,
                                &procedure.body,
                                &mut inherited,
                                true,
                            )?;
                        }
                        StatementKind::Record(record) => {
                            self.scoped_record(owner, file, &record.members, scopes)?
                        }
                        StatementKind::CallerExport(statement) => self.scoped_statements(
                            owner,
                            file,
                            std::slice::from_ref(statement),
                            scopes,
                            false,
                        )?,
                        _ => {}
                    }
                    Ok(())
                })();
                // A suspended nested loop must not leave its runtime-name frame
                // visible to the following independent source dependency.
                scopes.truncate(depth);
                match result {
                    Ok(()) => {}
                    Err(
                        error @ GraphError::Pending {
                            ..
                        },
                    ) => {
                        if let StatementKind::Import(import) = &statement.kind
                            && (import.namespace.is_none() || import.using)
                        {
                            scopes.last_mut().unwrap().pending_exports = true;
                        }
                        pending.get_or_insert(error);
                    }
                    Err(error) => return Err(error),
                }
            }
            pending.map_or(Ok(()), Err)
        })();
        if block {
            scopes.pop();
        }
        result
    }

    fn select_record_import_members(
        &mut self,
        owner: DeclarationId,
        file: FileInstanceId,
        members: &[RecordMember],
        scopes: &mut LocalScopes,
        fields_affect_dependencies: bool,
    ) -> Result<RecordSelection, GraphError> {
        let mut selected_members = Vec::new();
        let mut pending = None;
        for member in members {
            if let RecordMember::CompileTimeCases {
                cases, ..
            } = member
            {
                let needs_dependency = case_bodies(cases).any(record_contains_import)
                    || (fields_affect_dependencies
                        && case_bodies(cases).any(record_contains_physical_fields));
                let selected = self.scoped_case_body(owner, file, cases, scopes, needs_dependency);
                match selected {
                    Ok(Some(body)) => {
                        retain_record_bindings(scopes.last_mut().unwrap(), &body);
                        let body = self.select_record_import_members(
                            owner,
                            file,
                            &body,
                            scopes,
                            fields_affect_dependencies,
                        )?;
                        selected_members.extend(body.members);
                        retain_pending(&mut pending, body.pending);
                    }
                    Ok(None) => {
                        for body in case_bodies(cases) {
                            reserve_record_names(scopes.last_mut().unwrap(), body);
                        }
                        selected_members.push(member.clone());
                    }
                    Err(
                        error @ GraphError::Pending {
                            ..
                        },
                    ) => {
                        for body in case_bodies(cases) {
                            reserve_record_names(scopes.last_mut().unwrap(), body);
                        }
                        pending.get_or_insert(error);
                        selected_members.push(member.clone());
                    }
                    Err(error) => return Err(error),
                }
                continue;
            }
            let RecordMember::Conditional {
                condition,
                then_members,
                else_members,
                ..
            } = member
            else {
                selected_members.push(member.clone());
                continue;
            };
            let needs_dependency = record_contains_import(then_members)
                || record_contains_import(else_members)
                || (fields_affect_dependencies
                    && (record_contains_physical_fields(then_members)
                        || record_contains_physical_fields(else_members)));
            let selected = if let Some(selected) = self.selected_condition(file, condition.span) {
                selected
            } else if needs_dependency
                && self.specialization_for_span(file, condition.span).is_some()
            {
                reserve_record_names(scopes.last_mut().unwrap(), then_members);
                reserve_record_names(scopes.last_mut().unwrap(), else_members);
                let error = self.defer_condition(file, condition, lexical_context(owner, scopes));
                pending.get_or_insert(error);
                selected_members.push(member.clone());
                continue;
            } else {
                match self
                    .expand_local(file, condition, scopes, &mut vec![])
                    .and_then(|condition| self.constant_expression(file, &condition, &mut vec![]))
                {
                    Ok(value) => {
                        let selected = truth(&value, condition.span).map_err(|error| {
                            self.located(
                                SourceSpan {
                                    source: self.graph.files[file.index()].source,
                                    span: error.span,
                                },
                                error.message,
                            )
                        })?;
                        self.record_selection(file, condition.span, selected);
                        selected
                    }
                    Err(
                        GraphError::Pending {
                            ..
                        }
                        | GraphError::Unsupported {
                            ..
                        },
                    ) if !needs_dependency => {
                        reserve_record_names(scopes.last_mut().unwrap(), then_members);
                        reserve_record_names(scopes.last_mut().unwrap(), else_members);
                        selected_members.push(member.clone());
                        continue;
                    }
                    Err(
                        GraphError::Pending {
                            ..
                        }
                        | GraphError::Unsupported {
                            ..
                        },
                    ) => {
                        reserve_record_names(scopes.last_mut().unwrap(), then_members);
                        reserve_record_names(scopes.last_mut().unwrap(), else_members);
                        let error = self.defer_condition(
                            file,
                            condition,
                            DiscoveryConditionContext::Lexical {
                                declaration: owner,
                                scopes: scopes
                                    .iter()
                                    .map(|scope| DiscoveryLexicalScope {
                                        parameters: scope.parameters.clone(),
                                        statements: scope.statements.clone(),
                                        runtime_names: scope.runtime_names.clone(),
                                        record_members: scope.record_members.clone(),
                                    })
                                    .collect(),
                            },
                        );
                        pending.get_or_insert(error);
                        selected_members.push(member.clone());
                        continue;
                    }
                    Err(error) => return Err(error),
                }
            };
            let branch = if selected {
                then_members
            } else {
                else_members
            };
            retain_record_bindings(scopes.last_mut().unwrap(), branch);
            let branch = self.select_record_import_members(
                owner,
                file,
                branch,
                scopes,
                fields_affect_dependencies,
            )?;
            selected_members.extend(branch.members);
            retain_pending(&mut pending, branch.pending);
        }
        Ok(RecordSelection {
            members: selected_members,
            pending,
        })
    }

    fn scoped_case_body<T: Clone>(
        &mut self,
        owner: DeclarationId,
        file: FileInstanceId,
        cases: &jai_syntax::CompileTimeCases<T>,
        scopes: &[LocalScope],
        needs_dependency: bool,
    ) -> Result<Option<Vec<T>>, GraphError> {
        let header = cases.header();
        let choice = if let Some(choice) = self.selected_case(file, header.span) {
            choice
        } else {
            if needs_dependency && self.specialization_for_span(file, header.span).is_some() {
                return Err(self.defer_case(file, &header, lexical_context(owner, scopes)));
            }
            let evaluated = (|| {
                let mut expanded = header.clone();
                expanded.value = self.expand_local(file, &header.value, scopes, &mut vec![])?;
                for label in &mut expanded.labels {
                    *label = self.expand_local(file, label, scopes, &mut vec![])?;
                }
                self.constant_case(file, &expanded)
            })();
            match evaluated {
                Ok(choice) => {
                    self.record_case_selection(
                        file,
                        SourceSpan {
                            source: self.graph.files[file.index()].source,
                            span: header.span,
                        },
                        self.specialization_for_span(file, header.span).cloned(),
                        choice,
                        SourceConditionOrigin::Scalar,
                    );
                    choice
                }
                Err(
                    GraphError::Pending {
                        ..
                    }
                    | GraphError::Unsupported {
                        ..
                    },
                ) => {
                    return if needs_dependency {
                        Err(self.defer_case(file, &header, lexical_context(owner, scopes)))
                    } else {
                        Ok(None)
                    };
                }
                Err(error) => return Err(error),
            }
        };
        cases.selected_body(choice).map(Some).ok_or_else(|| {
            self.located(
                SourceSpan {
                    source: self.graph.files[file.index()].source,
                    span: header.span,
                },
                "compile-time case selection has an invalid fall-through destination",
            )
        })
    }

    fn expand_local(
        &self,
        file: FileInstanceId,
        expression: &Expression,
        scopes: &[LocalScope],
        active: &mut Vec<(usize, Symbol)>,
    ) -> Result<Expression, GraphError> {
        if let ExpressionKind::Name(name) = expression.kind
            && let Some((depth, value)) =
                scopes.iter().enumerate().rev().find_map(|(depth, scope)| {
                    scope
                        .values
                        .get(&name)
                        .map(|value| (depth, Some(value)))
                        .or_else(|| scope.pending_exports.then_some((depth, None)))
                })
        {
            let location = SourceSpan {
                source: self.graph.files[file.index()].source,
                span: expression.span,
            };
            let Some(value) = value else {
                return Err(self.semantic_pending(
                    location,
                    "scoped import exports are awaiting selected source discovery",
                ));
            };
            let LocalValue::Constant(constant) = value else {
                if scopes[depth]
                    .parameters
                    .iter()
                    .any(|parameter| parameter.name == name)
                    && self
                        .specialization_for_span(file, expression.span)
                        .is_some_and(|key| {
                            key.arguments()
                                .iter()
                                .any(|(parameter, _)| *parameter == name)
                        })
                {
                    return Ok(expression.clone());
                }
                return Err(self.semantic_pending(
                    location,
                    "scoped import dependency requires lexical semantic evaluation",
                ));
            };
            if active.contains(&(depth, name)) {
                return Err(self.located(
                    location,
                    "cyclic lexical constant dependency in scoped import",
                ));
            }
            active.push((depth, name));
            let result = self.expand_local(file, &constant.initializer, &scopes[..=depth], active);
            active.pop();
            let mut expanded = result?;
            if let Some(annotation) = &constant.ty {
                let Some(ty) = annotation.as_scalar() else {
                    return Err(self.semantic_pending(
                        location,
                        "typed constant annotation requires lexical semantic resolution",
                    ));
                };
                expanded = Expression {
                    span: expression.span,
                    kind: ExpressionKind::Cast(
                        jai_syntax::CastMode::Checked,
                        ty,
                        Box::new(expanded),
                    ),
                };
            }
            expanded.span = expression.span;
            return Ok(expanded);
        }
        if let ExpressionKind::QualifiedName(path) = &expression.kind
            && scopes
                .iter()
                .rev()
                .any(|scope| scope.values.contains_key(&path.root) || scope.pending_exports)
        {
            return Err(self.semantic_pending(
                SourceSpan {
                    source: self.graph.files[file.index()].source,
                    span: expression.span,
                },
                "scoped import dependency requires lexical namespace evaluation",
            ));
        }
        let mut expanded = expression.clone();
        match &mut expanded.kind {
            ExpressionKind::Unary(_, value)
            | ExpressionKind::Cast(_, _, value)
            | ExpressionKind::TypeCast {
                value, ..
            } => **value = self.expand_local(file, value, scopes, active)?,
            ExpressionKind::Binary(_, lhs, rhs) => {
                **lhs = self.expand_local(file, lhs, scopes, active)?;
                if let ExpressionKind::Binary(operation, _, _) = &expression.kind
                    && matches!(
                        operation,
                        jai_syntax::BinaryOp::LogicalAnd | jai_syntax::BinaryOp::LogicalOr
                    )
                    && let Ok(value) = self.constant_expression(file, lhs, &mut vec![])
                    && let Ok(value) = truth(&value, lhs.span)
                    && ((*operation == jai_syntax::BinaryOp::LogicalAnd && !value)
                        || (*operation == jai_syntax::BinaryOp::LogicalOr && value))
                {
                    return Ok(Expression {
                        span: expression.span,
                        kind: ExpressionKind::Bool(value),
                    });
                }
                **rhs = self.expand_local(file, rhs, scopes, active)?;
            }
            ExpressionKind::Conditional(value) => {
                *value.condition = self.expand_local(file, &value.condition, scopes, active)?;
                *value.then_value = self.expand_local(file, &value.then_value, scopes, active)?;
                if let Some(no) = &mut value.else_value {
                    **no = self.expand_local(file, no, scopes, active)?;
                }
            }
            _ => {}
        }
        Ok(expanded)
    }

    pub(super) fn semantic_pending(
        &self,
        location: SourceSpan,
        message: impl Into<String>,
    ) -> GraphError {
        let diagnostic = self.graph.diagnostic(location, message);
        let rendered = diagnostic.render(&self.graph.sources);
        GraphError::Pending {
            diagnostic,
            rendered,
        }
    }

    fn scoped_pending(&self, error: GraphError) -> GraphError {
        match error {
            GraphError::Unsupported {
                diagnostic, ..
            } => self.semantic_pending(
                diagnostic.location,
                "scoped import dependency is awaiting semantic compile-time evaluation",
            ),
            other => other,
        }
    }
}

fn truth(value: &jai_eval::Value, span: jai_source::Span) -> Result<bool, Diagnostic> {
    Ok(match value {
        jai_eval::Value::Bool(value) => *value,
        jai_eval::Value::Literal(value) => *value != 0,
        jai_eval::Value::Int(value) => value.value() != 0,
        jai_eval::Value::Float(value) => value.to_f64() != 0.0,
        jai_eval::Value::WeakFloat(value) => {
            value.round(value.default_type(), span)?.to_f64() != 0.0
        }
    })
}

fn retain_pending(pending: &mut Option<GraphError>, other: Option<GraphError>) {
    if pending.is_none() {
        *pending = other;
    }
}

fn lexical_context(owner: DeclarationId, scopes: &[LocalScope]) -> DiscoveryConditionContext {
    DiscoveryConditionContext::Lexical {
        declaration: owner,
        scopes: scopes
            .iter()
            .map(|scope| DiscoveryLexicalScope {
                parameters: scope.parameters.clone(),
                statements: scope.statements.clone(),
                runtime_names: scope.runtime_names.clone(),
                record_members: scope.record_members.clone(),
            })
            .collect(),
    }
}

fn new_anonymous_members(
    selected: &[RecordMember],
    original: &[RecordMember],
) -> Vec<RecordMember> {
    selected
        .iter()
        .filter(|member| {
            let RecordMember::AnonymousRecord(record) = member else {
                return false;
            };
            !original.iter().any(|member| {
                matches!(member, RecordMember::AnonymousRecord(other) if other.span == record.span)
            })
        })
        .cloned()
        .collect()
}

fn case_bodies<T>(cases: &jai_syntax::CompileTimeCases<T>) -> impl Iterator<Item = &[T]> {
    cases
        .arms
        .iter()
        .map(|arm| arm.body.as_slice())
        .chain(cases.default.iter().map(|arm| arm.body.as_slice()))
}

fn contains_import(statements: &[Statement]) -> bool {
    statements.iter().any(|statement| match &statement.kind {
        StatementKind::Import(_)
        | StatementKind::Using(_)
        | StatementKind::UsingDeclaration {
            ..
        } => true,
        StatementKind::CompileTimeIf {
            then_body,
            else_body,
            ..
        }
        | StatementKind::If(_, then_body, else_body) => {
            contains_import(then_body) || contains_import(else_body)
        }
        StatementKind::CompileTimeCases(cases) => case_bodies(cases).any(contains_import),
        StatementKind::Block(body)
        | StatementKind::Defer(body)
        | StatementKind::CheckScope {
            body, ..
        }
        | StatementKind::PushContext {
            body, ..
        }
        | StatementKind::While(_, body) => contains_import(body),
        StatementKind::Range(value) => contains_import(&value.body),
        StatementKind::ArrayLoop(value) => contains_import(&value.body),
        StatementKind::Cases(value) => {
            value.arms.iter().any(|(_, body, _)| contains_import(body))
                || value
                    .default
                    .as_ref()
                    .is_some_and(|body| contains_import(body))
        }
        StatementKind::Procedure(value) => contains_import(&value.body),
        StatementKind::CallerExport(value) => contains_import(std::slice::from_ref(value)),
        StatementKind::Record(value) => record_contains_import(&value.members),
        _ => false,
    })
}

fn retain_record_bindings(scope: &mut LocalScope, members: &[RecordMember]) {
    for member in members {
        let retained = match member {
            RecordMember::AnonymousRecord(record) => {
                retain_promoted_field_shadows(scope, &record.members);
                None
            }
            RecordMember::Field(field) => {
                scope.values.insert(field.name, LocalValue::Semantic);
                if !scope.runtime_names.contains(&field.name) {
                    scope.runtime_names.push(field.name);
                }
                if !scope.field_names.contains(&field.name) {
                    scope.field_names.push(field.name);
                }
                None
            }
            RecordMember::Constant(value) => {
                scope
                    .values
                    .insert(value.name, LocalValue::Constant(value.clone()));
                Some(Statement::new(
                    value.span,
                    StatementKind::Constant(value.clone()),
                ))
            }
            RecordMember::TypeAlias(value) => {
                scope.values.insert(value.name, LocalValue::Semantic);
                Some(Statement::new(
                    value.span,
                    StatementKind::TypeAlias(value.clone()),
                ))
            }
            RecordMember::Procedure(value) => {
                scope.values.insert(value.name, LocalValue::Semantic);
                Some(Statement::new(
                    value.span,
                    StatementKind::Procedure(value.clone()),
                ))
            }
            RecordMember::ProcedurePrototype(value) => {
                scope.values.insert(value.name, LocalValue::Semantic);
                Some(Statement::new(
                    value.span,
                    StatementKind::ProcedurePrototype(value.clone()),
                ))
            }
            RecordMember::Record(value) => {
                scope.values.insert(value.name, LocalValue::Semantic);
                Some(Statement::new(
                    value.span,
                    StatementKind::Record((**value).clone()),
                ))
            }
            RecordMember::Enum(value) => {
                scope.values.insert(value.name, LocalValue::Semantic);
                Some(Statement::new(
                    value.span,
                    StatementKind::Enum(value.clone()),
                ))
            }
            RecordMember::Placement(_)
            | RecordMember::Insert(_)
            | RecordMember::DefaultOverride {
                ..
            }
            | RecordMember::Assert {
                ..
            }
            | RecordMember::CompileTimeCases {
                ..
            }
            | RecordMember::Conditional {
                ..
            } => None,
        };
        if let Some(retained) = retained {
            scope.statements.push(retained);
        }
    }
}

/// Fields in an unnamed child have real physical promotion into its parent.
/// Its constant/method namespace still belongs to the actual child record.
fn retain_promoted_field_shadows(scope: &mut LocalScope, members: &[RecordMember]) {
    for member in members {
        match member {
            RecordMember::Field(field) => {
                scope
                    .values
                    .entry(field.name)
                    .or_insert(LocalValue::Semantic);
                if !scope.runtime_names.contains(&field.name) {
                    scope.runtime_names.push(field.name);
                }
                if !scope.field_names.contains(&field.name) {
                    scope.field_names.push(field.name);
                }
            }
            RecordMember::AnonymousRecord(record) => {
                retain_promoted_field_shadows(scope, &record.members)
            }
            _ => {}
        }
    }
}

/// Reserve only possible physical names while a layout guard is unresolved.
/// Neither an inactive field nor a child constant becomes runtime storage.
fn reserve_promoted_field_names(scope: &mut LocalScope, members: &[RecordMember]) {
    for member in members {
        match member {
            RecordMember::Field(field) => {
                scope
                    .values
                    .entry(field.name)
                    .or_insert(LocalValue::Semantic);
                if !scope.field_names.contains(&field.name) {
                    scope.field_names.push(field.name);
                }
            }
            RecordMember::AnonymousRecord(record) => {
                reserve_promoted_field_names(scope, &record.members)
            }
            RecordMember::Conditional {
                then_members,
                else_members,
                ..
            } => {
                reserve_promoted_field_names(scope, then_members);
                reserve_promoted_field_names(scope, else_members);
            }
            RecordMember::CompileTimeCases {
                cases, ..
            } => {
                for body in case_bodies(cases) {
                    reserve_promoted_field_names(scope, body);
                }
            }
            _ => {}
        }
    }
}

fn record_contains_physical_fields(members: &[RecordMember]) -> bool {
    members.iter().any(|member| match member {
        RecordMember::Field(_) => true,
        RecordMember::AnonymousRecord(record) => record_contains_physical_fields(&record.members),
        RecordMember::Conditional {
            then_members,
            else_members,
            ..
        } => {
            record_contains_physical_fields(then_members)
                || record_contains_physical_fields(else_members)
        }
        RecordMember::CompileTimeCases {
            cases, ..
        } => case_bodies(cases).any(record_contains_physical_fields),
        _ => false,
    })
}

fn record_contains_import(members: &[RecordMember]) -> bool {
    members.iter().any(|member| match member {
        RecordMember::Procedure(procedure) => contains_import(&procedure.body),
        RecordMember::Record(record) => record_contains_import(&record.members),
        RecordMember::AnonymousRecord(record) => record_contains_import(&record.members),
        RecordMember::Conditional {
            then_members,
            else_members,
            ..
        } => record_contains_import(then_members) || record_contains_import(else_members),
        RecordMember::CompileTimeCases {
            cases, ..
        } => case_bodies(cases).any(record_contains_import),
        _ => false,
    })
}

fn reserve_conditional_names(
    scope: &mut LocalScope,
    statements: &[Statement],
    discard: Option<Symbol>,
) {
    for statement in statements {
        let name = match &statement.kind {
            StatementKind::Constant(value) => Some(value.name),
            StatementKind::ConstantResults(declaration) => {
                for &(name, _) in &declaration.names {
                    if Some(name) != discard {
                        scope.values.entry(name).or_insert(LocalValue::Semantic);
                    }
                }
                None
            }
            StatementKind::Procedure(value) => Some(value.name),
            StatementKind::ProcedurePrototype(value) => Some(value.name),
            StatementKind::Record(value) => Some(value.name),
            StatementKind::Enum(value) => Some(value.name),
            StatementKind::TypeAlias(value) => Some(value.name),
            StatementKind::Import(value) => {
                if value.namespace.is_none() || value.using {
                    scope.pending_exports = true;
                }
                value.namespace
            }
            StatementKind::UsingDeclaration {
                declaration, ..
            } => {
                reserve_conditional_names(
                    scope,
                    std::slice::from_ref(declaration.as_ref()),
                    discard,
                );
                scope.pending_exports = true;
                None
            }
            StatementKind::Using(_) => {
                scope.pending_exports = true;
                None
            }
            StatementKind::CompileTimeIf {
                then_body,
                else_body,
                ..
            } => {
                reserve_conditional_names(scope, then_body, discard);
                reserve_conditional_names(scope, else_body, discard);
                None
            }
            StatementKind::CompileTimeCases(cases) => {
                for body in case_bodies(cases) {
                    reserve_conditional_names(scope, body, discard);
                }
                None
            }
            _ => None,
        };
        if let Some(name) = name {
            scope.values.entry(name).or_insert(LocalValue::Semantic);
        }
    }
}

fn reserve_record_names(scope: &mut LocalScope, members: &[RecordMember]) {
    for member in members {
        let name = match member {
            RecordMember::AnonymousRecord(record) => {
                reserve_promoted_field_names(scope, &record.members);
                None
            }
            RecordMember::Field(value) => {
                if !scope.field_names.contains(&value.name) {
                    scope.field_names.push(value.name);
                }
                Some(value.name)
            }
            RecordMember::Constant(value) => Some(value.name),
            RecordMember::Procedure(value) => Some(value.name),
            RecordMember::ProcedurePrototype(value) => Some(value.name),
            RecordMember::Record(value) => Some(value.name),
            RecordMember::Enum(value) => Some(value.name),
            RecordMember::TypeAlias(value) => Some(value.name),
            RecordMember::Conditional {
                then_members,
                else_members,
                ..
            } => {
                reserve_record_names(scope, then_members);
                reserve_record_names(scope, else_members);
                None
            }
            RecordMember::CompileTimeCases {
                cases, ..
            } => {
                for body in case_bodies(cases) {
                    reserve_record_names(scope, body);
                }
                None
            }
            _ => None,
        };
        if let Some(name) = name {
            scope.values.entry(name).or_insert(LocalValue::Semantic);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn graph(source: &str, dependency: Option<&str>) -> Result<ModuleGraph, GraphError> {
        let mut provider = SourceOverlay::new();
        provider
            .insert(
                Path::new("/scoped-imports/main.jai"),
                source.as_bytes().to_vec(),
            )
            .unwrap();
        if let Some(source) = dependency {
            provider
                .insert(
                    Path::new("/scoped-imports/dependency.jai"),
                    source.as_bytes().to_vec(),
                )
                .unwrap();
        }
        ModuleGraph::load_with_provider(
            Path::new("/scoped-imports/main.jai"),
            GraphOptions::default(),
            &provider,
        )
    }

    #[test]
    fn local_imports_load_canonical_module_without_exporting_aliases() {
        let graph = graph("first :: () { M :: #import,file \"dependency.jai\"; } second :: () { M :: #import,file \"./dependency.jai\"; }", Some("value :: 7; #scope_module; hidden :: 9;")).unwrap();
        assert_eq!(graph.modules().len(), 2);
        assert_eq!(graph.scoped_imports().len(), 2);
        assert!(graph.imports().is_empty());
        assert_eq!(
            graph.scoped_imports()[0].module(),
            graph.scoped_imports()[1].module()
        );
        let file = graph.module(graph.root()).unwrap().entry();
        assert!(matches!(
            graph.lookup(
                file,
                &NamePath {
                    root: graph.symbols().find("M").unwrap(),
                    members: vec![]
                }
            ),
            Err(LookupError::UnknownName(_))
        ));
        assert!(matches!(
            graph.lookup_module(
                graph.scoped_imports()[0].module(),
                &[graph.symbols().find("hidden").unwrap()]
            ),
            Err(LookupError::PrivateMember { .. })
        ));
    }

    #[test]
    fn inactive_scoped_import_never_resolves_a_missing_module() {
        let graph = graph("main :: () { #if false { Missing :: #import,file \"absent.jai\"; } else { M :: #import,file \"dependency.jai\"; } }", Some("value :: 7;")).unwrap();
        assert_eq!(graph.scoped_imports().len(), 1);
    }

    #[test]
    fn local_constants_keep_scope_and_forward_visibility_during_selection() {
        let graph = graph("guard :: true; main :: () { #if guard { Missing :: #import,file \"absent.jai\"; } guard :: false; { guard :: true; #if guard { M :: #import,file \"dependency.jai\"; } } }", Some("value :: 7;")).unwrap();
        assert_eq!(graph.scoped_imports().len(), 1);
    }

    #[test]
    fn earlier_selected_declarations_control_later_scoped_imports() {
        let graph = graph("main :: () { #if true { selected :: false; } #if selected { Missing :: #import,file \"absent.jai\"; } else { Lib :: #import,file \"dependency.jai\"; } }", Some("value :: 7;")).unwrap();
        assert_eq!(graph.scoped_imports().len(), 1);
    }

    #[test]
    fn nested_record_methods_discover_imports_in_the_record_constant_scope() {
        let graph = graph("enabled :: true; main :: () { Outer :: struct { Inner :: struct { enabled :: false; method :: () { #if enabled { Missing :: #import,file \"absent.jai\"; } else { Lib :: #import,file \"dependency.jai\"; } } } } }", Some("value :: 7;")).unwrap();
        assert_eq!(graph.scoped_imports().len(), 1);
    }

    #[test]
    fn record_conditionals_discover_only_selected_method_imports() {
        let graph = graph("R :: struct { #if true { enabled :: false; method :: () { #if enabled { Missing :: #import \"Absent\"; } else { Lib :: #import,file \"dependency.jai\"; } } } else { wrong :: () { Missing :: #import \"Absent\"; } } } main :: () {}", Some("value :: 7;")).unwrap();
        assert_eq!(graph.scoped_imports().len(), 1);
    }

    #[test]
    fn scalar_short_circuit_does_not_request_unreachable_runtime_guard() {
        let graph = graph("main :: (enabled: bool) { #if false && enabled { Missing :: #import,file \"absent.jai\"; } }", None).unwrap();
        assert!(graph.scoped_imports().is_empty());
    }

    #[test]
    fn loop_runtime_binding_keeps_shadow_identity_in_resumable_condition() {
        let mut provider = SourceOverlay::new();
        provider.insert(Path::new("/scoped-loop/main.jai"), b"iterator :: false; main :: () { for iterator: 0..1 { #if iterator { Missing :: #import \"Absent\"; } } }".to_vec()).unwrap();
        let mut discovery = GraphDiscovery::new(
            Path::new("/scoped-loop/main.jai"),
            GraphOptions::default(),
            &provider,
        )
        .unwrap();
        assert!(!discovery.advance().unwrap().is_complete());
        let condition = discovery.pending_conditions().next().unwrap();
        let DiscoveryConditionContext::Lexical {
            scopes, ..
        } = &condition.context
        else {
            panic!("expected lexical runtime environment")
        };
        let iterator = discovery.graph().symbols().find("iterator").unwrap();
        assert!(
            scopes
                .iter()
                .any(|scope| scope.runtime_names.contains(&iterator))
        );
        let request = condition.id;
        discovery.select_condition(request, false).unwrap();
        assert!(discovery.advance().unwrap().is_complete());
        assert!(discovery.graph().scoped_imports().is_empty());
    }

    #[test]
    fn runtime_guard_reports_lexical_semantic_dependency() {
        let error = graph(
            "main :: (enabled: bool) { #if enabled { Missing :: #import,file \"absent.jai\"; } }",
            None,
        )
        .unwrap_err();
        let GraphError::Pending {
            diagnostic, ..
        } = error
        else {
            panic!("expected pending lexical dependency")
        };
        assert!(diagnostic.message.contains("semantic compile-time"));
        assert_eq!(diagnostic.location.span.text("main :: (enabled: bool) { #if enabled { Missing :: #import,file \"absent.jai\"; } }"), "enabled");
    }

    #[test]
    fn suspended_guard_keeps_discovering_independent_imports_and_restores_loop_scope() {
        let mut provider = SourceOverlay::new();
        let source = "iterator::false; main::(){ for iterator:0..1 { #if #run true { M::#import \"Absent\"; } } #if iterator { Wrong::#import \"Absent\"; } Lib::#import,file \"dependency.jai\"; }";
        provider
            .insert(
                Path::new("/scoped-pending/main.jai"),
                source.as_bytes().to_vec(),
            )
            .unwrap();
        provider
            .insert(
                Path::new("/scoped-pending/dependency.jai"),
                b"value::7;".to_vec(),
            )
            .unwrap();
        let mut discovery = GraphDiscovery::new(
            Path::new("/scoped-pending/main.jai"),
            GraphOptions::default(),
            &provider,
        )
        .unwrap();
        assert!(!discovery.advance().unwrap().is_complete());
        // The loop's runtime iterator remains in its own guard context, while
        // the later guard sees the outer false constant and skips its import.
        assert_eq!(discovery.pending_conditions().count(), 1);
        assert_eq!(discovery.graph().scoped_imports().len(), 1);
        assert!(
            discovery
                .graph()
                .sources()
                .records()
                .iter()
                .any(|source| source.path() == Path::new("/scoped-pending/dependency.jai"))
        );
    }

    #[test]
    fn source_run_guard_waits_for_semantic_execution() {
        let source = "main :: () { #if #run true { Missing :: #import,file \"absent.jai\"; } }";
        let error = graph(source, None).unwrap_err();
        let GraphError::Pending {
            diagnostic, ..
        } = error
        else {
            panic!("expected pending compile-time execution")
        };
        assert!(diagnostic.message.contains("semantic compile-time"));
        assert!(diagnostic.location.span.text(source).contains("#run"));
    }

    #[test]
    fn same_instance_cycles_and_missing_selected_imports_keep_import_location() {
        let source = "main :: () { M :: #import,file \"dependency.jai\"; }";
        let cycle_graph = graph(
            source,
            Some("loop :: () { Root :: #import,file \"main.jai\"; }"),
        )
        .unwrap();
        assert_eq!(cycle_graph.modules().len(), 2);
        assert_eq!(cycle_graph.scoped_imports().len(), 2);
        assert!(
            cycle_graph
                .scoped_imports()
                .iter()
                .any(|edge| edge.module() == cycle_graph.root())
        );
        let error =
            graph("main :: () { #if true { M :: #import \"Absent\"; } }", None).unwrap_err();
        let GraphError::Located {
            diagnostic, ..
        } = error
        else {
            panic!("expected located missing module")
        };
        assert!(diagnostic.message.contains("Absent"));
        assert!(diagnostic.location.span.end > diagnostic.location.span.start);
    }
}
