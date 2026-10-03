//! Retained record aliases associate arguments against original formal identities.
use super::*;
use jai_source::SourceSpan;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RecordFormalId {
    pub(crate) declaration: DeclarationId,
    pub(crate) ordinal: usize,
}

/// A partial binding is immutable typed data from the alias's defining scope.
/// Its formal still belongs to the full originating record declaration.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct PartialRecordArgument {
    pub(crate) formal: RecordFormalId,
    pub(crate) value: BakedValue,
    pub(crate) location: SourceSpan,
}

#[derive(Clone)]
pub(crate) struct PartialRecordApplication {
    pub(crate) declaration: DeclarationId,
    pub(crate) file: FileInstanceId,
    pub(crate) location: SourceSpan,
    pub(crate) arguments: Vec<PartialRecordArgument>,
}

#[derive(Clone, Copy)]
pub(crate) enum CompletedRecordArgument<'a> {
    Baked(&'a BakedValue),
    Source {
        expression: &'a syntax::Expression,
        defaulted: bool,
    },
}

impl PartialRecordApplication {
    pub(crate) fn complete_arguments<'a>(
        &'a self,
        parameters: &'a [syntax::RecordParameter],
        supplied: &'a [syntax::CallArgument],
        location: SourceSpan,
    ) -> Result<Vec<(RecordFormalId, CompletedRecordArgument<'a>)>, Diagnostic> {
        let mut baked = vec![None; parameters.len()];
        for argument in &self.arguments {
            if argument.formal.declaration != self.declaration {
                return Err(Diagnostic::at_source(
                    argument.location,
                    "partial record argument belongs to another source declaration",
                ));
            }
            let Some(slot) = baked.get_mut(argument.formal.ordinal) else {
                return Err(Diagnostic::at_source(
                    argument.location,
                    "partial record argument has no original source formal",
                ));
            };
            if slot.replace(&argument.value).is_some() {
                return Err(Diagnostic::at_source(
                    argument.location,
                    "duplicate partial record argument",
                ));
            }
        }
        let remaining = baked
            .iter()
            .enumerate()
            .filter_map(|(index, value)| value.is_none().then_some(index))
            .collect::<Vec<_>>();
        let mut explicit = vec![None; parameters.len()];
        let mut positional = 0;
        let mut named = false;
        for argument in supplied {
            let span = argument.value.span;
            if argument.spread {
                return Err(Diagnostic::at_source(
                    SourceSpan {
                        span,
                        ..location
                    },
                    "partial record arguments cannot spread a runtime pack",
                ));
            }
            let formal = match argument.name {
                Some(name) => {
                    named = true;
                    parameters
                        .iter()
                        .position(|parameter| parameter.name == name)
                        .ok_or_else(|| {
                            Diagnostic::at_source(
                                SourceSpan {
                                    span,
                                    ..location
                                },
                                "unknown partial record argument",
                            )
                        })?
                }
                None => {
                    if named {
                        return Err(Diagnostic::at_source(
                            SourceSpan {
                                span,
                                ..location
                            },
                            "positional record argument follows a named argument",
                        ));
                    }
                    let formal = remaining.get(positional).copied().ok_or_else(|| {
                        Diagnostic::at_source(
                            SourceSpan {
                                span,
                                ..location
                            },
                            "too many remaining record arguments",
                        )
                    })?;
                    positional += 1;
                    formal
                }
            };
            if baked[formal].is_some() || explicit[formal].replace(&argument.value).is_some() {
                return Err(Diagnostic::at_source(
                    SourceSpan {
                        span,
                        ..location
                    },
                    "record formal supplied more than once across partial application",
                ));
            }
        }
        parameters
            .iter()
            .enumerate()
            .map(|(ordinal, parameter)| {
                let formal = RecordFormalId {
                    declaration: self.declaration,
                    ordinal,
                };
                let value = if let Some(value) = baked[ordinal] {
                    CompletedRecordArgument::Baked(value)
                } else {
                    let default = match &parameter.binding {
                        syntax::RecordParameterBinding::Typed {
                            default, ..
                        } => default.as_ref(),
                        syntax::RecordParameterBinding::InferredDefault(default) => Some(default),
                    };
                    let expression = explicit[ordinal].or(default).ok_or_else(|| {
                        Diagnostic::at_source(location, "missing remaining record argument")
                    })?;
                    CompletedRecordArgument::Source {
                        expression,
                        defaulted: explicit[ordinal].is_none(),
                    }
                };
                Ok((formal, value))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remaining_positional_arguments_keep_the_original_record_formal_identity() {
        let path = std::path::Path::new("/partial-record/main.jai");
        let text = "TwoD::struct(M:int,N:int){values:[M*N]int;} main::(){}";
        let mut overlay = jai_modules::SourceOverlay::new();
        overlay.insert(path, text.as_bytes().to_vec()).unwrap();
        let graph =
            ModuleGraph::load_with_provider(path, jai_modules::GraphOptions::default(), &overlay)
                .unwrap();
        let declaration = &graph.declarations()[0];
        let syntax::FileDeclarationKind::Record(record) = &declaration.syntax().kind else {
            panic!("fixture has a real original record");
        };
        let types = TypeRegistry::new();
        let alias = PartialRecordApplication {
            declaration: declaration.id(),
            file: declaration.file(),
            location: declaration.location(),
            arguments: vec![PartialRecordArgument {
                formal: RecordFormalId {
                    declaration: declaration.id(),
                    ordinal: 0,
                },
                value: BakedValue::integer(
                    jai_types::Integer::wrapping(jai_types::IntegerType::S64, 5),
                    &types,
                ),
                location: declaration.location(),
            }],
        };
        let supplied = vec![syntax::CallArgument {
            name: None,
            spread: false,
            value: syntax::Expression {
                span: declaration.location().span,
                kind: syntax::ExpressionKind::Integer(3),
            },
        }];
        let complete = alias
            .complete_arguments(&record.parameters, &supplied, declaration.location())
            .unwrap();
        assert_eq!(
            complete
                .iter()
                .map(|(formal, _)| formal.ordinal)
                .collect::<Vec<_>>(),
            [0, 1]
        );
        assert!(matches!(complete[0].1, CompletedRecordArgument::Baked(_)));
        assert!(matches!(
            complete[1].1,
            CompletedRecordArgument::Source {
                defaulted: false,
                ..
            }
        ));
        assert_eq!(record.parameters.len(), 2);
    }
}
