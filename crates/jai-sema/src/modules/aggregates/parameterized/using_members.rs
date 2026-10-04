//! Original selected record directives publish actual enum namespace bindings.
use super::*;
use jai_source::Symbol;
impl<F> TypeResolver<'_, '_, F>
where
    F: FnMut(FileInstanceId, &syntax::Expression) -> Result<ScalarConstant, LocatedDiagnostic>,
{
    pub(super) fn promote_source_using_members(
        &mut self,
        file: FileInstanceId,
        record: RecordBody<'_>,
        scope: &mut Substitution,
        names: &mut HashSet<Symbol>,
        source_members: &mut Vec<Symbol>,
    ) -> TypeResult<()> {
        for member in record.members {
            let syntax::RecordMember::Using(directive) = member else {
                continue;
            };
            let target = crate::record_using::target_names(&directive.target)
                .map_err(|e| failure(self.graph, file, e))?;
            if record.members.iter().any(|member| matches!(member, syntax::RecordMember::Field(field) if field.name == target[0])) { continue; }
            let annotation = type_expression(&directive.target).ok_or_else(|| {
                failure(
                    self.graph,
                    file,
                    Diagnostic::new(
                        directive.target.span,
                        "record using namespace requires original type syntax",
                    ),
                )
            })?;
            let ty = self.resolve(file, &annotation, Some(scope), directive.target.span)?;
            let mut values = self.records.member_enum(ty).map(|enumeration| enumeration.values.clone()).or_else(||self.nominals.enums.get(&ty).map(|enumeration| enumeration.members.iter().map(|(&name,&value)|(name,value)).collect::<Vec<_>>())).ok_or_else(||failure(self.graph,file,Diagnostic::new(directive.span,"record namespace using requires a ready canonical enum; record namespace export requires checked member recipes")))?;
            if values.len().saturating_add(source_members.len()) > 65_536 {
                return Err(failure(
                    self.graph,
                    file,
                    Diagnostic::new(
                        directive.span,
                        "record using namespace exceeds compiler declaration budget",
                    ),
                ));
            }
            values.sort_by(|a, b| {
                self.graph
                    .symbols()
                    .name(a.0)
                    .cmp(self.graph.symbols().name(b.0))
            });
            for (name, value) in values {
                if !crate::record_using::selected(&directive.selection, name, directive.span)
                    .map_err(|e| failure(self.graph, file, e))?
                {
                    continue;
                }
                if !names.insert(name) {
                    return Err(failure(
                        self.graph,
                        file,
                        Diagnostic::new(
                            directive.span,
                            "ambiguous promoted record namespace member",
                        ),
                    ));
                }
                super::members::shadow(
                    scope,
                    name,
                    BakedValue::Value(jai_ir::ConstantValue {
                        ty,
                        kind: jai_ir::ConstantKind::Enum(value),
                    }),
                );
                source_members.push(name);
            }
        }
        Ok(())
    }
}
