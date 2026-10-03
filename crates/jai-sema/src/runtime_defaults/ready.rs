//! Pure proofs over already resolved storage; no declaration is completed here.
use super::*;

enum ReadPath {
    Named(syntax::NamePath),
    Context(Vec<Symbol>),
}

impl Resolver<'_> {
    pub(crate) fn ready_runtime_parameter_default(
        &self,
        expression: &syntax::Expression,
        expected: Option<TypeId>,
    ) -> Result<Option<RuntimeDefaultRead>, Diagnostic> {
        let Some(path) = read_path(expression, self.symbols)? else {
            return Ok(None);
        };
        let span = expression.span;
        let path = match path {
            ReadPath::Named(path) => path,
            ReadPath::Context(members) => {
                return self.ready_context_default(&members, expected, span);
            }
        };
        if let Some(binding) = self.ready_local_default_binding(path.root, span)? {
            return self.ready_default_binding(binding, &path.members, expected, span);
        }
        if let Some(scope) = self.graph_scope {
            if let Ok((binding, members)) = scope.value_root(&path, span) {
                return self.ready_default_binding(binding, &members, expected, span);
            }
            let root = syntax::NamePath {
                root: path.root,
                members: Vec::new(),
            };
            if !scope.type_value_name_absent(&root) {
                return Ok(None);
            }
        } else if let Some(binding) = self.globals.get(&path.root) {
            return self.ready_default_binding(binding.clone(), &path.members, expected, span);
        }
        if self.symbols.name(path.root) != "context" {
            return Ok(None);
        }
        self.ready_context_default(&path.members, expected, span)
    }

    fn ready_context_default(
        &self,
        members: &[Symbol],
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Option<RuntimeDefaultRead>, Diagnostic> {
        let schema = self.context.ok_or_else(|| {
            Diagnostic::new(span, "runtime default requires a ready Context schema")
        })?;
        let place = Place::context(schema.definition.record_type, self.types)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        self.runtime_default_from_ready_storage(place, members, expected, span)
            .map(Some)
    }

    fn ready_default_binding(
        &self,
        binding: Binding,
        members: &[Symbol],
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Option<RuntimeDefaultRead>, Diagnostic> {
        match binding {
            Binding::Storage(storage) => self
                .runtime_default_from_ready_storage(storage.place(), members, expected, span)
                .map(Some),
            Binding::Namespace(module)
            | Binding::Imported(jai_modules::Binding::Module(module)) => {
                let scope = self.graph_scope.ok_or_else(|| {
                    Diagnostic::new(span, "runtime default namespace requires its source graph")
                })?;
                for count in (1..=members.len()).rev() {
                    if let Ok(binding) = scope.namespace_binding(module, &members[..count], span) {
                        return self.ready_default_binding(
                            scope.imported_value(binding, span)?,
                            &members[count..],
                            expected,
                            span,
                        );
                    }
                }
                Ok(None)
            }
            Binding::Imported(binding) => {
                let scope = self.graph_scope.ok_or_else(|| {
                    Diagnostic::new(span, "runtime default import requires its source graph")
                })?;
                let Some((place, mut path)) = scope
                    .runtime_default_imported_storage(binding, self.ast_source_location(span)?)?
                else {
                    return Ok(None);
                };
                path.extend_from_slice(members);
                self.runtime_default_from_ready_storage(place, &path, expected, span)
                    .map(Some)
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn runtime_default_from_ready_storage(
        &self,
        mut place: Place,
        members: &[Symbol],
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<RuntimeDefaultRead, Diagnostic> {
        let original_type = place.ty();
        let mut reversed = Vec::new();
        let root = loop {
            if reversed.len() >= 256 {
                return Err(Diagnostic::new(
                    span,
                    "runtime default exceeds storage path depth limit",
                ));
            }
            match place.kind() {
                jai_ir::PlaceKind::Global(_) => {
                    let scope = self.graph_scope.ok_or_else(|| {
                        Diagnostic::new(span, "runtime default requires its retained source graph")
                    })?;
                    break DefaultReadRoot::Global {
                        declaration: scope.runtime_default_owner(place, span)?,
                        ty: place.ty(),
                    };
                }
                jai_ir::PlaceKind::Context(ty) => {
                    break DefaultReadRoot::Context {
                        ty,
                    };
                }
                jai_ir::PlaceKind::Field(id) => {
                    let projection = self
                        .places
                        .projection(id)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                    reversed.push(DefaultReadStep::Field(projection.field));
                    place = projection.base;
                }
                jai_ir::PlaceKind::Dereference(id) => {
                    let projection = self
                        .places
                        .dereference_projection(id)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                    reversed.push(DefaultReadStep::Dereference(place.ty()));
                    place = loaded_place(&projection.pointer).ok_or_else(|| Diagnostic::new(span, "runtime default pointer requires a ready global or context storage read"))?;
                }
                jai_ir::PlaceKind::Local(_) => {
                    return Err(Diagnostic::new(
                        span,
                        "runtime parameter default cannot capture local storage",
                    ));
                }
                _ => {
                    return Err(Diagnostic::new(
                        span,
                        "runtime default requires record or pointer storage projections",
                    ));
                }
            }
        };
        reversed.reverse();
        let mut ty = original_type;
        for &member in members {
            while let jai_types::TypeKind::Pointer(pointee) = *self
                .types
                .kind(ty)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
            {
                if reversed.len() >= 256 {
                    return Err(Diagnostic::new(
                        span,
                        "runtime default exceeds storage path depth limit",
                    ));
                }
                reversed.push(DefaultReadStep::Dereference(pointee));
                ty = pointee;
            }
            let fields = self.field_path(ty, member, span)?;
            if reversed.len().saturating_add(fields.len()) > 256 {
                return Err(Diagnostic::new(
                    span,
                    "runtime default exceeds storage path depth limit",
                ));
            }
            for field in fields {
                ty = self
                    .types
                    .validate_field(ty, field)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                reversed.push(DefaultReadStep::Field(field));
            }
        }
        if expected.is_some_and(|expected| expected != ty) {
            return Err(Diagnostic::new(
                span,
                "runtime default differs from its declared parameter type",
            ));
        }
        RuntimeDefaultRead::checked(
            root,
            reversed,
            ty,
            self.ast_source_location(span)?,
            self.types,
        )
    }
}

fn read_path(
    expression: &syntax::Expression,
    symbols: &Symbols,
) -> Result<Option<ReadPath>, Diagnostic> {
    let mut current = expression;
    let mut members = Vec::new();
    let mut path = loop {
        if members.len() >= 256 {
            return Err(Diagnostic::new(
                expression.span,
                "runtime default exceeds storage path depth limit",
            ));
        }
        match &current.kind {
            syntax::ExpressionKind::Member {
                base,
                member,
            } => {
                members.push(*member);
                current = base;
            }
            syntax::ExpressionKind::Name(root) => {
                break syntax::NamePath {
                    root: *root,
                    members: Vec::new(),
                };
            }
            syntax::ExpressionKind::QualifiedName(path) => break path.clone(),
            syntax::ExpressionKind::Context => {
                let Some(root) = symbols.find("context") else {
                    return Ok(Some(ReadPath::Context(members.into_iter().rev().collect())));
                };
                break syntax::NamePath {
                    root,
                    members: Vec::new(),
                };
            }
            _ => return Ok(None),
        }
    };
    if path.members.len().saturating_add(members.len()) > 256 {
        return Err(Diagnostic::new(
            expression.span,
            "runtime default exceeds storage path depth limit",
        ));
    }
    path.members.extend(members.into_iter().rev());
    Ok(Some(ReadPath::Named(path)))
}

fn loaded_place(mut value: &ValueExpr) -> Option<Place> {
    for _ in 0..256 {
        match value {
            ValueExpr::Load(place) => return Some(*place),
            ValueExpr::Int(integer) => match integer.kind() {
                IntExprKind::Load(place) => return Some(place.place()),
                IntExprKind::Value(next) => value = next,
                _ => return None,
            },
            ValueExpr::Bool(BoolExpr::Load(place)) => return Some(place.place()),
            ValueExpr::Bool(BoolExpr::Value(next)) => value = next,
            ValueExpr::Float(float) => match float.kind() {
                FloatExprKind::Load(place) => return Some(*place),
                FloatExprKind::Value(next) => value = next,
                _ => return None,
            },
            _ => return None,
        }
    }
    None
}
