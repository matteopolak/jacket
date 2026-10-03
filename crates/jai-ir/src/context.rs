//! One typed context schema shared by all implicit-context procedure signatures.
use crate::ConstantValue;
use jai_types::TypeId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextDefinition {
    pub record_type: TypeId,
    pub pointer_type: TypeId,
    pub default: ConstantValue,
}
