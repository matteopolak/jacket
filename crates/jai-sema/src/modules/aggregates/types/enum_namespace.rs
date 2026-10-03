//! Infer an enum member domain from its actual defining source before values are ready.
use super::*;
impl Nominals<'_> {
    pub(in crate::modules) fn source_enum_member_type(
        &self,
        graph: &ModuleGraph,
        file: FileInstanceId,
        path: &NamePath,
        span: Span,
    ) -> Result<Option<TypeId>, Diagnostic> {
        let Some((&member, parents)) = path.members.split_last() else {
            return Ok(None);
        };
        let mut parent = NamePath {
            root: path.root,
            members: parents.to_vec(),
        };
        let mut file = file;
        let mut visited = HashSet::new();
        for _ in 0..256 {
            let Ok(jai_modules::Binding::Declaration(id)) = graph.lookup(file, &parent) else {
                return Ok(None);
            };
            if !visited.insert(id) {
                return Err(Diagnostic::new(
                    span,
                    "cyclic enum source namespace aliases",
                ));
            }
            let source = graph
                .declaration(id)
                .expect("enum namespace lookup retains declaration");
            match &source.syntax().kind {
                FileDeclarationKind::Enum(enumeration) => {
                    if !enumeration
                        .members
                        .iter()
                        .any(|source| source.name == member)
                    {
                        return Err(Diagnostic::new(span, "unknown enum member"));
                    }
                    return self
                        .declarations
                        .get(&id)
                        .copied()
                        .map(Some)
                        .ok_or_else(|| {
                            Diagnostic::new(span, "enum source has no canonical reservation")
                        });
                }
                FileDeclarationKind::TypeAlias(alias) => {
                    let TypeSyntax::Named(path) = &alias.ty else {
                        return Ok(None);
                    };
                    parent = path.clone();
                    file = source.file();
                }
                FileDeclarationKind::Constant(constant) if constant.ty.is_none() => {
                    let Some(TypeSyntax::Named(path)) = expression_type(&constant.initializer)
                    else {
                        return Ok(None);
                    };
                    parent = path;
                    file = source.file();
                }
                _ => return Ok(None),
            }
        }
        Err(Diagnostic::new(
            span,
            "enum source namespace exceeds alias depth limit",
        ))
    }
}
