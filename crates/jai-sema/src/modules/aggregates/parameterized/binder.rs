//! Associate arguments with parameters without changing evaluation or declaration order.
use jai_source::{Diagnostic, Span, Symbol};
use jai_syntax::{CallArgument, Expression, RecordParameter, RecordParameterBinding};
use std::collections::HashSet;

pub(crate) struct BoundArgument<'a> {
    pub(crate) parameter: &'a RecordParameter,
    pub(crate) expression: &'a Expression,
    pub(crate) defaulted: bool,
}

pub(crate) fn bind_arguments<'a>(
    parameters: &'a [RecordParameter],
    arguments: &'a [CallArgument],
    span: Span,
) -> Result<Vec<BoundArgument<'a>>, Diagnostic> {
    let mut names = HashSet::<Symbol>::new();
    for parameter in parameters {
        if !names.insert(parameter.name) {
            return Err(Diagnostic::new(
                parameter.span,
                "duplicate record template parameter",
            ));
        }
    }
    let mut slots = vec![None; parameters.len()];
    let mut positional = 0;
    let mut named = false;
    for argument in arguments {
        if argument.spread {
            return Err(Diagnostic::new(
                argument.value.span,
                "record template arguments cannot spread runtime sequences",
            ));
        }
        let index = match argument.name {
            Some(name) => {
                named = true;
                parameters
                    .iter()
                    .position(|parameter| parameter.name == name)
                    .ok_or_else(|| {
                        Diagnostic::new(argument.value.span, "unknown record template argument")
                    })?
            }
            None => {
                if named {
                    return Err(Diagnostic::new(
                        argument.value.span,
                        "positional record template argument follows a named argument",
                    ));
                }
                let index = positional;
                positional += 1;
                if index >= parameters.len() {
                    return Err(Diagnostic::new(
                        argument.value.span,
                        "too many record template arguments",
                    ));
                }
                index
            }
        };
        if slots[index].replace(&argument.value).is_some() {
            return Err(Diagnostic::new(
                argument.value.span,
                "record template parameter supplied more than once",
            ));
        }
    }
    parameters
        .iter()
        .zip(slots)
        .map(|(parameter, explicit)| {
            let default = match &parameter.binding {
                RecordParameterBinding::Typed { default, .. } => default.as_ref(),
                RecordParameterBinding::InferredDefault(expression) => Some(expression),
            };
            let expression = explicit.or(default).ok_or_else(|| {
                Diagnostic::new(span, "missing required record template argument")
            })?;
            Ok(BoundArgument {
                parameter,
                expression,
                defaulted: explicit.is_none(),
            })
        })
        .collect()
}
