//! Named and anonymous source procedures parse the same checked header syntax.
use super::*;

impl Parser<'_> {
    pub(super) fn source_procedure_parameters(&mut self) -> Result<Vec<Parameter>, Diagnostic> {
        self.need(Punct::OpenParen)?;
        let mut parameters = Vec::new();
        if !self.take(Punct::CloseParen) {
            loop {
                let parameter_start = self.token().span.start;
                let evaluation = self.parameter_evaluation(false)?;
                let using = self.keyword(Keyword::Using);
                if using && !self.allow_qualified {
                    return Err(self.error("using parameters require type and scope resolution"));
                }
                let baking = self.parameter_baking()?;
                let name = self.name()?;
                let mut variadic = false;
                let binding = if self.take(Punct::Infer) {
                    ParameterBinding::Defaulted {
                        ty: None,
                        expression: self.expression(0)?,
                    }
                } else {
                    self.need(Punct::Colon)?;
                    variadic = self.take(Punct::Range);
                    if variadic && (using || baking != ParameterBaking::None) {
                        return Err(self.error("variadic parameters cannot be using or baked"));
                    }
                    let ty = self.signature_type()?;
                    if self.take(Punct::Assign) {
                        if variadic {
                            return Err(self.error("variadic parameters cannot have defaults"));
                        }
                        let expression = self.expression(0)?;
                        if let Some(ty) = ty.as_scalar() {
                            ParameterBinding::Defaulted {
                                ty: Some(ty),
                                expression,
                            }
                        } else {
                            ParameterBinding::DefaultedType {
                                ty: Some(ty),
                                expression,
                            }
                        }
                    } else if let Some(ty) = ty.as_scalar() {
                        ParameterBinding::Required(ty)
                    } else {
                        ParameterBinding::RequiredType(ty)
                    }
                };
                parameters.push(Parameter {
                    evaluation,
                    name,
                    binding,
                    using,
                    baking,
                    variadic,
                    span: Span::new(parameter_start, self.tokens[self.at - 1].span.end),
                });
                if self.take(Punct::CloseParen) {
                    break;
                }
                self.need(Punct::Comma)?;
                if self.take(Punct::CloseParen) {
                    break;
                }
            }
        }
        Ok(parameters)
    }
    pub(super) fn source_procedure_header(
        &mut self,
        operator: bool,
    ) -> Result<(SourceProcedureHeader, bool, usize), Diagnostic> {
        let inline_hint = self.procedure_inline_hint()?;
        let parameters = self.source_procedure_parameters()?;
        let results = self.procedure_results()?;
        if parameters
            .iter()
            .filter(|parameter| parameter.variadic)
            .count()
            > 1
        {
            return Err(self.error("a procedure can have only one variadic parameter"));
        }
        let modifier_start = self.at;
        let modifiers = self.procedure_modifiers(operator)?;
        Ok((
            SourceProcedureHeader {
                callable: CallableHeaderSyntax {
                    deprecation: modifiers.deprecation,
                    notes: Vec::new(),
                    parameters,
                    results,
                    convention: modifiers.convention,
                    return_abi: modifiers.return_abi,
                    context: modifiers.context,
                },
                checks: modifiers.checks,
                inline_hint,
                execution: modifiers.execution,
                debug: modifiers.debug,
                compiler: None,
                expands: modifiers.expands,
                modify: None,
            },
            modifiers.symmetric,
            modifier_start,
        ))
    }
}
