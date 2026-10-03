//! Convert private capture tokens into validated final-IR structural paths.
use super::*;

impl Capture {
    pub(super) fn publish(
        &mut self,
        sources: &mut DebugSources,
        procedure: ProcedureId,
    ) -> Result<(), &'static str> {
        let mut scopes = HashMap::new();
        let mut statements = HashMap::new();
        let mut pending = Vec::new();
        let mut cleanup_parents = Vec::new();
        if let Some(block) = self.completed_block.take() {
            pending.push((BlockPath::procedure(procedure), block));
        }
        for (id, (parent, block)) in self.cleanups.drain() {
            cleanup_parents.push((id, parent));
            pending.push((BlockPath::cleanup(procedure, id), block));
        }
        while let Some((path, block)) = pending.pop() {
            scopes.insert(block.scope, path.clone());
            if let Some(location) = block.location {
                sources.insert_block(path.clone(), location);
            }
            for (index, statement) in block.statements {
                let statement_path = StatementPath::in_block(&path, index);
                statements.insert(statement.token, statement_path.clone());
                if let Some(location) = statement.location {
                    sources.insert_statement(statement_path.clone(), location.clone());
                    for relative in statement.generated {
                        let mut steps = statement_path.steps.to_vec();
                        steps.extend(relative.iter().copied());
                        sources.insert_statement(
                            StatementPath {
                                root: statement_path.root,
                                steps: steps.into(),
                            },
                            location.clone(),
                        );
                    }
                }
                for (relative, block) in statement.blocks {
                    let mut steps = statement_path.steps.to_vec();
                    steps.extend(relative.iter().copied());
                    pending.push((
                        BlockPath {
                            root: statement_path.root,
                            steps: steps.into(),
                        },
                        block,
                    ));
                }
            }
        }
        for (cleanup, parent) in cleanup_parents {
            let parent = scopes
                .get(&parent)
                .cloned()
                .ok_or("emitted cleanup lost its source lexical scope")?;
            sources.insert_cleanup_parent(procedure, cleanup, parent);
        }
        for (id, local) in self.locals.drain() {
            let scope = scopes
                .get(&local.scope)
                .cloned()
                .ok_or("emitted local lost its source lexical scope")?;
            let declaration = match local.declaration {
                CapturedDeclaration::Parameter(ordinal) => LocalDeclaration::Parameter(ordinal),
                CapturedDeclaration::Statement { token, relative } => {
                    let mut path = statements
                        .get(&token)
                        .cloned()
                        .ok_or("emitted local lost its source declaration statement")?;
                    path.steps = path.steps.iter().copied().chain(relative).collect();
                    LocalDeclaration::Statement(path)
                }
            };
            sources.insert_local(
                id,
                LocalSource {
                    name: local.name,
                    location: local.location,
                    scope,
                    declaration,
                },
            );
        }
        Ok(())
    }
}
