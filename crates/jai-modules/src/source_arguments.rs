//! Canonical, contextualized generic arguments independent of a runtime registry.
use super::{EnumParameter, ModuleType};
use jai_types::{FloatValue, Integer};
use std::hash::{Hash, Hasher};

/// Semantic specializations order arguments by the declaration's formal list.
/// Weak literals, contextual members, runtime addresses, and registry IDs cannot
/// occur here: the semantic binder must normalize them before responding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModuleBoundArgument {
    Type(ModuleType),
    Integer(Integer),
    Boolean(bool),
    Float(FloatValue),
    Enumeration(EnumParameter),
    String(Box<[u8]>),
}
impl Hash for ModuleBoundArgument {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Type(value) => value.hash(state),
            Self::Integer(value) => {
                value.ty().hash(state);
                value.bits().hash(state);
            }
            Self::Boolean(value) => value.hash(state),
            Self::Float(value) => {
                value.ty().hash(state);
                value.bits().hash(state);
            }
            Self::Enumeration(value) => {
                value.declaration.hash(state);
                value.value.ty().hash(state);
                value.value.bits().hash(state);
            }
            Self::String(value) => value.hash(state),
        }
    }
}

impl super::loader::Builder<'_> {
    pub(super) fn specialization_argument(
        &self,
        file: super::FileInstanceId,
        span: jai_source::Span,
        name: jai_source::Symbol,
    ) -> Option<&ModuleBoundArgument> {
        self.specialization_for_span(file, span)?
            .arguments()
            .iter()
            .find(|(formal, _)| *formal == name)
            .map(|(_, value)| value)
    }
    pub(super) fn specialization_argument_value(
        &self,
        file: super::FileInstanceId,
        expression: &jai_syntax::Expression,
    ) -> Result<Option<super::ParameterValue>, super::GraphError> {
        let name = match expression.kind {
            jai_syntax::ExpressionKind::Name(name)
            | jai_syntax::ExpressionKind::CompileVariable(name) => name,
            _ => return Ok(None),
        };
        let Some(argument) = self.specialization_argument(file, expression.span, name) else {
            return self.insertion_argument_value(file, name, expression.span);
        };
        use super::ParameterValue as V;
        Ok(Some(match argument {
            ModuleBoundArgument::Type(ty) => V::Type(ty.clone()),
            ModuleBoundArgument::Integer(value) => V::Scalar(jai_eval::Value::Int(*value)),
            ModuleBoundArgument::Boolean(value) => V::Scalar(jai_eval::Value::Bool(*value)),
            ModuleBoundArgument::Float(value) => V::Scalar(jai_eval::Value::Float(*value)),
            ModuleBoundArgument::Enumeration(value) => V::Enumeration(*value),
            ModuleBoundArgument::String(value) => {
                V::String(String::from_utf8(value.to_vec()).map_err(|_| {
                    self.located(
                        jai_source::SourceSpan {
                            source: self.graph.files[file.index()].source,
                            span: expression.span,
                        },
                        "module string parameter must contain UTF-8",
                    )
                })?)
            }
        }))
    }
    pub(super) fn insertion_argument_value(
        &self,
        file: super::FileInstanceId,
        name: jai_source::Symbol,
        span: jai_source::Span,
    ) -> Result<Option<super::ParameterValue>, super::GraphError> {
        use super::{ParameterValue as V, SourceCaptureValue as C};
        let Some(value) = self.graph.insertion_capture_value(file, name) else {
            return Ok(None);
        };
        Ok(Some(match value {
            C::Scalar(value) => V::Scalar(value.clone()),
            C::Type(value) => V::Type(value.clone()),
            C::Enumeration(value) => V::Enumeration(*value),
            C::String(value) => {
                V::String(String::from_utf8(value.to_vec()).map_err(|_| {
                    self.located(jai_source::SourceSpan {
                    source: self.graph.files[file.index()].source,
                    span,
                }, "module string parameter byte transport requires the byte-string binding phase")
                })?)
            }
        }))
    }
}
