//! Lexical promotion retains declaration identities and aliases actual storage.
use super::*;
use jai_modules::Binding as GraphBinding;
use jai_syntax::NamePath;
use jai_types::TypeKind;
use std::collections::HashSet;
use syntax::{UsingDirective, UsingNames, UsingSelection};
mod discovery;
mod evaluation;
mod operators;
mod selection;

pub(crate) struct UsingEnvironment {
    pub(crate) bindings: Vec<(Symbol, Binding)>,
    pub(crate) operator_declarations: Vec<jai_source::DeclarationId>,
}

struct UsingMember {
    name: Vec<u8>,
    binding: UsingMemberBinding,
}

enum UsingMemberBinding {
    Named(Binding),
    Operators(Vec<jai_source::DeclarationId>),
    Placeholder(jai_modules::PlaceholderId),
}

impl UsingMember {
    fn named(symbols: &Symbols, name: Symbol, binding: Binding) -> Self {
        Self {
            name: symbols.name(name).as_bytes().to_vec(),
            binding: UsingMemberBinding::Named(binding),
        }
    }
}

#[derive(Default)]
pub(crate) struct UsingState {
    readonly: HashSet<Place>,
}

impl Resolver<'_> {
    pub(crate) fn using_directive(
        &mut self,
        directive: &UsingDirective,
    ) -> Result<Statement, Diagnostic> {
        let span = directive.span;
        if let Some(publication) = self.checked_using_publication(span) {
            return self.apply_using_publication(directive, &publication);
        }
        if matches!(
            directive.selection,
            UsingSelection::Map(_)
                | UsingSelection::Only(UsingNames::Expression(_))
                | UsingSelection::Except(UsingNames::Expression(_))
        ) {
            return Err(Diagnostic::new(
                span,
                "computed using requires its checked source publication",
            ));
        }
        let (mut candidates, setup) = self.using_candidates(&directive.target)?;
        let names = candidates
            .iter()
            .map(|member| member.name.clone())
            .collect::<Vec<_>>();
        let selected = self.using_selected_names(&directive.selection, &names, span)?;
        let mut environment = UsingEnvironment {
            bindings: vec![],
            operator_declarations: vec![],
        };
        let mut renamed = HashSet::new();
        let mut placeholders = vec![];
        for (member, name) in candidates.drain(..).zip(selected) {
            let Some(name) = name else { continue };
            if !renamed.insert(name.clone()) {
                return Err(Diagnostic::new(span, "using produces duplicate names"));
            }
            match member.binding {
                UsingMemberBinding::Placeholder(marker) => {
                    let spelling = std::str::from_utf8(&name).map_err(|_| {
                        Diagnostic::new(span, "using mapped name must be valid UTF-8")
                    })?;
                    let name = self.symbols.find(spelling).ok_or_else(|| {
                        Diagnostic::new(span, "using placeholder name awaits source publication")
                    })?;
                    placeholders.push((name, marker));
                }
                UsingMemberBinding::Named(binding) => {
                    let spelling = std::str::from_utf8(&name).map_err(|_| {
                        Diagnostic::new(span, "using mapped name must be valid UTF-8")
                    })?;
                    let name = self.symbols.find(spelling).ok_or_else(|| {
                        Diagnostic::new(
                            span,
                            format!(
                                "using mapped name '{spelling}' awaits source name publication"
                            ),
                        )
                    })?;
                    environment.bindings.push((name, binding));
                }
                UsingMemberBinding::Operators(declarations) if name == member.name => {
                    environment.operator_declarations.extend(declarations);
                }
                UsingMemberBinding::Operators(_) => {
                    return Err(Diagnostic::new(
                        span,
                        "using map cannot rename an operator token",
                    ));
                }
            }
        }
        self.bind_checked_using_environment(
            span,
            environment.bindings,
            environment.operator_declarations,
            matches!(directive.selection, UsingSelection::All),
        )?;
        self.bind_checked_using_placeholders(
            span,
            &placeholders,
            matches!(directive.selection, UsingSelection::All),
        )?;
        Ok(Statement::Block(Block {
            statements: setup,
            flow: Flow::FallsThrough,
        }))
    }

    fn apply_using_publication(
        &mut self,
        directive: &UsingDirective,
        publication: &jai_modules::UsingPublication,
    ) -> Result<Statement, Diagnostic> {
        let span = directive.span;
        let (candidates, setup) = self.using_candidates(&directive.target)?;
        let mut bindings =
            Vec::with_capacity(publication.bindings.len() + publication.aliases.len());
        for &(name, binding) in &publication.bindings {
            let value = if matches!(binding, GraphBinding::StorageMember(_)) {
                self.imported_binding_value(binding, span)?
            } else {
                Binding::Imported(binding)
            };
            bindings.push((name, value));
        }
        for &(source, destination) in &publication.aliases {
            let source = self.symbols.get(source).ok_or_else(|| {
                Diagnostic::new(span, "using publication has an unavailable source name")
            })?;
            let member = candidates
                .iter()
                .find(|member| member.name == source.as_bytes())
                .ok_or_else(|| {
                    Diagnostic::new(
                        span,
                        "using publication source member is unavailable in its defining target",
                    )
                })?;
            let UsingMemberBinding::Named(binding) = &member.binding else {
                return Err(Diagnostic::new(
                    span,
                    "using place alias cannot refer to an operator token",
                ));
            };
            bindings.push((destination, binding.clone()));
        }
        self.bind_checked_using_environment(
            span,
            bindings,
            publication.selected_operator_declarations.clone(),
            matches!(directive.selection, UsingSelection::All),
        )?;
        self.bind_checked_using_placeholders(
            span,
            &publication.placeholders,
            matches!(directive.selection, UsingSelection::All),
        )?;
        Ok(Statement::Block(Block {
            statements: setup,
            flow: Flow::FallsThrough,
        }))
    }

    fn using_candidates(
        &mut self,
        target: &syntax::Expression,
    ) -> Result<(Vec<UsingMember>, Vec<Statement>), Diagnostic> {
        let span = target.span;
        if let Some(path) = expression_path(target) {
            let local = self.resolve_local_name(path.root, span)?;
            let graph = match local {
                Some(Binding::Namespace(module)) => Some(GraphBinding::Module(module)),
                Some(Binding::Imported(binding)) => Some(binding),
                Some(_) => None,
                None => self
                    .graph_scope
                    .and_then(|scope| scope.using_graph_binding(&path)),
            };
            if let Some(GraphBinding::Module(module)) = graph {
                let scope = self.graph_scope.ok_or_else(|| {
                    Diagnostic::new(span, "using a module requires a source graph")
                })?;
                let binding = if path.members.is_empty() || local.is_none() {
                    GraphBinding::Module(module)
                } else {
                    scope.namespace_binding(module, &path.members, span)?
                };
                if let GraphBinding::Module(module) = binding {
                    return Ok((self.using_module_members(module, span)?, vec![]));
                }
            }
        }
        let addressable = self.expression_place(target).ok();
        let (base, setup, readonly) = if let Some(place) = addressable {
            let ty = place.ty();
            if matches!(self.types.kind(ty), Ok(TypeKind::Pointer(_))) {
                let pointer = self.allocate_typed(ty)?;
                let setup = vec![Statement::Store(pointer.place(), ValueExpr::Load(place))];
                let base = self
                    .places
                    .dereference(ValueExpr::Load(pointer.place()), self.types)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                (base, setup, false)
            } else {
                let readonly = self.iteration_readonly_owner(place, span)?.is_some();
                let pointer_ty = self
                    .types
                    .pointer(ty)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                let pointer = self.allocate_typed(pointer_ty)?;
                let setup = vec![Statement::Store(
                    pointer.place(),
                    ValueExpr::AddressOf {
                        ty: pointer_ty,
                        place,
                    },
                )];
                let base = self
                    .places
                    .dereference(ValueExpr::Load(pointer.place()), self.types)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                (base, setup, readonly)
            }
        } else {
            let value = self.expr(target)?;
            if let Expr::Type(ty) = value {
                let members = self
                    .using_type_members(ty, span)?
                    .into_iter()
                    .map(|(name, binding)| UsingMember::named(self.symbols, name, binding))
                    .collect();
                return Ok((members, vec![]));
            }
            let ty = self.expression_type(&value, span)?;
            let pointer = matches!(self.types.kind(ty), Ok(TypeKind::Pointer(_)));
            let local = self.allocate_typed(ty)?;
            let value = self.coerce_value(value, ty, span)?;
            let setup = vec![Statement::Store(local.place(), value)];
            if pointer {
                let base = self
                    .places
                    .dereference(ValueExpr::Load(local.place()), self.types)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                (base, setup, false)
            } else {
                (local.place(), setup, true)
            }
        };
        if readonly {
            self.meta.using_names.readonly.insert(base);
        }
        let mut candidates = self
            .using_type_members(base.ty(), span)?
            .into_iter()
            .map(|(name, binding)| UsingMember::named(self.symbols, name, binding))
            .collect::<Vec<_>>();
        let mut pending = vec![(base.ty(), Vec::new(), HashSet::new())];
        let mut seen = candidates
            .iter()
            .map(|member| member.name.clone())
            .collect::<HashSet<_>>();
        while let Some((ty, prefix, mut ancestors)) = pending.pop() {
            if !ancestors.insert(ty) {
                return Err(Diagnostic::new(span, "cyclic using field promotion"));
            }
            let record = self.record_metadata(ty, span)?;
            for field in record.fields {
                let mut path = prefix.clone();
                path.push(field.id);
                if let Some(name) = field.name {
                    if !seen.insert(self.symbols.name(name).as_bytes().to_vec()) {
                        return Err(Diagnostic::new(span, "ambiguous using member"));
                    }
                    let mut place = base;
                    for id in &path {
                        place = self
                            .places
                            .field(place, *id, self.types)
                            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                    }
                    let storage = Storage::from_place(place, self.types)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                    candidates.push(UsingMember::named(
                        self.symbols,
                        name,
                        Binding::Storage(storage),
                    ));
                }
                if field.syntax.using() {
                    pending.push((field.ty, path, ancestors.clone()));
                }
            }
        }
        Ok((candidates, setup))
    }

    pub(crate) fn reject_using_write(
        &self,
        mut place: Place,
        span: Span,
    ) -> Result<(), Diagnostic> {
        loop {
            if self.meta.using_names.readonly.contains(&place) {
                return Err(Diagnostic::new(
                    span,
                    "using a temporary or read-only value exposes read-only fields",
                ));
            }
            place = match place.kind() {
                PlaceKind::Field(id) => {
                    self.places.projection(id).map(|projection| projection.base)
                }
                PlaceKind::Index(id) => self
                    .places
                    .index_projection(id)
                    .map(|projection| projection.base),
                PlaceKind::SequenceField(id) => self
                    .places
                    .sequence_projection(id)
                    .map(|projection| projection.base),
                _ => return Ok(()),
            }
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        }
    }
}

fn expression_path(expression: &syntax::Expression) -> Option<NamePath> {
    match &expression.kind {
        syntax::ExpressionKind::Name(name) => Some(NamePath {
            root: *name,
            members: vec![],
        }),
        syntax::ExpressionKind::QualifiedName(path) => Some(path.clone()),
        _ => None,
    }
}
