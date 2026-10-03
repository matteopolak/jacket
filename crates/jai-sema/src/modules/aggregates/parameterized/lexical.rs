//! Caller arguments retain lexical compile-time values until a parameter supplies a type.
use super::*;
use jai_source::Symbol;

#[derive(Clone)]
pub(crate) enum LexicalTypeArgument {
    Type(TypeId),
    Scalar(ScalarConstant),
    Typed(jai_ir::ConstantValue),
    Code(jai_types::CodeValueId),
}
#[derive(Clone, Default)]
pub(crate) struct LexicalTypeArguments {
    pub(crate) roots: HashMap<Symbol, LexicalTypeArgument>,
    pub(crate) namespaces: HashMap<TypeId, HashMap<Symbol, LexicalTypeArgument>>,
    pub(crate) codes: HashMap<(usize, usize), jai_types::CodeValueId>,
    pub(crate) types: HashMap<(usize, usize), TypeId>,
    pub(crate) values: HashMap<(usize, usize), jai_ir::ConstantValue>,
    pub(crate) bindings: HashMap<(usize, usize), LexicalTypeArgument>,
    pub(crate) templates: HashMap<(usize, usize), DeclarationId>,
    pub(crate) conversions: HashMap<TypeId, Vec<(jai_types::FieldId, TypeId)>>,
}
impl LexicalTypeArguments {
    pub(super) fn lookup(&self, path: &syntax::NamePath) -> Option<&LexicalTypeArgument> {
        let mut binding = self.roots.get(&path.root)?;
        for member in &path.members {
            let LexicalTypeArgument::Type(ty) = binding else {
                return None;
            };
            binding = self.namespaces.get(ty)?.get(member)?;
        }
        Some(binding)
    }
}
pub(super) fn lexical_scalar(value: &LexicalTypeArgument) -> Option<ScalarConstant> {
    match value {
        LexicalTypeArgument::Scalar(value) => Some(value.clone()),
        LexicalTypeArgument::Typed(value) => match &value.kind {
            jai_ir::ConstantKind::Int(value) | jai_ir::ConstantKind::Enum(value) => {
                Some(ScalarConstant::Int(*value))
            }
            jai_ir::ConstantKind::Bool(value) => Some(ScalarConstant::Bool(*value)),
            jai_ir::ConstantKind::Float(value) => Some(ScalarConstant::Float(*value)),
            _ => None,
        },
        _ => None,
    }
}
