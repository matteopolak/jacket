//! Discovery decisions retain source bindings while the graph interns final names.
use super::*;
use jai_modules::{
    FileUsingDecision, UsingAlias, UsingBinding, UsingPlaceholder, UsingStorageMember,
};

struct UsingSource {
    module: Option<jai_source::ModuleId>,
    storage: Option<(jai_source::DeclarationId, Vec<Symbol>)>,
    owner: Option<jai_source::DeclarationId>,
}

impl Resolver<'_> {
    pub(crate) fn using_source_decision(
        &mut self,
        directive: &UsingDirective,
        lexical: bool,
    ) -> Result<FileUsingDecision, Diagnostic> {
        let span = directive.span;
        let path = expression_path(&directive.target);
        let source_module = if let Some(path) = path.as_ref() {
            let local = self.resolve_local_name(path.root, span)?;
            let root = match local {
                Some(Binding::Namespace(module)) => Some(GraphBinding::Module(module)),
                Some(Binding::Imported(binding)) => Some(binding),
                Some(_) => None,
                None => self
                    .graph_scope
                    .and_then(|scope| scope.using_graph_binding(path)),
            };
            match root {
                Some(GraphBinding::Module(module))
                    if local.is_some() && !path.members.is_empty() =>
                {
                    self.graph_scope
                        .and_then(|scope| scope.namespace_binding(module, &path.members, span).ok())
                        .and_then(|binding| {
                            if let GraphBinding::Module(module) = binding {
                                Some(module)
                            } else {
                                None
                            }
                        })
                }
                Some(GraphBinding::Module(module)) => Some(module),
                _ => None,
            }
        } else {
            None
        };
        let storage_source = if !lexical {
            path.as_ref().and_then(|path| {
                self.graph_scope
                    .and_then(|scope| scope.using_storage_source(path))
            })
        } else {
            None
        };
        let source_owner = if source_module.is_none() {
            match self.expr(&directive.target) {
                Ok(Expr::Type(ty)) => self
                    .graph_scope
                    .and_then(|scope| scope.using_source_owner(ty)),
                Ok(value) if storage_source.is_some() => {
                    self.expression_type(&value, span).ok().and_then(|ty| {
                        self.graph_scope
                            .and_then(|scope| scope.using_source_owner(ty))
                    })
                }
                _ => None,
            }
        } else {
            None
        };
        let (members, _) = self.using_candidates(&directive.target)?;
        self.using_source_members_decision(
            directive,
            lexical,
            UsingSource {
                module: source_module,
                storage: storage_source,
                owner: source_owner,
            },
            members,
        )
    }

    /// A discarded nominal child is identified by its original declaration,
    /// without resolving or publishing the discard spelling as a name.
    pub(crate) fn using_source_declaration_decision(
        &mut self,
        directive: &UsingDirective,
        declaration: jai_source::DeclarationId,
    ) -> Result<FileUsingDecision, Diagnostic> {
        let span = directive.span;
        let Binding::Type(ty) =
            self.imported_binding_value(GraphBinding::Declaration(declaration), span)?
        else {
            return Err(Diagnostic::new(
                span,
                "discarded file using requires a nominal type declaration",
            ));
        };
        let members = self
            .using_type_members(ty, span)?
            .into_iter()
            .map(|(name, binding)| UsingMember::named(self.symbols, name, binding))
            .collect();
        let owner = self
            .graph_scope
            .and_then(|scope| scope.using_source_owner(ty))
            .ok_or_else(|| {
                Diagnostic::new(span, "discarded using has no canonical nominal owner")
            })?;
        self.using_source_members_decision(
            directive,
            false,
            UsingSource {
                module: None,
                storage: None,
                owner: Some(owner),
            },
            members,
        )
    }

    fn using_source_members_decision(
        &mut self,
        directive: &UsingDirective,
        lexical: bool,
        source: UsingSource,
        members: Vec<UsingMember>,
    ) -> Result<FileUsingDecision, Diagnostic> {
        let span = directive.span;
        let names = members
            .iter()
            .map(|member| member.name.clone())
            .collect::<Vec<_>>();
        let selected = self.using_selected_names(&directive.selection, &names, span)?;
        let mut decision = FileUsingDecision {
            source_module: source.module,
            ..Default::default()
        };
        let mut seen = HashSet::new();
        for (member, selected) in members.into_iter().zip(selected) {
            let Some(selected) = selected else {
                continue;
            };
            if !seen.insert(selected.clone()) {
                return Err(Diagnostic::new(span, "using produces duplicate names"));
            }
            match member.binding {
                UsingMemberBinding::Placeholder(placeholder) => {
                    let name = String::from_utf8(selected).map_err(|_| {
                        Diagnostic::new(span, "using mapped name must be valid UTF-8")
                    })?;
                    decision
                        .placeholders
                        .push(UsingPlaceholder { name, placeholder });
                }
                UsingMemberBinding::Operators(declarations) if selected == member.name => {
                    decision.selected_operator_declarations.extend(declarations)
                }
                UsingMemberBinding::Operators(_) => {
                    return Err(Diagnostic::new(
                        span,
                        "using map cannot rename an operator token",
                    ));
                }
                UsingMemberBinding::Named(binding) => {
                    let name = String::from_utf8(selected).map_err(|_| {
                        Diagnostic::new(span, "using mapped name must be valid UTF-8")
                    })?;
                    if let Binding::Storage(_) = &binding
                        && let Some((owner, prefix)) = &source.storage
                    {
                        let original = std::str::from_utf8(&member.name).map_err(|_| {
                            Diagnostic::new(span, "using source name must be valid UTF-8")
                        })?;
                        let source = self.symbols.find(original).ok_or_else(|| {
                            Diagnostic::new(span, "using source field has no canonical spelling")
                        })?;
                        let target = self.expression_place(&directive.target)?;
                        if matches!(
                            self.types.kind(target.ty()),
                            Ok(jai_types::TypeKind::Pointer(_))
                        ) {
                            return Err(Diagnostic::new(
                                span,
                                "file using through mutable pointer storage requires an initialized pointer capture",
                            ));
                        }
                        // Resolve the real canonical field before transporting source names.
                        self.member_place(target, source, span)?;
                        let mut fields = prefix.clone();
                        fields.push(source);
                        decision.storage_members.push(UsingStorageMember {
                            owner: *owner,
                            path: fields,
                            destination: name,
                        });
                        continue;
                    }
                    let binding = match binding {
                        Binding::Imported(binding) => Some(binding),
                        _ => source.owner.and_then(|declaration| {
                            let original = std::str::from_utf8(&member.name).ok()?;
                            let member = self.symbols.find(original)?;
                            Some(GraphBinding::SourceMember {
                                declaration,
                                member,
                            })
                        }),
                    };
                    if let Some(binding) = binding {
                        decision.bindings.push(UsingBinding { name, binding });
                    } else if lexical {
                        let original = std::str::from_utf8(&member.name).map_err(|_| {
                            Diagnostic::new(span, "using source name must be valid UTF-8")
                        })?;
                        let source = self.symbols.find(original).ok_or_else(|| {
                            Diagnostic::new(span, "using source member has no canonical spelling")
                        })?;
                        decision.aliases.push(UsingAlias {
                            source,
                            destination: name,
                        });
                    } else {
                        return Err(Diagnostic::new(
                            span,
                            "file using requires a source namespace or nominal static member",
                        ));
                    }
                }
            }
        }
        Ok(decision)
    }
}
