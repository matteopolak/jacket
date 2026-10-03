//! Render collisions from original bindings without changing their identity.
use super::*;

const MAX_COLLISION_ORIGINS: usize = 4;

impl Builder<'_> {
    pub(super) fn binding_collision(
        &self,
        previous: Binding,
        incoming: Binding,
        name: Symbol,
        location: SourceSpan,
    ) -> GraphError {
        let mut message = format!(
            "conflicting declaration or import '{}'",
            self.graph.symbols.name(name)
        );
        self.append_binding_origins(&mut message, previous, "existing");
        self.append_binding_origins(&mut message, incoming, "incoming");
        self.located(location, message)
    }

    fn append_binding_origins(&self, message: &mut String, binding: Binding, side: &str) {
        match binding {
            Binding::Declaration(id) => {
                if let Some(declaration) = self.graph.declaration(id) {
                    self.append_origin(message, declaration.syntax().location, side, "declaration");
                }
            }
            Binding::OverloadSet(id) => {
                if let Some(group) = self.graph.overload_set(id) {
                    for id in group.declarations().iter().take(MAX_COLLISION_ORIGINS) {
                        if let Some(declaration) = self.graph.declaration(*id) {
                            self.append_origin(
                                message,
                                declaration.syntax().location,
                                side,
                                "overload declaration",
                            );
                        }
                    }
                    if group.declarations().len() > MAX_COLLISION_ORIGINS {
                        message.push_str(&format!(
                            "\nnote: {} additional {side} overload declarations",
                            group.declarations().len() - MAX_COLLISION_ORIGINS
                        ));
                    }
                }
            }
            Binding::Parameter(id) => {
                if let Some(parameter) = self.graph.parameter(id) {
                    self.append_origin(message, parameter.location, side, "parameter");
                }
            }
            Binding::StorageMember(id) => {
                if let Some(member) = self.graph.source_storage_member(id) {
                    self.append_origin(message, member.location(), side, "storage member");
                }
            }
            Binding::SourceMember {
                declaration, ..
            } => {
                if let Some(declaration) = self.graph.declaration(declaration) {
                    self.append_origin(
                        message,
                        declaration.syntax().location,
                        side,
                        "namespace owner",
                    );
                }
            }
            // A namespace binding is an import link, not a source declaration.
            Binding::Module(_) => {}
        }
    }

    fn append_origin(&self, message: &mut String, location: SourceSpan, side: &str, kind: &str) {
        let Some(source) = self.graph.sources().get(location.source) else {
            return;
        };
        let text = source.text();
        let at = text.floor_char_boundary(location.span.start.min(text.len()));
        let prefix = &text[..at];
        let line = prefix.bytes().filter(|&byte| byte == b'\n').count() + 1;
        let column = prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1;
        message.push_str(&format!(
            "\n{}:{line}:{column}: note: {side} {kind}",
            source.path().display()
        ));
    }
}
