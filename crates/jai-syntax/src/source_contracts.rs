//! Bare external source stubs can retain an ordinary Jai packed signature.
use crate::{Parameter, ParameterBaking, ParameterBinding, ParameterEvaluation};

pub(crate) fn packed_defaults(parameters: &[Parameter]) -> bool {
    let Some(index) = parameters.iter().position(|parameter| parameter.variadic) else {
        return false;
    };
    parameters[index].baking == ParameterBaking::None
        && parameters[index].evaluation == ParameterEvaluation::Evaluate
        && matches!(
            parameters[index].binding,
            ParameterBinding::Required(_) | ParameterBinding::RequiredType(_)
        )
        && index + 1 < parameters.len()
        && parameters[index + 1..].iter().all(|parameter| {
            !parameter.variadic
                && matches!(
                    parameter.binding,
                    ParameterBinding::Defaulted { .. } | ParameterBinding::DefaultedType { .. }
                )
        })
}
