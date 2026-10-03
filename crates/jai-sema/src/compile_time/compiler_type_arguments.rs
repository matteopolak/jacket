//! Private source-domain preparation for compiler-only Type argument metadata.
//!
//! Static Type expressions avoid descriptor interning solely to pass an API
//! operand. Dynamic Type values require their real VM descriptor receipt.
use crate::Expr;
use jai_ir::SourceProcedureIdentity;
use jai_source::{Diagnostic, SourceRecord, SourceSpan};
use jai_types::{TypeId, TypeView};

#[derive(Clone)]
pub(crate) struct StaticCompilerTypeArgument {
    ty: TypeId,
    source: SourceProcedureIdentity,
}
impl StaticCompilerTypeArgument {
    pub(crate) fn ty(&self) -> TypeId {
        self.ty
    }
    pub(crate) fn source(&self) -> &SourceProcedureIdentity {
        &self.source
    }
    pub(crate) fn validate(&self, types: &dyn TypeView) -> Result<(), jai_types::TypeError> {
        types.kind(self.ty).map(|_| ())
    }
}

/// Only the already-bound semantic Type case enters this fast path. A computed
/// runtime Type returns None and remains in the descriptor-backed dynamic lane.
pub(crate) fn static_type_argument(
    expression: &Expr,
    types: &dyn TypeView,
    source: &SourceRecord,
    location: SourceSpan,
) -> Result<Option<StaticCompilerTypeArgument>, Diagnostic> {
    let Expr::Type(ty) = expression else {
        return Ok(None);
    };
    types
        .kind(*ty)
        .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
    let identity = SourceProcedureIdentity::new(source, location)
        .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
    Ok(Some(StaticCompilerTypeArgument {
        ty: *ty,
        source: identity,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::{SourceMap, Span};
    use jai_types::{RecordKind, TypeRegistry};
    #[test]
    fn static_type_metadata_retains_actual_source_without_interning_a_descriptor() {
        let mut types = TypeRegistry::new();
        let ty = types.reserve_record(RecordKind::Struct);
        types.define_record(ty, []).unwrap();
        let mut sources = SourceMap::default();
        let source = sources.insert("flags.jai".into(), "Record".into());
        let record = sources.get(source).unwrap();
        let location = SourceSpan {
            source,
            span: Span::new(0, 6),
        };
        let argument = static_type_argument(&Expr::Type(ty), &types, record, location)
            .unwrap()
            .unwrap();
        assert_eq!(argument.ty(), ty);
        assert!(argument.source().matches_source(record, location));
        assert!(types.runtime_type_header().is_none());
        argument.validate(&types).unwrap();
        assert!(argument.validate(&TypeRegistry::new()).is_err());
    }
    #[test]
    fn runtime_values_do_not_become_static_type_metadata() {
        let types = TypeRegistry::new();
        let mut sources = SourceMap::default();
        let source = sources.insert("flags.jai".into(), "true".into());
        assert!(
            static_type_argument(
                &Expr::Bool(jai_ir::BoolExpr::Constant(true)),
                &types,
                sources.get(source).unwrap(),
                SourceSpan {
                    source,
                    span: Span::new(0, 4)
                }
            )
            .unwrap()
            .is_none()
        );
    }
}
