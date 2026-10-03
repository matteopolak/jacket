//! Source-only Code slots are consumed before the checked runtime ABI is built.
use super::*;

#[derive(Clone)]
pub(crate) struct SourceCompilerSignature {
    parameters: Vec<ParameterSignature>,
    code_slot: usize,
}

pub(in crate::modules) fn lower_code_signature(
    syntax: &FileDeclarationKind,
    parameters: &mut Vec<ParameterSignature>,
    symbols: &Symbols,
    types: &TypeRegistry,
    span: Span,
) -> Result<Option<SourceCompilerSignature>, Diagnostic> {
    if !parameters
        .iter()
        .any(|parameter| parameter.ty == types.code_type())
    {
        return Ok(None);
    }
    let (name, mark) = match syntax {
        FileDeclarationKind::ProcedurePrototype(prototype) => match &prototype.binding {
            syntax::PrototypeBinding::Compiler(mark) => (prototype.name, mark),
            _ => return Ok(None),
        },
        // A runtime fallback cannot receive a Code value in storage.
        _ => return Ok(None),
    };
    let name = mark.tag.as_deref().unwrap_or_else(|| symbols.name(name));
    if SourceIntrinsic::parse(name) != Some(SourceIntrinsic::AddString) {
        return Err(Diagnostic::new(
            span,
            format!("#compiler `{name}` does not support a compile-only Code parameter"),
        ));
    }
    if parameters.len() != 4
        || parameters[0].ty != types.string()
        || !matches!(
            types.kind(parameters[1].ty),
            Ok(TypeKind::Integer(IntegerType::S64))
        )
        || parameters[2].ty != types.code_type()
    {
        return Err(Diagnostic::new(
            span,
            "compiler Code signature requires (string, s64, Code, Source_Code_Location)",
        ));
    }
    let metadata = SourceCompilerSignature {
        parameters: parameters.clone(),
        code_slot: 2,
    };
    parameters.remove(metadata.code_slot);
    Ok(Some(metadata))
}

impl Resolver<'_> {
    pub(crate) fn bind_compiler_code_arguments(
        &mut self,
        signature: &Signature,
        args: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Option<(Call, crate::runtime_defaults::RuntimeDefaultBindings)>, Diagnostic> {
        let Some(metadata) = self
            .meta
            .compiler_source_signatures
            .get(&signature.id)
            .cloned()
        else {
            return Ok(None);
        };
        let descriptor = self
            .types
            .procedure_definition(signature.ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
            .clone();
        self.check_call_context(&descriptor, span)?;
        let mut supplied = vec![false; metadata.parameters.len()];
        let mut bound = Vec::new();
        let mut omitted_reads = HashMap::new();
        let mut positional = 0;
        let mut named = false;
        for argument in args {
            if argument.spread {
                return Err(Diagnostic::new(
                    argument.value.span,
                    "compiler Code calls do not accept spread arguments",
                ));
            }
            let index = if let Some(name) = argument.name {
                named = true;
                metadata
                    .parameters
                    .iter()
                    .position(|parameter| parameter.name == name)
                    .ok_or_else(|| Diagnostic::new(argument.value.span, "unknown named argument"))?
            } else {
                if named {
                    return Err(Diagnostic::new(
                        argument.value.span,
                        "positional argument cannot follow a named argument",
                    ));
                }
                let index = positional;
                positional += 1;
                index
            };
            let parameter = metadata
                .parameters
                .get(index)
                .ok_or_else(|| Diagnostic::new(argument.value.span, "too many arguments"))?;
            if supplied[index] {
                return Err(Diagnostic::new(
                    argument.value.span,
                    "duplicate argument for parameter",
                ));
            }
            supplied[index] = true;
            let value = self.expr_expected(&argument.value, parameter.ty)?;
            if index == metadata.code_slot {
                self.validate_compiler_root_code(value, argument.value.span)?;
            } else {
                let runtime = index - usize::from(index > metadata.code_slot);
                bound.push((
                    jai_ir::ParameterId::new(runtime),
                    self.coerce_value(value, parameter.ty, argument.value.span)?,
                ));
            }
        }
        for (index, parameter) in metadata.parameters.iter().enumerate() {
            if supplied[index] {
                continue;
            }
            let default = parameter.default.as_ref().ok_or_else(|| {
                Diagnostic::new(
                    span,
                    format!(
                        "missing required argument '{}'",
                        self.symbols.name(parameter.name)
                    ),
                )
            })?;
            if index == metadata.code_slot {
                let code = self.materialize_code_default(default, span)?;
                self.validate_compiler_root_code(code, span)?;
            } else {
                let runtime = index - usize::from(index > metadata.code_slot);
                if let Some(ParameterDefault::RuntimeRead(read)) = default.prepared() {
                    omitted_reads.insert(jai_ir::ParameterId::new(runtime), read.clone());
                }
                bound.push((
                    jai_ir::ParameterId::new(runtime),
                    self.materialize_parameter_default(default, parameter.ty, span)?,
                ));
            }
        }
        Ok(Some((Call::new(signature.id, bound), omitted_reads)))
    }

    fn validate_compiler_root_code(&self, code: Expr, span: Span) -> Result<(), Diagnostic> {
        let Expr::Code(id) = code else {
            return Err(Diagnostic::new(
                span,
                "compiler scope argument requires a checked Code value",
            ));
        };
        let code = self.meta.codes.get(id).ok_or_else(|| {
            Diagnostic::new(
                span,
                "compiler scope Code belongs to another semantic session",
            )
        })?;
        if code.scope().is_some() {
            return Err(Diagnostic::new(
                span,
                "add_build_string into a captured Code scope is not implemented; no source was staged",
            ));
        }
        Ok(())
    }
}

/// Overload matching sees source slots; the selected concrete call consumes Code separately.
pub(crate) fn source_candidate<Origin>(
    declaration: Origin,
    source: &SourceCompilerSignature,
) -> crate::overloads::Candidate<Origin> {
    use crate::overloads::{ArgumentInfo, Candidate, CandidateVariadic, Parameter, TypePattern};
    Candidate {
        declaration,
        result_type_parameters: Vec::new(),
        variadic: CandidateVariadic::None,
        parameters: source
            .parameters
            .iter()
            .map(|parameter| Parameter {
                name: parameter.name,
                evaluation: parameter.evaluation,
                ty: TypePattern::Concrete(parameter.ty),
                default: parameter.default.as_ref().map(|default| match default {
                    ParameterDefault::Source(_) => ArgumentInfo::typed(parameter.ty),
                    ParameterDefault::Discarded => ArgumentInfo::typed(parameter.ty),
                    ParameterDefault::RuntimeRead(_) => ArgumentInfo::typed(parameter.ty),
                    ParameterDefault::Constant(value) => ArgumentInfo::constant(
                        crate::polymorphism::BakedValue::Value(value.clone()),
                        value.ty,
                    ),
                    ParameterDefault::CallerLocation => ArgumentInfo::caller_location(parameter.ty),
                    ParameterDefault::CodeNull {
                        ty,
                    } => ArgumentInfo::code_null(*ty),
                }),
                baking: syntax::ParameterBaking::None,
            })
            .collect(),
    }
}
