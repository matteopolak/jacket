//! Declare native data symbols without inventing owned initial storage.
use inkwell::{module::Module, types::BasicTypeEnum, values::GlobalValue};
use jai_ir::{ExternalData, GlobalId};
use std::{collections::HashMap, fmt};

#[derive(Debug)]
pub enum Error {
    SymbolConflict {
        symbol: String,
        first: Option<GlobalId>,
        second: GlobalId,
    },
}
impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SymbolConflict {
                symbol,
                first,
                second,
            } => write!(
                formatter,
                "external data symbol {symbol:?} has incompatible checked bindings ({first:?}, {second:?})"
            ),
        }
    }
}
impl std::error::Error for Error {}

struct Binding<'a, 'ctx> {
    declaration: &'a ExternalData,
    global: GlobalId,
    value: GlobalValue<'ctx>,
    alignment: u32,
}

#[derive(Default)]
pub(super) struct Declarations<'a, 'ctx> {
    bindings: HashMap<&'a str, Binding<'a, 'ctx>>,
}
impl<'a, 'ctx> Declarations<'a, 'ctx> {
    /// Owned storage must not steal or rename a symbol declared by an earlier extern.
    pub(super) fn check_owned_symbol(
        &self,
        module: &Module<'ctx>,
        global: GlobalId,
        symbol: &str,
    ) -> Result<(), Error> {
        if module.get_function(symbol).is_some() || module.get_global(symbol).is_some() {
            return Err(Error::SymbolConflict {
                symbol: symbol.into(),
                first: self.bindings.get(symbol).map(|binding| binding.global),
                second: global,
            });
        }
        Ok(())
    }
    pub(super) fn declare(
        &mut self,
        module: &Module<'ctx>,
        global: GlobalId,
        declaration: &'a ExternalData,
        ty: BasicTypeEnum<'ctx>,
        alignment: u32,
    ) -> Result<GlobalValue<'ctx>, Error> {
        let symbol = declaration.symbol();
        if let Some(existing) = self.bindings.get(symbol) {
            if existing.declaration.ty() != declaration.ty()
                || existing.declaration.source() != declaration.source()
                || existing.alignment != alignment
            {
                return Err(Error::SymbolConflict {
                    symbol: symbol.into(),
                    first: Some(existing.global),
                    second: global,
                });
            }
            return Ok(existing.value);
        }
        // LLVM otherwise silently gives a conflicting declaration a new symbol.
        if module.get_function(symbol).is_some() || module.get_global(symbol).is_some() {
            return Err(Error::SymbolConflict {
                symbol: symbol.into(),
                first: None,
                second: global,
            });
        }
        let value = module.add_global(ty, None, symbol);
        value.set_alignment(alignment);
        self.bindings.insert(
            symbol,
            Binding {
                declaration,
                global,
                value,
                alignment,
            },
        );
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_ir::ExternalDataSource;
    use jai_source::{Identities, SourceMap, SourceSpan, Span};
    use jai_types::{IntegerType, ScalarType, TypeRegistry};

    #[test]
    fn native_declarations_alias_only_matching_checked_storage() {
        let context = inkwell::context::Context::create();
        let module = context.create_module("authored_external_data");
        let types = TypeRegistry::new();
        let mut ids = Identities::default();
        let mut sources = SourceMap::default();
        let source = sources.insert("owned.jai".into(), "counter:s64 #elsewhere;".into());
        let location = SourceSpan {
            source,
            span: Span { start: 0, end: 22 },
        };
        let first = ExternalData::new(
            jai_ir::ExternalDataId::File(ids.declaration()),
            types.scalar(ScalarType::Int(IntegerType::S64)),
            ExternalDataSource::Program,
            "counter".into(),
            location,
            &types,
        )
        .unwrap();
        let alias = ExternalData::new(
            jai_ir::ExternalDataId::File(ids.declaration()),
            first.ty(),
            ExternalDataSource::Program,
            "counter".into(),
            location,
            &types,
        )
        .unwrap();
        let incompatible = ExternalData::new(
            jai_ir::ExternalDataId::File(ids.declaration()),
            types.scalar(ScalarType::Int(IntegerType::S32)),
            ExternalDataSource::Program,
            "counter".into(),
            location,
            &types,
        )
        .unwrap();
        let another_provider = ExternalData::new(
            jai_ir::ExternalDataId::File(ids.declaration()),
            first.ty(),
            ExternalDataSource::Library(jai_ir::ForeignLibrary {
                id: jai_ir::ForeignLibraryId::File(ids.declaration()),
                kind: jai_ir::ForeignLibraryKind::System {
                    name: "owned_provider".into(),
                },
                options: Default::default(),
            }),
            "counter".into(),
            location,
            &types,
        )
        .unwrap();
        let mut declarations = Declarations::default();
        let value = declarations
            .declare(
                &module,
                GlobalId::new(0),
                &first,
                context.i64_type().into(),
                8,
            )
            .unwrap();
        assert!(value.get_initializer().is_none());
        assert_eq!(value.get_alignment(), 8);
        assert_eq!(
            declarations
                .declare(
                    &module,
                    GlobalId::new(1),
                    &alias,
                    context.i64_type().into(),
                    8
                )
                .unwrap(),
            value
        );
        assert!(matches!(
            declarations.declare(
                &module,
                GlobalId::new(2),
                &incompatible,
                context.i32_type().into(),
                4
            ),
            Err(Error::SymbolConflict { .. })
        ));
        assert!(matches!(
            declarations.declare(
                &module,
                GlobalId::new(3),
                &another_provider,
                context.i64_type().into(),
                8
            ),
            Err(Error::SymbolConflict { .. })
        ));
        assert_eq!(module.get_globals().count(), 1);
        assert!(
            matches!(declarations.check_owned_symbol(&module, GlobalId::new(4), "counter"),
            Err(Error::SymbolConflict { first: Some(id), .. }) if id == GlobalId::new(0))
        );
        module.verify().unwrap();
    }

    #[test]
    fn data_cannot_silently_rename_an_existing_function_symbol() {
        let context = inkwell::context::Context::create();
        let module = context.create_module("authored_symbol_collision");
        module.add_function("collision", context.void_type().fn_type(&[], false), None);
        let types = TypeRegistry::new();
        let mut ids = Identities::default();
        let mut sources = SourceMap::default();
        let source = sources.insert("owned.jai".into(), "collision:s64 #elsewhere;".into());
        let data = ExternalData::new(
            jai_ir::ExternalDataId::File(ids.declaration()),
            types.scalar(ScalarType::Int(IntegerType::S64)),
            ExternalDataSource::Program,
            "collision".into(),
            SourceSpan {
                source,
                span: Span { start: 0, end: 24 },
            },
            &types,
        )
        .unwrap();
        assert!(matches!(
            Declarations::default().declare(
                &module,
                GlobalId::new(0),
                &data,
                context.i64_type().into(),
                8
            ),
            Err(Error::SymbolConflict { first: None, .. })
        ));
        assert!(module.get_global("collision").is_none());
        module.verify().unwrap();
    }
}
