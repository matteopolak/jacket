//! Alias expansion retains the exact producer environment at each callback node.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ContractStep {
    Element,
    Parameter(usize),
    Result(usize),
    Argument(usize),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ContractOwner {
    File,
    Procedure(ProcedureId),
}
#[derive(Clone)]
pub(crate) struct ContractOrigin {
    pub(crate) file: Option<jai_modules::FileInstanceId>,
    pub(crate) source: Option<jai_source::SourceId>,
    pub(crate) owner: ContractOwner,
    pub(crate) lexical_scopes: Vec<crate::local_declarations::LexicalScopeId>,
    pub(crate) target: Option<jai_types::LayoutPolicy>,
    pub(crate) substitution: Option<crate::polymorphism::Substitution>,
}
#[derive(Clone)]
pub(crate) struct CallbackSyntaxOrigin {
    pub(crate) environment: ContractOrigin,
    pub(crate) original: syntax::ProcedureTypeSyntax,
    pub(crate) proof: Option<
        std::sync::Arc<crate::procedure_values::source_annotations::CheckedSourceProcedureType>,
    >,
}
#[derive(Clone)]
pub(crate) struct ContractSyntax {
    pub(crate) syntax: syntax::TypeSyntax,
    pub(crate) origin: ContractOrigin,
    pub(crate) callback: Option<CallbackSyntaxOrigin>,
    children: HashMap<ContractStep, ContractSyntax>,
}
impl ContractSyntax {
    pub(crate) fn child(&self, step: ContractStep) -> &Self {
        self.children.get(&step).expect("normalized contract child")
    }
    pub(crate) fn resolve(
        source: &syntax::TypeSyntax,
        origin: ContractOrigin,
        mut resolve: impl FnMut(&syntax::TypeSyntax) -> Result<Self, Diagnostic>,
    ) -> Result<Self, Diagnostic> {
        let mut children = HashMap::new();
        let mut child = |step, source: &syntax::TypeSyntax| {
            let child = resolve(source)?;
            let syntax = child.syntax.clone();
            children.insert(step, child);
            Ok::<_, Diagnostic>(syntax)
        };
        let mut callback = None;
        let syntax = match source {
            syntax::TypeSyntax::Pointer(inner) => {
                syntax::TypeSyntax::Pointer(Box::new(child(ContractStep::Element, inner)?))
            }
            syntax::TypeSyntax::Slice(inner) => {
                syntax::TypeSyntax::Slice(Box::new(child(ContractStep::Element, inner)?))
            }
            syntax::TypeSyntax::DynamicArray(inner) => {
                syntax::TypeSyntax::DynamicArray(Box::new(child(ContractStep::Element, inner)?))
            }
            syntax::TypeSyntax::FixedArray { element, count } => syntax::TypeSyntax::FixedArray {
                element: Box::new(child(ContractStep::Element, element)?),
                count: count.clone(),
            },
            syntax::TypeSyntax::Procedure(signature) => {
                callback = Some(CallbackSyntaxOrigin {
                    environment: origin.clone(),
                    original: signature.clone(),
                    proof: None,
                });
                let mut signature = signature.clone();
                for (index, parameter) in signature.parameters.iter_mut().enumerate() {
                    parameter.ty = child(ContractStep::Parameter(index), &parameter.ty)?;
                }
                for (index, result) in signature.results.iter_mut().enumerate() {
                    result.ty = child(ContractStep::Result(index), &result.ty)?;
                }
                syntax::TypeSyntax::Procedure(signature)
            }
            syntax::TypeSyntax::Application(application) => {
                let mut application = application.clone();
                for (index, argument) in application.arguments.iter_mut().enumerate() {
                    if let Some(source) = callback_source_type(&argument.value) {
                        argument.value.kind = syntax::ExpressionKind::Type(child(
                            ContractStep::Argument(index),
                            &source,
                        )?);
                    }
                }
                syntax::TypeSyntax::Application(application)
            }
            other => other.clone(),
        };
        Ok(Self {
            syntax,
            origin,
            callback,
            children,
        })
    }
}
