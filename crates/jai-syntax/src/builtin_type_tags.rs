//! Contextual compiler type tags do not reserve ordinary identifier spellings.
use super::*;

impl Parser<'_> {
    /// Called only immediately after #type, after distinct/isa modifiers.
    pub(super) fn compiler_type_tag(&mut self) -> Option<BuiltinType> {
        if self.token().kind != Kind::Ident
            || self.token().spelling(self.source) != "any"
            || self
                .tokens
                .get(self.at + 1)
                .is_some_and(|token| token.kind == Kind::Punctuation(Punct::Dot))
        {
            return None;
        }
        self.at += 1;
        Some(BuiltinType::Any)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::{SourceMap, Symbols};

    #[test]
    fn lower_case_tag_requires_explicit_compiler_type_context() {
        let mut sources = SourceMap::default();
        let id = sources.insert(
            "tags.jai".into(),
            "Universal::#type any; any::struct{value:int;} named:any;".to_owned(),
        );
        let mut symbols = Symbols::default();
        let file = parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
        let [
            FileItem::Declaration(alias),
            FileItem::Declaration(_),
            FileItem::Declaration(global),
        ] = file.items()
        else {
            panic!("original declarations expected");
        };
        assert!(matches!(
            &alias.kind,
            FileDeclarationKind::TypeAlias(TypeAliasDeclaration {
                ty: TypeSyntax::Builtin(BuiltinType::Any),
                ..
            })
        ));
        let FileDeclarationKind::Global(GlobalDeclaration {
            declaration:
                Declaration::UnresolvedExplicit {
                    ty: TypeSyntax::Named(path),
                    ..
                },
            ..
        }) = &global.kind
        else {
            panic!("ordinary lower-case any must remain a named type");
        };
        assert_eq!(symbols.name(path.root), "any");
        assert_eq!(BuiltinType::from_spelling("any"), None);
    }

    #[test]
    fn variants_and_qualified_aliases_keep_their_existing_name_rules() {
        let mut sources = SourceMap::default();
        let id = sources.insert(
            "variants.jai".into(),
            "Unit::#type,distinct any; Alias::#type any.Member; Ordinary::#type Something;"
                .to_owned(),
        );
        let mut symbols = Symbols::default();
        let file = parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
        let [
            FileItem::Declaration(unit),
            FileItem::Declaration(alias),
            FileItem::Declaration(ordinary),
        ] = file.items()
        else {
            panic!("aliases expected");
        };
        let FileDeclarationKind::TypeAlias(TypeAliasDeclaration {
            ty: TypeSyntax::Variant { base, .. },
            ..
        }) = &unit.kind
        else {
            panic!("distinct alias expected");
        };
        assert!(matches!(base.as_ref(),TypeSyntax::Named(path) if symbols.name(path.root)=="any"));
        assert!(
            matches!(&alias.kind, FileDeclarationKind::TypeAlias(TypeAliasDeclaration{ty:TypeSyntax::Named(path), ..}) if path.members.len()==1 && symbols.name(path.root)=="any")
        );
        assert!(
            matches!(&ordinary.kind, FileDeclarationKind::TypeAlias(TypeAliasDeclaration{ty:TypeSyntax::Named(path), ..}) if symbols.name(path.root)=="Something")
        );
    }
}
